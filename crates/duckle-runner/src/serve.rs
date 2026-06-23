//! The `serve` subcommand: a lightweight web management console for running
//! and monitoring Duckle pipelines on a server, with no desktop app.
//!
//! It hosts a small self-contained web panel (embedded HTML, no Node, no extra
//! binary) backed by a tiny std-only HTTP server, so the whole console ships
//! inside the runner you already deploy. The panel has three views:
//!   - Operations: run history across all pipelines (status, duration, rows,
//!     errors) plus per-pipeline run logs.
//!   - Pipelines:  every pipeline in the workspace with its last status and an
//!     editable interval schedule.
//!   - Run:        trigger any pipeline on demand and see the result.
//!
//! Runs execute in-process through the same engine as `duckle-runner run`, are
//! serialized by a single lock (so a manual run and a scheduled run never
//! collide on the shared workspace env), and append the same run history
//! (`<workspace>/runs/<id>.json`) and NDJSON logs (`<workspace>/logs/<id>/`)
//! the desktop and runner already write. A background scheduler triggers any
//! pipeline whose interval has elapsed. No authentication: bind it to a
//! trusted network or localhost.

use duckle_connectors::CsvConnector;
use duckle_duckdb_engine::{
    append_run_record, compile_pipeline_sql, load_run_history, DuckdbEngine, PipelineDoc,
    PipelineEvent, RunRecord,
};
use duckle_plugin_sdk::{InspectError, SchemaInspector};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const PANEL_HTML: &str = include_str!("panel.html");

struct ServeArgs {
    host: String,
    port: u16,
    workspace: PathBuf,
    duckdb: Option<PathBuf>,
}

fn parse_serve_args() -> Result<ServeArgs, String> {
    let mut host = "127.0.0.1".to_string();
    let mut port: u16 = 8080;
    let mut workspace: Option<PathBuf> = None;
    let mut duckdb: Option<PathBuf> = None;
    let mut it = std::env::args().skip(2);
    while let Some(arg) = it.next() {
        let mut take = |label: &str| it.next().ok_or_else(|| format!("{} needs a value", label));
        match arg.as_str() {
            "--host" => host = take("--host")?,
            "--port" => {
                port = take("--port")?
                    .parse()
                    .map_err(|_| "--port must be a number".to_string())?
            }
            "--workspace" => workspace = Some(PathBuf::from(take("--workspace")?)),
            "--duckdb" => duckdb = Some(PathBuf::from(take("--duckdb")?)),
            "-h" | "--help" => {
                println!(
                    "duckle-runner serve - web management console\n\n\
                     USAGE:\n    duckle-runner serve [--host <ip>] [--port <n>] [--workspace <dir>] [--duckdb <path>]\n\n\
                     OPTIONS:\n    \
                     --host <ip>        Bind address (default 127.0.0.1; use 0.0.0.0 for remote access)\n    \
                     --port <n>         Port (default 8080)\n    \
                     --workspace <dir>  Workspace root holding pipelines, runs/, logs/ (default: current dir)\n    \
                     --duckdb <path>    DuckDB CLI (default: DUCKLE_DUCKDB_BIN, sibling bin/duckdb, or PATH)\n\n\
                     No authentication. Bind to localhost or a trusted network."
                );
                std::process::exit(0);
            }
            other => return Err(format!("unknown serve argument: {}", other)),
        }
    }
    let workspace = workspace.unwrap_or_else(|| PathBuf::from("."));
    Ok(ServeArgs {
        host,
        port,
        workspace,
        duckdb,
    })
}

struct State {
    workspace: PathBuf,
    duckdb: PathBuf,
    /// Serializes pipeline execution: the shared workspace env vars and DuckDB
    /// process make concurrent runs unsafe, so manual + scheduled runs queue.
    run_lock: Mutex<()>,
    /// Active browser-studio run. Used by `/api/studio/cancel`.
    current_run: Mutex<Option<DuckdbEngine>>,
}

pub fn run() -> Result<(), String> {
    let args = parse_serve_args()?;
    let workspace = args
        .workspace
        .canonicalize()
        .unwrap_or_else(|_| args.workspace.clone());
    let duckdb = crate::resolve_duckdb(args.duckdb.clone())?;

    // Set the workspace env once for the process; runs are serialized so these
    // stay consistent for every execution (matches the runner's run path).
    std::env::set_var("DUCKLE_DUCKDB_BIN", &duckdb);
    std::env::set_var("DUCKLE_WORKSPACE", &workspace);
    std::env::set_var("DUCKLE_LOG_DIR", workspace.join("logs"));

    let state = Arc::new(State {
        workspace: workspace.clone(),
        duckdb: duckdb.clone(),
        run_lock: Mutex::new(()),
        current_run: Mutex::new(None),
    });

    spawn_scheduler(state.clone());

    let addr = format!("{}:{}", args.host, args.port);
    let listener = TcpListener::bind(&addr).map_err(|e| format!("bind {}: {}", addr, e))?;
    eprintln!("duckle-runner: management console on http://{}", addr);
    eprintln!("duckle-runner: workspace {}", workspace.display());
    eprintln!("duckle-runner: DuckDB {}", duckdb.display());
    if args.host != "127.0.0.1" && args.host != "localhost" {
        eprintln!(
            "duckle-runner: WARNING - no authentication; exposed on {}",
            args.host
        );
    }

    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                let st = state.clone();
                std::thread::spawn(move || {
                    if let Err(e) = handle(s, &st) {
                        eprintln!("duckle-runner: request error: {}", e);
                    }
                });
            }
            Err(e) => eprintln!("duckle-runner: accept error: {}", e),
        }
    }
    Ok(())
}

// ── HTTP (minimal, std-only) ──

struct Request {
    method: String,
    path: String,
    query: HashMap<String, String>,
    body: Vec<u8>,
}

fn read_request(stream: &mut TcpStream) -> Result<Request, String> {
    // Read until the end of headers (\r\n\r\n), then the body by Content-Length.
    let mut buf = Vec::with_capacity(2048);
    let mut tmp = [0u8; 2048];
    let header_end;
    loop {
        let n = stream.read(&mut tmp).map_err(|e| e.to_string())?;
        if n == 0 {
            return Err("connection closed before request".into());
        }
        buf.extend_from_slice(&tmp[..n]);
        if let Some(pos) = find_subslice(&buf, b"\r\n\r\n") {
            header_end = pos;
            break;
        }
        if buf.len() > 1 << 20 {
            return Err("request headers too large".into());
        }
    }
    let head = String::from_utf8_lossy(&buf[..header_end]).to_string();
    let mut lines = head.split("\r\n");
    let request_line = lines.next().ok_or("empty request")?;
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("GET").to_string();
    let raw_target = parts.next().unwrap_or("/").to_string();
    let (path, query) = split_query(&raw_target);

    let mut content_length = 0usize;
    for line in lines {
        if let Some((k, v)) = line.split_once(':') {
            if k.trim().eq_ignore_ascii_case("content-length") {
                content_length = v.trim().parse().unwrap_or(0);
            }
        }
    }
    let mut body = buf[header_end + 4..].to_vec();
    while body.len() < content_length {
        let n = stream.read(&mut tmp).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        body.extend_from_slice(&tmp[..n]);
    }
    body.truncate(content_length);
    Ok(Request {
        method,
        path,
        query,
        body,
    })
}

fn find_subslice(hay: &[u8], needle: &[u8]) -> Option<usize> {
    hay.windows(needle.len()).position(|w| w == needle)
}

fn split_query(target: &str) -> (String, HashMap<String, String>) {
    let mut q = HashMap::new();
    let (path, qs) = match target.split_once('?') {
        Some((p, s)) => (p.to_string(), s),
        None => (target.to_string(), ""),
    };
    for pair in qs.split('&').filter(|s| !s.is_empty()) {
        let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
        q.insert(url_decode(k), url_decode(v));
    }
    (path, q)
}

fn url_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'%' if i + 2 < bytes.len() => {
                let h = (hex_val(bytes[i + 1]), hex_val(bytes[i + 2]));
                if let (Some(a), Some(b)) = h {
                    out.push(a * 16 + b);
                    i += 3;
                    continue;
                }
                out.push(b'%');
                i += 1;
            }
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            c => {
                out.push(c);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).to_string()
}

fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

fn respond(
    stream: &mut TcpStream,
    status: &str,
    content_type: &str,
    body: &[u8],
) -> Result<(), String> {
    let header = format!(
        "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nAccess-Control-Allow-Headers: content-type\r\nAccess-Control-Max-Age: 86400\r\nConnection: close\r\n\r\n",
        status,
        content_type,
        body.len()
    );
    stream
        .write_all(header.as_bytes())
        .map_err(|e| e.to_string())?;
    stream.write_all(body).map_err(|e| e.to_string())?;
    stream.flush().map_err(|e| e.to_string())
}

fn respond_json(stream: &mut TcpStream, value: &Value) -> Result<(), String> {
    respond(
        stream,
        "200 OK",
        "application/json",
        value.to_string().as_bytes(),
    )
}

fn respond_err(stream: &mut TcpStream, status: &str, msg: &str) -> Result<(), String> {
    respond(
        stream,
        status,
        "application/json",
        json!({ "error": msg }).to_string().as_bytes(),
    )
}

fn respond_ndjson_start(stream: &mut TcpStream) -> Result<(), String> {
    let header = "HTTP/1.1 200 OK\r\nContent-Type: application/x-ndjson\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nAccess-Control-Allow-Headers: content-type\r\nAccess-Control-Max-Age: 86400\r\nConnection: close\r\n\r\n";
    stream
        .write_all(header.as_bytes())
        .map_err(|e| e.to_string())?;
    stream.flush().map_err(|e| e.to_string())
}

fn write_ndjson_line(stream: &mut TcpStream, value: Value) -> Result<(), String> {
    let line = format!("{}\n", value);
    stream
        .write_all(line.as_bytes())
        .map_err(|e| e.to_string())?;
    stream.flush().map_err(|e| e.to_string())
}

fn api_studio_health(state: &State) -> Value {
    json!({
        "ok": true,
        "mode": "duckle-runner-serve",
        "workspace": state.workspace.to_string_lossy(),
        "duckdb": state.duckdb.to_string_lossy(),
    })
}

fn studio_pipeline_from_body(body: &[u8]) -> Result<PipelineDoc, String> {
    let value: Value = serde_json::from_slice(body).map_err(|e| format!("invalid json: {}", e))?;
    let pipeline = value.get("pipeline").cloned().unwrap_or(value);
    serde_json::from_value(pipeline).map_err(|e| format!("invalid pipeline: {}", e))
}

fn api_studio_compile(body: &[u8]) -> Result<Value, String> {
    let pipeline = studio_pipeline_from_body(body)?;
    let stages = compile_pipeline_sql(&pipeline).map_err(|e| e.to_string())?;
    serde_json::to_value(stages).map_err(|e| format!("serialize compile result: {}", e))
}

fn api_studio_run_inner(
    state: &State,
    body: &[u8],
    target_node_id: Option<String>,
    trigger: &str,
) -> Result<Value, String> {
    let value: Value = serde_json::from_slice(body).map_err(|e| format!("invalid json: {}", e))?;
    let pipeline = studio_pipeline_from_body(body)?;
    let pipeline_id = value
        .get("pipelineId")
        .and_then(|v| v.as_str())
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.to_string());
    let pipeline_name = value
        .get("pipelineName")
        .and_then(|v| v.as_str())
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.to_string());
    let workspace = value
        .get("workspacePath")
        .and_then(|v| v.as_str())
        .filter(|s| !s.trim().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| state.workspace.clone());
    let log_dir = workspace.join("logs");

    let _guard = state
        .run_lock
        .lock()
        .map_err(|_| "run lock poisoned".to_string())?;
    std::env::set_var("DUCKLE_DUCKDB_BIN", &state.duckdb);
    std::env::set_var("DUCKLE_WORKSPACE", &workspace);
    std::env::set_var("DUCKLE_LOG_DIR", &log_dir);

    let engine = DuckdbEngine::new(state.duckdb.clone()).for_new_run();
    *state
        .current_run
        .lock()
        .map_err(|_| "current run lock poisoned".to_string())? = Some(engine.clone());
    let result = engine.execute_pipeline_with_events(
        &pipeline,
        target_node_id.as_deref(),
        pipeline_name.as_deref(),
        |_| {},
    );
    *state
        .current_run
        .lock()
        .map_err(|_| "current run lock poisoned".to_string())? = None;

    if let Some(id) = &pipeline_id {
        let record = RunRecord::from_result(&result, trigger);
        let _ = append_run_record(&workspace, id, record);
    }

    serde_json::to_value(result).map_err(|e| format!("serialize run result: {}", e))
}

fn api_studio_run(state: &State, body: &[u8]) -> Result<Value, String> {
    api_studio_run_inner(state, body, None, "manual")
}

fn api_studio_run_partial(state: &State, body: &[u8]) -> Result<Value, String> {
    let value: Value = serde_json::from_slice(body).map_err(|e| format!("invalid json: {}", e))?;
    let target = value
        .get("targetNodeId")
        .and_then(|v| v.as_str())
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| "missing targetNodeId".to_string())?
        .to_string();
    api_studio_run_inner(state, body, Some(target), "partial")
}

fn api_studio_run_stream(
    state: &State,
    body: &[u8],
    target_node_id: Option<String>,
    trigger: &str,
    stream: &mut TcpStream,
) -> Result<(), String> {
    let value: Value = serde_json::from_slice(body).map_err(|e| format!("invalid json: {}", e))?;
    let pipeline = studio_pipeline_from_body(body)?;
    let pipeline_id = value
        .get("pipelineId")
        .and_then(|v| v.as_str())
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.to_string());
    let pipeline_name = value
        .get("pipelineName")
        .and_then(|v| v.as_str())
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.to_string());
    let workspace = value
        .get("workspacePath")
        .and_then(|v| v.as_str())
        .filter(|s| !s.trim().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| state.workspace.clone());
    let log_dir = workspace.join("logs");

    respond_ndjson_start(stream)?;

    let _guard = match state.run_lock.lock() {
        Ok(guard) => guard,
        Err(_) => {
            write_ndjson_line(
                stream,
                json!({ "kind": "error", "error": "run lock poisoned" }),
            )?;
            return Ok(());
        }
    };
    std::env::set_var("DUCKLE_DUCKDB_BIN", &state.duckdb);
    std::env::set_var("DUCKLE_WORKSPACE", &workspace);
    std::env::set_var("DUCKLE_LOG_DIR", &log_dir);

    let engine = DuckdbEngine::new(state.duckdb.clone()).for_new_run();
    match state.current_run.lock() {
        Ok(mut current) => *current = Some(engine.clone()),
        Err(_) => {
            write_ndjson_line(
                stream,
                json!({ "kind": "error", "error": "current run lock poisoned" }),
            )?;
            return Ok(());
        }
    }
    let result = engine.execute_pipeline_with_events(
        &pipeline,
        target_node_id.as_deref(),
        pipeline_name.as_deref(),
        |event: PipelineEvent| {
            let event_value = serde_json::to_value(event).unwrap_or_else(|e| {
                json!({
                    "type": "log",
                    "node_id": "runtime",
                    "level": "error",
                    "message": format!("serialize event: {}", e),
                })
            });
            let _ = write_ndjson_line(stream, json!({ "kind": "event", "event": event_value }));
        },
    );
    if let Ok(mut current) = state.current_run.lock() {
        *current = None;
    }

    if let Some(id) = &pipeline_id {
        let record = RunRecord::from_result(&result, trigger);
        let _ = append_run_record(&workspace, id, record);
    }

    let result_value = match serde_json::to_value(result) {
        Ok(value) => value,
        Err(e) => {
            write_ndjson_line(
                stream,
                json!({ "kind": "error", "error": format!("serialize run result: {}", e) }),
            )?;
            return Ok(());
        }
    };
    write_ndjson_line(stream, json!({ "kind": "result", "result": result_value }))
}

fn api_studio_run_partial_stream(
    state: &State,
    body: &[u8],
    stream: &mut TcpStream,
) -> Result<(), String> {
    let value: Value = serde_json::from_slice(body).map_err(|e| format!("invalid json: {}", e))?;
    let target = value
        .get("targetNodeId")
        .and_then(|v| v.as_str())
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| "missing targetNodeId".to_string())?
        .to_string();
    api_studio_run_stream(state, body, Some(target), "partial", stream)
}

fn format_inspect_error(err: InspectError) -> String {
    err.to_string()
}

fn api_studio_autodetect(state: &State, body: &[u8]) -> Result<Value, String> {
    let value: Value = serde_json::from_slice(body).map_err(|e| format!("invalid json: {}", e))?;
    let format = value
        .get("format")
        .and_then(|v| v.as_str())
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| "missing format".to_string())?;
    let options = value.get("options").cloned().unwrap_or_else(|| json!({}));
    let engine = DuckdbEngine::new(state.duckdb.clone());
    let inspection = match engine.inspect(format, options.clone()) {
        Ok(insp) => insp,
        Err(e) => {
            if matches!(format, "csv" | "tsv") {
                tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .map_err(|e| format!("tokio runtime: {}", e))?
                    .block_on(CsvConnector.inspect(options))
                    .map_err(format_inspect_error)?
            } else {
                return Err(e.to_string());
            }
        }
    };
    Ok(json!({
        "columns": inspection.schema,
        "sampleRows": inspection.sample_rows,
    }))
}

fn studio_workspace_from_query(state: &State, query: &HashMap<String, String>) -> PathBuf {
    query
        .get("workspacePath")
        .filter(|s| !s.trim().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| state.workspace.clone())
}

fn studio_pipeline_id_from_query(query: &HashMap<String, String>) -> Result<String, String> {
    query
        .get("pipelineId")
        .or_else(|| query.get("id"))
        .filter(|s| !s.trim().is_empty())
        .cloned()
        .ok_or_else(|| "missing pipelineId".to_string())
}

fn api_studio_history(state: &State, query: &HashMap<String, String>) -> Result<Value, String> {
    let workspace = studio_workspace_from_query(state, query);
    let pipeline_id = studio_pipeline_id_from_query(query)?;
    serde_json::to_value(load_run_history(&workspace, &pipeline_id))
        .map_err(|e| format!("serialize history: {}", e))
}

fn api_studio_logs(state: &State, query: &HashMap<String, String>) -> Result<Value, String> {
    let workspace = studio_workspace_from_query(state, query);
    let pipeline_id = studio_pipeline_id_from_query(query)?;
    let tail: usize = query
        .get("tail")
        .and_then(|t| t.parse().ok())
        .unwrap_or(200);
    let file = workspace
        .join("logs")
        .join(sanitize_segment(&pipeline_id))
        .join("runtime.log");
    let text = match std::fs::read_to_string(&file) {
        Ok(t) => t,
        Err(_) => {
            return Ok(json!({
                "entries": [],
                "file": file.to_string_lossy(),
            }))
        }
    };
    let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    let start = lines.len().saturating_sub(tail);
    let entries: Vec<Value> = lines[start..]
        .iter()
        .map(|l| serde_json::from_str::<Value>(l).unwrap_or_else(|_| json!({ "raw": l })))
        .collect();
    Ok(json!({
        "entries": entries,
        "file": file.to_string_lossy(),
    }))
}

fn api_studio_workspace_get(state: &State, query: &HashMap<String, String>) -> Result<Value, String> {
    let workspace = studio_workspace_from_query(state, query);
    load_studio_workspace(&workspace)
}

fn api_studio_workspace_save(state: &State, body: &[u8]) -> Result<Value, String> {
    let value: Value = serde_json::from_slice(body).map_err(|e| format!("invalid json: {}", e))?;
    let workspace = value
        .get("workspacePath")
        .and_then(|v| v.as_str())
        .filter(|s| !s.trim().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| state.workspace.clone());
    let workspace_state = value.get("state").unwrap_or(&value);
    save_studio_workspace(&workspace, workspace_state)?;
    Ok(json!({ "ok": true }))
}

fn read_json_file(path: &Path) -> Result<Value, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("read {}: {}", path.display(), e))?;
    serde_json::from_str(&text).map_err(|e| format!("parse {}: {}", path.display(), e))
}

fn load_studio_workspace(workspace: &Path) -> Result<Value, String> {
    let meta_path = workspace.join("duckle.json");
    if !meta_path.exists() {
        return Ok(Value::Null);
    }
    let meta = read_json_file(&meta_path)?;
    let repo_path = workspace.join("repository.json");
    let repo = if repo_path.exists() {
        read_json_file(&repo_path)?
    } else {
        json!([])
    };

    let mut pipeline_data = serde_json::Map::new();
    if let Some(items) = repo.as_array() {
        for item in items {
            let is_pipeline = item
                .get("type")
                .and_then(|v| v.as_str())
                .is_some_and(|t| t == "pipeline");
            let Some(id) = item.get("id").and_then(|v| v.as_str()) else {
                continue;
            };
            if !is_pipeline {
                continue;
            }
            let file = workspace.join("pipelines").join(format!("{}.json", id));
            if file.exists() {
                pipeline_data.insert(id.to_string(), read_json_file(&file)?);
            }
        }
    }

    Ok(json!({
        "version": meta.get("version").and_then(|v| v.as_u64()).unwrap_or(2),
        "engine": meta.get("engine").cloned().unwrap_or(Value::Null),
        "jobs": meta.get("jobs").cloned().unwrap_or_else(|| json!([])),
        "activeJobId": meta.get("activeJobId").cloned().unwrap_or(Value::Null),
        "repo": repo,
        "pipelineData": Value::Object(pipeline_data),
    }))
}

fn strip_payloads(value: &Value) -> Value {
    let Some(items) = value.as_array() else {
        return json!([]);
    };
    Value::Array(
        items
            .iter()
            .map(|item| {
                let Some(obj) = item.as_object() else {
                    return item.clone();
                };
                let mut next = obj.clone();
                next.remove("payload");
                Value::Object(next)
            })
            .collect(),
    )
}

fn write_json_file(path: &Path, value: &Value) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("create {}: {}", parent.display(), e))?;
    }
    let text = serde_json::to_string_pretty(value)
        .map_err(|e| format!("serialize {}: {}", path.display(), e))?;
    std::fs::write(path, text).map_err(|e| format!("write {}: {}", path.display(), e))
}

fn save_studio_workspace(workspace: &Path, state: &Value) -> Result<(), String> {
    std::fs::create_dir_all(workspace)
        .map_err(|e| format!("create {}: {}", workspace.display(), e))?;
    for dir in ["pipelines", "connections", "contexts", "routines", "docs"] {
        let path = workspace.join(dir);
        std::fs::create_dir_all(&path)
            .map_err(|e| format!("create {}: {}", path.display(), e))?;
    }

    let meta = json!({
        "version": state.get("version").and_then(|v| v.as_u64()).unwrap_or(2),
        "engine": state.get("engine").cloned().unwrap_or(Value::Null),
        "jobs": state.get("jobs").cloned().unwrap_or_else(|| json!([])),
        "activeJobId": state.get("activeJobId").cloned().unwrap_or(Value::Null),
    });
    write_json_file(&workspace.join("duckle.json"), &meta)?;

    let repo = strip_payloads(state.get("repo").unwrap_or(&Value::Null));
    write_json_file(&workspace.join("repository.json"), &repo)?;

    if let Some(pipelines) = state.get("pipelineData").and_then(|v| v.as_object()) {
        for (id, pipeline) in pipelines {
            write_json_file(
                &workspace.join("pipelines").join(format!("{}.json", id)),
                pipeline,
            )?;
        }
    }

    Ok(())
}

fn api_studio_cancel(state: &State) -> Result<Value, String> {
    let cancelled = match state
        .current_run
        .lock()
        .map_err(|_| "current run lock poisoned".to_string())?
        .as_ref()
    {
        Some(engine) => {
            engine.request_cancel();
            true
        }
        None => false,
    };
    Ok(json!({ "ok": true, "cancelled": cancelled }))
}

fn handle(mut stream: TcpStream, state: &State) -> Result<(), String> {
    let req = read_request(&mut stream)?;
    let route = (req.method.as_str(), req.path.as_str());
    match route {
        ("OPTIONS", _) => respond(&mut stream, "204 No Content", "text/plain", b""),
        ("GET", "/") | ("GET", "/index.html") => respond(
            &mut stream,
            "200 OK",
            "text/html; charset=utf-8",
            PANEL_HTML.as_bytes(),
        ),
        ("GET", "/api/summary") => respond_json(&mut stream, &api_summary(state)),
        ("GET", "/api/pipelines") => respond_json(&mut stream, &api_pipelines(state)),
        ("GET", "/api/studio/health") => respond_json(&mut stream, &api_studio_health(state)),
        ("GET", "/api/studio/workspace") => match api_studio_workspace_get(state, &req.query) {
            Ok(v) => respond_json(&mut stream, &v),
            Err(e) => respond_err(&mut stream, "400 Bad Request", &e),
        },
        ("POST", "/api/studio/workspace") => match api_studio_workspace_save(state, &req.body) {
            Ok(v) => respond_json(&mut stream, &v),
            Err(e) => respond_err(&mut stream, "400 Bad Request", &e),
        },
        ("POST", "/api/studio/compile") => match api_studio_compile(&req.body) {
            Ok(v) => respond_json(&mut stream, &v),
            Err(e) => respond_err(&mut stream, "400 Bad Request", &e),
        },
        ("POST", "/api/studio/run") => match api_studio_run(state, &req.body) {
            Ok(v) => respond_json(&mut stream, &v),
            Err(e) => respond_err(&mut stream, "400 Bad Request", &e),
        },
        ("POST", "/api/studio/run-partial") => match api_studio_run_partial(state, &req.body) {
            Ok(v) => respond_json(&mut stream, &v),
            Err(e) => respond_err(&mut stream, "400 Bad Request", &e),
        },
        ("POST", "/api/studio/run-stream") => {
            match api_studio_run_stream(state, &req.body, None, "manual", &mut stream) {
                Ok(()) => Ok(()),
                Err(e) => respond_err(&mut stream, "400 Bad Request", &e),
            }
        }
        ("POST", "/api/studio/run-partial-stream") => {
            match api_studio_run_partial_stream(state, &req.body, &mut stream) {
                Ok(()) => Ok(()),
                Err(e) => respond_err(&mut stream, "400 Bad Request", &e),
            }
        }
        ("POST", "/api/studio/autodetect") => match api_studio_autodetect(state, &req.body) {
            Ok(v) => respond_json(&mut stream, &v),
            Err(e) => respond_err(&mut stream, "400 Bad Request", &e),
        },
        ("GET", "/api/studio/history") => match api_studio_history(state, &req.query) {
            Ok(v) => respond_json(&mut stream, &v),
            Err(e) => respond_err(&mut stream, "400 Bad Request", &e),
        },
        ("GET", "/api/studio/logs") => match api_studio_logs(state, &req.query) {
            Ok(v) => respond_json(&mut stream, &v),
            Err(e) => respond_err(&mut stream, "400 Bad Request", &e),
        },
        ("POST", "/api/studio/cancel") => match api_studio_cancel(state) {
            Ok(v) => respond_json(&mut stream, &v),
            Err(e) => respond_err(&mut stream, "400 Bad Request", &e),
        },
        ("GET", "/api/pipeline") => match req.query.get("file") {
            Some(f) => match read_pipeline_file(state, f) {
                Ok(v) => respond_json(&mut stream, &v),
                Err(e) => respond_err(&mut stream, "404 Not Found", &e),
            },
            None => respond_err(&mut stream, "400 Bad Request", "missing file"),
        },
        ("GET", "/api/runs") => respond_json(
            &mut stream,
            &api_runs(state, req.query.get("id").map(|s| s.as_str())),
        ),
        ("GET", "/api/log") => respond_json(&mut stream, &api_log(state, &req.query)),
        ("GET", "/api/schedules") => respond_json(&mut stream, &load_schedules(state)),
        ("POST", "/api/schedules") => {
            let body: Value = serde_json::from_slice(&req.body).unwrap_or(json!({}));
            match save_schedule(state, &body) {
                Ok(v) => respond_json(&mut stream, &v),
                Err(e) => respond_err(&mut stream, "400 Bad Request", &e),
            }
        }
        ("POST", "/api/run") => {
            let body: Value = serde_json::from_slice(&req.body).unwrap_or(json!({}));
            let file = match body.get("file").and_then(|v| v.as_str()) {
                Some(f) => f.to_string(),
                None => return respond_err(&mut stream, "400 Bad Request", "missing file"),
            };
            match execute_one(state, &file, "manual") {
                Ok(v) => respond_json(&mut stream, &v),
                Err(e) => respond_err(&mut stream, "400 Bad Request", &e),
            }
        }
        _ => respond_err(&mut stream, "404 Not Found", "not found"),
    }
}

// ── Pipeline discovery ──

/// Scan the workspace for pipeline files (a `.json` with a top-level `nodes`
/// array), skipping bookkeeping folders. Returns (absolute path, id, value).
fn discover_pipelines(workspace: &Path) -> Vec<(PathBuf, String, Value)> {
    let mut out = Vec::new();
    let skip = [
        "runs",
        "logs",
        "connections",
        "node_modules",
        ".duckle",
        ".git",
        "target",
    ];
    let mut stack = vec![workspace.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let rd = match std::fs::read_dir(&dir) {
            Ok(r) => r,
            Err(_) => continue,
        };
        for entry in rd.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if !skip.contains(&name) {
                    stack.push(path);
                }
                continue;
            }
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let text = match std::fs::read_to_string(&path) {
                Ok(t) => t,
                Err(_) => continue,
            };
            let v: Value = match serde_json::from_str(&text) {
                Ok(v) => v,
                Err(_) => continue,
            };
            if v.get("nodes").and_then(|n| n.as_array()).is_some() {
                let id = path
                    .file_stem()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_default();
                out.push((path, id, v));
            }
        }
    }
    out.sort_by(|a, b| a.1.to_lowercase().cmp(&b.1.to_lowercase()));
    out
}

fn rel(workspace: &Path, path: &Path) -> String {
    path.strip_prefix(workspace)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn last_run(workspace: &Path, id: &str) -> Option<RunRecord> {
    // History is appended in order; the most recent record is last.
    load_run_history(workspace, id).into_iter().last()
}

fn api_pipelines(state: &State) -> Value {
    let scheds = load_schedules(state);
    let items: Vec<Value> = discover_pipelines(&state.workspace)
        .into_iter()
        .map(|(path, id, v)| {
            let last = last_run(&state.workspace, &id);
            let sched = scheds.get(&id).cloned().unwrap_or(json!({ "enabled": false, "intervalMinutes": 0 }));
            json!({
                "file": rel(&state.workspace, &path),
                "id": id,
                "name": v.get("name").and_then(|x| x.as_str()).unwrap_or(""),
                "nodeCount": v.get("nodes").and_then(|n| n.as_array()).map(|a| a.len()).unwrap_or(0),
                "edgeCount": v.get("edges").and_then(|e| e.as_array()).map(|a| a.len()).unwrap_or(0),
                "lastStatus": last.as_ref().map(|r| r.status.clone()),
                "lastAt": last.as_ref().map(|r| r.at.clone()),
                "lastDurationMs": last.as_ref().map(|r| r.duration_ms),
                "lastRows": last.as_ref().map(|r| r.rows),
                "schedule": sched,
            })
        })
        .collect();
    json!({ "pipelines": items })
}

fn api_summary(state: &State) -> Value {
    let pipes = discover_pipelines(&state.workspace);
    let mut total_runs = 0u64;
    let mut ok = 0u64;
    let mut failed = 0u64;
    for (_, id, _) in &pipes {
        for r in load_run_history(&state.workspace, id) {
            total_runs += 1;
            if r.status == "ok" {
                ok += 1;
            } else {
                failed += 1;
            }
        }
    }
    json!({
        "pipelineCount": pipes.len(),
        "totalRuns": total_runs,
        "ok": ok,
        "failed": failed,
        "workspace": state.workspace.to_string_lossy(),
    })
}

/// Run history across all pipelines (or one, when `id` is given), newest first,
/// each record tagged with its pipeline id/name.
fn api_runs(state: &State, only: Option<&str>) -> Value {
    let mut rows: Vec<Value> = Vec::new();
    for (path, id, v) in discover_pipelines(&state.workspace) {
        if let Some(want) = only {
            if want != id {
                continue;
            }
        }
        let name = v
            .get("name")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string();
        for r in load_run_history(&state.workspace, &id) {
            rows.push(json!({
                "id": id,
                "name": name,
                "file": rel(&state.workspace, &path),
                "at": r.at,
                "status": r.status,
                "durationMs": r.duration_ms,
                "rows": r.rows,
                "nodeCount": r.node_count,
                "trigger": r.trigger,
                "error": r.error,
                "category": r.category,
            }));
        }
    }
    // RunRecord.at is RFC3339 UTC, so a string sort orders by time; newest first.
    rows.sort_by(|a, b| {
        b.get("at")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .cmp(a.get("at").and_then(|v| v.as_str()).unwrap_or(""))
    });
    json!({ "runs": rows })
}

fn read_pipeline_file(state: &State, file: &str) -> Result<Value, String> {
    let path = resolve_in_workspace(&state.workspace, file)?;
    let text =
        std::fs::read_to_string(&path).map_err(|e| format!("read {}: {}", path.display(), e))?;
    serde_json::from_str(&text).map_err(|e| format!("parse {}: {}", path.display(), e))
}

/// Resolve a workspace-relative path and refuse anything that escapes the
/// workspace (no `..` traversal beyond the root).
fn resolve_in_workspace(workspace: &Path, file: &str) -> Result<PathBuf, String> {
    let candidate = workspace.join(file);
    let canon = candidate
        .canonicalize()
        .map_err(|_| format!("not found: {}", file))?;
    if !canon.starts_with(workspace) {
        return Err("path escapes workspace".into());
    }
    Ok(canon)
}

fn api_log(state: &State, query: &HashMap<String, String>) -> Value {
    let id = match query.get("id") {
        Some(i) => i,
        None => return json!({ "entries": [] }),
    };
    let tail: usize = query
        .get("tail")
        .and_then(|t| t.parse().ok())
        .unwrap_or(200);
    let file = state
        .workspace
        .join("logs")
        .join(sanitize_segment(id))
        .join("runtime.log");
    let text = match std::fs::read_to_string(&file) {
        Ok(t) => t,
        Err(_) => return json!({ "entries": [], "file": file.to_string_lossy() }),
    };
    let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    let start = lines.len().saturating_sub(tail);
    let entries: Vec<Value> = lines[start..]
        .iter()
        .map(|l| serde_json::from_str::<Value>(l).unwrap_or_else(|_| json!({ "raw": l })))
        .collect();
    json!({ "entries": entries, "file": file.to_string_lossy() })
}

/// Match the engine's per-pipeline log-folder sanitization (run_log.rs).
fn sanitize_segment(name: &str) -> String {
    let s: String = name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if s.is_empty() {
        "pipeline".into()
    } else {
        s
    }
}

// ── Schedules ──

fn schedules_path(workspace: &Path) -> PathBuf {
    workspace.join("panel-schedules.json")
}

/// Schedule store: { "<pipeline id>": { "enabled": bool, "intervalMinutes": n } }.
fn load_schedules(state: &State) -> Value {
    std::fs::read_to_string(schedules_path(&state.workspace))
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_else(|| json!({}))
}

fn save_schedule(state: &State, body: &Value) -> Result<Value, String> {
    let id = body
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or("missing id")?;
    let enabled = body
        .get("enabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let interval = body
        .get("intervalMinutes")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let mut all = load_schedules(state);
    let obj = all.as_object_mut().ok_or("schedule store corrupt")?;
    obj.insert(
        id.to_string(),
        json!({ "enabled": enabled, "intervalMinutes": interval }),
    );
    std::fs::write(schedules_path(&state.workspace), all.to_string())
        .map_err(|e| format!("write schedules: {}", e))?;
    Ok(json!({ "ok": true }))
}

// ── Execution ──

/// Run one pipeline by its workspace-relative file path, end to end: resolve
/// env/time placeholders (as the runner does), execute through the engine,
/// append a run-history record, and return a result summary. Serialized by the
/// run lock so a scheduled run never overlaps a manual one.
fn execute_one(state: &State, file: &str, trigger: &str) -> Result<Value, String> {
    let path = resolve_in_workspace(&state.workspace, file)?;
    let text =
        std::fs::read_to_string(&path).map_err(|e| format!("read {}: {}", path.display(), e))?;
    let mut doc: PipelineDoc =
        serde_json::from_str(&text).map_err(|e| format!("parse {}: {}", path.display(), e))?;

    let id = path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "pipeline".into());

    let _guard = state
        .run_lock
        .lock()
        .map_err(|_| "run lock poisoned".to_string())?;

    // Same placeholder resolution as `duckle-runner run`: ${ENV:KEY} secrets,
    // then the dynamic ${date}/${datetime}/... builtins.
    let env_file = state.workspace.join("secrets.env");
    crate::apply_env_pass(&mut doc, &state.workspace, &env_file)?;
    duckle_duckdb_engine::context::apply_time_builtins(&mut doc);

    let engine = DuckdbEngine::new(state.duckdb.clone());
    let result = engine.execute_pipeline_named(&doc, &id);

    let _ = append_run_record(
        &state.workspace,
        &id,
        RunRecord::from_result(&result, trigger),
    );

    Ok(json!({
        "id": id,
        "status": result.status,
        "durationMs": result.duration_ms,
        "error": result.error,
        "nodes": result.nodes.iter().map(|(nid, st)| json!({
            "id": nid, "status": st.status, "rows": st.rows, "durationMs": st.duration_ms, "error": st.error,
        })).collect::<Vec<_>>(),
    }))
}

// ── Scheduler ──

/// Background loop: every 30s, run any enabled pipeline whose interval has
/// elapsed since it last ran here. Timing is tracked in-memory from process
/// start (first run fires one interval after boot), so no clock parsing and no
/// surprise burst of runs on restart.
fn spawn_scheduler(state: Arc<State>) {
    std::thread::spawn(move || {
        let mut last_fired: HashMap<String, Instant> = HashMap::new();
        loop {
            std::thread::sleep(Duration::from_secs(30));
            let scheds = load_schedules(&state);
            let obj = match scheds.as_object() {
                Some(o) => o,
                None => continue,
            };
            // Map id -> its file path for the enabled, due ones.
            let pipes: HashMap<String, PathBuf> = discover_pipelines(&state.workspace)
                .into_iter()
                .map(|(p, id, _)| (id, p))
                .collect();
            for (id, cfg) in obj {
                let enabled = cfg
                    .get("enabled")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let minutes = cfg
                    .get("intervalMinutes")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                if !enabled || minutes == 0 {
                    last_fired.remove(id);
                    continue;
                }
                let interval = Duration::from_secs(minutes * 60);
                let due = match last_fired.get(id) {
                    Some(t) => t.elapsed() >= interval,
                    None => false, // first sighting: start the clock, fire next interval
                };
                let now = Instant::now();
                if last_fired.get(id).is_none() {
                    last_fired.insert(id.clone(), now);
                    continue;
                }
                if due {
                    if let Some(path) = pipes.get(id) {
                        let file = rel(&state.workspace, path);
                        last_fired.insert(id.clone(), now);
                        match execute_one(&state, &file, "scheduled") {
                            Ok(v) => eprintln!(
                                "duckle-runner: scheduled {} -> {}",
                                id,
                                v.get("status").and_then(|s| s.as_str()).unwrap_or("?")
                            ),
                            Err(e) => eprintln!("duckle-runner: scheduled {} failed: {}", id, e),
                        }
                    }
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT_TMP: AtomicUsize = AtomicUsize::new(0);

    struct TempRoot {
        path: PathBuf,
    }

    impl TempRoot {
        fn new() -> Self {
            let n = NEXT_TMP.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "duckle-runner-serve-test-{}-{}",
                std::process::id(),
                n
            ));
            let _ = std::fs::remove_dir_all(&path);
            std::fs::create_dir_all(&path).unwrap();
            Self { path }
        }
    }

    impl Drop for TempRoot {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }

    struct HttpResponse {
        status: String,
        headers: String,
        raw_body: String,
        body: Value,
    }

    fn make_state(workspace: PathBuf, duckdb: PathBuf) -> State {
        State {
            workspace,
            duckdb,
            run_lock: Mutex::new(()),
            current_run: Mutex::new(None),
        }
    }

    fn make_state_with_current_run(workspace: PathBuf, duckdb: PathBuf) -> State {
        State {
            workspace,
            current_run: Mutex::new(Some(DuckdbEngine::new(duckdb.clone()).for_new_run())),
            duckdb,
            run_lock: Mutex::new(()),
        }
    }

    fn write_pipeline(workspace: &Path, name: &str) {
        std::fs::write(
            workspace.join(name),
            r#"{"name":"Example Pipeline","nodes":[],"edges":[]}"#,
        )
        .unwrap();
    }

    fn request(state: State, target: &str) -> HttpResponse {
        request_with_method(state, "GET", target)
    }

    fn request_with_method(state: State, method: &str, target: &str) -> HttpResponse {
        request_with_body(state, method, target, "")
    }

    fn request_with_body(state: State, method: &str, target: &str, body: &str) -> HttpResponse {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let target = target.to_string();
        let method = method.to_string();
        let body = body.to_string();
        let server = std::thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            handle(stream, &state).unwrap();
        });

        let mut client = TcpStream::connect(addr).unwrap();
        write!(
            client,
            "{} {} HTTP/1.1\r\nHost: 127.0.0.1\r\nOrigin: http://localhost:5173\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            method,
            target,
            body.len(),
            body
        )
        .unwrap();
        client.flush().unwrap();

        let mut raw = String::new();
        client.read_to_string(&mut raw).unwrap();
        server.join().unwrap();

        let (head, body) = raw.split_once("\r\n\r\n").unwrap();
        let status = head.lines().next().unwrap().to_string();
        let headers = head.to_string();
        let raw_body = body.to_string();
        let body = if body.trim().is_empty() {
            Value::Null
        } else {
            serde_json::from_str(body).unwrap_or_else(|_| json!({ "raw": body }))
        };
        HttpResponse {
            status,
            headers,
            raw_body,
            body,
        }
    }

    #[test]
    fn summary_endpoint_returns_workspace_counts() {
        let tmp = TempRoot::new();
        let workspace = tmp.path.join("workspace");
        std::fs::create_dir_all(&workspace).unwrap();
        write_pipeline(&workspace, "example.json");
        let duckdb = tmp.path.join("duckdb");
        std::fs::write(&duckdb, "").unwrap();

        let response = request(make_state(workspace, duckdb), "/api/summary");

        assert!(
            response.status.starts_with("HTTP/1.1 200 OK"),
            "{}",
            response.status
        );
        assert_eq!(response.body["pipelineCount"], 1);
        assert_eq!(response.body["workspace"].as_str().is_some(), true);
    }

    #[test]
    fn pipelines_endpoint_discovers_pipeline_json() {
        let tmp = TempRoot::new();
        let workspace = tmp.path.join("workspace");
        std::fs::create_dir_all(&workspace).unwrap();
        write_pipeline(&workspace, "example.json");
        let duckdb = tmp.path.join("duckdb");
        std::fs::write(&duckdb, "").unwrap();

        let response = request(make_state(workspace, duckdb), "/api/pipelines");

        assert!(
            response.status.starts_with("HTTP/1.1 200 OK"),
            "{}",
            response.status
        );
        let pipelines = response.body["pipelines"].as_array().unwrap();
        assert_eq!(pipelines.len(), 1);
        assert_eq!(pipelines[0]["file"], "example.json");
        assert_eq!(pipelines[0]["id"], "example");
        assert_eq!(pipelines[0]["name"], "Example Pipeline");
    }

    #[test]
    fn pipeline_endpoint_refuses_path_traversal() {
        let tmp = TempRoot::new();
        let workspace = tmp.path.join("workspace");
        std::fs::create_dir_all(&workspace).unwrap();
        std::fs::write(tmp.path.join("outside.json"), r#"{"nodes":[],"edges":[]}"#).unwrap();
        let duckdb = tmp.path.join("duckdb");
        std::fs::write(&duckdb, "").unwrap();

        let response = request(
            make_state(workspace, duckdb),
            "/api/pipeline?file=../outside.json",
        );

        assert!(
            response.status.starts_with("HTTP/1.1 404 Not Found"),
            "{}",
            response.status
        );
        assert_eq!(response.body["error"], "path escapes workspace");
    }

    #[test]
    fn unknown_route_returns_json_404() {
        let tmp = TempRoot::new();
        let workspace = tmp.path.join("workspace");
        std::fs::create_dir_all(&workspace).unwrap();
        let duckdb = tmp.path.join("duckdb");
        std::fs::write(&duckdb, "").unwrap();

        let response = request(make_state(workspace, duckdb), "/api/does-not-exist");

        assert!(
            response.status.starts_with("HTTP/1.1 404 Not Found"),
            "{}",
            response.status
        );
        assert_eq!(response.body["error"], "not found");
    }

    #[test]
    fn studio_health_reports_bridge_state() {
        let tmp = TempRoot::new();
        let workspace = tmp.path.join("workspace");
        std::fs::create_dir_all(&workspace).unwrap();
        let duckdb = tmp.path.join("duckdb");
        std::fs::write(&duckdb, "").unwrap();

        let response = request(
            make_state(workspace.clone(), duckdb.clone()),
            "/api/studio/health",
        );

        assert!(
            response.status.starts_with("HTTP/1.1 200 OK"),
            "{}",
            response.status
        );
        assert_eq!(response.body["ok"], true);
        assert_eq!(response.body["mode"], "duckle-runner-serve");
        assert_eq!(
            response.body["workspace"].as_str(),
            Some(workspace.to_string_lossy().as_ref())
        );
        assert_eq!(
            response.body["duckdb"].as_str(),
            Some(duckdb.to_string_lossy().as_ref())
        );
        assert!(
            response.headers.contains("Access-Control-Allow-Origin: *"),
            "{}",
            response.headers
        );
    }

    #[test]
    fn options_preflight_returns_cors_headers() {
        let tmp = TempRoot::new();
        let workspace = tmp.path.join("workspace");
        std::fs::create_dir_all(&workspace).unwrap();
        let duckdb = tmp.path.join("duckdb");
        std::fs::write(&duckdb, "").unwrap();

        let response = request_with_method(
            make_state(workspace, duckdb),
            "OPTIONS",
            "/api/studio/health",
        );

        assert!(
            response.status.starts_with("HTTP/1.1 204 No Content"),
            "{}",
            response.status
        );
        assert!(
            response
                .headers
                .contains("Access-Control-Allow-Methods: GET, POST, OPTIONS"),
            "{}",
            response.headers
        );
        assert!(
            response
                .headers
                .contains("Access-Control-Allow-Headers: content-type"),
            "{}",
            response.headers
        );
    }

    #[test]
    fn studio_compile_returns_stage_sql_for_valid_pipeline() {
        let tmp = TempRoot::new();
        let workspace = tmp.path.join("workspace");
        std::fs::create_dir_all(&workspace).unwrap();
        let duckdb = tmp.path.join("duckdb");
        std::fs::write(&duckdb, "").unwrap();
        let body = r#"{
            "pipeline": {
                "nodes": [
                    {
                        "id": "sql1",
                        "position": { "x": 0, "y": 0 },
                        "data": {
                            "label": "sql1",
                            "componentId": "code.sql",
                            "properties": { "sql": "select 1 as n" }
                        }
                    }
                ],
                "edges": []
            }
        }"#;

        let response = request_with_body(
            make_state(workspace, duckdb),
            "POST",
            "/api/studio/compile",
            body,
        );

        assert!(
            response.status.starts_with("HTTP/1.1 200 OK"),
            "{}",
            response.status
        );
        let stages = response.body.as_array().unwrap();
        assert_eq!(stages.len(), 1);
        assert_eq!(stages[0]["node_id"], "sql1");
        assert_eq!(stages[0]["kind"], "view");
        assert!(stages[0]["sql"]
            .as_str()
            .unwrap()
            .to_lowercase()
            .contains("select 1 as n"));
    }

    #[test]
    fn studio_compile_returns_json_400_for_invalid_pipeline() {
        let tmp = TempRoot::new();
        let workspace = tmp.path.join("workspace");
        std::fs::create_dir_all(&workspace).unwrap();
        let duckdb = tmp.path.join("duckdb");
        std::fs::write(&duckdb, "").unwrap();
        let body = r#"{"pipeline":{"nodes":"not-an-array","edges":[]}}"#;

        let response = request_with_body(
            make_state(workspace, duckdb),
            "POST",
            "/api/studio/compile",
            body,
        );

        assert!(
            response.status.starts_with("HTTP/1.1 400 Bad Request"),
            "{}",
            response.status
        );
        assert!(response.body["error"]
            .as_str()
            .unwrap()
            .contains("invalid pipeline"));
    }

    #[test]
    fn studio_run_returns_json_400_for_invalid_pipeline() {
        let tmp = TempRoot::new();
        let workspace = tmp.path.join("workspace");
        std::fs::create_dir_all(&workspace).unwrap();
        let duckdb = tmp.path.join("duckdb");
        std::fs::write(&duckdb, "").unwrap();
        let body = r#"{"pipeline":{"nodes":"not-an-array","edges":[]}}"#;

        let response = request_with_body(
            make_state(workspace, duckdb),
            "POST",
            "/api/studio/run",
            body,
        );

        assert!(
            response.status.starts_with("HTTP/1.1 400 Bad Request"),
            "{}",
            response.status
        );
        assert!(response.body["error"]
            .as_str()
            .unwrap()
            .contains("invalid pipeline"));
    }

    #[test]
    fn studio_run_executes_pipeline_when_duckdb_is_available() {
        let duckdb = match std::env::var("DUCKLE_DUCKDB_BIN").ok().map(PathBuf::from) {
            Some(p) if p.exists() => p,
            _ => {
                eprintln!("skipping: set DUCKLE_DUCKDB_BIN to a duckdb CLI to run");
                return;
            }
        };
        let tmp = TempRoot::new();
        let workspace = tmp.path.join("workspace");
        std::fs::create_dir_all(&workspace).unwrap();
        let body = format!(
            r#"{{
                "pipeline": {{
                    "nodes": [
                        {{
                            "id": "sql1",
                            "position": {{ "x": 0, "y": 0 }},
                            "data": {{
                                "label": "sql1",
                                "componentId": "code.sql",
                                "properties": {{ "sql": "select 7 as n" }}
                            }}
                        }}
                    ],
                    "edges": []
                }},
                "pipelineId": "phase3",
                "pipelineName": "Phase 3",
                "workspacePath": "{}"
            }}"#,
            workspace.to_string_lossy().replace('\\', "\\\\")
        );

        let response = request_with_body(
            make_state(workspace.clone(), duckdb),
            "POST",
            "/api/studio/run",
            &body,
        );

        assert!(
            response.status.starts_with("HTTP/1.1 200 OK"),
            "{}",
            response.status
        );
        assert_eq!(response.body["status"], "ok");
        assert_eq!(response.body["nodes"]["sql1"]["status"], "ok");
        let preview_rows = response.body["preview"][0]["rows"].as_array().unwrap();
        assert_eq!(preview_rows[0]["n"], 7);

        let history_path = workspace.join("runs").join("phase3.json");
        let history = std::fs::read_to_string(history_path).unwrap();
        let records: Value = serde_json::from_str(&history).unwrap();
        assert_eq!(records[0]["status"], "ok");
        assert_eq!(records[0]["trigger"], "manual");
    }

    #[test]
    fn studio_run_partial_requires_target_node_id() {
        let tmp = TempRoot::new();
        let workspace = tmp.path.join("workspace");
        std::fs::create_dir_all(&workspace).unwrap();
        let duckdb = tmp.path.join("duckdb");
        std::fs::write(&duckdb, "").unwrap();
        let body = r#"{"pipeline":{"nodes":[],"edges":[]}}"#;

        let response = request_with_body(
            make_state(workspace, duckdb),
            "POST",
            "/api/studio/run-partial",
            body,
        );

        assert!(
            response.status.starts_with("HTTP/1.1 400 Bad Request"),
            "{}",
            response.status
        );
        assert_eq!(response.body["error"], "missing targetNodeId");
    }

    #[test]
    fn studio_run_partial_executes_target_when_duckdb_is_available() {
        let duckdb = match std::env::var("DUCKLE_DUCKDB_BIN").ok().map(PathBuf::from) {
            Some(p) if p.exists() => p,
            _ => {
                eprintln!("skipping: set DUCKLE_DUCKDB_BIN to a duckdb CLI to run");
                return;
            }
        };
        let tmp = TempRoot::new();
        let workspace = tmp.path.join("workspace");
        std::fs::create_dir_all(&workspace).unwrap();
        let body = format!(
            r#"{{
                "pipeline": {{
                    "nodes": [
                        {{
                            "id": "sql1",
                            "position": {{ "x": 0, "y": 0 }},
                            "data": {{
                                "label": "sql1",
                                "componentId": "code.sql",
                                "properties": {{ "sql": "select 11 as n" }}
                            }}
                        }}
                    ],
                    "edges": []
                }},
                "targetNodeId": "sql1",
                "pipelineId": "phase5",
                "pipelineName": "Phase 5",
                "workspacePath": "{}"
            }}"#,
            workspace.to_string_lossy().replace('\\', "\\\\")
        );

        let response = request_with_body(
            make_state(workspace.clone(), duckdb),
            "POST",
            "/api/studio/run-partial",
            &body,
        );

        assert!(
            response.status.starts_with("HTTP/1.1 200 OK"),
            "{}",
            response.status
        );
        assert_eq!(response.body["status"], "ok");
        assert_eq!(response.body["nodes"]["sql1"]["status"], "ok");
        let preview_rows = response.body["preview"][0]["rows"].as_array().unwrap();
        assert_eq!(preview_rows[0]["n"], 11);

        let history_path = workspace.join("runs").join("phase5.json");
        let history = std::fs::read_to_string(history_path).unwrap();
        let records: Value = serde_json::from_str(&history).unwrap();
        assert_eq!(records[0]["status"], "ok");
        assert_eq!(records[0]["trigger"], "partial");
    }

    #[test]
    fn studio_run_stream_emits_events_and_result_when_duckdb_is_available() {
        let duckdb = match std::env::var("DUCKLE_DUCKDB_BIN").ok().map(PathBuf::from) {
            Some(p) if p.exists() => p,
            _ => {
                eprintln!("skipping: set DUCKLE_DUCKDB_BIN to a duckdb CLI to run");
                return;
            }
        };
        let tmp = TempRoot::new();
        let workspace = tmp.path.join("workspace");
        std::fs::create_dir_all(&workspace).unwrap();
        let body = format!(
            r#"{{
                "pipeline": {{
                    "nodes": [
                        {{
                            "id": "sql1",
                            "position": {{ "x": 0, "y": 0 }},
                            "data": {{
                                "label": "sql1",
                                "componentId": "code.sql",
                                "properties": {{ "sql": "select 13 as n" }}
                            }}
                        }}
                    ],
                    "edges": []
                }},
                "pipelineId": "phase9",
                "pipelineName": "Phase 9",
                "workspacePath": "{}"
            }}"#,
            workspace.to_string_lossy().replace('\\', "\\\\")
        );

        let response = request_with_body(
            make_state(workspace.clone(), duckdb),
            "POST",
            "/api/studio/run-stream",
            &body,
        );

        assert!(
            response.status.starts_with("HTTP/1.1 200 OK"),
            "{}",
            response.status
        );
        assert!(
            response
                .headers
                .contains("Content-Type: application/x-ndjson"),
            "{}",
            response.headers
        );
        let lines: Vec<Value> = response
            .raw_body
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        assert!(lines
            .iter()
            .any(|v| v["kind"] == "event" && v["event"]["type"] == "started"));
        assert!(lines
            .iter()
            .any(|v| v["kind"] == "event" && v["event"]["type"] == "stage_finished"));
        let result = lines
            .iter()
            .find(|v| v["kind"] == "result")
            .expect("result line");
        assert_eq!(result["result"]["status"], "ok");
        assert_eq!(result["result"]["preview"][0]["rows"][0]["n"], 13);

        let history_path = workspace.join("runs").join("phase9.json");
        let history = std::fs::read_to_string(history_path).unwrap();
        let records: Value = serde_json::from_str(&history).unwrap();
        assert_eq!(records[0]["status"], "ok");
        assert_eq!(records[0]["trigger"], "manual");
    }

    #[test]
    fn studio_run_partial_stream_requires_target_node_id() {
        let tmp = TempRoot::new();
        let workspace = tmp.path.join("workspace");
        std::fs::create_dir_all(&workspace).unwrap();
        let duckdb = tmp.path.join("duckdb");
        std::fs::write(&duckdb, "").unwrap();
        let body = r#"{"pipeline":{"nodes":[],"edges":[]}}"#;

        let response = request_with_body(
            make_state(workspace, duckdb),
            "POST",
            "/api/studio/run-partial-stream",
            body,
        );

        assert!(
            response.status.starts_with("HTTP/1.1 400 Bad Request"),
            "{}",
            response.status
        );
        assert_eq!(response.body["error"], "missing targetNodeId");
    }

    #[test]
    fn studio_autodetect_requires_format() {
        let tmp = TempRoot::new();
        let workspace = tmp.path.join("workspace");
        std::fs::create_dir_all(&workspace).unwrap();
        let duckdb = tmp.path.join("duckdb");
        std::fs::write(&duckdb, "").unwrap();

        let response = request_with_body(
            make_state(workspace, duckdb),
            "POST",
            "/api/studio/autodetect",
            r#"{"options":{}}"#,
        );

        assert!(
            response.status.starts_with("HTTP/1.1 400 Bad Request"),
            "{}",
            response.status
        );
        assert_eq!(response.body["error"], "missing format");
    }

    #[test]
    fn studio_autodetect_csv_returns_columns_and_sample_rows() {
        let tmp = TempRoot::new();
        let workspace = tmp.path.join("workspace");
        std::fs::create_dir_all(&workspace).unwrap();
        let csv = workspace.join("people.csv");
        std::fs::write(&csv, "id,name\n1,Ada\n2,Linus\n").unwrap();
        let duckdb = tmp.path.join("duckdb");
        std::fs::write(&duckdb, "").unwrap();
        let body = format!(
            r#"{{
                "format": "csv",
                "options": {{
                    "path": "{}",
                    "hasHeader": true
                }}
            }}"#,
            csv.to_string_lossy().replace('\\', "\\\\")
        );

        let response = request_with_body(
            make_state(workspace, duckdb),
            "POST",
            "/api/studio/autodetect",
            &body,
        );

        assert!(
            response.status.starts_with("HTTP/1.1 200 OK"),
            "{}",
            response.status
        );
        let columns = response.body["columns"].as_array().unwrap();
        assert_eq!(columns.len(), 2);
        assert_eq!(columns[0]["name"], "id");
        assert_eq!(columns[1]["name"], "name");
        let rows = response.body["sampleRows"].as_array().unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0]["name"], "Ada");
    }

    #[test]
    fn studio_history_requires_pipeline_id() {
        let tmp = TempRoot::new();
        let workspace = tmp.path.join("workspace");
        std::fs::create_dir_all(&workspace).unwrap();
        let duckdb = tmp.path.join("duckdb");
        std::fs::write(&duckdb, "").unwrap();

        let response = request(make_state(workspace, duckdb), "/api/studio/history");

        assert!(
            response.status.starts_with("HTTP/1.1 400 Bad Request"),
            "{}",
            response.status
        );
        assert_eq!(response.body["error"], "missing pipelineId");
    }

    #[test]
    fn studio_history_returns_run_records_for_pipeline() {
        let tmp = TempRoot::new();
        let workspace = tmp.path.join("workspace");
        let runs = workspace.join("runs");
        std::fs::create_dir_all(&runs).unwrap();
        std::fs::write(
            runs.join("phase7.json"),
            r#"[{"at":"2026-01-01T00:00:00Z","status":"ok","duration_ms":5,"rows":2,"node_count":1,"trigger":"manual"}]"#,
        )
        .unwrap();
        let duckdb = tmp.path.join("duckdb");
        std::fs::write(&duckdb, "").unwrap();

        let response = request(
            make_state(workspace, duckdb),
            "/api/studio/history?pipelineId=phase7",
        );

        assert!(
            response.status.starts_with("HTTP/1.1 200 OK"),
            "{}",
            response.status
        );
        let records = response.body.as_array().unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0]["status"], "ok");
        assert_eq!(records[0]["trigger"], "manual");
    }

    #[test]
    fn studio_logs_returns_tailed_runtime_entries() {
        let tmp = TempRoot::new();
        let workspace = tmp.path.join("workspace");
        let log_dir = workspace.join("logs").join("phase7");
        std::fs::create_dir_all(&log_dir).unwrap();
        std::fs::write(
            log_dir.join("runtime.log"),
            "{\"message\":\"first\"}\n{\"message\":\"second\"}\n",
        )
        .unwrap();
        let duckdb = tmp.path.join("duckdb");
        std::fs::write(&duckdb, "").unwrap();

        let response = request(
            make_state(workspace, duckdb),
            "/api/studio/logs?pipelineId=phase7&tail=1",
        );

        assert!(
            response.status.starts_with("HTTP/1.1 200 OK"),
            "{}",
            response.status
        );
        let entries = response.body["entries"].as_array().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0]["message"], "second");
    }

    #[test]
    fn studio_cancel_reports_false_when_no_run_is_active() {
        let tmp = TempRoot::new();
        let workspace = tmp.path.join("workspace");
        std::fs::create_dir_all(&workspace).unwrap();
        let duckdb = tmp.path.join("duckdb");
        std::fs::write(&duckdb, "").unwrap();

        let response =
            request_with_method(make_state(workspace, duckdb), "POST", "/api/studio/cancel");

        assert!(
            response.status.starts_with("HTTP/1.1 200 OK"),
            "{}",
            response.status
        );
        assert_eq!(response.body["ok"], true);
        assert_eq!(response.body["cancelled"], false);
    }

    #[test]
    fn studio_cancel_reports_true_when_run_is_active() {
        let tmp = TempRoot::new();
        let workspace = tmp.path.join("workspace");
        std::fs::create_dir_all(&workspace).unwrap();
        let duckdb = tmp.path.join("duckdb");
        std::fs::write(&duckdb, "").unwrap();

        let response = request_with_method(
            make_state_with_current_run(workspace, duckdb),
            "POST",
            "/api/studio/cancel",
        );

        assert!(
            response.status.starts_with("HTTP/1.1 200 OK"),
            "{}",
            response.status
        );
        assert_eq!(response.body["ok"], true);
        assert_eq!(response.body["cancelled"], true);
    }
}

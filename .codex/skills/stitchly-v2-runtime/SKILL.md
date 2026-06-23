---
name: stitchly-v2-runtime
description: Run, debug, or explain Stitchly v2/Duckle fork local runtime modes. Use when Codex needs to start or troubleshoot UI-only Vite, UI plus HTTP bridge, Tauri desktop, duckle-runner serve, DuckDB CLI paths, ports, CORS, streaming runtime endpoints, cancellation, logs, history, or local development verification for the Stitchly v2 repository.
---

# Stitchly v2 Runtime

Use this skill to choose and operate the right local run mode for Stitchly v2. Keep changes aligned with the repository's existing Tauri, Vite, and `duckle-runner` structure.

## First Steps

1. Find the repo root. Expected root markers include `Cargo.toml`, `frontend/package.json`, `apps/desktop`, and `crates/duckle-runner`.
2. Read `references/runtime-doc-map.md`, then open only the repo docs relevant to the task.
3. Check current process/port state before changing commands or advice. Common ports are `5173` for Vite and `8080` for the HTTP bridge.
4. Preserve the three-mode model:
   - `ui-only`: browser UI with mock/browser fallbacks.
   - `ui-http`: browser UI plus headless HTTP bridge.
   - `tauri`: full desktop shell and Tauri IPC.

## Command Patterns

Use UI-only for canvas, panels, forms, and styling:

```bash
npm --prefix frontend run dev
```

Use UI plus HTTP bridge for browser-based real execution:

```bash
cargo run -p duckle-runner -- serve --host 127.0.0.1 --port 8080 --workspace . --duckdb /path/to/duckdb
VITE_DUCKLE_BACKEND=http VITE_DUCKLE_HTTP_URL=http://127.0.0.1:8080 npm --prefix frontend run dev
```

Use Tauri for desktop-specific validation:

```bash
cd apps/desktop
cargo tauri dev
```

## HTTP Bridge Checks

Prefer endpoint-level checks before UI debugging:

```bash
curl http://127.0.0.1:8080/api/studio/health
curl -i -X OPTIONS http://127.0.0.1:8080/api/studio/health
```

Streaming endpoints return NDJSON:

```text
POST /api/studio/run-stream
POST /api/studio/run-partial-stream
```

Each line is one object with `kind: "event"`, `kind: "result"`, or `kind: "error"`.

## Run History And Logs

When asked for recent runs, last run status, run history, or failed-stage diagnosis, check both the HTTP bridge and file-backed workspace artifacts.

1. Prefer the HTTP bridge when it is running:

```bash
curl --get http://127.0.0.1:8080/api/studio/history \
  --data-urlencode "workspacePath=<workspace_path>" \
  --data-urlencode "pipelineId=<pipeline_id>"
```

2. If the bridge is offline or returns no rows, inspect workspace files:

```text
<workspace>/runs/<pipeline_id>.json
<workspace>/logs/<pipeline_name>/runtime.log
```

3. Summarize the latest run with: timestamp, status, duration, trigger, rows, node count, error/category, last successful stage, failed stage, and any important branch/gate behavior seen in `runtime.log`.

4. Use logs to reconstruct stage order. Look for JSON lines with `event=stage_started`, `event=stage_finished`, `event=log`, and `event=run_finished`. If a pipeline failed after a gate/switch, explicitly note which downstream branches executed.

5. If names are unknown, resolve them from:

```text
<workspace>/repository.json
<workspace>/duckle.json
<workspace>/pipelines/<pipeline_id>.json
```

## Verification

Run the smallest reliable check for the task:

```bash
npm --prefix frontend run lint
cargo test -p duckle-runner serve::tests
```

Use full runner tests after bridge/runtime changes:

```bash
cargo test -p duckle-runner
```

Use Tauri only when the change touches desktop shell behavior, native dialogs, bundling, or IPC-specific code.

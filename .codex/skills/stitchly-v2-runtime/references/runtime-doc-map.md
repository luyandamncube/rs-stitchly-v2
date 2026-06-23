# Runtime Doc Map

Use this map after locating the Stitchly v2 repository root.

## Primary Docs

- `docs/00_foundation/00_duckle-fork-foundation.md`: fork overview and repository orientation.
- `docs/00_foundation/01_local-studio-surfaces.md`: browser routes, backend surfaces, APIs, and running services.
- `docs/00_foundation/02_local-studio-quickstart.md`: setup, dependencies, install notes, and first run commands.
- `docs/00_foundation/03_duckdb-cli-execution-model.md`: why DuckDB is a CLI dependency and how execution differs from linked/in-process DuckDB.
- `docs/00_foundation/04_dev-run-modes.md`: UI-only, UI plus HTTP bridge, and Tauri desktop modes.
- `docs/03_runtime/00_runtime-overview.md`: runtime architecture overview.
- `docs/03_runtime/01_execution-model.md`: how workflows execute.
- `docs/03_runtime/02_planner-and-stage-compilation.md`: planner and stage SQL compilation.
- `docs/03_runtime/03_duckdb-cli-and-sql-batching.md`: DuckDB CLI batching behavior.
- `docs/03_runtime/05_state-watermarks-and-history.md`: state, watermarks, run history.
- `docs/03_runtime/06_files-artifacts-and-workspaces.md`: workspace files and artifacts.
- `docs/03_runtime/07_logs-errors-and-debugging.md`: logs and failure debugging.
- `docs/03_runtime/08_external-dependencies.md`: required external tools.
- `docs/03_runtime/09_http-runtime-bridge.md`: HTTP bridge phases, endpoints, streaming, frontend backend modes.

## Runtime Code Areas

- `crates/duckle-runner/src/serve.rs`: local HTTP runner server and browser-studio bridge endpoints.
- `crates/duckdb-engine`: core DuckDB CLI execution and planning.
- `frontend/src/tauri-bridge.ts`: frontend runtime transport selection for mock, HTTP, and Tauri modes.
- `apps/desktop`: Tauri desktop shell and IPC commands.

## Common Checks

```bash
npm --prefix frontend run lint
cargo test -p duckle-runner serve::tests
cargo test -p duckle-runner
```

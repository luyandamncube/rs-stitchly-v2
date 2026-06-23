# Node Doc Map

Use this map after locating the Stitchly v2 repository root.

## Node Docs

- `docs/01_nodes/00_node-inventory.md`: inventory of existing node types and categories.
- `docs/01_nodes/01_source-node-contracts.md`: source nodes, input options, output contracts.
- `docs/01_nodes/02_transform-node-contracts.md`: transform nodes and SQL/dataframe-like behavior.
- `docs/01_nodes/03_sink-node-contracts.md`: sink/output nodes and terminal behavior.
- `docs/01_nodes/04_quality-node-contracts.md`: quality checks, gates, and deadletter expectations.
- `docs/01_nodes/05_control-and-code-node-contracts.md`: code/shell/control nodes, restrictions, and side effects.

## Runtime Docs

- `docs/03_runtime/01_execution-model.md`: how node stages execute.
- `docs/03_runtime/02_planner-and-stage-compilation.md`: how graph nodes compile into execution stages.
- `docs/03_runtime/03_duckdb-cli-and-sql-batching.md`: SQL batching and DuckDB CLI details.
- `docs/03_runtime/04_runtime-specs.md`: runtime specs and compatibility notes.
- `docs/03_runtime/07_logs-errors-and-debugging.md`: debugging node failures.

## Workflow Docs

- `docs/02_workflows/01_workflow-json-and-graph-shape.md`: graph shape and JSON conventions.
- `docs/02_workflows/11_debugging-failed-workflows.md`: workflow-level failure diagnosis.

## Code Areas

- `frontend/src`: node palette, canvas, properties panel, and frontend type definitions.
- `crates/duckdb-engine`: stage planning, SQL generation, execution, preview/result behavior.
- `crates/connectors`: connector-specific inspection and IO behavior.
- `crates/plugin-sdk`: shared connector and plugin traits/contracts.
- `crates/duckle-runner/src/serve.rs`: HTTP bridge tests that exercise runtime behavior.

## Node Contract Template

Use this template when documenting or designing a node:

```text
Node:
Category:
Purpose:
Config:
Inputs:
Processing:
Outputs:
Runtime dependencies:
Restrictions:
Failure behavior:
Tests/verification:
```

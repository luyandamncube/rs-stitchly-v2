# Workflow Doc Map

Use this map after locating the Stitchly v2 repository root.

## Core Workflow Docs

- `docs/02_workflows/00_workflow-design-principles.md`: general workflow design rules.
- `docs/02_workflows/01_workflow-json-and-graph-shape.md`: workflow JSON shape, nodes, and edges.
- `docs/02_workflows/02_local-file-to-duckdb.md`: local file ingestion into DuckDB.
- `docs/02_workflows/03_api-ingestion-patterns.md`: API ingestion workflow patterns.
- `docs/02_workflows/04_database-ingestion-patterns.md`: database source workflows.
- `docs/02_workflows/05_quality-gates-and-deadletters.md`: validation gates and deadletter handling.
- `docs/02_workflows/06_cdc-and-incremental-loads.md`: incremental loading, state, and watermarks.
- `docs/02_workflows/07_dolt-parquet-workflows.md`: Dolt repository sync and Parquet-first workflow design.
- `docs/02_workflows/08_dbt-modeling-patterns.md`: dbt-oriented modeling patterns.
- `docs/02_workflows/09_ai-and-rag-workflows.md`: AI/RAG workflow placeholders and patterns.
- `docs/02_workflows/10_orchestration-and-jobs.md`: jobs and orchestration.
- `docs/02_workflows/11_debugging-failed-workflows.md`: failed workflow debugging.
- `docs/02_workflows/12_migrating-v1-workflows.md`: migration patterns from Stitchly v1.

## Supporting Docs

- `docs/01_nodes/00_node-inventory.md`: available node inventory.
- `docs/01_nodes/01_source-node-contracts.md`: source node contracts.
- `docs/01_nodes/02_transform-node-contracts.md`: transform node contracts.
- `docs/01_nodes/03_sink-node-contracts.md`: sink node contracts.
- `docs/01_nodes/04_quality-node-contracts.md`: quality node contracts.
- `docs/01_nodes/05_control-and-code-node-contracts.md`: control/code node contracts.
- `docs/03_runtime/01_execution-model.md`: execution behavior.
- `docs/03_runtime/05_state-watermarks-and-history.md`: state and history.
- `docs/03_runtime/09_http-runtime-bridge.md`: browser runtime execution endpoints.

## Code Areas

- `frontend/src`: workflow graph editing and bridge calls.
- `crates/duckdb-engine`: planner and runtime execution.
- `crates/duckle-runner/src/serve.rs`: HTTP bridge endpoints that execute in-memory workflow JSON.

## Common Output Tables

For workflow plans, prefer these tables:

- Workflow inventory: workflow name, purpose, schedule/manual trigger, source system, output artifact.
- Node plan: node id, type, purpose, config, inputs, outputs.
- Edge plan: from node, to node, data contract.
- State plan: state key, watermark, manifest file, idempotency strategy.
- Validation plan: command/endpoint, expected result, failure signal.

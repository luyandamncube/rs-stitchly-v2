---
name: stitchly-v2-node-author
description: Build, modify, document, or debug Stitchly v2 workflow nodes. Use when Codex needs to inspect existing node types, add new source/transform/sink/quality/control/code nodes, update node contracts, wire frontend node UI to planner/runtime behavior, or verify node input/output/data-contract behavior in the Stitchly v2/Duckle fork.
---

# Stitchly v2 Node Author

Use this skill to work on Stitchly v2 nodes across UI, workflow JSON, planner, and runtime execution. Treat node contracts as the source of truth before changing implementation.

## First Steps

1. Find the repo root. Expected root markers include `Cargo.toml`, `frontend/package.json`, `docs/01_nodes`, `crates/duckdb-engine`, and `frontend/src`.
2. Read `references/node-doc-map.md`, then open only the repo docs relevant to the node category.
3. Classify the node as source, transform, sink, quality, control, or code.
4. Locate existing implementation patterns with `rg` before adding new abstractions.

## Node Change Checklist

- Define the node contract first: config shape, inputs, processing behavior, outputs, restrictions, runtime dependencies, and failure behavior.
- Keep UI config names aligned with workflow JSON and Rust planner/runtime expectations.
- Prefer existing DuckDB SQL compilation paths for data operations.
- Use external commands or shell nodes only when a connector/tool cannot be represented cleanly as SQL or an existing runtime primitive.
- Add or update docs under `docs/01_nodes` when behavior changes.
- Add focused tests for planner/runtime behavior when the change affects execution.

## Files To Inspect

Use `rg` to find the exact implementation. Common areas:

- `frontend/src` for node palette, forms, graph state, and runtime bridge calls.
- `crates/duckdb-engine` for planning, stage compilation, and execution.
- `crates/connectors` for connector-specific inspection or IO behavior.
- `crates/plugin-sdk` for shared connector/plugin contracts.
- `docs/01_nodes` for documented contracts.

## Verification

For UI-only node form changes:

```bash
npm --prefix frontend run lint
```

For planner/runtime node behavior:

```bash
cargo test -p duckle-runner serve::tests
```

For broader runtime changes:

```bash
cargo test -p duckle-runner
```

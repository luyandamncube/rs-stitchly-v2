---
name: stitchly-v2-workflow-author
description: Design, document, create, or debug Stitchly v2 workflow graphs and workflow JSON. Use when Codex needs to build workflows from nodes and edges, specify node configs, design idempotent or incremental pipelines, create Dolt-to-Parquet workflows, reason about DuckDB execution, or produce workflow docs for the Stitchly v2/Duckle fork.
---

# Stitchly v2 Workflow Author

Use this skill to produce workflow plans and workflow JSON that can be executed by the Stitchly v2 runtime. Favor concrete node configs, explicit graph edges, idempotency, and testable verification steps.

## First Steps

1. Find the repo root. Expected root markers include `Cargo.toml`, `frontend/package.json`, `docs/02_workflows`, and `docs/01_nodes`.
2. Read `references/workflow-doc-map.md`, then open only the repo docs relevant to the requested workflow.
3. Identify the workflow type: local file, API ingestion, database ingestion, CDC/incremental, Dolt/Parquet, dbt/modeling, AI/RAG, orchestration, or migration from v1.
4. Identify runtime mode for validation: usually `ui-http` for real browser execution, `duckle-runner serve` for endpoint checks, or direct tests for planner/runtime changes.

## Workflow Design Rules

- Define workflow name, purpose, inputs, outputs, state, and failure behavior before writing JSON.
- Use existing node types and contracts from `docs/01_nodes` unless the user asks to design a new node.
- Keep each workflow idempotent where practical. Prefer watermarks, manifests, content hashes, partitioned Parquet output, and merge/upsert semantics over blind full refreshes.
- Prefer Parquet for durable intermediate and final data when the user has not specified another sink.
- Use shell/code nodes only when they materially simplify external tooling, and document expected side effects.
- Include validation commands or endpoint checks alongside any workflow artifact.

## Output Shape

When planning a workflow, include:

- Workflow name.
- Node table with id, type, purpose, config, inputs, and outputs.
- Edge table or explicit source-to-target mapping.
- State/idempotency strategy.
- Expected files/artifacts.
- Manual and automated validation steps.

When creating workflow JSON, keep IDs stable and descriptive. Preserve the app's graph shape: a `nodes` array and an `edges` array.

## Verification

For docs-only workflow design, verify references and internal consistency.

For executable workflow changes, prefer:

```bash
npm --prefix frontend run lint
cargo test -p duckle-runner serve::tests
```

For HTTP runtime execution, check:

```bash
curl http://127.0.0.1:8080/api/studio/health
```

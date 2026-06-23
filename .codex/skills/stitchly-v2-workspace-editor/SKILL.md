---
name: stitchly-v2-workspace-editor
description: Inspect and edit file-backed Stitchly v2/Duckle workspaces directly. Use when Codex needs to list saved pipelines, read or patch duckle.json, repository.json, pipelines/*.json, connections/contexts/routines/docs, rename or duplicate pipelines, patch node configs, add/remove/rewire graph edges, validate workspace references, find orphaned files, or make safe JSON edits in a repo-local workspace.
---

# Stitchly v2 Workspace Editor

Use this skill when working directly with saved workspace files rather than only the live UI. Treat the workspace JSON as user data: inspect first, preserve unrelated fields, and make small reversible edits.

## Default Locations

- Repo root usually contains `Cargo.toml`, `frontend/package.json`, `crates/duckle-runner`, and `stitchly_workspace/`.
- Preferred repo-local workspace: `stitchly_workspace/`.
- Workspace layout:
  - `duckle.json`: workspace metadata, jobs, active job id.
  - `repository.json`: project tree and item names.
  - `pipelines/<id>.json`: ReactFlow nodes and edges for each pipeline.
  - `connections/`, `contexts/`, `routines/`, `docs/`: optional payload folders.

## Workflow

1. Locate the workspace. Prefer an explicit user path; otherwise check `stitchly_workspace/`, then search for `duckle.json` and `repository.json`.
2. List pipelines from `repository.json` and confirm matching `pipelines/<id>.json` files exist.
3. Read the target pipeline JSON before editing. Summarize node ids, labels, `componentId`, properties, and edges.
4. For edits, patch the smallest JSON section needed. Preserve positions, measured dimensions, selected/dragging flags, edge ids, and unknown fields unless the user asks to change them.
5. Keep `duckle.json.jobs`, `repository.json`, and `pipelines/*.json` consistent when renaming, duplicating, deleting, or adding pipelines.
6. Validate after edits by parsing JSON and checking references.

## Node JSON Shape

For canvas-rendered workflow nodes, use only node `type` values that the frontend registers: `source`, `transform`, or `sink`. Control components such as `ctl.switch`, `ctl.log`, and `ctl.die` should still use node `type: "transform"` with their control behavior represented by `data.componentId`. Do not write node `type: "control"`; React Flow will render it as a placeholder unless the UI registers that type.

## Common Tasks

- List pipelines:
  - read `repository.json` items with `type = "pipeline"`.
  - join each id to `pipelines/<id>.json`.
- Rename a pipeline:
  - update `repository.json` item `name`.
  - update matching `duckle.json.jobs[].name`.
- Patch a node config:
  - edit `pipelines/<id>.json` node `data.properties`.
  - do not edit `data.schema` or `data.sampleRows` unless requested.
- Duplicate a pipeline:
  - create a new stable id, copy `pipelines/<old>.json` to `pipelines/<new>.json`.
  - add a repository item under the same folder.
  - add a `duckle.json.jobs` entry.
- Validate references:
  - every repository pipeline id should have `pipelines/<id>.json`.
  - every pipeline file should have a repository item unless intentionally archived.
  - every edge source/target should match a node id.

## Tools

Use `jq` when available for inspection, but prefer structured parsing over string edits. Use `rg --files` and `rg` for discovery. For repo edits, use `apply_patch` when possible. Do not delete workspace files without explicit user approval.

## Verification

For JSON-only edits:

```bash
python -m json.tool stitchly_workspace/duckle.json >/dev/null
python -m json.tool stitchly_workspace/repository.json >/dev/null
python -m json.tool stitchly_workspace/pipelines/<id>.json >/dev/null
```

For UI/runtime impacts, also run:

```bash
npm --prefix frontend run lint
```

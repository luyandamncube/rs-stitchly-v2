---
name: stitchly-v2-dolt-pipelines
description: Build, document, and debug Stitchly v2 Dolt-to-Parquet workflows. Use when Codex works on DoltHub repositories, repo_config, sync_repo, parse_sync_result, skip gates, plan_exports, export_tables_to_stage, parse_export_result, validate_exports, publish_and_update_state, state DuckDB, Parquet artifacts, idempotent reruns, multi-table exports, or Dolt workflow scripts under docs/dolt_scripts.
---

# Stitchly v2 Dolt Pipelines

Use this skill for Dolt sync workflows that clone/pull DoltHub repos, export Dolt tables to Parquet, publish artifacts, and track processed commits in DuckDB state.

## Canonical Docs And Scripts

Start with:

- `docs/02_workflows/13_dolt-sync-pipeline-runbook.md`
- `docs/dolt_scripts/repo_config.sql` or repo-specific `docs/dolt_scripts/repo_config_<repo_key>.sql`
- `docs/dolt_scripts/sync_repo.sh`
- `docs/dolt_scripts/parse_sync_result.sh`
- `docs/dolt_scripts/plan_exports.sh`
- `docs/dolt_scripts/parse_export_plan.sql`
- `docs/dolt_scripts/export_tables_to_stage.sh`
- `docs/dolt_scripts/parse_export_result.sql`
- `docs/dolt_scripts/validate_exports.sql`
- `docs/dolt_scripts/publish_and_export_state.sh`

## Script Source Of Truth

Treat `docs/dolt_scripts/*` as the canonical source for Dolt node bodies. When editing one of these scripts or SQL files, also propagate the changed body into every affected runnable pipeline JSON under the active workspace, usually `stitchly_workspace/pipelines/*.json`.

Use this mapping unless the user specifies a different node name:

- `repo_config.sql` or `repo_config_<repo_key>.sql` -> `repo_config` SQL/source node config. Repo config is intentionally repo-specific; shared downstream scripts should stay common.
- `sync_repo.sh` -> `sync_repo` shell node config.
- `parse_sync_result.sh` -> `parse_sync_result` SQL node config.
- `assert_sync_ok.sql` -> `assert_sync_ok` SQL node config.
- `export_gate.sql` -> `export_gate` SQL node config.
- `plan_exports.sh` -> `plan_exports` shell node config.
- `parse_export_plan.sql` -> `parse_export_plan` SQL node config.
- `export_tables_to_stage.sh` -> `export_tables_to_stage` shell node config.
- `parse_export_result.sql` -> `parse_export_result` SQL node config.
- `validate_exports.sql` -> `validate_exports` SQL/QA node config.
- `publish_and_export_state.sh` -> `publish_and_update_state` shell node config.

When propagating, preserve pipeline IDs, node IDs, positions, edges, labels, and unrelated config. Patch only the relevant script/query/config field. For copied Dolt pipelines, update all matching Dolt pipelines unless the user asks for a single pipeline.

After propagation, validate both layers:

```bash
sh -n docs/dolt_scripts/sync_repo.sh
sh -n docs/dolt_scripts/export_tables_to_stage.sh
sh -n docs/dolt_scripts/publish_and_export_state.sh
python -m json.tool stitchly_workspace/pipelines/<pipeline_id>.json >/dev/null
```

## Workflow JSON Shape

When materializing this workflow directly into `pipelines/*.json`, represent `ctl.switch`, `ctl.die`, and `ctl.log` nodes with node `type: "transform"`. Keep the control behavior in `data.componentId`. The current canvas registers `source`, `transform`, and `sink` node renderers; `type: "control"` renders as a React Flow placeholder.

## Expected Workflow

Use a linear row-gated graph. `ctl.switch` is a data splitter, not an execution gate; downstream stages still execute even when a branch relation has zero rows.

```text
repo_config
  -> sync_repo
  -> parse_sync_result
  -> assert_sync_ok
  -> export_gate
  -> plan_exports
  -> parse_export_plan
  -> export_tables_to_stage
  -> parse_export_result
  -> validate_exports
  -> publish_and_update_state
  -> log_done
```

`assert_sync_ok` should fail with SQL `error(...)` when `sync_ok=false`. `export_gate` should pass only rows where `sync_ok=true` and `should_skip=false`. On unchanged commits, the gate emits zero rows and every downstream Dolt shell/parse/publish node must treat zero rows or empty stdout with exit code 0 as a successful no-op.

Expected rerun after successful publish:

```text
previous_commit = head_commit
should_skip = true
sync_status = unchanged
export_gate rows = 0
```

## State And Artifacts

Default state DB:

```text
.stitchly/state/dolt_sync.duckdb
```

Default artifact layout:

```text
artifacts/dolt/<repo_key>/<branch>/<table_name>/snapshots/commit=<commit>/data.parquet
```

Default local repo cache:

```text
.stitchly/cache/dolt/<repo_key>/repo
```

## Known Failure Checks

- If `export_tables_to_stage` runs on a rerun, inspect `parse_sync_result.previous_commit`, `head_commit`, and `should_skip`.
- If state rows exist manually but `previous_commit` is blank, verify DuckDB CLI argument ordering in `sync_repo`: prefer `duckdb "$state_db" -csv -c "..."`.
- If only the first table exports, ensure shell scripts loop over all upstream rows and `parse_export_result.sql` splits newline-delimited JSON stdout.
- If `plan_exports: upstream sync did not succeed`, inspect `assert_sync_ok` and `export_gate`; do not reintroduce `ctl.switch` as an execution gate.

## Dolt Run Triage

When inspecting a Dolt pipeline run, use the runtime run-history procedure first, then add Dolt-specific checks:

- Query `<workspace>/.stitchly/state/dolt_sync.duckdb` for `repo_key`, `branch`, `table_name`, `row_count`, `last_processed_commit`, and `updated_at`.
- Check published artifacts under `<workspace>/artifacts/dolt/<repo_key>/...` when the workspace-local artifact root is used.
- If `log_skip` and `fail_sync` both run, or export nodes run on an unchanged commit, suspect an old switch-gated workflow. Prefer the linear `assert_sync_ok -> export_gate` pattern.
- Compare `parse_sync_result` fields when available: `previous_commit`, `head_commit`, `should_skip`, `sync_ok`, and `sync_status`.

## Verification

Query state:

```bash
duckdb .stitchly/state/dolt_sync.duckdb -c "
select repo_key, branch, table_name, row_count, last_processed_commit, updated_at
from dolt_sync
where repo_key = '<repo_key>'
order by updated_at desc, table_name;
"
```

Check artifacts:

```bash
find artifacts/dolt/<repo_key> -name data.parquet -type f | sort
```

For script syntax:

```bash
sh -n docs/dolt_scripts/sync_repo.sh
sh -n docs/dolt_scripts/export_tables_to_stage.sh
sh -n docs/dolt_scripts/publish_and_export_state.sh
```

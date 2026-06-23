# Project Skills

This repo carries Stitchly-specific Codex skills under `.codex/skills`.

The repo copy is the source-controlled project version. Codex loads skills from the local Codex home, usually `~/.codex/skills`, so a new machine should sync the repo copy into that local cache before starting a new Codex session.

## Sync

From the repo root:

```bash
./scripts/sync_codex_skills.sh
```

If your Codex home is not `~/.codex`, set `CODEX_HOME`:

```bash
CODEX_HOME=/path/to/codex-home ./scripts/sync_codex_skills.sh
```

Start a new Codex session after syncing so the skill list is refreshed.

## Skills

| Skill | Purpose |
| --- | --- |
| `stitchly-v2-dolt-pipelines` | Build, debug, and document Dolt-to-Parquet pipelines, scripts, state tables, and idempotent reruns. |
| `stitchly-v2-node-author` | Add or debug Stitchly/Duckle node types, node contracts, planner/runtime behavior, and node docs. |
| `stitchly-v2-runtime` | Start and debug local runtime modes, HTTP bridge, DuckDB CLI paths, history, logs, and local verification. |
| `stitchly-v2-ui-author` | Change frontend UI, including node config panels, JSON tabs, run output accordions, previews, and workspace UI. |
| `stitchly-v2-workflow-author` | Design or create workflow graphs, node configs, edges, and workflow documentation. |
| `stitchly-v2-workspace-editor` | Inspect and edit file-backed workspaces, pipeline JSON, repository metadata, and graph wiring directly. |
| `stitchly-v2-workspace-sync` | Move, localize, configure, and sync workspace folders across machines. |

## Maintenance

When a skill changes locally during development, update the repo copy under `.codex/skills/<skill-name>` and validate it before committing.

Useful validation:

```bash
python /home/mncubel/.codex/skills/.system/skill-creator/scripts/quick_validate.py .codex/skills/<skill-name>
```

The sync script copies files into the local Codex cache but does not remove skills that exist only in the cache.

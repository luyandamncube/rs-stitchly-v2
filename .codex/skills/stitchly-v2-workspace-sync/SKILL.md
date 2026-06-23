---
name: stitchly-v2-workspace-sync
description: Move, localize, configure, and sync Stitchly v2/Duckle workspace folders across machines. Use when Codex needs to find the real workspace, copy ~/duckle_ws into a repo-local stitchly_workspace, set HTTP bridge --workspace paths, update browser localStorage/account workspace paths, document startup commands, decide what to git-track or ignore, or troubleshoot workspace button/load/save behavior in browser+HTTP mode.
---

# Stitchly v2 Workspace Sync

Use this skill when the task is about where Stitchly stores pipelines and how to make those files portable across machines.

## Preferred Layout

Use a repo-local workspace for syncable definitions:

```text
stitchly_workspace/
  duckle.json
  repository.json
  pipelines/
  connections/
  contexts/
  routines/
  docs/
```

Keep generated runtime data separate unless the user explicitly wants to version it:

```text
.stitchly/
artifacts/
runs/
logs/
```

## Find The Real Workspace

Search for known pipeline ids or workspace metadata:

```bash
find /home/mncubel -path '*/pipelines/*.json' -type f -print
find /home/mncubel -name duckle.json -o -name repository.json
```

Search by content:

```bash
find /home/mncubel -path '*/pipelines/*.json' -type f -print0   | xargs -0 grep -l 'post-no-preference\|earnings\|rates\|dolt'
```

## Move Workspace Into Repo

Copy without deleting the original:

```bash
mkdir -p stitchly_workspace
cp -a /home/mncubel/duckle_ws/. stitchly_workspace/
mkdir -p stitchly_workspace/connections stitchly_workspace/contexts stitchly_workspace/routines stitchly_workspace/docs
```

Then inspect:

```bash
find stitchly_workspace -maxdepth 3 -type f | sort
```

## Start Browser + HTTP Runtime

Backend:

```bash
cargo run -p duckle-runner -- serve   --host 127.0.0.1   --port 8080   --workspace /home/mncubel/rs-stitchly-v2/stitchly_workspace   --duckdb /snap/bin/duckdb
```

Frontend:

```bash
VITE_DUCKLE_BACKEND=http VITE_DUCKLE_HTTP_URL=http://127.0.0.1:8080 npm --prefix frontend run dev
```

## Browser Workspace State

Browser mode stores the selected workspace path in localStorage and may also store it on the active local account:

```js
const path = '/home/mncubel/rs-stitchly-v2/stitchly_workspace';
localStorage.setItem('duckle:workspace-path', path);
const active = localStorage.getItem('duckle:v1:active-account');
const accounts = JSON.parse(localStorage.getItem('duckle:v1:accounts') || '[]');
for (const acc of accounts) if (!active || acc.id === active) acc.workspacePath = path;
localStorage.setItem('duckle:v1:accounts', JSON.stringify(accounts));
location.reload();
```

## Troubleshooting

- If the top workspace button does nothing, check whether browser+HTTP mode has workspace bridge support; Tauri-only folder pickers do not work in plain browser mode.
- If Codex sees only old pipelines, inspect `stitchly_workspace/repository.json` and `stitchly_workspace/pipelines/*.json`; the UI may have saved browser localStorage state into the repo-local workspace.
- If run history exists but workspace files do not, the pipeline may have lived only in browser state or another workspace folder.

## Verification

```bash
python -m json.tool stitchly_workspace/duckle.json >/dev/null
python -m json.tool stitchly_workspace/repository.json >/dev/null
find stitchly_workspace/pipelines -name '*.json' -exec python -m json.tool {} >/dev/null \;
```

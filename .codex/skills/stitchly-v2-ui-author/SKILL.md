---
name: stitchly-v2-ui-author
description: Implement, review, or debug Stitchly v2 frontend UI changes in the Duckle fork. Use when Codex needs to edit React/TypeScript UI files, properties panels, node config tabs, JSON config editors, bottom output accordions, preview tables, workspace picker behavior, browser HTTP mode UI, i18n labels, CSS layout, or frontend verification in frontend/src.
---

# Stitchly v2 UI Author

Use this skill for frontend changes in `frontend/src`. Preserve existing UI patterns and keep changes scoped to the requested surface.

## First Steps

1. Inspect the component and nearby styles before editing.
2. Identify whether the UI runs in browser HTTP mode, Tauri mode, or both.
3. Keep state updates aligned with existing data contracts such as `DuckleNodeData`, `RunResult`, and workspace state.
4. Prefer existing helpers: `copyText`, `tauri-bridge`, `workspace.ts`, field renderers, manifests, and existing preview table styles.

## Common Surfaces

- Node config panel: `frontend/src/workflow-ui/PropertiesPanel.tsx`.
- Bottom output/debug panel: `frontend/src/workflow-ui/BottomPanel.tsx`.
- Workspace/file handling: `frontend/src/App.tsx`, `frontend/src/workspace.ts`, `frontend/src/tauri-bridge.ts`.
- Styles: `frontend/src/styles.css`.
- English labels: `frontend/src/i18n/locales/en.json`; use `defaultValue` for new labels when broad locale churn is not needed.

## UI Rules

- Keep buttons icon-first when the action is familiar; use lucide icons already present in the app.
- Avoid nested cards. Embedded tables inside accordions should not add redundant outer borders.
- Preserve stable dimensions for rows, buttons, tabs, and fixed tool surfaces.
- Do not use hero/landing-page patterns for operational UI.
- Keep text small and scannable in dense panels.

## Patterns From Recent Work

- JSON config tab edits `selected.data.properties` only, not full node graph structure.
- JSON editors should validate object JSON before applying and avoid mutating state while invalid.
- Bottom output rows can be accordions. Reuse `PreviewTable` for captured node previews, and show explicit no-preview state for shell/control/sink nodes.
- Browser HTTP workspace mode needs explicit bridge support; do not assume Tauri fs/dialog APIs exist in the browser.

## Verification

For UI-only changes:

```bash
npm --prefix frontend run lint
```

For HTTP bridge-related UI changes:

```bash
cargo test -p duckle-runner serve::tests
```

If a dev server is needed and not already running:

```bash
VITE_DUCKLE_BACKEND=http VITE_DUCKLE_HTTP_URL=http://127.0.0.1:8080 npm --prefix frontend run dev
```

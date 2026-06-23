# UI File Map

## Properties Panel

`frontend/src/workflow-ui/PropertiesPanel.tsx`

Used for node names, manifest-driven fields, schema, preview, advanced, validation, and JSON config tabs.

## Bottom Output

`frontend/src/workflow-ui/BottomPanel.tsx`

Used for problems, run output, console, node result rows, accordions, and preview tables.

## Runtime Transport

`frontend/src/tauri-bridge.ts`

Selects `mock`, `http`, or `tauri` backend based on `VITE_DUCKLE_BACKEND` or Tauri detection.

## Workspace

`frontend/src/workspace.ts`

Handles Tauri file-backed workspace logic and browser HTTP workspace bridge calls.

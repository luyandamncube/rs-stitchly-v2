# Workspace JSON Reference

## duckle.json

Expected keys: `version`, `engine`, `jobs`, `activeJobId`.

`jobs` entries use `{ id, name, dirty }`. Pipeline ids should match `repository.json` and `pipelines/<id>.json`.

## repository.json

Array of tree items. Common fields: `id`, `name`, `type`, `parentId`, `icon`. Pipeline items use `type: "pipeline"`.

Core folders normally include `pipelines`, `connections`, `contexts`, `routines`, and `docs` under `root`.

## pipelines/<id>.json

Top-level shape is usually `{ nodes, edges }`.

Node shape follows ReactFlow:

```json
{
  "id": "n_...",
  "type": "transform",
  "position": { "x": 0, "y": 0 },
  "data": {
    "label": "Inline SQL",
    "componentId": "code.sql",
    "properties": {}
  }
}
```

Edge shape usually includes `source`, `sourceHandle`, `target`, `targetHandle`, `type: "duckle"`, and `data.connectionType`.

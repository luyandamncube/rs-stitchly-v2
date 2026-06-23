# Workspace Sync Checklist

1. Identify current runtime workspace from HTTP bridge health:

```bash
curl http://127.0.0.1:8080/api/studio/health
```

2. Identify saved UI workspace from browser localStorage keys:

- `duckle:workspace-path`
- `duckle:v1:accounts`
- `duckle:v1:active-account`

3. Ensure `duckle.json.jobs` matches `repository.json` pipeline items.
4. Ensure every pipeline item has `pipelines/<id>.json`.
5. Commit only syncable definitions unless artifacts are intentionally versioned.

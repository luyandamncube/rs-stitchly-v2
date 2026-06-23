# Dolt Repo Config Reference

Common repos:

| repo_key | remote_url |
|---|---|
| earnings | post-no-preference/earnings |
| rates | post-no-preference/rates |
| stocks | post-no-preference/stocks |
| options | post-no-preference/options |

Typical config row:

```json
{
  "repo_key": "earnings",
  "remote_url": "post-no-preference/earnings",
  "branch": "master",
  "cache_root": ".stitchly/cache/dolt",
  "state_db": ".stitchly/state/dolt_sync.duckdb",
  "artifact_root": "artifacts/dolt",
  "force_snapshot": false
}
```

Use `post-no-preference/<repo>` for `dolt clone`; do not use the DoltHub web URL.

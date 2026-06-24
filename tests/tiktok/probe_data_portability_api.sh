#!/usr/bin/env bash
set -u

OUT="$OUT_DIR/data_portability.jsonl"
: > "$OUT"

cat >> "$OUT" <<'JSON'
{"mechanism":"TikTok Data Portability API","status":"not_probed","reason":"Export request creation is user/account-scoped and should not be triggered by a generic smoke test. Keep this mechanism only for an explicit own-account export workflow."}
JSON

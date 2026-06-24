#!/usr/bin/env bash
set -u

OUT="$OUT_DIR/research_api_response.json"

if [ -z "${TIKTOK_RESEARCH_ACCESS_TOKEN:-}" ]; then
  printf '{"mechanism":"TikTok Research API","status":"skipped","reason":"TIKTOK_RESEARCH_ACCESS_TOKEN not set"}\n' > "$OUT"
  exit 0
fi

if ! command -v curl >/dev/null 2>&1; then
  printf '{"mechanism":"TikTok Research API","status":"skipped","reason":"curl not found"}\n' > "$OUT"
  exit 0
fi

query="${TIKTOK_RESEARCH_QUERY:-tiktok}"

curl -sS -L \
  -X POST "https://open.tiktokapis.com/v2/research/video/query/?fields=id,video_description,create_time,region_code,share_count,view_count,like_count,comment_count" \
  -H "Authorization: Bearer ${TIKTOK_RESEARCH_ACCESS_TOKEN}" \
  -H "Content-Type: application/json" \
  -d "{\"query\":{\"and\":[{\"operation\":\"IN\",\"field_name\":\"keyword\",\"field_values\":[\"${query}\"]}]},\"max_count\":10}" \
  -o "$OUT"

#!/usr/bin/env bash
set -u

OUT="$OUT_DIR/display_api_response.json"

if [ -z "${TIKTOK_ACCESS_TOKEN:-}" ]; then
  printf '{"mechanism":"TikTok Display API","status":"skipped","reason":"TIKTOK_ACCESS_TOKEN not set"}\n' > "$OUT"
  exit 0
fi

if ! command -v curl >/dev/null 2>&1; then
  printf '{"mechanism":"TikTok Display API","status":"skipped","reason":"curl not found"}\n' > "$OUT"
  exit 0
fi

curl -sS -L \
  -X POST "https://open.tiktokapis.com/v2/video/list/?fields=id,title,video_description,duration,cover_image_url,embed_link,share_url" \
  -H "Authorization: Bearer ${TIKTOK_ACCESS_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{"max_count":20}' \
  -o "$OUT"

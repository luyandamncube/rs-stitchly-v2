#!/usr/bin/env bash
set -euo pipefail

# shellcheck source=docs/tiktok/tiktok_common.sh
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/tiktok_common.sh"

require_cmd curl
require_cmd jq

access_token="$(token_access_token)"
if [ -z "$access_token" ]; then
  echo "missing access token in $TIKTOK_TOKEN_FILE" >&2
  exit 1
fi

max_count="${TIKTOK_VIDEO_LIST_MAX_COUNT:-10}"
fields="id,title,cover_image_url,share_url,embed_link,duration,create_time"
url="$TIKTOK_VIDEO_LIST_URL?fields=$(url_encode "$fields")"
response="$TIKTOK_OUT_DIR/video_list_response.json"

curl -sS -L \
  --request POST "$url" \
  --header "Authorization: Bearer $access_token" \
  --header "Content-Type: application/json" \
  --data "{\"max_count\":$max_count}" \
  -o "$response"

jq . "$response"

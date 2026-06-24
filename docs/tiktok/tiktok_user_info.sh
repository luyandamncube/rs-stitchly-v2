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

fields="open_id,union_id,avatar_url,display_name"
url="$TIKTOK_USER_INFO_URL?fields=$(url_encode "$fields")"
response="$TIKTOK_OUT_DIR/user_info_response.json"

curl -sS -L \
  --request GET "$url" \
  --header "Authorization: Bearer $access_token" \
  -o "$response"

jq . "$response"

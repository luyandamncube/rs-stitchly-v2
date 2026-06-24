#!/usr/bin/env bash
set -euo pipefail

# shellcheck source=docs/tiktok/tiktok_common.sh
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/tiktok_common.sh"

require_cmd curl
require_cmd jq
require_env TIKTOK_CLIENT_KEY
require_env TIKTOK_CLIENT_SECRET

if [ ! -f "$TIKTOK_TOKEN_FILE" ]; then
  echo "token file not found: $TIKTOK_TOKEN_FILE" >&2
  exit 1
fi

refresh_token="${TIKTOK_REFRESH_TOKEN:-$(json_get "$TIKTOK_TOKEN_FILE" refresh_token)}"
if [ -z "$refresh_token" ]; then
  echo "missing refresh token" >&2
  exit 1
fi

response="$TIKTOK_OUT_DIR/oauth_refresh_response.json"

curl -sS -L \
  --request POST "$TIKTOK_TOKEN_URL" \
  --header "Content-Type: application/x-www-form-urlencoded" \
  --header "Cache-Control: no-cache" \
  --data-urlencode "client_key=$TIKTOK_CLIENT_KEY" \
  --data-urlencode "client_secret=$TIKTOK_CLIENT_SECRET" \
  --data-urlencode "grant_type=refresh_token" \
  --data-urlencode "refresh_token=$refresh_token" \
  -o "$response"

if ! jq -e '.access_token' "$response" >/dev/null 2>&1; then
  echo "token refresh did not return access_token; raw response:" >&2
  cat "$response" >&2
  exit 1
fi

jq --argjson saved_at "$(date +%s)" '. + {saved_at_epoch: $saved_at}' "$response" > "$TIKTOK_TOKEN_FILE.tmp"
mv "$TIKTOK_TOKEN_FILE.tmp" "$TIKTOK_TOKEN_FILE"
chmod 600 "$TIKTOK_TOKEN_FILE"
save_token_env "$TIKTOK_TOKEN_FILE"

echo "Refreshed token file: $TIKTOK_TOKEN_FILE"

#!/usr/bin/env bash
set -euo pipefail

# shellcheck source=docs/tiktok/tiktok_common.sh
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/tiktok_common.sh"

require_cmd curl
require_cmd jq
require_env TIKTOK_CLIENT_KEY
require_env TIKTOK_CLIENT_SECRET
require_env TIKTOK_REDIRECT_URI

code="${1:-${TIKTOK_AUTH_CODE:-}}"
if [ -z "$code" ] && [ -f "$TIKTOK_OUT_DIR/oauth_callback_code.txt" ]; then
  code="$(tr -d '\r\n' < "$TIKTOK_OUT_DIR/oauth_callback_code.txt")"
fi
if [ -z "$code" ]; then
  echo "Usage: $0 '<code-from-callback-page>'" >&2
  echo "Or run tiktok_wait_for_code.sh first so $TIKTOK_OUT_DIR/oauth_callback_code.txt exists." >&2
  exit 1
fi

mode="${TIKTOK_OAUTH_MODE:-web}"
response="$TIKTOK_OUT_DIR/oauth_token_response.json"

curl_args=(
  -sS -L
  --request POST "$TIKTOK_TOKEN_URL"
  --header "Content-Type: application/x-www-form-urlencoded"
  --header "Cache-Control: no-cache"
  --data-urlencode "client_key=$TIKTOK_CLIENT_KEY"
  --data-urlencode "client_secret=$TIKTOK_CLIENT_SECRET"
  --data-urlencode "code=$code"
  --data-urlencode "grant_type=authorization_code"
  --data-urlencode "redirect_uri=$TIKTOK_REDIRECT_URI"
  -o "$response"
)

if [ "$mode" = "desktop" ]; then
  verifier="${TIKTOK_CODE_VERIFIER:-}"
  if [ -z "$verifier" ] && [ -f "$TIKTOK_OUT_DIR/oauth_code_verifier.txt" ]; then
    verifier="$(tr -d '\r\n' < "$TIKTOK_OUT_DIR/oauth_code_verifier.txt")"
  fi
  if [ -z "$verifier" ]; then
    echo "desktop mode requires TIKTOK_CODE_VERIFIER or $TIKTOK_OUT_DIR/oauth_code_verifier.txt" >&2
    exit 1
  fi
  curl_args+=(--data-urlencode "code_verifier=$verifier")
fi

curl "${curl_args[@]}"

if ! jq -e '.access_token' "$response" >/dev/null 2>&1; then
  echo "token exchange did not return access_token; raw response:" >&2
  cat "$response" >&2
  exit 1
fi

jq --argjson saved_at "$(date +%s)" '. + {saved_at_epoch: $saved_at}' "$response" > "$TIKTOK_TOKEN_FILE.tmp"
mv "$TIKTOK_TOKEN_FILE.tmp" "$TIKTOK_TOKEN_FILE"
chmod 600 "$TIKTOK_TOKEN_FILE"
save_token_env "$TIKTOK_TOKEN_FILE"

echo "Saved token file: $TIKTOK_TOKEN_FILE"
echo "Saved token env: $TIKTOK_OUT_DIR/tiktok_tokens.env"

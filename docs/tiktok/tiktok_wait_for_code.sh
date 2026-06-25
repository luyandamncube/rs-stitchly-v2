#!/usr/bin/env bash
set -euo pipefail

# shellcheck source=docs/tiktok/tiktok_common.sh
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/tiktok_common.sh"

require_cmd curl
require_cmd jq
require_env TIKTOK_CALLBACK_CODE_URL

state="${1:-${TIKTOK_OAUTH_STATE:-}}"
if [ -z "$state" ] && [ -f "$TIKTOK_OUT_DIR/oauth_state.txt" ]; then
  state="$(tr -d '\r\n' < "$TIKTOK_OUT_DIR/oauth_state.txt")"
fi

if [ -z "$state" ]; then
  echo "missing state; run tiktok_make_auth_url.sh first or pass state as the first argument" >&2
  exit 1
fi

timeout_seconds="${TIKTOK_CODE_POLL_TIMEOUT_SECONDS:-180}"
interval_seconds="${TIKTOK_CODE_POLL_INTERVAL_SECONDS:-3}"
deadline="$(( $(date +%s) + timeout_seconds ))"
response="$TIKTOK_OUT_DIR/oauth_callback_code_response.json"
code_file="$TIKTOK_OUT_DIR/oauth_callback_code.txt"

echo "Waiting for OAuth code for state: $state"
echo "Polling: $TIKTOK_CALLBACK_CODE_URL"

while [ "$(date +%s)" -le "$deadline" ]; do
  curl -sS -L \
    --get "$TIKTOK_CALLBACK_CODE_URL" \
    --data-urlencode "state=$state" \
    --data-urlencode "consume=true" \
    -o "$response"

  if ! jq empty "$response" >/dev/null 2>&1; then
    echo "callback code endpoint did not return JSON: $TIKTOK_CALLBACK_CODE_URL" >&2
    cat "$response" >&2
    exit 1
  fi

  if jq -e '.code and (.code | length > 0)' "$response" >/dev/null 2>&1; then
    jq -r '.code' "$response" > "$code_file"
    chmod 600 "$code_file"
    echo "Received code: $code_file"
    exit 0
  fi

  if jq -e '.error' "$response" >/dev/null 2>&1; then
    error="$(jq -r '.error' "$response")"
    if [ "$error" != "not_found" ]; then
      echo "callback code endpoint returned error:" >&2
      cat "$response" >&2
      exit 1
    fi
  fi

  sleep "$interval_seconds"
done

echo "timed out waiting for OAuth code after ${timeout_seconds}s" >&2
exit 1

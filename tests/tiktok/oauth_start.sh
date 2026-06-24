#!/usr/bin/env bash
set -u

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUT_DIR="${OUT_DIR:-$ROOT/out}"
ENV_FILE="${ENV_FILE:-$ROOT/.env}"

mkdir -p "$OUT_DIR"

if [ -f "$ENV_FILE" ]; then
  set -a
  # shellcheck disable=SC1090
  source "$ENV_FILE"
  set +a
fi

if [ -z "${TIKTOK_CLIENT_KEY:-}" ]; then
  echo "missing TIKTOK_CLIENT_KEY in $ENV_FILE" >&2
  exit 1
fi

redirect_uri="${TIKTOK_REDIRECT_URI:-}"
if [ -z "$redirect_uri" ]; then
  echo "missing TIKTOK_REDIRECT_URI in $ENV_FILE" >&2
  exit 1
fi

scopes="${TIKTOK_SCOPES:-user.info.basic,video.list}"
state="${TIKTOK_OAUTH_STATE:-}"
if [ -z "$state" ]; then
  if command -v openssl >/dev/null 2>&1; then
    state="$(openssl rand -hex 16)"
  else
    state="stitchly-$(date +%s)-$$"
  fi
fi

if ! command -v python3 >/dev/null 2>&1; then
  echo "python3 is required for URL encoding" >&2
  exit 1
fi

url="$(
  python3 - "$TIKTOK_CLIENT_KEY" "$redirect_uri" "$scopes" "$state" <<'PY'
import sys
from urllib.parse import urlencode

client_key, redirect_uri, scopes, state = sys.argv[1:5]
params = {
    "client_key": client_key,
    "response_type": "code",
    "scope": scopes,
    "redirect_uri": redirect_uri,
    "state": state,
}
print("https://www.tiktok.com/v2/auth/authorize/?" + urlencode(params))
PY
)"

printf '%s\n' "$state" > "$OUT_DIR/oauth_state.txt"
printf '%s\n' "$url" > "$OUT_DIR/oauth_authorize_url.txt"

cat <<EOF
Open this URL in a browser:

$url

Expected callback:
$redirect_uri?code=...&state=$state

After approving, copy the callback 'code' into tests/tiktok/.env:

TIKTOK_AUTH_CODE=...

State saved to:
$OUT_DIR/oauth_state.txt
EOF

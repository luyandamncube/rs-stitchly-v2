#!/usr/bin/env bash
set -euo pipefail

# shellcheck source=docs/tiktok/tiktok_common.sh
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/tiktok_common.sh"

require_env TIKTOK_CLIENT_KEY
require_env TIKTOK_REDIRECT_URI

scopes="${TIKTOK_SCOPES:-user.info.basic,video.list}"
mode="${TIKTOK_OAUTH_MODE:-web}"

if command -v openssl >/dev/null 2>&1; then
  state="$(openssl rand -hex 16)"
else
  state="stitchly-$(date +%s)-$$"
fi

query="client_key=$(url_encode "$TIKTOK_CLIENT_KEY")"
query="$query&response_type=code"
query="$query&scope=$(url_encode "$scopes")"
query="$query&redirect_uri=$(url_encode "$TIKTOK_REDIRECT_URI")"
query="$query&state=$(url_encode "$state")"

if [ "$mode" = "desktop" ]; then
  require_cmd openssl
  verifier="$(openssl rand -base64 96 | tr -d '\n' | tr '+/' '-_' | tr -cd 'A-Za-z0-9._~-' | cut -c1-64)"
  challenge="$(printf '%s' "$verifier" | openssl dgst -sha256 -binary | od -An -tx1 | tr -d ' \n')"
  printf '%s\n' "$verifier" > "$TIKTOK_OUT_DIR/oauth_code_verifier.txt"
  query="$query&code_challenge=$(url_encode "$challenge")"
  query="$query&code_challenge_method=S256"
elif [ "$mode" != "web" ]; then
  echo "unsupported TIKTOK_OAUTH_MODE: $mode" >&2
  exit 1
fi

url="$TIKTOK_AUTH_URL?$query"

printf '%s\n' "$state" > "$TIKTOK_OUT_DIR/oauth_state.txt"
printf '%s\n' "$url" > "$TIKTOK_OUT_DIR/oauth_authorize_url.txt"

cat <<EOF
State:
$state

Open this URL:
$url

Saved URL:
$TIKTOK_OUT_DIR/oauth_authorize_url.txt
EOF

if [ "$mode" = "desktop" ]; then
  cat <<EOF

Saved desktop PKCE verifier:
$TIKTOK_OUT_DIR/oauth_code_verifier.txt
EOF
fi

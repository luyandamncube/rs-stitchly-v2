#!/usr/bin/env bash
set -euo pipefail

# shellcheck source=docs/tiktok/tiktok_common.sh
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/tiktok_common.sh"

open_browser="${TIKTOK_OPEN_BROWSER:-false}"

bash "$TIKTOK_ROOT/tiktok_make_auth_url.sh"

auth_url="$(tr -d '\r\n' < "$TIKTOK_OUT_DIR/oauth_authorize_url.txt")"
if [ "$open_browser" = "true" ]; then
  if command -v xdg-open >/dev/null 2>&1; then
    xdg-open "$auth_url" >/dev/null 2>&1 || true
  elif command -v open >/dev/null 2>&1; then
    open "$auth_url" >/dev/null 2>&1 || true
  fi
fi

cat <<EOF

Approve the TikTok login in your browser.
This script will wait for the Vercel callback postbox to receive the code.

EOF

bash "$TIKTOK_ROOT/tiktok_wait_for_code.sh"
code="$(tr -d '\r\n' < "$TIKTOK_OUT_DIR/oauth_callback_code.txt")"
bash "$TIKTOK_ROOT/tiktok_exchange_code.sh" "$code"

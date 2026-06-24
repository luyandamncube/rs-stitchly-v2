#!/usr/bin/env bash
set -euo pipefail

# shellcheck source=docs/tiktok/tiktok_common.sh
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/tiktok_common.sh"

require_cmd jq

if [ ! -f "$TIKTOK_TOKEN_FILE" ]; then
  echo "token file not found: $TIKTOK_TOKEN_FILE" >&2
  exit 1
fi

now="$(date +%s)"

jq --argjson now "$now" '
  . as $t
  | {
      token_file: "'"$TIKTOK_TOKEN_FILE"'",
      has_access_token: (($t.access_token // "") != ""),
      has_refresh_token: (($t.refresh_token // "") != ""),
      open_id_present: (($t.open_id // "") != ""),
      scope: ($t.scope // ""),
      saved_at_epoch: ($t.saved_at_epoch // null),
      access_token_expires_at_epoch: (if ($t.saved_at_epoch and $t.expires_in) then ($t.saved_at_epoch + $t.expires_in) else null end),
      refresh_token_expires_at_epoch: (if ($t.saved_at_epoch and $t.refresh_expires_in) then ($t.saved_at_epoch + $t.refresh_expires_in) else null end),
      access_token_seconds_remaining: (if ($t.saved_at_epoch and $t.expires_in) then ($t.saved_at_epoch + $t.expires_in - $now) else null end),
      refresh_token_seconds_remaining: (if ($t.saved_at_epoch and $t.refresh_expires_in) then ($t.saved_at_epoch + $t.refresh_expires_in - $now) else null end)
    }
' "$TIKTOK_TOKEN_FILE" | tee "$TIKTOK_OUT_DIR/auth_status.json"

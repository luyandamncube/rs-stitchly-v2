#!/usr/bin/env bash

set -u

TIKTOK_SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TIKTOK_ROOT="$TIKTOK_SCRIPT_DIR"
TIKTOK_ENV_FILE="${TIKTOK_ENV_FILE:-$TIKTOK_ROOT/.env.tiktok}"
TIKTOK_OUT_DIR="${TIKTOK_OUT_DIR:-$TIKTOK_ROOT/out}"
TIKTOK_TOKEN_FILE="${TIKTOK_TOKEN_FILE:-$TIKTOK_OUT_DIR/tiktok_tokens.json}"

TIKTOK_AUTH_URL="https://www.tiktok.com/v2/auth/authorize/"
TIKTOK_TOKEN_URL="https://open.tiktokapis.com/v2/oauth/token/"
TIKTOK_USER_INFO_URL="https://open.tiktokapis.com/v2/user/info/"
TIKTOK_VIDEO_LIST_URL="https://open.tiktokapis.com/v2/video/list/"

if [ -f "$TIKTOK_ENV_FILE" ]; then
  set -a
  # shellcheck disable=SC1090
  source "$TIKTOK_ENV_FILE"
  set +a
fi

mkdir -p "$TIKTOK_OUT_DIR"

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "missing required command: $1" >&2
    exit 1
  fi
}

require_env() {
  eval "value=\${$1:-}"
  if [ -z "$value" ]; then
    echo "missing env var: $1" >&2
    exit 1
  fi
}

url_encode() {
  local input="${1:-}"
  local output=""
  local i char hex
  for ((i = 0; i < ${#input}; i++)); do
    char="${input:i:1}"
    case "$char" in
      [a-zA-Z0-9.~_-]) output+="$char" ;;
      *) printf -v hex '%%%02X' "'$char"; output+="$hex" ;;
    esac
  done
  printf '%s' "$output"
}

json_get() {
  local file="$1"
  local key="$2"
  require_cmd jq
  jq -r --arg key "$key" '.[$key] // empty' "$file"
}

save_token_env() {
  local token_file="$1"
  require_cmd jq
  {
    printf 'export TIKTOK_ACCESS_TOKEN=%q\n' "$(jq -r '.access_token // ""' "$token_file")"
    printf 'export TIKTOK_REFRESH_TOKEN=%q\n' "$(jq -r '.refresh_token // ""' "$token_file")"
    printf 'export TIKTOK_OPEN_ID=%q\n' "$(jq -r '.open_id // ""' "$token_file")"
    printf 'export TIKTOK_GRANTED_SCOPES=%q\n' "$(jq -r '.scope // ""' "$token_file")"
    printf 'export TIKTOK_TOKEN_TYPE=%q\n' "$(jq -r '.token_type // ""' "$token_file")"
    printf 'export TIKTOK_ACCESS_TOKEN_EXPIRES_IN=%q\n' "$(jq -r '.expires_in // ""' "$token_file")"
    printf 'export TIKTOK_REFRESH_TOKEN_EXPIRES_IN=%q\n' "$(jq -r '.refresh_expires_in // ""' "$token_file")"
  } > "$TIKTOK_OUT_DIR/tiktok_tokens.env"
}

token_access_token() {
  json_get "$TIKTOK_TOKEN_FILE" access_token
}

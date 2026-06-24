#!/usr/bin/env bash
set -u

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUT_DIR="${OUT_DIR:-$ROOT/out}"
ENV_FILE="${ENV_FILE:-$ROOT/.env}"
RESPONSE="$OUT_DIR/oauth_token_response.json"
TOKENS_ENV="$OUT_DIR/oauth_tokens.env"

mkdir -p "$OUT_DIR"

if [ -f "$ENV_FILE" ]; then
  set -a
  # shellcheck disable=SC1090
  source "$ENV_FILE"
  set +a
fi

missing=0
for key in TIKTOK_CLIENT_KEY TIKTOK_CLIENT_SECRET TIKTOK_AUTH_CODE TIKTOK_REDIRECT_URI; do
  eval "value=\${$key:-}"
  if [ -z "$value" ]; then
    echo "missing $key in $ENV_FILE" >&2
    missing=1
  fi
done

if [ "$missing" -ne 0 ]; then
  exit 1
fi

if ! command -v curl >/dev/null 2>&1; then
  echo "curl is required" >&2
  exit 1
fi

if ! command -v python3 >/dev/null 2>&1; then
  echo "python3 is required for code decoding and token parsing" >&2
  exit 1
fi

code="$(python3 - "$TIKTOK_AUTH_CODE" <<'PY'
import sys
from urllib.parse import unquote
print(unquote(sys.argv[1]))
PY
)"

curl -sS -L \
  --request POST "https://open.tiktokapis.com/v2/oauth/token/" \
  --header "Content-Type: application/x-www-form-urlencoded" \
  --header "Cache-Control: no-cache" \
  --data-urlencode "client_key=${TIKTOK_CLIENT_KEY}" \
  --data-urlencode "client_secret=${TIKTOK_CLIENT_SECRET}" \
  --data-urlencode "code=${code}" \
  --data-urlencode "grant_type=authorization_code" \
  --data-urlencode "redirect_uri=${TIKTOK_REDIRECT_URI}" \
  -o "$RESPONSE"

python3 - "$RESPONSE" "$TOKENS_ENV" <<'PY'
import json
import sys
from pathlib import Path

response_path = Path(sys.argv[1])
tokens_env_path = Path(sys.argv[2])

try:
    data = json.loads(response_path.read_text())
except json.JSONDecodeError as exc:
    print(f"invalid JSON response: {exc}", file=sys.stderr)
    sys.exit(1)

if "access_token" not in data:
    print("OAuth exchange did not return access_token. Raw response:", file=sys.stderr)
    print(json.dumps(data, indent=2), file=sys.stderr)
    sys.exit(1)

def shell_quote(value: str) -> str:
    return "'" + value.replace("'", "'\"'\"'") + "'"

lines = [
    f"TIKTOK_ACCESS_TOKEN={shell_quote(data.get('access_token', ''))}",
    f"TIKTOK_REFRESH_TOKEN={shell_quote(data.get('refresh_token', ''))}",
    f"TIKTOK_TOKEN_TYPE={shell_quote(data.get('token_type', ''))}",
    f"TIKTOK_OPEN_ID={shell_quote(data.get('open_id', ''))}",
    f"TIKTOK_GRANTED_SCOPES={shell_quote(data.get('scope', ''))}",
    f"TIKTOK_ACCESS_TOKEN_EXPIRES_IN={data.get('expires_in', '')}",
    f"TIKTOK_REFRESH_TOKEN_EXPIRES_IN={data.get('refresh_expires_in', '')}",
]
tokens_env_path.write_text("\n".join(lines) + "\n")

print(f"wrote raw response: {response_path}")
print(f"wrote token snippet: {tokens_env_path}")
print("")
print("Copy these lines into tests/tiktok/.env:")
print("")
for line in lines[:2]:
    print(line)
PY

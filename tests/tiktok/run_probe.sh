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

export ROOT OUT_DIR
export INPUT_URLS="${INPUT_URLS:-$ROOT/input_urls.txt}"
export ENABLE_MEDIA_DOWNLOAD="${ENABLE_MEDIA_DOWNLOAD:-false}"

: > "$OUT_DIR/errors.jsonl"

run_step() {
  local name="$1"
  local script="$2"
  echo "== $name"
  if ! bash "$script"; then
    printf '{"mechanism":"%s","status":"error","message":"probe script failed"}\n' "$name" >> "$OUT_DIR/errors.jsonl"
  fi
}

run_step "yt-dlp metadata" "$ROOT/probe_ytdlp_metadata.sh"
run_step "yt-dlp download" "$ROOT/probe_ytdlp_download.sh"
run_step "TikTok oEmbed" "$ROOT/probe_oembed.sh"
run_step "TikTok Display API" "$ROOT/probe_display_api.sh"
run_step "TikTok Research API" "$ROOT/probe_research_api.sh"
run_step "TikTok Data Portability API" "$ROOT/probe_data_portability_api.sh"

if command -v python3 >/dev/null 2>&1; then
  python3 "$ROOT/write_report.py"
else
  echo "python3 not found; skipped report generation" >&2
fi

echo "done: $OUT_DIR"

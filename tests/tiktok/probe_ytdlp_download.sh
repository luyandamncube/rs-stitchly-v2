#!/usr/bin/env bash
set -u

OUT="$OUT_DIR/yt_dlp_download.jsonl"
: > "$OUT"

if [ "${ENABLE_MEDIA_DOWNLOAD:-false}" != "true" ]; then
  printf '{"mechanism":"yt-dlp download","status":"skipped","reason":"ENABLE_MEDIA_DOWNLOAD is not true"}\n' >> "$OUT"
  exit 0
fi

if ! command -v yt-dlp >/dev/null 2>&1; then
  printf '{"mechanism":"yt-dlp download","status":"skipped","reason":"yt-dlp not found"}\n' >> "$OUT_DIR/errors.jsonl"
  exit 0
fi

if [ ! -f "$INPUT_URLS" ]; then
  printf '{"mechanism":"yt-dlp download","status":"skipped","reason":"input_urls.txt not found"}\n' >> "$OUT_DIR/errors.jsonl"
  exit 0
fi

mkdir -p "$OUT_DIR/downloads"

while IFS= read -r url || [ -n "$url" ]; do
  case "$url" in
    ""|\#*) continue ;;
  esac

  err="$OUT_DIR/yt_dlp_download.tmp.err"
  if yt-dlp \
    --no-playlist \
    --write-info-json \
    --write-thumbnail \
    --paths "$OUT_DIR/downloads" \
    "$url" > /dev/null 2> "$err"; then
    safe_url="$(printf '%s' "$url" | sed 's/"/\\"/g')"
    printf '{"mechanism":"yt-dlp download","status":"ok","url":"%s"}\n' "$safe_url" >> "$OUT"
  else
    msg="$(tr '\n' ' ' < "$err" | sed 's/"/\\"/g')"
    safe_url="$(printf '%s' "$url" | sed 's/"/\\"/g')"
    printf '{"mechanism":"yt-dlp download","status":"error","url":"%s","message":"%s"}\n' "$safe_url" "$msg" >> "$OUT_DIR/errors.jsonl"
  fi
  rm -f "$err"
done < "$INPUT_URLS"

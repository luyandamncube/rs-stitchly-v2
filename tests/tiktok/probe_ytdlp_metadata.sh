#!/usr/bin/env bash
set -u

OUT="$OUT_DIR/yt_dlp_metadata.jsonl"
: > "$OUT"

if ! command -v yt-dlp >/dev/null 2>&1; then
  printf '{"mechanism":"yt-dlp metadata","status":"skipped","reason":"yt-dlp not found"}\n' >> "$OUT_DIR/errors.jsonl"
  exit 0
fi

if [ ! -f "$INPUT_URLS" ]; then
  printf '{"mechanism":"yt-dlp metadata","status":"skipped","reason":"input_urls.txt not found"}\n' >> "$OUT_DIR/errors.jsonl"
  exit 0
fi

while IFS= read -r url || [ -n "$url" ]; do
  case "$url" in
    ""|\#*) continue ;;
  esac

  tmp="$OUT_DIR/yt_dlp_metadata.tmp.json"
  err="$OUT_DIR/yt_dlp_metadata.tmp.err"
  if yt-dlp --dump-json --no-playlist --ignore-errors --no-warnings "$url" > "$tmp" 2> "$err" && [ -s "$tmp" ]; then
    cat "$tmp" >> "$OUT"
    printf '\n' >> "$OUT"
  else
    msg="$(tr '\n' ' ' < "$err" | sed 's/"/\\"/g')"
    safe_url="$(printf '%s' "$url" | sed 's/"/\\"/g')"
    printf '{"mechanism":"yt-dlp metadata","status":"error","url":"%s","message":"%s"}\n' "$safe_url" "$msg" >> "$OUT_DIR/errors.jsonl"
  fi
  rm -f "$tmp" "$err"
done < "$INPUT_URLS"

#!/usr/bin/env bash
set -u

OUT="$OUT_DIR/oembed.jsonl"
: > "$OUT"

if ! command -v curl >/dev/null 2>&1; then
  printf '{"mechanism":"TikTok oEmbed","status":"skipped","reason":"curl not found"}\n' >> "$OUT_DIR/errors.jsonl"
  exit 0
fi

if ! command -v python3 >/dev/null 2>&1; then
  printf '{"mechanism":"TikTok oEmbed","status":"skipped","reason":"python3 not found for URL encoding"}\n' >> "$OUT_DIR/errors.jsonl"
  exit 0
fi

if [ ! -f "$INPUT_URLS" ]; then
  printf '{"mechanism":"TikTok oEmbed","status":"skipped","reason":"input_urls.txt not found"}\n' >> "$OUT_DIR/errors.jsonl"
  exit 0
fi

while IFS= read -r url || [ -n "$url" ]; do
  case "$url" in
    ""|\#*) continue ;;
  esac

  encoded="$(python3 -c 'import sys, urllib.parse; print(urllib.parse.quote(sys.argv[1], safe=""))' "$url")"
  tmp="$OUT_DIR/oembed.tmp.json"
  err="$OUT_DIR/oembed.tmp.err"
  code="$(curl -sS -L -w '%{http_code}' -o "$tmp" "https://www.tiktok.com/oembed?url=$encoded" 2> "$err" || true)"
  safe_url="$(printf '%s' "$url" | sed 's/"/\\"/g')"
  if [ "$code" = "200" ] && [ -s "$tmp" ]; then
    printf '{"mechanism":"TikTok oEmbed","status":"ok","url":"%s","response":' "$safe_url" >> "$OUT"
    cat "$tmp" >> "$OUT"
    printf '}\n' >> "$OUT"
  else
    msg="$(tr '\n' ' ' < "$err" | sed 's/"/\\"/g')"
    printf '{"mechanism":"TikTok oEmbed","status":"error","url":"%s","http_status":"%s","message":"%s"}\n' "$safe_url" "$code" "$msg" >> "$OUT_DIR/errors.jsonl"
  fi
  rm -f "$tmp" "$err"
done < "$INPUT_URLS"

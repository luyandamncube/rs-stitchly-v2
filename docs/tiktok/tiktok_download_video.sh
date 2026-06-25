#!/usr/bin/env bash
set -uo pipefail

# shellcheck source=docs/tiktok/tiktok_common.sh
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/tiktok_common.sh"

video_url="${1:-${TIKTOK_VIDEO_URL:-}}"
download_root="${TIKTOK_DOWNLOAD_OUT_DIR:-$TIKTOK_OUT_DIR/downloads}"
manifest_file="${TIKTOK_DOWNLOAD_MANIFEST_FILE:-$TIKTOK_OUT_DIR/tiktok_download_manifest.jsonl}"
enable_download="${ENABLE_MEDIA_DOWNLOAD:-false}"
generated_at_epoch="$(date +%s)"
TIKTOK_REPO_ROOT="$(cd "$TIKTOK_ROOT/../.." && pwd)"
repo_local_ytdlp="$TIKTOK_REPO_ROOT/.stitchly/tools/yt-dlp-venv/bin/yt-dlp"
ytdlp_bin="${TIKTOK_YTDLP_BIN:-}"
ytdlp_format="${TIKTOK_YTDLP_FORMAT:-best[vcodec^=h264][ext=mp4]/best[vcodec!=none][ext=mp4]/best}"

emit_json() {
  local payload="$1"
  printf '%s\n' "$payload" | tee -a "$manifest_file"
}

json_error() {
  local status="$1"
  local message="$2"
  jq -cn \
    --arg status "$status" \
    --arg message "$message" \
    --arg video_url "$video_url" \
    --argjson generated_at_epoch "$generated_at_epoch" \
    '{
      ok: false,
      status: $status,
      message: $message,
      video_url: $video_url,
      generated_at_epoch: $generated_at_epoch
    }'
}

if [ -z "$video_url" ]; then
  mkdir -p "$(dirname "$manifest_file")"
  emit_json "$(json_error "missing_url" "Pass a TikTok URL as the first argument or set TIKTOK_VIDEO_URL.")"
  exit 0
fi

case "$video_url" in
  https://www.tiktok.com/*|http://www.tiktok.com/*|https://tiktok.com/*|http://tiktok.com/*|https://vm.tiktok.com/*|http://vm.tiktok.com/*|https://vt.tiktok.com/*|http://vt.tiktok.com/*) ;;
  *)
    mkdir -p "$(dirname "$manifest_file")"
    emit_json "$(json_error "invalid_url" "URL does not look like a TikTok URL.")"
    exit 0
    ;;
esac

if ! command -v jq >/dev/null 2>&1; then
  echo "missing required command: jq" >&2
  exit 1
fi

if [ -z "$ytdlp_bin" ]; then
  if command -v yt-dlp >/dev/null 2>&1; then
    ytdlp_bin="$(command -v yt-dlp)"
  elif [ -x "$repo_local_ytdlp" ]; then
    ytdlp_bin="$repo_local_ytdlp"
  fi
fi

if [ -z "$ytdlp_bin" ] || [ ! -x "$ytdlp_bin" ]; then
  mkdir -p "$(dirname "$manifest_file")"
  emit_json "$(json_error "missing_yt_dlp" "yt-dlp is not installed or not on PATH. Install it, or set TIKTOK_YTDLP_BIN to a yt-dlp executable.")"
  exit 0
fi

mkdir -p "$download_root" "$(dirname "$manifest_file")"

url_hash="$(printf '%s' "$video_url" | sha256sum | awk '{print substr($1, 1, 16)}')"
tmp_metadata="$download_root/metadata_${url_hash}.tmp.json"
metadata_stderr="$download_root/metadata_${url_hash}.stderr.txt"

metadata_exit_code=0
if "$ytdlp_bin" --dump-json --no-playlist --ignore-errors --no-warnings "$video_url" > "$tmp_metadata" 2> "$metadata_stderr"; then
  metadata_exit_code=0
else
  metadata_exit_code=$?
fi

if [ ! -s "$tmp_metadata" ] || ! jq empty "$tmp_metadata" >/dev/null 2>&1; then
  stderr_preview="$(tr '\n' ' ' < "$metadata_stderr" | cut -c 1-1000)"
  payload="$(
    jq -cn \
      --arg video_url "$video_url" \
      --arg download_root "$download_root" \
      --arg metadata_stderr_file "$metadata_stderr" \
      --arg stderr_preview "$stderr_preview" \
      --argjson metadata_exit_code "$metadata_exit_code" \
      --argjson generated_at_epoch "$generated_at_epoch" \
      '{
        ok: false,
        status: "metadata_failed",
        video_url: $video_url,
        download_root: $download_root,
        metadata_exit_code: $metadata_exit_code,
        metadata_stderr_file: $metadata_stderr_file,
        stderr_preview: $stderr_preview,
        generated_at_epoch: $generated_at_epoch
      }'
  )"
  rm -f "$tmp_metadata"
  emit_json "$payload"
  exit 0
fi

video_id="$(jq -r '.id // empty' "$tmp_metadata")"
if [ -z "$video_id" ]; then
  video_id="$url_hash"
fi

safe_video_id="$(printf '%s' "$video_id" | tr -cd 'A-Za-z0-9._-')"
if [ -z "$safe_video_id" ]; then
  safe_video_id="$url_hash"
fi

run_dir="$download_root/$safe_video_id"
metadata_file="$run_dir/metadata.json"
download_stdout="$run_dir/download.stdout.txt"
download_stderr="$run_dir/download.stderr.txt"

mkdir -p "$run_dir"
mv "$tmp_metadata" "$metadata_file"
if [ -s "$metadata_stderr" ]; then
  mv "$metadata_stderr" "$run_dir/metadata.stderr.txt"
  metadata_stderr="$run_dir/metadata.stderr.txt"
fi

title="$(jq -r '.title // ""' "$metadata_file")"
uploader="$(jq -r '.uploader // .channel // .creator // ""' "$metadata_file")"
duration="$(jq -r '.duration // null' "$metadata_file")"
webpage_url="$(jq -r --arg video_url "$video_url" '.webpage_url // $video_url' "$metadata_file")"

if [ "$enable_download" != "true" ]; then
  payload="$(
    jq -cn \
      --arg video_url "$video_url" \
      --arg webpage_url "$webpage_url" \
      --arg video_id "$video_id" \
      --arg title "$title" \
      --arg uploader "$uploader" \
      --arg metadata_file "$metadata_file" \
      --arg download_root "$download_root" \
      --arg run_dir "$run_dir" \
      --argjson duration "$duration" \
      --argjson generated_at_epoch "$generated_at_epoch" \
      '{
        ok: true,
        status: "metadata_only",
        download_enabled: false,
        video_url: $video_url,
        webpage_url: $webpage_url,
        video_id: $video_id,
        title: $title,
        uploader: $uploader,
        duration: $duration,
        metadata_file: $metadata_file,
        download_root: $download_root,
        run_dir: $run_dir,
        generated_at_epoch: $generated_at_epoch
      }'
  )"
  emit_json "$payload"
  exit 0
fi

download_exit_code=0
if "$ytdlp_bin" \
  --no-playlist \
  --format "$ytdlp_format" \
  --force-overwrites \
  --write-info-json \
  --write-thumbnail \
  --paths "$run_dir" \
  --output "%(id)s.%(ext)s" \
  --print after_move:filepath \
  "$video_url" > "$download_stdout" 2> "$download_stderr"; then
  download_exit_code=0
else
  download_exit_code=$?
fi

media_file="$(tail -n 1 "$download_stdout" | tr -d '\r')"
if [ -z "$media_file" ] || [ ! -f "$media_file" ]; then
  media_file="$(find "$run_dir" -maxdepth 1 -type f ! -name '*.json' ! -name '*.txt' ! -name '*.part' | sort | tail -n 1)"
fi

info_json_file="$(find "$run_dir" -maxdepth 1 -type f -name '*.info.json' | sort | tail -n 1)"
stderr_preview="$(tr '\n' ' ' < "$download_stderr" | cut -c 1-1000)"
selected_format_id=""
media_vcodec=""
media_acodec=""
media_width="null"
media_height="null"
if [ -n "$info_json_file" ] && [ -f "$info_json_file" ]; then
  selected_format_id="$(jq -r '.format_id // ""' "$info_json_file")"
  media_vcodec="$(jq -r '.vcodec // ""' "$info_json_file")"
  media_acodec="$(jq -r '.acodec // ""' "$info_json_file")"
  media_width="$(jq -r '.width // null' "$info_json_file")"
  media_height="$(jq -r '.height // null' "$info_json_file")"
fi

payload="$(
  jq -cn \
    --arg video_url "$video_url" \
    --arg webpage_url "$webpage_url" \
    --arg video_id "$video_id" \
    --arg title "$title" \
    --arg uploader "$uploader" \
    --arg metadata_file "$metadata_file" \
    --arg media_file "$media_file" \
    --arg info_json_file "$info_json_file" \
    --arg run_dir "$run_dir" \
    --arg ytdlp_format "$ytdlp_format" \
    --arg selected_format_id "$selected_format_id" \
    --arg media_vcodec "$media_vcodec" \
    --arg media_acodec "$media_acodec" \
    --arg download_stdout_file "$download_stdout" \
    --arg download_stderr_file "$download_stderr" \
    --arg stderr_preview "$stderr_preview" \
    --argjson duration "$duration" \
    --argjson media_width "$media_width" \
    --argjson media_height "$media_height" \
    --argjson download_exit_code "$download_exit_code" \
    --argjson generated_at_epoch "$generated_at_epoch" \
    '{
      ok: (($download_exit_code == 0) and ($media_file != "")),
      status: (if (($download_exit_code == 0) and ($media_file != "")) then "downloaded" else "download_failed" end),
      download_enabled: true,
      video_url: $video_url,
      webpage_url: $webpage_url,
      video_id: $video_id,
      title: $title,
      uploader: $uploader,
      duration: $duration,
      metadata_file: $metadata_file,
      media_file: $media_file,
      info_json_file: $info_json_file,
      run_dir: $run_dir,
      ytdlp_format: $ytdlp_format,
      selected_format_id: $selected_format_id,
      media_vcodec: $media_vcodec,
      media_acodec: $media_acodec,
      media_width: $media_width,
      media_height: $media_height,
      download_exit_code: $download_exit_code,
      download_stdout_file: $download_stdout_file,
      download_stderr_file: $download_stderr_file,
      stderr_preview: $stderr_preview,
      generated_at_epoch: $generated_at_epoch
    }'
)"

emit_json "$payload"
exit 0

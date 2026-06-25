#!/usr/bin/env bash
set -uo pipefail

# shellcheck source=docs/tiktok/tiktok_common.sh
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/tiktok_common.sh"

require_cmd jq

export TIKTOK_ENV_FILE
export TIKTOK_OUT_DIR
export TIKTOK_TOKEN_FILE

steps_file="$TIKTOK_OUT_DIR/auth_probe_steps.jsonl"
report_json="$TIKTOK_OUT_DIR/auth_probe_report.json"
report_md="$TIKTOK_OUT_DIR/auth_probe_report.md"

: > "$steps_file"

json_string() {
  jq -Rs . <<< "${1:-}"
}

run_step() {
  local name="$1"
  local script="$2"
  local stdout_file="$TIKTOK_OUT_DIR/${name}.stdout.txt"
  local stderr_file="$TIKTOK_OUT_DIR/${name}.stderr.txt"
  local started ended duration exit_code status

  started="$(date +%s)"
  if bash "$TIKTOK_ROOT/$script" > "$stdout_file" 2> "$stderr_file"; then
    exit_code=0
    status="ok"
  else
    exit_code=$?
    status="error"
  fi
  ended="$(date +%s)"
  duration="$((ended - started))"

  jq -n \
    --arg name "$name" \
    --arg script "$script" \
    --arg status "$status" \
    --arg stdout_file "$stdout_file" \
    --arg stderr_file "$stderr_file" \
    --argjson exit_code "$exit_code" \
    --argjson duration_seconds "$duration" \
    --rawfile stdout "$stdout_file" \
    --rawfile stderr "$stderr_file" \
    '{
      name: $name,
      script: $script,
      status: $status,
      exit_code: $exit_code,
      duration_seconds: $duration_seconds,
      stdout_file: $stdout_file,
      stderr_file: $stderr_file,
      stdout_preview: ($stdout | split("\n") | map(select(length > 0)) | .[0:8] | join("\n")),
      stderr_preview: ($stderr | split("\n") | map(select(length > 0)) | .[0:8] | join("\n"))
    }' >> "$steps_file"

  printf '%s: %s\n' "$name" "$status"
}

json_or_empty() {
  local source_file="$1"
  local fallback_file="$2"
  if [ -f "$source_file" ] && jq empty "$source_file" >/dev/null 2>&1; then
    printf '%s' "$source_file"
  else
    printf '{}\n' > "$fallback_file"
    printf '%s' "$fallback_file"
  fi
}

run_step "auth_status" "tiktok_auth_status.sh"
run_step "user_info_probe" "tiktok_user_info.sh"
run_step "video_list_probe" "tiktok_video_list.sh"

auth_status_file="$(json_or_empty "$TIKTOK_OUT_DIR/auth_status.json" "$TIKTOK_OUT_DIR/auth_status.empty.json")"
user_info_file="$(json_or_empty "$TIKTOK_OUT_DIR/user_info_response.json" "$TIKTOK_OUT_DIR/user_info_response.empty.json")"
video_list_file="$(json_or_empty "$TIKTOK_OUT_DIR/video_list_response.json" "$TIKTOK_OUT_DIR/video_list_response.empty.json")"

jq -s \
  --arg token_file "$TIKTOK_TOKEN_FILE" \
  --arg auth_status_file "$auth_status_file" \
  --arg user_info_file "$user_info_file" \
  --arg video_list_file "$video_list_file" \
  --slurpfile auth_status "$auth_status_file" \
  --slurpfile user_info "$user_info_file" \
  --slurpfile video_list "$video_list_file" \
  '. as $steps | {
    generated_at_epoch: now | floor,
    token_file: $token_file,
    ok: all($steps[]; .status == "ok"),
    steps: $steps,
    artifacts: {
      auth_status: $auth_status_file,
      user_info: $user_info_file,
      video_list: $video_list_file
    },
    auth_status: ($auth_status[0] // {}),
    user_info_summary: {
      ok: (
        (first($steps[] | select(.name == "user_info_probe") | .status) == "ok")
        and (($user_info[0].error // null) == null or (($user_info[0].error.code // "") == "ok"))
      ),
      open_id_present: (($user_info[0].data.user.open_id // "") != ""),
      display_name_present: (($user_info[0].data.user.display_name // "") != ""),
      error: ($user_info[0].error // null)
    },
    video_list_summary: {
      ok: (
        (first($steps[] | select(.name == "video_list_probe") | .status) == "ok")
        and (($video_list[0].error // null) == null or (($video_list[0].error.code // "") == "ok"))
      ),
      video_count: (($video_list[0].data.videos // []) | length),
      has_cursor: (($video_list[0].data.cursor // null) != null),
      has_more: ($video_list[0].data.has_more // null),
      error: ($video_list[0].error // null)
    }
  }' "$steps_file" > "$report_json"

{
  echo "# TikTok Auth Probe Report"
  echo
  jq -r '
    "- ok: `" + (.ok | tostring) + "`",
    "- token_file: `" + .token_file + "`",
    "- access_token_seconds_remaining: `" + ((.auth_status.access_token_seconds_remaining // "unknown") | tostring) + "`",
    "- refresh_token_seconds_remaining: `" + ((.auth_status.refresh_token_seconds_remaining // "unknown") | tostring) + "`",
    "- user_info_ok: `" + (.user_info_summary.ok | tostring) + "`",
    "- video_list_ok: `" + (.video_list_summary.ok | tostring) + "`",
    "- video_count: `" + (.video_list_summary.video_count | tostring) + "`",
    "",
    "## Steps",
    "",
    (.steps[] | "- `" + .name + "`: `" + .status + "` exit=`" + (.exit_code | tostring) + "` duration=`" + (.duration_seconds | tostring) + "s`")
  ' "$report_json"
} > "$report_md"

cat "$report_json"

if jq -e '.ok == true' "$report_json" >/dev/null; then
  exit 0
fi

exit 1

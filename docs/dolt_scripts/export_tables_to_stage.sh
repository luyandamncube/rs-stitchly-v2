set -eu

workspace="${DUCKLE_WORKSPACE:-$PWD}"
duckdb_bin="${DUCKLE_DUCKDB_BIN:-duckdb}"

json_escape() {
  sed 's/\\/\\\\/g; s/"/\\"/g'
}

csv_value() {
  tail -n +2 | tr -d '\r' | sed 's/^"//; s/"$//'
}

json_field_at() {
  field="$1"
  file="$2"
  offset="$3"
  "$duckdb_bin" -csv -c "select coalesce(cast(${field} as varchar), '') from read_json_auto('${file}') limit 1 offset ${offset}" |
    csv_value
}

cd "$workspace"

if [ "${DUCKLE_INPUT_ROW_COUNT:-}" = "0" ]; then
  exit 0
fi

if [ ! -s "$DUCKLE_INPUT_PATH" ]; then
  echo "export_tables_to_stage: upstream input file is empty or missing" >&2
  exit 1
fi

input_rows="$(
  "$duckdb_bin" -csv -c "select count(*) as n from read_json_auto('${DUCKLE_INPUT_PATH}')" |
    csv_value
)"

if [ -z "$input_rows" ] || [ "$input_rows" -eq 0 ]; then
  exit 0
fi

idx=0
while [ "$idx" -lt "$input_rows" ]; do
  repo_key="$(json_field_at repo_key "$DUCKLE_INPUT_PATH" "$idx")"
  branch="$(json_field_at branch "$DUCKLE_INPUT_PATH" "$idx")"
  repo_path="$(json_field_at repo_path "$DUCKLE_INPUT_PATH" "$idx")"
  table_name="$(json_field_at table_name "$DUCKLE_INPUT_PATH" "$idx")"
  previous_commit="$(json_field_at previous_commit "$DUCKLE_INPUT_PATH" "$idx")"
  head_commit="$(json_field_at head_commit "$DUCKLE_INPUT_PATH" "$idx")"
  export_mode="$(json_field_at export_mode "$DUCKLE_INPUT_PATH" "$idx")"
  reason="$(json_field_at reason "$DUCKLE_INPUT_PATH" "$idx")"
  stage_path="$(json_field_at stage_path "$DUCKLE_INPUT_PATH" "$idx")"
  snapshot_path="$(json_field_at snapshot_path "$DUCKLE_INPUT_PATH" "$idx")"
  plan_ok="$(json_field_at plan_ok "$DUCKLE_INPUT_PATH" "$idx")"
  should_export="$(json_field_at should_export "$DUCKLE_INPUT_PATH" "$idx")"

  if [ "$plan_ok" != "true" ]; then
    echo "export_tables_to_stage: row $idx plan_ok is not true" >&2
    exit 1
  fi

  if [ "$should_export" != "true" ]; then
    echo "export_tables_to_stage: row $idx should_export is not true" >&2
    exit 1
  fi

  if [ "$export_mode" != "snapshot" ]; then
    echo "export_tables_to_stage: row $idx unsupported export_mode=$export_mode" >&2
    exit 1
  fi

  if [ ! -d "$repo_path/.dolt" ]; then
    echo "export_tables_to_stage: row $idx Dolt repo not found at $repo_path" >&2
    exit 1
  fi

  mkdir -p "$(dirname "$stage_path")"

  row_count="$(
    cd "$repo_path" &&
      dolt sql -r csv -q "select count(*) as n from $table_name" |
      csv_value
  )"

  rm -f "$stage_path"

  (
    cd "$repo_path"
    dolt table export --force --file-type parquet "$table_name" "$workspace/$stage_path" >&2
  )

  if [ ! -s "$stage_path" ]; then
    echo "export_tables_to_stage: row $idx export did not create non-empty file at $stage_path" >&2
    exit 1
  fi

  file_size="$(wc -c < "$stage_path" | tr -d ' ')"

  repo_key_json="$(printf '%s' "$repo_key" | json_escape)"
  branch_json="$(printf '%s' "$branch" | json_escape)"
  repo_path_json="$(printf '%s' "$repo_path" | json_escape)"
  table_name_json="$(printf '%s' "$table_name" | json_escape)"
  previous_commit_json="$(printf '%s' "$previous_commit" | json_escape)"
  head_commit_json="$(printf '%s' "$head_commit" | json_escape)"
  export_mode_json="$(printf '%s' "$export_mode" | json_escape)"
  reason_json="$(printf '%s' "$reason" | json_escape)"
  stage_path_json="$(printf '%s' "$stage_path" | json_escape)"
  snapshot_path_json="$(printf '%s' "$snapshot_path" | json_escape)"

  printf '%s' '{"repo_key":"'
  printf '%s' "$repo_key_json"
  printf '%s' '","branch":"'
  printf '%s' "$branch_json"
  printf '%s' '","repo_path":"'
  printf '%s' "$repo_path_json"
  printf '%s' '","table_name":"'
  printf '%s' "$table_name_json"
  printf '%s' '","previous_commit":"'
  printf '%s' "$previous_commit_json"
  printf '%s' '","head_commit":"'
  printf '%s' "$head_commit_json"
  printf '%s' '","export_mode":"'
  printf '%s' "$export_mode_json"
  printf '%s' '","reason":"'
  printf '%s' "$reason_json"
  printf '%s' '","stage_path":"'
  printf '%s' "$stage_path_json"
  printf '%s' '","snapshot_path":"'
  printf '%s' "$snapshot_path_json"
  printf '%s' '","row_count":'
  printf '%s' "$row_count"
  printf '%s' ',"file_size_bytes":'
  printf '%s' "$file_size"
  printf '%s\n' ',"export_ok":true}'

  idx=$((idx + 1))
done

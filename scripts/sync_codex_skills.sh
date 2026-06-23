#!/usr/bin/env sh
set -eu

repo_root="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
source_dir="$repo_root/.codex/skills"
codex_home="${CODEX_HOME:-$HOME/.codex}"
target_dir="$codex_home/skills"

if [ ! -d "$source_dir" ]; then
  echo "No project skills found at $source_dir" >&2
  exit 1
fi

mkdir -p "$target_dir"

for skill_dir in "$source_dir"/*; do
  [ -d "$skill_dir" ] || continue
  skill_name="$(basename "$skill_dir")"
  mkdir -p "$target_dir/$skill_name"
  cp -a "$skill_dir/." "$target_dir/$skill_name/"
  echo "synced $skill_name -> $target_dir/$skill_name"
done

echo "done"

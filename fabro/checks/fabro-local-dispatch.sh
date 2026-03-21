#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
run_dir="$repo_root/.raspberry/runs/foundations"
log_dir="$repo_root/.raspberry/logs"
fabro_bin="/home/r/coding/fabro/target-local/debug/fabro"

mkdir -p "$run_dir" "$log_dir"

args=()
inserted_run_dir=false
for arg in "$@"; do
  args+=("$arg")
  if [[ "$arg" == "run" && "$inserted_run_dir" == false ]]; then
    args+=("--run-dir" "$run_dir")
    inserted_run_dir=true
  fi
done

if [[ "$inserted_run_dir" == false ]]; then
  echo "expected fabro subcommand sequence containing 'run'" >&2
  exit 2
fi

export FABRO_LOG_DIR="$log_dir"

exec "$fabro_bin" "${args[@]}"

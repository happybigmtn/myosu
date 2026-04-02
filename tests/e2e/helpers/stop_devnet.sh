#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
runtime_dir="${MYOSU_E2E_DIR:-$repo_root/target/e2e/devnet}"
state_file="$runtime_dir/devnet.env"
pid_file="$runtime_dir/devnet.pid"
tmp_root="$runtime_dir/tmp"

if [[ -f "$state_file" ]]; then
  # shellcheck disable=SC1090
  source "$state_file"
  pid_file="${PID_FILE:-$pid_file}"
  tmp_root="${TMP_ROOT:-$tmp_root}"
fi

if [[ -f "$pid_file" ]]; then
  node_pid="$(<"$pid_file")"
  if kill -0 "$node_pid" 2>/dev/null; then
    kill "$node_pid"
    for _ in $(seq 1 30); do
      if kill -0 "$node_pid" 2>/dev/null; then
        sleep 1
      else
        break
      fi
    done
    if kill -0 "$node_pid" 2>/dev/null; then
      kill -9 "$node_pid"
    fi
  fi
fi

rm -f "$pid_file" "$state_file"
rm -rf "$tmp_root"

echo "myosu e2e devnet stopped"

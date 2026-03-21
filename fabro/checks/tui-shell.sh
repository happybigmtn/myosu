#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -- "${script_dir}/../.." && pwd)"
target_dir="${repo_root}/.raspberry/cargo-target"

mkdir -p "${target_dir}"
cargo test -p myosu-tui --target-dir "${target_dir}"

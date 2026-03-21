#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/../.." && pwd)"
cd "$REPO_ROOT"

CARGO_TARGET_DIR="${MYOSU_CARGO_TARGET_DIR:-/tmp/myosu-cargo-target}"
export CARGO_TARGET_DIR

cargo test -p myosu-tui

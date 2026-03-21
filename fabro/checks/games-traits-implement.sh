#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/../.." && pwd)"
cd "$REPO_ROOT"

CARGO_TARGET_DIR="${MYOSU_CARGO_TARGET_DIR:-/tmp/myosu-cargo-target}"
export CARGO_TARGET_DIR

test -f outputs/games/traits/spec.md
test -f outputs/games/traits/review.md

cargo check -p myosu-games
cargo test -p myosu-games

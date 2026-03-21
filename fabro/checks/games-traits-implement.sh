#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
export CARGO_TARGET_DIR="${MYOSU_CARGO_TARGET_DIR:-$repo_root/.raspberry/cargo-target}"
mkdir -p "$CARGO_TARGET_DIR"

test -f outputs/games/traits/spec.md
test -f outputs/games/traits/review.md

cargo check -p myosu-games
cargo test -p myosu-games

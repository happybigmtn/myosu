#!/usr/bin/env bash
set -euo pipefail

# Slice 1: Create myosu-games-liars-dice Crate Skeleton
# Only tests packages that exist in the workspace.
# Later slices add: myosu-play, myosu-games-poker

# Bootstrap gate — crate skeleton integrity
cargo build -p myosu-games-liars-dice
cargo test -p myosu-games-liars-dice

# Smoke test that existing workspace packages still pass
cargo test -p myosu-games
cargo test -p myosu-tui

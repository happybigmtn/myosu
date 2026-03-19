#!/usr/bin/env bash
set -euo pipefail

test -f outputs/games/traits/spec.md
test -f outputs/games/traits/review.md

cargo check -p myosu-games
cargo test -p myosu-games

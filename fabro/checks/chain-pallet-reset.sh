#!/usr/bin/env bash
set -euo pipefail

test -f crates/myosu-chain/pallets/game-solver/src/lib.rs
test -f crates/myosu-chain/pallets/game-solver/Cargo.toml

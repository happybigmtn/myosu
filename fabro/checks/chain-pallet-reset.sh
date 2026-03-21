#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/../.." && pwd)"
cd "$REPO_ROOT"

test -f crates/myosu-chain/pallets/game-solver/src/lib.rs
test -f crates/myosu-chain/pallets/game-solver/Cargo.toml

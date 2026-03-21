#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/../.." && pwd)"
cd "$REPO_ROOT"

test -f crates/myosu-chain/runtime/src/lib.rs
test -d crates/myosu-chain/node/src
test -d crates/myosu-chain/common/src

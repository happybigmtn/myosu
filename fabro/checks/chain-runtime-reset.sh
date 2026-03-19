#!/usr/bin/env bash
set -euo pipefail

test -f crates/myosu-chain/runtime/src/lib.rs
test -d crates/myosu-chain/node/src
test -d crates/myosu-chain/common/src

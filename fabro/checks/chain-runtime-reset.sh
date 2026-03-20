#!/usr/bin/env bash
set -euo pipefail

# Honest proof for chain:runtime.
# The prior version was a no-op that only checked file existence.
# The bootstrap review (outputs/chain/runtime/review.md) established that
# the runtime cannot be built: imports nonexistent crates, no WASM build,
# node is scaffold-only. This script now performs a real proof.
#
# PASS: cargo check -p myosu-runtime succeeds
# FAIL: cargo check -p myosu-runtime fails (honest current state)

cd "$(dirname "$0")/../.."

echo "=== chain:runtime proof ==="
echo "Running: cargo check -p myosu-runtime"
echo "Expected: FAILS (runtime has missing dependencies per outputs/chain/runtime/review.md)"
echo ""

if cargo check -p myosu-runtime 2>&1; then
    echo ""
    echo "PASS: runtime type-checks"
    exit 0
else
    echo ""
    echo "FAIL: runtime has errors (honest current state)"
    exit 1
fi

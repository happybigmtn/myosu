#!/usr/bin/env bash
set -euo pipefail

# Honest proof for chain:pallet.
# The prior version was a no-op that only checked file existence.
# The bootstrap review (outputs/chain/pallet/review.md) established that
# the pallet has 50+ cargo check errors from embedded subtensor workspace-key
# dependencies. This script performs a real proof by running cargo check.
#
# The check may take several minutes on first run due to dependency resolution.
# If it times out, that is itself an honest failure (the pallet cannot be
# quickly verified).
#
# PASS: cargo check -p pallet-game-solver succeeds (no errors)
# FAIL: cargo check -p pallet-game-solver fails OR times out after 5 minutes

cd "$(dirname "$0")/../.."

echo "=== chain:pallet proof ==="
echo "Running: cargo check -p pallet-game-solver (5min timeout)"
echo "Expected: FAILS (50+ errors per outputs/chain/pallet/review.md)"
echo ""

# Use timeout to fail fast if the check hangs (e.g., on network fetches).
# 300s = 5 minutes is generous for a cargo check on a workspace with
# the pallet already having been checked once (subsequent runs are cached).
if timeout 300 cargo check -p pallet-game-solver 2>&1; then
    echo ""
    echo "PASS: pallet type-checks"
    exit 0
else
    EXIT_CODE=$?
    if [ $EXIT_CODE -eq 124 ]; then
        echo ""
        echo "FAIL: pallet check timed out after 5 minutes"
    else
        echo ""
        echo "FAIL: pallet has type errors (honest current state)"
    fi
    exit 1
fi

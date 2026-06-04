#!/usr/bin/env bash
# NEM-002A: compile-time guard for `Stage0SwapInterface` `max_price` bound.
#
# Stage-0's no-op identity swap intentionally returns `u64::MAX` from
# `SwapHandler::max_price()` because it has no real market to constrain.
# That makes the constant a documented time-bomb: a future real AMM
# implementer who inherits the default would silently re-introduce the
# unbounded-slippage vulnerability. This harness is the executable
# end-to-end proof that the structural guard exists, is reachable from
# the public `subtensor_swap_interface` API, is honored by the live
# `Stage0NoopSwap` runtime impl, and is backed by ADR-014.
#
# Five real assertions; failing any one of them fails the gate with a
# concrete error message naming the missing piece.
#
#   1. The `MAX_VALID_SWAP_PRICE_LIMIT` and `STRICT_MAX_VALID_SWAP_PRICE_LIMIT`
#      constants are present in the public `subtensor_swap_interface` API
#      and equal `u64::MAX` and `u64::MAX / 2` respectively (i.e. the
#      stage-0 opt-out ceiling and the strict ceiling that a real AMM
#      impl must respect are both defined and have the documented shape).
#   2. The `SwapPriceLimitBounded` trait + `PRICE_LIMIT_BOUND` const +
#      `is_within_strict_bound` method are all present in the public API
#      (so a future implementer has a single seam to override).
#   3. The live `Stage0NoopSwap` impl in the myosu-chain runtime carries
#      an explicit `impl SwapPriceLimitBounded` with
#      `PRICE_LIMIT_BOUND = MAX_VALID_SWAP_PRICE_LIMIT` (the stage-0
#      opt-out is explicit in the source, not silently inherited from
#      the trait default).
#   4. The runtime's `stage0_noop_swap_price_limit_bound_is_max` unit
#      test passes via `cargo test`, proving the live bound is exactly
#      `MAX_VALID_SWAP_PRICE_LIMIT` and `is_within_strict_bound()` is
#      `false` (the stage-0 opt-out is auditable in code, not in prose).
#   5. `docs/adr/014-swap-price-limit-bound.md` exists and cross-
#      references the NEM-002A plan row (so the seam has a written
#      rationale that survives a refactor).

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

# Use the same crate name the rest of the workspace uses.
swap_iface_crate="subtensor-swap-interface"
swap_iface_lib="crates/myosu-chain/pallets/swap-interface/src/lib.rs"
runtime_lib="crates/myosu-chain/runtime/src/lib.rs"
adr_path="docs/adr/014-swap-price-limit-bound.md"

fail() {
    printf 'swap_price_limit_bound: FAIL %s\n' "$1" >&2
    exit 1
}

# -- 1. The two constants are present in the swap-interface public API.
for needle in \
    'pub const MAX_VALID_SWAP_PRICE_LIMIT: u64 = u64::MAX;' \
    'pub const STRICT_MAX_VALID_SWAP_PRICE_LIMIT: u64 = u64::MAX / 2;' \
    'pub trait SwapPriceLimitBounded' \
    'const PRICE_LIMIT_BOUND: u64 = MAX_VALID_SWAP_PRICE_LIMIT;' \
    'fn is_within_strict_bound() -> bool'; do
    if ! grep -Fq -- "$needle" "$swap_iface_lib"; then
        fail "swap-interface public API is missing the marker: $needle"
    fi
done

# -- 2. Verify the trait + default-method shape end-to-end by compiling
#       and running the swap-interface unit tests. The new
#       `swap_price_limit_bounded_*` tests fail-closed if any of the
#       above markers is missing or has drifted.
if ! env SKIP_WASM_BUILD=1 cargo test -p "$swap_iface_crate" --quiet -- \
        swap_price_limit_bounded >/dev/null 2>&1; then
    fail "swap-interface unit tests for SwapPriceLimitBounded did not pass"
fi

# -- 3. The live `Stage0NoopSwap` impl carries the explicit override
#       (and not just the trait default). The const-eval guard is
#       `const _: () = assert!(...)`, so dropping the impl block would
#       fail to compile -- but we still grep for the explicit
#       `PRICE_LIMIT_BOUND: u64 = ...` line so a future refactor that
#       re-inlines the value can't silently regress to a default.
if ! grep -Eq -- 'impl[[:space:]]+(subtensor_swap_interface::)?SwapPriceLimitBounded[[:space:]]+for[[:space:]]+Stage0NoopSwap' "$runtime_lib"; then
    fail "runtime is missing the explicit 'impl SwapPriceLimitBounded for Stage0NoopSwap' block"
fi
if ! grep -Eq -- 'const[[:space:]]+PRICE_LIMIT_BOUND:[[:space:]]+u64[[:space:]]*=[[:space:]]*(subtensor_swap_interface::)?MAX_VALID_SWAP_PRICE_LIMIT;' "$runtime_lib"; then
    fail "runtime is missing the explicit 'PRICE_LIMIT_BOUND = MAX_VALID_SWAP_PRICE_LIMIT' override on Stage0NoopSwap"
fi
# The const-eval guard itself must be present (the structural fence).
if ! grep -Fq -- 'Stage0NoopSwap must opt-in to the unbounded bound' "$runtime_lib"; then
    fail "runtime is missing the const-eval guard message; the compile-time fence has been removed"
fi

# -- 4. The runtime's new unit test passes. The test name is
#       `stage0_noop_swap_price_limit_bound_is_max` (per the plan row
#       acceptance criterion 4).
runtime_test_output="$(
    env SKIP_WASM_BUILD=1 cargo test -p myosu-chain-runtime --quiet -- \
        stage0_noop_swap_price_limit_bound_is_max 2>&1
)"
if ! printf '%s\n' "$runtime_test_output" | grep -Eq '^test result: ok\. 1 passed'; then
    fail "runtime test stage0_noop_swap_price_limit_bound_is_max did not pass: $runtime_test_output"
fi

# -- 5. ADR-014 exists and is cross-referenced from the plan row.
if [[ ! -f "$adr_path" ]]; then
    fail "ADR-014 missing at $adr_path"
fi
# The ADR must reference both the NEM-002A plan row and the
# `SwapPriceLimitBounded` trait, otherwise it is a stub and the gate
# has not actually documented the seam.
for needle in 'NEM-002A' 'SwapPriceLimitBounded' 'STRICT_MAX_VALID_SWAP_PRICE_LIMIT'; do
    if ! grep -Fq -- "$needle" "$adr_path"; then
        fail "ADR-014 is missing the cross-reference marker: $needle"
    fi
done

printf 'SWAP_PRICE_LIMIT_BOUND_HARNESS myosu e2e ok trait=SwapPriceLimitBounded stage0_opt_out=MAX_VALID_SWAP_PRICE_LIMIT strict_ceiling=STRICT_MAX_VALID_SWAP_PRICE_LIMIT adr=docs/adr/014-swap-price-limit-bound.md\n'

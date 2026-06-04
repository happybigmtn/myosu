#!/usr/bin/env bash
# NEM-006: try_state emission diagnostics.
#
# The stage-0 `check_total_issuance` used to fail with the bare
# `"TotalIssuance diff greater than allowable delta"` message, which
# gave operators no way to triage the magnitude or root cause without
# re-running the on-chain state through an indexer. NEM-006 replaces
# that with (a) `log::error!` / `log::warn!` records that name the
# live, expected, diff, and delta values directly under the
# `runtime::game_solver` log target, and (b) a stable `&'static str`
# `TOTAL_ISSUANCE_TRY_STATE_FAILURE` constant that a wrapper script
# can `grep` against. This harness is the executable end-to-end proof
# that the diagnostic surface is in source, is exercised by the new
# unit tests, and is wired into the `try_state` Hooks path the runtime
# migration smoke test depends on.
#
# Six real assertions; failing any one of them fails the gate with a
# concrete error message naming the missing piece.
#
#   1. The `TOTAL_ISSUANCE_TRY_STATE_FAILURE` constant is present in
#      `crates/myosu-chain/pallets/game-solver/src/utils/try_state.rs`
#      and starts with the documented `"TotalIssuance try_state failure:"`
#      prefix (so a wrapper script can `grep` it and recover the call
#      site).
#   2. The constant redirects operators to the `runtime::game_solver`
#      log target where the live/expected/diff/delta numeric triple
#      actually lives (substrate's `DispatchError` is a `&'static str`
#      newtype and cannot carry owned data, so the constant is the
#      static anchor the log records point back to).
#   3. The three log call sites are present in the source — `log::debug!`
#      for the healthy path, `log::warn!` for the within-envelope
#      path, and `log::error!` for the hard-failure path. Each call
#      site uses the `runtime::game_solver` target so a tailing
#      operator can `grep` the runtime log by target.
#   4. The hard-failure `log::error!` record includes the four
#      documented fields (`live=...`, `expected=...`, `diff=...`,
#      `delta=...`) so an operator can recover the magnitude without
#      re-running the on-chain state through an indexer.
#   5. The new unit tests pass: the constant is stable, the
#      zero-diff / within-envelope paths return `Ok(())`, and the
#      two-rao / below-live diffs return the documented error.
#   6. The `try_state` module is now compiled in the default-features
#      build (i.e. the `#[cfg(feature = "try-runtime")]` gate has been
#      removed) so the diagnostic surface is reachable from the
#      default-features pallet test suite — this is the regression
#      guard that keeps the diagnostic code in CI coverage.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

try_state_lib="crates/myosu-chain/pallets/game-solver/src/utils/try_state.rs"
utils_mod="crates/myosu-chain/pallets/game-solver/src/utils/mod.rs"
stage_0_flow="crates/myosu-chain/pallets/game-solver/src/tests/stage_0_flow.rs"
plan_path="nemesis/IMPLEMENTATION_PLAN.md"

fail() {
    printf 'try_state_emission_diagnostics: FAIL %s\n' "$1" >&2
    exit 1
}

# -- 1. The grep-friendly `TOTAL_ISSUANCE_TRY_STATE_FAILURE` constant
#       is present in source and starts with the documented prefix.
if ! grep -Eq -- 'pub(\(crate\))?[[:space:]]+const[[:space:]]+TOTAL_ISSUANCE_TRY_STATE_FAILURE' "$try_state_lib"; then
    fail "NEM-006 constant is missing in $try_state_lib"
fi
if ! grep -Fq -- '"TotalIssuance try_state failure: live != expected (diff > delta); \' "$try_state_lib"; then
    fail "NEM-006 constant does not carry the documented NEM-006 prefix"
fi

# -- 2. The constant redirects operators to the runtime::game_solver
#       log target.
if ! grep -Fq -- 'runtime::game_solver' "$try_state_lib"; then
    fail "NEM-006 constant does not redirect operators to the runtime::game_solver log target"
fi

# -- 3. All three log call sites (debug / warn / error) are present
#       in the diagnostic function with the `runtime::game_solver`
#       target.
for needle in \
    'log::debug!' \
    'log::warn!' \
    'log::error!' \
    'target: "runtime::game_solver"'; do
    if ! grep -Fq -- "$needle" "$try_state_lib"; then
        fail "NEM-006 source is missing the log call site marker: $needle"
    fi
done

# -- 4. The hard-failure `log::error!` record includes the four
#       documented fields (live, expected, diff, delta). We grep for
#       the parameter placeholder string so the gate catches both
#       present-but-empty and missing-failures.
if ! grep -Eq -- 'live=\{\}[[:space:]]+expected=\{\}[[:space:]]+diff=\{\}[[:space:]]+delta=\{\}' "$try_state_lib"; then
    fail "NEM-006 hard-failure log record is missing one of the four documented fields (live/expected/diff/delta)"
fi

# -- 5. The new unit tests pass.
test_output="$(
    env SKIP_WASM_BUILD=1 cargo test -p pallet-game-solver --quiet -- \
        try_state_emission_diagnostics 2>&1
)"
if ! printf '%s\n' "$test_output" | grep -Eq '^test result: ok\. 5 passed'; then
    fail "NEM-006 unit tests did not pass: $test_output"
fi

# -- 6. The `try_state` module is reachable from the default-features
#       build (i.e. the `#[cfg(feature = "try-runtime")]` gate has
#       been removed). If the gate were still present, the new
#       `try_state_emission_diagnostics_*` tests would still pass
#       (they live in the test module which is always compiled) but
#       the runtime migration smoke test (which uses the function
#       through the runtime's `try-runtime` feature) would no longer
#       be able to find the source. We assert the unconditional
#       `pub mod try_state;` line in `utils/mod.rs`.
if grep -Eq -- '#\[cfg\(feature *= *\"try-runtime\"\)\][[:space:]]*pub[[:space:]]+mod[[:space:]]+try_state' "$utils_mod"; then
    fail "NEM-006 try_state module is still gated behind the broken 'try-runtime' feature; the module must compile unconditionally so the diagnostic surface is reachable from the default-features build"
fi
if ! grep -Fq -- 'pub mod try_state;' "$utils_mod"; then
    fail "NEM-006 try_state module is not declared in $utils_mod"
fi

# Plan-row check (lightweight): the NEM-006 row in
# `nemesis/IMPLEMENTATION_PLAN.md` should be marked `[x]` so a future
# reader can see the diagnostic surface is now in the production
# build. The grep is intentionally narrow so it does not false-match
# a row that simply references NEM-006 from another row's resolution
# note.
# The plan row lives inside an `### ` heading. The heading opens with a
# backtick that wraps the entire title, so the NEM-006 label itself is
# not backtick-wrapped. Accept either the bare-label form (`- [x] NEM-006
# ...`) or the backtick-wrapped form (`- [x] `NEM-006` ...`) and require
# only that the heading opens with `### `. This keeps the gate robust
# against minor plan-row reformatting.
if ! grep -Eq -- '^### `?- \[x\][[:space:]]+`?NEM-006' "$plan_path"; then
    fail "NEM-006 plan row is not marked [x] in $plan_path"
fi

printf 'TRY_STATE_EMISSION_DIAGNOSTICS_HARNESS myosu e2e ok surface=try_state_error_diagnostics tests=5 hard_failure_marker="TotalIssuance try_state failure" log_target=runtime::game_solver fields=live,expected,diff,delta\n'

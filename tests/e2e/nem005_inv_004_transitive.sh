#!/usr/bin/env bash
# NEM-005: Strengthen INV-004 to catch transitive dependencies.
#
# INV-004 ("Solver-Gameplay Separation") declares that the solver layer
# (`myosu-miner`) and the gameplay layer (`myosu-play`) must share game
# engine code but never share runtime state or trust boundaries. The
# pre-existing `inv_004_solver_and_gameplay_bins_do_not_depend_on_each_other`
# test in `crates/myosu-play/tests/invariants.rs` only catches the *direct*
# dep edges between `myosu-play` and `myosu-miner`. A transitive violation
# (e.g. `myosu-chain-client -> some-shared-dep -> myosu-miner`, or a
# `pub use` re-export of a miner-originating type into the
# `myosu-chain-client` public API) would silently bypass the check.
#
# NEM-005 closes the transitive gap with two complementary checks:
#
#   (1) `cargo tree -i myosu-miner --workspace` (the *reverse-dep* walk)
#       is the authoritative transitive check. On a clean trunk checkout
#       the only line in the output is the `myosu-miner v0.1.0 (...)`
#       package line itself -- no other workspace crate may appear as a
#       reverse-dependency, direct OR transitive. A future PR that
#       introduces a `myosu-chain-client` -> ... -> `myosu-miner` chain
#       (or any other crate) would add a new `└── ...` child of that
#       line and this harness would fail closed with the offender named.
#       Enforced by the unit test
#       `inv_004_chain_client_does_not_re_export_miner_types` in
#       `crates/myosu-play/tests/invariants.rs`.
#
#   (2) `std::any::type_name::<T>()` on every `pub struct` / `pub enum`
#       in `crates/myosu-chain-client/src/lib.rs` is the *type-origin*
#       check. A `pub use` re-export of a `myosu_miner::` type into the
#       `myosu-chain-client` public API would surface as a
#       `module_path!` starting with `myosu_miner::` -- the test
#       fail-closes on any such re-export. Enforced by the unit test
#       `inv_004_public_api_has_no_miner_origin` in
#       `crates/myosu-chain-client/src/lib.rs`.
#
# This harness is the executable end-to-end gate. Four real assertions;
# failing any one of them fails the gate with a concrete error message
# naming the missing piece.
#
#   1. `cargo tree -i myosu-miner --workspace` produces output whose only
#      non-empty line is the `myosu-miner v0.1.0 (...)` package line
#      itself (no other workspace crate may appear as a reverse-dep).
#   2. `cargo tree -p myosu-chain-client --edges normal` does not
#      contain the substring `myosu-miner` (forward transitive view).
#   3. `cargo test -p myosu-play --test invariants inv_004` runs both
#      the pre-existing direct-edge test AND the new transitive test,
#      both green.
#   4. `cargo test -p myosu-chain-client --lib inv_004` runs the
#      type-origin test, green.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

inv_004_test_path="crates/myosu-play/tests/invariants.rs"
chain_client_lib_path="crates/myosu-chain-client/src/lib.rs"
plan_row="nemesis/IMPLEMENTATION_PLAN.md"

fail() {
    printf 'nem005_inv_004_transitive: FAIL %s\n' "$1" >&2
    exit 1
}

# Sub-check 1: reverse-dep walk. The only line in `cargo tree -i myosu-miner
# --workspace` is the `myosu-miner v0.1.0 (...)` package header itself; any
# other `myosu-...` crate appearing in the output is a transitive
# reverse-dep violation.
reverse_tree="$(SKIP_WASM_BUILD=1 cargo tree -i myosu-miner --workspace 2>/dev/null || true)"
if [ -z "$reverse_tree" ]; then
    fail "cargo tree -i myosu-miner --workspace produced empty output"
fi
# The expected output is exactly one non-empty line: the package header.
# A future PR that introduces a transitive reverse-dep would add an
# indented `└── ...` child line below it.
reverse_tree_lines="$(printf '%s\n' "$reverse_tree" | grep -c 'myosu-' || true)"
if [ "$reverse_tree_lines" -ne 1 ]; then
    fail "expected exactly 1 'myosu-'-bearing line in 'cargo tree -i myosu-miner --workspace' \
(the bare 'myosu-miner' package header), got $reverse_tree_lines; transitive reverse-dep(s) \
detected. Full tree:\n$reverse_tree"
fi
if ! printf '%s' "$reverse_tree" | grep -q '^myosu-miner v'; then
    fail "the 'myosu-miner' package header line was missing from the reverse-dep walk; \
expected first line to start with 'myosu-miner v'. Full tree:\n$reverse_tree"
fi

# Sub-check 2: forward transitive view. `myosu-chain-client`'s forward
# tree must not contain the substring `myosu-miner` (any path through a
# transitive dep would surface as the literal `myosu-miner v0.1.0 (...)`
# package line in the forward tree).
chain_client_tree="$(SKIP_WASM_BUILD=1 cargo tree -p myosu-chain-client --edges normal 2>/dev/null || true)"
if [ -z "$chain_client_tree" ]; then
    fail "cargo tree -p myosu-chain-client --edges normal produced empty output"
fi
if printf '%s' "$chain_client_tree" | grep -q 'myosu-miner'; then
    fail "myosu-chain-client must not depend on myosu-miner (direct or transitive); \
sub-check 2 found 'myosu-miner' in the forward tree. Full tree:\n$chain_client_tree"
fi

# Sub-check 3: the integration test in myosu-play must pass (both the
# pre-existing direct-edge test AND the new transitive test).
inv_004_test_output="$(SKIP_WASM_BUILD=1 cargo test -p myosu-play --test invariants inv_004 2>&1 || true)"
if ! printf '%s' "$inv_004_test_output" | grep -q '^test inv_004_chain_client_does_not_re_export_miner_types'; then
    fail "the new transitive test 'inv_004_chain_client_does_not_re_export_miner_types' \
is not present in 'cargo test -p myosu-play --test invariants inv_004' output. Output:\n$inv_004_test_output"
fi
if ! printf '%s' "$inv_004_test_output" | grep -q '^test inv_004_solver_and_gameplay_bins_do_not_depend_on_each_other'; then
    fail "the pre-existing direct-edge test 'inv_004_solver_and_gameplay_bins_do_not_depend_on_each_other' \
is not present in 'cargo test -p myosu-play --test invariants inv_004' output. Output:\n$inv_004_test_output"
fi
if ! printf '%s' "$inv_004_test_output" | grep -q '^test result: ok'; then
    fail "cargo test -p myosu-play --test invariants inv_004 did not report 'test result: ok'. \
Output:\n$inv_004_test_output"
fi

# Sub-check 4: the type-origin test in myosu-chain-client must pass.
type_origin_test_output="$(SKIP_WASM_BUILD=1 cargo test -p myosu-chain-client --lib inv_004 2>&1 || true)"
if ! printf '%s' "$type_origin_test_output" | grep -q '^test tests::inv_004_public_api_has_no_miner_origin'; then
    fail "the type-origin test 'inv_004_public_api_has_no_miner_origin' is not present in \
'cargo test -p myosu-chain-client --lib inv_004' output. Output:\n$type_origin_test_output"
fi
if ! printf '%s' "$type_origin_test_output" | grep -q '^test result: ok'; then
    fail "cargo test -p myosu-chain-client --lib inv_004 did not report 'test result: ok'. \
Output:\n$type_origin_test_output"
fi

# Sub-check 5 (sanity): the source files touched by NEM-005 must exist
# and carry the new symbols. This is a cheap, fast doc-reg drift guard
# that catches a future refactor which silently renames or moves the
# new code.
if ! [ -f "$inv_004_test_path" ]; then
    fail "$inv_004_test_path is missing"
fi
if ! grep -q 'inv_004_chain_client_does_not_re_export_miner_types' "$inv_004_test_path"; then
    fail "the transitive test name 'inv_004_chain_client_does_not_re_export_miner_types' \
is missing from $inv_004_test_path"
fi
if ! [ -f "$chain_client_lib_path" ]; then
    fail "$chain_client_lib_path is missing"
fi
if ! grep -q 'inv_004_public_api_has_no_miner_origin' "$chain_client_lib_path"; then
    fail "the type-origin test name 'inv_004_public_api_has_no_miner_origin' \
is missing from $chain_client_lib_path"
fi

# Sub-check 6 (sanity): the NEM-005 plan row is marked complete with a
# resolution note. This is the only plan-file touch NEM-005 makes; the
# proof is in the test code, the plan update is just the bookkeeping.
if ! [ -f "$plan_row" ]; then
    fail "$plan_row is missing"
fi
if ! grep -q '### `- \[x\] NEM-005' "$plan_row"; then
    fail "the NEM-005 row in $plan_row is not marked complete (looking for '### \`- [x] NEM-005')"
fi
if ! grep -q 'NEM-005' "$plan_row" || ! grep -q 'inv_004_chain_client_does_not_re_export_miner_types' "$plan_row"; then
    fail "the NEM-005 resolution note in $plan_row does not cross-reference \
the new test name 'inv_004_chain_client_does_not_re_export_miner_types'"
fi

printf 'NEM005_INV_004_TRANSITIVE_HARNESS myosu e2e ok surface=inv_004_transitive_dep_check myosu_chain_client_origin_check=%s reverse_dep_lines=%s plan_row=%s\n' \
    "ok" "$reverse_tree_lines" "[x]"

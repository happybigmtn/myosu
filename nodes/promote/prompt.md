Goal: Implement the next approved `sdk:core` slice.

Inputs:
- `spec.md`
- `review.md`

Scope:
- work only inside the smallest next approved implementation slice
- treat the reviewed lane artifacts as the source of truth
- keep changes aligned with the owned surfaces for `sdk:core`

Required curated artifacts:
- `implementation.md`
- `verification.md`
- `promotion.md`


## Completed stages
- **preflight**: success
  - Script: `set +e
cargo build -p myosu-sdk
cargo test -p myosu-sdk
cargo test -p myosu-sdk -- --include-ignored  # no tests yet, but exits 0
cargo test -p myosu-sdk testing::tests::rps_passes_all_compliance_checks
cargo test -p myosu-sdk testing::tests::broken_game_fails_zero_sum_check
cargo test -p myosu-sdk testing::tests::convergence_test_detects_non_convergence
cargo test -p myosu-sdk scaffold::tests::generates_compilable_crate
cargo test -p myosu-sdk scaffold::tests::generated_tests_fail_with_todo
cargo test -p myosu-sdk scaffold::tests::refuses_to_overwrite_existing_directory
cargo test -p myosu-sdk register::tests::register_help_output
cargo test -p myosu-sdk register::tests::connection_timeout_error
true`
  - Stdout: (empty)
  - Stderr:
    ```
    (8 lines omitted)
    help: a package with a similar name exists: `myosu-tui`
    error: package ID specification `myosu-sdk` did not match any packages
    
    help: a package with a similar name exists: `myosu-tui`
    error: package ID specification `myosu-sdk` did not match any packages
    
    help: a package with a similar name exists: `myosu-tui`
    error: package ID specification `myosu-sdk` did not match any packages
    
    help: a package with a similar name exists: `myosu-tui`
    error: package ID specification `myosu-sdk` did not match any packages
    
    help: a package with a similar name exists: `myosu-tui`
    error: package ID specification `myosu-sdk` did not match any packages
    
    help: a package with a similar name exists: `myosu-tui`
    error: package ID specification `myosu-sdk` did not match any packages
    
    help: a package with a similar name exists: `myosu-tui`
    error: package ID specification `myosu-sdk` did not match any packages
    
    help: a package with a similar name exists: `myosu-tui`
    error: package ID specification `myosu-sdk` did not match any packages
    
    help: a package with a similar name exists: `myosu-tui`
    ```
- **implement**: success
  - Model: MiniMax-M2.7-highspeed, 145 tokens in / 38 out
  - Files: Cargo.lock, Cargo.toml, crates/myosu-sdk/Cargo.toml, crates/myosu-sdk/src/lib.rs, crates/myosu-sdk/src/register/mod.rs, crates/myosu-sdk/src/register/tests.rs, crates/myosu-sdk/src/scaffold/mod.rs, crates/myosu-sdk/src/scaffold/templates.rs, crates/myosu-sdk/src/scaffold/tests.rs, crates/myosu-sdk/src/testing/convergence.rs, crates/myosu-sdk/src/testing/game_valid.rs, crates/myosu-sdk/src/testing/mod.rs, crates/myosu-sdk/src/testing/tests.rs, docs/sdk/quickstart.md, docs/sdk/registration.md, docs/sdk/trait-reference.md, outputs/sdk/core/implementation.md, outputs/sdk/core/verification.md
- **verify**: success
  - Script: `set -e
cargo build -p myosu-sdk
cargo test -p myosu-sdk
cargo test -p myosu-sdk -- --include-ignored  # no tests yet, but exits 0
cargo test -p myosu-sdk testing::tests::rps_passes_all_compliance_checks
cargo test -p myosu-sdk testing::tests::broken_game_fails_zero_sum_check
cargo test -p myosu-sdk testing::tests::convergence_test_detects_non_convergence
cargo test -p myosu-sdk scaffold::tests::generates_compilable_crate
cargo test -p myosu-sdk scaffold::tests::generated_tests_fail_with_todo
cargo test -p myosu-sdk scaffold::tests::refuses_to_overwrite_existing_directory
cargo test -p myosu-sdk register::tests::register_help_output
cargo test -p myosu-sdk register::tests::connection_timeout_error`
  - Stdout:
    ```
    (65 lines omitted)
    
    
    running 0 tests
    
    test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 0.00s
    
    
    running 0 tests
    
    test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 0.00s
    
    
    running 0 tests
    
    test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 0.00s
    
    
    running 0 tests
    
    test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 0.00s
    
    
    running 0 tests
    
    test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 0.00s
    ```
  - Stderr:
    ```
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.26s
        Finished `test` profile [unoptimized + debuginfo] target(s) in 0.26s
         Running unittests src/lib.rs (/home/r/coding/myosu/.worktrees/autodev-live/fabro/programs/../../.raspberry/cargo-target/debug/deps/myosu_sdk-8d77028cfeca24ba)
       Doc-tests myosu_sdk
        Finished `test` profile [unoptimized + debuginfo] target(s) in 0.25s
         Running unittests src/lib.rs (/home/r/coding/myosu/.worktrees/autodev-live/fabro/programs/../../.raspberry/cargo-target/debug/deps/myosu_sdk-8d77028cfeca24ba)
       Doc-tests myosu_sdk
        Finished `test` profile [unoptimized + debuginfo] target(s) in 0.25s
         Running unittests src/lib.rs (/home/r/coding/myosu/.worktrees/autodev-live/fabro/programs/../../.raspberry/cargo-target/debug/deps/myosu_sdk-8d77028cfeca24ba)
        Finished `test` profile [unoptimized + debuginfo] target(s) in 0.25s
         Running unittests src/lib.rs (/home/r/coding/myosu/.worktrees/autodev-live/fabro/programs/../../.raspberry/cargo-target/debug/deps/myosu_sdk-8d77028cfeca24ba)
        Finished `test` profile [unoptimized + debuginfo] target(s) in 0.25s
         Running unittests src/lib.rs (/home/r/coding/myosu/.worktrees/autodev-live/fabro/programs/../../.raspberry/cargo-target/debug/deps/myosu_sdk-8d77028cfeca24ba)
        Finished `test` profile [unoptimized + debuginfo] target(s) in 0.25s
         Running unittests src/lib.rs (/home/r/coding/myosu/.worktrees/autodev-live/fabro/programs/../../.raspberry/cargo-target/debug/deps/myosu_sdk-8d77028cfeca24ba)
        Finished `test` profile [unoptimized + debuginfo] target(s) in 0.24s
         Running unittests src/lib.rs (/home/r/coding/myosu/.worktrees/autodev-live/fabro/programs/../../.raspberry/cargo-target/debug/deps/myosu_sdk-8d77028cfeca24ba)
        Finished `test` profile [unoptimized + debuginfo] target(s) in 0.25s
         Running unittests src/lib.rs (/home/r/coding/myosu/.worktrees/autodev-live/fabro/programs/../../.raspberry/cargo-target/debug/deps/myosu_sdk-8d77028cfeca24ba)
        Finished `test` profile [unoptimized + debuginfo] target(s) in 0.26s
         Running unittests src/lib.rs (/home/r/coding/myosu/.worktrees/autodev-live/fabro/programs/../../.raspberry/cargo-target/debug/deps/myosu_sdk-8d77028cfeca24ba)
        Finished `test` profile [unoptimized + debuginfo] target(s) in 0.26s
         Running unittests src/lib.rs (/home/r/coding/myosu/.worktrees/autodev-live/fabro/programs/../../.raspberry/cargo-target/debug/deps/myosu_sdk-8d77028cfeca24ba)
    ```
- **quality**: success
  - Script: `set -e
QUALITY_PATH='outputs/sdk/core/quality.md'
IMPLEMENTATION_PATH='outputs/sdk/core/implementation.md'
VERIFICATION_PATH='outputs/sdk/core/verification.md'
placeholder_hits=""
scan_placeholder() {
  surface="$1"
  if [ ! -e "$surface" ]; then
    return 0
  fi
  hits="$(rg -n -i -g '*.rs' -g 'Cargo.toml' -g '*.toml' 'TODO|stub|placeholder|future slice|not yet implemented|compile-only|for now|will implement' "$surface" || true)"
  if [ -n "$hits" ]; then
    if [ -n "$placeholder_hits" ]; then
      placeholder_hits="$(printf '%s\n%s' "$placeholder_hits" "$hits")"
    else
      placeholder_hits="$hits"
    fi
  fi
}
true
artifact_hits="$(rg -n -i 'manual proof still required|future slice|compile-only|placeholder|stub implementation|not yet fully implemented' "$IMPLEMENTATION_PATH" "$VERIFICATION_PATH" 2>/dev/null || true)"
warning_hits="$(rg -n 'warning:' "$IMPLEMENTATION_PATH" "$VERIFICATION_PATH" 2>/dev/null || true)"
manual_hits="$(rg -n -i 'manual proof still required|manual;' "$VERIFICATION_PATH" 2>/dev/null || true)"
placeholder_debt=no
warning_debt=no
artifact_mismatch_risk=no
manual_followup_required=no
[ -n "$placeholder_hits" ] && placeholder_debt=yes
[ -n "$warning_hits" ] && warning_debt=yes
[ -n "$artifact_hits" ] && artifact_mismatch_risk=yes
[ -n "$manual_hits" ] && manual_followup_required=yes
quality_ready=yes
if [ "$placeholder_debt" = yes ] || [ "$warning_debt" = yes ] || [ "$artifact_mismatch_risk" = yes ] || [ "$manual_followup_required" = yes ]; then
  quality_ready=no
fi
mkdir -p "$(dirname "$QUALITY_PATH")"
cat > "$QUALITY_PATH" <<EOF
quality_ready: $quality_ready
placeholder_debt: $placeholder_debt
warning_debt: $warning_debt
artifact_mismatch_risk: $artifact_mismatch_risk
manual_followup_required: $manual_followup_required

## Touched Surfaces
- (none declared)

## Placeholder Hits
$placeholder_hits

## Artifact Consistency Hits
$artifact_hits

## Warning Hits
$warning_hits

## Manual Followup Hits
$manual_hits
EOF
test "$quality_ready" = yes`
  - Stdout: (empty)
  - Stderr: (empty)
- **clear_promotion**: success
  - Script: `rm -f outputs/sdk/core/promotion.md`
  - Stdout: (empty)
  - Stderr: (empty)
- **review**: success
  - Model: gpt-5.4, 4.8m tokens in / 36.8k out
  - Files: crates/myosu-sdk/Cargo.toml, crates/myosu-sdk/src/testing/convergence.rs, crates/myosu-sdk/src/testing/game_valid.rs, crates/myosu-sdk/src/testing/mod.rs, crates/myosu-sdk/src/testing/tests.rs, outputs/sdk/core/implementation.md, outputs/sdk/core/promotion.md, outputs/sdk/core/quality.md, outputs/sdk/core/verification.md


# SDK Core Implementation Lane — Promotion

Decide whether `core-implement` is truly merge-ready.


Write `promotion.md` in this exact machine-readable form:

merge_ready: yes|no
manual_proof_pending: yes|no
reason: <one sentence>
next_action: <one sentence>

Only set `merge_ready: yes` when:
- `quality.md` says `quality_ready: yes`
- automated proof is sufficient for this slice
- any required manual proof has actually been performed
- no unresolved warnings or stale failures undermine confidence
- the implementation and verification artifacts match the real code.

Promotion stage ownership:
- you may write or replace `promotion.md` in this stage
- read `quality.md` before deciding `merge_ready`
- prefer not to modify source code here unless a tiny correction is required to make the promotion judgment truthful

Current Slice Contract:
Inspect the relevant repo surfaces, preserve existing doctrine, and produce the lane artifacts honestly.

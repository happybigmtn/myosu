# `sdk:core` Verification

## Lane
`core-implement`

## Slice
Slice 3 — Scaffold Tool (`AC-SDK-02`)

## Automated Proof Commands

All commands were run from the workspace root with a writable Cargo target override because the default shared target path is read-only in this sandbox:

```bash
export CARGO_TARGET_DIR=/tmp/myosu-sdk-slice3-full
```

Focused scaffold checks also used equivalent writable target dirs such as `/tmp/myosu-sdk-slice3-test1b`.

### Slice 3 Proof

```bash
$ cargo test -p myosu-sdk scaffold::tests::generates_compilable_crate
```

- Result: PASS
- Evidence:
  - the generated crate contains `Cargo.toml`, `src/lib.rs`, `src/game.rs`, `src/encoder.rs`, `src/renderer.rs`, `src/tests.rs`, and `README.md`
  - the generated `Cargo.toml` includes `tui = ["myosu-sdk/tui"]`
  - the test shells out to nested `cargo check --offline` and `cargo check --offline --features tui`, and both succeed

```bash
$ cargo test -p myosu-sdk scaffold::tests::generated_tests_fail_with_todo
```

- Result: PASS
- Evidence: the generated crate's nested `cargo test --offline` fails with the scaffolded compliance `todo!()` message, proving the crate compiles and the prewritten test stops at the intended boundary.

```bash
$ cargo test -p myosu-sdk scaffold::tests::refuses_to_overwrite_existing_directory
```

- Result: PASS
- Evidence: generation against a pre-existing directory returns the expected "already exists" error.

```bash
$ cargo test -p myosu-sdk scaffold::tests::generate_uses_default_directory_name
```

- Result: PASS
- Evidence: `ScaffoldGenerator::generate()` creates `myosu-games-<name>` in the current directory and returns that path.

### Regression / Broader Sanity

```bash
$ cargo build -p myosu-sdk
$ cargo test -p myosu-sdk
$ cargo test -p myosu-sdk -- --include-ignored
$ cargo test -p myosu-sdk testing::tests::rps_passes_all_compliance_checks
$ cargo test -p myosu-sdk testing::tests::broken_game_fails_zero_sum_check
$ cargo test -p myosu-sdk testing::tests::convergence_test_detects_non_convergence
$ cargo test -p myosu-sdk scaffold::tests::generates_compilable_crate
$ cargo test -p myosu-sdk scaffold::tests::generated_tests_fail_with_todo
$ cargo test -p myosu-sdk scaffold::tests::refuses_to_overwrite_existing_directory
$ cargo test -p myosu-sdk scaffold::tests::generate_uses_default_directory_name
```

- `cargo build -p myosu-sdk`: PASS
- `cargo test -p myosu-sdk`: PASS
  - 13 unit tests passed
  - doctests: 1 passed, 2 ignored
- `cargo test -p myosu-sdk -- --include-ignored`: PASS
  - 13 unit tests passed
  - doctests: 3 passed
- All three retained Slice 2 proof filters passed after the scaffold work.
- All four focused Slice 3 scaffold filters passed.

## Residual Risks
- The current generic `assert_game_valid` helper still assumes games whose reachable `turn()` value is also the usable info-set surface. The scaffold therefore leaves the intended compliance call as a replacement comment inside the generated test instead of typechecking it immediately.
- The scaffold emits a standalone game crate but does not yet edit the workspace member list for the larger repo. Adding the new crate to the workspace remains a manual step.

## Next Approved Slice
- Slice 4 — Registration CLI (`AC-SDK-04`)

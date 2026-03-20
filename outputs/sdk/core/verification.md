# `sdk:core` Verification

## Lane
`core-implement`

## Slice
Slice 2 — Trait Compliance Test Harness (`AC-SDK-03`)

## Automated Proof Commands

All commands were run from the workspace root with:

```bash
export CARGO_TARGET_DIR=/tmp/myosu-sdk-target
```

The local sandbox points Cargo at a read-only shared target directory by default, so the writable override above was required to get a valid proof run for this slice.

### Slice 2 Proof

```bash
$ cargo test -p myosu-sdk testing::tests::rps_passes_all_compliance_checks
```

- Result: PASS
- Evidence: the filter ran exactly 1 test, and that test passed after validating the RPS game checks and the convergence helper.

```bash
$ cargo test -p myosu-sdk testing::tests::broken_game_fails_zero_sum_check
```

- Result: PASS
- Evidence: the filter ran exactly 1 test, and that test passed by observing the expected validation panic for a deliberately non-zero-sum terminal payoff.

```bash
$ cargo test -p myosu-sdk testing::tests::convergence_test_detects_non_convergence
```

- Result: PASS
- Evidence: the filter ran exactly 1 test, and that test passed by observing the expected convergence failure for an undertrained solver.

### Broader Sanity Check

```bash
$ cargo build -p myosu-sdk
$ cargo test -p myosu-sdk
```

- `cargo build -p myosu-sdk`: PASS
- `cargo test -p myosu-sdk`: PASS
  - 12 unit tests passed
  - doctests: 1 passed, 2 ignored

## Residual Risks
- The generic `assert_game_valid` helper currently relies on the reachable `turn()` value also providing the info-set action surface used during traversal. That matches the reviewed RPS proof target and keeps this slice grounded in the trusted trait surface already present in the repo.
- Later lane slices still own the scaffold, registration, and documentation surfaces. This verification pass did not reopen or re-prove those areas.

## Next Approved Slice
- Slice 3 — Scaffold Tool (`AC-SDK-02`)


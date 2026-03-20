# `play:tui` Verification Artifact

## Proof Commands and Outcomes

This verification used a writable local target directory:

```bash
CARGO_TARGET_DIR=/tmp/myosu-target-play-tui
```

### Proof 1: `myosu-play` builds with blueprint startup resolution

```bash
CARGO_TARGET_DIR=/tmp/myosu-target-play-tui cargo build -p myosu-play
```

**Outcome**: exit 0.

### Proof 2: owned poker renderer surfaces stay green

```bash
CARGO_TARGET_DIR=/tmp/myosu-target-play-tui cargo test -p myosu-games-poker
```

**Outcome**: exit 0. `22` unit tests passed.

### Proof 3: training behavior still passes after blueprint wiring

```bash
CARGO_TARGET_DIR=/tmp/myosu-target-play-tui cargo test -p myosu-play training::tests
```

**Outcome**: exit 0. `6` unit tests passed.

### Proof 4: blueprint loading and lookup pass end-to-end

```bash
CARGO_TARGET_DIR=/tmp/myosu-target-play-tui cargo test -p myosu-play blueprint::tests
```

**Outcome**: exit 0. `6` unit tests passed.

Covered tests:

- `blueprint::tests::load_valid_artifact`
- `blueprint::tests::missing_dir_returns_error`
- `blueprint::tests::schema_mismatch_returns_error`
- `blueprint::tests::hash_mismatch_returns_error`
- `blueprint::tests::lookup_returns_valid_distribution`
- `blueprint::tests::distribution_sums_to_one`

### Proof 5: full `myosu-play` unit suite

```bash
CARGO_TARGET_DIR=/tmp/myosu-target-play-tui cargo test -p myosu-play
```

**Outcome**: exit 0. `12` unit tests passed.

## Slice-4 Gate Coverage

| Gate | Command Coverage | Status |
|------|------------------|--------|
| Valid artifact directory loads as blueprint | `blueprint::tests::load_valid_artifact` | Passed |
| Missing artifact path returns actionable not-found error | `blueprint::tests::missing_dir_returns_error` | Passed |
| Schema mismatch is rejected | `blueprint::tests::schema_mismatch_returns_error` | Passed |
| Hash mismatch is rejected | `blueprint::tests::hash_mismatch_returns_error` | Passed |
| Lookup returns legal actions | `blueprint::tests::lookup_returns_valid_distribution` | Passed |
| Distribution sums to 1.0 | `blueprint::tests::distribution_sums_to_one` | Passed |
| Training slice still builds and tests after startup wiring | `cargo build -p myosu-play`, `training::tests` | Passed |

## Verification Notes

- This slice used automated proof only.
- No warnings were emitted by the proof commands above.

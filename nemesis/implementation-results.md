# Nemesis Implementation Results

## Outcome

This pass closed the Nemesis plan against the live repo instead of the stale
task list.

- `NEM-003`, `NEM-009`, and `NEM-010` required real repo changes in this pass.
- `NEM-004`, `NEM-005`, `NEM-006`, and `NEM-007` were already fixed in live
  `HEAD`; this pass re-verified those proof surfaces and marked the plan
  satisfied instead of pretending they were still open.
- `NEM-001` and `NEM-002` remain blocked.
- `NEM-008` remains deferred because no truthful measurement-backed operator
  recommendation was produced.

## Proof Before Fix

### `NEM-001` GRANDPA finality stall

Direct repro used `/tmp/nemesis_finality_repro.sh`. The surviving authorities
kept importing best blocks after one authority was terminated, but finalized
height stayed pinned at `#2`.

The strongest direct proof was then static: reading
`finality-grandpa 0.16.3` `voter_set.rs` shows the voter threshold is computed
from total weight, and for a 3-authority equal-weight set that threshold is 3,
not 2. That means the requested `2-of-3 keeps finalizing` behavior is not a
small runtime bug waiting for a local patch; it is a mismatch between the plan
contract and the actual GRANDPA quorum math.

### `NEM-003` TotalIssuance delta rationale

The live default-build pallet test already measured the relevant drift:
`stage_0_try_state_delta_stays_well_above_default_epoch_drift`. That let this
pass close the task by documenting the real stage-0 meaning of the 1_000-rao
threshold in `WORKLIST.md`: it is an alert threshold while dust policy is still
open, not a proof that drift is acceptable up to 1_000 rao.

### `NEM-004` through `NEM-007`

These were re-verified directly against the live codebase and targeted tests:

- `NEM-004`: epoch skip event exists and is regression-tested.
- `NEM-005`: validator scoring is already symmetric over the action union.
- `NEM-006`: `Stage0NoopSwap` docs already explain the unbounded `max_price()`.
- `NEM-007`: Liar's Dice already uses the 1 MiB decode cap with oversized-payload tests.

## Root Cause And Changes Made

### Blocked contract work

- `NEM-001`: documented the true GRANDPA threshold problem in `WORKLIST.md` and
  marked the plan blocked.
- `NEM-002`: left blocked behind `NEM-001` because no truthful cross-node
  emission proof was added while the multi-node finality contract is still
  mis-scoped.

### Repo changes landed in this pass

- Updated `WORKLIST.md` `EM-DUST-001` so the stage-0 1_000-rao `try_state`
  delta is explicitly described as an alert threshold.
- Removed stale `DOC-OPS-001` from `WORKLIST.md` after verifying
  [AGENTS.md](/home/r/Coding/myosu/AGENTS.md) has no `@RTK.md` reference.
- Added [ADR 010](/home/r/Coding/myosu/docs/adr/010-inv-004-stage0-ci-enforcement.md)
  to record the explicit stage-0 decision that INV-004 stays enforced by CI and
  invariant tests rather than a runtime assertion.
- Updated [ADR README](/home/r/Coding/myosu/docs/adr/README.md) to index ADR 010.
- Updated [Nemesis plan](/home/r/Coding/myosu/nemesis/IMPLEMENTATION_PLAN.md)
  so satisfied tasks are marked satisfied and blocked/deferred tasks are called
  out explicitly.

## Validation

Commands actually run in this pass:

- `/tmp/nemesis_finality_repro.sh`
- `sed -n '1,120p' /home/r/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/finality-grandpa-0.16.3/src/voter_set.rs`
- `cargo test -p pallet-game-solver --quiet stage_0_try_state_delta_stays_well_above_default_epoch_drift`
- `cargo test -p pallet-game-solver --quiet legacy_epoch_skip_emits_event_when_state_is_inconsistent`
- `cargo test -p myosu-validator --quiet three_action_mismatch_uses_game_agnostic_normalization`
- `cargo test -p myosu-validator --quiet double_count`
- `cargo test -p myosu-chain-runtime --quiet stage0_noop_swap`
- `cargo test -p myosu-games-liars-dice --quiet oversized`
- `cargo test -p myosu-play --quiet inv_004_solver_and_gameplay_bins_do_not_depend_on_each_other`
- `rg -n '@RTK.md' AGENTS.md`
- `rg -n 'train_iterations|--train-iterations|default_value_t *= *0' crates/myosu-miner/src/cli.rs crates/myosu-miner/src/training.rs`

## Deferred And Blocked

- `NEM-001` is blocked until stage-0 re-scopes the multi-node finality
  expectation or changes authority count / voting weights.
- `NEM-002` is blocked until there is a truthful multi-node consensus baseline
  to hang a cross-node emission comparison on.
- `NEM-008` is deferred until someone actually runs iteration-to-score
  measurements and writes a real operator recommendation.

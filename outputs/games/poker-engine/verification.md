# Verification: games:poker-engine (encoder artifact ingress slice)

## Automated proof

Used a writable cargo target dir because the workspace-default shared target path is read-only in this environment.

### Commands run

```bash
CARGO_TARGET_DIR=/tmp/myosu-games-poker-target cargo build -p myosu-games-poker
CARGO_TARGET_DIR=/tmp/myosu-games-poker-target cargo test -p myosu-games-poker
```

### Outcomes

- `cargo build -p myosu-games-poker`: PASS
- `cargo test -p myosu-games-poker`: PASS
- Unit tests: 22 passed, 0 failed, 0 ignored
- Doc tests: 1 ignored example block in `training.rs`
- No compiler warnings were emitted by the successful proof commands

## Newly-covered artifact ingress proofs

- `solver::tests::with_encoder_bytes_accepts_root_artifact`
- `solver::tests::with_encoder_file_accepts_root_artifact`
- `solver::tests::checkpoint_roundtrip_with_encoder_artifact_preserves_epoch`
- `training::tests::new_with_encoder_bytes_accepts_root_artifact`
- `training::tests::new_with_encoder_file_resumes_checkpointed_solver`

## Existing coverage that remained green

- Solver guardrails:
  - `solver::tests::create_empty_solver`
  - `solver::tests::train_100_iterations`
  - `solver::tests::strategy_is_valid_distribution`
  - `solver::tests::checkpoint_roundtrip`
  - `solver::tests::exploitability_decreases`
  - `solver::tests::with_encoder_rejects_empty_abstraction_map`
- Training-session guardrails:
  - `training::tests::session_checkpoint_frequency`
  - `training::tests::new_with_encoder_rejects_empty_abstraction_map`
- Existing query/wire/exploit smoke coverage:
  - all `query::*` tests
  - all `wire::*` tests
  - all `exploit::*` tests

## What this proof establishes

- Poker-engine can ingest a real serialized `NlheEncoder` artifact from bytes or disk through owned `PokerSolver` APIs.
- `TrainingSession` can use the same artifact-backed path and resume a zero-state checkpoint without reaching into robopoker internals.
- The preflop artifact fixture is large enough to satisfy root-level encoder validation because it covers all 169 canonical preflop isomorphisms.

## What this proof does not establish

- Positive encoder-backed training past epoch 0.
- Positive exploitability reduction between trained and untrained NLHE strategies.
- Validator-grade remote exploitability scoring against a full abstraction artifact.

Those proofs remain blocked by upstream randomized `Game::root()` generation and the absence of a full NLHE abstraction artifact in this repo.

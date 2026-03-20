# Verification: games:poker-engine (solver guardrails slice)

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
- Unit tests: 17 passed, 0 failed, 0 ignored
- Doc tests: 1 ignored example block in `training.rs`

## Coverage from the passing suite

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
- Exploitability guardrails:
  - `exploit::tests::trained_strategy_low_exploit`
  - `exploit::tests::random_strategy_high_exploit`
  - `exploit::tests::remote_matches_local`
- Existing wire/query smoke coverage remained green:
  - all `wire::*` tests
  - all `query::*` tests

## What this proof establishes

- The solver wrapper now calls real MCCFR stepping when training is attempted.
- Missing abstraction maps are surfaced as explicit errors instead of panics.
- Zero-state checkpoints roundtrip correctly.
- Training-session checkpointing does not write state after a failed training step.

## What this proof does not establish

- Successful training with a real encoder-backed abstraction artifact.
- Actual exploitability reduction between trained and untrained NLHE strategies.
- Validator-grade parity for remote exploitability scoring.

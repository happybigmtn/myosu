# Implementation: games:poker-engine (solver guardrails slice)

## Slice intent

Tightened the local poker-engine wrapper so it behaves truthfully while real NLHE abstraction artifacts are still absent from the repo. This slice stayed inside the owned poker-engine surfaces and focused on solver, training-session, and exploitability behavior.

## Touched surfaces

- `crates/myosu-games-poker/src/solver.rs`
- `crates/myosu-games-poker/src/training.rs`
- `crates/myosu-games-poker/src/exploit.rs`

## What changed

### `solver.rs`

- Replaced the old `train()` behavior that only built a tree and incremented epochs with real MCCFR stepping via `rbp_mccfr::Solver::step()`.
- Added explicit error surfaces for abstraction failures and bad exploitability outputs:
  - `MissingEncoderAbstractions`
  - `OperationPanicked`
  - `InvalidExploitability`
- Added encoder-aware entry points:
  - `PokerSolver::with_encoder(...)`
  - `PokerSolver::validate_abstractions()`
  - `PokerSolver::load_with_encoder(...)`
- Kept `new()` and `load()` as zero-artifact checkpoint/lookup helpers, but training and exploitability now fail cleanly instead of panicking when the encoder is empty.

### `training.rs`

- Added `TrainingSession::new_with_encoder(...)` so downstream callers can pair checkpoints with a real abstraction artifact.
- Propagates solver failures from `train()` and `exploitability()` instead of hiding them.
- Checkpoint writes only occur after a successful training step.

### `exploit.rs`

- `poker_exploitability()` now returns `EmptyProfile` for zero-epoch profiles and converts missing-abstraction panics into `MissingEncoderAbstractions`.
- `remote_poker_exploitability()` now surfaces the same missing-encoder failure explicitly instead of panicking.

## Test changes

- Removed the ignored unit tests in the touched modules.
- Reframed the touched proof toward the slice’s actual contract:
  - zero-state checkpoint roundtrip remains valid
  - synthetic strategy lookups still return normalized distributions
  - missing abstraction maps fail with explicit errors instead of hidden panics

## Remaining blockers

- This repo still does not provide a real `NlheEncoder` artifact, so this slice does not claim successful encoder-backed training, convergence, or exploitability improvement on actual poker abstractions.
- `remote_poker_exploitability()` still uses the existing simplified synthetic-profile path; this slice only hardened its failure behavior.

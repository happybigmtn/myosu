# Implementation: games:poker-engine (encoder artifact ingress slice)

## Slice intent

This slice makes the poker-engine crate capable of accepting a concrete serialized `NlheEncoder` artifact through owned poker-engine APIs. The work stays inside `myosu-games-poker` and focuses on the next honest step after the guardrail slice: artifact ingress and checkpoint pairing, not simulated end-state training claims.

## Touched surfaces

- `crates/myosu-games-poker/Cargo.toml`
- `crates/myosu-games-poker/src/lib.rs`
- `crates/myosu-games-poker/src/solver.rs`
- `crates/myosu-games-poker/src/training.rs`
- `crates/myosu-games-poker/src/test_support.rs`

## What changed

### `solver.rs`

- Added encoder artifact ingress on the owned `PokerSolver` surface:
  - `PokerSolver::with_encoder_bytes(...)`
  - `PokerSolver::with_encoder_file(...)`
  - `PokerSolver::load_with_encoder_bytes(...)`
  - `PokerSolver::load_with_encoder_file(...)`
- Added explicit artifact error cases:
  - `EncoderArtifactRead`
  - `EncoderArtifactDecode`
- Kept validation centralized in `validate_abstractions()` so byte/file-backed constructors reject unusable encoder artifacts with the same error surface as direct `with_encoder(...)`.

### `training.rs`

- Added artifact-backed `TrainingSession` constructors:
  - `TrainingSession::new_with_encoder_bytes(...)`
  - `TrainingSession::new_with_encoder_file(...)`
- Added `TrainingError::EncoderLoad` so callers can distinguish artifact ingress failures from checkpoint load/save failures.
- Verified the zero-state checkpoint resume path works when the session is paired with a concrete encoder artifact.

### `test_support.rs`

- Added a poker-engine-owned test fixture that produces a real serialized `NlheEncoder` artifact for preflop validation.
- The fixture materializes all 169 canonical preflop isomorphisms and assigns them to a deterministic preflop abstraction bucket, which is enough to prove root-level encoder validation and zero-state checkpoint pairing without pretending to cover full NLHE training.

## Why this is the smallest honest next slice

- The previous slice established clean failure behavior when abstraction data is missing.
- Downstream miner and validator code still had no poker-engine-owned way to ingest a concrete encoder artifact without reaching into robopoker internals.
- This slice closes that gap and proves the byte/file-backed path works end-to-end on the owned surfaces.

## Remaining blockers

- Positive encoder-backed training past epoch 0 is still blocked by upstream randomized `Game::root()` generation plus the absence of a full NLHE abstraction artifact in this repo. A preflop-only fixture cannot honestly prove replayable training across random roots.
- Positive exploitability improvement proof remains blocked for the same reason: `exploitability()` builds a broader tree than the preflop fixture covers.

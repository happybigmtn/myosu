# Promotion: games:poker-engine

## Decision

Promote this slice to the next implementation step. Do not mark the overall poker-engine lane complete yet.

## Why this slice is promotable

- `outputs/games/poker-engine/quality.md` remains `quality_ready: yes`.
- `cargo build -p myosu-games-poker` passed.
- `cargo test -p myosu-games-poker` passed with 17 unit tests green and no ignored unit tests.
- The slice fixes a real local correctness issue: `PokerSolver::train()` now uses `Solver::step()` instead of only advancing epochs.
- The slice also replaces panic-driven missing-encoder failures with explicit errors that downstream miner and validator code can handle deliberately.

## Why the lane is not complete

- The repo still lacks a real NLHE abstraction artifact, so encoder-backed training and exploitability-improvement proof remain outside the evidence produced here.
- `remote_poker_exploitability()` still needs a stronger synthetic-profile story before it can serve as end-state validator scoring evidence.

## Recommended next slice

Thread a concrete encoder artifact or fixture into poker-engine tests and then restore proof for:

- successful training past epoch 0 with a real encoder
- checkpoint resume with encoder-backed state
- measured exploitability comparisons between trained and untrained strategies

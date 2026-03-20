Implemented the next poker-engine slice in [solver.rs](/home/r/.fabro/runs/20260320-01KM4XW4EPWKGT5TC5ATPE8TGD/worktree/crates/myosu-games-poker/src/solver.rs), [training.rs](/home/r/.fabro/runs/20260320-01KM4XW4EPWKGT5TC5ATPE8TGD/worktree/crates/myosu-games-poker/src/training.rs), and [exploit.rs](/home/r/.fabro/runs/20260320-01KM4XW4EPWKGT5TC5ATPE8TGD/worktree/crates/myosu-games-poker/src/exploit.rs). The key change is that `PokerSolver::train()` now uses real MCCFR stepping, while missing NLHE abstraction data is surfaced as explicit errors instead of panic-driven behavior. I also added encoder-aware entry points like `with_encoder`, `load_with_encoder`, and `TrainingSession::new_with_encoder`.

The curated artifacts are updated at [implementation.md](/home/r/.fabro/runs/20260320-01KM4XW4EPWKGT5TC5ATPE8TGD/worktree/outputs/games/poker-engine/implementation.md), [verification.md](/home/r/.fabro/runs/20260320-01KM4XW4EPWKGT5TC5ATPE8TGD/worktree/outputs/games/poker-engine/verification.md), and [promotion.md](/home/r/.fabro/runs/20260320-01KM4XW4EPWKGT5TC5ATPE8TGD/worktree/outputs/games/poker-engine/promotion.md). The promotion call is: promote this slice, but do not mark the overall lane complete until a real `NlheEncoder` artifact is threaded into proof.

Verification passed with:
`CARGO_TARGET_DIR=/tmp/myosu-games-poker-target cargo build -p myosu-games-poker`
and
`CARGO_TARGET_DIR=/tmp/myosu-games-poker-target cargo test -p myosu-games-poker`

Result: 17 unit tests passed, 0 failed, 0 ignored.
Implemented Slice 1 for `tui:shell` in [events.rs](/home/r/.fabro/runs/20260320-01KM755GFB9SS6HFPKZYFWVJDP/worktree/crates/myosu-tui/src/events.rs). The public `EventLoop::new()` path stays intact, but the module now has an internal injectable event-stream seam plus headless tests for tick, key, resize, async update, and cloned update senders, replacing the ignored TTY-only proof. I also added the curated implement-stage artifacts [implementation.md](/home/r/.fabro/runs/20260320-01KM755GFB9SS6HFPKZYFWVJDP/worktree/outputs/tui/shell/implementation.md), [verification.md](/home/r/.fabro/runs/20260320-01KM755GFB9SS6HFPKZYFWVJDP/worktree/outputs/tui/shell/verification.md), and [integration.md](/home/r/.fabro/runs/20260320-01KM755GFB9SS6HFPKZYFWVJDP/worktree/outputs/tui/shell/integration.md).

Verification passed with:
- `CARGO_TARGET_DIR=/tmp/myosu-cargo-target cargo test -p myosu-tui events:: -- --include-ignored`
- `CARGO_TARGET_DIR=/tmp/myosu-cargo-target cargo test -p myosu-tui shell::`
- `CARGO_TARGET_DIR=/tmp/myosu-cargo-target cargo test -p myosu-tui all_game_types_have_schema`
- `CARGO_TARGET_DIR=/tmp/myosu-cargo-target cargo test -p myosu-tui shell_draw_`
- `CARGO_TARGET_DIR=/tmp/myosu-cargo-target cargo test -p myosu-tui is_plain_text`

`quality.md` and `promotion.md` were not authored because this lane’s stage ownership reserves them for the Quality Gate and Review stages. I left the unrelated pre-existing edits in the chain pallet files untouched.
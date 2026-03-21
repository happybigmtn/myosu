Fixed the current `tui:shell` fixup without widening past Slice 1. [spec.md](/home/r/.fabro/runs/20260320-01KM77EQZAB3EH21TY8VTT8988/worktree/outputs/tui/shell/spec.md) now exposes only the active Slice 1 command as a live `**Proof gate**`, and marks later slice commands as `**Planned proof gate**` so verification stays aligned with the approved scope. I also updated [implementation.md](/home/r/.fabro/runs/20260320-01KM77EQZAB3EH21TY8VTT8988/worktree/outputs/tui/shell/implementation.md), [verification.md](/home/r/.fabro/runs/20260320-01KM77EQZAB3EH21TY8VTT8988/worktree/outputs/tui/shell/verification.md), and [integration.md](/home/r/.fabro/runs/20260320-01KM77EQZAB3EH21TY8VTT8988/worktree/outputs/tui/shell/integration.md) to reflect that this was a proof-contract repair, not a new source edit. `crates/myosu-tui/src/events.rs` was left unchanged because the Slice 1 code already passed.

Verified with:
- `env CARGO_TARGET_DIR=/tmp/myosu-cargo-target cargo test -p myosu-tui events::` -> 4 passed
- `env CARGO_TARGET_DIR=/tmp/myosu-cargo-target cargo test -p myosu-tui schema::tests::all_game_types_have_schema` -> 1 passed
- `env CARGO_TARGET_DIR=/tmp/myosu-cargo-target cargo test -p myosu-tui shell::tests::shell_draw_` -> 2 passed
- `env CARGO_TARGET_DIR=/tmp/myosu-cargo-target cargo test -p myosu-tui pipe::tests::is_plain_text_detects_ansi` -> 1 passed

I did not author `quality.md` or `promotion.md`, per the lane ownership rules. The next approved code slice is still the `shell.rs` Lobby-to-Game integration test.
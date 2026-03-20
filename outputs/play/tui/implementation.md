# `play:tui` Implementation Artifact

## Slice Coverage

This run implemented the next approved `play:tui` slice:

| Slice | Surface | Status | Files |
|-------|---------|--------|-------|
| Slice 3 | `TrainingTable` + `HeuristicBackend` | Done with one explicit command constraint | `crates/myosu-play/src/training.rs`, `crates/myosu-play/src/main.rs`, `crates/myosu-play/Cargo.toml`, `Cargo.lock` |

Slices 1 and 2 remain in place from the earlier lane run. Blueprint loading, advisor wiring, recorder work, and chain-backed play remain outside this implementation.

## What Landed

### `crates/myosu-play/src/training.rs`

Added a real training engine on top of robopoker gameplay types:

- `TrainingTable` now owns the current `Game`, hand root, action history, practice chip balance, hand number, pending training commands, board script, and bot backend.
- `SessionRecall` implements `rbp_gameplay::Recall` against the actual hand root plus applied action history, so the bot sees the same state the table is advancing.
- `BotBackend` is now the slice contract for local training bots: `strategy_name`, `action_distribution`, and `select_action`.
- `HeuristicBackend` produces legal weighted action distributions from live game state. It uses a simple preflop hand score plus postflop strength buckets to choose among fold, check, call, raise, and shove.
- `TrainingTable::renderer()` converts live table state into the existing `NlheRenderer` state model so the training slice can feed the already-approved TUI surfaces.
- `bot_delay_from_env()` reads `MYOSU_BOT_DELAY_MS`, and the async training advance path uses `tokio::time::sleep` before bot actions instead of a blocking pause.

### Training Commands

The slice now supports the commands that fit the current public robopoker API:

- `/deal A♠ K♥` sets hero hole cards for the next hand.
- `/board Q♥ J♥ 9♦` scripts the next hand’s community cards as streets are revealed.
- `/showdown` forces a passive runout so the hand resolves without more aggressive betting.

The command parser normalizes Unicode suit input to robopoker’s ASCII parser, so the spec examples work as written.

### One Explicit Constraint Inside This Slice

`/stack` and `/bot-stack` are wired as clear runtime errors instead of silent no-ops. The current `rbp_gameplay::Game` API exposes no public stack setters, so this slice stops at the honest boundary rather than faking support.

### `crates/myosu-play/src/main.rs`

`myosu-play --train` now constructs a `TrainingTable`, resolves bot delay through the environment-aware helper, advances the table to the first hero decision, snapshots the renderer from live table state, and logs the current bot strategy plus practice chip balance into the shell transcript before the shell loop starts.

This keeps the entrypoint aligned with Slice 3 without reaching into blueprint, advisor, recorder, or chain work.

### Dependency Surface

`crates/myosu-play/Cargo.toml` and `Cargo.lock` now carry the direct training dependencies required by the slice:

- `rbp-cards`
- `rbp-gameplay`
- `rand`

## Automated Coverage Added in This Slice

`training.rs` now carries six focused unit tests:

- `hand_completes_fold`
- `hand_completes_showdown`
- `deal_command_sets_cards`
- `bot_backend_fallback`
- `practice_chips_update`
- `alternating_button`

These cover the minimum proof the lane spec asked for, plus the additional slice-specific behaviors that were straightforward to prove automatically.

## Remaining Blockers Before the Next `play:tui` Slice

- Stack override commands need either upstream public setters in `rbp_gameplay::Game` or a local wrapper that can rebuild stack-sized hand roots safely.
- The current shell still renders a snapshot renderer created at training startup; live in-shell state mutation belongs to the next round of TUI/game-loop plumbing.
- Blueprint-backed bot play, advisor rendering, recorder output, and chain discovery are still untouched by this run.

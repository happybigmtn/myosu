# Wire poker renderer into TUI shell Lane — Review

Review only the current slice for `tui-full-implementation-wire-poker-renderer`.

Current Slice Contract:
Plan file:
- `genesis/plans/009-tui-full-implementation.md`

Child work item: `tui-full-implementation-wire-poker-renderer`

Full plan context (read this for domain knowledge, design decisions, and specifications):

# TUI Full Implementation

**Plan ID:** 009
**Status:** New
**Priority:** FOUNDATION — primary human-facing surface

`genesis/PLANS.md` at `genesis/PLANS.md` governs this document.

---

## Purpose / Big Picture

After this plan lands, the TUI will be a fully playable NLHE gameplay terminal. A human can start the TUI, see their hole cards, see the board, see the pot, see the recommended action, and play a complete hand against the best bot strategy. The five-panel shell layout will be wired to the poker renderer. Pipe mode will work end-to-end for agent interaction.

---

## Progress

- [ ] Fix `myosu-play` binary so it actually enters the terminal loop (not compile-only)
- [ ] Wire poker renderer into TUI shell
- [ ] Implement all poker screens: title, new game, play, hand history, settings
- [ ] Implement pipe mode end-to-end for agent protocol
- [ ] Implement theme system with poker-specific colors
- [ ] Add comprehensive integration tests

---

## Surprises & Discoveries

*(To be written during implementation)*

---

## Decision Log

- Decision: The `GameRenderer` trait is the integration contract between the game engine and the TUI.
  Rationale: This was already designed in `specs/031626-07-tui-implementation.md`. The TUI should never know about poker-specific rendering — it delegates to the `GameRenderer` impl.
  Date/Author: 2026-03-21 / Interim CEO

- Decision: Pipe mode is for agents, not humans. Humans use the interactive TUI.
  Rationale: The TUI has two modes. Interactive (normal) is for human play. Pipe mode (non-interactive) is for agents that consume/produce JSON game state via stdin/stdout.
  Date/Author: 2026-03-21 / Interim CEO

---

## Outcomes & Retrospective

*(To be written at plan completion)*

---

## Milestones

### M1: Fix `myosu-play` entry point
The current binary initializes the shell but exits without running the event loop. Fix it so `myosu-play train` enters the interactive terminal loop.

Proof: `cargo run -p myosu-play -- train` enters a blocking terminal loop. `kill %1` exits cleanly.

Key files:
- `crates/myosu-play/src/main.rs` — binary entry point
- `crates/myosu-tui/src/events.rs` — event loop
- `crates/myosu-tui/src/shell.rs` — shell state machine

### M2: Wire poker renderer into five-panel shell
Implement `GameRenderer` for NLHE in `crates/myosu-games-poker/src/renderer.rs`. Wire it so the state panel shows hole cards, board, pot, positions.

Proof: Running the TUI shows poker state in the state panel, not placeholder text.

Key files:
- `crates/myosu-games-poker/src/renderer.rs`
- `crates/myosu-tui/src/screens.rs`

### M3: Implement all poker screens
Implement: title screen, new game (stakes, positions), play screen, hand history, settings.

Proof: Tab navigation between all screens works; all screens render without panics.

Key files:
- `crates/myosu-tui/src/screens.rs` — screen management
- `crates/myosu-games-poker/src/renderer.rs` — poker-specific rendering

### M4: Implement pipe mode end-to-end
Pipe mode reads JSON game state from stdin, queries strategy, writes action to stdout.

Proof: `echo '{"game_state": {...}}' | cargo run -p myosu-play -- pipe | jq '.action'` returns a valid action.

### M5: Playable NLHE demo
A human can play a complete heads-up NLHE hand: deal, bet, showdown.

Proof: Run `cargo run -p myosu-play`, start a new game, play through one complete hand (preflop → flop → turn → river → showdown), observe the correct recommended action at each decision point, and exit cleanly.

---

## Context and Orientation

Current TUI state:
```
crates/myosu-tui/
├── src/
│   ├── lib.rs          # module declarations
│   ├── events.rs      # event loop (WORKS)
│   ├── input.rs       # input handling (WORKS)
│   ├── pipe.rs        # pipe mode (SCAFFOLD)
│   ├── renderer.rs    # Renderable + GameRenderer traits (WORKS)
│   ├── schema.rs      # Schema definitions (WORKS)
│   ├── screens.rs     # Screen management (PARTIAL)
│   ├── shell.rs       # Five-panel shell (WORKS)
│   └── theme.rs       # Theme styling (WORKS)
└── 0 test files

crates/myosu-play/src/main.rs
└── Binary that initializes shell but exits without loop
```

The five-panel shell layout:
```
┌────────────────────────────────────────────┐
│ HEADER: Myosu — NLHE vs Best Bot          │
├────────────────────────────────────────────┤
│ TRANSCRIPT: hand history + bot actions     │
├────────────────────────────────────────────┤
│ STATE: hole cards + board + pot + positions│
├────────────────────────────────────────────┤
│ DECLARATION: recommended action            │
├────────────────────────────────────────────┤
│ INPUT: player action (fold/call/raise)    │
└────────────────────────────────────────────┘
```

---

## Plan of Work

1. Fix the `myosu-play` binary entry point to call `shell.run()`
2. Implement `GameRenderer<NLHE>` in `myosu-games-poker`
3. Add poker screens to `screens.rs`
4. Wire the strategy query into the declaration panel
5. Implement pipe mode JSON protocol
6. Run an end-to-end playable hand

---

## Concrete Steps

```bash
# Check current entry point
cat crates/myosu-play/src/main.rs

# Fix: change shell initialization to shell.run()
# Current (broken):
#   let shell = Shell::new();
#   shell.init();
#   // missing: shell.run()
# Fixed:
#   let shell = Shell::new();
#   shell.init();
#   shell.run(); // blocks until exit

# Run the binary
cargo run -p myosu-play -- train
# Should block at the TUI screen, not exit immediately

# Test pipe mode
cat > /tmp/test_poker_state.json << 'EOF'
{"game":"nlhe","phase":"flop","hole":["As","Kd"],"board":["Qh","Jc","2s"],"pot":150,"position":"BTN"}
EOF
cat /tmp/test_poker_state.json | cargo run -p myosu-play -- pipe 2>/dev/null
# Expected: JSON with .action field

# Play a complete hand (manual test)
cargo run -p myosu-play -- train
# Type: n (new game)
# Type: 1 (stake: 100bb)
# Type: f (fold preflop — if fold is recommended)
# Or: c (call), r (raise amount)
# Repeat for flop, turn, river
# Observe showdown
# Type: q (quit)
```

---

## Validation

- `cargo run -p myosu-play -- train` enters a blocking terminal loop (not immediate exit)
- The five-panel shell shows poker state (hole cards, board, pot, positions)
- Pipe mode: input JSON → action JSON
- Complete hand: preflop → flop → turn → river → showdown without panics
- `cargo test -p myosu-tui` shows ≥ 20 passing integration tests
- `cargo test -p myosu-games-poker` shows ≥ 50 passing tests

---

## Failure Scenarios

| Scenario | Handling |
|----------|----------|
| Ratatui rendering differs in CI vs local terminal | Use `ratatui::mock::MockTerminal` for CI tests; local tests require real terminal |
| Strategy query blocks indefinitely | Add timeout to strategy query; show "computing..." in declaration panel after 100ms |
| Unicode card symbols rendering incorrectly | Use ASCII fallback: `Ah` instead of `A♥` for non-UTF8 terminals |


Workflow archetype: implement

Review profile: ux

Active plan:
- `genesis/plans/001-master-plan.md`

Active spec:
- `genesis/SPEC.md`

AC contract:
- Where: GameRenderer<NLHE> implementation and shell state panel integration
- How: State panel shows hole cards, board, pot, and positions via GameRenderer trait
- Required tests: cargo test -p myosu-games-poker -- renderer
- Verification plan: Running the TUI shows poker state in the state panel, not placeholder text
- Rollback condition: State panel shows placeholder text or panics on render

Proof commands:
- `cargo build -p myosu-games-poker`
- `cargo test -p myosu-games-poker`

Artifacts to write:
- `spec.md`
- `review.md`


Verification artifact must cover
- summarize the automated proof commands that ran and their outcomes

Nemesis-style security review
- Pass 1 — first-principles challenge: question trust boundaries, authority assumptions, and who can trigger the slice's dangerous actions
- Pass 2 — coupled-state review: identify paired state or protocol surfaces and check that every mutation path keeps them consistent or explains the asymmetry
- check secret handling, capability scoping, pairing/idempotence behavior, and privilege escalation paths

Focus on:
- slice scope discipline
- proof-gate coverage for the active slice
- touched-surface containment
- implementation and verification artifact quality
- remaining blockers before the next slice

Deterministic evidence:
- treat `quality.md` as machine-generated truth about placeholder debt, warning debt, manual follow-up, and artifact mismatch risk
- if `quality.md` says `quality_ready: no`, do not bless the slice as merge-ready


Write `promotion.md` in this exact machine-readable form:

merge_ready: yes|no
manual_proof_pending: yes|no
reason: <one sentence>
next_action: <one sentence>

Only set `merge_ready: yes` when:
- `quality.md` says `quality_ready: yes`
- automated proof is sufficient for this slice
- any required manual proof has actually been performed
- no unresolved warnings or stale failures undermine confidence
- the implementation and verification artifacts match the real code.

Review stage ownership:
- you may write or replace `promotion.md` in this stage
- read `quality.md` before deciding `merge_ready`
- when the slice is security-sensitive, perform a Nemesis-style pass: first-principles assumption challenge plus coupled-state consistency review
- include security findings in the review verdict when the slice touches trust boundaries, keys, funds, auth, control-plane behavior, or external process control
- prefer not to modify source code here unless a tiny correction is required to make the review judgment truthful

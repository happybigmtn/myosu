# TUI Full Implementation

**Plan ID:** 009
**Status:** In Progress
**Priority:** FOUNDATION — primary human-facing surface

`genesis/PLANS.md` at `genesis/PLANS.md` governs this document.

---

## Purpose / Big Picture

After this plan lands, the TUI will be a fully playable NLHE gameplay terminal.
A human can start the TUI, see their hole cards, see the board, see the pot,
see the recommended action, and play a complete hand against the best bot
strategy. The five-panel shell layout will be wired to the poker renderer. Pipe
mode will work end-to-end for agent interaction.

---

## Progress

- [x] Add a real `myosu-play` workspace crate with `train` and `pipe` entrypoints
- [x] Fix the TUI entry path so `myosu-play train` actually enters a blocking terminal loop
- [x] Wire poker renderer into the TUI shell for an honest generated-card demo hand
- [x] Extend the poker demo from one fixed snapshot to multi-street progression
- [x] Prove a stateful pipe loop for agent-style text interaction across one full demo hand
- [x] Restore a truthful lobby-to-game entry path for `train`
- [x] Surface query-backed demo advice in the declaration panel and pipe state
- [x] Allow `new` to restart into the next generated demo hand after completion
- [x] Add an optional artifact-backed advice path for `train` and `pipe`
- [x] Expose the advice source (`generated` vs `artifact`) in the play surface
- [x] Evaluate generated river showdowns and report the actual winner
- [x] Expose evaluated showdown strengths in the completed hand state
- [x] Make the shell header follow the live hand number across demo restarts
- [x] Emit shell metadata alongside each pipe-frame state update
- [x] Include legal-action hints in pipe `CLARIFY` and `ERROR` responses
- [x] Stop reprinting unchanged pipe frames after `CLARIFY` and `ERROR`
- [x] Make advertised pipe quit controls exit cleanly
- [x] Stop treating blank pipe input as EOF
- [x] Advertise `/quit` consistently in active-state pipe actions and recovery hints
- [x] Advertise `/quit` consistently in idle no-hand poker state
- [x] Auto-discover local blueprint artifacts before falling back to generated advice
- [x] Emit a pipe startup `INFO ...` line that exposes advice selection details
- [x] Preserve incomplete standard blueprint roots as explicit fallback detail
- [x] Expose discovery-root provenance in pipe startup info
- [x] Expose stable advice-selection reason codes in pipe startup info
- [x] Drive train startup summary from the same structured advice metadata
- [x] Expose structured selection/origin/reason tags in train startup summary
- [x] Emit pipe startup tags as plain tokens instead of debug-quoted strings
- [x] Expose the current pipe protocol name in startup info
- [x] Version the current pipe startup contract explicitly
- [x] Auto-load the local codexpoker blueprint export when it exists
- [ ] Implement all poker screens: title, new game, play, hand history, settings
- [ ] Implement pipe mode end-to-end for the real gameplay protocol
- [ ] Implement theme system with poker-specific colors
- [ ] Add comprehensive integration tests beyond the current `myosu-play` smoke surface

---

## Surprises & Discoveries

- Discovery: The current blocker was not the shell layout.
  The shell, renderer trait, and poker renderer were already in much better
  shape than the original plan assumed.
  Date/Author: 2026-03-28 / Codex

- Discovery: The active repo did not have an honest `myosu-play` crate at all.
  The missing binary crate and terminal loop were the real first gap, not
  renderer wiring from scratch.
  Date/Author: 2026-03-28 / Codex

- Discovery: The first truthful pipe boundary is plain-text command/response
  over a fixed sample snapshot, not full JSON gameplay orchestration.
  That is enough to prove the CLI seam without pretending the full protocol
  exists already.
  Date/Author: 2026-03-28 / Codex

- Discovery: The next blocker after booting `myosu-play` was not terminal
  ownership but mutation ownership.
  The shell could already parse input, but accepted actions did not update the
  renderer state. Refreshing completions after accepted input and giving the
  poker renderer a tiny deterministic state machine was enough to create a real
  street-to-street demo.
  Date/Author: 2026-03-28 / Codex

- Discovery: The next screen-flow gap was smaller than a full screen rewrite.
  The shell already had a working lobby transition path; it just needed truthful
  non-game header/declaration rendering and a lobby-first `train` entrypoint.
  Date/Author: 2026-03-28 / Codex

- Discovery: The next recommendation seam did not require pretending the solver
  was already live.
  A deterministic advice table tied to the demo hand is enough to prove where
  recommendation text belongs in both the declaration panel and the pipe state.
  Date/Author: 2026-03-28 / Codex

- Discovery: A demo session is more truthful than a demo dead end.
  Once the hand-complete state existed, the next honest step was letting the
  operator start another hand with `new`, even before true hand generation
  exists.
  Date/Author: 2026-03-28 / Codex

- Discovery: Replacing the fixed-card hand did not require claiming full random
  gameplay or real solver artifacts.
  A generated-card runout plus query-backed per-snapshot weighting is enough to
  turn the play wedge into a changing session while staying honest about the
  remaining gap to artifact-backed live solver advice.
  Date/Author: 2026-03-28 / Codex

- Discovery: The repo already had enough local pieces for optional real advice
  loading.
  `load_encoder_dir(...)`, `PokerSolver::load(...)`, and blueprint snapshotting
  were already present. The missing seam was simply wiring them into
  `myosu-play` and the renderer behind an explicit fallback boundary.
  Date/Author: 2026-03-28 / Codex

- Discovery: Once both advice paths existed, the next honesty bug was source
  ambiguity rather than missing code.
  The play surface needed to say whether a recommendation came from generated
  demo weighting or an artifact-backed blueprint. Exposing that directly in the
  declaration text, pipe state, and startup log keeps the seam explicit.
  Date/Author: 2026-03-28 / Codex

- Discovery: Generated cards made showdown truthfulness matter immediately.
  Once the river board and both hole-card sets were real, a generic
  `result=showdown` line was no longer enough. Evaluating the actual 7-card
  hands and reporting `winner=hero|villain|split` keeps completion state aligned
  with the cards already on screen.
  Date/Author: 2026-03-28 / Codex

- Discovery: Winner-only showdown output still hid useful truth.
  Once hand evaluation existed, exposing the normalized hero and villain
  strengths in pipe output made the completion surface much easier to audit and
  debug without widening into a full protocol redesign.
  Date/Author: 2026-03-28 / Codex

- Discovery: The next honesty bug after showdown truth was smaller than the
  gameplay protocol.
  The shared shell header was still freezing on the initial demo hand label.
  Making `context_label()` owned and state-derived let the visible TUI context
  track the real hand number across `new` transitions.
  Date/Author: 2026-03-28 / Codex

- Discovery: The next pipe improvement did not need a new gameplay protocol.
  The shell already knew the current game label, context label, and
  declaration text. Emitting that as a `META ...` line ahead of each `STATE`
  line made the text contract more explicit without pretending the final
  protocol exists.
  Date/Author: 2026-03-28 / Codex

- Discovery: The next protocol weakness was recovery, not transport.
  `ERROR invalid input` and a bare clarification prompt forced the caller to
  wait for the next state frame to recover. Including `legal=...` on `CLARIFY`
  and `ERROR` lines makes the text contract more self-correcting without
  widening into a new protocol design.
  Date/Author: 2026-03-28 / Codex

- Discovery: Once recovery hints existed, the next weakness was duplicate
  frames rather than missing information.
  Reprinting the exact same `META` and `STATE` lines after `CLARIFY` or
  `ERROR` only added noise. The tighter rule is simpler: only successful
  actions emit a fresh frame because only they mutate renderer state.
  Date/Author: 2026-03-28 / Codex

- Discovery: The next pipe bug was a contract mismatch, not a formatting
  problem.
  Completed-hand state already advertised `new|/quit`, but `/quit` still
  returned an error. Treating `/quit` and `quit` as explicit pipe-level exit
  controls makes the text contract match the surfaced actions.
  Date/Author: 2026-03-28 / Codex

- Discovery: The next pipe failure mode was input-shape ambiguity, not action
  parsing.
  Blank input was being collapsed into loop termination because the pipe reader
  treated empty lines like EOF. Returning blank lines distinctly and ignoring
  them in the loop keeps idle input from looking like disconnect.
  Date/Author: 2026-03-28 / Codex

- Discovery: Once quit worked globally, the next mismatch was discoverability.
  Active-state pipe frames still advertised only betting actions even though
  `/quit` was accepted everywhere. Appending `/quit` to active `actions=` and
  `legal=` hints makes the surfaced controls consistent across success, error,
  and completion cases.
  Date/Author: 2026-03-28 / Codex

- Discovery: The same quit discoverability bug also existed in the no-hand
  poker renderer.
  The static inactive path still exposed plain `quit` through completions and a
  bare `STATE idle`. Switching that surface to `/quit` and `STATE idle
  actions=/quit` makes the idle contract consistent with the live pipe loop.
  Date/Author: 2026-03-28 / Codex

- Discovery: The next advice improvement did not require bundled artifacts in
  the repo.
  The specs already define standard artifact locations, and the current repo
  simply does not have matching files checked in. That made auto-discovery plus
  honest generated fallback the right next step: prefer real local blueprint
  assets when present, but stay explicit when they are absent or fail to load.
  Date/Author: 2026-03-28 / Codex

- Discovery: The first auto-discovery pass still missed one of the spec's
  documented roots.
  `MYOSU_DATA_DIR/.myosu/blueprints/` is part of the intended search order, so
  adding it ahead of the home-directory fallback made the implementation align
  better with the existing blueprint-loading doctrine.
  Date/Author: 2026-03-28 / Codex

- Discovery: Honest fallback logic still needed one more machine-visible seam.
  `train` startup logs already said why advice was generated or artifact-backed,
  but pipe callers still had to infer that from later state. Emitting an
  upfront `INFO ...` line with source, selection mode, and detail makes the
  fallback reason explicit without pretending the pipe contract is finalized.
  Date/Author: 2026-03-28 / Codex

- Discovery: The next fallback bug was diagnostic flattening, not search order.
  A standard blueprint root could exist but be incomplete, and the old logic
  still reported that as "no local blueprint artifacts found." Preserving the
  first incomplete-root detail, while still preferring any later valid root,
  makes the generated fallback much more actionable.
  Date/Author: 2026-03-28 / Codex

- Discovery: Once fallback detail was explicit, provenance still mattered.
  A machine caller also needs to know which standard root actually won or
  failed. Adding `origin="explicit|env|data|repo|home|none"` to the startup
  `INFO ...` line makes the discovery path inspectable without widening the
  rest of the pipe contract.
  Date/Author: 2026-03-28 / Codex

- Discovery: Provenance still left one machine-facing gap.
  A caller should not have to parse English detail text to distinguish
  `missing`, `incomplete`, `load_failed`, `auto_loaded`, or `explicit`.
  Adding a stable `reason=...` field to startup info makes fallback handling
  simpler while keeping the protocol small.
  Date/Author: 2026-03-28 / Codex

- Discovery: Human and machine startup truth should come from the same source.
  Once pipe mode had structured `origin` and `reason` fields, the TUI startup
  log should not keep its own separate summary wording. Deriving the train-mode
  summary from the same structured advice selection avoids another honesty
  drift between surfaces.
  Date/Author: 2026-03-28 / Codex

- Discovery: Shared startup truth still needed the same visible tags.
  A shared source of truth helps, but humans still benefit from seeing
  `selection`, `origin`, and `reason` directly instead of only inside prose.
  Adding those tags to the train-mode summary keeps the shell log aligned with
  the pipe startup contract.
  Date/Author: 2026-03-28 / Codex

- Discovery: Structured pipe tags still needed cleaner token syntax.
  `advice_source`, `selection`, `origin`, and `reason` are enum-like values,
  so emitting them as plain tokens is more consistent with the existing
  `STATE ...` surface than debug-quoted strings. Only the free-form
  `detail="..."` field needs quoting.
  Date/Author: 2026-03-28 / Codex

- Discovery: The startup contract should name its own maturity boundary.
  Pipe mode is still the current text demo seam, not the final gameplay
  protocol. Adding `protocol=text_demo` to startup info makes that explicit to
  callers without requiring outside documentation.
  Date/Author: 2026-03-28 / Codex

- Discovery: Naming the protocol still left shape changes implicit.
  Once startup info became a real contract, it also needed a stable version
  tag. Adding `protocol_version=1` gives callers a clean compatibility hook
  without pretending the final gameplay protocol is here already.
  Date/Author: 2026-03-28 / Codex

- Discovery: The missing default artifact path was not hypothetical after all.
  A complete local codexpoker blueprint export already exists under
  `~/.codexpoker/blueprint`, and its mmap file layout is close enough to the
  Myosu request boundary that the play surface can query it directly without
  taking a crate dependency on the sibling repo.
  Date/Author: 2026-03-28 / Codex

---

## Decision Log

- Decision: The `GameRenderer` trait is the integration contract between the game engine and the TUI.
  Rationale: This was already designed in `specs/031626-07-tui-implementation.md`. The TUI should never know about poker-specific rendering — it delegates to the `GameRenderer` impl.
  Date/Author: 2026-03-21 / Interim CEO

- Decision: Pipe mode is for agents, not humans. Humans use the interactive TUI.
  Rationale: The TUI has two modes. Interactive (normal) is for human play. Pipe mode (non-interactive) is for agents that consume/produce JSON game state via stdin/stdout.
  Date/Author: 2026-03-21 / Interim CEO

- Decision: The first `myosu-play` entrypoint should be an honest demo wedge,
  not a fake full game.
  Rationale: A fixed snapshot plus real terminal loop and parse/clarify/action
  behavior proves the human/agent surface without inventing hand progression
  that does not exist yet.
  Date/Author: 2026-03-28 / Codex

---

## Outcomes & Retrospective

*(To be written at plan completion)*

---

## Milestones

### M1: Fix `myosu-play` entry point
The current binary initializes the shell but exits without running the event
loop. Fix it so `myosu-play train` enters the interactive terminal loop.

Status: Done 2026-03-28.

Proof: `myosu-play` now exists as a workspace crate, uses
`Shell::run_terminal(...)`, and compiles/tests/clippy cleanly. Interactive
manual proof remains the next local operator step because the terminal loop is
blocking by design.

Key files:
- `crates/myosu-play/src/main.rs` — binary entry point
- `crates/myosu-tui/src/events.rs` — event loop
- `crates/myosu-tui/src/shell.rs` — shell state machine

### M2: Wire poker renderer into five-panel shell
Implement `GameRenderer` for NLHE in `crates/myosu-games-poker/src/renderer.rs`.
Wire it so the state panel shows hole cards, board, pot, positions.

Status: Partially done 2026-03-28.

Proof: the live `myosu-play train` path now uses `NlheRenderer` against a fixed
sample snapshot, so the shell is rendering poker state rather than placeholder
content. The current wedge now goes further than that original proof target: it
  advances one generated-card hand across preflop, flop, turn, river, and a
  final complete state. `new` now restarts into the next generated hand with a
  different runout and updated advice. On this machine, default advice now also
  auto-loads the local codexpoker blueprint export from
  `~/.codexpoker/blueprint`, so the current missing pieces are true gameplay
  protocol, wider screen coverage, and a Myosu-native in-repo artifact bundle.

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

Status: Reframed.

Proof target remains open. The current repo now has a truthful first pipe seam:
plain-text input over a deterministic demo hand returns an `INFO ...` line that
describes advice source, selection mode, discovery origin, stable reason code,
and detail, then a `META ...` line plus `STATE ...`, then `ACTION ...`,
`CLARIFY ... legal=...`, or
`ERROR ... legal=...` responses, and each accepted action advances the printed
frame. Non-state-changing responses do not reprint the same frame. `quit` and
`/quit` now terminate the pipe loop with `QUIT`. Blank lines are ignored rather
than terminating the session. Active-state `actions=` and recovery `legal=`
hints now also include `/quit`, and the idle no-hand state advertises
`actions=/quit`. `myosu-play` now first tries the local codexpoker export at
`~/.codexpoker/blueprint`, then standard local blueprint artifact locations,
including
`MYOSU_BLUEPRINT_DIR`, `MYOSU_DATA_DIR/.myosu/blueprints/`, repo-local
`artifacts/`, and `~/.myosu/blueprints/`. On machines with the codexpoker
bundle present, startup now reports
`advice_source=artifact selection=auto origin=codexpoker_home reason=auto_loaded`.
If a standard root exists but is incomplete, the startup `INFO ...` line still
reports that exact root and the expected layout instead of flattening the
result into "no local blueprint artifacts found." Full JSON gameplay protocol
is still future work.

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
│   ├── pipe.rs        # pipe mode helper (WORKS for current text demo)
│   ├── renderer.rs    # Renderable + GameRenderer traits (WORKS)
│   ├── schema.rs      # Schema definitions (WORKS)
│   ├── screens.rs     # Screen management (PARTIAL)
│   ├── shell.rs       # Five-panel shell + terminal runner (WORKS)
│   └── theme.rs       # Theme styling (WORKS)
└── inline test coverage in core modules

crates/myosu-play/
├── Cargo.toml         # honest workspace binary crate
└── src/main.rs        # demo train/pipe entrypoints with tests
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

1. Land an honest `myosu-play` binary with blocking TUI and pipe demo paths
2. Extend the generated-card demo hand into real game progression beyond the
   current lobby-to-game wedge
3. Harden and generalize artifact-backed strategy advice beyond the local
   codexpoker export and explicit checkpoint path
4. Replace the current text demo with the real pipe gameplay protocol
5. Run an end-to-end playable hand

---

## Concrete Steps

```bash
# Current proof surface
cargo run -p myosu-play --quiet -- --smoke-test
cargo test -p myosu-play --quiet
cargo clippy -p myosu-play -- -D warnings

# Manual operator proof for the blocking TUI loop
cargo run -p myosu-play -- train

# Current pipe demo
printf 'call\n' | cargo run -p myosu-play --quiet -- pipe
# Expected: INFO ... followed by META/STATE and then ACTION call

# Current multi-street proof
printf 'call\ncall\ncall\nall-in\n' | cargo run -p myosu-play --quiet -- pipe
# Expected: PREFLOP -> FLOP -> TURN -> RIVER -> complete

# Future proof once full protocol exists
# echo '{"game_state": {...}}' | cargo run -p myosu-play -- pipe
```

---

## Validation

- `cargo run -p myosu-play --quiet -- --smoke-test` prints `SMOKE myosu-play ok`
- `cargo run -p myosu-play -- train` enters a blocking terminal loop (not immediate exit)
- The five-panel shell shows poker state (hole cards, board, pot, positions)
- Pipe mode currently proves parse/clarify/action on a deterministic multi-street hand
- On this machine, pipe startup now auto-loads the local codexpoker blueprint
  export and reports `origin=codexpoker_home reason=auto_loaded`
- Full JSON protocol remains open and should not be claimed yet
- Complete demo hand: preflop → flop → turn → river → complete without panics
- `cargo test -p myosu-tui` shows ≥ 20 passing integration tests
- `cargo test -p myosu-play` passes the current smoke surface

---

## Failure Scenarios

| Scenario | Handling |
|----------|----------|
| Ratatui rendering differs in CI vs local terminal | Use `ratatui::mock::MockTerminal` for CI tests; local tests require real terminal |
| Strategy query blocks indefinitely | Add timeout to strategy query; show "computing..." in declaration panel after 100ms |
| Unicode card symbols rendering incorrectly | Use ASCII fallback: `Ah` instead of `A♥` for non-UTF8 terminals |

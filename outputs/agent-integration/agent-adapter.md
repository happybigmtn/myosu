# `agent:experience` — Adapter to Implementation Family

## Purpose

This document maps the reviewed `agent:experience` lane to its concrete next steps
in the implementation family. It is the bridge between the bootstrap contract
(`spec.md` + `review.md` at `outputs/agent/experience/`) and the first real
implementation slice.

## Source: Reviewed Lane Contract

**Lane**: `agent:experience` (`outputs/agent/experience/spec.md` + `review.md`)
**Review verdict**: KEEP — proceed to implementation-family workflow
**Review date**: 2026-03-19

The lane owns the following surfaces:

| Surface | File | Status |
|---------|------|--------|
| `GameState` JSON schema | `docs/api/game-state.json` + `crates/myosu-tui/src/schema.rs` | **TRUSTED** |
| `pipe_output()` contract | `crates/myosu-tui/src/pipe.rs` | **TRUSTED** |
| `agent_context.rs` | `crates/myosu-tui/src/agent_context.rs` | **MISSING** |
| `journal.rs` | `crates/myosu-tui/src/journal.rs` | **MISSING** |
| `narration.rs` | `crates/myosu-tui/src/narration.rs` | **MISSING** |
| `--context` flag wiring | `crates/myosu-tui/src/pipe.rs` | **MISSING** |
| `--narrate` flag wiring | `crates/myosu-tui/src/pipe.rs` | **MISSING** |
| `reflect>` prompt | `crates/myosu-tui/src/pipe.rs` | **MISSING** |
| Lobby + game selection | `crates/myosu-tui/src/pipe.rs` | **MISSING** |
| `SpectatorRelay` | `crates/myosu-play/src/spectate.rs` | **MISSING** |
| `SpectateScreen` | `crates/myosu-tui/src/screens/spectate.rs` | **MISSING** |

---

## Upstream Dependency Map

```
tui:shell (82 tests, TRUSTED)
  └── GameRenderer, PipeMode, Events, Theme
      ↑
      │  (Slices 1-4 depend ONLY on this)
      │
games:traits (14 tests, TRUSTED)
  └── CfrGame, Profile, GameConfig, GameType
      ↑
      │  (Slices 5-9 depend on this via pipe_output)
      │
robopoker (ABSOLUTE PATH — git migration IN PROGRESS via games:traits)
  └── Game, Recall, Action
      ↑
      │
play:tui binary skeleton (MISSING — blocks Slices 3+)
  └── myosu-play main.rs CLI dispatch
      ↑
      │
chain:runtime (STUBBED for Phase 0 lobby)
```

---

## First Implementation Slice: `agent_context.rs` + `journal.rs`

### Why Slices 1 and 2 first

Both `agent_context.rs` (Slice 1) and `journal.rs` (Slice 2) depend **only on
`tui:shell`** which is already trusted. They do not require:

- `myosu-play` binary (blocked upstream)
- `robopoker` git migration (blocked upstream)
- `chain:runtime` (Phase 4 dependency)

This makes them the first honest slice that can be implemented, verified, and
merged without any upstream unblock.

### Slice 1: `agent_context.rs`

**File**: `crates/myosu-tui/src/agent_context.rs`

**What**:
- `AgentContext` struct with `load()`, `save()`, `default()` constructors
- `identity`: name, created timestamp, games_played, preferred_game
- `memory`: session_count, lifetime_result, observations vector
- `journal`: vector of `{session, hand, reflection}` entries
- Serde serialization to/from JSON
- `load()` returns `AgentContext` — file not found creates default identity
- `save()` writes atomically (write-to-temp + rename) to never corrupt existing file
- Roundtrip test: load → modify → save → reload → identical content

**Wire contract**:
- `PipeMode` calls `AgentContext::load(context_path)` on init (if `--context` given)
- `PipeMode` calls `context.save()` on clean drop
- Missing file → `AgentContext::default()` with generated name and empty state

**Proof gate**:
```
cargo test -p myosu-tui agent_context::tests::load_and_save_roundtrip
cargo test -p myosu-tui agent_context::tests::journal_appends_not_overwrites
cargo test -p myosu-tui agent_context::tests::missing_context_creates_new
```

### Slice 2: `journal.rs`

**File**: `crates/myosu-tui/src/journal.rs`

**What**:
- `Journal` struct wrapping a markdown file path
- `append_hand_entry(hand_id, board, held, result, reflection)` — appends one entry
- `append_session_summary(session_id, hands, result)` — appends session close
- **Invariant**: file is append-only; `save()` never truncates or rewrites
- Each entry is a markdown section with board state, held cards, result, optional reflection
- `Journal` is constructed from `AgentContext` path (journal lives alongside context file)

**Proof gate**:
```
cargo test -p myosu-tui journal::tests::append_hand_entry
cargo test -p myosu-tui journal::tests::append_session_summary
cargo test -p myosu-tui journal::tests::never_truncates
```

---

## Slice 3: `--context` Flag Wiring in `PipeMode`

**File**: `crates/myosu-tui/src/pipe.rs`

**What**:
- Add `context_path: Option<PathBuf>` field to `PipeMode`
- Add `--context <path>` CLI flag to `myosu-play`
- On `PipeMode::new()`: load context if path provided
- On `PipeMode` drop: save context
- Wire: the `--context` flag feeds into `AgentContext::load()`

**Blocker**: `myosu-play` binary skeleton (owned by `play:tui` lane)

**Proof gate**: Play 10 hands with `--context ./test.json` → shutdown → restart with same path → memory + journal preserved

---

## Slice 4: `reflect>` Prompt After Hand

**File**: `crates/myosu-tui/src/pipe.rs`

**What**:
- After `HAND COMPLETE` block in pipe mode, output `reflect>` prompt
- Block on stdin: empty line → skip; non-empty → append to `Journal`
- Test: reflection prompt appears after hand, empty skips, non-empty is saved

**Proof gate**:
```
cargo test -p myosu-tui pipe::tests::reflection_prompt_after_hand
cargo test -p myosu-tui pipe::tests::empty_reflection_skips
cargo test -p myosu-tui pipe::tests::reflection_saved_to_journal
```

---

## Slice 5: `narration.rs` — Rich Prose Engine

**File**: `crates/myosu-tui/src/narration.rs`

**What**:
- `NarrationEngine::narrate(&GameState) -> String`
- Board texture analysis: "dry" (3 suits, no connections), "wet" (paired, suited, connected), "neutral"
- Session arc: stack trajectory, opponent history from context file
- Atmospheric prose format matching AX-03 example in source spec
- Does NOT require `robopoker` git migration (uses `GameState` which is already in `schema.rs`)

**Proof gate**:
```
cargo test -p myosu-tui narration::tests::narrate_includes_board_texture
cargo test -p myosu-tui narration::tests::narrate_includes_session_context
cargo test -p myosu-tui narration::tests::terse_and_narrate_same_game_state
```

---

## Slice 6: `--narrate` Flag Wiring

**File**: `crates/myosu-tui/src/pipe.rs`

**What**:
- Add `narrate: bool` field to `PipeMode`
- Add `--narrate` CLI flag to `myosu-play`
- When `narrate == true`: use `NarrationEngine` instead of `pipe_output()`
- Underlying game state identical in both modes (proven by test)

**Blocker**: `myosu-play` binary skeleton

---

## Slice 7: Lobby + Game Selection in Pipe Mode

**File**: `crates/myosu-tui/src/pipe.rs`

**What**:
- When no `--subnet` provided in pipe mode: render lobby
- `info <id>` command shows subnet details
- Subnet selection starts the game
- **Phase 0**: hardcoded stub data (no live chain query)

**Blocker**: `myosu-play` binary skeleton; chain discovery stubbed for Phase 0

---

## Slices 8–9: Spectator (Phase 1)

**Files**: `crates/myosu-play/src/spectate.rs`, `crates/myosu-tui/src/screens/spectate.rs`

**What**:
- `SpectatorRelay`: Unix domain socket at `~/.myosu/spectate/<session_id>.sock`
- Fog-of-war enforced at relay (hole cards never emitted during play)
- `SpectateScreen`: renders events with fog-of-war; `r` key reveals hole cards after showdown

**Blockers**: `myosu-play` binary; `play:tui` lane completion

---

## Recommended Manifest Additions

Following the pattern of `myosu-games-traits-implementation.yaml`, create:

```
fabro/programs/myosu-agent-experience-implementation.yaml
fabro/run-configs/implement/agent-experience.toml
fabro/workflows/implement/agent-experience.fabro
fabro/checks/agent-experience-implement.sh
```

The manifest should produce `implementation.md` and `verification.md` artifacts
under `outputs/agent/experience/` and use the same milestone chain:
`reviewed` → `implemented` → `verified`.

---

## Blockers Summary

| Blocker | Severity | Owner | Affects |
|---------|----------|-------|---------|
| `robopoker` absolute path deps | HIGH | `games:traits` lane | Slices 5–9 (full integration) |
| `myosu-play` binary skeleton | HIGH | `play:tui` lane | Slices 3, 6, 7, 8–9 |
| Chain discovery (lobby) | MEDIUM | `chain:runtime` lane | Slice 7 (Phase 0: stub OK) |
| Spectator socket path convention | LOW | `play:tui` lane | Slice 8 |

**No new upstream unblocks required before starting Slices 1–2.**

---

## Decision

**Proceed to implementation family.** Slices 1 and 2 are implementable immediately.
The `robopoker` blocker affects later slices (5–9) and is being actively resolved in
the `games:traits` lane. The `myosu-play` blocker affects Slices 3 and beyond but
does not prevent the first honest slice from being written and verified in isolation.

An implementation-family manifest for `agent:experience` should be created following
the `myosu-games-traits-implementation.yaml` pattern.

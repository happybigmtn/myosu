# `agent:experience` Integration Review

## Judgment: **PROCEED** — Implementation-family workflow

**Decision**: Start `agent:experience` Slices 1–4 immediately. Begin `play:tui` Slice 1 concurrently. The `agent:experience` lane is spec-ready and its immediate slices have no external blockers. The product frontier needs an implementation family next, not another upstream unblock.

---

## Rationale

### The `agent:experience` spec is sound

The lane spec at `outputs/agent/experience/spec.md` is a mature, honest document. AX-01 through AX-05 are well-reasoned, with explicit in-scope/out-of-scope boundaries. The reflection/opt-in design decisions are documented. The append-only journal, optional reflection prompt, and fog-of-war enforcement at the relay (not renderer) are all sound architectural choices.

The `game-state.json` schema and `schema.rs` implementation are already trusted (16 tests pass, 10 game types covered). This is the strongest surface in the lane.

### Upstream `tui:shell` and `games:traits` are trusted

Both upstream lanes have passing tests:
- `tui:shell`: 82 tests pass — `Shell`, `GameRenderer`, `PipeMode`, `Events`, `Theme` all proven
- `games:traits`: 14 tests pass — `CfrGame`, `Profile`, `GameConfig`, `GameType` all proven

Slices 1–4 of `agent:experience` depend only on these trusted surfaces. There is no honest reason to wait.

### `play:tui` binary is a sequential dependency, not a parallel blocker

`play:tui`'s `reviewed` milestone is a prerequisite for `agent:experience` Slices 3–9 (CLI flag wiring, narration, lobby, spectator). However, `play:tui` has a clear, ordered 7-slice implementation path. Its blockers are known and owned:
- `myosu-play` crate creation (Slice 1) — unambiguous
- `myosu-games-poker` crate creation (Slice 2) — unambiguous

These are not open-ended research problems. They are implementation tasks.

### The alternative (wait for `play:tui` before starting `agent:experience`) is worse

If the product frontier waits for `play:tui`'s `reviewed` milestone before touching `agent:experience`, the critical path becomes serial:
```
play:tui Slice 1 → play:tui Slice 2 → ... → play:tui reviewed → agent:experience
```

That adds weeks of sequential delay for work that could have been done in parallel.

The correct model is:
```
play:tui Slice 1 (concurrent) ──► play:tui reviewed ──► agent:experience Slices 3–9
tui:shell (trusted) ──► agent:experience Slices 1–4 (concurrent, starts now)
```

---

## Slice Readiness Matrix

| Slice | Module | Blockers | Start? |
|-------|--------|----------|--------|
| Slice 1 | `agent_context.rs` | None — `tui:shell` and `games:traits` trusted | **YES — now** |
| Slice 2 | `journal.rs` | None — `tui:shell` and `games:traits` trusted | **YES — now** |
| Slice 3 | `--context` flag wiring | `play:tui` binary skeleton (concurrent work) | **YES — concurrent** |
| Slice 4 | `reflect>` prompt | Same as Slice 3 | **YES — concurrent** |
| Slice 5 | `narration.rs` | Same as Slice 3 | After Slice 3 |
| Slice 6 | `--narrate` flag | Same as Slice 3 | After Slice 5 |
| Slice 7 | Lobby + game selection | `play:tui` Slice 1 + chain stub | After Slice 6 |
| Slice 8 | `SpectatorRelay` | `play:tui` binary | After Slice 7 |
| Slice 9 | `SpectateScreen` | `play:tui` binary | After Slice 8 |

---

## Remaining Blockers (for Slices 3–9)

### 1. `myosu-play` binary does not exist — BLOCKING Slice 3+

**Owned by**: `play:tui` lane, Slice 1

All CLI flag wiring (`--context`, `--narrate`, `--spectate`) requires `crates/myosu-play/src/main.rs` to exist. This is the primary sequential dependency for the second tier of slices.

**Resolution**: `play:tui` Slice 1 creates the binary skeleton. Once it exists, Slices 3–4 can wire to it.

### 2. `robopoker` Git Migration — BLOCKING integration testing only

**Owned by**: `games:traits` lane (Slice 1 complete; robopoker migrated to git rev)

The `games:traits` lane completed its Slice 1 robopoker migration in a prior session. The pinned git rev is `04716310143094ab41ec7172e6cea5a2a66744ef`. This resolves the CI blocker.

**Impact on this lane**: Slices 1–4 (unit-testable without the full game loop) are unaffected. Slices 5–9 that exercise the full game loop need the migration verified. This should be confirmed before Slice 5 begins.

### 3. `docs/api/game-state.json` precondition check — AMBIGUOUS

**Owned by**: Unknown — `myosu-product.yaml` lists this as a precondition for the `agent` unit

The run config at `fabro/run-configs/product/agent-experience.toml` specifies this as a precondition check:
```
- label: game_state_schema_present
  type: file_exists
  path: ../../docs/api/game-state.json
```

The file **does exist** at `docs/api/game-state.json` (verified). However, the check path in the manifest uses `../../docs/api/game-state.json` relative to the `outputs/agent/experience/` directory, which resolves to `docs/api/game-state.json` from the repo root — correct.

**Resolution**: This is a false positive in the manifest path construction. The file exists. The check should pass once the manifest path is verified against the actual file layout.

---

## What the Product Frontier Needs Next

### Recommendation: Implementation-family workflow for `agent:experience`

The `agent:experience` lane should move to an implementation-family workflow. The 9 slices are well-defined, ordered, and mostly independent. The first two slices can be implemented and tested in isolation before the `play:tui` binary is available.

The `play:tui` lane should continue its own implementation-family workflow in parallel.

### Do NOT start a new upstream unblock

The remaining blockers are owned by existing lanes:
- `myosu-play` binary → `play:tui` lane
- robopoker git migration → `games:traits` lane (already resolved)

Starting a new "upstream unblock" lane would add coordination overhead without solving any blocker faster. The existing lanes own their blockers.

---

## Decision Log

- **2026-03-20**: Decision to proceed with implementation-family workflow for `agent:experience` Slices 1–4 immediately. `play:tui` continues in parallel. No new upstream unblock lane needed.
- **2026-03-20**: Confirmed `docs/api/game-state.json` exists — the `game_state_schema_present` precondition in the manifest is satisfied. Path ambiguity in the run config should be corrected to use an absolute path or repo-root-relative path.
- **2026-03-20**: robopoker git migration confirmed resolved by `games:traits` Slice 1. Integration testing for full game loop (Slices 5+) should verify this before proceeding.

---

## File Reference Index

| File | Role |
|------|------|
| `outputs/agent/experience/spec.md` | Lane specification (source of truth for scope and slices) |
| `outputs/agent/experience/review.md` | Lane review (KEEP judgment; upstream status) |
| `specsarchive/031626-10-agent-experience.md` | AX-01..05 source spec |
| `specsarchive/031626-17-spectator-protocol.md` | SP-01..03 spectator protocol spec |
| `docs/api/game-state.json` | JSON schema (TRUSTED) |
| `crates/myosu-tui/src/schema.rs` | Rust schema implementation (TRUSTED, 16 tests) |
| `crates/myosu-tui/src/pipe.rs` | PipeMode partial implementation (TRUSTED, 6 tests) |
| `outputs/play/tui/spec.md` | `play:tui` lane spec (upstream) |
| `outputs/play/tui/review.md` | `play:tui` lane review (KEEP) |
| `outputs/tui/shell/spec.md` | `tui:shell` lane spec (trusted upstream) |
| `outputs/games/traits/spec.md` | `games:traits` lane spec (trusted upstream) |
| `fabro/run-configs/product/agent-experience.toml` | Fabro run config for this lane |
| `fabro/programs/myosu-product.yaml` | Product frontier manifest |

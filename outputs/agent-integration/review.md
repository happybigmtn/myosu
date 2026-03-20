# `agent-integration` Lane Review

## Judgment: **PROCEED** — implementation-family with phased gate

The `agent:experience` lane spec is sound and the reviewed artifacts are honest. The integration surface is real and partially built. The remaining work decomposes cleanly into two groups: **unblocked slices** (1, 2, 5) that can begin immediately, and **blocked slices** (3, 4, 6, 7, 8, 9) that must wait for `play:tui` Slice 1.

This lane is not a no-go. It is a **phased-go with explicit blockers**.

---

## Rationale for PROCEED (with phased gate)

1. **The spec is done and honest.** `agent:experience` review (KEEP judgment) accurately identifies what's missing and what's blocked. No speculative surfaces — every missing module is named with a file path and a clear contract.

2. **The integration surface is real.** `PipeMode` exists in `crates/myosu-tui/src/pipe.rs`. `GameRenderer::pipe_output()` exists in `crates/myosu-tui/src/renderer.rs`. `GameState` schema is fully implemented with 16 tests passing. These are not sketches — they are the actual integration points.

3. **Slices 1, 2, 5 are unblocked right now.** `agent_context.rs`, `journal.rs`, and `narration.rs` depend only on `games:traits` (`GameType`) and `tui:shell`, both of which are trusted. They do not need `myosu-play` binary or robopoker git migration. Starting them now does not waste parallel work.

4. **The blockers are owned and tracked.** `play:tui` Slice 1 is tracked in `outputs/play/tui/review.md`. The robopoker git migration is tracked in `outputs/games/traits/review.md`. Neither is a surprise — they are explicit, dated, and owned.

5. **No design ambiguity remains.** The `GameRenderer` trait is frozen. The `AgentContext` schema is in the spec. The `reflect>` prompt format is in the spec. The `SpectatorRelay` socket path is in the spec. An implementation-family agent can proceed from these documents alone.

---

## Explicit Go/No-Go by Slice

| Slice | Module | Decision | Blocker |
|-------|--------|----------|---------|
| 1 | `agent_context.rs` | **GO** | None — `games:traits` trusted |
| 2 | `journal.rs` | **GO** | None — `GameType` from trusted upstream |
| 3 | `--context` wiring | **WAIT** | `play:tui` Slice 1 (`myosu-play` binary) |
| 4 | `reflect>` prompt | **WAIT** | Slice 3 |
| 5 | `narration.rs` | **GO** | None — depends only on `GameState` |
| 6 | `--narrate` wiring | **WAIT** | Slice 5 + `play:tui` Slice 1 |
| 7 | Lobby | **WAIT** | `play:tui` Slice 1 |
| 8 | `SpectatorRelay` | **WAIT** | `play:tui` Slice 1 |
| 9 | `SpectateScreen` | **WAIT** | Slice 8 |

---

## Upstream Dependency Status

| Dependency | Owner | Status | Impact if Delayed |
|------------|-------|--------|-------------------|
| `tui:shell` (82 tests) | `tui:shell` lane | **TRUSTED** | No impact — stable contract |
| `games:traits` (14 tests) | `games:traits` lane | **TRUSTED** | No impact on Slices 1, 2, 5 |
| `robopoker` git migration | `games:traits` lane | **BLOCKED** | Integration tests fail; Slices 3+ can't validate end-to-end |
| `myosu-play` binary skeleton | `play:tui` lane | **MISSING** | Slices 3, 4, 6, 7, 8, 9 cannot be wired |
| `chain:runtime` | `chain:runtime` lane | **FUTURE** | Lobby stubbed for Phase 0; acceptable |
| Spectator socket path | `play:tui` lane | **LOW** | Must confirm `~/.myosu/spectate/<id>.sock` vs `{data-dir}` convention |

---

## What Happens Next

1. **`games:traits`** continues resolving the robopoker git migration (Slice 1 of that lane). This unblocks all integration testing.

2. **`play:tui`** executes Slice 1 (binary skeleton) in parallel. This unblocks Slices 3, 4, 6, 7, 8, 9 of `agent:experience`.

3. **`agent:experience` (this lane family)** begins Slices 1, 2, 5 immediately and watches `play:tui` Slice 1 completion to gate the remaining slices.

4. **This review is updated** when `play:tui` Slice 1 completes and when robopoker git migration resolves.

---

## Recommendation

**Do not wait.** Begin the implementation-family workflow for `agent:experience` Slices 1, 2, and 5 in parallel with `play:tui` Slice 1. The blocker separation is clean and the dependency chain is understood.

The two critical blockers (`robopoker` git migration and `myosu-play` binary) are **owned elsewhere** and **tracked in their respective lane reviews**. This lane should not hold its own progress waiting for them — it should start where it can and gate the rest honestly.

---

## Evidence

| Check | Result |
|-------|--------|
| `GameRenderer::pipe_output()` exists | YES — `renderer.rs:45` |
| `PipeMode` exists | YES — `pipe.rs:19` |
| `GameState` schema implemented | YES — 16 tests pass |
| `AgentContext` spec in `agent:experience` | YES — AC-AX-01 in reviewed spec |
| `myosu-play` binary exists | NO — not in `Cargo.toml` |
| `robopoker` git dependency | NO — absolute path deps remain |
| `play:tui` binary plan exists | YES — `outputs/play/tui/spec.md` Slice 1 |

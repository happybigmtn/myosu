# `agent:integration` Lane Review

## Judgment: **PROCEED** — integration surfaces are sound; upstream unblocks owned by others

---

## Evidence Reviewed

| Artifact | Source | Status |
|----------|--------|--------|
| `outputs/agent/experience/spec.md` | `agent:experience` lane | Reviewed — 9 slices defined, schemas sound |
| `outputs/agent/experience/review.md` | `agent:experience` lane | Reviewed — KEEP judgment, implementation-family recommended |
| `crates/myosu-tui/src/schema.rs` | Trusted implementation | Reviewed — 939 lines, 16 tests passing |
| `crates/myosu-tui/src/pipe.rs` | Trusted implementation | Reviewed — basic PipeMode exists, 6 tests |
| `crates/myosu-tui/src/lib.rs` | Trusted exports | Reviewed — `GameRenderer`, `Theme` exported |
| `fabro/programs/myosu-product.yaml` | Program manifest | Reviewed — `agent:experience` lane defined with reviewed milestone |
| `outputs/games/traits/review.md` | `games:traits` lane | Reviewed — robopoker git migration identified as Slice 1 |
| `outputs/play/tui/review.md` | `play:tui` lane | Reviewed — binary skeleton identified as Slice 1 |

---

## Integration Assessment

### What Is Sound

1. **`schema.rs` is the strongest integration point.** 939 lines, 16 tests, full JSON schema for 10 game types. The `legal_actions` exhaustive array is a strong contract — agents never need to guess what's legal. This is the foundation that all agent-facing surfaces build on.

2. **`GameRenderer` trait is a clean integration boundary.** Object-safe (`dyn`), no `Sized` or `Copy` constraints. `PipeMode` consumes `&dyn GameRenderer` without knowing the concrete type. The trait is stable and well-tested (82 tests in `tui:shell`).

3. **`agent:experience` does not call `games:traits` directly.** Integration is indirect: `games:traits` types implement `GameRenderer`, which `PipeMode` consumes. This means `agent:experience` slices can proceed without directly depending on `games:traits` evolution.

4. **Spectator socket path and hand history path are separate concerns.** `~/.myosu/spectate/<session_id>.sock` (spectator relay) and `{data-dir}/hands/hand_{N}.json` (hand history) do not conflict. No adapter layer needed.

5. **No new adapter crate required for Phase 1.** The integration is at the CLI flag wiring level. If complexity grows, `crates/myosu-tui/src/agent_adapter.rs` is a reasonable future refactor, but it is not needed now.

### What Requires Upstream Resolution Before Testing

1. **`robopoker` git migration** — `games:traits` lane owns this. Absolute filesystem paths (`/home/r/coding/robopoker/...`) will cause build failures on any clean checkout or CI environment. `games:traits` review explicitly identifies this as Slice 1. Cannot run integration tests until resolved.

2. **`myosu-play` binary skeleton** — `play:tui` lane owns this. Slices 3, 6, and 7 of `agent:experience` require CLI flag wiring (`--context`, `--narrate`, lobby). There is no binary to wire into without `play:tui` Slice 1.

### What Does NOT Block

- Chain discovery stub for lobby (MEDIUM, defer to Phase 4, `chain:runtime` dependency)
- Spectator socket path convention (LOW, no conflict expected with `play:tui` data dir)

---

## Decision: Proceed to Implementation-Family Workflow

### Rationale

1. **Integration surfaces are not the constraint.** `schema.rs`, `GameRenderer`, `PipeMode`, and the CLI flag design are all sound. The `agent:experience` review's KEEP judgment stands.

2. **Upstream unblocks are owned and tracked.** `games:traits` owns robopoker git migration. `play:tui` owns binary skeleton. These are not integration problems — they are upstream delivery problems that `agent:experience` depends on.

3. **Slices 1–2 can start immediately.** `agent_context.rs` and `journal.rs` depend only on `tui:shell` (trusted, 82 tests) and the `schema.rs` types. They do not require `myosu-play` binary skeleton or `robopoker` git migration to implement. They can be specified and prototyped now.

4. **The dependency chain is explicit.** The `agent:experience` spec already shows the slice ordering:
   ```
   Phase 1: Slice 1 → 2 → 3 → 4 (can start with Slice 1-2 now)
   Phase 2: Slice 5 → 6 → 7 (blocked on Slice 3 + play:tui binary)
   Phase 3: Slice 8 → 9 (blocked on play:tui binary)
   ```
   This is honest — the review documents the blockers. The integration assessment confirms the surfaces are ready.

### What This Lane Owns

- Producing `agent-adapter.md` (done) — integration contract for the control plane
- Tracking upstream blocker resolution
- Confirming when `games:traits` robopoker migration and `play:tui` binary skeleton are delivered
- Updating this review as upstream blockers resolve

### What This Lane Does NOT Own

- `robopoker` git migration (owned by `games:traits`)
- `myosu-play` binary skeleton (owned by `play:tui`)
- Chain discovery for lobby (defer to Phase 4)
- WebSocket upgrade for spectator (defer to Phase 4)

---

## Lane Readiness

| Dimension | Status | Notes |
|-----------|--------|-------|
| Integration surfaces | **READY** | schema.rs, GameRenderer, PipeMode all sound |
| `agent:experience` spec | **READY** | 9 slices, reviewed, KEEP |
| `games:traits` upstream | **PARTIAL** | robopoker git migration unresolved; Slice 1-2 of agent:experience can proceed |
| `play:tui` upstream | **PARTIAL** | binary skeleton missing; Slices 3-9 blocked until resolved |
| Upstream unblocks owned | **YES** | games:traits owns robopoker; play:tui owns binary |
| Implementation slices | **READY** | Slices 1-2 can begin immediately |

---

## Recommendation

**Proceed to implementation-family workflow.** The integration surfaces are sound and the upstream blockers are owned by specific lanes with clear slice ownership. Begin `agent:experience` Slices 1 (`agent_context.rs`) and 2 (`journal.rs`) now — they have no upstream dependencies beyond trusted `tui:shell` surfaces.

Track `games:traits` Slice 1 (robopoker git migration) and `play:tui` Slice 1 (binary skeleton) as upstream dependencies. This review should be updated when those lanes deliver.

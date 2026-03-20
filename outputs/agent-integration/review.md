# `agent:integration` Lane Review

## Judgment: **KEEP** — proceed to Phase 0 implementation slices

This lane is the integration layer that delivers the `agent:experience` surfaces. The JSON schema (`schema.rs`) is the universal adapter that makes pipe mode, narration, spectator relay, and future HTTP/WS APIs composable. The `agent:experience` review judged KEEP and recommended proceeding to implementation-family workflow — this lane IS that implementation family.

The primary blockers (robopoker git migration, play:tui binary skeleton) are owned by other lanes and should be resolved in parallel. This lane can begin Phase 0 work on surfaces that don't require those blockers.

---

## Rationale for KEEP

1. **JSON Schema is the universal adapter**: `GameState` + `LegalAction` + `GamePhase` + `MetaInfo` types in `schema.rs` (939 lines, 16 tests passing) are the shared vocabulary that enables pipe mode, narration, spectator relay, and future HTTP/WS APIs to all compose cleanly. This is the strongest integration surface in the product.

2. **Upstream surfaces are trusted**: `tui:shell` (82 tests pass) and `games:traits` (14 tests pass) are already in the trusted state. The `GameRenderer` trait and `PipeMode` driver exist and compile. The lane builds on proven infrastructure.

3. **Decision from `agent:experience` review is clear**: The review explicitly states "proceed to implementation-family workflow next" and identifies this lane as the delivery vehicle for that workflow. No further deliberation needed.

4. **Integration points are well-defined**: The lane has clear upstream dependencies (games:traits, tui:shell, agent:experience spec) and clear downstream deliverables (myosu-play binary with --pipe, --context, --narrate, --spectate flags). The integration surface is bounded.

5. **Phase ordering is honest**: Phase 0 is standalone (no chain dependency). Phase 1 adds spectator. Phase 2 adds chain-connected features. Each phase is independently meaningful and deliverable.

---

## Proof Expectations

To consider this lane **proven**, the following evidence must be available:

| Proof | How to Verify |
|-------|--------------|
| Schema tests pass | `cargo test -p myosu-tui schema::tests` exits 0 |
| Pipe mode outputs valid GameState text | `cargo test -p myosu-tui pipe::tests` exits 0 |
| --context flag preserves memory across sessions | Play 10 hands → restart → memory preserved |
| --narrate flag produces board texture prose | Narrated output contains "dry" or "wet" or "connected" |
| Journal is append-only | Write 100 entries; file only grows; never truncates |
| Spectator relay emits valid JSON events | `cargo test -p myosu-play spectate::tests` exits 0 |
| Fog-of-war enforced at relay | Hole cards never appear in relay output during play |

---

## Remaining Blockers

### 1. `robopoker` Git Migration (HIGH — blocks integration testing)

Both `games:traits` and `tui:shell` depend on `robopoker` via **absolute filesystem paths**. This prevents `cargo build` and `cargo test` from running on clean checkout or CI.

**Owned by**: `games:traits` lane (RF-01..04)
**Impact on this lane**: Cannot verify integration between `agent:experience` surfaces and the actual game engine until robopoker is migrated to a proper git dependency.
**Resolution**: This lane should proceed with Phase 0 surfaces that can be tested in isolation (schema tests, pipe output tests using mock renderers).

### 2. `myosu-play` Binary Skeleton Missing (HIGH — blocks CLI delivery)

The `myosu-play` binary (`crates/myosu-play/src/main.rs`) does not exist. All `--pipe`, `--context`, `--narrate`, and `--spectate` flags require modifications to this binary's CLI dispatch.

**Owned by**: `play:tui` lane (Slice 1: binary skeleton)
**Impact on this lane**: Cannot deliver integrated surfaces as a runnable binary until the binary skeleton exists.
**Resolution**: `play:tui` lane must deliver the binary skeleton before or concurrently with `agent:integration` Slice INT-2.

### 3. Chain Discovery Stubbed (MEDIUM — Phase 2 only)

The lobby (AC-AX-05) requires querying the chain or miner for active subnet information. This is stubbed for Phase 0 with hardcoded data.

**Owned by**: `chain:runtime` lane
**Impact on this lane**: Agents in pipe mode can only select from hardcoded subnet data in Phase 0. This is acceptable for Phase 0 proof.
**Resolution**: Real chain integration is Phase 2 (depends on `chain:runtime`).

---

## Lane Readiness

| Dimension | Status | Notes |
|-----------|--------|-------|
| JSON Schema | **TRUSTED** | 939 lines, 16 tests pass; covers 10 game types |
| GameRenderer trait | **TRUSTED** | `pipe_output()` contract exists; used by PipeMode |
| PipeMode driver | **TRUSTED** | 6 tests pass; basic stdin/stdout works |
| agent_context.rs | **MISSING** | Needed for Slice INT-3 |
| narration.rs | **MISSING** | Needed for Slice INT-4 |
| journal.rs | **MISSING** | Needed for Slice INT-3 |
| SpectatorRelay | **MISSING** | Needed for Slice INT-5 |
| myosu-play binary | **MISSING** | Owned by play:tui lane |
| robopoker git deps | **BLOCKER** | Owned by games:traits lane |
| Integration slices | **DEFINED** | 5 slices, INT-1 through INT-5 |

---

## Decision: Implementation Family Next

The `agent:experience` review explicitly states "proceed to implementation-family workflow next." This `agent:integration` lane is that workflow — it delivers the implementation slices that wire the agent:experience surfaces to runnable binaries.

**No further upstream unblocks are needed before proceeding.** The primary blockers (robopoker git migration, myosu-play binary skeleton) are owned by other lanes and should be resolved in parallel. This lane can begin with:

1. **INT-1**: Verify schema and pipe_output contract (can use mock renderers, no robopoker needed)
2. **INT-3**: Agent context + journal + --context wiring (depends on play:tui binary, but design is clear from spec)
3. **INT-4**: Narration + --narrate wiring (depends on narration.rs implementation)

The implementation family is ready to begin. The decision is **implementation family next**, not another upstream unblock.

---

## Recommendation

**Proceed to implementation-family workflow.** This lane should work in parallel with:
- `games:traits` resolving robopoker git migration
- `play:tui` delivering the binary skeleton

Begin with INT-1 (schema verification using mocks) while waiting for those blockers to clear. Once the binary skeleton exists, proceed with INT-3 and INT-4 in sequence.

Update this review after each slice completes to track proof availability and remaining blockers.

# `agent-integration` Lane Review

## Judgment: **KEEP** — proceed to implementation-family workflow

The `agent:experience` lane produced honest, well-scoped artifacts. The lane's own review correctly identifies what is trusted, what is missing, and what depends on other lanes. Product should proceed to the `agent:experience` implementation-family workflow, starting with Slices 1–2 immediately and Slice 3 once `play:tui` Slice 1 lands.

---

## Rationale for KEEP

### 1. `agent:experience` review is sound

The lane's own `review.md` verdict of **KEEP** is well-reasoned:
- Spec quality: AX-01..05 + SP-01..03 are mature drafts with documented decision logs
- Upstream trusted: `tui:shell` (82 tests) and `games:traits` (14 tests) are both TRUSTED
- Schema is the strongest surface: fully implemented, 939 lines, 16 tests passing
- Slice dependency chain is clean: Slices 1–4 are sequential with no external dependencies beyond `tui:shell`
- Scope is bounded: explicitly excludes agent-to-agent social interaction, autonomy over system parameters, and emotion modeling

### 2. `agent:experience` surfaces are correctly attributed

The `agent-adapter.md` produced here maps every agent-facing surface to its owning module and upstream dependency. The map is accurate:
- All currently MISSING surfaces are correctly labeled MISSING (not stubbed as working)
- All TRUSTED surfaces have test evidence cited
- Cross-lane dependencies are explicit and traceable to lane artifacts

### 3. No hidden coupling

The `agent:experience` lane does not introduce implicit dependencies on chain, validator, or miner lanes. Its Phase 2 (HTTP/WS APIs) and Phase 3 (spectator WebSocket) correctly gate on `chain:runtime`. Phase 0/1 work can proceed entirely offline.

---

## Cross-Lane Blocker Assessment

### BLOCKER: `robopoker` Git Migration

**Severity**: HIGH
**Owned by**: `games:traits` lane
**Blocks**: All Phase 1+ slices once they require integration testing

The `agent:experience` slices ultimately call into `games:traits` → `robopoker` via absolute filesystem paths. This blocks clean checkout and CI. Slices 1–2 can proceed because they only touch `agent_context.rs` and `journal.rs` (pure Rust, no robopoker calls), but Slice 3+ requires the full `PipeMode` → `GameRenderer` → `games:traits` chain.

**This lane cannot unblock this.** `games:traits` owns the resolution. Tracking should be in that lane's review.

### BLOCKER: `myosu-play` Binary Missing

**Severity**: HIGH
**Owned by**: `play:tui` lane
**Blocks**: Slices 3–9 (all flag wiring, lobby, spectator)

The `--context`, `--narrate`, `--spectate` flags all require modifications to `myosu-play`'s CLI dispatch in `main.rs`. Without the binary skeleton, none of this wiring can be tested end-to-end.

**This lane cannot unblock this.** `play:tui` owns Slice 1 (binary skeleton). `agent:experience` Slice 3 can begin once `play:tui` Slice 1 is complete, and the two can proceed in parallel thereafter.

### BLOCKER: Chain Discovery Stubbed

**Severity**: MEDIUM (for Slice 7 only)
**Owned by**: `chain:runtime` lane (Phase 4 resolution path)
**Note**: Already documented as stubbed for Phase 0 in `agent:experience/spec.md`

The lobby (Slice 7) requires querying active subnets from chain or miner. For Phase 0, this is documented as stubbed with hardcoded data. This is an acceptable progressive disclosure — it does not block Phase 1 slices.

---

## What This Lane Found Honestly

1. **`agent:experience` spec is accurate.** Every MISSING surface is correctly identified as such. There is no false confidence in the artifacts.

2. **`agent:experience` review is well-calibrated.** The KEEP judgment is based on upstream trust (which is real) and blocker acknowledgment (which is honest). The lane is not declaring itself done — it is declaring itself ready to implement.

3. **The lane boundary is clean.** `agent:experience` is terminal: it has no trusted downstream outputs. It consumes `tui:shell` and `games:traits`, extends them with agent affordances, and stops there.

4. **No hidden cross-lane contamination.** The `agent-adapter.md` shows no unexpected dependencies on chain, validator, or miner lanes. The only cross-lane coupling is `play:tui` (binary ownership) and `chain:runtime` (Phase 2/4 integration).

5. **Spectator relay path is sound but early.** The Phase 0 Unix socket approach is the right MVP. The Phase 1 WebSocket upgrade path via miner axon is correctly deferred.

---

## Decision: Implementation-Family Workflow Next

**Proceed to `agent:experience` implementation-family workflow**, starting with:

| Slice | What | Immediate unblock needed |
|-------|------|--------------------------|
| Slice 1 | `agent_context.rs` | None (only uses std/Rust) |
| Slice 2 | `journal.rs` | None (only uses std/Rust + filesystem) |
| Slice 3 | `--context` wiring in `PipeMode` | `play:tui` Slice 1 (binary skeleton) |
| Slice 4 | `reflect>` prompt | Slice 3 complete |
| Slice 5 | `narration.rs` | `games:traits` GameState data model |
| Slice 6 | `--narrate` wiring | Slice 5 + Slice 3 |
| Slice 7 | Lobby in pipe mode | `play:tui` Slice 1 |
| Slice 8 | `SpectatorRelay` | `play:tui` binary + Slice 3 |
| Slice 9 | `SpectateScreen` | Slice 8 |

**Do NOT proceed to another upstream unblock from this lane.** The upstream unblocks (robopoker migration, `play:tui` binary) are correctly owned by their respective lanes and tracked there. `agent:experience` should not absorb those blockers — it should wait for those lanes to complete.

---

## Lane Readiness

| Dimension | Status | Notes |
|-----------|--------|-------|
| `agent:experience` spec | **READY** | AX-01..05 + SP-01..03 sound; 9 slices defined |
| `agent:experience` review | **HONEST** | KEEP judgment well-calibrated; blockers correctly attributed |
| Upstream `tui:shell` | **TRUSTED** | 82 tests pass |
| Upstream `games:traits` | **TRUSTED** | 14 tests pass |
| `play:tui` binary | **MISSING** | Blocks Slice 3+; owned by `play:tui` lane |
| `robopoker` git dep | **BLOCKER** | Blocks Phase 1+ integration; owned by `games:traits` lane |
| Spectator relay | **SPEC ONLY** | Phase 0 socket path sound; implementation missing |
| Cross-lane contamination | **NONE** | No hidden dependencies on chain/validator/miner lanes |

---

## Recommendation

**Start the `agent:experience` implementation-family workflow immediately.** Slices 1 and 2 require no additional upstream work — they are pure Rust files. Slices 3–7 should proceed in coordination with `play:tui` lane as that binary skeleton becomes available.

The `agent:experience` lane is done with specification. It has produced honest, complete artifacts. The next honest step is implementation.

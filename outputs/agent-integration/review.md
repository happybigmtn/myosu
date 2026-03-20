# `agent-integration` Lane Review

## Judgment: **PROCEED** — implementation-family next, robopoker is the critical path

This lane consolidates the decision about what comes after `agent:experience` (the last remaining ready product lane) based on the reviewed artifacts that lane produced.

---

## Source Artifacts Assessed

| Artifact | Status | Notes |
|----------|--------|-------|
| `outputs/agent/experience/spec.md` | **COMPLETE** | 9 slices defined (Slices 1-9); all surfaces catalogued; dependency chain clear |
| `outputs/agent/experience/review.md` | **COMPLETE** | KEEP judgment; 4 blockers documented; proof expectations defined |
| `specsarchive/031626-10-agent-experience.md` | **DRAFT** | Source spec with AC-AX-01..05; design philosophy well-reasoned |

---

## Decision: Implementation-Family Next

**Verdict**: The `agent:experience` lane has produced its reviewed artifacts. The product frontier's next step is to begin the **implementation-family workflow** — executing the 9 slices in phase order.

**Rationale**:

1. **The lane is ready**: `agent:experience` spec.md and review.md are complete. The KEEP judgment in the review is sound: the source specs are mature, the upstream dependencies (`tui:shell`, `games:traits`) are trusted, and the slice dependency chain is clean.

2. **The blockers are not this lane's responsibility**: The two HIGH blockers (robopoker git migration, missing `myosu-play` binary) are owned by other lanes (`games:traits` and `play:tui` respectively). This lane should not wait on them — it should document the dependency and proceed.

3. **Parallelization is possible**: The `games:traits` lane is already working the robopoker git migration. The `play:tui` lane is already working the binary skeleton. The product frontier can begin implementation-family planning while those complete.

4. **The 9 slices are sequential but start simply**: Slices 1-4 (agent context, journal, --context wiring, reflect prompt) depend only on `tui:shell` which is already trusted with 82 tests. Implementation can begin immediately on these.

---

## Critical Path: Robopoker Git Migration

The robopoker git migration (absolute path deps → `git = "https://..."`) is the **hard blocker** for any Phase 1+ testing. Both `tui:shell` and `games:traits` depend on robopoker via absolute filesystem paths:

```
/home/r/coding/robopoker/crates/...  ← current (broken in clean checkout / CI)
```

**Impact on agent:experience implementation**:
- Slices 1-4: Can implement against `tui:shell` mock; testing requires robopoker resolved
- Slices 5-9: Full integration testing requires robopoker git migration complete

**This lane recommends**: Track robopoker git migration as a **critical-path dependency** for the implementation-family. Do not schedule Phase 1+ slice completion dates until the migration is confirmed.

---

## Secondary Blocker: `myosu-play` Binary

The `myosu-play` binary skeleton does not exist yet. Slices 3 (--context wiring), 6 (--narrate), 7 (lobby), and 8-9 (spectator) all require modifications to the CLI dispatch in `main.rs`.

**Impact on agent:experience implementation**:
- Slices 3, 6, 7: Require binary CLI modifications
- Slices 8-9: Require `myosu-play/src/spectate.rs`

**This lane recommends**: Confirm `play:tui` Slice 1 (binary skeleton) is on track before scheduling Slices 3+.

---

## What This Lane Produced

| Artifact | Purpose |
|----------|---------|
| `outputs/agent-integration/agent-adapter.md` | Integration reference: how an agent harness connects to `agent:experience` surfaces (pipe protocol, context files, narration, journal, spectator relay) |
| `outputs/agent-integration/review.md` | This document: decision record for product frontier next step |

---

## Lane Readiness Assessment

| Dimension | Status | Notes |
|-----------|--------|-------|
| `agent:experience` artifacts | **COMPLETE** | spec.md + review.md both reviewed |
| `agent:experience` judgment | **KEEP** | Proceed to implementation-family |
| robopoker dependency | **CRITICAL PATH** | Owned by `games:traits`; must resolve before Phase 1+ |
| myosu-play binary | **BLOCKING** | Owned by `play:tui`; must resolve before Slices 3+ |
| Implementation slices defined | **YES** | 9 slices, phased, clear boundaries |
| Adapter reference | **COMPLETE** | 7 slices defined for reference adapter implementation |

---

## Recommendation

**Start the implementation-family workflow for `agent:experience` now.** The lane is ready and the artifacts are reviewed.

Begin Slices 1-2 (`agent_context.rs`, `journal.rs`) immediately — they depend only on `tui:shell` which is trusted. Slices 3-4 can follow as soon as the binary skeleton exists. Slices 5-9 can proceed once robopoker is resolved.

The `games:traits` lane should be queried weekly for robopoker migration status. The `play:tui` lane should be queried for binary skeleton progress.

---

## Next Steps

1. **Immediately**: Begin Slice 1 (`agent_context.rs`) and Slice 2 (`journal.rs`) — no external dependencies
2. **This week**: Confirm `play:tui` binary skeleton timeline for Slice 3 scheduling
3. **Weekly**: Check `games:traits` robopoker migration status
4. **When robopoker resolves**: Schedule Phase 2 (Slices 5-7) and Phase 3 (Slices 8-9) completion dates
5. **When binary exists**: Begin Slice 3 (--context wiring) concurrently with Slice 2

# `agent:integration` Lane Review

## Judgment: **PROCEED** — implementation-family workflow

This lane has two inputs:
1. The `agent:experience` lane review (`outputs/agent/experience/review.md`) — **KEEP**
2. The current upstream readiness state across all bootstrap lanes

The judgment of `agent:integration` is not about `agent:experience` (that judgment
is already made). It is about whether the **product as a whole** is ready to enter
an implementation-family workflow or whether another upstream unblock must be resolved
first.

The answer is: **proceed to implementation-family workflow.** The last remaining ready
product lane (`agent:experience`) has been honestly reviewed and is KEEP. No critical
upstream unblock stands between product and beginning slice execution.

---

## Basis: `agent:experience` Review Result

`outputs/agent/experience/review.md` records **KEEP — proceed to implementation-family
workflow** with the following key findings:

| Dimension | Status |
|-----------|--------|
| Spec quality | AX-01..05 + SP-01..03 are sound |
| Upstream `tui:shell` | **READY** — 82 tests pass |
| Upstream `games:traits` | **READY** — 14 tests pass |
| Schema (`schema.rs`) | **TRUSTED** — 16 tests pass, 939 lines |
| robopoker dep | **BLOCKER** (owned by `games:traits`) |
| `play:tui` binary | **PARTIAL** — skeleton missing, needed Slice 3+ |
| Implementation slices | **DEFINED** — 9 slices, sequential, minimal coupling |

The `agent:experience` lane is the last remaining **ready product lane** in the
current bootstrap set. The four bootstrap lanes are:

| Lane | Status |
|------|--------|
| `games:traits` | Trusted (14 tests, `CfrGame`, `Profile`, `GameType`, `StrategyQuery` all exist) |
| `tui:shell` | Trusted (82 tests, `GameRenderer`, `PipeMode`, `Events`, `Theme` all exist) |
| `chain:runtime` | Restart lane — blocked on pallet review |
| `chain:pallet` | Restart lane — blocked on runtime review |
| `agent:experience` | **KEEP — ready to implement** |

---

## Upstream Dependency Analysis

### robopoker Git Migration — Owned Elsewhere

The `agent:experience` review identifies the `robopoker` absolute-path dependency as
a HIGH-priority blocker. However, this is **not a decision gate for `agent:integration`**
for the following reasons:

1. **Ownership**: The `games:traits` lane owns robopoker migration. This is documented
   in `outputs/games/traits/review.md` as their Slice 1 fix.

2. **Slices 1–4 are unblocked**: `agent_context.rs`, `journal.rs`, `--context` wiring,
   and `reflect>` prompt all depend only on `tui:shell` (which is trusted and does not
   itself depend on robopoker in its public API surface).

3. **Slices 5–9 require integration**: Narration, lobby, and spectator relay will need
   robopoker resolved before end-to-end proof is possible. But this is a **later
   phase gate**, not a **start gate**.

**Recommendation**: Begin Slices 1–4 immediately. Track robopoker resolution as a
Phase 2 gate in `agent:experience` review, not as a blocker for this lane.

### `play:tui` Binary Skeleton — Partial Blocker

The `myosu-play` binary does not exist yet. This is required for:
- Slice 3 (`--context` wiring) — HIGH priority
- Slice 6 (`--narrate` wiring) — MEDIUM
- Slice 7 (lobby) — MEDIUM
- Slice 8–9 (spectator) — HIGH

**Mitigation**: Slices 1–2 (`agent_context.rs`, `journal.rs`) can be implemented and
tested entirely within `crates/myosu-tui/src/` without the binary. The `AgentContext`
and `Journal` types can be unit-tested in isolation.

**Recommendation**: Begin Slices 1–2 immediately. `play:tui` binary skeleton must
complete before Slice 3, but does not block the first honest slice.

### Chain Runtime Lanes — Not on Critical Path for Agent

`chain:runtime` and `chain:pallet` are restart lanes. They are relevant to
`agent:experience` only for Phase 4 (lobby chain queries, spectator WebSocket upgrade).
They are **not on the critical path** for Phase 1–3 of `agent:experience`.

---

## Decision: Implementation-Family Workflow

The product should enter an **implementation-family workflow** now, not wait for
another upstream unblock. The reasoning:

1. **`agent:experience` is the last ready product lane.** All four bootstrap lanes
   have been reviewed. Three are trusted or restart. `agent:experience` is KEEP'd.
   There is no fifth lane blocking product progress.

2. **Slices 1–2 are fully unblocked.** `AgentContext` and `Journal` types depend
   only on `tui:shell`. They can be implemented, unit-tested, and proven before
   any binary skeleton exists.

3. **The `robopoker` blocker is owned by `games:traits`.** That lane should proceed
   on its own schedule. `agent:experience` can reach Slice 4 without it.

4. **An honest first slice is available.** `agent_context.rs` (Slice 1) is a
   well-defined, self-contained type with clear acceptance criteria: load/save
   roundtrip, missing-file creates default, journal appends not overwrites.

---

## What Implementation-Family Means Here

Following the `outputs/README.md` convention, an implementation-family lane produces:

| Artifact | Meaning |
|----------|---------|
| `spec.md` | Lane contract and next slices (owned by `agent:experience` spec) |
| `review.md` | Trust assessment and keep/reopen/reset judgment (this document) |
| `implementation.md` | Records the concrete slice the lane changed |
| `verification.md` | Records proof result, residual risks, next slice |

`agent:integration` is the **review and integration layer** — it consumes the
`agent:experience` spec and review, determines readiness, and produces the adapter
capability spec. Implementation belongs to `agent:experience`.

The decision to enter implementation-family workflow means:
- **`agent:experience` begins Slice 1 (`agent_context.rs`) immediately**
- The `robopoker` git migration is tracked as a Phase 2 gate in `agent:experience`'s
  own review, not as a prerequisite for this lane
- The `play:tui` binary skeleton is tracked as a Slice 3 prerequisite, not a
  Slice 1 prerequisite
- `agent:integration` artifacts (this document + `agent-adapter.md`) are complete

---

## Lane Readiness

| Dimension | Status | Notes |
|-----------|--------|-------|
| `agent:experience` spec | **READY** | 9 slices defined, boundaries clear |
| `agent:experience` review | **KEEP** | Proceed to implementation |
| Upstream `tui:shell` | **TRUSTED** | 82 tests pass |
| Upstream `games:traits` | **TRUSTED** | 14 tests pass |
| Schema (`schema.rs`) | **TRUSTED** | 16 tests, 939 lines |
| robopoker git migration | **BLOCKER (owned elsewhere)** | `games:traits` owns; Phase 2 gate for agent:exp |
| `play:tui` binary | **PARTIAL** | Slice 3+ gate, not Slice 1 gate |
| Agent adapter surface | **DEFINED** | `agent-adapter.md` documents contract |

---

## Recommendation

**Begin `agent:experience` Slice 1 immediately.**

Slice 1 (`agent_context.rs`) is the smallest honest first slice:
- Self-contained type in `crates/myosu-tui/src/agent_context.rs`
- Depends only on `tui:shell` (trusted)
- Three unit tests: load/save roundtrip, missing file creates default, journal appends
- No binary, no chain, no robopoker dependency required for the type itself

**Next lane decision after `agent:experience` Slice 1 completes:**
- If robopoker is still unresolved → continue Slices 2–4, gate Phase 2 on robopoker
- If `play:tui` binary exists → parallelize Slices 3–4 alongside Slice 5
- If both resolved → proceed to Phase 2 (narration + lobby) and Phase 3 (spectator)

**This lane (`agent:integration`) is complete.** The `agent-adapter.md` defines the
integration contract. The `review.md` delivers the decision: implementation-family
workflow is the correct next step.

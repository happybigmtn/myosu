# `agent-integration` Lane Review

## Judgment: **GO** — implementation-family slices 1–2 are unblocked

The `agent:experience` lane has a valid KEEP judgment from its bootstrap review.
This integration review synthesizes the lane artifacts to answer the product-level
question: **does Myosu product need an implementation-family workflow next, or
does something upstream still need to unblock?**

The answer is: **implementation-family slices 1–2 are ready to run now.** Slices
1 (`agent_context.rs`) and 2 (`journal.rs`) have no upstream blockers beyond
`tui:shell`, which is already trusted with 82 passing tests.

---

## Source Artifacts

This review synthesizes:
- `outputs/agent/experience/spec.md` — full lane specification
- `outputs/agent/experience/review.md` — KEEP judgment, 2026-03-19
- `fabro/programs/myosu-product.yaml` — program manifest lane structure
- `fabro/run-configs/product/agent-experience.toml` — bootstrap run config

---

## What the Bootstrap Review Established

The `agent:experience` bootstrap review (KEEP) found:

1. **Spec quality is sound** — AX-01..05 and SP-01..03 are well-reasoned and
   documented.
2. **Upstream is trusted** — `tui:shell` (82 tests) and `games:traits` (14 tests)
   are both in the trusted state.
3. **Schema is production-ready** — `schema.rs` (939 lines, 16 tests pass) is the
   strongest surface in the lane.
4. **Slice dependency chain is clean** — Slices 1–4 have no external dependencies
   beyond `tui:shell`. Slices 5–9 depend on slices 1–4 or on `play:tui`.
5. **Scope is bounded** — agent-to-agent social interaction, system parameter
   autonomy, and emotion/affect modeling are explicitly out of scope.

---

## What This Integration Review Resolves

The bootstrap review left one question open: **given that the lane is KEEP, does
product proceed to implementation or wait for something?**

This review answers that by examining the actual dependency graph of each slice:

| Slice | Content | External Dependencies | Status |
|-------|---------|----------------------|--------|
| 1 | `agent_context.rs` | `tui:shell` only | **UNBLOCKED** |
| 2 | `journal.rs` | `tui:shell` only | **UNBLOCKED** |
| 3 | `--context` wiring | `myosu-play` binary | BLOCKED: `play:tui` |
| 4 | `reflect>` prompt | Slice 3 + `journal.rs` | BLOCKED: `play:tui` |
| 5 | `narration.rs` | `tui:shell` only | **UNBLOCKED** |
| 6 | `--narrate` wiring | Slice 5 + `play:tui` binary | BLOCKED: `play:tui` |
| 7 | Lobby + selection | `play:tui` binary + chain stub | BLOCKED: `play:tui` |
| 8 | `SpectatorRelay` | `myosu-play` binary | BLOCKED: `play:tui` |
| 9 | `SpectateScreen` | Slice 8 + `play:tui` binary | BLOCKED: `play:tui` |

**Slices 1, 2, and 5 are fully unblocked.** Slices 1–2 together form the
smallest honest first implementation slice. Slice 5 can run in parallel since it
has the same unblocked status.

---

## Critical Path Item (Not Owned by This Lane)

The `robopoker` git migration is the highest-priority cross-lane dependency.
All `robopoker` references use absolute filesystem paths instead of git
dependencies. This blocks `cargo build` and `cargo test` in any clean environment.

**Ownership**: `games:traits` lane.
**Impact on this lane**: All slices that call into `games:traits` (which is all of
them) cannot run full integration tests until `robopoker` is resolved. This
affects Slices 5–9 most acutely, but Slices 1–4 also ultimately depend on it.

**This lane should not proceed past Slice 4 without confirming `robopoker`
resolution**, because Slices 5–9 require integration testing that cannot succeed
with absolute-path dependencies.

---

## What Needs to Happen Next

### Immediate (no blockers)

1. **Start `agent:experience` implementation-family workflow** targeting Slices
   1–2 (`agent_context.rs` + `journal.rs`).
2. **Track `games:traits` robopoker resolution** as the critical path item.
   This is not a reason to pause Slices 1–2, but it must be resolved before
   Slice 5 integration testing can succeed.

### Short-term (depends on `play:tui` Slice 1)

3. Once `myosu-play` binary skeleton exists, **Slices 3–4 unblock immediately**.
4. **Slice 5 (`narration.rs`) runs in parallel with Slices 3–4** — it has no
   `play:tui` dependency.

### Medium-term

5. Slices 6–9 remain blocked pending `play:tui` binary + robopoker resolution.
6. Slice 7 (lobby) needs a chain stub for Phase 0 — this is self-contained and
   can be implemented once `play:tui` binary exists.

---

## Lane Readiness for Implementation

| Dimension | Status | Notes |
|-----------|--------|-------|
| Specification | **READY** | `spec.md` defines 9 slices with clear boundaries |
| Review judgment | **KEEP** | `review.md` 2026-03-19 |
| Upstream: `tui:shell` | **TRUSTED** | 82 tests pass |
| Upstream: `games:traits` | **TRUSTED** | 14 tests pass |
| Upstream: `play:tui` | **MISSING** | Binary skeleton not yet built |
| robopoker git dep | **BLOCKER** | Owned by `games:traits` lane |
| Implementation slice 1 | **UNBLOCKED** | `agent_context.rs`, depends only on `tui:shell` |
| Implementation slice 2 | **UNBLOCKED** | `journal.rs`, depends only on `tui:shell` |
| Implementation slice 5 | **UNBLOCKED** | `narration.rs`, depends only on `tui:shell` |
| Slices 3–4, 6–9 | **BLOCKED** | Need `play:tui` binary skeleton |

---

## Recommendation

**GO for implementation-family slices 1–2.** The lane is well-specified, the
upstream is trusted, and these two slices have zero external blockers beyond
`tui:shell`. The critical path item (`robopoker` migration, owned by
`games:traits`) should be tracked in parallel — it does not block Slices 1–2
but must be resolved before Slice 5 integration testing.

The honest next product action is to launch the implementation-family workflow
for `agent:experience` targeting Slices 1 and 2, while monitoring
`games:traits` for robopoker resolution and `play:tui` for binary skeleton
completion.

# `agent-integration` Lane Review

## Judgment: **KEEP** — proceed to implementation-family workflow (Track A)

This integration artifact confirms the `agent:experience` lane is ready for implementation, catalogues the concrete integration surfaces and blockers, and delivers a clear two-track recommendation to the control plane.

---

## Rationale for KEEP

### 1. The `agent:experience` lane has a clean KEEP verdict

From `outputs/agent/experience/review.md`:

> **Proceed to implementation-family workflow next.** The lane is well-specified and the upstream is trusted.

The lane's `spec.md` defines 9 sequential, minimally-coupled slices. The `review.md` confirms:
- Spec quality: AX-01..05 + SP-01..03 are mature drafts
- Upstream is trusted: `tui:shell` (82 tests) and `games:traits` (14 tests) both pass
- Schema is the strongest surface: `schema.rs` (939 lines, 16 tests) fully implemented
- Slice dependency chain is clean: Slices 1–4 independent of external deps beyond `tui:shell`
- Scope is bounded: explicitly excludes agent-to-agent social interaction, autonomy, affect

### 2. The integration blockers are catalogued and owned

The `agent-adapter.md` identifies two HIGH blockers:

| Blocker | Owner | Impact | Resolution Path |
|---------|-------|--------|----------------|
| `robopoker` absolute-path deps | `games:traits` lane | Blocks full integration testing past Slice 4 | Git dependency migration |
| `myosu-play` binary missing | `play:tui` lane (Slice 1) | Blocks Slices 3, 4, 6, 7, 8, 9 | Create crate + `main.rs` skeleton |

Both blockers have clear resolution paths and are **owned by other lanes**, not by `agent:experience`. This is not a fundamental unblock — it is coordination work.

### 3. The schema is a stable, trusted integration contract

`crates/myosu-tui/src/schema.rs` (939 lines, 16 tests passing) is the most production-ready surface in the entire lane. It is the **integration contract** between the game layer and the agent layer. No changes to it are required for any of the 9 slices — all slices consume it as a trusted dependency.

---

## Decision: Track A — Parallel Implementation Family

**Track A (recommended)**: Proceed with implementation-family workflow immediately, treating `myosu-play` binary skeleton as a parallel co-requisite owned by `play:tui`.

**Track B (cautious)**: Wait for `robopoker` git migration to complete first.

The `agent:experience/review.md` explicitly endorses Track A:

> The `robopoker` path issue is a build-system concern, not a functional correctness concern — slices 1–4 can be developed and unit-tested using the existing absolute-path setup.

**Verdict: Track A.** The functional correctness of the 9 slices can be developed independently of the build-system concern. Integration testing at the boundary requires the git migration; unit testing does not.

---

## Integration Readiness Assessment

| Dimension | Status | Notes |
|-----------|--------|-------|
| `agent:experience` spec | **READY** | 9 slices, sequential, minimal coupling |
| `agent:experience` review | **KEEP** | Proceed to implementation |
| Upstream `tui:shell` | **TRUSTED** | 82 tests pass |
| Upstream `games:traits` | **TRUSTED** | 14 tests pass |
| `schema.rs` | **TRUSTED** | 16 tests pass; stable integration contract |
| `pipe.rs` skeleton | **TRUSTED** | 6 tests pass; needs `--context`, `--narrate`, `reflect>`, lobby |
| `myosu-play` binary | **MISSING** | HIGH blocker; owned by `play:tui` Slice 1 |
| `robopoker` git migration | **PENDING** | HIGH blocker; owned by `games:traits` Slice 1 |
| Spectator socket path | **UNCONFIRMED** | `~/.myosu/spectate/` vs `play:tui` data-dir convention |
| Implementation slices 1–4 | **UNBLOCKED** | Can begin immediately; no external deps beyond trusted `tui:shell` |
| Implementation slices 5–9 | **BLOCKED** | Blocked on `myosu-play` binary and upstream slices |

---

## What This Lane Owns

`agent-integration` is not a product lane — it is the **integration synthesis lane** for the `agent:experience` output. Its deliverables are:

| Artifact | Purpose |
|----------|---------|
| `outputs/agent-integration/agent-adapter.md` | Concrete integration surface: flag types, constructor signatures, blocker registry, next-step table |
| `outputs/agent-integration/review.md` | This document — integration verdict and decision record |

The actual implementation of the 9 slices is owned by `agent:experience`. This lane's job is complete once the adapter is documented and the decision is delivered to the control plane.

---

## Recommendation to Control Plane

**Start `agent:experience` Slices 1–4 immediately.** These slices (`agent_context.rs`, `journal.rs`, `--context` wiring, `reflect>` prompt) have no external dependencies beyond the already-trusted `tui:shell`. They can be implemented and unit-tested today.

**In parallel, request `play:tui` Slice 1 (`myosu-play` binary skeleton).** This is the coordination dependency that gates Slices 3+.

**Do not wait for `robopoker` git migration to start.** The migration is a build-system fix, not a functional requirement for slice development. Full integration testing requires it; unit testing does not.

**Update `outputs/agent/experience/review.md` after each slice completes** to track proof availability and remaining blockers, per the recommendation in that review.

---

## Evidence

- `outputs/agent/experience/spec.md` — 9 slices defined, surfaces catalogued
- `outputs/agent/experience/review.md` — KEEP verdict, proof expectations, 4 blockers catalogued
- `crates/myosu-tui/src/pipe.rs` — 6 tests pass; `--pipe` flag exists
- `crates/myosu-tui/src/schema.rs` — 16 tests pass; full `GameState` implementation
- `crates/myosu-games/` — 14 tests pass; `games:traits` trusted upstream
- `crates/myosu-tui/` — 82 tests pass; `tui:shell` trusted upstream
- `Cargo.toml` — `crates/myosu-play/` commented out in members (confirms binary is absent)
- `outputs/play/tui/spec.md` — confirms `myosu-play` binary is owned by `play:tui` lane

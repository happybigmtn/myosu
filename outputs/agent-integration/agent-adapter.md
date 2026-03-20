# Agent Integration Adapter

## Purpose

This document is the integration surface for the `agent:experience` lane. It maps
how the lane's surfaces fit into the broader Myosu product, resolves the
implementation-family vs. upstream-unblock decision, and provides the concrete
slice map for the next honest work unit.

## What `agent:experience` Produces

The lane owns the **agent-facing presentation layer** for Myosu — every surface
through which programmatic agents (LLMs, bots, scripts) perceive and act upon the
game world:

| Surface | File | Status |
|---------|------|--------|
| `--pipe` mode driver | `crates/myosu-tui/src/pipe.rs` | Partial — foundation exists, needs `--context`, `--narrate`, `reflect>`, lobby |
| `AgentContext` struct | `crates/myosu-tui/src/agent_context.rs` | **Missing** — Slice 1 |
| `Journal` struct | `crates/myosu-tui/src/journal.rs` | **Missing** — Slice 2 |
| `NarrationEngine` | `crates/myosu-tui/src/narration.rs` | **Missing** — Slice 5 |
| `SpectatorRelay` | `crates/myosu-play/src/spectate.rs` | **Missing** — Slice 8 |
| `SpectateScreen` | `crates/myosu-tui/src/screens/spectate.rs` | **Missing** — Slice 9 |
| JSON schema | `crates/myosu-tui/src/schema.rs` | **Trusted** — fully implemented, 16 tests pass |
| JSON schema docs | `docs/api/game-state.json` | **Trusted** — complete |

## Slice Dependency Map

The lane is organized into 9 slices across 3 phases. The critical distinction
for scheduling is which slices have upstream dependencies and which are
self-contained:

```
Phase 1 — Agent Identity (tui:shell only — TRUSTED)
  Slice 1: agent_context.rs  ──► no external deps beyond tui:shell  [UNBLOCKED]
  Slice 2: journal.rs        ──► no external deps beyond tui:shell  [UNBLOCKED]
  Slice 3: --context wiring  ──► needs myosu-play binary skeleton       [BLOCKED: play:tui]
  Slice 4: reflect> prompt   ──► needs Slice 3 + journal.rs             [BLOCKED: play:tui]

Phase 2 — Narration + Pipe Mode (Phase 1 complete)
  Slice 5: narration.rs      ──► no external deps beyond tui:shell  [UNBLOCKED]
  Slice 6: --narrate wiring  ──► needs Slice 5 + play:tui binary      [BLOCKED: play:tui]
  Slice 7: lobby + selection ──► needs play:tui binary + chain stub   [BLOCKED: play:tui, chain]

Phase 3 — Spectator (play:tui binary + Phase 1)
  Slice 8: SpectatorRelay    ──► needs myosu-play binary              [BLOCKED: play:tui]
  Slice 9: SpectateScreen    ──► needs Slice 8 + spectate socket       [BLOCKED: play:tui]
```

## Upstream Dependency Status

| Upstream | Status | Used By |
|----------|--------|---------|
| `tui:shell` | **TRUSTED** — 82 tests pass | All slices (GameRenderer, PipeMode, Shell, Theme) |
| `games:traits` | **TRUSTED** — 14 tests pass | All slices (CfrGame, GameType, Profile) |
| `play:tui` binary | **MISSING** — Slice 1 not executed | Slices 3, 4, 6, 7, 8, 9 |
| `robopoker` git dep | **BLOCKER** — absolute path deps | All slices via games:traits |
| `chain:runtime` | **RESTART** — not compiled | Slice 7 (lobby query, stubbed for Phase 0) |

## Critical Path Item: `robopoker` Git Migration

The highest-priority upstream unblock is **not owned by this lane**. The
`games:traits` review identifies that all `robopoker` dependencies use absolute
filesystem paths (`/home/r/coding/robopoker/crates/...`) instead of proper git
dependencies. This blocks `cargo build` and `cargo test` on any clean checkout or
CI environment.

This is tracked under `games:traits` ownership. **This lane cannot proceed past
Slice 4 without the robopoker resolution**, because Slices 5–9 require full
integration testing that cannot run against absolute-path dependencies.

## Decision: Implementation Family Next

**Verdict: Proceed with `agent:experience` implementation-family slices 1–2 now.**

Rationale:
- Slices 1 (`agent_context.rs`) and 2 (`journal.rs`) depend **only** on
  `tui:shell`, which is already trusted with 82 passing tests.
- They have **no dependency** on the `myosu-play` binary skeleton, which is the
  primary `play:tui` deliverable.
- They have **no dependency** on `chain:runtime`.
- They can be implemented, tested, and reviewed independently of all other lanes.
- Slices 1–2 are the smallest honest first slice — they deliver a real,
  verifiable capability (persistent agent context) without depending on any
  untrusted or missing surfaces.

**What to do next:**

1. Start `agent:experience` implementation-family workflow targeting Slices 1–2.
2. Simultaneously track `games:traits` robopoker resolution as the critical path
   item.
3. Monitor `play:tui` Slice 1 (binary skeleton) progress — once that completes,
   Slices 3–4 unblock immediately.
4. Slice 5 (`narration.rs`) can run in parallel with Slices 3–4 since it has no
   `play:tui` dependency.
5. Slices 6–9 remain blocked pending `play:tui` binary + robopoker resolution.

## What `agent-adapter` Is Not

This document does not re-state the full lane specification (see
`outputs/agent/experience/spec.md`) or the full review judgment (see
`outputs/agent/experience/review.md`). It is the **integration synthesis** —
the bridge between the reviewed lane and the next action decision.

## Phase Map Summary

| Phase | Slices | Blocked By | Next Action |
|-------|--------|------------|-------------|
| Phase 1a | 1, 2 | nothing (tui:shell trusted) | **Start now** |
| Phase 1b | 3, 4 | `play:tui` binary skeleton | Wait for play:tui Slice 1 |
| Phase 2 | 5 | nothing (tui:shell trusted) | Run parallel to Phase 1b |
| Phase 2 | 6, 7 | play:tui + chain stub | Wait |
| Phase 3 | 8, 9 | play:tui + Phase 1 | Wait |

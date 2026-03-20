`★ Insight ─────────────────────────────────────`
**What the agent-integration lane does:** It sits between the `agent:experience` spec/review artifacts and the actual implementation work. Its job is to honestly assess which implementation slices can proceed and which are blocked on upstream dependencies.

**Key insight from the analysis:** The 9 `agent:experience` implementation slices split cleanly into:
- **3 that can start now** (slices 1, 2, 5) — depend only on trusted `tui:shell`
- **6 that must wait** — blocked on `play:tui` binary, chain runtime, or sequential dependencies

This is a classic "partially unblocked" scenario where the honest answer is "start implementation for what you can, track the rest."
`─────────────────────────────────────────────────`

## Completed: `agent-integration` Lane Bootstrap

**Artifacts produced:**

- **`outputs/agent-integration/agent-adapter.md`** (8.3KB) — Documents the integration contract between external agents and Myosu across 4 surfaces: pipe mode, agent context, narration mode, and spectator relay. Includes protocol formats, schema, fog-of-war enforcement, and a go/no-go table for each slice.

- **`outputs/agent-integration/review.md`** (5.0KB) — Honest judgment: **KEEP — Proceed with Partial Implementation Family**. Maps all 9 slices by dependency and readiness. Recommends starting slices 1, 2, 5 immediately; tracking remaining slices as blocked.

**Key decision from review:**

| Slices | Status | Action |
|--------|--------|--------|
| 1, 2, 5 | **READY** | Start implementation now |
| 3, 4, 6, 7, 8, 9 | **BLOCKED** | Wait for `play:tui` binary or `chain:runtime` |

The `agent:experience` artifacts are sound. Product needs an implementation family next, but it's partially unblocked — start with the 3 slices that have no upstream dependencies.
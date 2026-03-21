All artifacts are written. Here's a summary of the lane execution:

---

## Lane Execution Summary

**Lane**: `agent-integration`
**Status**: Complete

### What Was Done

1. **Inspected inputs** — Read `AGENTS.md`, `SPEC.md`, `PLANS.md`, master index, fabro-primary-executor decision, `specsarchive/031626-10-agent-experience.md`, `specsarchive/031626-17-spectator-protocol.md`, `docs/api/game-state.json`, and `crates/myosu-tui/src/schema.rs`

2. **Executed `agent:experience` lane** — The lane already had well-formed `spec.md` and `review.md` artifacts at `outputs/agent/experience/`. Verified their correctness against the source specs and codebase:
   - `schema.rs`: 939 lines, 16 tests — **trusted**
   - `pipe.rs`: 217 lines, 6 tests — **solid foundation, incomplete features**
   - `agent_context.rs`, `journal.rs`, `narration.rs`: **all missing** (AX-01..04 unimplemented)
   - Spectator relay and screen: **missing** (SP-01..02 unimplemented)

3. **Produced `agent-adapter.md`** — Documents the integration layer binding agent:experience surfaces to downstream consumers (miners, validators, LLM agents, chain gameplay). Includes wire formats, architectural decisions, and the honest implementation status table.

4. **Produced `review.md`** — Judgment: **UPSTREAM UNBLOCK FIRST**, then implementation-family workflow.

### Decision: Implementation Family or Upstream Unblock?

**Upstream unblock first.** The `agent:experience` lane is correctly KEEP (well-specified, upstream trusted). But the integration assessment reveals two blockers that prevent any implementation work from being tested in a clean environment:

| Blocker | Owner | Priority |
|---------|-------|----------|
| `robopoker` git migration (absolute path → git dep) | `games:traits` lane | **HIGH** — blocks clean env `cargo test` |
| `myosu-play` binary skeleton | `play:tui` lane | **HIGH** — blocks flag-wiring slices 3+ |

**Immediate next actions** (not this lane's work, but documented in review):
- `games:traits` Slice 1: replace `/home/r/coding/robopoker/...` path deps with git deps
- `play:tui` Slice 1: create `myosu-play` binary skeleton with `--pipe` dispatch

After those resolve: `agent:experience` Slices 1–2 can begin immediately (depend only on trusted `tui:shell`), while `play:tui` continues building the binary skeleton for the flag-wiring slices.

### Required Artifacts

| Artifact | Path | Status |
|----------|------|--------|
| `spec.md` | `outputs/agent/experience/spec.md` | Pre-existing, verified honest |
| `review.md` | `outputs/agent/experience/review.md` | Pre-existing, verified honest |
| `agent-adapter.md` | `outputs/agent-integration/agent-adapter.md` | **Written** (252 lines) |
| `review.md` | `outputs/agent-integration/review.md` | **Written** (135 lines) |

`★ Insight ─────────────────────────────────────`
- **Doctrine surfaces vs. implementation surfaces**: This lane revealed that the existing `agent:experience` spec artifacts were pre-written but didn't reflect the honest state of the codebase — `agent_context.rs`, `journal.rs`, and `narration.rs` are all referenced in the spec but don't exist. The integration review correctly identified this as a KEEP-with-implementation-needed situation, not a spec problem.
- **Integration review as gatekeeper**: The `agent-integration` lane's primary value is the judgment call — `agent:experience` is ready to implement BUT the environment isn't set up for clean testing. This is the correct posture for a bootstrap-stage product: spec first, implementation second, integration verification third.
- **Dependency topology determines sequence**: The `robopoker` migration is the most impactful single action because it unblocks the most lanes simultaneously — it's the root of the `games:traits` → `tui:shell` → `agent:experience` dependency chain.
─────────────────────────────────────────────────
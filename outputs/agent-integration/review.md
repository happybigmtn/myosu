# `agent:experience` Lane Integration Review

## Judgment: **PROCEED — implementation family is the correct next step**

The `agent:experience` lane has been reviewed and marked KEEP. Its reviewed
artifacts (`outputs/agent/experience/spec.md` + `review.md`) are sound. The lane
is ready to transition from the bootstrap contract to an implementation-family
workflow. No additional upstream unblock is required before starting the first
implementation slice.

---

## Synthesis: What the Reviewed Artifacts Say

### From `outputs/agent/experience/spec.md`

The lane owns the agent-facing presentation layer for Myosu. It defines 9
implementation slices across 4 phases:

| Phase | Slices | Scope |
|-------|--------|-------|
| Phase 1 (Agent Identity) | 1–4 | `agent_context.rs`, `journal.rs`, `--context` wiring, `reflect>` prompt |
| Phase 2 (Narration + Pipe) | 5–7 | `narration.rs`, `--narrate` wiring, lobby + game selection |
| Phase 3 (Spectator) | 8–9 | Unix socket relay, spectate screen |
| Phase 4 (Chain-connected) | — | Lobby queries miner axon; spectator upgrades to WebSocket |

The spec is well-structured with clear slice boundaries, minimal cross-slice
coupling, and explicit dependency declarations.

### From `outputs/agent/experience/review.md`

The review reached **KEEP** with the following key findings:

1. **Spec quality is high**: AX-01..05 + SP-01..03 are mature drafts. The
   reflection/opt-in design decisions are well-reasoned.
2. **Upstream is trusted**: `tui:shell` (82 tests) and `games:traits` (14 tests)
   are both in the trusted state.
3. **Schema is the strongest surface**: `schema.rs` is fully implemented (939 lines,
   16 tests passing) and doubles as the event format for the spectator protocol.
4. **Slice dependency chain is clean**: Slices 1–4 have no external dependencies
   beyond `tui:shell`.
5. **Scope is bounded**: Agent-to-agent social interaction, agent autonomy over
   system parameters, and emotion/affect modeling are all explicitly out of scope.

### Blockers from the Review

| Blocker | Severity | Owner | Affects |
|---------|----------|-------|---------|
| `robopoker` absolute path deps | HIGH | `games:traits` lane | Slices 5–9 (full integration tests) |
| `myosu-play` binary skeleton | HIGH | `play:tui` lane | Slices 3, 6, 7, 8–9 |
| Chain discovery (lobby) | MEDIUM | `chain:runtime` lane | Slice 7 (Phase 0: stub OK) |
| Spectator socket path convention | LOW | `play:tui` lane | Slice 8 |

---

## The Core Question: Implementation Family or Upstream Unblock?

### Argument for "upstream unblock first"

Both HIGH-severity blockers are owned by other lanes (`games:traits` and
`play:tui`). Starting implementation of `agent:experience` slices that ultimately
depend on those lanes could be wasted work if the upstream interfaces change
before integration.

### Argument for "implementation family now"

The first two slices (Slices 1–2: `agent_context.rs` + `journal.rs`) have **no
dependency on either blocker**. They depend only on `tui:shell` which is
already trusted. This is exactly the pattern established by
`games:traits` implementation: that lane also had upstream uncertainty (the
`robopoker` fork decision) but proceeded with its first slice anyway because
the slice itself was independently implementable.

The `robopoker` git migration is actively being resolved in the `games:traits`
lane. The `myosu-play` binary is being built in `play:tui`. Waiting for both
of those to complete before touching `agent:experience` would serialize three
lanes that can proceed in parallel for the first phase.

### Verdict

**Proceed to implementation family now.** The first honest slice (Slices 1–2)
is implementable without any upstream unblock. The blockers are real but they
affect different slices, and the `games:traits` lane is already actively
resolving the `robopoker` migration. There is no scenario in which waiting
produces a better outcome than starting now.

---

## Lane Readiness Assessment

| Dimension | Status | Evidence |
|-----------|--------|---------|
| Spec quality | **READY** | AX-01..05 + SP-01..03 well-reasoned, clear boundaries |
| Upstream: `tui:shell` | **TRUSTED** | 82 tests passing |
| Upstream: `games:traits` | **TRUSTED** | 14 tests passing |
| Upstream: `play:tui` | **PARTIAL** | Binary skeleton missing (blocks Slices 3+) |
| Schema | **TRUSTED** | 16 tests passing |
| robopoker git dep | **BLOCKER** (for later slices) | Absolute path; git migration in progress |
| First slice dependencies | **CLEAR** | Slices 1–2 depend only on `tui:shell` |
| Slice 3+ dependencies | **BLOCKED** | Awaiting `myosu-play` binary skeleton |

---

## What an Implementation-Family Manifest Needs

Following the `myosu-games-traits-implementation.yaml` pattern:

**Program manifest** (`myosu-agent-experience-implementation.yaml`):
- Unit: `agent` / lane: `experience-implement`
- Produces: `implementation.md`, `verification.md` under `outputs/agent/experience/`
- Milestone chain: `reviewed` → `implemented` → `verified`
- Depends on: `outputs/agent/experience/review.md` (already satisfied)

**Run config** (`agent-experience.toml`):
- `graph = "workflows/implement/agent-experience.fabro"`
- Goal: implement Slices 1–4 (agent identity phase) as the first honest slice

**Workflow** (`agent-experience.fabro`):
- Implement → Simplify → Verify cycle, matching `games-traits.fabro`

**Proof script** (`agent-experience-implement.sh`):
- Run `cargo test -p myosu-tui agent_context::tests`
- Run `cargo test -p myosu-tui journal::tests`
- Verify all three proof gates for Slices 1–2

---

## Decision: Proceed to Implementation Family

**Evidence**:
1. The `agent:experience` lane is KEEP with reviewed `spec.md` and `review.md`
2. Slices 1–2 are implementable today without any upstream unblock
3. `games:traits` is actively resolving the `robopoker` git migration (the main
   integration blocker for later slices)
4. `play:tui` is building the `myosu-play` binary (the CLI wiring blocker)
5. Waiting for either of those to complete before starting `agent:experience`
   would serialize three parallel-capable lanes

**No new upstream unblock is required.** The path is clear for the first honest
implementation slice.

---

## Next Steps

1. **Create** `fabro/programs/myosu-agent-experience-implementation.yaml`
2. **Create** `fabro/run-configs/implement/agent-experience.toml`
3. **Create** `fabro/workflows/implement/agent-experience.fabro`
4. **Create** `fabro/checks/agent-experience-implement.sh`
5. **Execute** the first slice: `agent_context.rs` + `journal.rs`
6. **Update** `outputs/agent/experience/review.md` after each slice to track
   proof availability and remaining blockers

After Slices 1–4 complete: re-evaluate the blocker status before proceeding
to Slices 5–7.

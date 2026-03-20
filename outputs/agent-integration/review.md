# `agent:integration` Lane Review

## Judgment: **PROCEED** — implementation-family next; unblocks are owned elsewhere

This review synthesizes the `agent:experience` lane artifacts (`outputs/agent/experience/spec.md`
and `outputs/agent/experience/review.md`) to answer the product decision:
**does Myosu need an implementation family next, or another upstream unblock first?**

The answer is **implementation-family next**. The upstream blockers are real but are owned
by other lanes. `agent:experience` can begin Phase 1 slices immediately.

---

## Decision Basis

### Evidence from `agent:experience` Review

The `agent:experience` lane passed review with judgment **KEEP** (2026-03-20).
Key evidence:

- **Specification quality**: AX-01..05 and SP-01..03 are mature. The
  reflection/opt-in design is well-reasoned. The append-only journal, optional
  reflection, and fog-of-war enforcement at the relay are sound choices.
- **Trusted upstream**: `tui:shell` (82 tests) and `games:traits` (14 tests) are
  both in the trusted state. `GameRenderer`, `PipeMode`, `CfrGame`, `Profile` are
  all available and compiling.
- **Schema is production-ready**: `schema.rs` (939 lines, 16 tests) is the strongest
  surface in the lane. It is fully implemented and doubles as the spectator event format.
- **Slice dependency chain is clean**: Slices 1–4 have no external dependencies beyond
  `tui:shell`. Slices 5–7 depend on Phase 1 completing. Slices 8–9 depend on
  `play:tui` binary.
- **Scope is bounded**: Agent-to-agent social interaction, autonomy over system
  parameters, and emotion/affect modeling are explicitly out of scope.

### Evidence for Upstream Blockers

| Blocker | Owner | Severity | Blocks | Status |
|---------|-------|----------|--------|--------|
| `robopoker` git migration | `games:traits` lane | HIGH | Phase 1+ integration testing | Unresolved |
| `myosu-play` binary dispatch | `play:tui` lane | HIGH | Slices 3, 6, 7, 9 | Binary skeleton exists; flags not wired |
| Chain discovery (lobby) | `chain:runtime` | MEDIUM | Slice 7 real data | Stubbed for Phase 0 |
| Spectator socket path convention | `play:tui` lane | LOW | Slice 8 | Needs confirmation |

None of these blockers are owned by `agent:experience`. The `games:traits` lane owns
the `robopoker` migration. The `play:tui` lane owns the binary dispatch. The
`chain:runtime` lane owns the miner axon endpoint.

### Evidence for Proceeding

1. **Slices 1–2 can start immediately**: `agent_context.rs` and `journal.rs` depend
   only on `tui:shell`, which is already trusted. There is no reason to wait.

2. **Slices 3–4 can start as soon as `play:tui` binary dispatch exists**: The
   `--context` wiring and `reflect>` prompt are straightforward additions to
   `PipeMode`. They do not require `robopoker` git migration.

3. **Slices 5–7 (narration, lobby) can run in parallel with robopoker migration**:
   The narration engine and lobby rendering are local to `myosu-tui`. They can be
   implemented and unit-tested with stubbed game state until integration testing
   requires the real `robopoker` path.

4. **Schema is trusted and stable**: The JSON schema and `schema.rs` are not
   changing as part of this lane. The integration risk is low.

---

## What Implementation-Family Must Handle

The implementation-family workflow must track the following constraints that the
`agent:experience` review identified:

### Integration constraints for the implementation agent

1. **`robopoker` git migration is a prerequisite for end-to-end testing**, not for
   beginning implementation. Phase 1 slices 1–4 can proceed without it. The
   implementation agent should begin with `agent_context.rs` and `journal.rs`
   immediately and wire `--context` and `reflect>` as soon as `play:tui` binary
   dispatch is available.

2. **Journal append-only invariant is non-negotiable**. The implementation must use
   O_APPEND mode and must never call `seek(0)` or `rewrite`. A single truncation
   silently corrupts agent memory with no detectable error at write time.

3. **Pipe mode format is a stable contract**. Existing field keys and value formats
   must not change. New fields may be added. If a breaking change is required, the
   implementation must version the format (e.g., `pipe_v2`) and provide a migration
   path.

4. **Fog-of-war is enforced at the relay, not at the renderer**. Any implementation
   of `SpectatorRelay` must redact `hole_cards` before emitting, regardless of what
   the renderer sends. The relay is the last gateway.

5. **Slices 5–7 (narration, lobby) should use stubbed game state for unit tests**
   until `robopoker` git migration lands. The test surface `GameState` builder in
   `schema.rs` is sufficient for isolated testing.

6. **Slice 7 lobby queries are stubbed for Phase 0**. Real chain data requires
   `chain:runtime` completion. The implementation should accept hardcoded lobby
   data as a temporary measure and wire the chain query interface only.

### Ownership transfer

| Slice | Owner | Depends on |
|-------|-------|-----------|
| Slice 1: `agent_context.rs` | implementation agent | tui:shell (trusted) |
| Slice 2: `journal.rs` | implementation agent | tui:shell (trusted) |
| Slice 3: `--context` wiring | implementation agent | `play:tui` binary dispatch |
| Slice 4: `reflect>` prompt | implementation agent | Slice 3 |
| Slice 5: `narration.rs` | implementation agent | Slice 3 |
| Slice 6: `--narrate` wiring | implementation agent | Slice 5 |
| Slice 7: lobby | implementation agent | Slice 3 + chain discovery stub |
| Slice 8: `SpectatorRelay` | implementation agent | Slice 3 |
| Slice 9: `SpectateScreen` | implementation agent | Slice 8 |

---

## Remaining Blockers for the Product

These are not `agent:experience` lane blockers, but they are product-level blockers
that affect when the full agent experience is functional:

| Blocker | Owner | Severity | Blocks |
|---------|-------|----------|--------|
| `robopoker` git migration | `games:traits` | HIGH | Full integration testing |
| `play:tui` binary dispatch | `play:tui` | HIGH | Slices 3, 6, 7, 9 |
| `poker-engine` lane | `poker-engine` | HIGH | Real solver responses in pipe mode |
| `chain:runtime` | `chain:runtime` | MEDIUM | Real lobby data; spectator WS upgrade |
| `miner` service | `miner` | HIGH | Live subnet info; real exploitability scores |

These are tracked in the respective lane artifacts. They do not block the
implementation agent from beginning Phase 1 work on `agent:experience`.

---

## Recommendation

**Begin the implementation-family workflow for `agent:experience` now.**

The lane is ready. The upstream is trusted. The slices are defined. The primary
blockers (`robopoker` git migration, `play:tui` binary dispatch) are owned by
other lanes and should be tracked there. The implementation agent should start
with Slices 1 and 2 immediately, wire Slices 3 and 4 as soon as the
`play:tui` binary dispatch is available, and continue through the slice chain
while other lanes resolve their blockers in parallel.

The next honest review of this lane should occur after Slice 4 completes
(`reflect>` prompt wired), at which point the first user-visible agent behavior
is functional and the integration constraints can be re-assessed against the
actual implementation.

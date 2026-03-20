# `agent:experience` Integration Review

## Forward-Looking Decision: **IMPLEMENTATION — start Slices 1–2 now; coordinate upstream unblocks for Slices 5–9**

---

## Decision Rationale

The `agent:experience` lane is the last remaining ready product lane. Its `spec.md` and `review.md` (in `outputs/agent/experience/`) are complete and honest. The question this review answers is not whether the lane is ready in isolation — the lane review already says KEEP — but what the product should do **next**: proceed to an implementation-family workflow, or invest in another upstream unblock first.

**The honest answer: both are true, in sequence.**

### Evidence for Implementation Now (Slices 1–2)

1. **Upstream is trusted**: `tui:shell` has 82 tests passing and `games:traits` has 14 tests passing. Slices 1 (`agent_context.rs`) and 2 (`journal.rs`) depend only on `tui:shell`'s `Shell`, `GameRenderer`, and `PipeMode` — all trusted.

2. **No external dependencies for Slices 1–2**: `agent_context.rs` and `journal.rs` are pure local state management. They do not call into `robopoker`, do not need the `myosu-play` binary, and do not need chain connectivity.

3. **Clean slice boundary**: Slice 1 (identity/memory) and Slice 2 (journal) are independently testable. `cargo test -p myosu-tui agent_context::tests::*` and `cargo test -p myosu-tui journal::tests::*` are the proof gates — no environment setup required beyond a temp directory.

4. **Schema is already trusted**: `schema.rs` (939 lines, 16 tests) is the most production-ready surface in the lane. It does not need to change for Slices 1–2.

### Evidence for Upstream Coordination (Slices 5–9)

5. **robopoker git migration is on the critical path for Slices 5–9**: Both `tui:shell` and `games:traits` use absolute filesystem paths to `robopoker`. Slices 5–9 require full integration testing through the entire stack. Until `robopoker` is a proper git dependency, CI cannot run these slices.

6. **`myosu-play` binary is the gate for Slices 3, 6, 7, 8, 9**: All agent-facing flag wiring (`--context`, `--narrate`, lobby, spectate) requires the binary's CLI dispatch. `play:tui` lane owns this. Confirm `play:tui` Slice 1 (binary skeleton) timeline before scheduling these.

7. **Chain discovery stub is a known gap for Slice 7**: The lobby (Slice 7) requires subnet information. The stub approach (hardcoded data for Phase 0) is documented in the lane spec, but the real integration depends on `chain:runtime`.

---

## Integration Status Summary

| Dimension | Status | Evidence |
|-----------|--------|---------|
| Lane spec quality | **SOUND** | `outputs/agent/experience/spec.md` — 9 slices, clear boundaries, 368 lines |
| Lane review judgment | **KEEP** | `outputs/agent/experience/review.md` — "proceed to implementation-family workflow" |
| Upstream (`tui:shell`) | **TRUSTED** | 82 tests pass |
| Upstream (`games:traits`) | **TRUSTED** | 14 tests pass |
| Schema (`schema.rs`) | **TRUSTED** | 16 tests pass, 939 lines |
| robopoker git migration | **BLOCKER** | Owned by `games:traits` lane; on critical path for Slices 5–9 |
| `myosu-play` binary | **BLOCKER** | Owned by `play:tui` lane; required for Slices 3, 6, 7, 8, 9 |
| Slice 1–2 implementability | **READY NOW** | No external deps beyond trusted `tui:shell` |
| Slice 3–4 implementability | **BLOCKED** | Waiting on `play:tui` Slice 1 |
| Slice 5–9 implementability | **BLOCKED** | Waiting on robopoker git migration + `play:tui` binary |

---

## Recommendation

### Immediate (this session)

**Start `agent:experience` Slices 1 and 2** under the `implement/` workflow family.

The `implement/` family is the correct fit because:
- These are bounded slices with clear proof commands
- The main risk is implementation quality, not architecture
- `games:traits` is already using `implement/game-traits.fabro` — same pattern

**Slice 1** (`agent_context.rs`): `AgentContext` struct with `load()`, `save()`, `default()`; JSON serialization; roundtrip test.
**Slice 2** (`journal.rs`): `Journal` struct; append-only markdown; `append_hand_entry()`; `never_truncates` invariant.

Proof gate: `cargo test -p myosu-tui agent_context::tests::*` and `cargo test -p myosu-tui journal::tests::*`.

### Near-term (parallel)

1. **Coordinate with `games:traits` lane** to confirm robopoker git migration timeline. This is the critical path for Slices 5–9.

2. **Coordinate with `play:tui` lane** to confirm binary skeleton timeline (Slice 1 of that lane). This gates Slices 3, 6, 7, 8, 9.

3. **Stub chain discovery** in `agent_context.rs` for the lobby (Slice 7 Phase 0). Use hardcoded `SubnetInfo` data. Do not wait for `chain:runtime` to implement the lobby.

### Medium-term

- Slices 3–4 (--context wiring + reflect prompt) once `play:tui` binary lands
- Slices 5–6 (narration engine + --narrate flag) once Slices 1–4 complete
- Slices 7 (lobby) once chain stub is in place
- Slices 8–9 (spectator relay + screen) once `play:tui` binary + `chain:runtime` stable

---

## What Would Change This Decision

| If... | Then... |
|-------|---------|
| `games:traits` lane completes robopoker git migration | Slices 5–9 become immediately actionable |
| `play:tui` lane produces binary skeleton | Slices 3, 6, 7 become actionable |
| `chain:runtime` lane produces subnet query API | Slice 7 lobby can use real data instead of stub |
| Any slice encounters an architectural problem | Escalate to `conformance/` family for spec audit before proceeding |
| Schema tests begin failing after a slice | Roll back slice; do not proceed until schema is restored as trusted |

---

## Integration Review Judgment

| Question | Answer |
|-----------|--------|
| Is `agent:experience` spec sound? | **YES** |
| Is upstream trusted? | **YES** (`tui:shell`, `games:traits`) |
| Are there blockers? | **YES** — owned by OTHER lanes, not `agent:experience` |
| Should implementation start? | **YES — Slices 1–2 now, Slices 3–9 as upstream unblocks** |
| Does product need another upstream unblock? | **YES — coordinate with `games:traits` and `play:tui`** |
| What family next? | **`implement/` for Slices 1–2** |

---

## Coordination Actions Required

| Action | Owner | Blocks |
|--------|-------|--------|
| Confirm robopoker git migration timeline | `games:traits` lane | Slices 5–9 |
| Confirm `myosu-play` binary skeleton timeline | `play:tui` lane | Slices 3, 6, 7, 8, 9 |
| Implement Slice 1 (`agent_context.rs`) | `agent:experience` lane | Slice 3 |
| Implement Slice 2 (`journal.rs`) | `agent:experience` lane | Slice 4 |
| Wire `--context` flag to CLI | `agent:experience` + `play:tui` | Slice 3 |
| Stub lobby chain queries | `agent:experience` lane | Slice 7 Phase 0 |

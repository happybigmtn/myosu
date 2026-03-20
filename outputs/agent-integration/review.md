# `agent:integration` Lane Review

## Judgment: **PROCEED — product needs an implementation family next**

The `agent:experience` lane is the last remaining ready product lane. Its review delivers a clear KEEP judgment with a defined implementation path. The product frontier should now open an implementation-family workflow for `agent:experience` rather than waiting for additional upstream unblocking.

---

## Rationale for PROCEED

### 1. `agent:experience` is specification-complete

The lane has a mature spec (`spec.md`) covering 9 implementation slices across 4 phases. The schema (`schema.rs`, 939 lines, 16 tests) is already trusted. The pipe mode driver exists and compiles. The lane does not need further specification work before implementation begins.

### 2. Upstream is sufficiently trusted

`tui:shell` (82 tests) and `games:traits` (14 tests) are both in the trusted state. Implementation can begin against these contracts. The remaining upstream risk is:
- `robopoker` git migration — blocks full integration testing, but Slice 1 (`agent_context.rs`) and Slice 2 (`journal.rs`) can proceed immediately since they depend only on serde JSON, not on `robopoker` directly.
- `play:tui` binary skeleton — blocks Slice 3+ CLI wiring, but the core modules can be built in isolation.

### 3. The 9 slices are sequential and minimally coupled

Slices 1–4 have no external dependencies beyond `tui:shell`. Slices 5–7 depend on Phase 1 completing. Slices 8–9 depend on the `play:tui` binary existing. This means work can begin on Slices 1–2 immediately while `play:tui` Slice 1 is being completed in parallel.

### 4. The `games:traits` blocker is being handled separately

The `robopoker` git migration is owned by the `games:traits` lane. That work is tracked there and does not need to block `agent:experience` from beginning. When `games:traits` completes the migration, full integration testing for `agent:experience` will become possible.

### 5. No alternative upstream unblock is actionable

The other options for the product frontier are:
- Waiting for `chain:runtime` — this is a much larger effort and is on the critical path for multiple lanes
- Waiting for `play:tui` binary skeleton — this can proceed in parallel with `agent:experience` Slices 1–2

There is no single upstream unblock that, if resolved, would unblock all of `agent:experience`. The lane is ready to begin with the available upstream trust level.

---

## Decision: Implementation Family Workflow

**Product needs an implementation family workflow next.** The `agent:experience` lane should transition from bootstrap (spec + review) to implementation (slice execution).

The implementation should proceed in phase order:

| Phase | Slices | Immediate Action |
|-------|--------|-----------------|
| Phase 1 | Slice 1 → Slice 4 | Begin `agent_context.rs` and `journal.rs` immediately |
| Phase 2 | Slice 5 → Slice 7 | Begin `narration.rs` after Phase 1; coordinate with `play:tui` for binary |
| Phase 3 | Slice 8 → Slice 9 | Await `play:tui` Slice 1 (binary skeleton) |

---

## Proof Expectations for Implementation Readiness

To consider the implementation-family workflow **proven**, the following evidence must be available at each slice boundary:

| Slice | Proof |
|-------|-------|
| Slice 1 | `cargo test -p myosu-tui agent_context::tests::load_and_save_roundtrip` passes |
| Slice 2 | `cargo test -p myosu-tui journal::tests::append_hand_entry` passes |
| Slice 3 | Agent plays 10 hands, shuts down, restarts → memory preserved |
| Slice 4 | `cargo test -p myosu-tui pipe::tests::reflection_prompt_after_hand` passes |
| Slice 5 | `cargo test -p myosu-tui narration::tests::narrate_includes_board_texture` passes |
| Slice 6 | `--narrate` and `--pipe` produce identical game state from same input |
| Slice 7 | `cargo test -p myosu-tui pipe::tests::lobby_presented_without_subnet_flag` passes |
| Slice 8 | `cargo test -p myosu-play spectate::tests::relay_emits_events` passes |
| Slice 9 | `cargo test -p myosu-tui spectate::tests::renders_fog_of_war` passes |

---

## Remaining Blockers and Their Owners

| Blocker | Severity | Owner | Impact on `agent:experience` |
|---------|----------|-------|------------------------------|
| `robopoker` git migration | HIGH | `games:traits` | Full integration testing blocked; Slices 1–2 unaffected |
| `myosu-play` binary skeleton | HIGH | `play:tui` | Slices 3, 6, 7, 8, 9 need CLI dispatch |
| Chain discovery for lobby | MEDIUM | `chain:runtime` | Slice 7 lobby stubbed for Phase 0 |
| Spectator socket path | LOW | `play:tui` | Slice 8 uses `~/.myosu/spectate/`; needs confirmation |

---

## Lane Readiness Assessment

| Dimension | Status | Notes |
|-----------|--------|-------|
| Specification | **COMPLETE** | 9 slices defined; schema trusted; pipe mode driver exists |
| Upstream trust | **SUFFICIENT** | `tui:shell` (82 tests), `games:traits` (14 tests) both trusted |
| Upstream blockers | **NON-BLOCKING** | `robopoker` and `play:tui` owned elsewhere; work can proceed |
| Implementation slices | **DEFINED** | Sequential, minimal coupling, phase-ordered |
| Integration adapter | **COMPLETE** | `agent-adapter.md` captures all integration points |
| Decision | **PROCEED** | Implementation family workflow is the right next step |

---

## Recommendation

Open an implementation-family workflow for `agent:experience`. Begin with:
1. **Immediate**: Slices 1 (`agent_context.rs`) and 2 (`journal.rs`) — no external dependencies beyond trusted `tui:shell`
2. **Parallel**: Coordinate with `play:tui` lane for binary skeleton (Slice 3 dependency)
3. **When ready**: Slices 5–7 (narration + lobby) after Phase 1 completes
4. **Eventually**: Slices 8–9 (spectator) after `play:tui` binary exists

Do not wait for `robopoker` git migration to begin. The core modules can be built and tested in isolation.

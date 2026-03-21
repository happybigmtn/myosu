# `agent-integration` Lane Review

## Judgment: **KEEP** — proceed with implementation family

This lane is a sequencing and integration review, not a specification or implementation lane. Its job is to observe the reviewed state of `agent:experience` and all upstream dependencies and render an honest verdict on whether the implementation family can start.

The verdict is **KEEP**: the implementation family can and should begin now, starting with slices 1–2 which have no external blockers.

---

## Rationale for KEEP

1. **`agent:experience` is KEEP-reviewed.** `outputs/agent/experience/review.md` explicitly recommends "proceed to implementation-family workflow next." This lane does not override that judgment — it confirms it with additional dependency analysis.

2. **Slices 1–2 have no external blockers.** `agent_context.rs` and `journal.rs` depend only on `tui:shell` (82 tests, trusted) and `games:traits` (14 tests, trusted). They can be implemented and tested entirely independently of the `myosu-play` binary or `play:tui` lane.

3. **`games:traits` is unblocked.** Slice 1 (robopoker git rev pins) is complete. The remaining `robopoker` absolute-path references in `myosu-tui/Cargo.toml` are a cleanliness issue but do not block `agent:experience` slices 1–2, which do not call `robopoker` directly.

4. **`play:tui` Slice 1 is the honest critical path.** Slices 3–9 of `agent:experience` all require modifying the `myosu-play` binary CLI dispatch. The binary does not exist yet. This is a real dependency, but it is owned by `play:tui` lane — which is itself KEEP-reviewed. The implementation family should start where it can (slices 1–2) and coordinate with `play:tui` Slice 1 as the parallel critical path.

5. **No RESET-worthy blocker.** A RESET would be warranted only if `agent:experience` were blocked on an upstream that the product lane could itself unblock. That is not the case here. The `play:tui` binary is owned externally to this decision, `robopoker` git migration is owned by `games:traits`, and `chain:runtime` is not required for Phase 0.

---

## Decision: Implementation Family Next

**The product frontier should start the `agent:experience` implementation family now.**

The sequencing is:

```
START NOW (no external deps beyond tui:shell):
  - agent:experience Slice 1: agent_context.rs
  - agent:experience Slice 2: journal.rs

PARALLEL CRITICAL PATH (owned by play:tui lane):
  - play:tui Slice 1: myosu-play binary skeleton + CLI dispatch

AFTER play:tui Slice 1 lands:
  - agent:experience Slices 3-7: --context, reflect>, narration, --narrate, lobby
  - agent:experience Slices 8-9: SpectatorRelay, SpectateScreen (after Phase 2)

AFTER chain:runtime (Phase 2 only):
  - Lobby queries miner axon (real chain data)
  - Spectator WS upgrade via miner axon
```

---

## Proof Expectations

The implementation family is **proven** when:

| Slice | What | Proof gate |
|-------|------|-----------|
| 1 | `agent_context.rs` | `cargo test -p myosu-tui agent_context::tests::load_and_save_roundtrip` |
| 1 | `agent_context.rs` | `cargo test -p myosu-tui agent_context::tests::journal_appends_not_overwrites` |
| 1 | `agent_context.rs` | `cargo test -p myosu-tui agent_context::tests::missing_context_creates_new` |
| 2 | `journal.rs` | `cargo test -p myosu-tui journal::tests::append_hand_entry` |
| 2 | `journal.rs` | `cargo test -p myosu-tui journal::tests::never_truncates` |
| 3 | `--context` wiring | Agent plays 10 hands → restart → memory preserved |
| 4 | `reflect>` prompt | `cargo test -p myosu-tui pipe::tests::reflection_prompt_after_hand` |
| 5 | `narration.rs` | `cargo test -p myosu-tui narration::tests::narrate_includes_board_texture` |
| 6 | `--narrate` wiring | `--narrate` output contains board texture + session arc |
| 7 | Lobby | `cargo test -p myosu-tui pipe::tests::lobby_presented_without_subnet_flag` |
| 8 | `SpectatorRelay` | `cargo test -p myosu-play spectate::tests::relay_emits_events` |
| 9 | `SpectateScreen` | `cargo test -p myosu-tui spectate::tests::renders_fog_of_war` |

---

## Remaining Blockers

### 1. `myosu-play` Binary Does Not Exist (HIGH — blocks slices 3–9)

The `myosu-play` binary is defined in `play:tui`'s spec and is the vehicle through which all `--pipe`, `--context`, `--narrate`, and `--spectate` flags are exposed. The binary skeleton does not exist yet.

**Impact**: Slices 3 (`--context` wiring), 6 (`--narrate` wiring), 7 (lobby), and 8–9 (spectator) all require modifications to `myosu-play`'s `main.rs` CLI dispatch.

**Resolution**: `play:tui` lane Slice 1 must complete before or concurrently with `agent:experience` slices 3–7.

### 2. `robopoker` Absolute Paths in `myosu-tui/Cargo.toml` (MEDIUM — blocks full integration testing)

`crates/myosu-tui/Cargo.toml` still has absolute filesystem path references to `robopoker`. This prevents `cargo test --all` from passing on a clean checkout or CI environment.

**Impact on slices 1–2**: Low — `agent_context.rs` and `journal.rs` do not call `robopoker` directly. They call through `tui:shell` and `games:traits`, which are already git-dep migrated.

**Impact on slices 3–9**: High — full integration testing requires the entire crate graph to build without local `robopoker`.

**Resolution**: `games:traits` lane should extend its robopoker git migration work to cover `myosu-tui`. Until then, slices 1–2 can proceed independently.

### 3. Spectator Socket Path Convention Not Confirmed (LOW — affects slice 8)

AC-SP-01 specifies `~/.myosu/spectate/<session_id>.sock` — this convention should be confirmed against `play:tui`'s data directory convention (`{data-dir}/hands/hand_{N}.json`).

**Resolution**: Verify `play:tui` data directory convention before slice 8 implementation. Likely no change needed, but must be confirmed.

---

## Lane Readiness

| Dimension | Status | Notes |
|-----------|--------|-------|
| `agent:experience` spec | **READY** | AX-01..05 + SP-01..03 are sound; 9 slices defined |
| `agent:experience` review | **KEEP** | "Proceed to implementation-family workflow next" |
| `tui:shell` upstream | **TRUSTED** | 82 tests pass |
| `games:traits` upstream | **TRUSTED** | 14 tests pass; Slice 1 git rev done |
| `play:tui` binary | **MISSING** | Slice 1 required before slices 3–9 |
| `robopoker` in `myosu-tui` | **BLOCKER (medium)** | Blocks full integration testing; doesn't stop slices 1–2 |
| `chain:runtime` | **NOT REQUIRED for Phase 0** | Phase 2 only |
| Schema (`schema.rs`) | **TRUSTED** | Full implementation; 16 tests pass |

---

## What This Lane Does Not Decide

- **Which agent uses Myosu first.** The adapter is game-agnostic and protocol-agnostic beyond the pipe mode contract. Any LLM, bot, or script that can read stdout and write stdin can use it.
- **The structure of the implementation family.** That is owned by the lane that implements it. This lane only confirms the sequencing.
- **The fate of `agent:experience` after slices 1–9.** If the implementation family encounters a structural problem that requires changing the spec, `agent:experience` should be reopened.

---

## Recommendation

**Start the `agent:experience` implementation family now, in two parallel tracks:**

1. **Track A (starts now):** Slices 1–2 (`agent_context.rs`, `journal.rs`). No external dependencies beyond trusted `tui:shell`.
2. **Track B (starts now, owned by `play:tui` lane):** Slice 1 — `myosu-play` binary skeleton + CLI dispatch.

When Track B lands, Tracks A and B merge and slices 3–9 proceed.

The `agent-integration` lane has served its purpose. It has rendered an honest verdict: **KEEP, with `play:tui` Slice 1 as the critical path.** The next `agent-integration` artifact should be a decision log entry recording that the implementation family has started.

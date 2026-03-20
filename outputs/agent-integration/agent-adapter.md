# `agent:experience` Integration Adapter

## Purpose

This document maps the `agent:experience` lane specification to the concrete codebase surfaces
that must be created or modified to implement it. It is the integration contract: the set of
coupling points, slice dependencies, and adapter promises that the implementation lane must honor.

Source artifacts:
- `outputs/agent/experience/spec.md` — lane specification (AX-01..05, SP-01..03)
- `outputs/agent/experience/review.md` — lane review (JUDGMENT: KEEP)

---

## Integration Surface

### Upstream Providers (what this lane consumes)

| Provider | Surface Used | Trust Status |
|----------|-------------|--------------|
| `tui:shell` | `GameRenderer` trait, `PipeMode` driver, `Events`, `Theme` | **Trusted** — 82 tests pass |
| `games:traits` | `CfrGame`, `Profile`, `GameConfig`, `GameType`, `StrategyQuery/Response` | **Trusted** — 14 tests pass |
| `play:tui` binary | `myosu-play` CLI dispatch, main.rs entrypoint | **Missing** — binary skeleton absent; blocks Slices 3+ |
| `robopoker` | `Game`, `Recall`, `Action` (via `games:traits` re-exports) | **BLOCKER** — absolute path deps unresolved in `tui:shell` and `games:traits` |

### Downstream Consumers (what consumes this lane)

| Consumer | Interface Used |
|----------|---------------|
| LLM agents | `myosu-play --pipe --context <path> --narrate` (stdin/stdout pipe protocol) |
| Bot scripts | JSON schema via `docs/api/game-state.json` + `crates/myosu-tui/src/schema.rs` |
| Spectator clients | Unix domain socket at `~/.myosu/spectate/<session_id>.sock` (Phase 0) |

### Adapter Contract

The `agent:experience` adapter promises:

1. **Pipe protocol stability** — `myosu-play --pipe` produces ANSI-free, machine-parseable text on stdout and reads actions on stdin, regardless of game type or player position.

2. **Context file isolation** — agent context files are serde-validated on load; invalid files produce a clear error and do not crash the binary; context is never exposed to opponents.

3. **Journal append-only guarantee** — `journal.rs` never truncates; `save()` is called at each hand completion; the file grows monotonically.

4. **Fog-of-war at the relay** — `SpectatorRelay` strips hole cards from all events before `showdown`; enforcement is at the relay, not at the renderer.

5. **Schema fidelity** — the `GameState` JSON produced by `schema.rs` is the canonical machine-readable representation; pipe output and narration must be consistent with it.

---

## Files to Create

| File | Slices | Responsibility |
|------|--------|----------------|
| `crates/myosu-tui/src/agent_context.rs` | 1, 3 | `AgentContext` struct; `load()`, `save()`, `default()`; serde JSON; journal field management |
| `crates/myosu-tui/src/journal.rs` | 2, 4 | `Journal` struct; `append_hand_entry()`; append-only markdown; never truncates |
| `crates/myosu-tui/src/narration.rs` | 5, 6 | `NarrationEngine::narrate(&GameState) -> String`; board texture analysis; session arc weaving |
| `crates/myosu-tui/src/pipe.rs` | 3, 4, 6, 7 | Extend `PipeMode` with `--context`, `--narrate`; add `reflect>` prompt; add lobby rendering |
| `crates/myosu-play/src/spectate.rs` | 8 | `SpectatorRelay`; Unix domain socket; `emit(&GameEvent)`; fog-of-war enforcement |
| `crates/myosu-tui/src/screens/spectate.rs` | 9 | `SpectateScreen`; fog-of-war rendering; `r`/`n`/`q` keybindings |

---

## Files to Modify

| File | Slices | Change |
|------|--------|--------|
| `crates/myosu-play/src/main.rs` | 3, 6, 7 | Add `--context <path>`, `--narrate` CLI flags; wire to `PipeMode` constructor |
| `crates/myosu-tui/src/pipe.rs` | 3, 4, 6, 7 | Add `context_path: Option<PathBuf>` and `narrate: bool` fields to `PipeMode`; add lobby rendering branch |
| `crates/myosu-tui/src/shell.rs` | 9 | Add `Screen::Spectate` variant if not already present |

---

## Slice Dependency Chain

```
Slice 1 (agent_context.rs)
  └─► Slice 3 (--context wiring in PipeMode)
            └─► Slice 4 (reflect> prompt)
                      └─► Slice 7 (lobby + game selection)

Slice 2 (journal.rs)
  └─► Slice 4 (reflect> prompt — journal appends)

Slice 1+2 (agent_context + journal)
  └─► Slice 5 (narration.rs engine)
            └─► Slice 6 (--narrate wiring)
                      └─► Slice 7 (lobby)
                                └─► Slice 8 (SpectatorRelay)
                                          └─► Slice 9 (SpectateScreen)
```

**Key constraint**: Slices 1 and 2 are independent of each other and can run in parallel. Both are required before Slice 5 (narration) can begin.

**Phase gates**:
- Phase 1 (Slices 1–4): Only requires `tui:shell` — no new binary, no new socket
- Phase 2 (Slices 5–7): Requires `tui:shell` + `games:traits` — no new binary, no new socket
- Phase 3 (Slices 8–9): Requires `play:tui` binary skeleton (Slice 1 of `play:tui` lane)
- Phase 4: Requires `chain:runtime` for lobby chain queries and WebSocket upgrade

---

## Coupling Risk Register

### Risk 1: `robopoker` Absolute Path Coupling (HIGH — blocks all phases)
**Location**: `tui:shell` and `games:traits` both reference robopoker via absolute filesystem paths.

**Impact**: All 9 slices ultimately call into `games:traits` or `tui:shell`, which call into robopoker.
`cargo build` and `cargo test` fail on any clean checkout or CI environment.

**What must be preserved**: The `CfrGame`, `Profile`, `Encoder` re-exports in `games:traits`; the `GameRenderer` trait in `tui:shell`.

**What must be reduced**: The `path = "/home/r/coding/robopoker/..."` dependencies must be replaced with git URLs. This is owned by the `games:traits` implementation lane (Slice 1 of that lane). Do not proceed past Phase 1 without this resolved.

**Verification**:
```bash
cargo fetch
cargo test -p myosu-games
```

### Risk 2: `myosu-play` Binary Skeleton Absent (HIGH — blocks Slices 3, 6, 7, 8)
**Location**: No `main.rs` exists at `crates/myosu-play/src/main.rs` that wires `--pipe`, `--context`, `--narrate` flags to `PipeMode`.

**Impact**: Slices 3, 6, 7, and 8 all require modifications to the CLI dispatch layer. Without the binary skeleton, these slices cannot be tested end-to-end.

**What must be preserved**: The `PipeMode` constructor signature; the `Gameshell` enum dispatch in main.

**What must be reduced**: The `play:tui` lane (unit: `play` in `myosu-product.yaml`) owns the binary skeleton. Slice 1 of `play:tui` must complete before or concurrently with `agent:experience` Slice 3.

**Verification**: `cargo build -p myosu-play` exits 0.

### Risk 3: Schema Trust Boundary (MEDIUM — slices 5, 6)
**Location**: `crates/myosu-tui/src/schema.rs` (trusted) — `GameStateBuilder`, `LegalAction`, `GamePhase`.

**Impact**: `NarrationEngine` and pipe output both derive from the same `GameState`. If the schema's interpretation of game state is wrong, both narration and pipe output will be wrong in the same way — which partially mitigates the risk but doesn't eliminate it.

**What must be preserved**: The `schema.rs` trust verdict from `outputs/agent/experience/review.md` — 16 tests pass.

**What must be reduced**: No action in early slices; risk is accepted. The schema is the most production-ready surface in the lane.

### Risk 4: Spectator Socket Path Convention Not Agreed (LOW — blocks Slice 8)
**Location**: `AC-SP-01` specifies `~/.myosu/spectate/<session_id>.sock`.

**Impact**: If `play:tui` uses a different data directory convention (`{data-dir}/hands/hand_{N}.json` per `outputs/play/tui/spec.md`), the spectator socket path must align.

**What must be reduced**: Confirm `play:tui` data directory convention before Slice 8. Likely no change needed, but must be confirmed explicitly.

### Risk 5: Journal File Locking (MEDIUM — slices 2, 4)
**Location**: `crates/myosu-tui/src/journal.rs`.

**Impact**: If two processes open the journal simultaneously (e.g., `myosu-play` and a backup script), the append could race. The append is not atomic at the OS level.

**What must be preserved**: The append-never-truncates invariant.

**What must be reduced**: No action in Phase 1. For Phase 2, consider `fsync` after each append or a file-locking mechanism. This is not a blocker for the first honest slice.

### Risk 6: Chain Discovery Stub in Lobby (MEDIUM — blocks Slice 7)
**Location**: Slice 7 lobby queries chain or miner for active subnet info.

**Impact**: `chain:runtime` is not ready. Hardcoded lobby data must be used for Phase 0.

**What must be preserved**: The lobby rendering in pipe mode; the `info <id>` command interface.

**What must be reduced**: Stub chain queries with hardcoded data. AC-AX-05 can be demonstrated with fake miner counts and exploitability numbers. Real chain integration is Phase 4.

---

## Implementation Sequence Recommendation

Based on slice dependencies and blocker status:

**Immediate (no blockers)**:
- Slice 1 (`agent_context.rs`) — depends only on `tui:shell` (trusted)
- Slice 2 (`journal.rs`) — depends only on `tui:shell` (trusted)

**After Slice 1 complete**:
- Slice 3 (`--context` wiring) — blocked until `play:tui` binary skeleton exists
- Slice 5 (`narration.rs`) — blocked until Slice 2 is complete

**After `play:tui` Slice 1 (binary skeleton)**:
- Slice 3 (`--context` wiring) — can now be wired end-to-end
- Slice 4 (`reflect>`) — depends on Slice 3

**After Slice 5 complete**:
- Slice 6 (`--narrate` wiring)

**After Slices 3+4+6 complete**:
- Slice 7 (lobby)

**After Slice 6+7 complete**:
- Slice 8 (`SpectatorRelay`)
- Slice 9 (`SpectateScreen`)

---

## Concrete Proof Gates

| Slice | Test Command | Expected |
|-------|-------------|----------|
| 1 | `cargo test -p myosu-tui agent_context::tests::load_and_save_roundtrip` | Exit 0 |
| 1 | `cargo test -p myosu-tui agent_context::tests::journal_appends_not_overwrites` | Exit 0 |
| 2 | `cargo test -p myosu-tui journal::tests::append_hand_entry` | Exit 0 |
| 2 | `cargo test -p myosu-tui journal::tests::never_truncates` | Exit 0 |
| 3 | `cargo build -p myosu-play -- --context ./test.json --narrate` | Exit 0 |
| 4 | `cargo test -p myosu-tui pipe::tests::reflection_prompt_after_hand` | Exit 0 |
| 5 | `cargo test -p myosu-tui narration::tests::narrate_includes_board_texture` | Exit 0 |
| 6 | `cargo test -p myosu-tui pipe::tests::narrate_mode_produces_prose` | Exit 0 |
| 7 | `cargo test -p myosu-tui pipe::tests::lobby_presented_without_subnet_flag` | Exit 0 |
| 8 | `cargo test -p myosu-play spectate::tests::relay_emits_events` | Exit 0 |
| 8 | `cargo test -p myosu-play spectate::tests::events_are_valid_json` | Exit 0 |
| 9 | `cargo test -p myosu-tui spectate::tests::renders_fog_of_war` | Exit 0 |

All slices: `cargo test -p myosu-tui schema::tests` must continue to pass (schema trusted surface).

---

## What This Adapter Does NOT Cover

- Agent-to-agent social interaction (explicitly out of scope per AX-01)
- Agent autonomy over system parameters (out of scope per AX-01)
- Emotion/affect modeling (out of scope per AX-01)
- HTTP/WebSocket API surfaces (Phase 2, blocked on `chain:runtime`)
- Miner axon integration for lobby (Phase 4, blocked on `chain:runtime`)
- Multi-agent coordination beyond pipe mode (out of scope per AX-01)

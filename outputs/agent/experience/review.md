# `agent:experience` Lane Review

Historical note: this review predates the current `myosu-play` subcommand CLI.
For the live Stage 0 surface, read `myosu-play --pipe` as `myosu-play pipe`.

## Judgment: **KEEP** — proceed to implementation-family workflow

This lane is ready to move from specification into implementation. The source specs are
sound, the lane boundaries are clear, and the upstream dependencies (tui:shell, games:traits)
are already trusted with passing tests. The 9 implementation slices are sequential and
minimally coupled.

---

## Rationale for KEEP

1. **Spec quality**: `specsarchive/031626-10-agent-experience.md` (AX-01..05) and
   `specsarchive/031626-17-spectator-protocol.md` (SP-01..03) are mature drafts. The
   reflection/opt-in design decisions are well-reasoned and documented in decision logs.
   The append-only journal, optional reflection, and fog-of-war enforcement at the relay
   (not renderer) are all sound architectural choices.

2. **Upstream is trusted**: `tui:shell` (82 tests) and `games:traits` (14 tests) are
   already in the trusted state. The `GameRenderer` trait and `PipeMode` driver exist
   and compile. The lane builds on proven infrastructure, not on speculation.

3. **Schema is the strongest surface**: `schema.rs` is fully implemented (939 lines,
   16 tests passing) and covers 10 game types. It is the most production-ready artifact
   in the lane and doubles as the event format for the spectator protocol. This is a
   solid foundation to build on.

4. **Slice dependency chain is clean**: Slices 1–4 (agent context, journal, --context flag,
   reflect prompt) have no external dependencies beyond `tui:shell`. Slices 5–7 (narration,
   --narrate, lobby) depend on slices 1–4 completing first. Slices 8–9 (spectator relay
   and screen) depend on `play:tui`'s binary existing (which is its own lane's Slice 1).

5. **Scope is bounded**: The lane does NOT include agent-to-agent social interaction,
   agent autonomy over system parameters, or emotion/affect modeling. These are
   explicitly out of scope in AX-01 and will not creep in.

---

## Proof Expectations

To consider this lane **proven**, the following evidence must be available:

| Proof | How to Verify |
|-------|--------------|
| Agent context persists across sessions | `AgentContext::load()` → play 10 hands → `AgentContext::save()` → restart → `AgentContext::load()` → memory + journal preserved |
| Journal is never truncated | Write 1000 hand entries; verify file only grows; `save()` at each step never rewrites |
| Reflection prompt appears after hand | Pipe mode emits `HAND COMPLETE` + `reflect>`; empty line skips; non-empty line appears in journal |
| `--narrate` produces board texture | Narrated output contains "dry" or "wet" or "connected"; same underlying `GameState` in both modes |
| `--narrate` produces session arc | Narrated output contains stack trajectory or opponent history from context file |
| Lobby presented without `--subnet` | `myosu-play pipe` (no subnet) → lobby output → `info 1` → subnet detail output |
| Spectator relay emits valid JSON | Connect to `~/.myosu/spectate/<id>.sock`; receive valid `GameEvent` JSON lines |
| Spectator fog-of-war enforced at relay | Hole cards never appear in relay output during active play; only after `showdown` event |
| Schema tests pass | `cargo test -p myosu-tui schema::tests` exits 0 |

---

## Remaining Blockers

### 1. `robopoker` Git Migration (HIGH — blocks Phase 1+)

Both `tui:shell` and `games:traits` depend on `robopoker` via **absolute filesystem
paths** (`/home/r/coding/robopoker/crates/...`). This is documented in
`outputs/play/tui/spec.md` and `outputs/games/traits/review.md` as the highest-priority
Slice 1 fix for those lanes.

**Impact on this lane**: All 9 slices ultimately call into `games:traits` or `tui:shell`,
which call into `robopoker`. Until `robopoker` is migrated to a proper git dependency
(`git = "https://..."` in `Cargo.toml`), `cargo build` and `cargo test` will fail on
any clean checkout or CI environment.

**Resolution**: `games:traits` lane owns this resolution. This lane should not proceed
past Slice 4 without confirming the `robopoker` dependency is resolved, because slices
5–9 will require full integration testing.

### 2. `myosu-play` Binary Does Not Exist (HIGH — blocks Slice 3+)

The `myosu-play` binary is defined in `play:tui`'s spec and is the vehicle through which
the `pipe` mode plus future `--context`, `--narrate`, and spectator flags are exposed. The binary
skeleton does not exist yet.

**Impact on this lane**: Slices 3 (--context wiring), 6 (--narrate wiring), 7 (lobby),
and 8–9 (spectator) all require modifications to `myosu-play`'s `main.rs` CLI dispatch.

**Resolution**: `play:tui` lane Slice 1 (binary skeleton) must complete before or
concurrently with `agent:experience` Slice 3.

### 3. Chain Discovery Is Stubbed in Lobby (MEDIUM — blocks Slice 7)

The lobby (Slice 7) requires querying the chain or a miner for active subnet
information. AC-AX-05 shows the lobby displaying miner count, exploitability, and
game status — all of which require live data.

**Resolution for Slice 7**: Stub the chain query with hardcoded lobby data for Phase 0.
Real chain integration is Phase 4 (depends on `chain:runtime`).

### 4. Spectator Socket Path Convention Not Agreed (LOW — blocks Slice 8)

AC-SP-01 specifies `~/.myosu/spectate/<session_id>.sock` — this is a convention that
should be confirmed against `play:tui`'s data directory convention (`outputs/play/tui/spec.md`
uses `{data-dir}/hands/hand_{N}.json`). If `play:tui` uses a different base path,
the spectator socket path must align.

**Resolution**: Verify `play:tui` data directory convention before Slice 8 implementation.
Likely no change needed, but must be confirmed.

---

## Lane Readiness

| Dimension | Status | Notes |
|-----------|--------|-------|
| Specification | **READY** | AX-01..05 + SP-01..03 are sound; 9 slices defined with clear boundaries |
| Upstream (tui:shell) | **READY** | 82 tests pass; `GameRenderer`, `PipeMode` trusted |
| Upstream (games:traits) | **READY** | 14 tests pass; `CfrGame`, `GameType`, `StrategyQuery` trusted |
| Upstream (play:tui) | **PARTIAL** | Binary skeleton missing; needed for Slice 3+ |
| Schema (`schema.rs`) | **TRUSTED** | Full implementation; 16 tests pass |
| robopoker dependency | **BLOCKER** | Absolute path deps; must migrate to git before Phase 1 testing |
| Implementation slices | **DEFINED** | 9 slices, sequential, minimal cross-slice coupling |
| Spectator protocol | **SPEC ONLY** | Not yet implemented; fully specified but no code |

---

## Recommendation

**Proceed to implementation-family workflow next.** The lane is well-specified and
the upstream is trusted. The primary blocker (robopoker git migration) is owned by
the `games:traits` lane and should be tracked there. This lane can begin with Slices
1–2 (`agent_context.rs` and `journal.rs`) immediately, as they depend only on
`tui:shell` which is already trusted.

Once `play:tui` Slice 1 (binary skeleton) is complete, Slices 3–4 (--context wiring
and reflect prompt) can proceed in parallel with continued work on Slice 5 (narration).

The `review.md` should be updated after each slice completes to track proof
availability and remaining blockers.

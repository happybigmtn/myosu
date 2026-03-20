# `agent:integration` Lane Review

## Judgment: **KEEP** — proceed with Slices 1–2; coordinate on blockers before Slices 3+

The `agent:experience` lane specification is sound (JUDGMENT: KEEP from `outputs/agent/experience/review.md`).
The integration surface is well-mapped. The 9-slice dependency chain is clean and the coupling risks are
accurately identified. Slices 1 and 2 can begin immediately against the trusted `tui:shell` upstream.
Slices 3–9 are blocked on external dependencies that must be resolved in parallel.

---

## Rationale for KEEP

1. **Spec quality is proven**: The `agent:experience` spec (AX-01..05 + SP-01..03) is mature, the lane boundaries are correct, and the 9-slice decomposition is the smallest honest first slice that produces working code.

2. **Upstream is trusted**: `tui:shell` (82 tests) and `games:traits` (14 tests) are both in the trusted state. The `GameRenderer` trait and `PipeMode` driver exist and compile. The lane builds on proven infrastructure.

3. **Slice dependency chain is genuinely sequential**: Slices 1–4 have no external dependencies beyond `tui:shell`. Slices 5–7 add `narration.rs` and lobby, still only requiring `tui:shell` + `games:traits`. Slices 8–9 (spectator) are gated behind `play:tui`'s binary skeleton. The phasing is real, not artificial.

4. **Coupling risks are correctly identified**: The `agent-adapter.md` accurately registers the two HIGH-risk blockers (robopoker git migration, `play:tui` binary skeleton) and correctly assigns ownership for their resolution.

5. **Schema is the strongest foundation**: `schema.rs` (939 lines, 16 tests passing) is the most production-ready surface in the lane. It is the contract that both pipe output and narration must respect. Its trust verdict from the `agent:experience` review is well-founded.

---

## Blockers

### 1. `robopoker` Git Migration (HIGH — blocks all phases beyond Phase 1)

Both `tui:shell` and `games:traits` depend on `robopoker` via **absolute filesystem paths**
(`/home/r/coding/robopoker/crates/...`). This is documented in `outputs/agent/experience/review.md`
as the highest-priority resolution.

**Impact on this lane**: All 9 slices ultimately call into `games:traits` or `tui:shell`, which
call into `robopoker`. Until the migration is complete, `cargo build` and `cargo test` will fail
on any clean checkout or CI environment.

**Resolution ownership**: `games:traits` implementation lane (Slice 1 of that lane). The `games:traits/implementation.md`
already tracks this as Slice 1.

**This lane's posture**: Slices 1 and 2 can proceed (they don't require `cargo build` of the full
`myosu-tui` crate yet — only the new `agent_context.rs` and `journal.rs` files). However, any
integration testing that exercises the full pipe mode will fail until the robopoker migration is
complete.

### 2. `myosu-play` Binary Skeleton (HIGH — blocks Slices 3, 6, 7, 8)

The `myosu-play` binary is defined in `play:tui`'s spec and is the vehicle through which
all `--pipe`, `--context`, `--narrate`, and `--spectate` flags are exposed. The binary
skeleton does not exist yet.

**Impact on this lane**: Slices 3 (`--context` wiring), 6 (`--narrate` wiring), 7 (lobby),
and 8–9 (spectator) all require modifications to `myosu-play`'s `main.rs` CLI dispatch.

**Resolution ownership**: `play:tui` lane Slice 1 (binary skeleton) in `myosu-product.yaml`.
This lane should not wait passively — the implementation lane for `agent:experience` should
coordinate with `play:tui` to establish the CLI flag contract early.

**This lane's posture**: Slice 3 can be designed against the flag contract now (document the
expected `clap` flags in `agent-adapter.md`), but the end-to-end integration test cannot run
until the binary skeleton exists.

### 3. Chain Discovery Is Stubbed in Lobby (MEDIUM — blocks Slice 7)

The lobby (Slice 7) requires querying the chain or a miner for active subnet information.
AC-AX-05 shows the lobby displaying miner count, exploitability, and game status — all
requiring live data.

**Resolution for Slice 7**: Stub the chain query with hardcoded lobby data for Phase 0.
Real chain integration is Phase 4 (depends on `chain:runtime`). This is acceptable and
documented in `agent-adapter.md`.

### 4. Spectator Socket Path Convention Not Agreed (LOW — blocks Slice 8)

AC-SP-01 specifies `~/.myosu/spectate/<session_id>.sock` — this is a convention that
should be confirmed against `play:tui`'s data directory convention (`{data-dir}/hands/hand_{N}.json`).

**Resolution**: Verify `play:tui` data directory convention before Slice 8 implementation.
Likely no change needed, but must be confirmed explicitly. Low severity because the socket
path is internal to the implementation.

---

## Lane Readiness

| Dimension | Status | Notes |
|-----------|--------|-------|
| Specification (`agent:experience/spec.md`) | **READY** | AX-01..05 + SP-01..03 are sound |
| Integration surface (`agent-adapter.md`) | **READY** | Files, slices, coupling risks all mapped |
| Upstream: `tui:shell` | **TRUSTED** | 82 tests pass; `GameRenderer`, `PipeMode` trusted |
| Upstream: `games:traits` | **TRUSTED** | 14 tests pass; `CfrGame`, `GameType` trusted |
| Schema (`schema.rs`) | **TRUSTED** | Full implementation; 16 tests pass |
| robopoker git migration | **BLOCKER** | Owned by `games:traits` lane |
| `play:tui` binary skeleton | **BLOCKER** | Owned by `play:tui` lane; must coordinate on CLI flag contract |
| Slice 1 (`agent_context.rs`) | **READY TO BEGIN** | No external deps beyond `tui:shell` |
| Slice 2 (`journal.rs`) | **READY TO BEGIN** | No external deps beyond `tui:shell` |
| Slice 3 (`--context` wiring) | **BLOCKED** | Waiting on `play:tui` binary skeleton |
| Slice 4 (`reflect>` prompt) | **BLOCKED** | Waiting on Slice 3 |
| Slice 5 (`narration.rs`) | **BLOCKED** | Waiting on Slice 2 |
| Slice 6 (`--narrate` wiring) | **BLOCKED** | Waiting on Slice 5 |
| Slice 7 (lobby) | **BLOCKED** | Waiting on Slices 3+4+6 |
| Slices 8–9 (spectator) | **BLOCKED** | Waiting on `play:tui` binary + Slices 6+7 |

---

## What the Implementation Lane Must Preserve

### Preserve: Schema as the canonical contract
The `GameState` JSON schema is the foundation. Both pipe output and narration must be
consistent with it. Any divergence between `schema.rs` and the rendering modules must be
detected by tests, not by runtime observation.

### Preserve: Journal append-only invariant
`journal.rs` must never call `write()` or `truncate()`. Only `append` operations are allowed.
This is not just a convention — it is the journal's core promise to the agent consumer.

### Preserve: Fog-of-war at the relay
`SpectatorRelay` must strip hole cards before emission, not rely on the renderer to do it.
The relay is the enforcement point. This must be verified by a test that explicitly checks
hole cards are absent from pre-showdown events.

### Preserve: Context file isolation
`AgentContext::load()` must validate the JSON structure before acting on it. Malformed
context files must produce a clear error, not silently corrupt state.

### Preserve: Pipe output plain-text guarantee
`PipeMode::pipe_output()` must never produce ANSI escape sequences. This is the contract
with agent consumers. The existing `is_plain_text()` test in `tui:shell` must continue to pass.

---

## Risks the Implementation Lane Must Reduce

### Risk 1: robopoker git migration
Reduce by: coordinating with `games:traits` lane to ensure the migration lands before any
integration testing that exercises the full pipe mode. Track this as a dependency in the
implementation lane's slice plan.

### Risk 2: CLI flag contract drift
Reduce by: agreeing on the `--context` and `--narrate` flag shapes with `play:tui` lane
before Slice 3 begins. Write the `clap` argument definitions in `agent-adapter.md` as a
contract. If `play:tui` uses a different flag shape, the adapter must be updated.

### Risk 3: Journal file locking
Reduce by: adding `fsync` after each append in `journal.rs` as a low-cost mitigation.
This does not eliminate the race but makes it unlikely in practice.

### Risk 4: Narration engine testability
Reduce by: writing `narration.rs` tests against a fixed `GameState` fixture. The same
fixture should be usable in both terse and narrate modes to verify consistency (per
`narration::tests::terse_and_narrate_same_game_state` in the spec).

---

## Recommendation

**Proceed with Slices 1 and 2 immediately.** Both are fully unblocked — they depend only
on the trusted `tui:shell` upstream. `agent_context.rs` and `journal.rs` can be designed,
implemented, and tested in isolation.

**Coordinate on the CLI flag contract now.** Before Slice 3 is attempted, establish the
exact `--context <path>` and `--narrate` flag shapes with the `play:tui` lane. The flag
contract should be written into `agent-adapter.md` as an explicit agreement between the two
lanes. Do not wait for the `play:tui` binary skeleton to begin this conversation.

**Track robopoker migration actively.** The `games:traits` implementation lane owns this
resolution. This lane should check the status before any integration testing. If the
migration slips, Slices 1–2 can still be completed but Slices 3+ cannot be integration-tested.

**Slice 7 (lobby) should use hardcoded data from day one.** Do not build a stub mechanism
and then replace it — build the hardcoded lobby directly and document that it is hardcoded.
The Phase 4 upgrade path is clear and documented in `agent-adapter.md`.

---

## File Reference Index

| File | Role |
|------|------|
| `outputs/agent/experience/spec.md` | Lane specification (AX-01..05, SP-01..03) |
| `outputs/agent/experience/review.md` | Lane review (JUDGMENT: KEEP) |
| `outputs/agent-integration/agent-adapter.md` | This lane's integration adapter |
| `outputs/agent-integration/review.md` | This file |
| `fabro/programs/myosu-product.yaml` | Program manifest (unit: `agent`, lane: `experience`) |
| `fabro/run-configs/product/agent-experience.toml` | Run config for this lane |
| `crates/myosu-tui/src/agent_context.rs` | Slice 1 target (MISSING) |
| `crates/myosu-tui/src/journal.rs` | Slice 2 target (MISSING) |
| `crates/myosu-tui/src/narration.rs` | Slice 5 target (MISSING) |
| `crates/myosu-tui/src/pipe.rs` | Slices 3, 4, 6, 7 modification target |
| `crates/myosu-play/src/spectate.rs` | Slice 8 target (MISSING) |
| `crates/myosu-tui/src/screens/spectate.rs` | Slice 9 target (MISSING) |
| `crates/myosu-tui/src/schema.rs` | Trusted schema contract |

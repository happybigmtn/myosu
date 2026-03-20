# `agent-integration` Lane Review

## Judgment: **KEEP** — proceed to implementation-family workflow

The `agent:experience` lane is ready to move from specification into implementation. The source specs are sound, the lane boundaries are clear, and the critical upstream dependencies (`tui:shell`, `games:traits`) are already trusted with passing tests. The 9 implementation slices are sequential and minimally coupled.

---

## Source Artifacts Used

| Artifact | Location | Status |
|----------|----------|--------|
| Lane spec | `outputs/agent/experience/spec.md` | Reviewed — KEEP |
| Lane review | `outputs/agent/experience/review.md` | Reviewed — KEEP |
| Adapter contract | `outputs/agent-integration/agent-adapter.md` | This document's companion |
| Game state schema | `docs/api/game-state.json` | Trusted |
| Schema Rust impl | `crates/myosu-tui/src/schema.rs` | Trusted (16 tests) |

---

## Upstream Dependency Status

| Upstream | Status | Impact if Missing |
|----------|--------|------------------|
| `tui:shell` (82 tests) | **TRUSTED** | `GameRenderer`, `PipeMode` unavailable — nothing compiles |
| `games:traits` (14 tests) | **TRUSTED** | `CfrGame`, `Profile` unavailable — nothing compiles |
| `play:tui` binary | **KEEP, SLICES DEFINED** | `--pipe` flags have no CLI home; needed for Slices 3+ |
| `docs/api/game-state.json` | **TRUSTED** | JSON schema unavailable; only text pipe mode usable |
| `robopoker` (git dep) | **IN-PROGRESS** (owned by `games:traits`) | Absolute path deps block CI; must be resolved before Phase 1 integration testing |

**Verdict**: All hard upstream dependencies are either trusted or KEEP with defined slices. `agent:experience` is not blocked — it can begin with Slices 1–2 immediately.

---

## Decision: Implementation Family or Upstream Unblock?

**Decision: Implementation family next.**

Rationale:

1. **`games:traits` is KEEP with implementation lane unblocked.** The robopoker absolute-path migration (Slice 1 of `games:traits`) is tracked in `outputs/games/traits/review.md` as an in-progress risk, not a blocking gate for `agent:experience`. `agent:experience` Slices 1–4 do not call into `robopoker` directly — they depend only on `tui:shell` which already passes tests.

2. **`play:tui` is KEEP with ordered slices.** The `myosu-play` binary skeleton is Slice 1 of `play:tui`. It is needed for Slices 3+ of `agent:experience` (--context wiring, --narrate wiring, lobby). The `play:tui` review correctly identifies the slice ordering. The two lanes can run in parallel for Slices 1–2.

3. **No additional upstream spec is needed.** The `spectator-protocol` (AX-01..05, SP-01..03) is specification-only but the implementation is fully defined in the `agent:experience` spec's 9 slices. No separate upstream unblock is required.

4. **The `docs/api/game-state.json` precondition check passes.** The `game_state_schema_present` check in `myosu-product.yaml` resolves to true — `docs/api/game-state.json` exists and is trusted.

---

## What an Implementation-Family Workflow Must Preserve

### Preservable: `GameRenderer::pipe_output()` Contract

The `GameRenderer` trait in `crates/myosu-tui/src/renderer.rs` is the **only** integration contract between `agent:experience` and `tui:shell`. If it changes, `PipeMode` breaks.

**Preserve**: Treat `pipe_output()` as frozen for Phase 1. Any new required methods require a coordinated `tui:shell` + `agent:experience` migration.

### Preservable: `AgentContext` Serde Shape

Agents write scripts against the context JSON file format. Changing the field names or structure breaks existing agent scripts without a migration path.

**Preserve**: Add-only to `AgentContext`. Never rename or remove fields. New fields must be `Option<T>` with `#[serde(default)]`.

### Preservable: Journal Append-Only Invariant

The journal file is append-only. Any implementation that truncates, rewrites, or overwrites entries breaks the agent's autobiographical record.

**Preserve**: `journal.rs` must only open files with `OpenOptions::append(true)`. Never use `write()` without `append()`.

### Preservable: Fog-of-War at Relay

Hole cards must never appear in spectator relay output during active play. This is a security/privacy contract, not just a UX choice.

**Preserve**: Enforce fog-of-war in `SpectatorRelay::emit()`, not in the rendering screen. The relay is the last gate before data leaves the process.

### Preservable: No Blocking Sleep in Event Loop

`PipeMode` runs on an async event loop. Any blocking call freezes the TTY.

**Preserve**: Use `tokio::time::sleep` for bot thinking delays. Ensure bot tasks are spawned as separate async tasks.

### Preservable: No Absolute Path Dependencies in `myosu-play`

`myosu-play/Cargo.toml` must use only git or crates.io dependencies. Absolute paths break CI and prevent clean checkouts.

**Preserve**: When adding dependencies in Slices 3+, use `cargo add` (git or crates.io). Never edit `Cargo.toml` with `path = "/home/r/..."`.

---

## Slice Readiness Matrix

| Slice | Module | Ready to Start | Blocks |
|-------|--------|----------------|--------|
| Slice 1 | `agent_context.rs` | **YES** | Nothing beyond `tui:shell` |
| Slice 2 | `journal.rs` | **YES** | Nothing beyond `tui:shell` |
| Slice 3 | `--context` wiring | **YES** | Needs `myosu-play` binary (Slice 1 of `play:tui`) — parallelize |
| Slice 4 | `reflect>` prompt | **YES** | Needs Slice 3 context wiring |
| Slice 5 | `narration.rs` | **YES** | Needs Phase 1 context from Slices 1–2 |
| Slice 6 | `--narrate` wiring | **BLOCKED** | Needs Slice 5 + `play:tui` binary |
| Slice 7 | Lobby | **BLOCKED** | Needs `play:tui` binary + chain stub (Phase 0: hardcoded) |
| Slice 8 | `SpectatorRelay` | **BLOCKED** | Needs `play:tui` binary |
| Slice 9 | `SpectateScreen` | **BLOCKED** | Needs Slice 8 relay |

**Parallelization opportunity**: `play:tui` Slice 1 (binary skeleton) and `agent:experience` Slices 1–2 can proceed concurrently. Both depend only on trusted upstreams.

---

## Blockers That Are NOT Blocking This Lane

| Blocker | Owned By | Does NOT Block |
|---------|----------|----------------|
| robopoker git migration | `games:traits` | `agent:experience` Slices 1–4 (don't call robopoker) |
| `play:tui` binary skeleton | `play:tui` | `agent:experience` Slices 1–2 |
| Chain discovery stub | `chain:runtime` (Phase 4) | Lobby stub with hardcoded data (Slice 7 Phase 0) |
| Miner axon HTTP | `chain:runtime` (Phase 2) | Lobby Phase 0 with hardcoded data |

---

## Recommendation

**Spin up an implementation-family workflow for `agent:experience` now.** The lane is well-specified, the upstream is trusted, and the parallelization opportunity with `play:tui` is real. Begin with:

1. **`agent:experience` Slices 1–2** (agent_context.rs, journal.rs) — immediately, no external dependencies
2. **`play:tui` Slice 1** (myosu-play binary skeleton) — concurrently, unblocks `agent:experience` Slice 3
3. **`agent:experience` Slice 3** (--context wiring) — once `play:tui` binary exists
4. **`agent:experience` Slice 4** (reflect prompt) — after Slice 3

Track the robopoker git migration in `games:traits` independently. It must be resolved before Phase 2 integration testing (when `agent:experience` calls into `games:traits` through `play:tui`).

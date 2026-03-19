# `play:tui` Lane Review

## Judgment Summary

**Judgment: KEEP — Ready for Implementation-Family Workflow**

The `play:tui` lane is a **pure implementation lane** built on two already-trusted upstream lanes. The spec accurately reflects the current state: the binary, NLHE renderer, training mode, blueprint backend, and solver advisor are all absent, but the design is sound and the implementation slices are correctly ordered from smallest to full product.

The lane is **unblocked** for an implementation-family workflow immediately.

---

## Explicit Judgment by Surface

| Surface | Judgment | Rationale |
|---------|----------|-----------|
| `crates/myosu-tui/` (shell, 82 tests) | **KEEP** | Trusted upstream; all tests pass |
| `crates/myosu-games/` (traits, 14 tests) | **KEEP** | Trusted upstream; all tests pass |
| `crates/myosu-play/` binary scaffold | **READY TO BUILD** | No existing code; spec correctly defines required modules |
| `crates/myosu-games-poker/` NLHE renderer | **READY TO BUILD** | Spec TU-08 is complete and accurate |
| `TrainingTable` | **READY TO BUILD** | Spec TU-09 is complete and accurate |
| `BlueprintBackend` | **READY TO BUILD** | Spec TU-10 is complete and accurate |
| `SolverAdvisor` | **READY TO BUILD** | Spec TU-11 is complete and accurate |
| `chain:runtime` integration | **REOPEN LATER** | Blocked on `chain:runtime` lane; not needed for Phase B |

---

## Proof Expectations

### Required Proof Commands (in order)

```bash
# Proof 1: Binary skeleton builds
cargo build -p myosu-play

# Proof 2: NLHE renderer compiles and renders
cargo test -p myosu-games-poker

# Proof 3: Training mode — fold hand
cargo test -p myosu-play training::tests::hand_completes_fold

# Proof 4: Training mode — showdown
cargo test -p myosu-play training::tests::hand_completes_showdown

# Proof 5: Blueprint loading (mock artifact)
cargo test -p myosu-play blueprint::tests::load_valid_artifact

# Proof 6: Solver advisor formatting
cargo test -p myosu-play advisor::tests::format_distribution_text

# Proof 7: End-to-end — hero plays one complete hand
# (manual; run myosu-play --train and play one hand to showdown)
```

### Remaining Blockers

| Blocker | Severity | Status |
|---------|----------|--------|
| `myosu-play` crate does not exist | **Critical** | Must be created in Slice 1 |
| `myosu-games-poker` crate does not exist | **Critical** | Must be created in Slice 2 |
| `robopoker` absolute path deps (in `myosu-games`) | **Blocking CI** | `games:traits` lane's Slice 1; must resolve before `play:tui` calls robopoker APIs |
| Two `#[ignore]` TTY tests in `myosu-tui/events.rs` | **Low** | Pre-existing gap in `tui:shell`; does not block `play:tui` but means async event loop paths are CI-blind until headless mock is added |

### Proof Sufficiency

- **Trusted upstream**: `tui:shell` (82 tests) and `games:traits` (14 tests) provide a solid foundation — the `GameRenderer` trait contract is the integration point and is already proven object-safe.
- **Implementation gap**: The missing surfaces (`myosu-play`, `NlheRenderer`, `TrainingTable`, `BlueprintBackend`, `SolverAdvisor`) are all well-specified in the archive specs (TU-08 through TU-12). The spec defines exact data structures, API signatures, and expected test shapes.
- **No design ambiguity**: Every module that needs to exist has a precise description of its responsibility, inputs, outputs, and test requirements.

---

## Lane Readiness

**Is the lane ready for an implementation-family workflow next?**

**Yes.**

The upstream lanes (`tui:shell`, `games:traits`) are fully bootstrapped and trusted. The `play:tui` spec defines an unambiguous, ordered sequence of implementation slices. There are no design decisions pending — the architecture (shell + GameRenderer trait + TrainingTable + BlueprintBackend + SolverAdvisor) is settled.

The only blocker that could interrupt an implementation-family workflow is the robopoker absolute-path dependency in `myosu-games`. If that is not resolved before Slice 3 (when `TrainingTable` wraps robopoker's `Game`), the implementation will fail at compile time. This is tracked in the `games:traits` lane and must be resolved before `play:tui` reaches Phase B.

---

## Concrete Risks to Preserve

### Risk 1: Robopoker Absolute Path Coupling Spreads to `myosu-play`
**Location**: `crates/myosu-play/Cargo.toml` (will be created in Slice 1)

If `myosu-play` inherits the same absolute path dependencies as `myosu-games`, CI will break and contributors without the `robopoker` repo at `/home/r/coding/robopoker/` cannot build.

**Preserve**: `myosu-play` must use the resolved git dependencies once `games:traits` completes its Slice 1 migration. Do not add `path = "/home/r/coding/robopoker/..."` entries to `myosu-play/Cargo.toml`.

**Verify**: `cargo fetch` succeeds without local robopoker.

### Risk 2: `GameRenderer` Trait Changes Break `NlheRenderer`
**Location**: `crates/myosu-games-poker/src/renderer.rs`

The `GameRenderer` trait in `myosu-tui` is the **only** integration contract. If the trait changes (e.g., a new required method is added), `NlheRenderer` must be updated simultaneously.

**Preserve**: Treat `GameRenderer` as frozen for Phase B. Any trait changes require a coordinated `tui:shell` + `play:tui` migration.

### Risk 3: Blueprint Artifact Format Drift
**Location**: `crates/myosu-play/src/blueprint.rs`

The blueprint loading code assumes a specific artifact format (schema v1, SHA-256 hashes, mmap layout). If the miner training pipeline changes the artifact format, loaded blueprints will silently produce garbage strategies.

**Preserve**: Always validate `BlueprintManifest.schema_version` and both hashes before using an artifact. Never silently fall back — always surface an actionable error message.

### Risk 4: Bot Thinking Delay Blocks Event Loop
**Location**: `crates/myosu-play/src/training.rs`

The 200–500ms bot delay must be implemented as async sleep, not blocking sleep. Blocking sleep in the training loop will freeze the TUI event loop.

**Preserve**: Use `tokio::time::sleep` for the thinking delay. Ensure the bot task is spawned as a separate async task so the event loop can still process input.

---

## Implementation Slices Summary

| Slice | Module | Hard Dependency | Next Gate |
|-------|--------|-----------------|-----------|
| 1 | `myosu-play` binary skeleton | `tui:shell` | `cargo build -p myosu-play` |
| 2 | `NlheRenderer` with hardcoded states | Slice 1 | `cargo test -p myosu-games-poker` |
| 3 | `TrainingTable` + `HeuristicBackend` | Slice 2 + `games:traits` | `cargo test -p myosu-play training::` |
| 4 | `BlueprintBackend` | Slice 3 | `cargo test -p myosu-play blueprint::` |
| 5 | `SolverAdvisor` | Slice 4 | `cargo test -p myosu-play advisor::` |
| 6 | `Recorder` (hand history) | Slice 3 | `cargo test -p myosu-play recorder::` |
| 7 | Chain discovery + miner client | `chain:runtime` (future) | — |

---

## File Reference Index

| File | Role |
|------|------|
| `specsarchive/031626-05-gameplay-cli.md` | Binary + discovery + game loop + bot + recorder spec |
| `specsarchive/031626-07-tui-implementation.md` | TU-08 (NLHE renderer), TU-09 (training), TU-10 (blueprint), TU-11 (advisor) |
| `crates/myosu-tui/src/renderer.rs` | `GameRenderer` trait definition (trusted upstream) |
| `crates/myosu-games/src/traits.rs` | `games:traits` re-exports (trusted upstream) |
| `outputs/tui/shell/spec.md` | `tui:shell` lane spec (upstream) |
| `outputs/tui/shell/review.md` | `tui:shell` lane review (upstream) |
| `outputs/games/traits/spec.md` | `games:traits` lane spec (upstream) |
| `outputs/games/traits/review.md` | `games:traits` lane review (upstream) |
| `fabro/run-configs/product/play-tui.toml` | Fabro run config for this lane |
| `fabro/workflows/bootstrap/play-tui.fabro` | Fabro bootstrap workflow |
| `outputs/play/tui/spec.md` | This lane's spec artifact |
| `outputs/play/tui/review.md` | This file |

# Myosu Foundations — Foundation Plan

**Lane**: `foundations`
**Date**: 2026-03-20
**State**: Bootstrap run in progress

---

## What This Lane Is

`foundations` is the meta-bootstrap lane for the Myosu Fabro frontier. It does not own a single crate — it owns the honest state-of-the-frontier assessment that every other lane depends on. Its only job is to produce two artifacts:

- `outputs/foundations/foundation-plan.md` (this file): the current lane inventory and next-step priorities
- `outputs/foundations/review.md`: the trust judgment and defect register

---

## Current Lane Inventory

### `games:traits` — TRUSTED, IMPLEMENTATION UNBLOCKED

**Milestone reached**: `reviewed` (2026-03-19)

The `myosu-games` crate is a clean leaf. All 10 unit tests pass, all 4 doctests pass, and the git-dependency migration from absolute paths to `happybigmtn/robopoker` at `04716310143094ab41ec7172e6cea5a2a66744ef` is done. The crate owns the public re-exports of robopoker's CFR traits plus the `GameType`, `GameConfig`, `StrategyQuery`, `StrategyResponse` types.

**Next step**: Run the implementation lane. Slice 1 (git dep migration) is done. Slice 2 (add explicit crate name) and Slice 3 (edition pin resolution) are next. The lane can run any time.

**Proof**:
```bash
cargo test -p myosu-games   # 10 unit + 4 doctest, exit 0
```

---

### `tui:shell` — PARTIALLY REVIEWED, REOPEN ON 3 MODULES

**Milestone reached**: `specified` + partial `review.md` judgment

The `myosu-tui` crate compiles and its core modules (`screens.rs`, `input.rs`, `renderer.rs`, `theme.rs`) are production-quality. However, three modules have high-severity proof gaps:

| Module | Issue | Severity |
|--------|-------|----------|
| `schema.rs` | Claims 20-game coverage, only 3 have full roundtrip tests | HIGH |
| `events.rs` | Two async loop tests are `#[ignore]` due to TTY requirement; no headless alternative | HIGH |
| `shell.rs` | Integration chain (`handle_key → handle_submit → screens.apply_command`) is untested; 5 of 8 screens never rendered in tests | HIGH |

**Next step**: The TUI shell lane is blocked on `games:traits` for the schema types, but the core modules are solid. An honest implementation lane would address the proof gaps before declaring the TUI a trusted surface.

**Proof**:
```bash
cargo test -p myosu-tui   # passes but 2 tests are #[ignore]
```

---

### `chain:runtime` — RESET, BLOCKED ON RESTART

**Milestone reached**: `runtime_reviewed` judgment produced

The current `crates/myosu-chain/runtime/` is a non-building transplant. The workspace root explicitly comments it out (`# "crates/myosu-chain" # Stage 1: Substrate chain fork`). The `runtime/src/lib.rs` imports dependency crates that do not exist in the workspace (`subtensor-runtime-common`, `subtensor-macros`, etc.). The node binary (`node/src/main.rs`) calls unimplemented modules. The WASM build path is entirely absent.

**Salvageable inputs**: `NetUid` type, `Currency` domain types, `polkadot-sdk stable2407` git ref, the pallet directory structure, `AxonInfo`/`PrometheusInfo`/`NeuronCertificate` data types.

**What is not salvageable**: The entire `runtime/src/lib.rs` construct block, all `subtensor_*` imports, the node module declarations, the WASM builder configuration.

**Next step**: Restart from Phase 0. Begin with workspace wiring, establish a minimal Substrate runtime (`frame_system + pallet_balances + pallet_sudo + pallet_timestamp`), prove it builds to WASM, then layer in the node binary.

**Proof** (after restart):
```bash
cargo build -p myosu-runtime   # must exit 0
```

---

### `chain:pallet` — RESET, BLOCKED ON RESTART

**Milestone reached**: `pallet_reviewed` judgment produced

`cargo check -p pallet-game-solver` produces 50+ unique errors across three waves: (1) missing crates, (2) broken API shapes from `sp_runtime` removals in `stable2407`, (3) type mismatches from the subtensor workspace-key dependencies embedded at every level. The pallet's 36 migration files, 7 RPC info files, swap files, and coinbase files all carry dead `subtensor_*` imports.

**Salvageable inputs**: `stubs.rs`, `swap_stub.rs`, `benchmarks.rs`, the confirmed `polkadot-sdk stable2407` git ref, `AxonInfo`/`PrometheusInfo`/`NeuronCertificate` data types, `RateLimitKey` enum, the pallet module directory structure.

**What is not salvageable**: Everything that imports a `subtensor_*` workspace key, `safe_math` in `epoch/math.rs`, `extensions/subtensor.rs`, all migration/rpc/swap/coinbase modules.

**Next step**: Phase 1 restart is purely mechanical: add missing deps + delete broken modules + fix imports. The `chain:runtime` lane must reach `runtime_reviewed` before `chain:pallet` can safely restart.

**Proof** (after restart):
```bash
cargo check -p pallet-game-solver   # must exit 0
```

---

### `games:poker-engine` — SPEC ONLY, BLOCKED ON GREENFIELD

**Milestone reached**: `specified` + `review.md` judgment

`crates/myosu-games-poker/` does not exist. The lane spec defines 16 required tests across 6 slices, but no code exists yet. The `games:traits` lane is a prerequisite and is now implementation-ready.

**Next step**: Begin implementation lane with Slice 1 (crate skeleton). The lane is unblocked once `games:traits` is confirmed stable (it is).

**Proof** (after Slice 1):
```bash
cargo build -p myosu-games-poker   # must exit 0
```

---

### `games:multi-game` — SPEC ONLY, BLOCKED ON GREENFIELD + FABRO DEFECT

**Milestone reached**: `specified` + `review.md` judgment

`crates/myosu-games-liars-dice/` does not exist. The `ExploitMetric` type does not exist in `myosu-games`. The spectator relay and spectator TUI screen are unimplemented.

**Critical additional block**: The `games:multi-game` lane was previously dispatched by Raspberry and returned a false-submit — the submit path claimed success without actually producing the required artifacts. This indicates a defect in the Fabro/Raspberry detach path for this lane. The lane cannot be honestly re-run until the detach path is repaired.

**Next step**:
1. Repair the Fabro/Raspberry false-submit defect (see Frontier Tasks below)
2. Run the `games:multi-game` lane to produce honest artifacts
3. Begin Slice 1 (crate skeleton)

**Proof** (after repair and re-run):
```bash
test -f outputs/games/multi-game/spec.md   # must exit 0
test -f outputs/games/multi-game/review.md  # must exit 0
```

---

## The Two Explicit Frontier Tasks

### Task 1: Fix Raspberry/Fabro Defects Only When Discovered by Real Execution

**What it means**: The `execute/status/watch` command path in Fabro/Raspberry must produce trustworthy truth. When a lane is dispatched and claims to have produced artifacts, that claim must be verifiable — not guessed from a run directory scan.

**Current state**: The `games:multi-game` false-submit is the first discovered instance. The lane's `review.md` exists (produced by a previous run) but the artifacts the lane claims to have produced are not actually present at the expected paths.

**Required action**: Identify whether the false-submit is a Raspberry dispatch bug (claiming success without running), a Fabro detach bug (the work didn't make it back to the worktree), or a worktree sync bug (the worktree is stale relative to the Fabro run). Fix the specific bug, then re-run the affected lane.

**The rule**: Do not speculatively fix Fabro/Raspberry internals. Wait for real execution to surface a defect, then fix that specific defect.

### Task 2: Convert `games:multi-game` False-Submit into Truthful Failure or Successful Live Run

**What it means**: The `games:multi-game` lane was dispatched and returned success but did not produce its artifacts. The lane must be re-run and either (a) fail honestly with a clear error or (b) succeed and produce the real artifacts.

**Current state**: `outputs/games/multi-game/spec.md` and `review.md` exist — but these may have been produced by a bootstrap agent running without the Fabro workflow (the `foundations.fabro` workflow was missing from `fabro/workflows/bootstrap/` at session start). The `games:multi-game` lane's own `multi-game.fabro` workflow exists but has not produced honest artifacts via the proper Fabro run path.

**Required action**:
1. Confirm whether `outputs/games/multi-game/spec.md` and `review.md` are honest products of the `multi-game.fabro` workflow or bootstrap-generated placeholders
2. Repair the `foundations.fabro` missing-workflow defect (already done in this run)
3. Re-run `games:multi-game` via the proper Fabro path to produce verifiable artifacts

---

## Priority Order for Next Execution

Given the blocking relationships and trust levels:

| Priority | Lane | Why |
|----------|------|-----|
| 1 | `games:traits` (impl) | Unconditionally unblocked. Run any time. |
| 2 | `games:poker-engine` (impl) | Blocked only on `games:traits` (done). Run any time. |
| 3 | `chain:runtime` (restart) | Independent. Run in parallel with poker-engine. |
| 4 | `chain:pallet` (restart) | Blocked on `chain:runtime`. Run after runtime restarts. |
| 5 | `games:multi-game` (bootstrap) | Blocked on Fabro detach repair. Fix the false-submit first. |
| 6 | `tui:shell` (impl) | Blocked on `games:traits` (for schema types). Run after traits impl. |

---

## What Can Run Now

- `games:traits` implementation lane — immediately unblocked
- `games:poker-engine` implementation lane — immediately unblocked
- `chain:runtime` restart lane — immediately unblocked (independent)

## What Must Wait

- `chain:pallet` restart — must wait for `chain:runtime` to reach `runtime_reviewed`
- `games:multi-game` bootstrap — must wait for Fabro detach repair + false-submit diagnosis
- `tui:shell` implementation — must wait for `games:traits` implementation to add schema types

---

## Critical Defect Register

| # | Defect | Location | Severity | Status |
|---|--------|----------|----------|--------|
| 1 | `foundations.fabro` workflow missing from `fabro/workflows/bootstrap/` | `fabro/workflows/bootstrap/foundations.fabro` | CRITICAL | **FIXED in this run** |
| 2 | `games:multi-game` false-submit: lane claims artifacts not present | Fabro/Raspberry detach path | CRITICAL | OPEN |
| 3 | `chain:runtime` non-building transplant | `crates/myosu-chain/runtime/` | HIGH | OPEN — restart required |
| 4 | `chain:pallet` 50+ compilation errors | `crates/myosu-chain/pallets/game-solver/` | HIGH | OPEN — restart required |
| 5 | `myosu-games-poker` greenfield crate | `crates/myosu-games-poker/` | HIGH | OPEN — implementation required |
| 6 | `myosu-games-liars-dice` greenfield crate | `crates/myosu-games-liars-dice/` | HIGH | OPEN — implementation required |
| 7 | TTY dependency blocking `events.rs` CI proof | `crates/myosu-tui/src/events.rs` | MEDIUM | OPEN — needs headless mock |
| 8 | 17 of 20 game types untested in `schema.rs` | `crates/myosu-tui/src/schema.rs` | MEDIUM | OPEN — needs roundtrip tests |

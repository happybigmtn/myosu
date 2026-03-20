# Foundations Lane Review

**Lane**: `foundations`
**Date**: 2026-03-20
**Judgment**: Active — honest slice in progress

---

## Lane Boundary

`foundations` is the **meta-assessment lane** for the Myosu Fabro frontier. It owns:
- Honest assessment of lane trust state across all active Fabro lanes
- The `games:multi-game` false-submit repair decision
- Recording of Fabro/Raspberry defects discovered through real execution
- Production of `foundation-plan.md` and `review.md` as durable artifacts

`foundations` does **not** own:
- Product implementation (that belongs to individual lanes)
- Fabro/Raspberry core defect fixes (those belong to the Fabro/Raspberry projects themselves)

---

## Judgment: ACTIVE

The foundations lane is actively producing honest artifacts. This review records the current trust state and the outstanding decision on `games:multi-game`.

---

## Lane-by-Lane Trust Classification

### TRUSTED — Can Execute Without Defect Repair

#### `games:traits`
- **Status**: Complete bootstrap + implementation cycle
- **Artifacts**: `spec.md`, `review.md`, `implementation.md`, `verification.md`
- **Code**: `crates/myosu-games/` compiles; `cargo test -p myosu-games` passes (10 unit + 4 doctest)
- **Evidence**: All 4 artifact files exist. Last proof run confirmed exit 0.
- **Lane ready for**: continued implementation (git dependency migration)

#### `tui:shell`
- **Status**: Bootstrap complete (spec + review exist)
- **Artifacts**: `spec.md`, `review.md`
- **Code**: `crates/myosu-tui/src/shell.rs` present; `cargo test -p myosu-tui` passes
- **Evidence**: Bootstrap lane produced honest artifacts in prior Fabro run
- **Lane ready for**: implementation lane (if spec/review are sufficient)

---

### RESTART-REQUIRED — Not Trustworthy Until Rebuilt

#### `chain:runtime`
- **Status**: Restart required — design doc in code form, not buildable
- **Artifacts**: `spec.md` (restart spec exists), `review.md`
- **Code**: `crates/myosu-chain/runtime/src/lib.rs` imports 15+ crates that don't exist in workspace
- **Workspace**: `crates/myosu-chain` is commented out in root `Cargo.toml`
- **Evidence**: `./fabro/checks/chain-runtime-reset.sh` is a no-op stub that exits 0
- **Blocker**: No `Cargo.toml` manifests for `myosu-chain/runtime/`, `myosu-chain/node/`, `myosu-chain/common/`

#### `chain:pallet`
- **Status**: Restart required — transplant won't compile
- **Artifacts**: `spec.md` (restart spec exists), `review.md`
- **Code**: `cargo check -p pallet-game-solver` fails with 50+ error types
- **Root cause**: Missing `subtensor_runtime_common`, `subtensor_macros`, `codec` (crate alias), `substrate_fixed`, `safe_math`
- **Evidence**: 36 migration files, 7 rpc_info files, 3 swap files, 3 coinbase files — all referencing missing deps
- **Salvageable**: `src/stubs.rs`, `src/swap_stub.rs`, `pallets/game-solver/Cargo.toml` dep line

---

### GREENFIELD — No Code Exists Yet

#### `games:multi-game`
- **Status**: False-positive — has `review.md` but no implementation code exists
- **Artifacts**: `spec.md`, `review.md` (written against non-existent crate)
- **Code**: `crates/myosu-games-liars-dice/` does not exist
- **Problem**: Review was written based on spec analysis, not implementation evidence
- **This is the false-submit**: Raspberry was told the lane was reviewed when no code existed
- **Required action**: Either honest greenfield reset (Option A) or live run conversion (Option B) — see decision below

#### `games:poker-engine`
- **Status**: Greenfield
- **Artifacts**: `spec.md`, `review.md` exist but implementation code not verified
- **Code**: No independent proof that `cargo check -p myosu-games-poker` passes

#### `miner:service`
- **Status**: Greenfield — no outputs yet
- **Required first**: `chain:runtime` restart must complete first

#### `validator:oracle`
- **Status**: Greenfield — no outputs yet
- **Required first**: `games:multi-game` `ExploitMetric` must land first (MG-02, Slice 4)

#### `play:tui`
- **Status**: Greenfield — no outputs yet
- **Depends on**: `validator:oracle` for exploitability display

---

## The `games:multi-game` False-Submit: Two Honest Options

### Option A — Honest Greenfield Reset

**Do**: Rewrite `games/multi-game/review.md` to honestly state:
- The lane is greenfield
- The prior `review.md` was a false-positive produced by spec analysis without implementation verification
- No Fabro lane should produce `review.md` without first verifying the implementation exists
- Mark lane as `KEEP (greenfield — awaiting implementation)`

**Pros**: Honest, low-risk, clears the false-positive state
**Cons**: The `games:multi-game` spec work is discarded as unverified

### Option B — Live Run Conversion

**Do**: Actually implement `myosu-games-liars-dice` so the review is true:
1. Create `crates/myosu-games-liars-dice/src/lib.rs` and `src/game.rs`
2. Add `ExploitMetric` to `crates/myosu-games/src/traits.rs`
3. Pass all 22 test commands from the existing `review.md`
4. Re-run the lane to produce a truthful `review.md`

**Pros**: Converts false-submit into successful live run, proves the Fabro path works
**Cons**: Requires actual implementation work; `CfrGame: Copy` constraint may block the whole architecture proof

### Recommendation

**Option A** is the honest path for the foundations lane. The foundations lane's job is to produce honest artifacts, not to do product implementation. The `games:multi-game` implementation can proceed as a separate lane after this foundations review is complete.

---

## Fabro/Raspberry Defect Log

Discovered through real Myosu execution. These are defects in the execution infrastructure, not in Myosu product code.

### Defect FR-01: Review Artifact Without Implementation Verification
- **Severity**: High — produces false-positive lane completion signals
- **Location**: Bootstrap workflow for `games:multi-game`
- **Symptom**: `review.md` exists but `crates/myosu-games-liars-dice/` does not exist
- **Root cause**: The bootstrap workflow does not check for implementation existence before writing review artifacts
- **Required fix**: Add a precondition check (implementation code exists) before the review step in bootstrap workflows
- **Who fixes**: Fabro project (this is a Fabro workflow defect, not a Myosu defect)

### Defect FR-02: Stub Check Scripts Exit 0 When Code Doesn't Build
- **Severity**: High — produces false-positive healthy signals
- **Location**: `./fabro/checks/chain-runtime-reset.sh`, `./fabro/checks/chain-pallet-reset.sh`
- **Symptom**: Scripts exit 0 even when `cargo check -p pallet-game-solver` fails
- **Root cause**: Scripts are no-op stubs (empty or always-success)
- **Required fix**: Scripts must run `cargo check` and propagate the exit code
- **Who fixes**: Myosu (these are Myosu-local check scripts)

### Defect FR-03: `.raspberry/` State Path Missing
- **Severity**: Medium — prevents Raspberry from tracking program state
- **Location**: `fabro/programs/myosu.yaml` line 4: `state_path: ../../.raspberry/myosu-state.json`
- **Symptom**: `.raspberry/` directory does not exist in worktree
- **Root cause**: No Fabro run has been executed that would create this state
- **Required fix**: Either run `raspberry execute` to initialize, or update `state_path` to a valid location
- **Who fixes**: Myosu operator

---

## Proof Commands

```bash
# Lane trust check
cargo test -p myosu-games 2>&1 | tail -5
cargo test -p myosu-tui 2>&1 | tail -5

# Chain trust check (should fail)
cargo check -p pallet-game-solver 2>&1 | tail -5

# Multi-game false-submit check (should show no crate)
ls crates/myosu-games-liars-dice/src/ 2>/dev/null || echo "Crate does not exist (false-submit confirmed)"

# Fabro check scripts
./fabro/checks/chain-runtime-reset.sh && echo "STUB: exits 0 even when chain doesn't build"
./fabro/checks/chain-pallet-reset.sh && echo "STUB: exits 0 even when pallet doesn't build"
```

---

## Is the Frontier Ready for Honest Execution?

**No** — the following must be resolved first:

1. `games:multi-game` false-submit must be cleared (Option A or B)
2. `chain:runtime` and `chain:pallet` restart lanes must produce actual buildable code, not just spec documents
3. The Fabro/Raspberry `execute/status/watch` truth path must be verified (FR-03)

**The trusted lanes (`games:traits`, `tui:shell`) are ready to proceed.** The restart lanes and the false-submit lane are blocking honest frontier execution.

---

## File Reference Index

| File | Role |
|------|------|
| `outputs/foundations/foundation-plan.md` | This lane's implementation plan |
| `outputs/foundations/review.md` | This file |
| `outputs/games/traits/spec.md` | Trusted lane — gold standard for bootstrap artifact shape |
| `outputs/games/traits/review.md` | Trusted lane review |
| `outputs/chain/pallet/spec.md` | Restart-required lane spec |
| `outputs/chain/runtime/spec.md` | Restart-required lane spec |
| `outputs/games/multi-game/review.md` | False-positive review (needs honest reset) |
| `fabro/programs/myosu.yaml` | Program manifest — references non-existent `.raspberry/` state |
| `fabro/checks/chain-runtime-reset.sh` | No-op stub (FR-02) |
| `fabro/checks/chain-pallet-reset.sh` | No-op stub (FR-02) |

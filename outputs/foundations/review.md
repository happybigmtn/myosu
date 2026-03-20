# Foundations Lane Review

**Lane**: `foundations`
**Date**: 2026-03-20

---

## Keep / Reopen / Reset Judgment

**Judgment: KEEP**

The `foundations` lane has produced honest artifacts that accurately characterize the bootstrap frontier. The key findings are:

1. **One lane is falsely submitted** (`games:multi-game` — spec reviewed, implementation absent)
2. **One lane is partially trusted** (`tui:shell` — 4 modules KEEP, 3 modules REOPEN)
3. **Two lanes require restart** (`chain:runtime`, `chain:pallet` — not buildable)
4. **One lane is fully trusted** (`games:traits` — implementation lane unblocked)

This assessment is consistent with the evidence in each lane's `review.md`.

---

## Is the Foundations Lane Complete?

**Yes — for the bootstrap slice.**

The `foundations` lane has produced the two required artifacts:

- `outputs/foundations/foundation-plan.md` — honest frontier status
- `outputs/foundations/review.md` — this file

Both artifacts are internally consistent and reference the same lane judgments.

---

## Frontend Execution Posture

After this review, the Raspberry control plane should hold the following posture for each bootstrap lane:

| Lane | Status | Control Plane Posture |
|------|--------|----------------------|
| `games:traits` | **TRUSTED** | Ready for implementation lane extension |
| `tui:shell` | **PARTIAL** | 4 modules usable; `schema`, `events`, `shell` treated as REOPEN until CI-compatible proof exists |
| `chain:runtime` | **RESTART** | No execution; restart lane must begin from workspace wiring |
| `chain:pallet` | **RESTART** | No execution; restart lane must begin from Phase 1 |
| `games:multi-game` | **NOT STARTED** | `review.md` is a spec-level judgment only; implementation lane has not run |
| `foundations` | **BOOTSTRAPPED** | Artifacts produced; lane complete for this slice |

---

## Concrete Risks the Implementation Lanes Must Preserve or Reduce

### Risk 1: `games:multi-game` False Submit Misleads Control Plane

**Location**: `outputs/games/multi-game/review.md` says "Judgment: KEEP" and "Is the Lane Ready for an Implementation-Family Workflow Next? Yes" despite `crates/myosu-games-liars-dice/` not existing.

**What must be preserved**: The spec-level judgment is correct — the spec is coherent and the `GameType::LiarsDice` hook exists in `myosu-games`. The architectural case is sound.

**What must be reduced**: The review must be updated to clarify that "KEEP" means "spec is coherent" not "implementation is present and proven." The implementation lane must produce the crate and pass proof commands before the lane is marked as implementation-ready.

**Verification**: `cargo build -p myosu-games-liars-dice && cargo test -p myosu-games-liars-dice` exits 0.

---

### Risk 2: `tui:shell` Proof Gaps Become Hidden Debt

**Location**: `crates/myosu-tui/src/schema.rs` (17 of 20 game types untested), `events.rs` (2 `#[ignore]` tests), `shell.rs` (no integration chain test).

**What must be preserved**: The 4 modules that are production-quality (`screens.rs`, `input.rs`, `renderer.rs`, `theme.rs`) must not regress.

**What must be reduced**: `schema.rs` must have roundtrip tests for all 20 `LegalAction` variants. `events.rs` must have a headless alternative to the TTY-dependent tests. `shell.rs` must have an integration test covering the input→screen→draw chain.

**Verification**: All `#[ignore]` event tests pass in CI. All 20 schema game types have roundtrip proof. Shell integration test exists and passes.

---

### Risk 3: `chain:runtime` Cannot Build — Blocks All Downstream Lanes

**Location**: Root `Cargo.toml` line comments out `crates/myosu-chain`. `runtime/src/lib.rs` imports non-existent workspace keys.

**What must be preserved**: The `polkadot-sdk stable2407` git dep confirmed working in `pallet-game-solver/Cargo.toml`. The `NetUid(u16)` domain type in `runtime/src/lib.rs` is clean and should be preserved in restart.

**What must be reduced**: All `subtensor_*` workspace key imports. All unimplemented node modules. The absent WASM build path.

**Verification**: `cargo build -p myosu-runtime` exits 0.

---

### Risk 4: `chain:pallet` Has 50+ Compile Errors — Cannot Be Extended

**Location**: `pallet-game-solver/src/lib.rs` and 36 migration files.

**What must be preserved**: `stubs.rs`, `swap_stub.rs`, `benchmarks.rs`, `AxonInfo`, `PrometheusInfo`, `NeuronCertificate`, `RateLimitKey`, pallet directory structure, and `polkadot-sdk stable2407` dep.

**What must be reduced**: All `subtensor_*` imports. `extensions/subtensor.rs` (removed APIs). All 36 migration files. `safe_math` dependency in `epoch/math.rs`. The 80+ storage items encoding Alpha/TAO state.

**Verification**: `cargo check -p pallet-game-solver` exits 0.

---

### Risk 5: `execute/status/watch` Truth Is Not Yet Trustworthy

**Location**: Raspberry command rendering and Fabro run-truth bridge.

**What must be preserved**: The bootstrap program manifest structure (`fabro/programs/myosu-bootstrap.yaml`) and the curated output roots.

**What must be reduced**: Any gap between what `raspberry status` reports and what `fabro inspect` actually shows. The `games:multi-game` lane's false submit is a symptom of this — the control plane accepted a `review.md` artifact without verifying the implementation was present.

**Verification**: `raspberry status --manifest fabro/programs/myosu.yaml` accurately reflects the state of each lane based on actual proof commands, not just artifact presence.

---

## Proof Commands

| Lane | Bootstrap Gate Command | Expected |
|------|----------------------|----------|
| `games:traits` | `cargo test -p myosu-games` | Exit 0, 10 unit + 4 doctest pass |
| `tui:shell` | `./fabro/checks/tui-shell.sh` | Exit 0 (precondition only; full proof requires CI) |
| `chain:runtime` | `./fabro/checks/chain-runtime-reset.sh` | Exit 0 (surface check only; restart lane must add build proof) |
| `chain:pallet` | `./fabro/checks/chain-pallet-reset.sh` | Exit 0 (surface check only; restart lane must add compile proof) |
| `games:multi-game` | `cargo build -p myosu-games-liars-dice` | Exit 0 (currently fails — implementation not started) |

**Note**: The current `chain-runtime-reset.sh` and `chain-pallet-reset.sh` scripts are surface checks only (they exit 0 if files exist). They do not prove the code compiles. The restart lanes must add real compile/build proof commands.

---

## File Reference Index

| File | Role |
|------|------|
| `outputs/foundations/spec.md` | This lane's spec artifact |
| `outputs/foundations/review.md` | This file |
| `outputs/games/traits/spec.md` | Trusted lane spec |
| `outputs/games/traits/review.md` | Trusted lane review |
| `outputs/tui/shell/review.md` | Partial trust review (3 modules REOPEN) |
| `outputs/chain/runtime/review.md` | Restart review (workspace broken) |
| `outputs/chain/pallet/review.md` | Restart review (50+ errors) |
| `outputs/games/multi-game/review.md` | Spec-reviewed but implementation not started |
| `fabro/programs/myosu-bootstrap.yaml` | Bootstrap program manifest |
| `fabro/programs/myosu.yaml` | Full program manifest (frontier units) |

---

## Relationship to Frontier Tasks

The two active frontier tasks map directly to the risks above:

| Frontier Task | Primary Risk | Mitigation |
|--------------|-------------|------------|
| Fix Raspberry/Fabro defects when discovered by real Myosu execution | Risk 5 (`execute/status/watch` truth) | Slice 5: Stabilize `execute/status/watch` truth |
| Convert `games:multi-game` false-submit into truthful failure or live run | Risk 1 (false submit) | Slice 1: Rerun `games:multi-game` with truthful execution |

Both frontier tasks are honest next actions. The `foundations` lane has confirmed they are real defects, not phantom ones.

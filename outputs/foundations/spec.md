# Foundations Lane Spec

**Lane**: `foundations`
**Date**: 2026-03-20
**Type**: Meta-bootstrap / Frontier Assessment

## Purpose / User-Visible Outcome

The `foundations` lane produces the first honest reviewed slice of the Myosu bootstrap frontier. It is the lane that audits all other bootstrap lanes, records which surfaces are trustworthy, which need restart, and which need reopening — and sets the execution posture for the Fabro/Raspberry control plane.

After this lane completes, a contributor can open `outputs/foundations/review.md` and know exactly which bootstrap lanes are safe to extend versus which require repair before they can produce trustworthy artifacts.

## Current Trusted Inputs

### Bootstrap Program
- `fabro/programs/myosu-bootstrap.yaml` — defines 3 units (`games`, `tui`, `chain`) with 4 lanes (`traits`, `shell`, `runtime`, `pallet`)

### Execution Plane Surfaces
- `fabro/workflows/bootstrap/*.fabro` — workflow graphs for each bootstrap lane
- `fabro/run-configs/bootstrap/*.toml` — run configs for each bootstrap lane
- `fabro/prompts/bootstrap/*.md` — plan/implement/review prompt trio
- `fabro/checks/*.sh` — proof helper scripts

### Curated Output Roots (existing reviewed artifacts)
| Lane | `spec.md` | `review.md` | Judgment |
|------|-----------|-------------|----------|
| `games:traits` | ✅ exists | ✅ exists | **KEEP** — trusted leaf crate |
| `tui:shell` | ✅ exists | ✅ exists | **PARTIAL** — 4 modules KEEP, 3 REOPEN |
| `chain:runtime` | ✅ exists | ✅ exists | **RESTART** — workspace broken |
| `chain:pallet` | ✅ exists | ✅ exists | **RESTART** — 50+ compile errors |
| `games:multi-game` | ✅ exists | ✅ exists | **KEEP (spec)** — implementation not started |

## Current Broken or Missing Surfaces

### 1. `games:multi-game` — False Submit (Critical)

**The claim**: `games:multi-game/review.md` says "Judgment: KEEP" and the lane is "ready for implementation" with a full test inventory.

**The reality**: `crates/myosu-games-liars-dice/` does not exist. The review is a spec-level judgment only — it assessed that the spec is coherent, not that the implementation is present. No code has been written.

**What this means for the control plane**: The `games:multi-game` lane produced a `review.md` artifact that implies the lane is further along than it is. This is the "false submit" referenced in the frontier task. The lane must be reclassified as **not started** until the implementation lane produces real proof.

**Required action**: The `games:multi-game` lane must be rerun with a truthful execution that either (a) produces the `myosu-games-liars-dice` crate and passes the Slice 1 proof commands, or (b) produces a truthful failure record if the implementation is blocked.

### 2. `tui:shell` — Three Modules Need Reopened Proof

**The claim**: The TUI is a trusted leaf crate.

**The reality**: Three modules have high-severity proof gaps:
- `schema.rs` — only 3 of 20 game types have roundtrip tests; 17 are unscrupulously claimed
- `events.rs` — 2 async tests are `#[ignore]` due to TTY requirement; no headless alternative
- `shell.rs` — integration chain (input → screen → draw) is never exercised end-to-end

**Required action**: The `tui:shell` lane cannot be marked as fully bootstrapped until `schema`, `events`, and `shell` have CI-compatible proof. Until then, those modules should be treated as **reopened** surfaces.

### 3. `chain:runtime` — Workspace Does Not Build

**The reality**:
- `crates/myosu-chain` is commented out in root `Cargo.toml`
- `runtime/src/lib.rs` references workspace keys that don't exist (`subtensor-runtime-common`, etc.)
- Node binary is scaffold with no `.rs` files
- WASM build path is entirely absent

**Required action**: Full RESTART from Phase 0. The restart must begin with workspace wiring, not with pallet logic.

### 4. `chain:pallet` — 50+ Compile Errors

**The reality**:
- `pallet-game-solver` fails `cargo check` with 50+ errors
- Imports reference `subtensor_*` workspace keys that don't exist
- `epoch/math.rs` depends on missing `safe_math` crate
- `extensions/subtensor.rs` uses polkadot-sdk APIs removed in stable2407

**Required action**: RESTART from Phase 1. Salvage: `stubs.rs`, `swap_stub.rs`, `AxonInfo`, `PrometheusInfo`, `NeuronCertificate`, `RateLimitKey`, pallet module structure, and the confirmed `polkadot-sdk stable2407` git dep.

## Lane Boundary

The `foundations` lane owns the **frontier-level** assessment view — the curated artifact that tells the Raspberry control plane and human contributors the honest status of every bootstrap lane. It does not own the repair work for individual lanes; that belongs to the implementation lanes or restart lanes for each affected surface.

## Deliverables

| Artifact | Path | Purpose |
|----------|------|---------|
| `foundation-plan.md` | `outputs/foundations/foundation-plan.md` | This file — honest bootstrap frontier status |
| `review.md` | `outputs/foundations/review.md` | Keep/reopen/restart judgment + next actions |

## Proof / Check Shape

The `foundations` lane is bootstrapped when both artifacts exist and are internally consistent:

```bash
# Both artifacts must exist
test -f outputs/foundations/foundation-plan.md
test -f outputs/foundations/review.md

# Both must reference the same lanes with the same judgments
# (checked manually or via rubric)
```

## Next Implementation Slices

### Slice 1 — Fix `games:multi-game` False Submit (Highest Priority)
Rerun the `games:multi-game` lane with a truthful Fabro execution that either produces the `myosu-games-liars-dice` crate or records a honest failure reason.

**Proof**: `cargo build -p myosu-games-liars-dice && cargo test -p myosu-games-liars-dice`

### Slice 2 — Repair `tui:shell` Proof Gaps
Add headless alternatives for `events.rs` TTY tests. Add roundtrip tests for all 20 game schema types in `schema.rs`. Add integration test for the `shell.rs` input→screen→draw chain.

**Proof**: All `#[ignore]` events tests pass in CI. All 20 game schema types have roundtrip proof. Shell integration test passes.

### Slice 3 — Restart `chain:runtime`
Begin Phase 0: wire the workspace, establish minimal Substrate runtime (`frame_system + pallet_balances + pallet_sudo + pallet_timestamp`), prove it builds to WASM.

**Proof**: `cargo build -p myosu-runtime` exits 0.

### Slice 4 — Restart `chain:pallet`
Begin Phase 1: add missing deps, delete broken modules, fix imports. Preserve: `stubs.rs`, `swap_stub.rs`, `benchmarks.rs`, data types, and pallet structure.

**Proof**: `cargo check -p pallet-game-solver` exits 0.

### Slice 5 — Stabilize `execute/status/watch` Truth
Fix Raspberry/Fabro defects only when they are discovered by real Myosu execution. After each fix, rerun the affected frontier until `execute/status/watch` truth is trustworthy.

**Proof**: `raspberry status --manifest fabro/programs/myosu.yaml` produces accurate lane health signals.

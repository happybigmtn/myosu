# 003 - Chain Runtime Reduction

## Purpose / Big Picture

The chain crate is a 242K-line Bittensor subtensor fork carrying 10 pallets and
EVM/Frontier integration unused in stage-0. This plan reduces the runtime to
the stage-0 surface: pallet-game-solver, minimal registration, staking, and
emission mechanics. Consolidates prior plans 003, 004, and 005.

## Context and Orientation

Prior plans produced partial results:
- SwapInterface no-op stub exists
- CRV3 timelock stripped
- `pallet-game-solver` renamed but carries 69 extrinsics, 95+ errors, 50+ migrations

The runtime at `crates/myosu-chain/runtime/src/lib.rs` still wires the full
subtensor surface. The node includes EVM/Frontier service code.

## Architecture

Target runtime pallet set:
```
System, Timestamp, Balances, TransactionPayment,
GameSolver (index 7), AdminUtils, Utility
```

Remove or feature-gate:
```
pallet-drand, pallet-crowdloan, pallet-proxy, pallet-registry,
pallet-swap, pallet-swap-interface, pallet-transaction-fee,
EVM/Frontier integration
```

## Progress

- [x] (pre-satisfied) M1. SwapInterface no-op stub
  - Surfaces: `crates/myosu-chain/pallets/swap/`
Proof command: `cargo check -p myosu-chain-runtime --features fast-runtime`

### Milestone 2: Strip unused pallets from runtime

- [ ] M2. Remove or feature-gate unused pallets from runtime construction
  - Surfaces: `crates/myosu-chain/runtime/src/lib.rs`, `crates/myosu-chain/runtime/Cargo.toml`
  - What exists after: Runtime constructs only with stage-0 pallets. Unused
    pallets behind a `full-runtime` feature flag.
  - Why now: 200K lines of dead code is maintenance and security burden.
Proof command: `SKIP_WASM_BUILD=1 cargo check -p myosu-chain-runtime --features fast-runtime`
  - Tests: `cargo test -p myosu-chain-runtime --quiet`

### Milestone 3: Reduce pallet-game-solver extrinsic surface

- [ ] M3. Remove or stub extrinsics not used in stage-0
  - Surfaces: `crates/myosu-chain/pallets/game-solver/src/lib.rs`
  - What exists after: Pallet has <=20 extrinsics. Removed extrinsics return
    `DispatchError::Other("not available in stage-0")`.
  - Why now: 69 extrinsics make the pallet incomprehensible.
Proof command: `cargo test -p pallet-game-solver --quiet -- stage_0`
  - Tests: `cargo test -p pallet-game-solver --quiet`

### Milestone 4: Strip EVM/Frontier from node service

- [ ] M4. Remove Frontier service configuration from node
  - Surfaces: `crates/myosu-chain/node/src/service.rs`, `crates/myosu-chain/node/Cargo.toml`
  - What exists after: Node binary without EVM/Frontier. Faster compile, smaller binary.
  - Why now: EVM unused in stage-0, adds significant compile time.
Proof command: `SKIP_WASM_BUILD=1 cargo check -p myosu-chain --features fast-runtime`
  - Tests: `SKIP_WASM_BUILD=1 cargo test -p myosu-chain --test stage0_local_loop --quiet`

### Milestone 5: Resolve chain TODO/FIXME comments

- [ ] M5. Triage 122 TODO/FIXME comments: fix, remove, or document as accepted
  - Surfaces: All files under `crates/myosu-chain/`
  - What exists after: <20 TODO/FIXME remain, each with justification.
  - Why now: TODOs in migration code are ticking time bombs.
Proof command: `rg -c "TODO|FIXME" crates/myosu-chain/ --type rust | awk -F: '{sum+=$2} END {print sum}'`
  - Tests: Count returns <20

## Surprises & Discoveries

- `substrate_fixed` pinned to encointer fork v0.6.0 is load-bearing for Yuma
  determinism. Must preserve even as other deps are stripped.
- `safe-math` and `share-pool` under `crates/myosu-chain/support/` are used by
  pallet-game-solver and must not be removed.
- Removing pallets may break storage migration ordering. Must verify migration
  indices after removal.

## Decision Log

- Decision: Feature-gate rather than delete unused pallet source files.
  - Why: Preserves option to re-enable without git archaeology.
  - Failure mode: Feature-gated code rots silently.
  - Mitigation: Monthly CI build with `full-runtime` feature.
  - Reversible: yes

- Decision: Cap extrinsic surface at 20, not strip to zero.
  - Why: Registration, staking, weight submission, subnet management needed.
  - Failure mode: Keeping too many extrinsics defeats the purpose.
  - Mitigation: Enumerate the exact extrinsics needed before starting.
  - Reversible: yes

## Validation and Acceptance

1. `cargo check -p myosu-chain-runtime --features fast-runtime` passes.
2. `cargo test -p pallet-game-solver --quiet -- stage_0` passes.
3. Runtime pallet count <= 7.
4. Pallet extrinsic count <= 20.
5. TODO/FIXME count < 20 across chain crate.

## Outcomes & Retrospective
_Updated after milestones complete._

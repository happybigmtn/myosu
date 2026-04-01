# Chain Runtime Restart Spec

**Lane**: `chain:runtime`
**Date**: 2026-03-28
**Status**: Runtime and node restored, local devnet proof landed, `GameSolver` now occupies runtime index 7, and the active `myosu-chain` dependency graph no longer includes `pallet-subtensor`

---

## Current State Inventory

Fresh source inspection plus the restored runtime compile show a different shape
than the earlier bootstrap review captured.

### What exists now

| Surface | Path | Current reality |
|---------|------|-----------------|
| Runtime sources | `crates/myosu-chain/runtime/src/lib.rs` | A large runtime file exists. This is no longer a missing-file problem. |
| Node sources | `crates/myosu-chain/node/src/*.rs` | `main.rs`, `service.rs`, `rpc.rs`, `command.rs`, `cli.rs`, `client.rs`, and EVM-related files all exist. |
| Common crate | `crates/myosu-chain/common/src/*.rs` | `currency.rs`, `evm_context.rs`, and `lib.rs` exist. |
| Game-solver pallet | `crates/myosu-chain/pallets/game-solver/` | Active workspace member, green again on the restored workspace line, and now configured inside the runtime. |
| Root workspace | `Cargo.toml` | Runtime and node are now real workspace members. |
| Runtime manifest | `crates/myosu-chain/runtime/Cargo.toml` | Real package exists with `myosu_chain_runtime` as the lib crate name. |
| Runtime build helper | `crates/myosu-chain/runtime/build.rs` | Real `substrate-wasm-builder` path now builds the runtime wasm instead of faking `WASM_BINARY = None`. |
| Node manifest | `crates/myosu-chain/node/Cargo.toml` | Real package exists and now reaches an honest compile and local boot path. |

### What is still broken

The runtime lane is no longer blocked on node restart, pallet dependency-line drift, first runtime
integration, or final runtime identity. The remaining reopened surfaces are support-lane cleanup:
the upstream Frontier benchmark incompatibility and historical subtensor-era cleanup outside the
live node/runtime path.

1. The runtime crate compiles from the repo root and now uses a real wasm build path.
2. The node crate compiles from the repo root, builds a local spec, and produces blocks on local
   `--dev`.
3. `pallet-game-solver` now compiles from the repo root on the same workspace line as the restored
   chain.
4. The local runtime still requires a tactical transaction-pool constraint: the fork-aware pool
   tears down the restarted local chain during startup, so local chains now default to the stable
   single-state pool unless the operator explicitly sets `--pool-type`.
5. `runtime/src/lib.rs` now contains `impl pallet_game_solver::Config for Runtime`, and
   `construct_runtime!` includes `GameSolver: pallet_game_solver = 7`.
6. Runtime transaction-extension and coldkey-swap dispatch-guard ownership now come from
   `pallet-game-solver`, not `pallet-subtensor`.
7. The runtime API payload surface now comes from `pallet_game_solver::rpc_info::*`, and
   `cargo check -p myosu-chain` stays green after that cutover.
8. Generated/local chain specs now emit `gameSolver` instead of `subtensorModule`.
9. The runtime version now advertises `spec_name = "myosu-chain"` and `impl_name = "myosu-chain"`,
   while the node banner identifies as `Myosu Node`.
10. `pallet_subtensor_swap::Config` now depends on `GameSolver` for `SubnetInfo`, `BalanceOps`,
    `TaoReserve`, and `AlphaReserve` instead of mixing `GameSolver` and `pallet_subtensor`.
11. The runtime helper cluster for identity registration, commitments, bond resets, and tempo
    lookup now also points at `GameSolver`, and `runtime/src/lib.rs` no longer contains direct
    `SubtensorModule::...` calls.
12. The live `migrate_init_total_issuance` runtime-upgrade hook now points at
    `pallet_game_solver::migrations::...` instead of `pallet_subtensor::migrations::...`.
13. The swap-simulation runtime API now constructs orders through
    `pallet_game_solver::{GetAlphaForTao, GetTaoForAlpha}` instead of the subtensor aliases.
14. The active node crate no longer imports `pallet_subtensor` outside the runtime boundary; the
    last leftover benchmark import was removed from `node/src/benchmarking.rs`.
15. The runtime no longer publicly re-exports `pallet_subtensor`; remaining subtensor references
    are internal runtime structure or dependent-pallet wiring, not downstream runtime exports.
16. `subtensor-transaction-fee` now compiles and test-harnesses against `pallet_game_solver`
    instead of `pallet_subtensor`.
17. `admin-utils` now also compiles and test-harnesses against `pallet_game_solver` instead of
    `pallet_subtensor`.
18. The runtime safe-mode and proxy policy shell now matches `RuntimeCall::GameSolver(...)`
    instead of carrying side-by-side `RuntimeCall::SubtensorModule(...)` call handling.
19. The runtime benchmark manifest wiring now includes `frame-system-benchmarking` plus the local
    runtime-benchmark feature fanout needed to reach the real next benchmark frontier.
20. The direct runtime/node manifest dependency edges to `pallet-subtensor` are gone, so the old
    pallet is no longer part of the active `myosu-chain` package graph.

### Verified failure evidence

From verification on 2026-03-28:

- `cargo metadata --format-version 1 --no-deps` now passes
- `cargo check -p myosu-chain-runtime --quiet` now passes
- `cargo check -p myosu-chain --quiet` now passes
- `cargo check -p pallet-game-solver --quiet` now passes
- `cargo run -p myosu-chain --quiet -- build-spec --chain dev` now succeeds
- `cargo run -p myosu-chain --quiet -- --dev --tmp --one --force-authoring` now succeeds and
  imports blocks on the default local path
- `cargo run -p myosu-chain --quiet -- build-spec --chain dev | rg -n '"(gameSolver|subtensorModule)"'`
  now reports `gameSolver`
- the live node banner now prints `Myosu Node`
- `rg -n "type (SubnetInfo|BalanceOps|TaoReserve|AlphaReserve) =" crates/myosu-chain/runtime/src/lib.rs`
  now reports all four swap-runtime bindings on `GameSolver`
- `rg -n "SubtensorModule::" crates/myosu-chain/runtime/src/lib.rs` now returns no matches
- `rg -n "migrate_init_total_issuance" crates/myosu-chain/runtime/src/lib.rs`
  now reports the runtime-upgrade hook under `pallet_game_solver::migrations`
- `rg -n "GetAlphaForTao::<Runtime>|GetTaoForAlpha::<Runtime>" crates/myosu-chain/runtime/src/lib.rs`
  now reports both swap-simulation order aliases under `pallet_game_solver`
- `rg -n "pallet_subtensor\\b" crates/myosu-chain/node/src/benchmarking.rs crates/myosu-chain/node/src/command.rs crates/myosu-chain/node/src/rpc.rs crates/myosu-chain/node/src/service.rs crates/myosu-chain/node/src/cli.rs crates/myosu-chain/node/src/client.rs`
  now returns no matches
- `rg -n "myosu_chain_runtime::pallet_subtensor|pub use pallet_subtensor;" crates -g '!**/target/**'`
  now returns no matches
- `cargo check -p subtensor-transaction-fee --quiet` now passes
- `cargo test -p subtensor-transaction-fee --no-run` now passes
- `cargo test -p subtensor-transaction-fee --lib tests::test_remove_stake_fees_tao -- --exact`
  now passes
- `cargo check -p pallet-admin-utils --quiet` now passes
- `cargo test -p pallet-admin-utils --no-run` now passes
- `cargo test -p pallet-admin-utils --lib tests::test_sudo_set_default_take -- --exact`
  now passes
- `rg -n "RuntimeCall::SubtensorModule|\\[pallet_subtensor, SubtensorModule\\]" crates/myosu-chain/runtime/src/lib.rs`
  now returns no matches
- `rg -n "GameSolver: pallet_game_solver = 7|SubtensorModule: pallet_subtensor = 7|impl pallet_subtensor::Config for Runtime|pallet-subtensor\\.workspace" crates/myosu-chain/runtime/src/lib.rs crates/myosu-chain/runtime/Cargo.toml crates/myosu-chain/node/Cargo.toml`
  now reports only `GameSolver: pallet_game_solver = 7`
- `cargo tree -p myosu-chain --invert pallet-subtensor` now reports no active package path
- `cargo check -p myosu-chain-runtime --features runtime-benchmarks --quiet` still fails, but the
  failure is now a narrower Frontier benchmark incompatibility in `pallet-ethereum`
  (`EnsureEthereumTransaction` missing `try_successful_origin`) rather than missing local runtime
  benchmark manifest wiring

This keeps the lane in restart mode only in the broader chain sense. The runtime/node restart
portion is now real; the active restart frontier has moved past final runtime identity and into
support-lane cleanup.

---

## Restart Boundary

The restart begins at the point where the checked-in chain surface becomes
honest:

1. the root workspace includes the chain packages that are supposed to build,
2. the runtime compiles against the current dependency graph, and
3. the runtime and node crates exist as real packages that can be checked directly.

The active blocker is no longer package truth, runtime dependency truth, node source truth, the
reopened pallet alignment problem, initial runtime integration, runtime API payload identity,
native/node presentation identity, swap-pallet trait ownership, runtime helper ownership, the
transaction-fee/admin-utils consumer paths, the runtime call-policy shell, or final cutover
between `SubtensorModule` and `GameSolver`. It is now support-lane cleanup: the narrower local
transaction-pool constraint, the upstream Frontier benchmark incompatibility, and historical
subtensor-era cleanup outside the live path.

---

## Salvageable Inputs

The following surfaces remain worth preserving during restart work:

- `crates/myosu-chain/node/src/*.rs`
  Reason: the node structure exists and should be repaired, not recreated.
- `crates/myosu-chain/runtime/src/lib.rs`
  Reason: runtime wiring exists and can serve as a concrete migration target.
- `crates/myosu-chain/common/src/currency.rs`
  Reason: domain types still matter, even if macros and imports must change.
- `crates/myosu-chain/common/src/evm_context.rs`
  Reason: self-contained helper surface.
- `crates/myosu-chain/pallets/game-solver/src/stubs.rs`
  Reason: already-authored local substitutes for stripped subtensor pieces.
- `crates/myosu-chain/pallets/game-solver/src/swap_stub.rs`
  Reason: documents the intended no-op swap direction.

---

## Non-Salvageable or Must-Strip Inputs

The following shapes still cannot be trusted as-is:

- any assumption that the earlier `pallet-game-solver` green state still reflects the current
  repo-root workspace

---

## Next Implementation Slices

### Slice 1: Runtime package truth

Keep the restored package manifests in place so Cargo can see the chain surfaces
that actually exist.

Proof target:

```bash
cargo metadata --format-version 1 --no-deps
```

### Slice 2: Preserve node and devnet truth

Keep the restored node dependency graph, chain spec, consensus wiring, and local dev boot path
honest while downstream chain work resumes.

Proof target:

```bash
cargo check -p myosu-chain --quiet
cargo run -p myosu-chain --quiet -- build-spec --chain dev
cargo run -p myosu-chain --quiet -- --dev --tmp --one --force-authoring
```

### Slice 3: Support-lane cleanup truth

Now that the standalone pallet compiles on the shared workspace line, owns the active runtime API
payload surface, emits `gameSolver` in spec generation, and occupies runtime index 7, keep the
support surfaces honest: dependency graph, benchmark lane, and historical cleanup.

Proof target:

```bash
rg -n "GameSolver: pallet_game_solver = 7|SubtensorModule: pallet_subtensor = 7|impl pallet_subtensor::Config for Runtime|pallet-subtensor\\.workspace" crates/myosu-chain/runtime/src/lib.rs crates/myosu-chain/runtime/Cargo.toml crates/myosu-chain/node/Cargo.toml
cargo tree -p myosu-chain --invert pallet-subtensor
cargo run -p myosu-chain --quiet -- build-spec --chain dev | rg -n '"(gameSolver|subtensorModule)"'
cargo check -p myosu-chain-runtime --features runtime-benchmarks --quiet
```

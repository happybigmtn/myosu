# Chain Restart

**Plan ID:** 007
**Status:** Complete — chain compiles, local devnet runs, `GameSolver` now occupies runtime index 7, and the active `myosu-chain` dependency graph no longer includes `pallet-subtensor`; remaining benchmark and historical-cleanup work is follow-on
**Priority:** FOUNDATION — everything downstream depends on this

`genesis/PLANS.md` at `genesis/PLANS.md` governs this document.

---

## Purpose / Big Picture

After this plan lands, `crates/myosu-chain` will be re-enabled in the workspace, the Substrate runtime will be wired correctly, and `cargo build -p myosu-chain` will produce a runnable binary that can start a local devnet and produce blocks. The game-solver pallet will be integrated into the runtime.

This is the most technically demanding plan in the 180-day turnaround. It involves deep Substrate/FRAME knowledge and is the hardest to reverse.

---

## Progress

- [x] Re-audit the real chain entry boundary instead of trusting the old plan text
- [x] Strip transaction-extension, guard, and migration baggage from `pallet-game-solver`
- [x] Restore a real `Config` section for `pallet-game-solver`
- [x] Remove optional `rpc_info` and runtime-common adapter tail from the pallet crate
- [x] Reconstruct the missing shared chain primitive/runtime crates needed by the pallet
- [x] Get `cargo check -p pallet-game-solver` to pass
- [x] Create the missing chain manifests and runtime/node packages after the pallet boundary is honest
- [x] Resume runtime wiring enough for `cargo check -p myosu-chain-runtime`
- [x] Restore the node dependency and source spine
- [x] Reconcile the `pallet-game-solver` dependency-line split exposed by runtime restore
- [x] Resume local devnet proof
- [x] Add a first honest runtime integration slice for `pallet-game-solver`
- [x] Restore the real game-solver extension and coldkey-swap guard surfaces, then hand those runtime roles to `GameSolver`
- [x] Move the runtime API payload surface from `pallet_subtensor::rpc_info::*` to `pallet_game_solver::rpc_info::*`
- [x] Move generated/local genesis patch identity from `subtensorModule` to `gameSolver`
- [x] Move the active native runtime and node presentation identity from `node-subtensor` / `Subtensor Node` to `myosu-chain` / `Myosu Node`
- [x] Move the swap pallet's `SubnetInfo` and `BalanceOps` dependency from `SubtensorModule` to `GameSolver`
- [x] Move the runtime helper cluster for identity registration, commitments, bond resets, and tempo lookup from `SubtensorModule` to `GameSolver`
- [x] Move the swap pallet reserve bindings from `pallet_subtensor` to `pallet_game_solver`
- [x] Move the live runtime `migrate_init_total_issuance` hook from `pallet_subtensor` to `pallet_game_solver`
- [x] Move the swap-simulation runtime API order aliases from `pallet_subtensor` to `pallet_game_solver`
- [x] Move the `subtensor-transaction-fee` production path and mock-runtime harness from `pallet_subtensor` to `pallet_game_solver`
- [x] Move the `admin-utils` production path and mock-runtime harness from `pallet_subtensor` to `pallet_game_solver`
- [x] Move the runtime safe-mode/proxy policy shell from side-by-side `SubtensorModule` + `GameSolver` matching to `GameSolver` as the live call surface
- [x] Restore the missing local runtime-benchmark manifest wiring so the benchmark lane reaches a real upstream frontier
- [x] Execute the final cutover so `GameSolver` occupies runtime index 7 instead of running alongside `SubtensorModule`
- [x] Remove the direct runtime/node manifest dependency edges that kept `pallet-subtensor` in the active `myosu-chain` package graph

---

## Surprises & Discoveries

- The shortest truthful restart path is not runtime integration first. It is pallet reduction first.
- `cargo check -p pallet-game-solver` improved from a 462-error failure fanout to a 75-error
  frontier after stripping extensions/guards/migrations, restoring the config section, and
  removing optional RPC/runtime-common adapter surfaces.
- Reconstructing `runtime_common`, `subtensor_runtime_common`, `safe_math`, `share_pool`,
  `subtensor_swap_interface`, and `subtensor_macros`, then restoring the real config/commit
  storage shape, was enough to get `cargo check -p pallet-game-solver` green again.
- Adding real `runtime/` and `node/` manifests plus workspace membership was enough to get
  `cargo metadata --format-version 1 --no-deps` green again.
- Restoring `runtime/src/{check_nonce,migrations,sudo_wrapper,transaction_payment_wrapper}.rs`
  removed the missing-file layer from the runtime check, but that turned out not to be the
  durable blocker.
- Restoring the opentensor workspace dependency spine and reducing the runtime transaction
  extension tuple was enough to get `cargo check -p myosu-chain-runtime` green again.
- That same workspace restoration exposed a split chain state: the runtime/node side now builds
  against the restored opentensor SDK/frontier line, while `pallet-game-solver` still carries the
  earlier parity-based dependency line and no longer compiles from the repo-root workspace.
- Restoring the node chain-spec helpers, local chain-spec builder, Aura/Babe consensus surfaces,
  and the runtime wasm build path was enough to get `cargo check -p myosu-chain` green again.
- The first honest local-devnet boot exposed a narrower runtime integration problem: the fork-aware
  transaction pool tears down the restarted local chain during startup, while the single-state pool
  stays stable and produces blocks.
- The shortest truthful fix was local-only, not global: keep honoring explicit operator choice, but
  default local chains to the stable single-state transaction pool until the fork-aware path is
  reconciled for this runtime.
- With that override in place, `cargo run -p myosu-chain -- --dev --tmp --one --force-authoring`
  now builds the spec, starts the node, and imports blocks on the default local path.
- Collapsing `pallet-game-solver` onto the shared workspace dependency line removed the fixed-point
  trait split and restored `cargo check -p pallet-game-solver` from the repo root.
- The smallest honest runtime integration slice was to configure `pallet-game-solver` directly in
  `runtime/src/lib.rs` and add it to `construct_runtime!` without ripping out `pallet_subtensor` in
  the same change.
- That integration slice compiles cleanly, and the node still builds specs and produces blocks, so
  the live gap is no longer "how does the pallet enter the runtime?" but "what is the final
  runtime identity once both pallets exist side-by-side?"
- The first attempt to move runtime transaction-extension and dispatch-guard ownership failed only
  because local stub modules had replaced still-existing real implementations. Promoting the real
  game-solver extension and guard modules closed that gap without new design work.
- The first attempt to move runtime API methods to `GameSolver` failed because
  `subtensor-custom-rpc-runtime-api` was concretely typed against
  `pallet_subtensor::rpc_info::*`. Retargeting that crate itself to
  `pallet_game_solver::rpc_info::*` was enough to make the runtime and node green again.
- The remaining genesis identity leak was just hardcoded preset JSON. Changing the runtime preset
  and local chain-spec patch keys from `subtensorModule` to `gameSolver` worked, and the node
  still initialized genesis and imported blocks.
- The remaining native identity leak was similarly narrow. Changing the runtime version strings,
  runtime lib crate name, node imports, and CLI presentation from `node-subtensor` /
  `Subtensor Node` to `myosu-chain` / `Myosu Node` did not disturb compile or block production.
- The swap pallet turned out to be another tractable cutover seam. `pallet-game-solver` already
  had the methods and storage needed to satisfy `SubnetInfo` and `BalanceOps`, so porting the
  small adapter impl block from `pallet-subtensor` was enough to let
  `pallet_subtensor_swap::Config` depend on `GameSolver` instead of `SubtensorModule`.
- The remaining direct runtime helper calls were another narrow seam. Identity registration,
  commitments, bond resets, and tempo lookup still called `SubtensorModule::...` inside
  `runtime/src/lib.rs`, but `pallet-game-solver` already exposed the same methods, so that helper
  cluster could move to `GameSolver` directly without another compatibility layer.
- The swap pallet reserve bindings were also narrower than they looked. Once the swap pallet used
  `GameSolver` for `SubnetInfo` and `BalanceOps`, it still referenced `pallet_subtensor` for
  `TaoCurrencyReserve` and `AlphaCurrencyReserve`, but `pallet-game-solver` already exported the
  same reserve types. Moving those bindings completed the swap pallet's runtime-side dependency
  cutover without changing its external behavior.
- The runtime upgrade seam was similarly narrow once inspected directly. The live
  `migrate_init_total_issuance` hook still pointed at `pallet_subtensor`, but the identical
  migration file already existed under `pallet-game-solver`; the real blocker was that the
  game-solver migration namespace still stubbed that module out. Exposing just that one migration
  module was enough to move runtime-upgrade ownership over without restoring the whole historical
  migration tree.
- The last swap-facing runtime API seam was also smaller than it looked. Swap simulation still
  built orders through `pallet_subtensor::{GetAlphaForTao, GetTaoForAlpha}`, but
  `pallet-game-solver` already exported the same aliases. Repointing those order constructors kept
  `build-spec` and local authoring proof green.
- The first bounded downstream consumer migration also stayed narrow. `subtensor-transaction-fee`
  still depended on `pallet_subtensor` in both production code and its mock-runtime harness, but
  the live logic only needed call-shape parity plus the same stake/subnet/reserve surfaces that
  `pallet-game-solver` already exposes. Repointing the crate and then carrying the test harness
  across the same boundary kept runtime and node proof green and restored a live fee-pallet test.
- `admin-utils` turned out to have the same shape at a bigger size. Although it touches a wide
  parameter surface, its dependency was still mostly crate-shaped rather than algorithm-shaped.
  Repointing the pallet to `pallet-game-solver`, carrying its mock-runtime test harness over, and
  exposing the one stripped `migrate_create_root_network` namespace those tests still called kept
  the pallet green without changing the runtime shell yet.
- The next runtime-shell seam was concentrated rather than broad. The remaining
  `RuntimeCall::SubtensorModule(...)` matches lived in `SafeModeWhitelistedCalls` and
  `ProxyType::filter`, not across the whole runtime. Repointing those policy matches to
  `RuntimeCall::GameSolver(...)` removed the last subtensor call-policy duplication without
  disturbing runtime compile or local node boot proof.
- The benchmark lane was also narrower than it first appeared once inspected honestly. The
  runtime manifest had never carried `frame-system-benchmarking` or the local benchmark feature
  fanout, so `runtime-benchmarks` could not even reach a meaningful compile frontier. Wiring that
  local manifest surface in moved the failure to a more truthful upstream boundary in
  `pallet-ethereum` rather than leaving it as a missing local dependency problem.
- The structural final cutover was smaller than it looked. Once the helper, policy, runtime-API,
  and downstream-consumer seams had moved, the remaining work reduced to deleting the stale
  `impl pallet_subtensor::Config for Runtime`, moving `GameSolver` to index 7, updating the
  runtime-side trait imports, and removing the direct runtime/node manifest edges that still kept
  the old pallet in the active package graph.
- The useful dependency proof is now negative. `cargo tree -p myosu-chain --invert
  pallet-subtensor` no longer finds a package path, which means `pallet-subtensor` is now a
  historical workspace crate rather than part of the live node/runtime graph.

---

## Decision Log

- Decision: Treat Plan 007 as a pallet restart before it becomes a runtime restart.
  Rationale: the checked-in plan assumed runtime/node manifests that do not exist yet, while the
  real blocker is still the `pallet-game-solver` dependency boundary.
  Date/Author: 2026-03-21 / Interim CEO

- Decision: Disable `pallet-crowdloan` and `pallet-drand` for now.
  Rationale: Current chain review work already marks them as strip-first dependencies. Defer until Phase 3.
  Date/Author: 2026-03-21 / Interim CEO

---

## Outcomes & Retrospective

*(To be written at plan completion)*

---

## Milestones

### M1: Make `pallet-game-solver` an honest compile target
Strip stale extension/guard/migration/RPC baggage and restore the pallet config surface so the
 remaining failures point at real missing shared crates rather than broken pallet shape.

Proof: `cargo check -p pallet-game-solver` now passes

### M2: Reconstruct the missing shared chain layer
Create or restore the chain-local primitive/runtime crates that the pallet still expects
(`subtensor_runtime_common`, `runtime_common`, `safe_math`, `share_pool`, and related macro/swap
 surfaces), or reduce the pallet further until those expectations are no longer needed.

Proof: `cargo check -p pallet-game-solver` remains green after reconstruction

### M3: Create real runtime and node manifests
Only after the pallet boundary is honest, create the missing `Cargo.toml` manifests for the chain
packages and reintroduce runtime wiring.

Proof: `cargo metadata --format-version 1 --no-deps`

### M4: Resume node integration and devnet proof
Preserve the now-green runtime, restore the node crate, and then verify local block production.

Proof: `cargo check -p myosu-chain`, `cargo run -p myosu-chain -- build-spec --chain dev`, and
local `--dev` block production

---

## Context and Orientation

Current chain state:
```
crates/myosu-chain/
├── node/               # ACTIVE — compiles and now produces blocks on local `--dev`
├── runtime/            # ACTIVE — compiles and now builds a real wasm binary
├── pallets/
│   ├── game-solver/    # ACTIVE — green on the restored workspace line and now configured in runtime
│   ├── subtensor/      # ACTIVE — still occupies the historical runtime position at index 7
│   ├── admin-utils/    # NEEDED
│   ├── commitments/   # NEEDED
│   ├── registry/       # NEEDED
│   ├── swap/           # NEEDED
│   ├── transaction-fee/ # NEEDED
│   ├── crowdloan/      # DISABLED
│   ├── drand/          # DISABLED
│   ├── proxy/          # DISABLED
│   ├── shield/         # DISABLED
│   └── utility/        # DISABLED
├── primitives/
│   ├── safe-math/      # EXISTS
│   └── share-pool/     # EXISTS
└── support/
    ├── macros/         # EXISTS
    ├── procedural-fork/ # EXISTS
    ├── linting/        # EXISTS
    └── tools/          # EXISTS
```

The active proof surface has now advanced past runtime compilation, node restoration,
pallet-line reconciliation, first runtime integration, runtime API cutover, genesis identity
cutover, native/node presentation identity cleanup, full swap-runtime dependency cutover, runtime
helper cutover, runtime-upgrade cutover, transaction-fee/admin-utils consumer cutover, runtime
policy-shell cutover, and final runtime-shell cutover. The remaining reopened work is no longer
index-7 ownership. It is support-lane cleanup: the benchmark frontier in Frontier/EVM and
historical subtensor-era naming/comments/tests outside the live node/runtime path.

---

## Plan of Work

1. Preserve the now-green runtime, node, standalone pallet, and final runtime cutover as the
   current chain baseline
2. Keep the local devnet proof honest, including the local transaction-pool constraint
3. Treat `GameSolver` at index 7 as the settled runtime identity for downstream work
4. Reduce or remove the remaining historical subtensor naming and support surfaces now that the runtime
   ownership boundary is explicit
5. Carry the runtime-benchmarks failure as an upstream/support-lane issue unless a local patch is
   intentionally adopted

---

## Concrete Steps

```bash
cargo metadata --format-version 1 --no-deps
cargo check -p myosu-chain-runtime
cargo check -p myosu-chain
cargo check -p pallet-game-solver
cargo run -p myosu-chain -- build-spec --chain dev
cargo run -p myosu-chain -- --dev --tmp --one --force-authoring
```

---

## Validation

- runtime and node manifests are now real workspace packages
- `cargo metadata --format-version 1 --no-deps` passes from the repo root
- `cargo check -p myosu-chain-runtime` passes from the repo root
- `cargo check -p myosu-chain` passes from the repo root
- `cargo check -p pallet-game-solver` passes from the repo root
- `cargo run -p myosu-chain -- build-spec --chain dev` produces a real local chain spec
- `cargo run -p myosu-chain -- --dev --tmp --one --force-authoring` produces blocks on the default
  local path
- local chains currently default to the single-state transaction pool unless the operator explicitly
  sets `--pool-type`, because the fork-aware pool still tears down the restarted local runtime
- `runtime/src/lib.rs` now contains `impl pallet_game_solver::Config for Runtime`
- `construct_runtime!` now includes `GameSolver: pallet_game_solver = 7`
- runtime transaction-extension and coldkey-swap dispatch-guard ownership now come from
  `pallet-game-solver`
- the runtime API payload surface now comes from `pallet_game_solver::rpc_info::*`
- `cargo run -p myosu-chain --quiet -- build-spec --chain dev | rg -n '"(gameSolver|subtensorModule)"'`
  now reports `gameSolver` and no longer reports `subtensorModule`
- the active runtime version strings now use `myosu-chain`
- the active node CLI/banner now identifies as `Myosu Node`
- `pallet_subtensor_swap::Config` now uses `GameSolver` for both `SubnetInfo` and `BalanceOps`
- `rg -n "RuntimeCall::SubtensorModule" crates/myosu-chain/runtime/src/lib.rs` now returns no
  matches
- `rg -n "GameSolver: pallet_game_solver = 7|SubtensorModule: pallet_subtensor = 7|impl pallet_subtensor::Config for Runtime|pallet-subtensor\\.workspace" crates/myosu-chain/runtime/src/lib.rs crates/myosu-chain/runtime/Cargo.toml crates/myosu-chain/node/Cargo.toml`
  now reports only `GameSolver: pallet_game_solver = 7`
- `cargo tree -p myosu-chain --invert pallet-subtensor` now reports no active package path
- `cargo check -p myosu-chain-runtime --features runtime-benchmarks --quiet` still fails in
  upstream `pallet-ethereum` because `EnsureEthereumTransaction` is missing
  `try_successful_origin`

---

## Failure Scenarios

| Scenario | Handling |
|----------|----------|
| `procedural-fork` diverges from upstream `frame-support-procedural` | Pin to specific upstream commit; do not auto-update |
| Runtime `construct_runtime!` macro errors | Work backwards from the error — usually missing `RuntimeEvent` or `RuntimeCall` variants |
| Substrate toolchain version mismatch | Pin `rust-toolchain.toml` to the version that works with `polkadot-sdk stable2407` |
| Runtime API cutover compiles but breaks node RPC | Keep the existing RPC method surface, but retarget the runtime API crate payload types first and verify with `cargo check -p myosu-chain` plus local block production |
| Chain produces blocks but RPC returns errors | Check that the RPC module is properly registered in the runtime |

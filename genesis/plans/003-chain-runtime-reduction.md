# Strip Runtime to the Stage-0 Chain Core

Status: Completed 2026-03-29.

This ExecPlan is a living document. Maintained per `genesis/PLANS.md`.

## Purpose / Big Picture

The runtime currently compiles, but it still carries the wrong chain. It wires
in Drand, Crowdloan, Frontier/EVM self-contained extrinsics, the swap pallet,
and other subtensor baggage that stage 0 does not need. `OS.md` says the chain
should know about subnets, neurons, weights, staking, and emissions. Everything
else is drag.

After this plan, `myosu-chain-runtime` will be a smaller, stage-0 runtime that
keeps `GameSolver` at runtime index 7 and removes the major non-stage-0 paths
that currently distort the architecture.

## Progress

- [x] (2026-03-28) Verified that `cargo check -p myosu-chain-runtime`
  succeeds in the current repo state.
- [x] (2026-03-28) Confirmed that the runtime still wires `pallet_drand`,
  `pallet_crowdloan`, `fp_self_contained`, Frontier/EVM, and swap.
- [x] (2026-03-28) Confirmed that the live `pallet-game-solver` Config trait no
  longer depends on `pallet_drand::Config + pallet_crowdloan::Config`, so the
  first blocker has moved from the pallet supertrait to the runtime surface.
- [x] (2026-03-28) Removed Drand from the live runtime path, including the
  runtime Config impl, Drand-specific offchain transaction helpers, and the
  Drand priority transaction extension.
- [x] (2026-03-28) Removed Crowdloan from the live runtime path and replaced
  the leftover `MaxContributors` constant with a plain stage-0 runtime
  constant instead of the old pallet wiring.
- [x] (2026-03-28) Removed `fp_self_contained` and switched the runtime back to
  ordinary Substrate `generic::UncheckedExtrinsic`.
- [x] (2026-03-28) Removed the commitments pallet from the live runtime path
  and switched `GameSolver` to empty runtime commitment adapters so Drand is no
  longer reintroduced through that side door.
- [x] (2026-03-29) Removed the dead swap runtime-API shell from the stripped
  runtime and node RPC assembly, so stage 0 no longer exports
  `current_alpha_price` / `sim_swap_*` methods that nothing in the active
  chain-client, miner, validator, or gameplay loop still consumes.
- [x] (2026-03-29) Removed the remaining Frontier/EVM runtime and node wiring,
  including the dead `pallet_evm_chain_id` dependency, the
  `sudo_set_evm_chain_id` admin path, and the empty node-side
  `ethereum`/`EthConfiguration` placeholder plumbing that no longer served any
  live stage-0 surface.
- [x] (2026-03-29) Removed the dead runtime-local `Registry` and `Contracts`
  carryovers, including the registry permission shim, the contracts runtime
  API surface, and the corresponding runtime dependencies. The stripped node
  and shared client still compile without them.
- [x] (2026-03-29) Removed `MevShield` from the live stage-0 path by deleting
  the coldkey-swap `submit_encrypted` exception, the runtime pallet wiring,
  and the node's no-op `mev_shield` author/revealer plumbing.
- [x] Remove or stub swap-era runtime wiring so `GameSolver` no longer depends
  on the full swap pallet.
- [x] (2026-03-29) Reconciled the live pallet indexes with the stage-0
  allowlist: `GameSolver` remains at index 7, dead stage-0-extraneous pallets
  are stripped, and `AdminUtils` remains the only intentionally carried
  bootstrap helper outside the core chain path.
- [x] (2026-03-29) Produced a clean stripped-runtime compile proof with
  `cargo fmt --all`, `SKIP_WASM_BUILD=1 cargo check -p myosu-chain-runtime`,
  `SKIP_WASM_BUILD=1 cargo check -p myosu-chain`, and
  `SKIP_WASM_BUILD=1 cargo test -p myosu-chain-runtime stage0_noop_swap --quiet`.

## Surprises & Discoveries

- Observation: Runtime build success is masking architectural debt.
  Evidence: `cargo check -p myosu-chain-runtime` succeeds even though
  `runtime/src/lib.rs` still contains the exact surfaces the doctrine says to
  strip.

- Observation: The older plan's first blocker was stale.
  Evidence: `crates/myosu-chain/pallets/game-solver/src/macros/config.rs`
  depends only on `frame_system::Config`; the drand/crowdloan supertrait no
  longer exists there.

- Observation: The swap pallet is still a major coupling point.
  Evidence: `runtime/src/lib.rs` still sets `type SwapInterface = Swap`, and
  the runtime still registers `Swap: pallet_subtensor_swap = 28`.

- Observation: a stripped runtime can still carry dead outward swap baggage
  even after the live pallet path stops using the full swap pallet.
  Evidence: `runtime/src/lib.rs` and `node/src/rpc.rs` were still exporting the
  old swap runtime API and node RPC shell until this slice removed them, even
  though the surviving stage-0 client only used `neuronInfo_getNeuronsLite`.

- Observation: the last runtime EVM dependency was no longer in the runtime's
  main wiring, but in an admin side door.
  Evidence: removing `impl pallet_evm_chain_id::Config for Runtime` still left
  the runtime blocked through `pallet_admin_utils::Config`, whose only live
  chain-id use was the inherited `sudo_set_evm_chain_id` extrinsic.

- Observation: the old scaffold allowlist is still stricter than the live
  stage-0 bootstrap path.
  Evidence: after stripping `Registry`, `Contracts`, and `MevShield`, the live
  runtime still carries `AdminUtils` because the validator/bootstrap path uses
  sudo subnet normalization. The stricter older allowlist was stale, not the
  runtime index truth.

- Observation: Drand was still entering through more than one route.
  Evidence: After removing the obvious runtime Drand wiring, compilation still
  failed because `pallet_commitments` required `pallet_drand::Config` and
  `SubtensorTransactionExtension` also required `pallet_drand::Config`.

## Decision Log

- Decision: Strip unwanted runtime paths instead of hiding them behind default
  feature gates.
  Rationale: The stage-0 target is a smaller fork, not a long-lived dual-mode
  runtime.
  Date/Author: 2026-03-28 / Codex

- Decision: Keep `GameSolver` at runtime index 7 while stripping everything
  around it.
  Rationale: `OS.md` and `AGENTS.md` treat index 7 as a stable stage-0
  invariant.
  Date/Author: 2026-03-28 / Codex

## Outcomes & Retrospective

The runtime is now honestly stage-0-shaped. `GameSolver` stays at runtime
index 7, the swap-era path is reduced to the no-op seam the pallet still
needs, and the dead drand, crowdloan, commitments, Frontier/EVM, registry,
contracts, and MEV-shield surfaces are gone from the live runtime/node path.
The remaining extra pallet is `AdminUtils`, and that carry is now explicit
rather than accidental because the bootstrap flow still uses its sudo subnet
normalization hooks.

## Context and Orientation

The files that matter are:

- `crates/myosu-chain/runtime/src/lib.rs`, which defines imports, Config impls,
  `construct_runtime!`, transaction extensions, and extrinsic types.
- `crates/myosu-chain/runtime/Cargo.toml`, which carries runtime dependencies.
- `crates/myosu-chain/pallets/game-solver/`, because runtime strip-down must
  stay aligned with the surviving pallet surface.

Known strip targets in `runtime/src/lib.rs`:

- `impl pallet_drand::Config for Runtime`
- `CreateBare<pallet_drand::Call<Runtime>>`
- `CreateSignedTransaction<pallet_drand::Call<Runtime>>`
- `impl fp_self_contained::SelfContainedCall for RuntimeCall`
- `impl pallet_crowdloan::Config for Runtime`
- `Drand: pallet_drand = 26`
- `Crowdloan: pallet_crowdloan = 27`
- `Swap: pallet_subtensor_swap = 28`
- Frontier/EVM imports and pallet registrations around Ethereum/EVM/BaseFee

The current runtime is therefore not wrong because it fails to compile. It is
wrong because it still compiles the old network shape.

## Milestones

### Milestone 1: Remove Drand and Crowdloan from the live runtime

Delete the Drand and Crowdloan runtime Config impls, remove their transaction
extension and offchain wiring, and remove both pallets from
`construct_runtime!`.

Proof commands:

    cargo check -p myosu-chain-runtime
    rg -n "pallet_drand|pallet_crowdloan" crates/myosu-chain/runtime/src/lib.rs

Acceptance is the runtime still compiling and the grep no longer finding live
runtime references outside historical comments or test-only paths.

### Milestone 2: Remove self-contained Ethereum extrinsics and Frontier/EVM

Delete the `fp_self_contained` extrinsic types and self-contained call
implementation, then remove the Ethereum/EVM/BaseFee pallet wiring and any
leftover node-side placeholder plumbing that only existed to support it.

Proof commands:

    cargo check -p myosu-chain-runtime
    rg -n "fp_self_contained|pallet_ethereum|pallet_evm|BaseFee" \
      crates/myosu-chain/runtime/src/lib.rs

Acceptance is the runtime compiling with ordinary Substrate extrinsics and no
live Frontier/EVM path.

### Milestone 3: Replace swap-era runtime coupling with the stage-0 seam

Remove the live swap pallet registration from the runtime and switch the pallet
integration to the no-op or narrow stage-0 interface described by the active
pallet plan.

Proof commands:

    cargo check -p myosu-chain-runtime
    rg -n "Swap: pallet_subtensor_swap|type SwapInterface = Swap" \
      crates/myosu-chain/runtime/src/lib.rs

Acceptance is the runtime compiling without the full swap pallet in the live
path.

## Plan of Work

Read `runtime/src/lib.rs` top to bottom and delete the unwanted runtime pieces
in the same order they currently appear: Drand, self-contained Ethereum,
Crowdloan, Frontier/EVM, then swap. Update the runtime dependency list only as
needed after the code is stripped. Keep the surviving runtime small enough that
later node and pallet work are working on the intended network, not a staged
mock of the old one.

## Concrete Steps

From `/home/r/coding/myosu`:

    cargo check -p myosu-chain-runtime
    nl -ba crates/myosu-chain/runtime/src/lib.rs | sed -n '105,190p;1085,1110p;1405,1495p;1605,1660p'
    rg -n "pallet_drand|pallet_crowdloan|fp_self_contained|pallet_ethereum|pallet_evm|Swap: pallet_subtensor_swap" \
      crates/myosu-chain/runtime/src/lib.rs

## Validation and Acceptance

This plan is complete when:

- `cargo check -p myosu-chain-runtime` passes.
- `runtime/src/lib.rs` no longer contains live Drand, Crowdloan,
  `fp_self_contained`, Frontier/EVM, or swap-pallet registrations.
- `GameSolver` remains wired at runtime index 7.

## Idempotence and Recovery

This plan is intentionally destructive. Retry by removing one runtime surface
at a time and recompiling immediately after each deletion. If one removal
uncovers a hidden dependency, keep the deletion and adapt the dependent code
rather than reintroducing the old path.

## Interfaces and Dependencies

This plan has no upstream dependency and is the first active chain step. It
blocks the node, pallet, and participant plans.

# Minimalize Node Service and Devnet Surface

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This plan follows `genesis/PLANS.md`.

## Purpose / Big Picture

The node package is still frontier-heavy and carries placeholder chain-spec files. This plan delivers a minimal, stage-0 node service that can build a reproducible devnet spec and run local block production with the stripped runtime.

## Progress

- [x] (2026-03-28 21:19Z) Audited node entrypoints and found placeholder `devnet.rs` and `testnet.rs` plus broad `service.rs` coupling.
- [ ] Constrain chain-spec ownership to `devnet` and `localnet` for stage-0.
- [ ] Reduce service wiring to one consensus path and one startup mode for local devnet.
- [ ] Reduce RPC surface to stage-0 essentials (system, transaction payment, game-solver needed RPCs).
- [ ] Add a reproducible `build-spec` + boot command path for operator documentation and CI smoke checks.
- [ ] Add focused node tests for command parsing and chain-spec generation.

## Surprises & Discoveries

- Observation: node source includes multiple consensus and EVM bridge modules not required for stage-0 proof goals.
  Evidence: `crates/myosu-chain/node/src/service.rs`, `consensus/*.rs`, `ethereum.rs`, `conditional_evm_block_import.rs`.
- Observation: placeholder chain-spec files exist in active source tree.
  Evidence: `crates/myosu-chain/node/src/chain_spec/devnet.rs`, `testnet.rs`.

## Decision Log

- Decision: keep only one operator-facing stage-0 startup path in this plan.
  Rationale: multiple launch modes create ambiguity and flaky proofs.
  Inversion (failure mode): if we keep parallel startup paths during stabilization, CI and operator commands diverge.
  Date/Author: 2026-03-28 / Genesis

- Decision: node minimalization depends on runtime reduction and does not edit pallet behavior.
  Rationale: avoid cross-layer regressions.
  Inversion (failure mode): editing node and pallet logic together hides which layer broke boot.
  Date/Author: 2026-03-28 / Genesis

## Failure Modes

| Codepath | Realistic failure | Handling |
|---|---|---|
| Chain-spec generation | Invalid genesis config causes immediate startup panic | Add `build-spec` smoke command and JSON validity check before merge |
| RPC module wiring | Missing runtime API at startup causes RPC init failure | Gate with `cargo check -p myosu-chain` and targeted RPC init test |
| Service startup | Node boots but does not author blocks | Add one deterministic local run smoke command and log assertion |

## Outcomes & Retrospective

- Pending implementation.

## Context and Orientation

Owned files in this plan:
- `crates/myosu-chain/node/Cargo.toml`
- `crates/myosu-chain/node/src/command.rs`
- `crates/myosu-chain/node/src/service.rs`
- `crates/myosu-chain/node/src/rpc.rs`
- `crates/myosu-chain/node/src/chain_spec/mod.rs`
- `crates/myosu-chain/node/src/chain_spec/devnet.rs`
- `crates/myosu-chain/node/src/chain_spec/localnet.rs`
- `crates/myosu-chain/node/src/main.rs`

Not owned here:
- Runtime pallet composition (`003`)
- Pallet internal logic (`005`)

## Milestones

### Milestone 1: Chain-spec scope cleanup

Consolidate stage-0 chain-spec entrypoints and make `devnet` authoritative.

Proof command:

    rg -n "mod devnet|mod localnet|mod testnet" crates/myosu-chain/node/src/chain_spec/mod.rs
    test -s crates/myosu-chain/node/src/chain_spec/devnet.rs

### Milestone 2: Service path simplification

Reduce `service.rs` to one stage-0 consensus/startup path suitable for local verification.

Proof command:

    rg -n "new_full|new_partial|consensus" crates/myosu-chain/node/src/service.rs

### Milestone 3: RPC surface reduction

Keep only required RPC modules for stage-0 proofs.

Proof command:

    rg -n "create_full|transaction_payment|subtensor|game_solver|rpc" crates/myosu-chain/node/src/rpc.rs

### Milestone 4: Devnet boot proof path

Provide one reproducible command pair for devnet spec generation and boot.

Proof command:

    cargo run -p myosu-chain -- --chain devnet build-spec --disable-default-bootnode > /tmp/myosu-devnet-spec.json
    test -s /tmp/myosu-devnet-spec.json

### Milestone 5: Node compile and command tests

Add targeted tests and compile gate for node stage-0 path.

Proof command:

    cargo check -p myosu-chain --features fast-runtime
    cargo test -p myosu-chain command --quiet

## Plan of Work

1. Normalize chain-spec ownership around `devnet`.
2. Simplify service and RPC initialization.
3. Lock one operator proof path for spec generation and startup.
4. Add targeted command tests.

## Concrete Steps

From `/home/r/coding/myosu`:

    sed -n '1,220p' crates/myosu-chain/node/src/chain_spec/mod.rs
    sed -n '1,260p' crates/myosu-chain/node/src/service.rs
    sed -n '1,240p' crates/myosu-chain/node/src/rpc.rs
    cargo check -p myosu-chain --features fast-runtime

## Validation and Acceptance

Accepted when:
- node compiles with `cargo check -p myosu-chain --features fast-runtime`
- `devnet` build-spec command outputs non-empty JSON
- command parsing test path passes

## Idempotence and Recovery

- Re-running `build-spec` is safe.
- If node boot regresses, restore `service.rs` last passing revision and replay milestones in order.

## Artifacts and Notes

- Update `outputs/chain/runtime/review.md` and `outputs/chain/pallet/review.md` with node-interface implications.

## Interfaces and Dependencies

Depends on: `003-chain-runtime-reduction.md`
Blocks: `007-miner-validator-bootstrap.md`, `011-security-observability-release.md`

```text
runtime (from 003)
      |
      v
node chain_spec (devnet/localnet)
      |
      v
service.rs + rpc.rs startup path
      |
      v
devnet build-spec + boot proof command
```

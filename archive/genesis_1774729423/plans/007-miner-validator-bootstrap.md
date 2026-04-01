# Bootstrap Miner, Validator, and Shared Chain Client

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This plan follows `genesis/PLANS.md`.

## Purpose / Big Picture

Stage-0 exit criteria require one miner and one validator loop with deterministic scoring behavior. This plan creates the minimum executable off-chain service set: shared chain client crate, miner training/serve loop, validator scoring/weight submission loop, and determinism proof harness.

## Progress

- [x] (2026-03-28 21:34Z) Confirmed miner/validator crates are not active workspace members and must be bootstrapped with minimal scope.
- [ ] Create `myosu-chain-client` crate for shared RPC/extrinsic calls used by miner and validator.
- [ ] Add `myosu-miner` crate with MCCFR step loop + strategy serving endpoint.
- [ ] Add `myosu-validator` crate with exploitability scoring loop + weight submission command.
- [ ] Add determinism proof harness for INV-003 (same input -> same score across validators).
- [ ] Add bootstrap service runbooks and one command set for local operator loop.

## Surprises & Discoveries

- Observation: root workspace comments show intended miner/validator crates, but they are currently disabled.
  Evidence: `Cargo.toml` workspace members comments.
- Observation: `myosu-play` already has strong local advisor behavior that can seed miner artifact/bootstrap behavior.
  Evidence: `crates/myosu-play/src/main.rs`, `crates/myosu-games-poker/src/{request.rs,robopoker.rs,solver.rs}`.

## Decision Log

- Decision: build `myosu-chain-client` first and force both miner and validator through it.
  Rationale: prevents duplicated chain logic and conflicting signing/submit code.
  Inversion (failure mode): separate ad-hoc clients create score-submission divergence and untraceable bugs.
  Date/Author: 2026-03-28 / Genesis

- Decision: keep bootstrap HTTP/service surfaces intentionally small (single strategy endpoint, single score-submit endpoint path).
  Rationale: stage-0 proof needs determinism and operability, not broad API surface.
  Inversion (failure mode): large service API before invariant gates increases attack and regression surface.
  Date/Author: 2026-03-28 / Genesis

## Failure Modes

| Codepath | Realistic failure | Handling |
|---|---|---|
| Miner training loop | Long-running loop publishes stale profile under race | Use snapshot publish boundary and explicit version stamp |
| Validator scoring | Network timeout causes partial score batch | Retry with bounded backoff and batch-level idempotency key |
| Weight submission | Duplicate or malformed weights accepted/rejected inconsistently | Centralize payload build/validation in `myosu-chain-client` |

## Outcomes & Retrospective

- Pending implementation.

## Context and Orientation

Owned files in this plan (new and existing):
- `Cargo.toml` (workspace members)
- `crates/myosu-chain-client/Cargo.toml`
- `crates/myosu-chain-client/src/lib.rs`
- `crates/myosu-miner/Cargo.toml`
- `crates/myosu-miner/src/main.rs`
- `crates/myosu-validator/Cargo.toml`
- `crates/myosu-validator/src/main.rs`
- `fabro/run-configs/services/miner-service.toml`
- `fabro/run-configs/services/validator-oracle.toml`

Not owned here:
- Runtime/node/pallet stabilization (`003`-`005`)
- Wire/checkpoint hardening (`008`)

## Milestones

### Milestone 1: Shared chain client crate

Create a shared client crate with typed methods for register, fetch scores, and submit weights.

Proof command:

    test -s crates/myosu-chain-client/Cargo.toml
    test -s crates/myosu-chain-client/src/lib.rs
    cargo check -p myosu-chain-client

### Milestone 2: Miner bootstrap binary

Create miner binary with train/serve subcommands and strategy response endpoint.

Proof command:

    test -s crates/myosu-miner/src/main.rs
    cargo check -p myosu-miner

### Milestone 3: Validator bootstrap binary

Create validator binary with score/submit subcommands and exploitability pipeline stub.

Proof command:

    test -s crates/myosu-validator/src/main.rs
    cargo check -p myosu-validator

### Milestone 4: Determinism harness (INV-003)

Add test/harness proving two validator runs on same input yield identical score payloads.

Proof command:

    cargo test -p myosu-validator inv_003 --quiet

### Milestone 5: Operator runbook and service configs

Align Fabro service run-configs and local run commands with the new crates.

Proof command:

    rg -n "myosu-miner|myosu-validator|myosu-chain-client" fabro/run-configs/services/miner-service.toml fabro/run-configs/services/validator-oracle.toml Cargo.toml

## Plan of Work

1. Land shared chain client first.
2. Add miner and validator binaries with minimal command surfaces.
3. Add determinism test harness and service runbook updates.

## Concrete Steps

From `/home/r/coding/myosu`:

    rg -n "myosu-miner|myosu-validator|myosu-chain-client" Cargo.toml || true
    cargo check -p myosu-chain-runtime -p myosu-chain
    cargo check -p myosu-miner -p myosu-validator -p myosu-chain-client

## Validation and Acceptance

Accepted when:
- `myosu-chain-client`, `myosu-miner`, and `myosu-validator` compile
- determinism harness for INV-003 passes
- service run-configs reference new binaries correctly

## Idempotence and Recovery

- Compile/test proofs are repeatable.
- If one binary blocks compile, keep other services frozen and recover by crate-local rollback.

## Artifacts and Notes

- Update `outputs/miner/service/*` and `outputs/validator/oracle/*` after each milestone.

## Interfaces and Dependencies

Depends on: `003-chain-runtime-reduction.md`, `004-node-devnet-minimalization.md`, `005-pallet-game-solver-simplification.md`, `008-artifact-wire-checkpoint-hardening.md`
Blocks: `011-security-observability-release.md`

```text
runtime+node+pallet stable (003/004/005)
                 |
                 v
        myosu-chain-client (shared)
             /           \
            v             v
      myosu-miner    myosu-validator
            \          /
             v        v
          INV-003 determinism harness
```

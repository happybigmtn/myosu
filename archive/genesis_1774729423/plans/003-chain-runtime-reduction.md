# Reduce Chain Runtime to Stage-0 Core

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This plan follows `genesis/PLANS.md`.

## Purpose / Big Picture

The runtime currently carries more subtensor/frontier surface than stage-0 requires. This plan reduces runtime scope to the smallest set needed for local devnet blocks and game-solver integration, while preserving a reversible path for deferred features.

## Progress

- [x] (2026-03-28 21:15Z) Identified runtime hot paths and compile blockers in `runtime/src/lib.rs` and workspace manifest wiring.
- [ ] Define and commit a stage-0 runtime pallet allowlist and remove non-allowlisted runtime wiring.
- [ ] Remove drand/crowdloan supertrait pressure from runtime-facing config boundaries.
- [ ] Replace `fp_self_contained` coupling with standard Substrate extrinsic flow for stage-0.
- [ ] Add focused runtime proof tests for nonce checks, migrations routing, and transaction payment wrappers.
- [ ] Produce a reproducible runtime build proof command for CI and operator runbooks.

## Surprises & Discoveries

- Observation: `myosu-chain-runtime` still enables many EVM/frontier and dual-token assumptions by default.
  Evidence: `crates/myosu-chain/runtime/Cargo.toml` and `crates/myosu-chain/runtime/src/lib.rs`.
- Observation: runtime helper modules exist (`check_nonce`, `migrations`, wrappers) but are not treated as first-class proof targets.
  Evidence: `crates/myosu-chain/runtime/src/check_nonce.rs`, `migrations.rs`, `sudo_wrapper.rs`, `transaction_payment_wrapper.rs`.

## Decision Log

- Decision: runtime reduction is structural-only in this plan; no new emissions behavior lands here.
  Rationale: mixing structural strip-down with behavior changes hides regressions.
  Inversion (failure mode): if we edit Yuma/emission behavior during runtime reduction, runtime compile recovery may mask consensus regressions.
  Date/Author: 2026-03-28 / Genesis

- Decision: cap each milestone to at most 8 owned files.
  Rationale: runtime complexity is already high; small slices are reversible.
  Inversion (failure mode): large edits across runtime + node + pallet at once create multi-cause failures that are hard to unwind.
  Date/Author: 2026-03-28 / Genesis

## Failure Modes

| Codepath | Realistic failure | Handling |
|---|---|---|
| Runtime block execution | Invalid extrinsic dispatch after removing self-contained path | Add compile gate + targeted runtime smoke test before merging |
| Nonce verification | Nonce bypass/regression due wrapper rewiring | Add focused tests in `check_nonce.rs` and include in CI job |
| Runtime migration hooks | Migration path silently skipped | Add explicit migration routing assertion in runtime tests |

## Outcomes & Retrospective

- Pending implementation.

## Context and Orientation

Owned files in this plan:
- `Cargo.toml`
- `crates/myosu-chain/runtime/Cargo.toml`
- `crates/myosu-chain/runtime/src/lib.rs`
- `crates/myosu-chain/runtime/src/check_nonce.rs`
- `crates/myosu-chain/runtime/src/migrations.rs`
- `crates/myosu-chain/runtime/src/sudo_wrapper.rs`
- `crates/myosu-chain/runtime/src/transaction_payment_wrapper.rs`

Not owned here (handled in other plans):
- Node service/rpc/spec wiring (`004`)
- Pallet behavior and emissions math (`005`)

## Milestones

### Milestone 1: Stage-0 runtime allowlist

Pin the minimal pallet/runtime composition in `runtime/src/lib.rs` and remove references to stripped stage-0-out-of-scope paths.

Proof command:

    rg -n "construct_runtime!|pallet_game_solver|pallet_balances|pallet_timestamp" crates/myosu-chain/runtime/src/lib.rs

### Milestone 2: Remove supertrait blockers

Eliminate runtime-facing dependence on drand/crowdloan config supertrait requirements where they leak into stage-0 runtime config.

Proof command:

    rg -n "pallet_drand::Config|pallet_crowdloan::Config" crates/myosu-chain/runtime/src/lib.rs || true

### Milestone 3: Replace self-contained path

Swap `fp_self_contained` extrinsic dependencies for the standard Substrate extrinsic path for stage-0.

Proof command:

    rg -n "fp_self_contained|SelfContainedCall" crates/myosu-chain/runtime/src/lib.rs || true

### Milestone 4: Runtime helper proof tests

Add/adjust helper tests for nonce checking, migration hook wiring, and transaction payment wrapper behavior.

Proof command:

    cargo test -p myosu-chain-runtime check_nonce --quiet
    cargo test -p myosu-chain-runtime migrations --quiet

### Milestone 5: Reproducible runtime build proof

Guarantee one stable compile command for operator and CI usage.

Proof command:

    cargo check -p myosu-chain-runtime --features fast-runtime

## Plan of Work

1. Freeze the stage-0 runtime surface in `runtime/src/lib.rs`.
2. Remove supertrait and self-contained blockers.
3. Add focused runtime helper tests.
4. Validate with one stable runtime check command.

## Concrete Steps

From `/home/r/coding/myosu`:

    sed -n '1,260p' crates/myosu-chain/runtime/src/lib.rs
    rg -n "fp_self_contained|drand|crowdloan|construct_runtime!" crates/myosu-chain/runtime/src
    cargo check -p myosu-chain-runtime --features fast-runtime

## Validation and Acceptance

Accepted when:
- runtime compiles with `--features fast-runtime`
- `fp_self_contained` coupling is removed from runtime entrypoints
- runtime helper tests for nonce/migration routing pass

## Idempotence and Recovery

- Proof commands are repeatable.
- If runtime fails after strip-down, restore the last passing commit for `runtime/src/lib.rs` and re-apply milestones one at a time.

## Artifacts and Notes

- Update `outputs/chain/runtime/spec.md` and `outputs/chain/runtime/review.md` after each major slice.

## Interfaces and Dependencies

Depends on: `002-spec-corpus-normalization.md`
Blocks: `004-node-devnet-minimalization.md`, `005-pallet-game-solver-simplification.md`, `007-miner-validator-bootstrap.md`

```text
Cargo.toml + runtime/Cargo.toml
            |
            v
runtime/src/lib.rs (surface reduction)
            |
            v
runtime helper modules (check_nonce/migrations/wrappers)
            |
            v
stable runtime compile proof for CI + operators
```

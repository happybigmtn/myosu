# Simplify `pallet-game-solver` for Stage-0 Consensus

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This plan follows `genesis/PLANS.md`.

## Purpose / Big Picture

`pallet-game-solver` currently carries broad subtensor-era behavior, migrations, and token mechanics. This plan narrows the pallet to stage-0 needs: subnet registration, weight submission, exploit-quality scoring flow, and deterministic emission distribution without AMM-era complexity.

## Progress

- [x] (2026-03-28 21:24Z) Audited pallet modules and confirmed large migration and staking surface beyond stage-0 scope.
- [ ] Define stage-0-owned pallet modules and gate/remove out-of-scope swap/timelock behavior.
- [ ] Keep commit-reveal v2 only and remove CRV3 timelock paths.
- [ ] Route all Swap/Proxy/Commitments calls through explicit stage-0 stubs.
- [ ] Add deterministic emission accounting checks tied to invariant gates.
- [ ] Add focused tests for registration, weights, consensus math, and emission totals.

## Surprises & Discoveries

- Observation: pallet has a very large migrations tree including CRV3 timelock-era transitions.
  Evidence: `crates/myosu-chain/pallets/game-solver/src/migrations/*.rs`.
- Observation: stub files exist but are not yet the dominant execution path.
  Evidence: `crates/myosu-chain/pallets/game-solver/src/stubs.rs`, `swap_stub.rs`, and callsites in `lib.rs`.

## Decision Log

- Decision: separate structural strip (module gating/stubs) from behavioral tuning (emission formulas).
  Rationale: avoids hidden regressions and makes rollback simple.
  Inversion (failure mode): changing structure and economics together risks silent emission accounting drift.
  Date/Author: 2026-03-28 / Genesis

- Decision: preserve existing test module topology and add tests inside `src/tests/` rather than inventing a second harness.
  Rationale: use existing coverage conventions.
  Inversion (failure mode): introducing parallel harnesses weakens confidence and duplicates fixture logic.
  Date/Author: 2026-03-28 / Genesis

## Failure Modes

| Codepath | Realistic failure | Handling |
|---|---|---|
| Commit/reveal | Old CRV3 path remains reachable and causes invalid reveal timing | Remove CRV3 path, enforce v2 route, add targeted reveal tests |
| Weight submission | Validator submits malformed weights and state accepts them | Keep strict validation in `subnets/weights.rs` and add invalid-input tests |
| Emission distribution | Emission sum mismatch vs block emission | Add no-ship invariant test with explicit accounting assertion |

## Outcomes & Retrospective

- Pending implementation.

## Context and Orientation

Owned files in this plan:
- `crates/myosu-chain/pallets/game-solver/src/lib.rs`
- `crates/myosu-chain/pallets/game-solver/src/stubs.rs`
- `crates/myosu-chain/pallets/game-solver/src/swap_stub.rs`
- `crates/myosu-chain/pallets/game-solver/src/subnets/registration.rs`
- `crates/myosu-chain/pallets/game-solver/src/subnets/weights.rs`
- `crates/myosu-chain/pallets/game-solver/src/epoch/run_epoch.rs`
- `crates/myosu-chain/pallets/game-solver/src/epoch/math.rs`
- `crates/myosu-chain/pallets/game-solver/src/tests/{consensus.rs,weights.rs,emission.rs,epoch.rs}`

Not owned here:
- Runtime construct wiring (`003`)
- Node bootstrap and RPC wiring (`004`)

## Milestones

### Milestone 1: Stage-0 module boundary

Declare and enforce which pallet modules are live in stage-0 and which are stubbed/deferred.

Proof command:

    rg -n "mod subnets|mod epoch|mod coinbase|mod swap|mod staking|mod migrations" crates/myosu-chain/pallets/game-solver/src/lib.rs

### Milestone 2: Commit-reveal v2 only

Remove or fully gate CRV3/timelock migration and runtime paths.

Proof command:

    rg -n "crv3|timelock|migrate_crv3" crates/myosu-chain/pallets/game-solver/src || true

### Milestone 3: Stub route finalization

Ensure swap/proxy/commitment hooks route through explicit stage-0 stub boundaries.

Proof command:

    rg -n "swap_stub|stubs::|SwapInterface" crates/myosu-chain/pallets/game-solver/src/lib.rs crates/myosu-chain/pallets/game-solver/src/stubs.rs crates/myosu-chain/pallets/game-solver/src/swap_stub.rs

### Milestone 4: Emission accounting invariant

Add deterministic assertion that total distributed emission matches configured block emission over test epochs.

Proof command:

    cargo test -p pallet-game-solver emission --quiet

### Milestone 5: Validator-path proofs

Add/strengthen tests for registration, weights, and consensus update behavior.

Proof command:

    cargo test -p pallet-game-solver registration --quiet
    cargo test -p pallet-game-solver weights --quiet
    cargo test -p pallet-game-solver consensus --quiet

## Plan of Work

1. Lock module boundaries in `lib.rs`.
2. Remove CRV3/timelock path.
3. Make stubs first-class boundaries.
4. Add deterministic emission and weight/consensus tests.

## Concrete Steps

From `/home/r/coding/myosu`:

    sed -n '1,260p' crates/myosu-chain/pallets/game-solver/src/lib.rs
    rg -n "crv3|timelock|swap_stub|weights|run_epoch" crates/myosu-chain/pallets/game-solver/src
    cargo test -p pallet-game-solver weights --quiet

## Validation and Acceptance

Accepted when:
- stage-0 module boundaries are explicit in `lib.rs`
- CRV3/timelock paths are absent or unreachable in stage-0
- weight/consensus/emission tests pass

## Idempotence and Recovery

- Test commands are repeatable.
- If behavior regresses, rollback the last milestone and rerun only the affected test family (`weights`, `consensus`, or `emission`).

## Artifacts and Notes

- Update `outputs/chain/pallet/spec.md` and `outputs/chain/pallet/review.md` after each accepted milestone.

## Interfaces and Dependencies

Depends on: `003-chain-runtime-reduction.md`, `002-spec-corpus-normalization.md`
Blocks: `007-miner-validator-bootstrap.md`, `011-security-observability-release.md`

```text
runtime config (003)
      |
      v
pallet lib.rs module boundary
      |
      +--> subnets/{registration,weights}
      +--> epoch/{run_epoch,math}
      +--> stubs/swap_stub boundaries
      |
      v
deterministic emission + consensus tests
```

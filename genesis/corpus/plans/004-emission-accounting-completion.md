# 004 - Emission Accounting Completion

## Purpose / Big Picture

The Yuma Consensus emission mechanism is wired into pallet-game-solver but the
coinbase logic assumes a root network and AMM that are disabled in stage-0. This
plan completes the single-token emission path so that quality-weighted token
distribution actually works on a local devnet, closing stage-0 exit criteria
6, 7, and 12.

## Context and Orientation

Stage-0 exit criteria still requiring runtime verification:
- **Criterion 6:** Two validators score the same miner identically (INV-003).
- **Criterion 7:** Yuma-style economics distribute emissions by quality.
- **Criterion 12:** Emission accounting stays green + all invariants pass.

The emission rewrite was identified in `THEORY.MD` as necessary because the
coinbase logic assumes root network + AMM. The decision was made to use a
single-token model (no dual Alpha/TAO).

Current state: `pallet-game-solver` has emission code but it has not been
verified end-to-end on a running chain with real validator weight submissions.

## Architecture

```
Validator submits weights (commit-reveal v2)
    → pallet-game-solver processes weight matrix
    → Yuma Consensus computes dividends (substrate_fixed v0.6.0)
    → Coinbase distributes single-token emissions
    → Emission accounting storage updated
```

## Progress

- [x] (pre-satisfied) M1. Yuma Consensus logic present in pallet
  - Surfaces: `crates/myosu-chain/pallets/game-solver/src/`
  - Evidence: `cargo check -p pallet-game-solver`

### Milestone 2: Complete single-token coinbase path

- [ ] M2. Rewrite coinbase to use single-token emission without root network
  - Surfaces: `crates/myosu-chain/pallets/game-solver/src/coinbase/`
  - What exists after: Coinbase distributes emissions proportional to Yuma
    dividends using a single token. No AMM, no root network, no dual-token.
  - Why now: Emissions are the economic engine. Without them, stage-0 is
    incomplete.
Proof command: `cargo test -p pallet-game-solver --quiet -- emission`
  - Tests: `cargo test -p pallet-game-solver --quiet -- coinbase`

### Milestone 3: Prove cross-validator determinism (INV-003)

- [ ] M3. Write integration test proving two validators agree within epsilon
  - Surfaces: `crates/myosu-validator/tests/`, `crates/myosu-games/src/traits.rs`
  - What exists after: Test instantiates two validators, feeds identical game
    state + strategy, asserts scores match within 1e-6.
  - Why now: INV-003 is the hardest invariant to prove. Must be proven before
    claiming stage-0 complete.
Proof command: `SKIP_WASM_BUILD=1 cargo test -p myosu-validator --quiet -- determinism`
  - Tests: `SKIP_WASM_BUILD=1 cargo test -p myosu-validator --quiet -- determinism`

### Milestone 4: End-to-end emission test on local devnet

- [ ] M4. Script that boots devnet, registers miner + validator, submits weights,
  and verifies emission distribution
  - Surfaces: `tests/e2e/emission_flow.sh` (new)
  - What exists after: Reproducible script proving the full emission cycle.
  - Why now: This is the stage-0 completion proof.
Proof command: `bash tests/e2e/emission_flow.sh`
  - Tests: `bash tests/e2e/emission_flow.sh`

## Surprises & Discoveries

- `substrate_fixed` pinned to encointer fork v0.6.0 is load-bearing for Yuma
  output determinism. Upgrading it would break INV-003.
- The emission code has TODO comments about storage being "used for both
  alpha/tao" (`pallet-game-solver/src/lib.rs:2267`). This dual-use must be
  resolved for single-token clarity.

## Decision Log

- Decision: Single-token emission (no AMM pools).
  - Why: AMM pools add massive complexity for zero stage-0 value. Already
    decided in THEORY.MD.
  - Failure mode: Need dual-token later and have to rewrite.
  - Mitigation: Keep token abstraction clean so dual-token can be added later.
  - Reversible: yes (additive change to add second token)

- Decision: Prove INV-003 with unit test before live chain test.
  - Why: Unit tests are faster to iterate on. Live chain test is the final gate.
  - Failure mode: Unit test passes but live chain diverges due to runtime state.
  - Mitigation: Both tests required.
  - Reversible: yes

## Validation and Acceptance

1. Single-token coinbase distributes emissions in pallet tests.
2. Two validators agree within epsilon on identical inputs (unit test).
3. End-to-end script proves emission flow on local devnet.

## Outcomes & Retrospective
_Updated after milestones complete._

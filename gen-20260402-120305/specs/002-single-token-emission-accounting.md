# Specification: Single-Token Emission Accounting

Source: Genesis Plan 004 (Emission Accounting Completion), SPEC.md stage-0 exit criteria 6/7/12
Status: Draft
Depends-on: 001-chain-runtime-reduction

## Purpose

The Yuma Consensus mechanism is wired into pallet-game-solver but the coinbase
logic assumes a root network and AMM that are disabled in stage-0. Without a
working single-token emission path, the chain cannot distribute economic rewards
proportional to measured solver quality — which is the core value proposition of
the protocol. This spec closes the three remaining stage-0 exit criteria that
require runtime verification: cross-validator determinism (criterion 6),
quality-weighted emission distribution (criterion 7), and emission accounting
integrity (criterion 12).

## Whole-System Goal

Current state: Yuma Consensus logic exists in pallet-game-solver and computes
dividend weights using `substrate_fixed` v0.6.0 fixed-point arithmetic. The
coinbase function references root network and AMM paths that are disabled in
stage-0, preventing actual token distribution. Validators submit weights via
commit-reveal v2, but no test proves that two independent validators agree on
scores or that emissions flow to miners.

This spec adds: A single-token coinbase path that distributes emissions
proportional to Yuma Consensus dividends without requiring a root network or
AMM. Verification that two validators produce identical scores on the same input
within epsilon. An end-to-end proof that the full emission flow operates on a
local devnet.

If all ACs land: Miners receive token emissions proportional to their measured
solver quality, two validators agree within epsilon on identical inputs, and the
emission accounting invariants hold on a running chain.

Still not solved here: Multi-node emission distribution, dual-token economics,
cross-subnet emission routing, and live devnet persistence.

## Scope

In scope:
- Rewriting the coinbase function for single-token emission without root
  network or AMM dependencies
- Proving cross-validator determinism (INV-003) with identical inputs
- Proving end-to-end emission flow on a local devnet
- Ensuring emission accounting storage is updated correctly after distribution

Out of scope:
- Dual-token (Alpha/TAO) economics
- Cross-subnet emission routing
- AMM or swap mechanics
- Multi-node emission verification (see 006-multi-node-devnet)
- Validator incentive design beyond quality-weighted emissions

## Current State

Yuma Consensus logic is present in pallet-game-solver and computes a weight
matrix from validator submissions. The `substrate_fixed` crate (encointer fork
v0.6.0) provides deterministic fixed-point arithmetic for dividend calculation.
Commit-reveal v2 (hash-based) is the active weight submission mechanism.

The coinbase function in pallet-game-solver contains paths that reference root
network lookup and AMM-based token conversion, both of which are disabled or
stubbed in stage-0. No integration test currently proves that emissions are
distributed or that validators agree on scores.

The validator binary at `crates/myosu-validator/` computes exploitability scores
deterministically using the game traits from `crates/myosu-games/src/traits.rs`.
INV-003 (Game Verification Determinism) is supported by the code design but has
not been proven by test.

## What Already Exists

| Sub-problem | Existing code / flow | Reuse / extend / replace | Why |
|---|---|---|---|
| Yuma Consensus computation | pallet-game-solver weight processing | Reuse | Core algorithm is correct |
| Fixed-point arithmetic | `substrate_fixed` v0.6.0 (encointer fork) | Reuse | Determinism depends on this exact pin |
| Commit-reveal v2 | pallet-game-solver weight submission | Reuse | Hash-based submission works |
| Validator scoring | `crates/myosu-validator/src/validation.rs` | Reuse | Deterministic exploitability measurement |
| SwapInterface stub | `crates/myosu-chain/pallets/swap/` | Reuse | No-op satisfies call sites |
| Coinbase function | pallet-game-solver coinbase logic | Replace | Assumes root network + AMM |
| Stage-0 flow tests | `pallet-game-solver -- stage_0` | Extend | Add emission distribution assertions |

## Non-goals

- Designing a long-term token economic model.
- Supporting multiple subnets with cross-subnet emission splits.
- Implementing AMM or swap functionality.
- Changing the Yuma Consensus algorithm itself.
- Supporting validator slashing or penalties.

## Behaviors

The coinbase function distributes a per-epoch emission amount to neurons on a
subnet proportional to their Yuma Consensus dividend weights. The distribution
uses only a single token type without requiring root network lookup or AMM
conversion. Emission amounts are written to on-chain storage and are queryable.

When two validators independently score the same miner's strategy using
identical game parameters, their computed exploitability scores agree within
epsilon (1e-6). This determinism is a consequence of the fixed-point arithmetic
in `substrate_fixed` and the deterministic game evaluation in the validator
binary.

On a local devnet, the full emission flow operates end-to-end: a miner
registers on a subnet, trains and serves a strategy, a validator scores the
strategy and submits weights via commit-reveal v2, the chain processes the weight
matrix through Yuma Consensus, and the coinbase distributes emissions to the
miner proportional to the computed quality.

Emission accounting storage reflects the distributed amounts accurately after
each epoch. No tokens are created or destroyed outside the coinbase path.

## Acceptance Criteria

- The coinbase distributes single-token emissions proportional to Yuma Consensus
  dividends in pallet tests, without referencing root network or AMM logic.
- Two validators produce identical scores (within 1e-6 epsilon) when given
  identical miner strategy inputs, verified in an isolated test.
- An end-to-end script boots a local devnet, registers a miner and validator,
  submits weights, and observes emission distribution to the miner.
- Emission accounting storage balances are consistent after distribution — no
  token leakage or creation outside the coinbase.
- The `substrate_fixed` v0.6.0 pin is preserved and not upgraded.

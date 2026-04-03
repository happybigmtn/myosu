# Specification: Integration Test Harness

Source: Genesis Plan 006 (Integration Test Harness), ASSESSMENT.md test gaps
Status: Draft
Depends-on: 002-single-token-emission-accounting

## Purpose

Stage-0 has unit tests, property-based tests, and smoke tests, but no
integration test proves the full chain-to-gameplay loop in a single reproducible
run. The stage-0 exit criteria require runtime verification of cross-validator
determinism, emission distribution, and the complete train-score-emit-play
cycle. Without a CI-gatable integration test harness, these criteria can only
be verified manually through operator playbooks — which is fragile, slow, and
does not prevent regressions. A reproducible harness replaces manual playbook
verification with automated proof.

## Whole-System Goal

Current state: `myosu-play --smoke-test` proves the play binary loads and
renders. `pallet-game-solver -- stage_0` proves pallet logic in isolation.
The repo now also carries shell-based proofs for the full local loop, validator
determinism, and emission flow, plus a Rust `stage0_local_loop` integration
test. The remaining work is keeping those proofs truthful, reproducible, and
green in CI rather than inventing the harness from scratch.

This spec adds: Shell-based integration test scripts that boot a local devnet,
exercise the full loop, and assert on observable outcomes. A CI job that runs
these tests after the chain-core job passes.

If all ACs land: A single script proves the complete stage-0 local loop in
under 5 minutes on a clean checkout. Cross-validator determinism (INV-003) is
proven by automated test. The integration tests run in CI and prevent
regressions.

Still not solved here: Multi-node integration testing, performance benchmarking,
load testing, and chaos/fault-injection testing.

## Scope

In scope:
- Helper scripts for managing a local devnet lifecycle (start, stop, wait for
  block)
- Full local loop integration test: chain boots, miner trains and serves,
  validator scores and submits weights, play runs a hand
- Cross-validator determinism test proving INV-003
- CI job for integration tests

Out of scope:
- Multi-node integration tests (see 006-multi-node-devnet)
- Performance or load testing
- Fault injection or chaos testing
- Testing the Python research stack (see 010-python-research-quality-gates)
- Testing chain runtime upgrades

## Current State

The CI pipeline now includes an `integration-e2e` job after `chain-core`. The
repo carries `tests/e2e/helpers/start_devnet.sh`, `stop_devnet.sh`, and
`wait_for_block.sh`, plus `tests/e2e/local_loop.sh`,
`tests/e2e/validator_determinism.sh`, and `tests/e2e/emission_flow.sh` as the
stage-0 proof surfaces.

The operator bundle at `.github/scripts/prepare_operator_network_bundle.sh`
contains startup scripts that could serve as a reference for devnet lifecycle
management in tests.

Miner training is the slowest step in the loop (~30 seconds with smallest
configuration). The chain binary takes 2-3 minutes to compile with
`fast-runtime`. Test design must account for these timings.

## What Already Exists

| Sub-problem | Existing code / flow | Reuse / extend / replace | Why |
|---|---|---|---|
| Smoke test | `myosu-play --smoke-test` | Reuse | Already proves play binary loads |
| Pallet tests | `pallet-game-solver -- stage_0` | Reuse | Already proves pallet logic |
| Chain boot test | `myosu-chain --test stage0_local_loop` | Reuse | Already proves the chain boot plus the consolidated local-loop contract |
| Operator bundle scripts | `.github/scripts/prepare_operator_network_bundle.sh` | Reference | Contains devnet startup patterns |
| E2E proof scripts | `tests/e2e/local_loop.sh`, `tests/e2e/validator_determinism.sh`, `tests/e2e/emission_flow.sh` | Reuse and harden | The cross-binary proof paths already exist |
| CI pipeline | `.github/workflows/ci.yml` | Reuse and harden | `integration-e2e` already gates the proof scripts after `chain-core` |
| Validator determinism code | `crates/myosu-validator/src/validation.rs` | Reuse | Scoring is already deterministic by design and has a bounded in-crate proof |

## Non-goals

- Replacing existing unit tests or smoke tests with integration tests.
- Testing inherited chain fork functionality beyond the stage-0 surface.
- Achieving integration test execution time under 1 minute (the full loop
  inherently takes minutes).
- Testing against a persistent or remote devnet.

## Behaviors

A set of helper functions manage the local devnet lifecycle: starting a chain
node with fast-runtime, waiting for block production, and cleanly stopping the
node after tests complete. These helpers handle process lifecycle, port
allocation, and cleanup on both success and failure.

The full local loop test executes the stage-0 cycle end-to-end: the chain
produces blocks, a miner registers and trains a strategy with the smallest
viable configuration, the miner serves the strategy via HTTP, a validator
queries the miner and computes an exploitability score, the validator submits
weights to the chain, and the play binary runs a hand against the served
strategy. The test asserts on observable outcomes at each stage.

The cross-validator determinism test runs two independent validator scoring
passes against the same miner strategy and asserts that the computed
exploitability scores agree within epsilon (1e-6), proving INV-003.

The integration test CI job runs after the chain-core job passes, uses a
pre-built or cached chain binary, and fails the pipeline if any integration
test fails.

## Acceptance Criteria

- A local loop integration test script passes on a clean checkout, proving the
  complete chain-to-gameplay cycle.
- A cross-validator determinism test proves that two independent scoring passes
  agree within 1e-6 epsilon on identical inputs.
- The integration test suite completes in under 5 minutes.
- A CI job runs the integration tests and gates the pipeline on their results.
- Test scripts handle cleanup on both success and failure, leaving no orphan
  processes.

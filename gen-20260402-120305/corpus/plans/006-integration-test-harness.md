# 006 - Integration Test Harness

## Purpose / Big Picture

Stage-0 has smoke tests and unit tests but no integration test proving the full
chain -> miner -> validator -> play loop in a single automated run. This plan
builds the harness that replaces manual playbook verification with reproducible
CI-gatable tests. Carries forward prior plan 013.

## Context and Orientation

Current test coverage:
- `myosu-play --smoke-test`: Proves play loads artifacts and renders
- `pallet-game-solver -- stage_0`: Proves pallet logic in isolation
- `myosu-chain --test stage0_local_loop`: Proves chain boots
- Missing: No test connects miner training -> validator scoring -> play

## Architecture

```
tests/e2e/
├── local_loop.sh              # Full loop: chain + miner + validator + play
├── validator_determinism.sh   # INV-003 proof (two validators)
├── emission_flow.sh           # Emission distribution
└── helpers/
    └── chain_helpers.sh       # Shared chain start/stop/wait
```

## Progress

- [x] (pre-satisfied) M1. Smoke test infrastructure exists
  - Surfaces: `crates/myosu-play/src/main.rs`
Proof command: `SKIP_WASM_BUILD=1 cargo run -p myosu-play --quiet -- --smoke-test`

### Milestone 2: Chain helper scripts

- [ ] M2. Write helper scripts for managing a local devnet
  - Surfaces: `tests/e2e/helpers/chain_helpers.sh` (new)
  - What exists after: Functions `start_devnet()`, `stop_devnet()`,
    `wait_for_block()` that manage a local chain process.
  - Why now: Every integration test needs a running chain.
Proof command: `bash tests/e2e/helpers/chain_helpers.sh --self-test`
  - Tests: Chain starts and produces at least 1 block

### Milestone 3: Full local loop integration test

- [ ] M3. Write test that boots chain, runs miner, runs validator, runs play
  - Surfaces: `tests/e2e/local_loop.sh` (new)
  - What exists after: Single script proving the full stage-0 loop in <5 min.
  - Why now: THE proof that stage-0 works end-to-end.
Proof command: `bash tests/e2e/local_loop.sh`
  - Tests: Script exits 0 with structured output

### Milestone 4: Validator determinism test (INV-003)

- [ ] M4. Prove two validators agree on identical inputs
  - Surfaces: `tests/e2e/validator_determinism.sh` (new)
  - What exists after: Script runs two validators against same miner, compares
    scores, asserts within epsilon (1e-6).
  - Why now: INV-003 is the hardest invariant.
Proof command: `bash tests/e2e/validator_determinism.sh`
  - Tests: Max divergence < 1e-6

### Milestone 5: CI integration

- [ ] M5. Add e2e test job to GitHub Actions
  - Surfaces: `.github/workflows/ci.yml`
  - What exists after: CI includes `integration` job after chain-core passes.
  - Why now: Tests are only valuable if they run automatically.
Proof command: `gh run list --branch trunk --limit 1`
  - Tests: Integration CI job green

## Surprises & Discoveries

- Chain binary takes 2--3 minutes to compile with `fast-runtime`. E2E tests need
  aggressive caching or pre-built binary.
- Miner training is the slowest step (~30s for minimal blueprint). Tests should
  use smallest possible configuration.

## Decision Log

- Decision: Shell scripts (not Rust test harness) for integration tests.
  - Why: Integration tests orchestrate multiple processes. Shell is natural.
  - Failure mode: Shell tests harder to maintain.
  - Mitigation: <200 lines each, shared helpers.
  - Reversible: yes

## Validation and Acceptance

1. `bash tests/e2e/local_loop.sh` passes on clean checkout.
2. `bash tests/e2e/validator_determinism.sh` proves INV-003.
3. Integration tests run in CI.

## Outcomes & Retrospective
_Updated after milestones complete._

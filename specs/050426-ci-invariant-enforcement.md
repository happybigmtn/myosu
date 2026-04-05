# Specification: CI Pipeline and Invariant Enforcement

## Objective

Define the structure, sequencing, and correctness guarantees of the myosu CI
pipeline and the six project invariants it enforces. This spec serves as the
canonical reference for what the pipeline must check, how jobs relate to each
other, and where known gaps exist.

## Evidence Status

All claims below are verified against the current codebase as of 2026-04-05.

| Claim | Source | Status |
|-------|--------|--------|
| 10 CI jobs with documented sequencing | `.github/workflows/ci.yml` | Verified |
| INV-004 cargo-tree gate in `active-crates` | `.github/workflows/ci.yml:98-111` | Verified |
| INV-003 determinism proof in `integration-e2e` | `tests/e2e/validator_determinism.sh` | Verified |
| Workspace clippy denies arithmetic-side-effects, expect-used, indexing-slicing, unwrap-used | `Cargo.toml:25-33` | Verified |
| 7 suppressed RUSTSECs from inherited Substrate stack | `.github/workflows/ci.yml:258-274` | Verified |
| 4 E2E scripts exist (local_loop, validator_determinism, emission_flow, two_node_sync) | `tests/e2e/` | Verified |
| `actions/checkout@v6` used without SHA pin | `.github/workflows/ci.yml` (all jobs) | Verified |
| `SKIP_WASM_BUILD=1` set on off-chain jobs | `.github/workflows/ci.yml` (active-crates, chain-core, integration-e2e, operator-network, chain-clippy) | Verified |

## Pipeline Architecture

### Jobs (10 total)

| Job | Purpose | Runner |
|-----|---------|--------|
| `repo-shape` | Validates workspace structure against doctrine | ubuntu-latest |
| `python-research-qa` | Ruff lint + pytest on research code (numpy, pytest, ruff) | ubuntu-latest |
| `active-crates` | Cargo check, focused tests, INV-004 boundary, full test suite, clippy, rustfmt | ubuntu-latest |
| `chain-core` | Runtime/pallet/node check, stage-0 pallet tests, runtime tests | ubuntu-latest |
| `integration-e2e` | local_loop.sh and validator_determinism.sh proofs | ubuntu-latest |
| `doctrine` | Canonical spec integrity verification | ubuntu-latest |
| `dependency-audit` | cargo-audit with managed RUSTSEC allowlist | ubuntu-latest |
| `plan-quality` | Genesis plan quality checks (milestones, proof commands) | ubuntu-latest |
| `operator-network` | Operator bundle and named-network bootstrap packaging | ubuntu-latest |
| `chain-clippy` | Strict clippy on runtime, pallet, and node packages | ubuntu-latest |

### Sequencing

```
repo-shape
  |
  +---> python-research-qa
  +---> active-crates
  +---> doctrine
  +---> dependency-audit
  +---> plan-quality
  +---> operator-network
  +---> chain-clippy
  +---> chain-core ---> integration-e2e
```

`repo-shape` is the universal gate. All other jobs depend on it. `integration-e2e`
depends on `chain-core`. The remaining 7 jobs run in parallel after `repo-shape`.

### Concurrency

The pipeline uses concurrency groups keyed on workflow name + PR number (or ref),
with `cancel-in-progress: true`. This prevents stale runs from consuming runners.

## Invariants

Six project invariants are defined. The pipeline enforces a subset directly; the
remainder rely on structural or manual verification.

| ID | Name | CI Enforcement |
|----|------|----------------|
| INV-001 | Structured closure honesty | `doctrine` job (spec integrity) |
| INV-002 | Proof honesty (no false greens) | Structural (tests must fail when broken) |
| INV-003 | Game verification determinism (epsilon < 1e-6) | `integration-e2e` via `validator_determinism.sh` |
| INV-004 | Solver-gameplay separation | `active-crates` via `cargo tree` gate |
| INV-005 | Plan/land coherence | `plan-quality` + `doctrine` jobs |
| INV-006 | Robopoker fork coherence | Not directly enforced in CI |

### INV-003: Determinism Gate

The `validator_determinism.sh` script runs two independent validator instances
against the same game state and asserts their scores agree within epsilon < 1e-6.
This runs in the `integration-e2e` job, which has a 10-minute timeout.

### INV-004: Solver-Gameplay Separation Gate

The `active-crates` job runs `cargo tree` on both `myosu-play` and `myosu-miner`
and fails if either depends on the other. This is a hard structural boundary:
gameplay code must never import solver logic, and vice versa.

### Workspace Clippy Lints

The workspace `Cargo.toml` denies four clippy lints at the workspace level:

- `arithmetic-side-effects = "deny"` -- prevents unchecked arithmetic in on-chain code
- `expect-used = "deny"` -- no panicking expect calls
- `indexing-slicing = "deny"` -- no unchecked array/slice indexing
- `unwrap-used = "deny"` -- no panicking unwrap calls

These apply to all workspace members and are enforced by both the `active-crates`
clippy step and the `chain-clippy` job.

### Dependency Audit Allowlist

Seven RUSTSECs are suppressed, all from the inherited Substrate/opentensor fork:

- `ring` via libp2p QUIC in the node stack
- `wasmtime` via the inherited executor/runtime stack
- `tracing-subscriber 0.2.25` via the inherited runtime graph

The audit runs with `-D warnings` so any non-ignored advisory fails the gate.

## E2E Test Scripts

| Script | Purpose | CI Job |
|--------|---------|--------|
| `local_loop.sh` | Full miner/validator/play proof on live devnet | `integration-e2e` |
| `validator_determinism.sh` | Cross-validator scoring agreement | `integration-e2e` |
| `emission_flow.sh` | On-chain emission distribution proof | Not wired into CI |
| `two_node_sync.sh` | Named-network peer discovery proof | Not wired into CI |

## Acceptance Criteria

- All 10 jobs pass on every PR merge to `trunk`.
- INV-003 determinism epsilon remains below 1e-6 as verified by `validator_determinism.sh`.
- INV-004 cargo-tree gate rejects any PR that introduces a dependency between `myosu-play` and `myosu-miner`.
- Workspace clippy denials (arithmetic-side-effects, expect-used, indexing-slicing, unwrap-used) produce hard failures, not warnings.
- `cargo audit` fails on any advisory not in the managed allowlist.
- `repo-shape` gates all downstream jobs -- no job runs if workspace structure is invalid.
- Concurrency groups prevent stale CI runs from completing.

## Verification

```bash
# Run the full CI pipeline locally (subset)
SKIP_WASM_BUILD=1 cargo check -p myosu-games -p myosu-play -p myosu-miner
SKIP_WASM_BUILD=1 cargo clippy --all-targets -- -D warnings

# INV-004: verify solver-gameplay boundary
cargo tree -p myosu-play --edges normal | grep -c myosu-miner  # must be 0
cargo tree -p myosu-miner --edges normal | grep -c myosu-play  # must be 0

# INV-003: run determinism proof
bash tests/e2e/validator_determinism.sh

# Dependency audit
cargo audit -D warnings \
  --ignore RUSTSEC-2025-0009 \
  --ignore RUSTSEC-2025-0055 \
  --ignore RUSTSEC-2023-0091 \
  --ignore RUSTSEC-2024-0438 \
  --ignore RUSTSEC-2025-0118 \
  --ignore RUSTSEC-2026-0020 \
  --ignore RUSTSEC-2026-0021

# Repo shape
bash .github/scripts/check_stage0_repo_shape.sh
```

## Open Questions

1. **SHA-pinned actions**: All `actions/checkout@v6` references use a tag, not a
   SHA hash. This is a low-severity supply-chain risk. Should these be pinned to
   full SHAs with version comments?

2. **Unwired E2E scripts**: `emission_flow.sh` and `two_node_sync.sh` exist but
   are not wired into any CI job. Should they be added to `integration-e2e` or a
   separate job? What is the expected runtime cost?

3. **INV-006 enforcement**: Robopoker fork coherence has no automated CI gate.
   Should a diff-based or hash-based check be added?

4. **Multi-node consensus finality**: No test validates that two nodes reach
   consensus and finalize blocks. `two_node_sync.sh` checks peer discovery but
   not finality. Is this an acceptable gap for stage 0?

5. **Fuzzing and adversarial inputs**: No fuzzing or adversarial input testing
   exists for wire codecs (`myosu-games-poker` serialization). Should
   `cargo-fuzz` targets be added?

6. **Runtime upgrade/migration tests**: No tests verify that a runtime upgrade
   preserves storage and state. Should these be added before any mainnet
   deployment?

7. **Miner training convergence**: The pipeline tests that miners run, but does
   not assert training quality or convergence. Is this intentionally deferred?

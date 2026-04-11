# Specification: CI Pipeline and Invariant Enforcement

## Objective

Define the structure, sequencing, and correctness guarantees of the myosu CI
pipeline and the six project invariants it enforces. This spec serves as the
canonical reference for what the pipeline must check, how jobs relate to each
other, and where known gaps exist.

## Evidence Status

All claims below are verified against the current codebase as of 2026-04-10.

| Claim | Source | Status |
|-------|--------|--------|
| 11 CI jobs with documented sequencing | `.github/workflows/ci.yml` | Verified |
| INV-004 cargo-tree gate in `active-crates` | `.github/workflows/ci.yml` | Verified |
| INV-003 determinism proof in `integration-e2e` | `tests/e2e/validator_determinism.sh` | Verified |
| Runtime migration smoke test runs in `chain-core` | `.github/workflows/ci.yml`, `crates/myosu-chain/node/src/chain_spec/devnet.rs` | Verified |
| Robopoker fork coherence has an advisory CI job | `.github/workflows/ci.yml`, `.github/scripts/check_robopoker_fork_status.sh` | Verified |
| Workspace clippy denies arithmetic-side-effects, expect-used, indexing-slicing, unwrap-used | `Cargo.toml` | Verified |
| 19 RUSTSECs are explicitly allowlisted in the current dependency-audit gate | `.github/workflows/ci.yml` | Verified |
| 7 runtime/E2E proofs are wired into `integration-e2e` | `.github/workflows/ci.yml`, `tests/e2e/` | Verified |
| Fast all-corpus research-game solver harness is wired into `active-crates` | `.github/workflows/ci.yml`, `tests/e2e/research_games_harness.sh` | Verified |
| All mutable GitHub Action refs are SHA-pinned and checkout disables credential persistence | `.github/workflows/ci.yml` | Verified |
| `SKIP_WASM_BUILD=1` is set on the off-chain jobs that should reuse the cached runtime wasm | `.github/workflows/ci.yml` | Verified |

## Pipeline Architecture

### Jobs (11 total)

| Job | Purpose | Runner |
|-----|---------|--------|
| `repo-shape` | Validates workspace structure against doctrine | ubuntu-latest |
| `robopoker-fork-coherence` | Advisory fork-divergence report for INV-006 | ubuntu-latest |
| `python-research-qa` | Ruff lint + pytest on research code (numpy, pytest, ruff) | ubuntu-latest |
| `active-crates` | Cargo check, focused tests, research-game corpus harness, INV-004 boundary, full test suite, clippy, rustfmt | ubuntu-latest |
| `chain-core` | Runtime/pallet/node check, pallet stage-0 tests, runtime tests, migration smoke | ubuntu-latest |
| `integration-e2e` | local_loop, sync, finality, restart, emission, determinism proofs | ubuntu-latest |
| `doctrine` | Canonical spec integrity verification | ubuntu-latest |
| `dependency-audit` | cargo-audit with managed RUSTSEC allowlist | ubuntu-latest |
| `plan-quality` | Genesis plan quality checks (milestones, proof commands) | ubuntu-latest |
| `operator-network` | Operator bundle and named-network bootstrap packaging | ubuntu-latest |
| `chain-clippy` | Strict clippy on runtime, pallet, and node packages | ubuntu-latest |

### Sequencing

```
repo-shape
  |
  +---> robopoker-fork-coherence
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
depends on `chain-core`. The remaining 8 jobs run in parallel after `repo-shape`.

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
| INV-006 | Robopoker fork coherence | Advisory `robopoker-fork-coherence` job |

### INV-003: Determinism Gate

The `validator_determinism.sh` script runs two independent validator instances
against the same game state and asserts their scores agree within epsilon < 1e-6.
This runs in the `integration-e2e` job, which currently has a 15-minute timeout
because it now covers seven shell proofs instead of the earlier two-proof slice.

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

Nineteen RUSTSECs are currently suppressed. This set includes inherited
Substrate/opentensor debt plus the already-deferred direct `bincode 1.x`
usage in the owned game crates. The gate still runs with `-D warnings`, so any
advisory outside the explicit allowlist fails CI.

The audit runs with `-D warnings` so any non-ignored advisory fails the gate.

## E2E Test Scripts

| Script | Purpose | CI Job |
|--------|---------|--------|
| `local_loop.sh` | Full miner/validator/play proof on live devnet | `integration-e2e` |
| `two_node_sync.sh` | Named-network peer discovery proof | `integration-e2e` |
| `four_node_finality.sh` | Four-authority GRANDPA finality proof | `integration-e2e` |
| `consensus_resilience.sh` | Restart/catch-up proof for the fourth authority | `integration-e2e` |
| `cross_node_emission.sh` | Cross-node emission/state agreement proof | `integration-e2e` |
| `validator_determinism.sh` | Cross-validator scoring agreement | `integration-e2e` |
| `emission_flow.sh` | On-chain emission distribution proof | `integration-e2e` |
| `bootstrap_manifest` | Rust-owned manifest for 22 research game identities, dedicated/portfolio route split, rule files, chain ids, and solver families | consumed by research harnesses |
| `research_games_harness.sh` | Fast all-corpus proof for 22 research game identities, dedicated solver roundtrips, dedicated `STRENGTH` roundtrips, the exact Kuhn benchmark roundtrip, portfolio solver roundtrips, and play surfaces | `active-crates` |
| `research_portfolio_harness.sh` | Fast 20-game portfolio checkpoint/query/response/scoring roundtrip, portfolio `STRENGTH` roundtrip, and offline validation proof plus scoped checkpoint/query mismatch rejection | `active-crates` via `research_games_harness.sh` |
| `research_strength_harness.sh` | Dedicated all-corpus rule-aware strength proof for portfolio, dedicated research, and exact Kuhn benchmark routes, including cross-game and malformed typed-query rejection | `active-crates` |
| `Clippy (game example binaries)` | Lints the example binaries that produce legacy bootstrap artifacts, strength queries, quality/latency budgets, checkpoints, roundtrips, and manifests | `active-crates` |

## Acceptance Criteria

- All 11 jobs pass on every PR merge to `trunk`, except the explicitly advisory
  `robopoker-fork-coherence` job which may report drift without blocking the run.
- INV-003 determinism epsilon remains below 1e-6 as verified by `validator_determinism.sh`.
- INV-004 cargo-tree gate rejects any PR that introduces a dependency between `myosu-play` and `myosu-miner`.
- Workspace clippy denials (arithmetic-side-effects, expect-used, indexing-slicing, unwrap-used) produce hard failures, not warnings.
- `cargo audit` fails on any advisory not in the managed allowlist.
- `repo-shape` gates all downstream jobs -- no job runs if workspace structure is invalid.
- Concurrency groups prevent stale CI runs from completing.

## Verification

```bash
# Run the full CI pipeline locally (subset)
SKIP_WASM_BUILD=1 cargo check \
  -p myosu-games \
  -p myosu-games-kuhn \
  -p myosu-games-poker \
  -p myosu-games-liars-dice \
  -p myosu-games-portfolio \
  -p myosu-tui \
  -p myosu-play \
  -p myosu-chain-client \
  -p myosu-miner \
  -p myosu-validator
SKIP_WASM_BUILD=1 cargo clippy -p myosu-games-portfolio -- -D warnings

# Research-game corpus proof
bash tests/e2e/research_games_harness.sh

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
  --ignore RUSTSEC-2026-0021 \
  --ignore RUSTSEC-2025-0141 \
  --ignore RUSTSEC-2024-0388 \
  --ignore RUSTSEC-2025-0057 \
  --ignore RUSTSEC-2024-0384 \
  --ignore RUSTSEC-2020-0168 \
  --ignore RUSTSEC-2022-0061 \
  --ignore RUSTSEC-2024-0436 \
  --ignore RUSTSEC-2024-0370 \
  --ignore RUSTSEC-2025-0010 \
  --ignore RUSTSEC-2021-0127 \
  --ignore RUSTSEC-2026-0002 \
  --ignore RUSTSEC-2024-0442

# Repo shape
bash .github/scripts/check_stage0_repo_shape.sh
```

## Open Questions

1. **Advisory vs blocking fork coherence**: Should `robopoker-fork-coherence`
   stay `continue-on-error`, or should INV-006 become a hard gate once the fork
   changelog and divergence policy settle?

2. **`dtolnay/rust-toolchain` handling**: `zizmor --min-severity medium` is
   currently clean, but raw `zizmor` still reports low-severity
   `superfluous-actions` advisories on this helper action. Should the workflow
   switch to explicit `rustup` shell steps or carry an explicit allowlist?

3. **Integration runtime budget**: `integration-e2e` now runs seven proofs
   under a 15-minute budget. Should the job stay monolithic for artifact reuse,
   or split once additional multi-node/operator proofs land?

4. **Fuzzing and adversarial inputs**: No `cargo-fuzz` targets exist even though
   property-style codec proofs now exist in unit tests. Should true fuzz
   harnesses be added for wire codecs and chain-facing parsers?

5. **Weight-submission isolation**: The validator weight-submission path is
   exercised by `local_loop.sh`, not by a dedicated E2E proof. Is that broad
   coverage sufficient, or should weight submission get its own focused script?

6. **Cross-architecture determinism**: The multi-node proofs are currently
   single-architecture local checks. Should CI add an ARM/x86 split before the
   project treats emission agreement as hardware-agnostic?

7. **Runtime upgrade/migration coverage depth**: The new devnet smoke test
   proves a fresh-genesis upgrade path. Should follow-on coverage add a
   non-empty-state migration or a multi-node upgrade rehearsal?

8. **Miner training convergence**: The pipeline tests that miners run, but does
   not assert training quality or convergence. Is this intentionally deferred?

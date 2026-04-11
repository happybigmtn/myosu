# Specification: CI Pipeline and Quality Gates

## Objective

Define the current CI pipeline structure, the quality gates it enforces, and the invariants it protects. This spec covers the GitHub Actions workflow, E2E test harnesses, and the relationship between CI jobs and system invariants.

## Evidence Status

### Verified facts (code-grounded)

#### Workflow structure (`.github/workflows/ci.yml`)

- Triggers: push and PR to `trunk`/`main`, plus `workflow_dispatch` — `ci.yml:3-12`
- Concurrency: cancel-in-progress per workflow+PR/ref — `ci.yml:14-16`
- Permissions: `contents: read` only — `ci.yml:18-19`

#### CI Jobs (11 total)

| Job | Depends On | Purpose |
|-----|-----------|---------|
| `repo-shape` | — | Stage-0 workspace and doctrine verification via `check_stage0_repo_shape.sh` |
| `robopoker-fork-coherence` | repo-shape | Fork divergence report (advisory, `continue-on-error: true`) |
| `python-research-qa` | repo-shape | Ruff lint + pytest on Python research files (Python 3.13) |
| `active-crates` | repo-shape | Cargo check/test/clippy/fmt for 11 active crates |
| `chain-core` | repo-shape | Runtime check, pallet tests, migration smoke test |
| `integration-e2e` | chain-core | 7 E2E integration proofs (15-min timeout) |
| `doctrine` | repo-shape | Canonical spec integrity via `check_doctrine_integrity.sh` |
| `dependency-audit` | repo-shape | `cargo audit` with 19-advisory allowlist |
| `plan-quality` | repo-shape | Genesis plan milestone/proof command verification |
| `operator-network` | repo-shape | Operator bundle and fresh-machine proof |
| `chain-clippy` | repo-shape | Chain-specific clippy (runtime, pallet, node) |

#### Active-crates job detail

- 11 crates checked: myosu-games, myosu-games-kuhn, myosu-games-poker, myosu-games-liars-dice, myosu-games-portfolio, myosu-games-canonical, myosu-tui, myosu-play, myosu-chain-client, myosu-miner, myosu-validator — `ci.yml:102-113`
- Property tests: serialization roundtrip (games), all Kuhn tests, portfolio core, canonical playtrace, TUI shell state — `ci.yml:117-124`
- Smoke tests: poker preflop→flop, Kuhn smoke — `ci.yml:122-124`
- Canonical manifest gate: 10 games, 10 snapshot=ok — `ci.yml:128-135`
- E2E harnesses: canonical_ten_play, research_play, research_games (22 games), research_strength — `ci.yml:136-143`
- INV-004 boundary: cargo tree checks both directions (play→miner, miner→play) — `ci.yml:145-158`
- Full test suite for all 11 crates — `ci.yml:160-173`
- Clippy with `-D warnings` for all 11 crates + bootstrap game examples — `ci.yml:175-200`
- Rustfmt with edition 2024 check — `ci.yml:202-216`
- Environment: `SKIP_WASM_BUILD=1` — `ci.yml:83`

#### Chain-core job detail

- `cargo check -p myosu-chain-runtime --features fast-runtime` — `ci.yml:247`
- `cargo check -p pallet-game-solver` — `ci.yml:250`
- `cargo check -p myosu-chain --features fast-runtime` — `ci.yml:253`
- Stage-0 pallet tests: `cargo test -p pallet-game-solver --quiet -- stage_0` — `ci.yml:256`
- Runtime tests: `cargo test -p myosu-chain-runtime --quiet` — `ci.yml:259`
- WASM build for migration test: `cargo build -p myosu-chain-runtime --features fast-runtime` — `ci.yml:262`
- Migration smoke test: `devnet_runtime_upgrade_smoke_test_passes_on_fresh_genesis` — `ci.yml:265`

#### Integration-e2e job detail (7 proofs)

- `local_loop.sh` — single-node block production — `ci.yml:297`
- `two_node_sync.sh` — block propagation between nodes — `ci.yml:300`
- `four_node_finality.sh` — GRANDPA finality with 4 authorities — `ci.yml:303`
- `consensus_resilience.sh` — recovery after authority restart — `ci.yml:306`
- `cross_node_emission.sh` — emission agreement across nodes — `ci.yml:309`
- `validator_determinism.sh` — identical scoring (INV-003) — `ci.yml:312`
- `emission_flow.sh` — emission accounting integrity — `ci.yml:315`
- Timeout: 15 minutes — `ci.yml:272`
- Depends on `chain-core` (must pass first) — `ci.yml:270`
- Rust target: `wasm32v1-none` (not `wasm32-unknown-unknown`) — `ci.yml:284`

#### Dependency audit

- `cargo audit -D warnings` with 19 `--ignore` entries — `ci.yml:356-376`
- Advisory IDs span RUSTSEC-2020 through RUSTSEC-2026 — `ci.yml:358-376`
- Allowlist aligned with WORKLIST.md SEC-001 documentation — `ci.yml:350-355`

#### Actions pinning

- `actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd` (v6) — all jobs
- `actions/setup-python@a26af69be951a213d495a4c3e4e4022e16d87065` (v5) — python-research-qa
- `dtolnay/rust-toolchain@29eef336d9b2848a0b548edc03f92a220660cdb8` (stable) — all Rust jobs
- `Swatinem/rust-cache@e18b497796c12c097a38f9edb9d0641fb99eee32` (v2) — Rust jobs
- `taiki-e/install-action@2e5e45e985ec9321fdd2b05aa8de0872b125c29f` — cargo-audit install
- All pinned to full SHA with version comments — `ci.yml` throughout
- `persist-credentials: false` on all checkouts — `ci.yml` throughout

#### Invariant enforcement in CI

| Invariant | CI Enforcement |
|-----------|---------------|
| INV-003 (validator determinism) | `validator_determinism.sh` E2E proof |
| INV-004 (solver-gameplay separation) | `cargo tree` boundary check in active-crates |
| INV-006 (robopoker fork coherence) | `check_robopoker_fork_status.sh` (advisory) |

### Recommendations (intended system)

- Plan 002 proposes `tests/e2e/promotion_manifest.sh` as a new CI gate
- Plan 008 proposes reducing the 19-advisory allowlist through triage
- CI should eventually enforce INV-001 (structured closure honesty) and INV-005 (plan/land coherence) — not currently automated

### Hypotheses / unresolved questions

- Whether the 15-minute E2E timeout is sufficient for all 7 proofs on CI runners
- Whether `chain-clippy` job should block the integration-e2e job or run in parallel
- Whether `wasm32v1-none` vs `wasm32-unknown-unknown` target difference between E2E and chain-core jobs is intentional

## Acceptance Criteria

- All 11 CI jobs pass on the trunk branch
- `repo-shape` validates that required files and directory structure exist
- `active-crates` checks, tests, clips, and formats 11 crates without errors or warnings
- Canonical manifest produces exactly 10 games with 10 snapshot=ok
- INV-004 cargo tree check passes in both directions
- All 7 E2E integration proofs pass within 15-minute timeout
- `cargo audit` passes with documented allowlist (any new advisory outside the list fails the gate)
- All actions pinned to full SHA hashes with `persist-credentials: false`
- `robopoker-fork-coherence` runs but does not block pipeline (`continue-on-error: true`)
- Python research QA (ruff + pytest) passes on research files

## Verification

```bash
# Local verification of key CI steps (not full pipeline)

# Repo shape
bash .github/scripts/check_stage0_repo_shape.sh

# Active crates (fast local check)
SKIP_WASM_BUILD=1 cargo check \
  -p myosu-games -p myosu-games-kuhn -p myosu-games-poker \
  -p myosu-games-liars-dice -p myosu-games-portfolio \
  -p myosu-games-canonical -p myosu-tui -p myosu-play \
  -p myosu-chain-client -p myosu-miner -p myosu-validator

# Canonical manifest
manifest="$(SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-canonical \
  --example canonical_manifest)"
echo "$manifest" | grep -c '^CANONICAL_GAME '  # expect 10
echo "$manifest" | grep -c 'snapshot=ok'        # expect 10

# INV-004
SKIP_WASM_BUILD=1 cargo tree -p myosu-play --edges normal | grep 'myosu-miner' || echo "PASS"
SKIP_WASM_BUILD=1 cargo tree -p myosu-miner --edges normal | grep 'myosu-play' || echo "PASS"

# Doctrine integrity
bash .github/scripts/check_doctrine_integrity.sh

# Dependency audit (requires cargo-audit installed)
cargo audit -D warnings \
  --ignore RUSTSEC-2025-0009 --ignore RUSTSEC-2025-0055 \
  --ignore RUSTSEC-2023-0091 --ignore RUSTSEC-2024-0438 \
  --ignore RUSTSEC-2025-0118 --ignore RUSTSEC-2026-0020 \
  --ignore RUSTSEC-2026-0021 --ignore RUSTSEC-2025-0141 \
  --ignore RUSTSEC-2024-0388 --ignore RUSTSEC-2025-0057 \
  --ignore RUSTSEC-2024-0384 --ignore RUSTSEC-2020-0168 \
  --ignore RUSTSEC-2022-0061 --ignore RUSTSEC-2024-0436 \
  --ignore RUSTSEC-2024-0370 --ignore RUSTSEC-2025-0010 \
  --ignore RUSTSEC-2021-0127 --ignore RUSTSEC-2026-0002 \
  --ignore RUSTSEC-2024-0442
```

## Open Questions

1. **E2E timeout margin:** The 15-minute timeout covers 7 sequential integration proofs. Each devnet startup takes >60 seconds. Is the margin comfortable on GitHub-hosted runners, or does it occasionally time out?
2. **WASM target inconsistency:** Chain-core uses `wasm32-unknown-unknown` target, integration-e2e uses `wasm32v1-none`. Is this intentional (different compilation requirements) or drift?
3. **Robopoker fork job as advisory:** The fork coherence check is `continue-on-error: true`. Should it eventually become blocking (INV-006 is an S2 invariant)?
4. **Missing invariant enforcement:** INV-001 (structured closure honesty) and INV-002 (proof honesty) have no CI automation. Is this planned?
5. **Advisory allowlist growth:** The allowlist has grown to 19 entries. Plan 008 proposes triage. What's the target allowlist size?
6. **Python research QA scope:** The Python research files (`main.py`, `methods.py`, `runner.py`, `metrics.py`, `data.py`) are not in a package or virtual environment. Should they be formalized?

# Genesis Assessment

Date: 2026-04-02
Scope: Full repository audit -- all crates, pallets, Python research stack, CI, git history, doctrine files, specs, plans, archived genesis runs, and operational surfaces.
Prior art: `genesisarchive/ASSESSMENT.md` (2026-03-28/30) -- this assessment supersedes it with a fresh code review and updated plan-quality evaluation.

## The Reframe

**What the project says it is:** A decentralized game-solving chain for imperfect-information games using MCCFR strategy computation, Yuma Consensus for economic coordination, and verifiable quality scoring.

**What the code shows it actually is:** A working local-loop prototype connecting a Substrate chain fork, MCCFR-trained poker solvers, deterministic validator scoring, and a polished TUI gameplay surface -- with a massive inherited chain codebase (242K lines) that is mostly unused baggage, a clean gameplay stack (23K lines) that is production-quality, and a Python research layer (~3.2K lines across 5 root files) that is a separate experiment framework sharing the repo.

**One-sentence summary:** Myosu is a 17-day-old, agent-built game-solving subnet prototype where the gameplay and solver crates are mature and CI-gated, the chain fork carries 10x the code needed for stage-0, and no external users or deployments exist yet.

## Six Forcing Questions

### 1. Demand Reality

There is no evidence of external production demand.

- No payment integration, telemetry, user accounts, or external contributors.
- No issues, PRs, or community activity from outside the team.
- README targets developers, not end users.
- Internal usage is real: the repo is actively operated via Fabro/Raspberry, and `myosu-play` compiles and runs as a local NLHE advisor.

Assessment: advanced prototype with internal operator usage. Not yet a product anyone would panic without.

### 2. Status Quo

Without Myosu, the target user would:
- Buy PioSolver ($250--2,500) for NLHE strategy computation.
- Run ad-hoc MCCFR training via robopoker or academic implementations.
- Have no verifiable quality scoring, no incentive market, no multi-game generalization.
- Use no decentralized alternative -- none exists.

Pain Myosu removes: fragmentation between solver quality, verification, incentives, and playable output. The full loop (train -> score -> emit -> play) does not exist anywhere else.

### 3. Desperate Specificity

The ONE person this is for today:

**Solo protocol engineer / founder-operator** building a verifiable game-solving network with limited bandwidth.

What keeps them up at night:
- "Can I prove the full local loop before making public claims?"
- "Can I reduce the 242K-line chain fork enough to ship a believable devnet?"
- "Is the agent-driven development pipeline reliable enough to trust?"

What gets them funded:
- A reproducible demo: chain produces blocks, solver submits strategy, validator scores it, human plays against it, quality is measurable.

### 4. Narrowest Wedge

The smallest thing someone could depend on THIS WEEK:

**`myosu-play` -- a local NLHE training and strategy advisor backed by MCCFR-trained blueprints.**

Why it is real:
- Compiles and runs today (CI-gated, smoke-tested).
- 4-tier artifact auto-discovery with graceful fallback.
- Full TUI with multi-screen state machine, pipe mode for agent integration.
- Does NOT require the chain to be running.

What is NOT a weekly wedge:
- Full miner/validator emissions market on devnet with cross-validator determinism.

### 5. Observation and Surprise

1. **Implementation maturity mismatch.** The gameplay stack (`myosu-games`, `myosu-games-poker`, `myosu-tui`, `myosu-play`) is production-quality: 23K lines, comprehensive tests, property-based testing via proptest, zero `unwrap()` calls in active crates, zero `unimplemented!()` calls. The chain side (242K lines, 11+ pallets) has 122 TODO/FIXME comments, many in critical paths (game-solver migrations, benchmark weights, drand types).

2. **Zero unwrap/expect in active crates.** The workspace clippy lints (`unwrap_used = "deny"`, `expect_used = "warn"`) are enforced. All unwrap calls live in the inherited chain fork. This is an unusually clean codebase for a 17-day-old project.

3. **The chain is a Bittensor subtensor fork carrying enormous baggage.** 20+ pallets, EVM/Frontier integration, dual-token AMM, drand VRF, crowdloan mechanics -- none needed for stage-0. `pallet-game-solver` is essentially a renamed `pallet-subtensor` with 69 extrinsics, 95+ errors, 60+ events, and 50+ storage migrations.

4. **78% of commits are machine-authored.** 27 from Fabro, 12 from Codex, 10 from a human contributor, 1 from the operator. The recovery rate (4 revert/recovery commits out of 50 = 8%) indicates occasional agent pipeline failures requiring human intervention.

5. **Three prior genesis runs exist** in `archive/` plus `genesisarchive/` with 21 plans. The previous plans are structurally sound but several are bootstrap-only (deliverables are spec/review files rather than working code).

6. **The Python research layer is a separate system.** `main.py`, `data.py`, `methods.py`, `metrics.py`, `runner.py` implement an ML experiment framework for evaluating game-solving approaches against a 20-game corpus. This is research infrastructure, not part of the Rust product. It has its own quality issues (exponential complexity in `metrics.py:74-84`, `__import__()` anti-pattern in `methods.py:807`, broad exception catching).

7. **232 unmerged remote branches.** Almost all are Fabro execution artifacts (`fabro/run/*`, `fabro/meta/*`). Severe branch sprawl.

8. **CI references genesis plan files by name.** `check_stage0_repo_shape.sh` requires specific genesis plan files to exist. Renumbering plans without updating the script will break CI.

### 6. Future-Fit

**Strengths for longevity:**
- Game trait abstraction (`myosu-games`) proven additive: Liar's Dice joined without poker changes.
- Wire protocol (`wire.rs`) with versioned magic bytes prevents silent corruption.
- Checkpoint format versioning (4-byte magic + version) is forward-compatible.
- `myosu-chain-client` as shared seam prevents DRY violations across miner/validator/play.

**Risks for longevity:**
- Substrate/Polkadot SDK fork pinned to opentensor revision -- upstream drift is inevitable.
- robopoker fork pinned by git rev -- upstream divergence must be tracked (INV-006).
- Single-operator model with no multi-tenant or permissioned access.
- No observability beyond print-based logging (tracing in miner/validator, print in play/games).

## Source-Code Findings by Component

### Active Crates (23K lines, CI-gated)

| Crate | Lines | Quality | Tests | Issues |
|-------|-------|---------|-------|--------|
| `myosu-games` | 641 | Excellent | Property-based (proptest) | None |
| `myosu-games-poker` | 9,400 | Good | Wire round-trip, action encoding | 2 justified unsafe blocks (mmap) |
| `myosu-games-liars-dice` | 2,100 | Good | Game state, wire protocol | None |
| `myosu-tui` | 3,200 | Good | Shell state machine | None |
| `myosu-play` | 2,700 | Good | Smoke tests, integration | None |
| `myosu-chain-client` | 2,560 | Excellent | 13 unit tests, comprehensive error enum | None |
| `myosu-miner` | 1,400 | Good | Report formatting | None |
| `myosu-validator` | 1,600 | Good | Report formatting | None |
| `myosu-keys` | 1,960 | Excellent | 17 tests, crypto validation | None |

### Chain Crate (242K lines, partially CI-gated)

| Component | Lines (est.) | Quality | Tests | Issues |
|-----------|-------------|---------|-------|--------|
| `pallet-game-solver` | 40K+ | Fair | Stage-0 flow tests | 122 TODO/FIXME across chain |
| `myosu-chain-runtime` | 15K+ | Fair | CI-checked | Wires full subtensor surface |
| `myosu-chain` (node) | 8K+ | Fair | CI-checked | Placeholder chain specs |
| Other pallets (10) | 170K+ | Inherited | Mixed | EVM, AMM, drand -- unused |

### Python Research Stack (not CI-gated)

| File | Lines | Quality | Issues |
|------|-------|---------|--------|
| `main.py` | 489 | Good | Broad exception catching |
| `data.py` | 1,060 | Good | Hardcoded dataset paths, silent fallbacks |
| `methods.py` | 1,105 | Good | `__import__()` anti-pattern at line 807 |
| `metrics.py` | 246 | Good | 2^n permutation test will hang for n>20 |
| `runner.py` | 337 | Good | No checkpoint/recovery mechanism |

## What Works

1. **Local gameplay loop.** `myosu-play` runs NLHE poker hands against MCCFR-trained strategy with a polished TUI. Smoke-tested in CI.
2. **Multi-game architecture.** Liar's Dice crate proves the `GameType` enum and trait abstraction work additively.
3. **Solver pipeline.** Miner trains MCCFR blueprints, checkpoints them, serves via HTTP axon. Validator queries and scores deterministically.
4. **Key management.** `myosu-keys` encrypts with XSalsa20Poly1305 + scrypt KDF, sets 0o600 file permissions, supports import/export.
5. **CI pipeline.** 7 jobs covering repo shape, active crates, chain core, doctrine integrity, plan quality, operator network, chain clippy.
6. **Operator bootstrap.** `myosu-keys print-bootstrap` generates miner/validator commands. Operator bundle with chain specs, startup scripts, verification.
7. **Wire protocol.** Binary encoding/decoding with versioned magic bytes for game state serialization.

## What is Broken

1. **CI references genesis plan files by name.** `check_stage0_repo_shape.sh` requires `genesis/plans/002-*.md`, `genesis/plans/010-*.md`, `genesis/plans/020-*.md` to exist. Renumbering plans without updating the script breaks CI.
2. **Python research stack has no CI gate.** No linting, no tests, no type checking for the 5 Python files at repo root.
3. **`run_eval.py` assumes stdout JSON but `main.py` writes to file.** The external evaluation harness will fail to parse results.
4. **Placeholder chain specs.** `node/src/chain_spec/devnet.rs` and `testnet.rs` are functional but minimal.

## What is Half-Built

1. **Chain runtime reduction.** Runtime still wires full subtensor surface. Stubs exist but surface area is unreduced.
2. **Multi-node devnet.** Chain builds locally, but no multi-node configuration or persistent devnet.
3. **Validator determinism proof.** INV-003 supported by code but no integration test proves two validators agree.
4. **Emission accounting.** Yuma Consensus wired but coinbase logic assumes root network + AMM which are disabled.
5. **Observability.** Miner/validator use `tracing`. Play/games use print. No unified observability.
6. **Benchmark weights.** Multiple pallet dispatches use placeholder weights with TODO comments.

## Tech-Debt Inventory

| ID | Location | Severity | Description |
|----|----------|----------|-------------|
| TD-01 | `crates/myosu-chain/` | High | 242K lines of inherited subtensor fork, ~200K unused |
| TD-02 | `crates/myosu-chain/pallets/` | High | 10 pallets (EVM, AMM, drand, crowdloan) unused in stage-0 |
| TD-03 | Chain TODO/FIXME | Medium | 122 TODO/FIXME comments, many in migrations |
| TD-04 | Benchmark weights | Medium | Placeholder weights in pallet dispatches |
| TD-05 | Python `__import__()` | Low | `methods.py:807` circular import workaround |
| TD-06 | Python exponential | Low | `metrics.py:74-84` generates 2^n permutations |
| TD-07 | Branch sprawl | Low | 232 unmerged remote branches |
| TD-08 | `COMPLETED.md` | Low | Empty stub file |

## Security Risks

| ID | Threat | Likelihood | Impact | Current Mitigation |
|----|--------|------------|--------|-------------------|
| SR-01 | Inherited chain vulnerabilities from subtensor fork | Medium | High | Pinned to specific revision; no independent audit |
| SR-02 | robopoker fork drift corrupting MCCFR correctness | Low | High | INV-006 + fork changelog; no automated check |
| SR-03 | Memory-mapped blueprint files (2 unsafe blocks) | Low | Medium | SAFETY comments; read-only artifacts |
| SR-04 | Placeholder chain specs used in production | Low | High | Only devnet/testnet exist |
| SR-05 | Substrate SDK upstream security patches not applied | Medium | High | Fork pinned; no CVE tracking process |
| SR-06 | Operator key password via environment variable | Low | Low | Standard practice; no HSM support |

## Test Gaps

| Component | Unit | Integration | E2E | Edge-Case | Critical Gaps |
|-----------|------|-------------|-----|-----------|---------------|
| myosu-games | Proptest | N/A | N/A | Yes | None |
| myosu-games-poker | Yes | N/A | N/A | Wire RT | Solver edge cases |
| myosu-games-liars-dice | Yes | N/A | N/A | Basic | Minimal coverage |
| myosu-tui | Shell state | N/A | N/A | Basic | No render tests |
| myosu-play | Smoke | INV-004 | N/A | Basic | No live miner test |
| myosu-chain-client | 13 tests | N/A | N/A | Endpoint | No RPC test |
| myosu-miner | Reports | N/A | N/A | None | No training test |
| myosu-validator | Reports | N/A | N/A | None | No scoring test |
| myosu-keys | 17 tests | N/A | N/A | Crypto RT | None |
| pallet-game-solver | Stage-0 | N/A | N/A | Some | Migration tests TODO |
| Python research | None | None | None | None | Entirely untested |

## Git Metrics Summary

| Metric | Value |
|--------|-------|
| Repo age | 17 days (first commit 2026-03-16) |
| Total trunk commits | 50 |
| Contributors | 4 identities (2 human, 2 automated) |
| Machine-authored commits | 78% (Fabro 54%, Codex 24%) |
| Recovery/revert commits | 4 (8%) |
| Commits/week (avg) | 16.7 (bursty: 46 in first 2 days) |
| Active Rust LOC | 23K (active crates) + 242K (chain fork) |
| Python LOC | ~3.2K (research stack root files) |
| Unmerged remote branches | 232 |
| CI jobs | 7 |

## Documentation Staleness Table

| Document | Last Updated | Current | Stale Fields |
|----------|-------------|---------|--------------|
| `README.md` | 2026-03-18 | Mostly current | May reference old plan numbers |
| `SPEC.md` | 2026-03-18 | Current | None |
| `INVARIANTS.md` | 2026-03-18 | Current | References `ops/` files that may not exist |
| `OS.md` | 2026-03-18 | Partially stale | References root `DESIGN.md` |
| `THEORY.MD` | 2026-03-18 | Current | Narrative of stage-0 achievement |
| `AGENTS.md` | 2026-03-18 | Current | Agent role definitions |
| `PLANS.md` | 2026-03-18 | Current | ExecPlan format definition |
| `COMPLETED.md` | Unknown | Empty stub | No content |
| `docs/execution-playbooks/` | 2026-03-18 | Partially stale | Some playbooks reference outdated surfaces |

## Implementation-Status Table (Prior Genesis Plans)

| Plan | Title | Verified Status | Evidence |
|------|-------|-----------------|----------|
| 002 | Spec Corpus Normalization | **Verified** | 19 non-empty specs, doctrine CI passes |
| 003 | Chain Runtime Reduction | **Partially verified** | Stubs exist; runtime still wires full surface |
| 004 | Node Devnet Minimalization | **Partially verified** | Node builds; placeholder chain specs |
| 005 | Pallet Game-Solver Simplification | **Partially verified** | Stage-0 tests pass; 50+ migrations remain |
| 006 | Game Traits and Poker Boundaries | **Verified** | Traits proven with two games |
| 007 | Miner-Validator Bootstrap | **Verified** | Both binaries build, register, serve, score |
| 008 | Artifact Wire Checkpoint Hardening | **Verified** | Versioned wire protocol, checkpoint format |
| 009 | Play TUI Productization | **Verified** | Multi-screen TUI, pipe mode, CI-gated |
| 010 | CI Proof Gates Expansion | **Verified** | 7 CI jobs, 5 verification scripts |
| 011 | Security Observability Release | **Stub only** | No dashboard, no audit |
| 012 | Multi-Game Architecture Proof | **Verified** | Liar's Dice crate, separate subnet |
| 013 | Integration Test Harness | **Partially verified** | Smoke tests; no full harness |
| 014 | OS Refresh and Operator Docs | **Verified** | OS.md, playbooks, operator bundle |
| 015 | Malinka Retirement | **Verified** | `.malina/` exists but inactive |
| 016 | Fabro Control Plane Cutover | **Verified** | `fabro/` with programs, workflows |
| 017 | Rationalize Fabro Surfaces | **Partially verified** | Programs exist; incomplete workflows |
| 018 | Genesis Corpus Adjudication | **Verified** | This run is the adjudication |
| 019 | Future Synth Genesis Governance | **Stub only** | No governance process implemented |
| 020 | Second-Game Subnet Proof | **Verified** | Liar's Dice on subnet 3 |
| 021 | Operator Hardening | **Verified** | Operator bundle with chain specs |

## Code-Review Coverage List

Files actually read during this review:

**Active Crates:**
- `crates/myosu-play/src/main.rs`, `cli.rs`, `discovery.rs`, `live.rs`, `blueprint.rs`
- `crates/myosu-miner/src/main.rs`, `lib.rs`, `cli.rs`, `chain.rs`, `axon.rs`, `strategy.rs`, `training.rs`
- `crates/myosu-validator/src/main.rs`, `lib.rs`, `cli.rs`, `chain.rs`, `validation.rs`
- `crates/myosu-games/src/lib.rs`, `traits.rs`, `registry.rs`
- `crates/myosu-games-poker/src/` (all 10 files)
- `crates/myosu-games-liars-dice/src/` (all 7 files)
- `crates/myosu-tui/src/` (all 9 files)
- `crates/myosu-chain-client/src/lib.rs`
- `crates/myosu-keys/src/lib.rs`, `main.rs`, `storage.rs`

**Chain Crate (sampled):**
- `crates/myosu-chain/node/src/main.rs`, `cli.rs`, `service.rs`
- `crates/myosu-chain/runtime/src/lib.rs`
- `crates/myosu-chain/pallets/game-solver/src/lib.rs`
- `crates/myosu-chain/common/src/lib.rs`

**CI/Scripts:**
- `.github/workflows/ci.yml`
- `.github/scripts/check_stage0_repo_shape.sh`
- `.github/scripts/check_doctrine_integrity.sh`
- `.github/scripts/check_plan_quality.sh`
- `.github/scripts/check_operator_network_bootstrap.sh`

**Doctrine:**
- `README.md`, `SPEC.md`, `AGENTS.md`, `INVARIANTS.md`, `OS.md`, `PLANS.md`, `THEORY.MD`, `COMPLETED.md`, `fabro.toml`, `Cargo.toml`, `rust-toolchain.toml`, `rustfmt.toml`

**Python:**
- `main.py`, `data.py`, `methods.py`, `metrics.py`, `runner.py`
- `research/autoresearch/scripts/run_eval.py`

**Prior Genesis:**
- `genesisarchive/ASSESSMENT.md`
- All 21 plan headers in `genesisarchive/plans/`

**Specs:**
- `specs/` directory listing (19 spec files confirmed present and non-empty per CI)

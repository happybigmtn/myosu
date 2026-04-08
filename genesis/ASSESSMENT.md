# Myosu Assessment

Generated: 2026-04-07
Codebase snapshot: trunk @ 4e0b37f
Previous corpus: .auto/fresh-input/genesis-previous-20260407-234710 (historical context only)

---

## Problem Statement

**How might we build a permissionless protocol that turns game-theory computation into a decentralized commodity, starting from the working local loop we already have, without drowning in the inherited complexity of the Bittensor fork?**

The repo has a proven single-machine loop: chain authors blocks, miner trains MCCFR strategy, validator scores deterministically, gameplay consumes it. Three games work. The challenge is no longer "can we build it" but "can we ship it cleanly enough for operators to run it."

## Target Users and Success Criteria

| User | What success looks like |
|------|----------------------|
| **Miner operator** | Downloads binary, points at devnet, trains strategy, earns emission within one session |
| **Validator operator** | Downloads binary, points at devnet, scores miners deterministically, submits weights |
| **Node operator** | Runs chain node, participates in consensus, sees block production and finality |
| **Game player** | Launches TUI or pipe, plays poker against trained bot, sees strategy quality |
| **Game developer** | Adds a new game crate, implements traits, registers on-chain without touching poker code |
| **Protocol researcher** | Reads specs and ADRs, understands emission mechanics, can reason about economic changes |

## What the Project Says It Is

AGENTS.md and OS.md describe myosu as a "decentralized game-solving protocol for imperfect-information games." The architecture diagram shows four layers:

1. **Chain** (Substrate) — subnets, neurons, weights, emissions via Yuma Consensus
2. **Solvers/Miners** — off-chain MCCFR training and strategy serving
3. **Validators** — off-chain exploitability scoring and weight submission
4. **Gameplay** — TUI, pipe, and HTTP surfaces for humans and agents

The project claims stage-0 is mostly proven and the work remaining is hardening, packaging, and research gates.

## What the Code Actually Shows

### Verified truths

- **The local loop works.** `tests/e2e/local_loop.sh` proves chain + miner + validator + gameplay integration for both poker and Liar's Dice as distinct subnets on one devnet.
- **Multi-game architecture is real.** Liar's Dice was added with zero changes to poker code. Kuhn poker exists as a third-game proof. The `GameType` enum and `CfrGame` trait hierarchy work.
- **Validator determinism is proven.** `tests/e2e/validator_determinism.sh` covers both poker and Liar's Dice. INV-003 has E2E coverage.
- **Multi-node consensus works.** `four_node_finality.sh` proves 4-authority GRANDPA finality. `consensus_resilience.sh` proves restart catch-up. `cross_node_emission.sh` proves emission agreement.
- **CI is substantial.** 9 CI jobs: repo-shape, active-crates (check/test/clippy/fmt), chain-core, E2E integration (7 scripts), doctrine integrity, dependency audit, plan quality, operator-network, chain-clippy.
- **Key management exists.** `myosu-keys` supports create, import (keyfile/mnemonic/raw-seed), export, switch-active, change-password, list — all with network-namespaced storage.
- **Operator documentation exists.** Quickstart, architecture, troubleshooting, upgrading guides under `docs/operator-guide/`.

### Structural concerns

- **Duplicated pallet.** `pallet-game-solver` (91.6K lines) and `pallet-subtensor` (90.6K lines) are near-identical copies. The runtime aliases game-solver AS `pallet_subtensor` (`pallet_subtensor = { package = "pallet-game-solver" }`). The original `pallet-subtensor` is dead code. This is ~90K lines of inherited Bittensor code that exists twice.
- **Inherited complexity behind stubs.** The NoOpSwap identity stub (`Stage0NoopSwap`) papers over 37 swap callsites. AMM, dual-token, root-network, Alpha emission logic all exist behind feature gates and stubs but are not exercised.
- **~44 inherited migration files** in game-solver — all subtensor-era migrations for a chain state myosu has never had.
- **Naming confusion.** The active pallet is `pallet-game-solver` but every code reference uses `pallet_subtensor::` because of the Cargo alias. This will confuse every new contributor.
- **Fabro/Raspberry execution model is aspirational.** AGENTS.md and OS.md reference `fabro/workflows/`, `fabro/run-configs/`, `fabro/programs/`, `.raspberry/` extensively, but the `fabro/` directory does not exist. Only `fabro.toml` exists at root.

## What Works

| Surface | Status | Evidence |
|---------|--------|----------|
| Chain block production (local devnet) | Working | `tests/e2e/local_loop.sh`, CI `chain-core` |
| Aura + GRANDPA consensus | Working | `four_node_finality.sh`, `consensus_resilience.sh` |
| pallet-game-solver stage-0 flow | Working | `cargo test -p pallet-game-solver stage_0_flow` (26 tests) |
| Emission coinbase | Working | Coinbase unit tests + `emission_flow.sh` E2E |
| Yuma epoch/consensus math | Working | `determinism.rs` (2 tests), inherited math suite |
| Poker solver integration | Working | `myosu-games-poker` wraps robopoker, smoke tests pass |
| Liar's Dice solver | Working | `myosu-games-liars-dice` complete, tests pass |
| Kuhn poker solver | Working | `myosu-games-kuhn` complete, tests pass |
| Miner bounded training | Working | `myosu-miner --train-iterations` for all games |
| Miner HTTP axon (poker) | Working | `axon.rs` serves strategy over HTTP |
| Validator deterministic scoring | Working | `validator_determinism.sh`, unit tests |
| Gameplay smoke (poker + Kuhn) | Working | `myosu-play --smoke-test`, `--game kuhn --smoke-test` |
| TUI shell with ratatui | Working | `myosu-tui` shell state tests (33 test fns) |
| Key management CLI | Working | 19 test functions, operator guide |
| Chain client RPC wrapper | Working | `myosu-chain-client` (2.2K lines, 16 tests) |
| Cross-node emission agreement | Working | `cross_node_emission.sh` |
| Operator bundle packaging | Working | `check_operator_network_bootstrap.sh` |
| Fresh-machine operator proof | Working | `check_operator_network_fresh_machine.sh` |
| INV-004 dependency separation | Working | CI enforces via `cargo tree` |

## What Is Broken

No actively broken surfaces detected. The codebase compiles and tests pass on the default stage-0 build path. Specific known limitations:

| Issue | Impact |
|-------|--------|
| Poker training on bootstrap artifacts panics upstream | `isomorphism not found` — by design (sparse artifacts), but blocks F-007 convergence work |
| Miner HTTP axon is poker-only | Liar's Dice uses file-based query/response path only |
| `fabro/` directory does not exist | All execution-model references in AGENTS.md/OS.md are aspirational |

## What Is Half-Built

| Surface | State | Gap |
|---------|-------|-----|
| Token economics | Research-only | ADR 008 exists as `Proposed`, needs multi-contributor review (F-003) |
| Miner convergence quality | No quality benchmark | Validator self-scores miner checkpoint; no independent exploitability metric (F-007) |
| polkadot-sdk migration | Feasibility study done | ADR 009 shows 21 opentensor-only commits; actual migration not started |
| Docker/container packaging | Not started | No Dockerfile, no docker-compose |
| Production deployment | Not started | Devnet-only; no production chain spec, no monitoring |
| Web gameplay surface | Not started | TUI and pipe only |
| Python research layer | Disconnected | 3.5K lines of CFR research code (main.py, methods.py, etc.) with no integration into Rust crates |
| `pallet-subtensor` cleanup | Not done | 90.6K lines of dead code still in tree |
| Inherited migration cleanup | Not done | 44 subtensor-era migration files still in game-solver |
| Pallet renaming | Not done | Every code reference uses `pallet_subtensor::` via Cargo alias |

## Tech Debt Inventory

### Critical (blocks operator confidence)

1. **Duplicated pallet** — 90.6K lines of `pallet-subtensor` dead code. Confuses code search, bloats compilation, risks accidentally linking the wrong crate.
2. **Pallet naming** — `pallet-game-solver` is aliased as `pallet_subtensor` everywhere. New contributors will search for `pallet_subtensor` and find two pallets. No grep for "game-solver" finds runtime usage.
3. **44 inherited migrations** — All for Bittensor chain state that myosu never had. Dead weight in build and test surface.

### High (affects development velocity)

4. **18+ cargo audit advisories** — Allowlisted in CI. Mix of inherited Substrate deps (`ring`, `proc-macro-error`, `paste`, etc.) and direct usage (`bincode 1.x` in game crates).
5. **NoOpSwap stub complexity** — 37 callsites pass through an identity stub that returns 1:1 conversion. The swap-interface trait (`SwapEngine`) and V3 AMM implementation both exist but are unused. Token economics decision is deferred.
6. **Stale doc surfaces** — THEORY.MD (97K), IMPLEMENTATION_PLAN.md (12.5K), and several root .md files contain stale cross-references and historical context mixed with active doctrine.
7. **Fabro/Raspberry ghost infrastructure** — AGENTS.md and OS.md describe an execution model that doesn't exist on disk. This creates a false impression of operational maturity.

### Medium (ongoing friction)

8. **Inherited pallet test suites** — game-solver has hundreds of inherited subtensor tests behind `legacy-subtensor-tests` feature gate. Many test concepts (Alpha/TAO dual-token, AMM pools, root network, EVM) that myosu doesn't use.
9. **Python research layer** — 3.5K lines across 5 root-level files with no package management beyond `pip install numpy ruff pytest`. Disconnected from Rust codebase.
10. **Full-runtime feature gate** — Several pallets (Sudo, Multisig, Preimage, Scheduler, Proxy, SafeMode, AdminUtils) only exist behind `full-runtime`. The stage-0 default build excludes them, but their config impls clutter the runtime file.
11. **Sparse poker bootstrap artifacts** — Any poker `--train-iterations > 0` fails upstream. This is by design but blocks convergence research.

## Security Risks

| Risk | Severity | Status |
|------|----------|--------|
| 18 cargo audit advisories in allowlist | Medium | Documented in WORKLIST.md SEC-001. Mix of unmaintained deps and known vulnerabilities. `bincode 1.x` is a direct dependency in game crates. |
| Insecure randomness pallet included in full-runtime | Low | `pallet_insecure_randomness_collective_flip` behind `full-runtime` feature. Not used by stage-0. |
| NoOpSwap identity has no economic security | Low (stage-0) | By design for single-token stage-0. Real risk only if production deployment uses NoOpSwap. |
| Mmap checkpoint loading | Low | Documented in SECURITY.md. Checkpoints are local files; no remote loading without operator action. |
| Wire codec decode budget | Mitigated | Hardened to 1 MiB for poker (C-013). |
| GRANDPA finality threshold | Documented | 4-authority set requires 3/3 threshold. Well-understood Substrate behavior. |
| No rate limiting on miner HTTP axon | Medium | `axon.rs` serves strategy over HTTP with no authentication or rate limiting. Devnet-only risk. |

## Test Gaps

| Gap | Current state | Risk |
|-----|--------------|------|
| Miner convergence quality | No quality benchmark; validator self-scores | Cannot document minimum training iterations |
| Emission dust accounting | Measured (2 rao/block) but policy deferred | Dust accumulates unbounded over long epochs |
| HTTP axon security | No tests for malformed requests, DoS | Devnet-only, but untested surface |
| Key management edge cases | 19 tests, but no concurrent access or corruption recovery | Single-operator assumed |
| Runtime upgrade path | One smoke test (`devnet_runtime_upgrade_smoke_test_passes_on_fresh_genesis`) | No real migration exercised |
| Cross-game scoring fairness | No test | Different games may have incomparable quality metrics |

## Documentation Staleness

| Document | Lines | State |
|----------|-------|-------|
| THEORY.MD | ~2400 | Historical. 97K bytes of CFR theory. Accurate but not referenced by active code. |
| IMPLEMENTATION_PLAN.md | ~153 | Partially stale. References specs from `050426-*` generation. Some tasks completed, some blocked. |
| AGENTS.md | ~374 | Partially stale. References fabro/ paths that don't exist. Core architecture section accurate. |
| OS.md | ~260 | Mostly current. References fabro/raspberry surfaces that don't exist. Stage-0 truth statements accurate. |
| WORKLIST.md | ~14 | Current. Active follow-up items with clear ownership. |
| README.md | ~143 | Current. Accurate proof commands and operator loop. |
| SPEC.md (root) | ~150 | Current. Meta-spec describing spec types. |
| PLANS.md (root) | ~149 | Current. ExecPlan format requirements. |
| INVARIANTS.md | ~88 | Current. All 6 invariants accurately described. |
| SECURITY.md | ~90 | Current. Disclosure process and scope accurate. |
| CHANGELOG.md | ~73 | Current. 0.1.0 baseline documented. |
| LEARNINGS.md | ~22 | Current. Operational wisdom from review cycles. |

## Implementation Status Table

Status of prior claims from AGENTS.md and IMPLEMENTATION_PLAN.md, verified against code:

| Claim | Verified | Evidence |
|-------|----------|----------|
| Chain produces blocks on local devnet | Yes | `local_loop.sh` |
| pallet-game-solver at runtime index 7 | Yes | `construct_runtime!` line 1275 (aliased as SubtensorModule) |
| Poker subnet registration and evaluation | Yes | E2E local loop |
| Miner MCCFR training | Yes | Bounded training works for all games |
| Validator deterministic scoring | Yes | `validator_determinism.sh` covers poker + Liar's Dice |
| Two validators identical scores (INV-003) | Yes | `validator_determinism.sh` |
| Yuma emission distribution | Yes | `emission_flow.sh`, coinbase unit tests |
| Human plays poker against bot | Yes | `myosu-play --smoke-test` |
| TUI training mode with solver advisor | Partial | TUI shell exists (`myosu-tui`), but solver advisor integration not independently verified |
| Multi-game zero-change extensibility | Yes | Liar's Dice added, Kuhn poker added, zero poker changes |
| INV-004 solver-gameplay separation | Yes | CI enforces via `cargo tree` |
| Emission accounting invariant | Partial | Unit tests pass, E2E `emission_flow.sh` passes, but dust policy deferred |
| Fabro/Raspberry execution model | No | `fabro/` directory does not exist |
| Four chain spec variants | Yes | localnet, devnet, testnet (test_finney), finney in `chain_spec/` |
| Operator network bundle | Yes | `check_operator_network_bootstrap.sh`, `check_operator_network_fresh_machine.sh` |
| Release process | Partial | `ops/release.sh --dry-run` exists but no actual release cut |
| 82 ACs across 14 stages | Stale | This is the original AGENTS.md master plan count. Actual completion tracked in IMPLEMENTATION_PLAN.md shows 17 completed items (C-001 through C-017). |

## Code Review Coverage

Source files directly read during this assessment:

| File | Lines | Purpose |
|------|-------|---------|
| `Cargo.toml` (root) | 202 | Workspace config, 12 members, opentensor polkadot-sdk fork |
| `AGENTS.md` | 374 | Kernel OS document |
| `README.md` | 143 | Project entry point |
| `INVARIANTS.md` | 88 | 6 hard invariants |
| `SPEC.md` (root) | 150 | Spec type definitions |
| `PLANS.md` (root) | 149 | ExecPlan format |
| `WORKLIST.md` | 14 | Active follow-ups |
| `OS.md` | 260 | Operating system doctrine |
| `IMPLEMENTATION_PLAN.md` | 153 | Priority work tracker |
| `SECURITY.md` | 90 | Disclosure policy |
| `LEARNINGS.md` | 22 | Operational wisdom |
| `CHANGELOG.md` | 73 | Release history |
| `fabro.toml` | 24 | Execution config (MiniMax model) |
| `.github/workflows/ci.yml` | 419 | CI pipeline |
| `crates/myosu-games/src/traits.rs` | 500 | Game trait hierarchy |
| `crates/myosu-games/src/registry.rs` | 160 | Game registry |
| `crates/myosu-miner/src/lib.rs` | 224 | Miner report formatting |
| `crates/myosu-validator/src/validation.rs` | 735 | Validator scoring logic |
| `crates/myosu-chain/runtime/src/lib.rs` | 1400+ | Runtime composition, construct_runtime |
| `crates/myosu-chain/runtime/Cargo.toml` | 130+ | Runtime dependencies (pallet alias) |
| `crates/myosu-chain/pallets/game-solver/src/coinbase/run_coinbase.rs` | 80+ | Emission distribution |
| `crates/myosu-chain/pallets/game-solver/src/tests/stage_0_flow.rs` | 80+ | Stage-0 integration tests |
| `crates/myosu-chain/pallets/game-solver/Cargo.toml` | 10+ | Pallet package name |
| `crates/myosu-chain/pallets/subtensor/Cargo.toml` | 10+ | Original pallet package name |

Additionally, 5 parallel exploration agents read across:
- All crate entry points (lib.rs, main.rs) for all 11 workspace members
- Complete test file inventories across all pallets
- All E2E test scripts
- All spec files (46 total)
- All ADR files (10 + template)
- All operator guide documents
- All ops/ infrastructure files
- Previous planning snapshot (14 plan files + 5 corpus files)

## Repo Constraints

1. **opentensor polkadot-sdk fork** — Pinned to rev `71629fd`. Not upstream Substrate. Contains subtensor-specific patches. Migration to upstream is a research gate (ADR 009).
2. **robopoker fork** — `happybigmtn/robopoker` pinned to specific rev. Needs serde, encoder constructors, clustering API, file checkpoints (RF-01 through RF-04 in AGENTS.md). INV-006 tracks fork coherence.
3. **Single-developer operation** — Commit history shows one contributor. All review and decision gates are self-referential.
4. **Rust 2024 edition** — `edition = "2024"` in workspace. Requires recent nightly or stable.
5. **wasm32 target** — Runtime requires `wasm32-unknown-unknown` or `wasm32v1-none` for WASM build.

## Assumption Ledger

| Assumption | Status | Evidence |
|-----------|--------|----------|
| pallet-game-solver is the active pallet | Verified | Runtime Cargo.toml: `pallet_subtensor = { package = "pallet-game-solver" }` |
| NoOpSwap provides correct stage-0 economics | Verified | `swap_stub.rs` tests, coinbase unit tests |
| Multi-game architecture requires zero poker changes | Verified | Liar's Dice and Kuhn additions, CI INV-004 check |
| 4-authority GRANDPA threshold is 3/3 | Verified | `four_node_finality.sh` kills 1, 3 keep finalizing |
| Sparse poker bootstrap artifacts are intentional | Verified | AGENTS.md, WORKLIST.md MINER-QUAL-001 |
| Fabro/Raspberry is the execution model | Unverified | Directory does not exist. Config file points to MiniMax model. |
| 82 ACs across 14 stages is the complete scope | Needs update | Some completed, some stale, numbering from early AGENTS.md |
| Emission dust is bounded | Measured, not decided | 2 rao/block worst case, but no accumulation policy |
| robopoker fork tracks v1.0.0 baseline | Assumed | `docs/robopoker-fork-changelog.md` exists, INV-006 CI check is advisory (continue-on-error) |

## Opportunity Framing

### Strongest Direction: Close Stage-0, Then Operator Packaging

The codebase has proven its core thesis: decentralized game-solving with MCCFR works end-to-end. The remaining stage-0 work is primarily cleanup (dead code, naming, stale docs) and research gates (token economics, convergence benchmarks). After stage-0, the highest-leverage work is operator packaging: containers, monitoring, and a multi-node devnet that operators can actually run.

### Rejected Direction: Expand Game Portfolio Now

Adding more games (beyond the three already working) would not accelerate stage-0 exit or improve operator confidence. The multi-game architecture is already proven. Game expansion is a stage-1 concern.

### Rejected Direction: Production Chain Deployment

Attempting production deployment before the pallet is cleaned, token economics are decided, and the SDK migration path is understood would create irreversible technical debt. The NoOpSwap stub is explicitly not production-ready.

### Rejected Direction: Web Gameplay Surface

Building a web frontend before the operator network exists would create a product without infrastructure. TUI and pipe serve stage-0 needs adequately.

## DX Assessment (Developer-Facing)

### First-Run Friction

- **Zero to compile**: Requires Rust nightly/stable 2024 edition, `wasm32-unknown-unknown` target, protobuf compiler. README mentions none of these prerequisites. Quickstart guide covers them but lives in `docs/operator-guide/quickstart.md`, not README.
- **Zero to running**: `SKIP_WASM_BUILD=1 cargo test -p myosu-chain --test stage0_local_loop --quiet` is the fastest proof, but requires a pre-cached runtime WASM. Cold builds take many minutes with no progress indication.
- **Error clarity**: Build errors from missing `protoc` or wrong WASM target are opaque Substrate errors, not myosu-specific.

### Copy-Paste Onboarding Honesty

- README proof commands are copy-paste honest. They work.
- The `fabro run` commands in README do not work (directory missing).
- Operator guide quickstart is comprehensive and honest about prerequisites.

### Fastest Path to Success Moment

Running `cargo test -p myosu-games-kuhn --quiet` (Kuhn poker tests) is the fastest meaningful success — ~30s on warm cache, exercises game traits, solver, wire codec. This is not documented as an onboarding path.

### Uncertainty Reduction

A new developer reading only README + AGENTS.md would be confused by:
1. Two pallets with near-identical content
2. Fabro/Raspberry references with no on-disk counterpart
3. The `pallet_subtensor` naming everywhere for something called game-solver
4. 97K bytes of THEORY.MD with no clear connection to running code

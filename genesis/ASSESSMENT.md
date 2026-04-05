# Myosu Assessment

Date: 2026-04-05
Scope: Full repo review of `/home/r/Coding/myosu` at commit `b1be135` on `trunk`.

---

## How Might We

**How might we prove that a permissionless, incentive-aligned game-solving network
can produce, score, and distribute Nash-approximate strategy for imperfect-information
games -- starting with a single honest local loop that connects chain, miner,
validator, and gameplay into one verifiable surface?**

---

## What the Project Says It Is

Myosu is a decentralized game-solving protocol for imperfect-information games.
Miners produce Nash-approximate strategies via MCCFR. Validators score quality
deterministically. Yuma Consensus distributes emissions to the strongest solvers.
Humans and agents play through the same text interface.

The repo targets "stage 0": proving the full loop works on a single local node
before claiming any network-level product.

## What the Code Shows It Is

A 267K-line Rust workspace (556 `.rs` files) with a working-but-inherited
Substrate chain, three game implementations (poker, Liar's Dice, Kuhn poker),
off-chain miner/validator binaries, a TUI-based gameplay surface, and a
growing E2E integration harness. The chain layer is a heavily reduced fork of
Bittensor's `subtensor` pallet, carrying significant inherited complexity.

The local loop is proven: chain authors blocks, miner trains and serves strategy,
validator scores deterministically, gameplay consumes trained artifacts, and
E2E scripts validate the full path. Multi-game architecture is proven with
two additional games requiring zero changes to existing code.

---

## Target Users and Success Criteria

### Primary Users

1. **Operators** -- run chain nodes, miners, and validators. Success = zero-friction
   local bootstrap, clear upgrade path, honest diagnostics.
2. **Game-solver researchers** -- contribute new games or improve strategy quality.
   Success = clean game trait interface, verifiable deterministic scoring.
3. **Players/agents** -- consume strategy through TUI, pipe, or HTTP. Success =
   useful game-theoretic advice with transparent provenance.

### Success Criteria for Stage-0 Exit

Per `OS.md` and `INVARIANTS.md`:

- Chain compiles, authors blocks on local devnet
- `pallet-game-solver` at runtime index 7 with Yuma Consensus
- Poker subnet registers and runs solver evaluation
- Miner produces MCCFR strategy, validator scores deterministically
- Two validators produce identical scores for same miner (INV-003)
- Yuma distributes emissions proportional to quality
- Human can play poker against trained bot
- Multi-game architecture proven (Liar's Dice + Kuhn)
- No dependency path between myosu-play and myosu-miner (INV-004)
- Emission accounting: sum(distributions) == block_emission * epochs

### Current Status Against Exit Criteria

| Criterion | Status | Evidence |
|-----------|--------|----------|
| Chain authors blocks | PROVEN | `stage0_local_loop` test, E2E scripts |
| Game-solver pallet at index 7 | PROVEN | Runtime construct_runtime! macro |
| Poker subnet registration | PROVEN | `stage_0_flow.rs` tests |
| Miner MCCFR training | PROVEN | `myosu-miner` bounded training + checkpoint |
| Deterministic validator scoring | PROVEN | `inv_003_determinism` test, `validator_determinism.sh` |
| Two-validator agreement | PARTIALLY PROVEN | Unit test determinism, E2E cross-validator script |
| Yuma emission distribution | PROVEN WITH CAVEATS | `epoch.rs` tests, but inherited AMM complexity persists |
| Human poker gameplay | PROVEN | `myosu-play --smoke-test`, TUI train mode |
| Multi-game architecture | PROVEN | Liar's Dice + Kuhn poker, zero existing code changes |
| INV-004 separation | PROVEN | CI `cargo tree` gate |
| Emission accounting | PARTIALLY PROVEN | `stage_0_flow` coinbase assertions |

---

## What Works

### Game Trait System (`myosu-games`)
- Clean re-export of robopoker CFR traits (`CfrGame`, `CfrEdge`, `CfrInfo`, `Profile`, `Encoder`)
- `GameType` enum with `NlheHeadsUp`, `KuhnPoker`, `LiarsDice`, `Custom(String)`
- `StrategyQuery<I>` / `StrategyResponse<E>` wire types with validation
- Property-based testing via `proptest` for serialization roundtrips
- Extensible via `#[non_exhaustive]` enums and `Custom` variants

### Poker Engine (`myosu-games-poker`)
- Full `PokerSolver` wrapping robopoker's `NlheProfile` + `NlheEncoder`
- Binary wire codec for strategy queries/responses
- Artifact loading from directory (encoder) and file (checkpoint)
- `NlheRenderer` for TUI and pipe output with solver advisor overlay
- Blueprint system for demo and artifact-backed gameplay

### Multi-Game Proofs
- **Liar's Dice**: Complete MCCFR solver (`LiarsDiceSolver<N>` with generic tree count),
  wire codec, renderer, full test coverage. 2-player, configurable dice/faces.
- **Kuhn Poker**: Complete analytical solver (closed-form Nash), wire codec,
  renderer. Simplest possible game for testing the full seam.

### Miner Binary (`myosu-miner`)
- Chain probe, registration, axon serving
- Bounded MCCFR training via `run_training_batch`
- Strategy serving (one-shot file and HTTP axon)
- Clean CLI with `clap` derive

### Validator Binary (`myosu-validator`)
- Deterministic scoring via L1 distance between expected/observed distributions
- Score normalization: `1.0 / (1.0 + l1_distance)`
- Multi-game support (poker + Liar's Dice)
- On-chain weight submission path
- INV-003 determinism tested

### Gameplay Surface (`myosu-play`)
- Three modes: `--smoke-test`, `train` (TUI), `pipe` (agent-native)
- Chain discovery of best miner by incentive
- Live miner query with staleness tracking
- Multi-game support via `GameSelection` enum
- Extensive test coverage (pipe responses, smoke reports, discovery, live advice)

### Chain Layer
- Stripped Substrate runtime with ~9 pallets in default build
- `pallet-game-solver` (renamed subtensor) with stage-0 extrinsic surface
- No-op swap stub (`NoOpSwap<B>`) satisfying 37 callsites
- Devnet/testnet chain specs
- Node-owned `--stage0-local-loop-smoke` proof

### TUI (`myosu-tui`)
- Shell with screens (Lobby, Onboarding, Table, Loading)
- Event system with `UpdateEvent`
- `PipeMode` for agent-native text interface
- `GameRenderer` trait for game-agnostic rendering

### Key Management (`myosu-keys`)
- Create, import (keyfile/mnemonic/raw seed), export, list, switch, change password
- Network-aware (devnet, testnet)
- Password via environment variable (not CLI arg)

### CI Pipeline
- 9 parallel jobs: repo-shape, Python QA, active-crates, chain-core,
  integration-e2e, doctrine, dependency-audit, plan-quality, operator-network, chain-clippy
- INV-004 dependency boundary check
- `cargo audit` with documented ignore list
- E2E: `local_loop.sh`, `validator_determinism.sh`

### E2E Integration
- `local_loop.sh`: Full miner/validator/play proof on live devnet
- `validator_determinism.sh`: Cross-validator scoring agreement
- `emission_flow.sh`: On-chain emission distribution proof
- `two_node_sync.sh`: Named-network peer discovery proof

---

## What Is Broken or Half-Built

### Inherited Chain Complexity
- `pallet-game-solver` is a renamed copy of `pallet-subtensor` with ~200+ storage items,
  only ~80 needed for stage-0. The inherited epoch/coinbase code is ~4000 lines
  carrying AMM, Alpha/TAO dual-token, root-network, and multi-subnet logic
  that the stage-0 single-token model does not use.
- `pallet-subtensor` (the original) still exists alongside `pallet-game-solver`
  as a parallel copy. Both have identical test directories (44 test files each).
  This is the single largest source of dead code in the repo.
- Legacy pallets (`admin-utils`, `drand`, `crowdloan`, `registry`, `proxy`,
  `swap`, `swap-interface`, `transaction-fee`, `utility`) still exist as
  source directories even though they are feature-gated out of the default build.

### Emission Accounting Gaps
- `run_coinbase.rs` carries inherited logic for root-network emission,
  multi-subnet weighted distribution, and AMM-based token conversion that
  the stage-0 identity-swap model renders dead.
- The `NoOpSwap` stub passes all 1:1 amounts through, which means emission
  distribution "works" but doesn't actually test meaningful economic behavior.
- `emission_flow.sh` exists but the on-chain emission assertion surface
  is thinner than the coinbase code complexity suggests it should be.

### Python Research Layer
- `main.py`, `methods.py`, `runner.py`, `metrics.py`, `data.py` (~125K lines)
  are a parallel research/simulation layer. CI runs `ruff check` and two test files.
  This layer is not connected to the Rust codebase.
- No dependency management (`requirements.txt` or `pyproject.toml` with deps).
  CI installs `numpy pytest ruff` ad-hoc.

### Documentation Staleness
- `IMPLEMENTATION_PLAN.md` (78K) is a massive historical artifact. Many items
  are checked off but the document itself references stale code paths.
- `THEORY.MD` (97K) is useful historical context but some specific claims
  are overtaken by code changes.
- `AGENTS.md` references `pallet_subtensor::Config` dependencies and
  audit findings that predate the rename to `pallet-game-solver`.

### Operator Experience Gaps
- No Docker/container packaging
- No systemd units shipped (though `deploy-bootnode.sh` generates one)
- `myosu-keys` requires `MYOSU_KEY_PASSWORD` env var but there is no
  first-run guidance for key creation flow
- Multi-node devnet requires manual bootnode coordination

---

## Tech Debt Inventory

| Area | Severity | Description |
|------|----------|-------------|
| Dual pallet copies | HIGH | `pallet-subtensor` and `pallet-game-solver` are parallel copies with identical test suites |
| Inherited epoch/coinbase complexity | HIGH | ~4000 lines of AMM/root-network/multi-token logic behind identity swap |
| Legacy pallet source dirs | MEDIUM | 10 pallet directories still exist even when feature-gated out |
| Python dependency management | MEDIUM | No `requirements.txt` or lockfile for research layer |
| `IMPLEMENTATION_PLAN.md` staleness | LOW | 78K historical doc with stale cross-references |
| `THEORY.MD` size | LOW | 97K single document; useful but unwieldy |
| Inherited storage items | MEDIUM | ~194 storage items in game-solver pallet, ~80 needed |
| `cargo audit` ignore list | MEDIUM | 7 RUSTSECs suppressed from inherited Substrate stack |
| `actions/checkout@v6` without SHA pin | LOW | CI uses tag-based action refs, not SHA-pinned |
| Test duplication in chain pallets | HIGH | 44 test files duplicated between subtensor and game-solver |

---

## Security Risks

| Risk | Severity | Detail |
|------|----------|--------|
| No-op swap stub disables slippage protection | HIGH (future) | `max_price` returns `u64::MAX`. Safe for stage-0 identity swaps but must not ship with real AMM |
| Inherited `unsafe` blocks in Substrate | MEDIUM | Not myosu-authored; inherited from polkadot-sdk fork |
| 7 suppressed cargo audit advisories | MEDIUM | All in inherited Substrate/opentensor fork deps (ring, wasmtime, tracing-subscriber) |
| Key password via env var | LOW | Correct approach but env vars can leak via /proc on shared systems |
| No rate limiting on miner HTTP axon | MEDIUM | Axon serves strategy via HTTP with no auth or rate limiting |
| Chain spec hardcoded test authorities | LOW | `//Alice`, `//Bob` authority URIs in devnet specs; appropriate for stage-0 |
| `SKIP_WASM_BUILD=1` in CI | LOW | Acceptable for off-chain crate tests but means CI doesn't verify wasm compilation |

---

## Test Coverage Assessment

### Well-Tested Areas
- Game trait serialization (proptest roundtrips)
- Poker solver wire codec
- Validator deterministic scoring (INV-003)
- Gameplay pipe protocol
- TUI shell state machine
- Swap stub identity behavior
- Kuhn poker solver (analytical Nash)
- Liar's Dice solver convergence

### Test Gaps
- **On-chain emission flow end-to-end**: `emission_flow.sh` exists but assertion surface is thin
- **Multi-node consensus**: `two_node_sync.sh` tests peer discovery but not consensus finality under adversarial conditions
- **Miner training convergence**: Bounded training tested but no convergence quality assertion
- **Live miner HTTP query**: Tested only via mock in `myosu-play`, no integration test against real axon
- **Key management edge cases**: Basic CLI operations tested, password change and export less so
- **Chain runtime upgrade**: No upgrade/migration test path
- **Negative security tests**: No fuzzing, no adversarial input testing on wire codecs

---

## Implementation Status Table (Prior Plans)

| Plan | Claimed Status | Verified Against Code |
|------|---------------|----------------------|
| 002 - Spec/doctrine sync | Complete | VERIFIED: Doctrine hierarchy consistent |
| 003 - Runtime reduction | Complete | VERIFIED: 9-pallet default build confirmed |
| 004 - Minimal devnet node | Complete | VERIFIED: Node builds, authors blocks |
| 005 - Stage-0 pallet reduction | Complete | VERIFIED: ≤20 extrinsics in default build |
| 006 - Game boundary hardening | Complete | VERIFIED: INV-004 CI gate exists |
| 007 - Miner/validator bring-up | Complete | VERIFIED: Both binaries functional |
| 008 - Artifact/wire hardening | Complete | VERIFIED: Binary wire codec, checkpoint save/load |
| 009 - Poker play/TUI productization | Complete | VERIFIED: Train + pipe modes, smoke tests |
| 010 - GitHub Actions proof | Complete | VERIFIED: CI pipeline with 9 jobs |
| 011 - Security/observability | Complete | VERIFIED: Security audit doc, cargo audit in CI |
| 012 - Multi-game proof | Complete | VERIFIED: Liar's Dice + Kuhn poker |
| 013 - Integration harness | Complete | VERIFIED: E2E scripts in `tests/e2e/` |
| 020 - Second-game subnet proof | Complete | VERIFIED: Multi-subnet local loop |
| 021 - Operator hardening | In Progress | PARTIALLY VERIFIED: Key management, operator bundle, bootnode prep exist. Multi-node devnet not yet closed. |

---

## Code-Review Coverage

The following source files were directly read during this assessment:

| File | Lines | Purpose |
|------|-------|---------|
| `Cargo.toml` (workspace) | 201 | Workspace members, dependencies, fork pins |
| `README.md` | 143 | Entry point and proof commands |
| `OS.md` | 260 | Operating system document, stage-0 criteria |
| `INVARIANTS.md` | 86 | Six hard invariants (INV-001 through INV-006) |
| `AGENTS.md` | 355 | Kernel document with architecture, audit findings, priorities |
| `SECURITY.md` | 90 | Vulnerability reporting and scope |
| `SPEC.md` | 151 | Spec type definitions and workflow |
| `THEORY.MD` | 100 (first) | Stage-0 operating theory and history |
| `IMPLEMENTATION_PLAN.md` | 100 (first) | Implementation queue with RT-001 through RT-004 |
| `.github/workflows/ci.yml` | 347 | Full CI pipeline definition |
| `crates/myosu-chain/pallets/game-solver/src/lib.rs` | 100 (first) | Pallet entry point, Stage0SwapInterface |
| `crates/myosu-chain/pallets/game-solver/src/swap_stub.rs` | 189 | NoOpSwap implementation |
| `crates/myosu-chain/pallets/game-solver/src/epoch/run_epoch.rs` | 80 (first) | Yuma epoch mechanism |
| `crates/myosu-chain/pallets/game-solver/src/coinbase/run_coinbase.rs` | 80 (first) | Coinbase emission distribution |
| `crates/myosu-games/src/traits.rs` | 500 | Game trait system, StrategyQuery/Response |
| `crates/myosu-miner/src/main.rs` | 93 | Miner entry point |
| `crates/myosu-validator/src/validation.rs` | 668 | Validator scoring logic |
| `crates/myosu-play/src/main.rs` | 1780 | Play surface entry point |

Additionally, subagents performed thorough exploration of:
- All chain pallets (game-solver, subtensor, admin-utils, drand, crowdloan, swap, etc.)
- All off-chain crates (games, poker, liars-dice, kuhn, play, miner, validator, keys, tui, chain-client)
- Git history (50 recent commits)
- E2E test scripts
- Operator guide and architecture docs
- Previous genesis planning snapshot

---

## Assumption Ledger

### Verified Assumptions
- The chain produces blocks on local devnet (E2E proven)
- The no-op swap stub satisfies all callsites without runtime errors
- Poker, Liar's Dice, and Kuhn all implement the game trait seam cleanly
- Validator deterministic scoring works for both poker and Liar's Dice
- INV-004 (solver-gameplay separation) holds with zero dependency paths
- The game trait system is extensible via `#[non_exhaustive]` and `Custom` variants

### Assumptions Needing Proof
- **Emission accounting correctness**: Inherited coinbase logic produces correct
  distributions under the identity-swap model. The test surface covers basic flow
  but not edge cases (e.g., what happens with zero-stake subnets, empty epochs).
- **Two-node consensus**: `two_node_sync.sh` tests peer discovery and block sync
  but not GRANDPA finality under partition or validator disagreement.
- **Miner training convergence quality**: The bounded training produces a checkpoint
  but there is no automated assertion that the strategy has converged to a
  useful quality level.
- **Operator bundle completeness**: `prepare_operator_network_bundle.sh` produces
  a bundle but the bundle has not been tested on a fresh machine.

### Hypotheses (Open Questions)
- Can the inherited Yuma Consensus code be simplified for single-subnet stage-0
  without breaking the emission distribution invariant?
- What is the minimum viable encoder size for meaningful poker strategy quality?
  (Full encoder is 7-11 GB RAM with 138M entries.)
- Is the opentensor polkadot-sdk fork essential, or can myosu migrate to
  upstream polkadot-sdk with acceptable effort?

---

## Opportunity Framing

### Strongest Direction: Close Stage-0, Prepare for Multi-Node Devnet

The repo is remarkably close to stage-0 exit. The local loop is proven. The
remaining work is:

1. **Harden emission accounting** -- simplify inherited coinbase, prove the
   invariant `sum(distributions) == block_emission * epochs` robustly.
2. **Multi-node devnet proof** -- two-node sync exists; extend to 3+ nodes
   with automated finality checks.
3. **Operator packaging** -- container images, stable chain spec distribution,
   first-run documentation that works on a fresh machine.

This direction is preferred because it builds directly on proven assets and
closes the gap between "local proof" and "operator-ready devnet."

### Rejected Direction: Broad Game Portfolio Expansion

Adding more games now would be premature. The three-game proof (poker, Liar's
Dice, Kuhn) is sufficient to validate multi-game architecture. Adding games
before the emission and network layers are solid would spread effort without
increasing stage-0 confidence.

### Rejected Direction: Web/Mobile Gameplay Surface

Building a web or mobile frontend for gameplay is explicitly out of scope for
stage-0. The TUI and pipe interface serve both human and agent consumption.
A web surface is a post-stage-0 product decision, not a stage-0 engineering task.

### Rejected Direction: Full Chain Rewrite

Rewriting `pallet-game-solver` from scratch instead of continuing to reduce
the inherited subtensor code would be higher risk with no near-term payoff.
The reduction strategy (strip, stub, test) has been proven to work and is
more predictable than a ground-up rewrite.

---

## DX Assessment (Developer-Facing Surfaces)

### First-Run Friction
- `README.md` has clear proof commands but assumes Rust toolchain + wasm target
  are already installed. Missing: `rustup target add wasm32-unknown-unknown` in quickstart.
- `SKIP_WASM_BUILD=1` is required for most commands but not explained until
  deep in `AGENTS.md`. A developer running bare `cargo test` will wait for a
  full wasm build.
- The `fabro` / `raspberry` execution model is referenced throughout `OS.md`
  and `README.md` but these tools are not standard Rust ecosystem tools and
  may confuse new contributors.

### Copy-Paste Onboarding Honesty
- The proof commands in `README.md` are honest -- they work as documented.
- The operator loop commands assume `fabro` and `raspberry` are installed,
  which a new developer will not have.
- The E2E test scripts are the most honest runnable surface.

### Error Clarity
- Error types are well-structured with `thiserror` across miner, validator,
  and play surfaces.
- Chain-level errors inherit Substrate's generic dispatch error types, which
  are less helpful.

### Fastest Path to Success Moment
- `cargo run -p myosu-play --quiet -- --smoke-test` produces a meaningful
  success output in seconds (after initial compile). This is a good T0 moment.
- The compile time for the full workspace is significant due to Substrate
  dependencies. First build can take 10+ minutes.

---

## Constraints

1. **Substrate dependency**: The opentensor polkadot-sdk fork is a hard constraint.
   Build times, dependency complexity, and upstream divergence risk all flow from this.
2. **Robopoker fork**: The MCCFR engine comes from a forked repo with documented
   changes. Core algorithm correctness depends on tracking the fork (INV-006).
3. **Single developer**: Git history shows primarily automated commits (`myosu: auto bug checkpoint`,
   `myosu: review completed items`). This is effectively a single-developer project
   with heavy automation assistance.
4. **Memory requirements**: Full NLHE encoder is 7-11 GB. This constrains who
   can run a production-grade miner.
5. **Stage-0 scope discipline**: The repo has been intentionally aggressive about
   not expanding scope beyond the local proof. This is a strength, not a constraint,
   but it means the planning corpus should respect that posture.

# Genesis Assessment (Phase 1: Think)

Date: 2026-03-28
Scope audited: repository root docs, all files in `plans/` and `specs/`, active crates under `crates/`, CI workflow, recent git history, and dependency graph.

## 1. Demand Reality

There is no evidence of external production demand yet.

What we can prove right now:
- Real internal usage exists: the repo is actively operated via Fabro/Raspberry artifacts (`fabro/`, `.raspberry/`, `outputs/`).
- The playable value surface exists today: `myosu-play` + `myosu-games-poker` + `myosu-tui` compile and run locally.
- No evidence of external payments, user telemetry, or "panic if it vanished" behavior was found in-repo.

Assessment: this is an advanced prototype with internal operator usage, not yet a market-proven product.

## 2. Status Quo

Without Myosu, the workflow for the target problem is:
- Use standalone poker solvers/trainers (`robopoker`, commercial trainers, custom scripts) for strategy quality.
- Use direct Substrate/Bittensor forks or pure off-chain experiments for incentive-market research.
- Use ad-hoc docs/plans and local scripts for project orchestration.

Pain Myosu is trying to remove:
- Fragmentation between solver quality, validator scoring, incentives, and a user-facing gameplay experience.
- Lack of one coherent loop from training -> scoring -> emissions -> playability.

Current practical alternative in this repo today is still “duct-taped local demo + heavy chain fork in progress,” not a finished integrated product.

## 3. Desperate Specificity

The one person this codebase is currently for:

**Role**: Solo protocol engineer / founder-operator building a verifiable game-solving network.

What keeps them up at night:
- "Can I prove deterministic validator scoring (INV-003) before scaling claims?"
- "Can I reduce the chain fork complexity enough to ship a believable stage-0 devnet?"
- "Can I keep plans/specs/doctrine coherent while still shipping code?"

What gets them promoted/funded:
- A reproducible demo where a chain-backed solver market produces measurable quality and a human can play against it.

## 4. Narrowest Wedge (What Someone Could Depend On This Week)

The smallest weekly value wedge is:

**A deterministic, local NLHE training and advisor experience (`myosu-play train` / `myosu-play pipe`) backed by artifact-loaded strategy advice.**

Why this wedge is real now:
- Compiles and runs today in active crates.
- Has extensive tests across `myosu-play`, `myosu-games-poker`, and `myosu-tui`.
- Does not require the chain/runtime to be fully stabilized.

What is not a weekly wedge yet:
- Full miner/validator emissions market on local devnet with deterministic cross-validator agreement proof end-to-end.

## 5. Observation & Surprise

Surprises from exploration:
- The repo has strong local product implementation momentum (TUI/play/poker) and strong chain code volume, but very weak alignment between docs and code status.
- `specs/031626-07-tui-implementation.md` and `specs/031626-10-agent-experience.md` are empty while being referenced as active.
- The `specs/` directory contains many duplicate legacy mirror specs alongside numbered specs, causing active-control-plane ambiguity.
- `specs/031626-11-agent-coordination-mechanism.md` and `specs/031626-12-nlhe-incentive-mechanism.md` are exact duplicates with mismatched naming intent.
- CI gates only the gameplay-facing crates; chain/runtime/node are mostly off-gate despite being central to the mission.
- The chain runtime/node remain heavily subtensor/frontier-shaped, conflicting with several strip-down claims in planning docs.

Half-built in revealing ways:
- `pallet-game-solver` contains stubs (`stubs.rs`, `swap_stub.rs`) that represent intended decoupling, but they are not wired into active module flow.
- `node/src/chain_spec/devnet.rs` and `testnet.rs` are placeholders, while localnet/legacy surfaces remain broad.
- Background operational artifacts (`outputs/*`) include reviews that still describe already-shipped surfaces as "not yet built," proving doc drift.

## 6. Future-Fit

The project’s core bet:
- **Verifiable quality markets for imperfect-information strategy computation** (miners compute, validators score exploitability, chain allocates incentives).

How it compounds if executed:
- Shared game-trait abstractions + repeatable validation + operator tooling can become a reusable protocol substrate for multiple games.

How it decays if not corrected:
- Spec/plan drift and chain complexity will keep trust and shipping velocity low.
- Duplicate doctrine surfaces will keep decisions reversible in the wrong places (organizational entropy).
- CI blind spots on chain/runtime will allow subtle regressions to accumulate until integration breaks late.

## What Works, What’s Broken, What’s Half-Built

### What Works

- Active gameplay stack compiles (`myosu-games`, `myosu-games-poker`, `myosu-tui`, `myosu-play`).
- Rich unit/integration-style test surface in the gameplay stack.
- `myosu-play` smoke path exists and is CI-gated.
- Fabro/Raspberry scaffolding and outputs discipline exist and are actively used.
- `PLANS.md` and `SPEC.md` authoring standards are clear and useful.

### What’s Broken

- Active doctrine inconsistency across `specs/`, `plans/`, and output reviews.
- Missing active spec content (empty 031626-07 and 031626-10).
- Duplicate/legacy specs still in active `specs/` namespace.
- CI does not gate chain/runtime/node surfaces.
- Existing plan set (`plans/`) depends on missing `specs/031826-*` references and behaves more like historical migration logs than live execution plans.

### What’s Half-Built

- Chain simplification intent exists, but runtime/node/pallet still carry significant subtensor/frontier complexity.
- Stub decoupling path exists in code but is not fully adopted.
- Multi-game strategy is described in depth but execution is still concentrated in poker + TUI local surfaces.

## Tech Debt Inventory

| Area | Debt | Evidence (file paths) | Impact |
|---|---|---|---|
| Spec corpus hygiene | Empty active specs | `specs/031626-07-tui-implementation.md`, `specs/031626-10-agent-experience.md` | Breaks planning integrity; downstream specs depend on missing content |
| Spec duplication | Mirror legacy specs in active namespace | `specs/031626-*.md` (non-numbered mirrors), `specs/031626-11-agent-coordination-mechanism.md` == `031626-12-nlhe-incentive-mechanism.md` | Conflicting source of truth |
| Plan dependency integrity | Plans depend on missing `031826` specs | `plans/031826-*.md`, `plans/031926-*.md` | Execution plans cannot be cleanly replayed by novice operator |
| Chain surface complexity | Runtime still broad and frontier-heavy | `crates/myosu-chain/runtime/src/lib.rs`, `crates/myosu-chain/node/src/lib.rs`, `crates/myosu-chain/node/Cargo.toml` | High maintenance and integration risk |
| Stub integration debt | Decoupling stubs not wired | `crates/myosu-chain/pallets/game-solver/src/stubs.rs`, `swap_stub.rs`, `lib.rs` module wiring | Intended simplification path stalled |
| Node chain spec completeness | Placeholder chain specs | `crates/myosu-chain/node/src/chain_spec/devnet.rs`, `testnet.rs` | Incomplete operational readiness |
| CI coverage debt | Chain/runtime/node not gated | `.github/workflows/ci.yml` | Regressions escape into trunk |
| Ops doc freshness | Reviews/specs stale vs current code | `outputs/play/tui/review.md`, multiple `outputs/*/review.md` | Misleading operator decisions |
| Workspace packaging clarity | Checked-in manifests not active/healthy as workspace packages | `crates/myosu-chain/pallets/subtensor/Cargo.toml`, `support/linting/Cargo.toml`, `support/procedural-fork/Cargo.toml`, `support/tools/Cargo.toml` | Contributor confusion and tooling failures |

## Security Risks Found

| Risk | Evidence (file paths) | Severity | Notes |
|---|---|---|---|
| Large attack surface retained in runtime/node | `crates/myosu-chain/runtime/src/lib.rs`, `crates/myosu-chain/node/Cargo.toml` | High | Frontier/EVM/swap/drand/crowdloan surfaces retained while stage-0 hardening is incomplete |
| Untrusted artifact loading for advice | `crates/myosu-play/src/main.rs`, `crates/myosu-games-poker/src/codexpoker.rs` | Medium | Auto-loading from local dirs without trust roots/signing policy can poison advice outputs |
| Unsafe mmap use paths | `crates/myosu-games-poker/src/codexpoker.rs` | Medium | `unsafe` mmap is expected but requires strict validation/limits discipline |
| bincode decode without explicit size caps in several surfaces | `crates/myosu-games-poker/src/wire.rs`, `solver.rs`, `artifacts.rs` | Medium | Potential memory/DoS exposure if any untrusted bytes enter boundary |
| CI blind spot for chain security-critical surfaces | `.github/workflows/ci.yml` | High | Security regressions in runtime/pallet/node can merge undetected |

## Test Coverage Gaps (Specific Untested/Under-Gated Modules)

### Not CI-Gated (even where local tests exist)
- Entire chain family (`pallet-game-solver`, runtime, node, swap/transaction-fee/admin-utils/drand/etc.)
  - CI currently only gates `myosu-games`, `myosu-tui`, `myosu-games-poker`, `myosu-play`.

### Sparse/No Direct Tests
- Node operational core:
  - `crates/myosu-chain/node/src/service.rs`
  - `crates/myosu-chain/node/src/rpc.rs`
  - `crates/myosu-chain/node/src/command.rs`
  - `crates/myosu-chain/node/src/chain_spec/devnet.rs` (placeholder)
  - `crates/myosu-chain/node/src/chain_spec/testnet.rs` (placeholder)
- Runtime helper modules with thin direct tests:
  - `crates/myosu-chain/runtime/src/check_nonce.rs`
  - `crates/myosu-chain/runtime/src/migrations.rs`
  - `crates/myosu-chain/runtime/src/sudo_wrapper.rs`
  - `crates/myosu-chain/runtime/src/transaction_payment_wrapper.rs`
- Boundary RPC/API/support crates with minimal/no tests:
  - `crates/myosu-chain/pallets/swap-interface/src/lib.rs`
  - `crates/myosu-chain/pallets/swap/rpc/src/lib.rs`
  - `crates/myosu-chain/pallets/swap/runtime-api/src/lib.rs`
  - `crates/myosu-chain/pallets/subtensor/rpc/src/lib.rs`
  - `crates/myosu-chain/pallets/subtensor/runtime-api/src/lib.rs`
  - `crates/myosu-chain/support/tools/src/bump_version.rs`
  - `crates/myosu-chain/support/tools/src/spec_version.rs`

## One-Sentence Reality

Myosu is currently a strong local poker-training-and-advisor product layer attached to an ambitious but still over-complex and under-gated chain fork whose documentation control plane has drifted out of sync with implementation reality.

## Existing Plan Assessment (`plans/`)

| Existing plan | Rating | Why | Genesis action |
|---|---|---|---|
| `plans/031826-clean-up-myosu-for-fabro-primary-executor.md` | Missing context | Depends on missing `specs/031826-*`; mostly historical migration work already executed | Replace with dropped-plan record + preserve learnings in governance/spec normalization plan |
| `plans/031826-bootstrap-fabro-primary-executor-surface.md` | Missing context | Useful history, but now execution journal; same missing dependency issue | Replace with dropped-plan record; harvest durable rules into `genesis/SPEC.md` + plan 002 |
| `plans/031926-design-myosu-fabro-workflow-library.md` | Strong but misplaced | Good decomposition ideas, but belongs in durable spec/control-plane architecture, not active live plan | Merge insights into plan 002/011 and archive original via dropped-plan record |
| `plans/031926-decompose-myosu-into-raspberry-programs.md` | Strong but misplaced | Valuable frontier model; mixed with historical references and missing deps | Merge insights into plan 011 + master roadmap; archive original via dropped-plan record |
| `plans/031926-iterative-execution-and-raspberry-hardening.md` | Weak for active execution | Time-bound run journal with stale run IDs, pending tasks tied to old lane states | Replace with dropped-plan record and move active hardening into concrete CI/ops plans |


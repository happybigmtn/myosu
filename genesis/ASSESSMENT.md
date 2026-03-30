# Genesis Assessment

Date: 2026-03-28
Updated: 2026-03-30 for Genesis adjudication sync after later execution closed
the miner/validator loop, the local two-subnet multi-game proof, the main
`myosu-play` decomposition work, and most doctrine cleanup plans.
Scope: Full repository audit -- root doctrine, all specs/, all plans/, all crates/, CI, git history (50 commits), dependency graph, archived genesis runs.
Prior art: `archive/genesis_1774729423/ASSESSMENT.md` (same-day prior run) -- this assessment supersedes it with deeper code analysis and plan-quality review.

## 1. Demand Reality

There is no evidence of external production demand.

Proof of internal usage:
- The repo is actively operated via Fabro/Raspberry (`fabro/`, `outputs/`).
- `myosu-play` + `myosu-games-poker` + `myosu-tui` compile and run today as a local NLHE advisor.
- 1,575 total commits, 94.8% from Fabro (automated), 5.2% from `happybigmtn` (human operator).

Proof of no external demand:
- No payment integration, no telemetry, no user accounts.
- No external contributors, no issues, no PRs from outside.
- README targets developers, not users.

Assessment: advanced prototype with internal operator usage. Not yet a product anyone would panic without.

## 2. Status Quo

Without Myosu, the target user would:
- Buy PioSolver ($250-2,500) for NLHE strategy computation.
- Run ad-hoc MCCFR training via robopoker or academic implementations.
- Have no verifiable quality scoring, no incentive market, no multi-game generalization.
- Use no decentralized alternative -- none exists.

Pain Myosu removes: fragmentation between solver quality, verification, incentives, and playable output. The full loop (train -> score -> emit -> play) does not exist anywhere else.

Current practical state: "duct-taped local demo + heavy chain fork in progress."

## 3. Desperate Specificity

The ONE person this is for today:

**Solo protocol engineer / founder-operator** building a verifiable game-solving network with limited bandwidth.

What keeps them up at night:
- "Can I prove deterministic validator scoring (INV-003) before making claims?"
- "Can I reduce the 216K-line chain fork enough to ship a believable devnet?"
- "Are my 37 spec files actually consistent, or am I planning against contradictions?"

What gets them funded:
- A reproducible demo: chain produces blocks, solver submits strategy, validator scores it, human plays against it, quality is measurable.

## 4. Narrowest Wedge

The smallest thing someone could depend on THIS WEEK:

**`myosu-play train` / `myosu-play pipe` -- a local NLHE training and strategy advisor backed by MCCFR-trained blueprints.**

Why it's real:
- Compiles and runs today (CI-gated, smoke-tested).
- 4-tier artifact auto-discovery with graceful fallback.
- Full TUI with 8-screen state machine, pipe mode for agent integration.
- Does NOT require the chain to be stable.

What is NOT a weekly wedge:
- Full miner/validator emissions market on devnet with cross-validator determinism.

## 5. Observation and Surprise

### Surprising findings

1. **Implementation maturity mismatch.** The gameplay stack (myosu-games, myosu-games-poker, myosu-tui, myosu-play) is production-quality: 256K lines, comprehensive tests, no `unimplemented!()` calls, property-based testing via proptest. But CI only gates these 4 crates -- the entire chain side (216K lines, 13 pallets) flies blind.

2. **Spec corpus is actively misleading.** `specs/031626-07-tui-implementation.md` is 0 bytes. `specs/031626-10-agent-experience.md` is 0 bytes. Both are referenced as active in the master index. Meanwhile, `specs/031626-11` and `specs/031626-12` are byte-identical duplicates with different names. There are also 13 non-numbered mirror copies of numbered specs in the same directory.

3. **The chain is a Bittensor subtensor fork carrying enormous baggage.** The runtime has 20+ pallets, EVM/Frontier integration, dual-token AMM, drand VRF, crowdloan mechanics -- none of which are needed for stage-0. The `pallet-game-solver` is essentially a renamed copy of `pallet-subtensor` with 69 extrinsics, 95+ errors, 60+ events, and 50+ storage migrations. The game-solving pallet has stubs (`stubs.rs`, `swap_stub.rs`) that correctly implement no-ops, but the runtime still wires the full subtensor surface.

4. **Three previous genesis runs exist in archive.** `archive/genesis_1774727580/`, `archive/genesis_1774728012/`, and `archive/genesis_1774729423/`. The latest (1774729423) produced 11 plans plus 5 dropped records. Those plans are structurally sound but lack: ASCII architecture diagrams, failure mode tables in every plan, and concrete implementation substance (several are bootstrap-only plans whose deliverables are spec/review files rather than working code).

5. **myosu-play had a serious entry-point monolith at audit time.** The
   original audit found `myosu-play/src/main.rs` at 31,337 lines. Later `009`
   execution reduced that to 1,597 lines and extracted companion modules such
   as `cli.rs` and `blueprint.rs`, which means the finding was real but is no
   longer current in its original form.

6. **The robopoker dependency is an external fork.** `happybigmtn/robopoker` provides `rbp-core`, `rbp-mccfr`, `rbp-nlhe`, `rbp-cards`, `rbp-gameplay`. Pinned via git rev. INV-006 governs fork coherence. No CHANGELOG.md was found documenting divergence from v1.0.0 baseline.

### Half-built in revealing ways

- `node/src/chain_spec/devnet.rs` and `testnet.rs` are placeholders -- operational readiness is incomplete.
- At audit time the off-chain services were not yet part of the live proof
  story. Later execution moved both `myosu-miner` and `myosu-validator` into
  the workspace and into the node-owned stage-0 proof loop.
- `outputs/*/review.md` files describe surfaces as "not yet built" when the code already exists -- doc drift.
- `DESIGN.md` is referenced in `OS.md` but may not exist or may be stale.

## 6. Future-Fit

**Core bet:** Verifiable quality markets for imperfect-information strategy computation.

**How it compounds:**
- Shared game-trait abstractions + repeatable validation + tokenized incentives create a flywheel: more miners -> better strategies -> more players -> more revenue -> more miners.
- Multi-game generalization means each new game (20 planned) reuses the same chain, validator, and TUI infrastructure.
- Compute moat: trained MCCFR strategies take months to converge. Early miners have structural advantage that forks can't shortcut.

**How it decays if not corrected:**
- Spec/plan drift will erode trust in documentation faster than code improves.
- Chain complexity (216K lines of Bittensor baggage) will block devnet stability.
- CI blind spots on chain will allow subtle consensus regressions.
- Without a working miner/validator loop, the chain is an expensive static binary.

---

## What Works

| Surface | Evidence | Quality |
|---------|----------|---------|
| `myosu-games` | 6+ property-based tests, clean trait abstractions, GameRegistry | High |
| `myosu-games-poker` | 10 modules, 3,738 lines, MCCFR solver + 2 inference backends | High |
| `myosu-tui` | 16+ shell state tests, 8-screen state machine, pipe mode | High |
| `myosu-play` | Smoke test CI-gated, 4-tier artifact discovery, train+pipe modes | High |
| `PLANS.md` / `SPEC.md` meta-rules | Clear, actionable, well-structured | High |
| `INVARIANTS.md` | 6 hard rules with enforcement and measurement | High |
| `OS.md` | Complete autonomous company OS, mission, stages, revenue model | High |
| `ops/decision_log.md` | 20+ decisions with rationale | High |

## What's Broken

| Surface | Evidence | Impact |
|---------|----------|--------|
| Empty canonical specs | `specs/031626-07` (0 bytes), `specs/031626-10` (0 bytes) | Downstream plans reference missing content |
| Duplicate specs | `031626-11` == `031626-12` byte-identical; 13 non-numbered mirrors | Conflicting source of truth |
| Hosted CI closure still unproven | The workflow is now published and hosted runs exist on the current repo surface. Run `23730306070` showed the real remaining blockers: two environment-dependent `myosu-play` startup tests plus missing `protoc` in the chain jobs | The last `010` claim is now down to publishing those fixes and capturing one fully green hosted timing run |
| Named network specs remain skeletal | `node/src/chain_spec/devnet.rs`, `testnet.rs` are still tiny compared with `localnet.rs` | Broader non-local packaging is weaker than the local proof surface |
| Genesis corpus drift required adjudication | The original report and parts of this assessment kept describing an earlier active-plan set | Doctrine readers could misread what is actually live now |

## What's Half-Built

| Surface | State | Gap |
|---------|-------|-----|
| Named network packaging | The local owned smoke path is real, but named `devnet` / `testnet` surfaces are still thin | The local proof is stronger than the broader deployment story |
| Future synth governance | Genesis adjudication is now real and the synth procedure now lives in `genesis/PLANS.md` | The policy exists, but it still needs a future real-world synth pass to prove it is easy to follow |
| Multi-game live serving | Liar's Dice is proven locally through miner/validator/play and the two-subnet harness, but live HTTP advice is still poker-only | The second-game network surface is not yet symmetric with poker |
| Agent experience | Agent transports described in OS.md but no implementation | Empty spec, no code |

## Tech Debt Inventory

| # | Area | Debt | Evidence | Impact | Owning Plan |
|---|------|------|----------|--------|-------------|
| TD-01 | Spec corpus | Empty canonical specs + duplicates | `specs/031626-07`, `031626-10`, `031626-11==12` | Planning integrity | 002 |
| TD-02 | CI closure evidence | Hosted runs now exist on the current repo surface, but the long cargo lanes have not yet been recorded to completion in doctrine | The final `010` claim is not yet fully proved | 010 |
| TD-03 | Chain runtime | 20+ pallets when 6 needed | `crates/myosu-chain/runtime/src/lib.rs` | Maintenance burden, compile time, attack surface | 003 |
| TD-04 | Chain node packaging | Named `devnet` / `testnet` chain specs remain skeletal | The localnet/operator proof is stronger than the broader packaged surface | 004 |
| TD-05 | Workspace packaging | Non-member crates checked in | `pallets/subtensor/Cargo.toml`, `support/linting`, `support/procedural-fork`, `support/tools` | Contributor confusion | 003 |
| TD-06 | Genesis corpus freshness | Report/assessment drifted after later execution work | Readers could inherit the wrong active-plan set | 018 |
| TD-07 | Play entrypoint size | `myosu-play/src/main.rs` is still 1,597 lines after decomposition | Maintenance and review burden remain non-trivial | 009 |
| TD-08 | Robopoker fork | No CHANGELOG.md documenting divergence from v1.0.0 | INV-006 | Undocumented algorithm changes | 006 |
| TD-09 | Artifact trust | Auto-loading from local dirs without signing | `codexpoker.rs`, `artifacts.rs` | Poisoned advice | 008 |

## Security Risks

| # | Risk | Evidence | Severity | Owning Plan |
|---|------|----------|----------|-------------|
| SR-01 | Large runtime attack surface | 20+ pallets including EVM/Frontier | High | 003 |
| SR-02 | Untrusted artifact loading | `crates/myosu-play/src/main.rs` auto-discovery | Medium | 008 |
| SR-03 | Unsafe mmap without bounds | `crates/myosu-games-poker/src/codexpoker.rs` | Medium | 008 |
| SR-04 | bincode decode without size caps | `wire.rs`, `solver.rs`, `artifacts.rs` | Medium | 008 |
| SR-05 | Hosted CI closure still open after the first full current-surface run | Hosted runner evidence now exists on the current repo surface; run `23730306070` passed `Stage-0 Repo Shape`, `Plan Quality`, and `Doctrine Integrity`, then failed concretely on two `myosu-play` startup tests plus missing `protoc` in the chain jobs | Medium | 010 |
| SR-06 | No key management infrastructure | `myosu-keys` crate doesn't exist | High | 011 |

## Test Coverage Gaps

### Not CI-gated (even where local tests exist)
- Entire chain family: `pallet-game-solver` (35 test modules), `pallet-subtensor` (35 test modules), runtime, node, all supporting pallets.
- Total chain test code: ~56K lines in subtensor tests alone.

### Sparse or no direct tests
- `node/src/service.rs`, `rpc.rs`, `command.rs` -- node operational core
- `node/src/chain_spec/devnet.rs`, `testnet.rs` -- placeholder chain specs
- `runtime/src/check_nonce.rs`, `migrations.rs`, `sudo_wrapper.rs`, `transaction_payment_wrapper.rs`
- `pallets/swap-interface/src/lib.rs`, swap RPC, subtensor RPC, subtensor runtime API
- `support/tools/src/bump_version.rs`, `spec_version.rs`

### Missing test categories
- Hosted CI evidence now exists, but it does not yet cover the current local
  stage-0 repo surface because the published remote repo is behind
- Liar's Dice does not yet have a live HTTP miner-query proof analogous to poker
- Future synth governance is now documented, but it has not yet been exercised
  by a post-policy synth run

## One-Sentence Reality

Myosu is now a locally proven two-game stage-0 chain loop with one remaining
hosted-CI closure gap caused by specific fixable hosted blockers and a still-
messy canonical spec corpus.

## Existing Plan Assessment

### Prior genesis plans (archive/genesis_1774729423/plans/)

| Plan | Rating | Strengths | Weaknesses | Genesis Action |
|------|--------|-----------|------------|----------------|
| 001-master-plan | Strong | Clear 4-phase structure, dependency graph | No ASCII diagrams, vague phase boundaries | Enhance with diagrams, tighter milestones, specific proof commands |
| 002-spec-corpus-normalization | Strong | Correct problem identification, clear milestones | Proof commands are file-existence checks, not content validation | Enhance with content-quality gates |
| 003-chain-runtime-reduction | Strong | Correct scope (structural only), 8-file cap | Missing: which pallets specifically stay/go, no architecture diagram | Enhance with explicit allowlist and ASCII diagram |
| 004-node-devnet-minimalization | Strong | Good separation from runtime plan | Missing: what "minimal devnet" actually means in concrete terms | Enhance with devnet definition |
| 005-pallet-game-solver-simplification | Strong | Correct identification of stub path | Missing: which of 69 extrinsics survive stage-0 | Enhance with extrinsic allowlist |
| 006-game-traits-and-poker-boundaries | Weak (bootstrap-only) | Good problem framing | All milestones produce spec/review files, not code. This is a planning plan, not an implementation plan. The game traits already exist and work. | Replace with implementation-focused plan that extends existing working code |
| 007-miner-validator-bootstrap | Strong | Correct architecture (shared client), INV-003 harness | Missing: what the miner actually does (MCCFR step), what the validator actually scores | Enhance with concrete service behavior |
| 008-artifact-wire-checkpoint-hardening | Strong | Correct security concerns identified | Missing: specific size caps, signing scheme, version format | Enhance with concrete security mitigations |
| 009-play-tui-productization | Strong | Excellent IA mockups, state taxonomy | Missing: concrete code changes needed, test names that don't exist yet | Enhance with implementation specifics |
| 010-ci-proof-gates-expansion | Strong | Doctrine checks in CI is excellent idea | Missing: what specifically the chain CI job runs | Enhance with exact CI job definitions |
| 011-security-observability-release | Not fully reviewed | Likely strong framing | Likely missing specifics | Enhance |

### Prior dropped-plan records

| Record | Disposition | Notes |
|--------|------------|-------|
| 012-dropped-031826-clean-up-myosu-for-fabro | Correct drop | Work already executed |
| 013-dropped-031826-bootstrap-fabro-executor | Correct drop | Work already executed |
| 014-dropped-031926-design-fabro-workflow | Correct drop | Insights merged into active plans |
| 015-dropped-031926-decompose-raspberry | Correct drop | Insights merged into active plans |
| 016-dropped-031926-iterative-execution | Correct drop | Stale run journal |

### Active specs assessment (specs/)

| Spec | Status | Quality | Notes |
|------|--------|---------|-------|
| 031626-00-master-index | Active | High | Comprehensive but references empty specs |
| 031626-01-chain-fork-scaffold | Active | High | 11 ACs, detailed, well-reasoned |
| 031626-02a-game-engine-traits | Active | High | Clear rationale, realistic scope |
| 031626-02b-poker-engine | Active | High | Clean dependency on 02a |
| 031626-03-game-solving-pallet | Active | High | 10 ACs, Yuma fidelity requirement clear |
| 031626-04a-miner-binary | Active | Medium | Good but needs update for current code state |
| 031626-04b-validator-oracle | Active | Medium | Good but INV-003 harness needs more detail |
| 031626-05-gameplay-cli | Active | Medium | Partially superseded by working myosu-play |
| 031626-06-multi-game-architecture | Active | Medium | Liar's Dice proof is now closed locally; the remaining gap is live second-game network symmetry |
| 031626-07-tui-implementation | **EMPTY** | None | 0 bytes, referenced as active |
| 031626-08-abstraction-pipeline | Active | Medium | Important for miner artifact quality |
| 031626-09-launch-integration | Active | Medium | End-to-end integration spec |
| 031626-10-agent-experience | **EMPTY** | None | 0 bytes, referenced as active |
| 031626-11-agent-coordination | Active | Low | Duplicate of 031626-12 |
| 031626-12-nlhe-incentive | Active | Medium | Canonical incentive spec (duplicate of 11) |
| 031626-13-n-player-trait | Active | Medium | N-player generalization design |
| 031626-14-poker-variant-family | Active | Medium | PLO/6-max/short-deck planning |
| 031626-15-key-management | Active | Medium | Key infrastructure design |
| 031626-16-cross-game-scoring | Active | Medium | Cross-game quality normalization |
| 031626-17-spectator-protocol | Active | Low | Aspirational, no implementation path |
| 031626-18-operational-rpcs | Active | Medium | Runtime API additions |
| 031626-19-game-engine-sdk | Active | Medium | SDK scaffold for third-party games |
| 031626-99 legacy executor enhancement spec | Historical drift | Low | Already historical-only in `specsarchive/`; keep out of the active control plane |
| 13 non-numbered mirrors | Legacy | Low | Duplicates of numbered specs, should be archived |

# Genesis Report

Generated: 2026-04-07
Codebase snapshot: trunk @ 4e0b37f
Previous corpus: .auto/fresh-input/genesis-previous-20260407-234710

---

## Corpus Summary

This genesis corpus was produced from a full-depth review of the myosu repository. Five parallel exploration agents read across all 11 workspace crates, 267K lines of Rust, 2,771 test functions, 46 spec files, 10 ADRs, 7 E2E test scripts, and the complete operational infrastructure. The previous planning snapshot was used as historical context to identify what changed and what remained constant.

### Files Produced

| File | Purpose |
|------|---------|
| `genesis/ASSESSMENT.md` | What works, what's broken, what's half-built, tech debt, security, test gaps, DX |
| `genesis/SPEC.md` | System specification grounded in current code behavior |
| `genesis/DESIGN.md` | User-facing surfaces, interaction flows, crate architecture |
| `genesis/PLANS.md` | Plan index with sequencing rationale and dependency graph |
| `genesis/plans/001-master-plan.md` | Phase structure and plan overview |
| `genesis/plans/002-dead-pallet-removal.md` | Delete 90.6K lines of dead pallet-subtensor code |
| `genesis/plans/003-pallet-naming-normalization.md` | Eliminate confusing pallet_subtensor alias |
| `genesis/plans/004-inherited-migration-cleanup.md` | Remove 30+ subtensor-era migrations |
| `genesis/plans/005-stale-document-cleanup.md` | Fix ghost references in AGENTS.md, OS.md, README.md |
| `genesis/plans/006-phase-1-decision-gate.md` | Checkpoint: reduced codebase verified |
| `genesis/plans/007-emission-dust-policy.md` | Decide truncation dust handling |
| `genesis/plans/008-test-gap-closure.md` | HTTP axon security, key corruption, cross-game scoring |
| `genesis/plans/009-miner-quality-benchmark.md` | Independent quality measurement surface |
| `genesis/plans/010-phase-2-decision-gate.md` | Checkpoint: hardening verified |
| `genesis/plans/011-container-packaging.md` | Docker images and docker-compose devnet |
| `genesis/plans/012-readme-onboarding-overhaul.md` | Developer-friendly README |
| `genesis/plans/013-fabro-ghost-cleanup.md` | Resolve fabro/raspberry references vs reality |
| `genesis/plans/014-token-economics-research-gate.md` | Complete ADR 008 review |
| `genesis/plans/015-sdk-migration-research-gate.md` | Classify opentensor fork patches |

## Major Findings

### 1. The local loop is proven and robust

The core thesis — decentralized game-solving with MCCFR, Yuma Consensus, and deterministic validation — works end-to-end. Three games (poker, Liar's Dice, Kuhn poker) run on the same chain without interfering. Multi-node consensus, emission agreement, restart resilience, and validator determinism are all proven with E2E scripts and CI enforcement. This is a genuinely functional system.

### 2. Inherited complexity is the primary risk

The `pallet-game-solver` (active, 91.6K lines) and `pallet-subtensor` (dead, 90.6K lines) are near-copies. Both compile on every build. The active pallet is aliased as `pallet_subtensor` in the runtime, creating a naming trap for any new contributor who greps for "subtensor" and finds two pallets. Additionally, 44 subtensor-era migrations exist for chain state myosu never had. This inherited complexity does not cause bugs today, but it slows every future change and inflates compilation time.

### 3. Two research gates are externally blocked

- **Token economics (F-003):** ADR 008 exists as `Proposed` but needs multi-contributor review. The NoOpSwap stub is correct for stage-0 but is explicitly not production-ready. No code can resolve this.
- **Miner convergence (F-007):** The validator self-scores the miner checkpoint, so `score=1.0` is always the happy path. No independent quality benchmark exists. Poker is additionally blocked because bootstrap artifacts are intentionally sparse.

### 4. The execution model references are aspirational

AGENTS.md and OS.md describe an elaborate fabro/raspberry execution model with workflow graphs, run configs, program manifests, and control plane state. The `fabro/` directory does not exist. This creates a false impression of operational maturity that undermines the otherwise honest documentation.

### 5. Security posture is adequate for devnet, not production

18 cargo audit advisories in the CI allowlist. No rate limiting on the miner HTTP axon. Insecure randomness pallet behind feature gate. Wire codec decode budget hardened to 1 MiB. Overall: reasonable for a stage-0 devnet, but several items need resolution before any public deployment.

## Recommended Direction

**Close stage-0 by reducing inherited complexity, then package for operators.**

The system works. The remaining work is not about proving the thesis — it's about making the codebase honest and operator-friendly. The four-phase plan (reduce → harden → package → research) follows the same logic as the previous planning snapshot, refined with current code reality.

### Top 3 Immediate Priorities

1. **Plan 002: Dead Pallet Removal** — Highest leverage. Delete 90.6K lines of dead code. Zero dependencies. Immediate clarity improvement for every future contributor.

2. **Plan 005: Stale Document Cleanup** — Independent of code changes. Fix the fabro ghost references. Move THEORY.MD to archive. Make docs match reality.

3. **Plan 003: Pallet Naming Normalization** — After Plan 002 completes. Eliminate the `pallet_subtensor` alias so the codebase has one name for one thing.

### For multi-worker teams

Start Plans 002 and 005 in parallel immediately. Plan 014 (token economics) can begin at any time if a reviewer with economics context is available.

## What Changed Since Previous Snapshot

The previous planning snapshot (April 7, 2026 earlier run) identified the same core issues: duplicated pallet, inherited complexity, two externally-blocked research gates, stale documentation. The previous plan also used reduce-first sequencing.

**What's new in this corpus:**

1. **Fabro ghost infrastructure explicitly called out.** The previous snapshot mentioned fabro/raspberry but didn't highlight that the directories don't exist. This corpus makes the discrepancy a concrete cleanup plan (013).
2. **Pallet naming normalization as a separate plan.** The previous snapshot bundled naming into "dead code removal." This corpus separates them because the Cargo alias rename requires different verification than file deletion.
3. **Emission dust policy as a concrete plan.** The previous snapshot noted the dust measurement but left the policy as a worklist item. This corpus makes it Plan 007 with specific options and acceptance criteria.
4. **Miner quality benchmark surface as a plan.** The previous snapshot's F-007 blocker description is accurate but didn't propose a path forward. Plan 009 proposes using exploitability-based benchmarking for Liar's Dice (which has no encoder dependency) as the starting point.
5. **Container packaging included.** The previous snapshot had this as Plan 010. This corpus moves it to Plan 011 (after hardening) because Docker images should package the final binary form.

## Explicit "Not Doing" List

These are intentionally excluded from the planning horizon:

| Item | Reason |
|------|--------|
| Game portfolio expansion | Three games prove the architecture. New games are stage-1. |
| Web/mobile gameplay frontend | TUI and pipe serve stage-0. Web is a product decision, not an engineering prerequisite. |
| Full AMM token economics implementation | Research gate (Plan 014) must close first. NoOpSwap is correct for stage-0. |
| Production deployment | Devnet-only until stage-0 exits and operator packaging is complete. |
| Upstream polkadot-sdk migration | Research gate (Plan 015) must close first. The fork works. |
| Governance mechanisms | No governance needed until there are multiple stakeholders. |
| Runtime upgrade/migration paths | No deployed state to migrate. Fresh genesis is the stage-0 model. |
| Benchmarking and weight calibration | Not needed until production runtime. |
| Public testnet operations | Devnet-only until operator packaging exists. |
| Robopoker fork changes (RF-01 through RF-04) | These are prerequisites for richer poker training but do not block stage-0 exit. The sparse bootstrap artifacts are by design. |
| Python research layer integration | 3.5K lines of disconnected CFR research. Not part of the product. |
| TUI solver advisor verification | The TUI shell works, but solver advisor integration is not independently proven. This is a nice-to-have, not a gate. |

## Comparison to Previous Planning Snapshot

| Aspect | Previous Snapshot | This Corpus |
|--------|------------------|-------------|
| Dead pallet identified | Yes (Plan 002) | Yes (Plan 002) |
| Sequencing | Reduce → Network → Package → Research | Reduce → Harden → Package → Research |
| Plan count | 14 | 15 (added naming normalization) |
| Decision gates | 1 (Plan 009) | 2 (Plans 006, 010) |
| Fabro gap identified | Mentioned | Explicit cleanup plan (013) |
| Dust policy | Worklist item | Concrete plan (007) with options |
| Quality benchmark | Blocker description | Concrete plan (009) with approach |
| Container packaging | Plan 010 | Plan 011 (after hardening gate) |
| Research gates | Plans 013-014 | Plans 014-015 (renumbered) |

The fundamental diagnosis is the same. The plan structure is refined with more explicit checkpoints and a clearer separation between naming and deletion work.

# Myosu Decision Log

## 2026-03-17: Design reimagination — design.md is the vision, codexpoker is the engine

Decision: After initial reconciliation that pulled design toward codexpoker's
density, reversed direction. design.md's elevated aesthetic is the target.
codexpoker's proven engine patterns power it underneath.

Visual identity (from design.md, preserved):
- Declarations as hero text (signature element)
- Separator rhythm (─── as breath between sections)
- Two-space indent as information hierarchy
- Field-label state panel with column alignment
- Prose action log ("solver raises to 6bb" not "↑ preflop · Seat 0 raises-to 6")
- One accent per screen (monochrome default, color enters sparingly)
- EQUILIBRIUM section for solver advisor (flush-left ALLCAPS header)

Engine (from codexpoker, ported):
- FlexRenderable layout allocation, Renderable trait, streaming buffer
- Blueprint loading, TrainingTable, BotBackend, action parser shorthands

Explicitly NOT adopted from codexpoker:
- Icon-prefix log format (→ ↑ ✕ ◈ ◉ ~) — prose verbs instead
- SplitBorder prefix (┃) — two-space indent instead
- Separator-joined clauses (·) — column alignment instead
- 7 simultaneous ColorRole colors — restrained palette instead
- Gameboard-above-composer ordering — state-above-log per design.md

Panel ordering: design.md's state-above-log. Decision context (cards, board,
advisor) highest in viewport. Log scrolls below. This is the opposite of
codexpoker's gameboard-above-composer pattern, but correct for a learning tool
where the state panel is primary and the log is secondary.

## 2026-03-17: Codexpoker blueprint as test fixture for TU-10/TU-11

Decision: Use the existing trained MCCFR blueprint at ~/.codexpoker/blueprint/
(113M infosets, 335M edges, schema v1) as the reference test fixture for
blueprint loading (TU-10) and solver advisor (TU-11) validation.
Validated: +2.35 chips/hand edge vs random (1000 hands, ~117 mbb/h), mirror
converges near 0 (-0.54 chips/hand, 49.9% win rate). Practice probe confirms
mixed strategies with valid distributions.
Test criteria: edge > 1.0 chips/hand vs random (catches corruption), mirror
|edge| < 1.0 chips/hand (catches seat-dependent bugs), distributions sum to
1.0 (catches normalization errors).
Myosu schema v1 designed to be compatible with this artifact format. Set
MYOSU_BLUEPRINT_DIR=~/.codexpoker/blueprint to use.

## 2026-03-17: Expand TUI spec from 7 to 12 ACs — live NLHE poker game

Decision: Add TU-08..12 covering NLHE poker renderer, training mode, blueprint
loading, solver advisor, and session stats. Port from codexpoker prototype (33K
lines of production TUI code) rather than building from scratch.
Key addition: solver advisor panel shows trained MCCFR action distribution for
hero's current decision point. ON by default in training mode (learning is the
value prop), OFF by default in chain mode (miner strategy privacy).
Training mode enables standalone play without chain infrastructure — proves the
gameplay experience before miners/validators are running. Training commands
(/deal, /board, /stack, /showdown) enable scenario drilling with solver guidance.
Total ACs: 77 → 82. Total spec changes: TUI spec, IMPLEMENT.md, AGENTS.md,
master index.

## 2026-03-17: Final 10/10 audit — 4 blockers found, 2 new

Decision: (1) Genesis MUST set FirstEmissionBlockNumber + SubtokenEnabled for
game subnet, otherwise emission silently never flows. (2) Replace subtensor's
custom transaction fee handler with standard FungibleAdapter — the custom
handler hard-depends on pallet_subtensor_swap::Config even with Alpha fees
disabled. Both added to CF-04 and CF-08 respectively.
Also: set SubnetMechanism=0 (Stable) for single-token 1:1 identity swap.
Override on_runtime_upgrade to no-op. Replace forked pallet_utility with
standard version. Strip leasing extrinsics (110/111).

## 2026-03-16: Fork Bittensor rather than deploy as native subnet

Decision: Hard fork subtensor to create a dedicated game-solving chain.
Alternatives considered:
- Deploy as Bittensor subnet (Option A): ~$1-2M TAO lock, no chain control
- Hybrid with codexpoker L1 (Option C): clean separation but more integration work
Rationale: Need control over chain parameters, game-specific tokenomics, and
subnet lifecycle. Forking subtensor gives us Substrate + proven incentive math
at fraction of the cost of building from scratch.

## 2026-03-16: Robopoker v1.0.0 as core solver engine

Decision: Depend on robopoker via git tag, not vendor/fork.
Rationale: v1.0.0 is the first stable release with production MCCFR, clustering,
and blueprint infrastructure. Maintaining upstream fidelity (INV-006) keeps us
on the improvement track and avoids maintenance burden.

## 2026-03-16: Malinka as autonomous development framework

Decision: Structure the repo for malinka's task-first development loop.
Rationale: The project has clear specs, bounded stages, and proof gates. Malinka's
structured RESULT/BLOCKED closure and plan-based tracking fits the multi-crate
Substrate build.

## 2026-03-16: Poker44 subnet analysis (Bittensor SN126)

Decision: Adopt infrastructure patterns, reject domain approach.
Analysis: Poker44 (github.com/Poker44/Poker44-subnet) is a bot DETECTION
subnet, not a strategy subnet. Miners classify hands as bot/human. No CFR,
no solvers, no game theory. Useful patterns: three-fallback set_weights()
for SDK version variance, SHA256(seed:window) deterministic seeding,
16-miner rotation sampling per cycle. Patterns rejected: 97% emission burn
(hostile economics), over-aggressive sanitization (strips all useful data),
random showdown winners (no hand evaluation). Reference for VO-03, VO-05,
VO-06 infrastructure only.

## 2026-03-16: Project name "myosu" (묘수)

Decision: Korean word meaning "brilliant move" or "masterstroke."
Rationale: Reflects game-solving focus, Korean gaming culture (baduk, StarCraft),
and is clean/memorable as a romanized word.

## 2026-03-17: Pre-implementation audit — subtensor is more complex than specs assumed

Decision: Expand CF stage from 5 to 11 ACs. Add SwapInterface stub, strip
drand/crowdloan supertraits, replace fp_self_contained, strip CRV3 timelock,
port safe-math/share-pool primitives, stub ProxyInterface/CommitmentsInterface.
Findings: pallet_subtensor::Config requires pallet_drand::Config + pallet_crowdloan::Config
as supertraits (blocks all compilation). SwapInterface is called in registration, staking,
AND emission (not a leaf dependency). fp_self_contained from Frontier used for extrinsic
types. CRV3 timelock depends on pallet_drand::Pulses. CF-07 (supertrait strip) must be
the very first commit or nothing compiles.
Estimate revised: 8-11 days for chain fork (was 3-5 days).

## 2026-03-17: Rewrite emission distribution, don't port

Decision: Write GS-06 emission from scratch rather than porting subtensor's coinbase.
Rationale: subtensor's run_coinbase (957 lines) assumes root network, AMM pricing,
multi-subnet weight allocation, and Alpha/TAO dual-token model. 80% of this code
is unnecessary for a single-subnet game-solving chain. Porting would import massive
unused complexity. A clean 50-100 line emission distribution matching our spec is
safer and more maintainable.

## 2026-03-17: Single-token model, not dual Alpha/TAO

Decision: Myosu uses a single game token (MYOSU), not subtensor's dual Alpha/TAO
model with AMM pools.
Rationale: Alpha tokens, subnet-specific AMM pools, and share-pool staking add ~30
storage items and 800+ lines of complexity for zero Stage 0 value. Direct token
staking is simpler and sufficient for game-solving incentives.

## 2026-03-17: Expand robopoker fork scope from 2 to 4 ACs

Decision: Add RF-03 (expose clustering APIs) and RF-04 (file-based checkpoints).
Findings: The clustering pipeline (rbp-clustering) is entirely database-bound —
Layer::cluster() takes &Client. No standalone file path exists. Also, no
file-based checkpoint save/load exists in robopoker; only PostgreSQL persistence.
Both are prerequisites for miners running without database infrastructure.

## 2026-03-17: Commit-reveal v2 only (hash-based), strip CRV3 timelock

Decision: Port only the hash-based commit-reveal mechanism (v2). Strip CRV3
timelock encryption entirely.
Rationale: CRV3 depends on pallet_drand::Pulses for timelock encryption. Since
pallet_drand is stripped in the fork, CRV3 cannot function. Hash-based
commit-reveal (store hash, reveal within N blocks) is sufficient for protecting
validator weight vectors from inter-validator copying.

## 2026-03-17: Vendor subtensor_runtime_common as myosu_runtime_common

Decision: Copy subtensor/common/ into myosu workspace rather than rewriting.
Rationale: NetUid, MechId, NetUidStorageIndex, TaoCurrency, AlphaCurrency are
used in nearly every pallet file. Rewriting is risky and unnecessary. For the
single-token model, alias TaoCurrency = AlphaCurrency = same underlying u64
newtype. This keeps code compatible while eliminating the dual-token complexity.

## 2026-03-17: Share-pool cannot be replaced with simple StorageDoubleMap

Decision: Port share-pool crate as-is (452 lines).
Rationale: Final validation audit found that 20+ stake functions in
stake_utils.rs depend on share-pool's proportional accounting (SharePool struct,
SharePoolDataOperations trait). The share pool backs Alpha/TotalHotkeyAlpha/
TotalHotkeyShares storage and is used in the epoch's stake weight calculation,
coinbase dividend distribution, and pruning system. Replacing it would require
rewriting the entire staking layer — unacceptable risk for Stage 0.

## 2026-03-17: Emission split 61/21/18 (miners/validators/owner)

Decision: 61% miners, 21% validators, 18% subnet owner.
Rationale: Matches Bittensor's 18% owner cut — proven sustainable for
protocol development funding. 22% validators gives headroom for
exploitability computation at scale (192 miners × full tree traversal per
tempo). Per-participant earnings: miners 0.31% each (192), validators
0.34% each (64) — validators slightly higher per-participant, justified by
their identical hardware requirements (same 5.5 GB encoder) and burst
compute that must complete within tempo. Owner cut funds development,
abstraction computation, and future game expansion. Future subnets may
have third-party owners with their own cut.

## 2026-03-17: CFR research validation of incentive mechanism

Decision: Full abstract-game exploitability (not LBR) as scoring method.
Research findings: Pluribus trained in 8 days on single 64-core server at
$144, proving commodity hardware sufficiency. Verification asymmetry is
~50-100x (LBR takes 30 min vs days of training). DCFR converges 2-3x
faster than CFR+. Our ~542 buckets are in the same range as Pluribus (200
buckets/street), making full best-response tractable. Abstraction must be
protocol-defined (not miner-chosen) to prevent gaming.

## 2026-03-17: NLHE Incentive Mechanism design spec (031626-12)

Decision: Exploitability is the sole scoring metric. 61/21/18
miner/validator/owner emission split. Tempo = 180 blocks (~36 min).
BondsPenalty beta = 0.1 from genesis. Yuma v3 from genesis.
Rationale: Synthesized from Bittensor docs (Yuma Consensus, hyperparameters),
robopoker MCCFR architecture audit, subtensor source analysis. Key insight:
exploitability is the only rigorous measure of strategy quality — proxy metrics
(epoch count, throughput) are gameable. The 60/40 split (vs Bittensor's 50/50)
reflects that game-solving miners bear heavier compute cost (6-8 GB RAM,
continuous multi-day training) vs validators (periodic query + score).

## 2026-03-17: Add VO-07 (two-validator INV-003 agreement test)

Decision: Add a named AC that starts two independent validator instances, has
them score the same miner, and asserts score equality within epsilon.
Rationale: INV-003 (game verification determinism) is the single most important
correctness property. VO-03 and VO-04 test determinism in isolation, but no
existing AC proves two independently initialized validators agree. This is the
no-ship gate for consensus validity.

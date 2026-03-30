# Specification: Incentive Layer Design — Enhanced Emission Mechanics for Solver Markets

Source: Hyperspace AGI pattern analysis + master spec incentive gap analysis
Status: Draft
Date: 2026-03-30
Depends-on: GS-01..10 (pallet foundation), VO-01..07 (validator oracle)
Prior-art: https://github.com/hyperspaceai/agi (provenance DAG, uptime curves, capability weighting)

## Purpose

Define the incentive mechanisms that sit above Yuma Consensus to create a
solver market with healthy dynamics: sustained participation, exploration of
novel strategies, and fair compensation across subnets with different compute
requirements.

The base layer (GS-05 Yuma Consensus, GS-06 emission) distributes tokens
proportional to exploitability scores. This works but creates three problems:

1. **Plateau-sitting**: a miner who reaches a good strategy has no incentive
   to keep improving — it earns the same emission whether it's stagnant or
   actively exploring. The system rewards position, not velocity.

2. **Churn vulnerability**: a new miner that registers, collects one epoch of
   emission, and deregisters costs the network (registration slot, validator
   evaluation time) without contributing sustained compute. No mechanism
   penalizes ephemeral participation.

3. **Cross-subnet compute imbalance**: PLO's game tree is ~100x larger than
   NLHE HU. A flat emission split across subnets means PLO miners earn the
   same reward for 100x more work. Rational miners avoid hard games.

This spec adds three mechanisms to address these problems, each inspired by
proven patterns from Hyperspace AGI's distributed research network, adapted
for myosu's on-chain game-solving context.

## Whole-System Goal

Current state:
- the live stage-0 pallet already runs a real local emission loop with miner
  incentive, validator dividend, and emission vectors visible in the owned
  local proof
- GS-06 distributes emission proportional to Yuma Consensus scores
- Emission split stays stage-0 simple relative to the richer future modifiers
- No provenance tracking, no uptime consideration, no compute-cost weighting
- Miners are scored on absolute exploitability only

This spec adds:
- **Strategy provenance tracking** — on-chain lineage of strategy improvements
- **Logarithmic uptime bonus** — emission multiplier rewarding sustained participation
- **Subnet compute weights** — per-subnet emission scaling based on game complexity

If all mechanisms land:
- Miners who demonstrate measurable improvement earn more than plateau-sitters
- Long-running miners earn up to ~80% more per epoch than fly-by-night registrants
- PLO miners earn proportionally more than NLHE HU miners, reflecting compute cost
- The chain has an auditable record of how strategies evolved over time

Still not solved here:
- Cross-subnet strategy transfer (see planned spec 031626-16-cross-game-scoring)
- Slashing for dishonest miners or validators
- Dynamic compute weight adjustment (manual Sudo for bootstrap)
- Off-chain gossip layer for validators (future optimization)

## Why This Spec Exists As One Unit

- All three mechanisms modify emission distribution, which happens in a single
  code path (GS-06 `distribute_emissions`). Designing them together prevents
  conflicting incentives.
- Provenance tracking requires both pallet storage (on-chain) and validator
  evaluation changes (off-chain). The design must account for both sides.
- Uptime bonus and compute weights are multiplicative modifiers on the same
  base emission. Their interaction must be specified together.

## Scope

In scope:
- Design of three emission-modifying mechanisms with concrete formulas
- Storage extensions to `pallet_game_solver` for provenance and uptime
- Validator-side changes for provenance reporting
- Miner-side changes for strategy submission metadata
- Modified emission formula integrating all three mechanisms
- Concrete test scenarios for each mechanism

Out of scope:
- CRDT-based off-chain gossip for validators — operational optimization, not
  incentive design. Worth revisiting once validator count exceeds 10.
- Cross-subnet knowledge transfer credits — requires multi-game subnets to
  exist first. Planned for spec 031626-16.
- Slashing — requires a fraud proof mechanism not yet designed
- Dynamic compute weight oracle — manual Sudo tuning for bootstrap

## What Already Exists

| Sub-problem | Existing code / flow | Reuse / extend / replace | Why |
|-------------|----------------------|--------------------------|-----|
| Emission distribution | GS-06 `distribute_emissions` | extend | Add multipliers to existing proportional distribution |
| Weight submission | GS-04 `set_weights` | extend | Validators report provenance metadata alongside weights |
| Neuron registration | GS-03 `register_neuron` | extend | Track registration block for uptime computation |
| Subnet creation | GS-02 `create_subnet` | extend | Add `compute_weight` parameter |
| Exploitability scoring | VO-04 `score_miner` | extend | Track delta from previous epoch's score |

## Non-goals

- Replace Yuma Consensus — these mechanisms are modifiers, not replacements
- Guarantee convergence to Nash equilibrium — the market incentivizes it, doesn't mandate it
- Penalize bad miners — that's slashing, a separate concern
- Optimize for maximum throughput — correctness and incentive alignment first

---

## Mechanism 1: Strategy Provenance DAG

### Problem

Miners are scored on absolute exploitability. A miner at 50 mbb/h that hasn't
improved in 100 epochs earns the same as a miner at 50 mbb/h that just
improved from 200 mbb/h. The system has no concept of *trajectory* — only
position.

### Inspiration

Hyperspace AGI tracks every experiment's parent via an `inspiredBy` field and
hypothesis string, forming a DAG with measurable `maxDepth` per domain. Their
agents show lineage chains 7 generations deep, enabling the network to
distinguish genuine explorers from stale participants.

### Design

Each miner's strategy submission carries a **provenance record**: a pointer to
its previous best exploitability score and the current score. The chain stores
this as a linked list per miner per subnet, forming a provenance chain.

**On-chain storage** (extends GS-01):

```rust
/// Per-miner strategy provenance on a subnet
/// Records the exploitability trajectory: each entry links to its predecessor
#[pallet::storage]
pub type Provenance<T: Config> = StorageDoubleMap<
    _,
    Identity, u16,          // subnet_id
    Identity, u16,          // uid
    ProvenanceRecord,
    OptionQuery,
>;

#[derive(Encode, Decode, Clone, TypeInfo, MaxEncodedLen)]
pub struct ProvenanceRecord {
    pub current_exploit: u32,    // mbb/h * 1000 (fixed-point, 3 decimal places)
    pub previous_exploit: u32,   // mbb/h * 1000 from prior epoch
    pub improvement_streak: u16, // consecutive epochs with improvement
    pub epoch_registered: u64,   // block number of first registration
    pub total_epochs: u32,       // epochs this miner has been scored
}
```

**Improvement bonus formula**:

```
delta = (previous_exploit - current_exploit) / previous_exploit
improvement_bonus = if delta > 0.001 { min(delta * IMPROVEMENT_SCALAR, MAX_IMPROVEMENT_BONUS) } else { 0 }
```

Where:
- `IMPROVEMENT_SCALAR = 2.0` — a 10% improvement yields a 20% emission bonus
- `MAX_IMPROVEMENT_BONUS = 0.5` — capped at 50% bonus to prevent gaming via
  intentional initial sandbagging
- `delta > 0.001` threshold prevents noise from counting as improvement

**Streak multiplier**: consecutive epochs of improvement compound:

```
streak_factor = 1.0 + 0.05 * min(improvement_streak, 10)
effective_bonus = improvement_bonus * streak_factor
```

A miner improving for 10 consecutive epochs gets 1.5x the improvement bonus.
Capped at 10 to prevent runaway compounding.

**Anti-sandbagging**: a miner that deliberately submits a bad initial strategy
to create artificial improvement headroom is handled by two guards:
1. The base Yuma score still dominates — a bad strategy earns near-zero base
   emission regardless of improvement bonus
2. `MAX_IMPROVEMENT_BONUS = 0.5` means the bonus can never exceed 50% of base
   emission — sandbagging sacrifices more base emission than the bonus recovers

**Validator-side** (extends VO-04):

Validators already compute exploitability per miner. The provenance record is
updated by reading the miner's previous `Provenance` record from chain and
comparing with the new score. No additional miner queries needed — the
validator has all information.

```rust
fn update_provenance(
    chain: &ChainClient,
    subnet_id: u16,
    uid: u16,
    new_exploit: u32,
) -> ProvenanceRecord {
    let prev = chain.provenance(subnet_id, uid);
    match prev {
        Some(p) => ProvenanceRecord {
            current_exploit: new_exploit,
            previous_exploit: p.current_exploit,
            improvement_streak: if new_exploit < p.current_exploit {
                p.improvement_streak.saturating_add(1)
            } else {
                0
            },
            epoch_registered: p.epoch_registered,
            total_epochs: p.total_epochs.saturating_add(1),
        },
        None => ProvenanceRecord {
            current_exploit: new_exploit,
            previous_exploit: new_exploit,
            improvement_streak: 0,
            epoch_registered: current_block,
            total_epochs: 1,
        },
    }
}
```

**Who writes provenance**: the chain itself, during epoch execution (GS-05).
After Yuma Consensus computes the final score for each miner, the epoch
function reads the previous `Provenance` record, computes the delta, and
writes the updated record. This is fully on-chain and deterministic — no
off-chain reporting needed.

### Why not a full DAG?

Hyperspace tracks cross-experiment ancestry (experiment B was inspired by
experiment A's config). For myosu, miners produce a *single evolving strategy*
per subnet, not branching experiments. A linear provenance chain (current →
previous → previous) is sufficient. If miners later support parallel strategy
branches, this can be extended to a DAG by adding a `parent_hash` field.

---

## Mechanism 2: Logarithmic Uptime Bonus

### Problem

Registration is cheap (burn cost). A miner can register, serve a pre-trained
strategy, collect one epoch's emission, and deregister. This wastes a neuron
slot and validator evaluation cycles without contributing ongoing compute.

### Inspiration

Hyperspace AGI uses `U(t) = 1 + 0.2 * ln(1 + t/12)` where `t` is measured in
12-hour increments. A 30-day node earns 83% more per Pulse round than a new
node. The logarithmic shape rewards commitment with diminishing returns —
preventing permanent incumbency advantage.

### Design

Each miner's emission is multiplied by an uptime factor based on how many
epochs they've been continuously active.

**Uptime factor formula**:

```
U(e) = 1.0 + UPTIME_SCALE * ln(1 + e / UPTIME_HALF_LIFE)
```

Where:
- `e` = consecutive epochs active (from `ProvenanceRecord.total_epochs`)
- `UPTIME_SCALE = 0.25` — controls the steepness of the bonus
- `UPTIME_HALF_LIFE = 24` — measured in epochs. At tempo=100 blocks, 6s/block,
  24 epochs ≈ 4 hours. Miners reach half the asymptotic bonus within ~4 hours
  of continuous operation.

**Concrete values** (tempo=100, 6s blocks = 10 min/epoch):

| Duration | Epochs | U(e) | Bonus |
|----------|--------|------|-------|
| 10 min | 1 | 1.010 | +1.0% |
| 1 hour | 6 | 1.057 | +5.7% |
| 4 hours | 24 | 1.173 | +17.3% |
| 24 hours | 144 | 1.447 | +44.7% |
| 7 days | 1008 | 1.930 | +93.0% |
| 30 days | 4320 | 2.296 | +129.6% |

A miner running for 24 hours earns ~45% more per epoch than a brand-new miner
with the same strategy quality. After 7 days the bonus approaches 2x but
never creates an insurmountable wall — a new miner with a significantly better
strategy can still earn more in base emission.

**Reset condition**: if a miner fails to serve a valid strategy response to
any validator query in an epoch (score = 0), the uptime counter resets to 0.
This prevents zombie registrations from accumulating uptime.

**On-chain storage** (extends GS-01):

No new storage needed — `ProvenanceRecord.total_epochs` serves as the uptime
counter. It resets when a miner is pruned and re-registers (new `ProvenanceRecord`).

**On-chain computation** (extends GS-05/GS-06):

The uptime bonus is applied during emission distribution:

```rust
fn emission_with_uptime(base_emission: u64, total_epochs: u32) -> u64 {
    let e = total_epochs as f64;
    let uptime_factor = 1.0 + UPTIME_SCALE * (1.0 + e / UPTIME_HALF_LIFE).ln();
    // Fixed-point: compute using I64F64 to match Yuma's arithmetic
    (base_emission as f64 * uptime_factor) as u64
}
```

**Interaction with Yuma**: the uptime bonus is applied *after* Yuma computes
the raw incentive split. Yuma determines the relative ranking; uptime scales
the absolute emission. This preserves Yuma's consensus properties — the
relative ordering of miners doesn't change, only the magnitude.

> **Open question**: should uptime bonus apply to validators too? Validators
> with longer uptime have more consistent bond EMAs, which Yuma already
> rewards via the bond mechanism. Adding an explicit uptime bonus may
> double-count. **Recommendation**: miner-only for bootstrap. Revisit when
> validator churn becomes a problem.

---

## Mechanism 3: Subnet Compute Weights

### Problem

Game trees vary in size by orders of magnitude:

| Game | Approximate game tree size | Relative MCCFR cost |
|------|---------------------------|---------------------|
| Liar's Dice (2p, 6-sided) | ~10^4 | 1x |
| NLHE Heads-Up (abstracted) | ~10^9 | 100x |
| NLHE 6-max | ~10^14 | 10,000x |
| PLO Heads-Up | ~10^12 | 1,000x |
| Mahjong (simplified) | ~10^15 | 100,000x |

Equal emission per subnet means a PLO miner earns the same as a Liar's Dice
miner while spending 1000x more compute. Rational miners flock to easy
subnets, leaving hard games unsolved.

### Inspiration

Hyperspace AGI uses 9 capability types with explicit point multipliers
(inference +10%, research +12%, proxy +8%, etc.). Nodes self-select into roles
based on hardware. The weights are set manually by governance, not computed
dynamically.

### Design

Each subnet has a `compute_weight` parameter (u16, default 100) that scales
its share of the global emission pool. Instead of equal emission per subnet,
emission is proportional to compute weight.

**Emission per subnet formula**:

```
subnet_emission = block_emission * compute_weight[subnet] / sum(compute_weight[active_subnets])
```

**Example** with 3 active subnets:

| Subnet | Game | compute_weight | Emission share |
|--------|------|---------------|---------------|
| 1 | NLHE HU | 100 | 100/250 = 40% |
| 2 | PLO HU | 100 | 100/250 = 40% |
| 3 | Liar's Dice | 50 | 50/250 = 20% |

The subnet owner (or Sudo) sets `compute_weight` at creation. It can be
adjusted via `set_subnet_hyperparams` (GS-02). No oracle or dynamic
adjustment for bootstrap — governance sets weights based on known game tree
sizes.

**On-chain storage** (extends GS-01):

```rust
/// Compute weight for emission scaling — higher weight = more emission share
/// Default 100. Set by subnet owner or Sudo.
#[pallet::storage]
pub type ComputeWeight<T: Config> = StorageMap<_, Identity, u16, u16, ValueQuery>;
// Default = 100
```

**On-chain computation** (extends GS-06):

```rust
fn subnet_emission_share(
    subnet_id: u16,
    block_emission: u64,
    active_subnets: &[u16],
) -> u64 {
    let weight = ComputeWeight::<T>::get(subnet_id);
    let total_weight: u64 = active_subnets
        .iter()
        .map(|s| ComputeWeight::<T>::get(*s) as u64)
        .sum();
    if total_weight == 0 { return 0; }
    block_emission * weight as u64 / total_weight
}
```

**Guard rails**:
- `compute_weight` minimum: 10 (prevents zero-weight subnets from consuming slots)
- `compute_weight` maximum: 10000 (prevents a single subnet from dominating emission)
- Changes take effect at the next epoch boundary, not mid-epoch

**Future direction**: replace Sudo-set weights with an oracle that estimates
compute cost from observed miner performance metrics (iterations per epoch,
convergence rate). This requires measuring miner compute, which is a
non-trivial anti-gaming problem. Manual weights are correct for bootstrap.

---

## Combined Emission Formula

The three mechanisms compose multiplicatively on top of Yuma's base incentive:

```
final_emission[uid] = base_yuma_incentive[uid]
                    * subnet_share(subnet_id, block_emission)
                    * (1 + improvement_bonus[uid])
                    * uptime_factor(total_epochs[uid])
```

Where:
- `base_yuma_incentive[uid]` = Yuma's normalized incentive score (0.0–1.0),
  already scaled to the miner's share of the 50% miner emission
- `subnet_share` = the subnet's compute-weighted share of block emission
- `improvement_bonus` = provenance-derived improvement reward (0.0–0.5)
- `uptime_factor` = logarithmic uptime multiplier (1.0–~2.3)

**Maximum effective multiplier**: a miner at peak improvement bonus (50%)
with 30 days uptime (2.3x) earns `1.5 * 2.3 = 3.45x` a baseline miner's
emission. This is intentionally generous — it rewards exactly the behavior
the network needs (sustained, improving computation).

**Total emission conservation**: the improvement bonus and uptime factor
increase individual miner emission, which means the total emission per block
exceeds `BlockEmission`. Two options:

**Option A — Inflationary bonuses**: bonuses mint additional tokens. Total
emission per block = `BlockEmission * average_multiplier`. Simple to implement.
Creates mild inflation that rewards active participants at the expense of
passive token holders.

**Option B — Redistributive bonuses**: bonuses are funded by reducing emission
to miners without bonuses. Total emission per block is always exactly
`BlockEmission`. Zero inflation. More complex to implement (requires
normalizing after applying multipliers).

**Recommendation**: Option B (redistributive) for launch. It preserves the
economic invariant that exactly `BlockEmission` tokens are minted per block,
which simplifies tokenomics reasoning. Implementation: apply all multipliers,
then renormalize so the sum equals `BlockEmission * subnet_share`.

```rust
fn distribute_with_bonuses(
    subnet_id: u16,
    raw_incentives: &[(u16, u64)],    // uid → raw Yuma incentive
    provenances: &[(u16, ProvenanceRecord)],
    subnet_emission: u64,
) -> Vec<(u16, u64)> {
    // Apply multipliers
    let weighted: Vec<(u16, f64)> = raw_incentives.iter().map(|(uid, raw)| {
        let prov = find_provenance(provenances, *uid);
        let improvement = compute_improvement_bonus(&prov);
        let uptime = compute_uptime_factor(prov.total_epochs);
        (*uid, *raw as f64 * (1.0 + improvement) * uptime)
    }).collect();

    // Renormalize to subnet_emission total (Option B — redistributive)
    let total_weighted: f64 = weighted.iter().map(|(_, w)| w).sum();
    if total_weighted == 0.0 { return vec![]; }

    weighted.iter().map(|(uid, w)| {
        (*uid, (w / total_weighted * subnet_emission as f64) as u64)
    }).collect()
}
```

---

## Interaction with Existing Specs

### Pallet extensions (GS-01..10)

| Storage / extrinsic | Change | Spec |
|---------------------|--------|------|
| `Provenance` storage map | NEW — add to GS-01 | This spec |
| `ComputeWeight` storage map | NEW — add to GS-01 | This spec |
| `create_subnet` params | EXTEND — add `compute_weight` param | GS-02 |
| `set_subnet_hyperparams` | EXTEND — include `compute_weight` | GS-02 |
| `run_epoch` | EXTEND — update `Provenance` after Yuma | GS-05 |
| `distribute_emissions` | EXTEND — apply bonuses + renormalize | GS-06 |

### Validator extensions (VO-01..07)

| Component | Change | Spec |
|-----------|--------|------|
| No validator changes needed | Provenance is computed on-chain during epoch | — |

The provenance record is entirely on-chain. Validators already submit
exploitability scores via weights. The chain computes the delta between
current and previous scores during epoch execution. This eliminates any
trust requirement on validators for provenance reporting.

### Invariant impact

| Invariant | Impact | Mitigation |
|-----------|--------|------------|
| INV-003 (determinism) | Provenance update and uptime computation must be deterministic | All math uses `substrate_fixed` types; no floating-point in on-chain code |
| INV-004 (solver-gameplay separation) | No impact — incentive mechanics are chain-only | N/A |

---

## Decision Log

- 2026-03-17: Strategy provenance as linear chain, not full DAG — miners
  produce one evolving strategy per subnet, not branching experiments. If
  multi-branch training is added later, extend to DAG with `parent_hash`.
- 2026-03-17: Logarithmic uptime curve over linear — prevents permanent
  incumbency lock-in while still rewarding commitment. Shape borrowed from
  Hyperspace's `U(t) = 1 + 0.2 * ln(1 + t/12)`, adapted for epoch-based
  timing.
- 2026-03-17: Manual compute weights over oracle — dynamic weight adjustment
  requires measuring miner compute (iteration count, convergence rate), which
  is gameable. Manual Sudo weights are correct for bootstrap phase with <5
  subnets.
- 2026-03-17: Redistributive bonuses (Option B) over inflationary — preserves
  `BlockEmission` invariant, simplifies tokenomics. The tradeoff (more complex
  renormalization) is a one-time implementation cost.
- 2026-03-17: Anti-sandbagging via `MAX_IMPROVEMENT_BONUS = 0.5` cap rather
  than reputation history — simpler, no additional state. A miner that
  deliberately submits a bad strategy sacrifices far more in base Yuma
  emission than it gains from the 50% improvement bonus.
- 2026-03-17: No CRDT gossip layer for validators — Hyperspace uses Loro
  CRDTs for leaderboard state to avoid on-chain bloat. Myosu validators
  submit weights on-chain every tempo, which is already low-frequency enough
  (~10 min). CRDT gossip is an optimization for high-frequency validation
  with many validators. Revisit when validator count exceeds 10.
- 2026-03-17: No cross-subnet transfer credits yet — requires multiple game
  subnets to exist first. Hyperspace's cross-domain DAG depth metric
  (`maxDepth` per domain) is the inspiration, but the implementation depends
  on spec 031626-16 (cross-game scoring) defining how to measure transfer
  quality.

## Test Scenarios

### Provenance

| Scenario | Expected |
|----------|----------|
| Miner improves from 200→180 mbb/h (10% improvement) | `improvement_bonus = min(0.1 * 2.0, 0.5) = 0.2` (20% bonus) |
| Miner improves from 200→100 mbb/h (50% improvement) | `improvement_bonus = min(0.5 * 2.0, 0.5) = 0.5` (capped at 50%) |
| Miner stays at 200 mbb/h (no improvement) | `improvement_bonus = 0` |
| Miner regresses from 100→150 mbb/h (worse) | `improvement_bonus = 0`, streak resets to 0 |
| 5-epoch improvement streak, 10% improvement | `streak_factor = 1.25`, `effective_bonus = 0.2 * 1.25 = 0.25` |
| New miner first epoch | `improvement_bonus = 0` (no previous score to compare) |

### Uptime

| Scenario | Expected |
|----------|----------|
| Brand-new miner (0 epochs) | `U(0) = 1.0` (no bonus) |
| 1 hour (6 epochs) | `U(6) ≈ 1.057` (+5.7%) |
| 24 hours (144 epochs) | `U(144) ≈ 1.447` (+44.7%) |
| Miner goes offline, re-registers | `total_epochs` resets to 0, uptime resets |
| Miner scores 0 in an epoch (unresponsive) | `total_epochs` resets to 0 |

### Compute weights

| Scenario | Expected |
|----------|----------|
| 2 subnets: NLHE HU (100), PLO (100) | Each gets 50% of block emission |
| 3 subnets: NLHE HU (100), PLO (300), Liar's Dice (50) | PLO gets 300/450 = 66.7% |
| 1 subnet: NLHE HU (100) | Gets 100% of block emission |
| Sudo changes PLO weight from 100→300 mid-epoch | Takes effect next epoch |

### Combined

| Scenario | Expected |
|----------|----------|
| Miner A: best strategy, no improvement, 1 hour uptime | `base * 1.0 * 1.057` |
| Miner B: second-best strategy, 10% improvement, 24 hours uptime | `base * 1.2 * 1.447 = base * 1.736` |
| Miner B can overtake Miner A despite lower base score | Yes — if B's base is >61% of A's base (`1.0*1.057 / 1.736 ≈ 0.609`) |

## Milestone Verification

| # | Scenario | Validates | Mechanism |
|---|----------|-----------|-----------|
| 1 | Miner improves strategy, receives higher emission than stagnant miner with same score | Provenance bonus | 1 |
| 2 | Improvement streak of 5 epochs yields compounding bonus | Streak multiplier | 1 |
| 3 | Sandbagging miner (bad start → fast improve) earns less total than honest miner | Anti-sandbagging | 1 |
| 4 | 24-hour miner earns ~45% more than new miner with identical strategy | Uptime bonus | 2 |
| 5 | Miner that goes unresponsive loses uptime bonus on re-registration | Uptime reset | 2 |
| 6 | PLO subnet (weight=300) earns 3x NLHE (weight=100) per miner | Compute weights | 3 |
| 7 | Total minted emission per block equals `BlockEmission` exactly | Redistributive conservation | Combined |
| 8 | All computations produce identical results in GS-05 determinism test | INV-003 | All |

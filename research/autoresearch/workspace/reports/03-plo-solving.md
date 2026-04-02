# Solving Strategies for Pot-Limit Omaha (PLO, 4-Card)

**Date:** 2026-03-30
**Scope:** Comprehensive survey of PLO-specific solving approaches, 2013--2026
**Status:** Research report (no implementation)

---

## Executive Summary

Pot-Limit Omaha (PLO) is the second most popular poker variant worldwide, yet it remains unsolved and no superhuman AI has been demonstrated for it. The fundamental obstacle is combinatorial: PLO deals four hole cards instead of two, producing 270,725 raw starting hand combinations (vs. 1,326 in NLHE) and approximately 16,432 strategically distinct hands after suit isomorphism (vs. 169 in NLHE). This ~100x expansion in card abstraction space, combined with the constraint that players must use exactly two of four hole cards (generating 6 sub-hands per player per board), produces a game tree several orders of magnitude larger than NLHE heads-up. No tight bound on PLO's game tree has been published, but conservative estimates place it well beyond 10^80 nodes for heads-up play with standard bet discretization.

The dominant approach to PLO solving has been adaptation of NLHE techniques -- CFR-based blueprint computation with aggressive card and action abstraction, followed by postflop refinement. MonkerSolver established the commercial standard in 2016--2018, computing preflop and postflop solutions for PLO using bucketed abstraction on 64GB+ RAM servers. Since 2023, a new generation of solvers has emerged: Simple Omaha (abstraction-free postflop solving via vector algorithms), PLO Genius (cloud-based with 7.9M+ precomputed solutions), FlopHero (real-time river solving), and GTO Wizard PLO (multiway solving with QRE adoption in 2025). On the academic side, PokerRL-Omaha (2020--2022) adapted Single Deep CFR to PLO with distributed computing, but no published result demonstrates competitive play against strong human opponents. The gap between NLHE solver maturity and PLO solver maturity remains substantial, making PLO the natural stress test for solver architectures that claim scalability.

The core open problems are: (1) preflop abstraction quality -- equity distributions in PLO are so clustered that bucketing errors are amplified relative to NLHE, (2) full-game solving -- no end-to-end system exists that handles preflop through river without severe abstraction, (3) multiway PLO -- the game is predominantly played 6-max, where multiplayer Nash equilibrium is both theoretically and computationally problematic, and (4) PLO Hi/Lo -- the split-pot variant doubles the strategic dimension and has received almost no solver attention.

---

## Game Complexity Analysis

### Combinatorial Explosion from Four Hole Cards

| Metric | PLO (4-card) | NLHE (2-card) | Ratio |
|--------|-------------|---------------|-------|
| Raw starting hand combos | C(52,4) = 270,725 | C(52,2) = 1,326 | 204x |
| Strategically distinct preflop hands (suit isomorphism) | ~16,432 | 169 | ~97x |
| Sub-hands per player per board (two-card selections) | C(4,2) = 6 | 1 | 6x |
| Board-card selections for hand construction | C(5,3) = 10 | C(5,3)..C(5,5) = variable | n/a |
| Possible 5-card hands per player per board | 6 x 10 = 60 | 21 (7 choose 5) | ~3x |
| PLO5 starting hand combos | C(52,5) = 2,598,960 | n/a | ~10x vs PLO4 |

### Game Tree Size

No tight bound has been published for PLO specifically. However, we can reason about relative scale:

- NLHE HU has approximately 10^161--10^165 game tree nodes (Johanson 2013). PLO HU replaces the ~1,326 preflop hand combos with ~270,725, increasing the root branching factor by ~204x. This propagation through four streets with pot-limit bet sizing (which constrains the action space more than no-limit) suggests a game tree conservatively 10^2--10^3 larger in the card dimension alone.
- The pot-limit constraint partially offsets the card complexity by reducing extreme bet sizing options. Typical PLO action discretization uses 4--6 actions per node (fold/check, call, 1/3 pot, 1/2 pot, 2/3 pot, pot), compared to 5--8+ in NLHE.
- Postflop information sets scale with both the card space expansion and the 6-sub-hand evaluation requirement. Each equity calculation is approximately 6x more expensive per hand comparison.
- MonkerSolver PLO preflop solves routinely require 64--128 GB RAM and weeks of computation on dedicated hardware.

### Action Space Under Pot-Limit

The pot-limit constraint bounds the maximum bet at the pot size, eliminating the continuous action space problem of no-limit and producing a finite game tree. However, pot-limit betting causes exponential pot growth (~20x by river), collapsing effective SPRs quickly and making deep-stack PLO especially complex. Typical solver discretization: flop {33%, 50%, 67%, 100% pot}, turn {50%, 75%, 100%}, river {50%, 75%, 100%, 150%}, each with raise/re-raise options capped at pot.

---

## Historical Progression

### 2013--2015: NLHE Techniques Dominate, PLO Neglected

The CFR revolution focused exclusively on NLHE. Cepheus solved limit hold'em HU (2015), and the research community's attention turned to NLHE. PLO was considered a natural extension but too computationally expensive for existing approaches. Early PLO analysis tools were limited to equity calculators (e.g., ProPokerTools, Omaha Indicator) without equilibrium computation.

### 2016--2018: MonkerSolver Establishes the PLO Standard

MonkerSolver became the first commercial solver to tackle PLO preflop and postflop computation. Its approach:

- **Card abstraction:** Hands are bucketed by equity distribution using k-means clustering. The bucket count is user-configurable but typically 200--1000 preflop buckets for PLO4, compared to 50--200 for NLHE.
- **Action abstraction:** Users define bet sizing trees via a custom syntax (PPT notation). The solver explores user-specified bet trees rather than the full action space.
- **Algorithm:** CFR-based iteration over the abstracted game tree. Convergence is measured by exploitability in the abstract game (not the full game).
- **Compute requirements:** 64 GB+ RAM minimum for preflop PLO4, 128 GB recommended. Full preflop solves take days to weeks.
- **Output:** Strategy tables mapping abstracted hands to action frequencies at each decision node.

MonkerSolver remains the industry standard for preflop PLO solutions as of 2026, used by virtually all serious PLO study platforms (PLO Mastermind, PreflopWhiz, RangeConverter).

### 2019--2020: PioSolver Enters PLO (Limited)

PioSOLVER, the dominant NLHE postflop solver, added limited PLO support. It can solve specific postflop spots given fixed preflop ranges but cannot handle preflop computation for PLO due to the hand space explosion. PioSolver PLO remains a niche tool for postflop analysis of narrow spots, not a general PLO solver.

### 2020--2022: Deep RL Reaches Omaha

PokerRL-Omaha (diditforlulz273, open-source) extended the PokerRL framework to Omaha:

- Integrated Single Deep CFR (SD-CFR) and Deep CFR with distributed computing
- The first open-source deep RL implementation for Omaha poker
- Required significant upgrades to the distributed computing scheme to handle the larger game
- Neural network agent significantly outperformed earlier NN baselines in PLO head-to-head play
- Training improvements yielded approximately 11% faster convergence in terms of loss
- No claim of superhuman or near-superhuman play

### 2022--2024: Cloud Solver Explosion

Multiple cloud-based PLO solvers launched or matured:

- **PLO Genius** (launched ~2022, major expansion 2024--2026): 7.9 million pre-calculated solutions, with ~20,000 new spots added weekly. Cloud-based with no local compute required. Covers PLO4 and PLO5, preflop and postflop, with aggregate report features for board texture analysis.
- **GTO Wizard PLO** (expanded 2023--2025): Added PLO4 and PLO5 support with multiway solving. In April 2025, upgraded its engine from Nash Equilibrium to Quantal Response Equilibrium (QRE), improving accuracy on low-frequency nodes by 25%.
- **Vision GTO Trainer** (Run It Once): Multiway PLO solutions including SRP spots for 3-way pots.
- **PLO+** (mobile solver): GTO solver for PLO available on mobile platforms.

### 2025--2026: Abstraction-Free and Real-Time Approaches

- **Simple Omaha** (2024--2025): The first poker solver to offer abstraction-free postflop PLO computation using vector algorithms. No bucketing, no Monte Carlo approximation. Local calculation with three algorithm variants (A1, A2, A3) trading RAM for precision. Minimum 32 GB, recommended 128 GB RAM.
- **FlopHero** (2025): Automated PLO study platform with real-time river solving. Rivers solve in seconds without manual tree building. Covers every flop and turn with precomputed solutions, solves rivers on-demand.
- **Deepsolver** (2025--2026): The first commercial neural-network-powered poker solver, currently NLHE only but with PLO support announced. Combines CFR accuracy with neural network speed, solving one street at a time with 0.59% average Nash Distance.

---

## Current Best Approaches

### 1. MonkerSolver: Tabular CFR with Heavy Abstraction

**Algorithm:** Standard CFR (likely MCCFR variant) operating on a user-defined abstracted game tree.

**Card abstraction:** k-means clustering on equity distributions with Earth Mover's Distance (EMD) as the metric. Users configure bucket counts; higher bucket counts increase accuracy but require more RAM and time.

**Action abstraction:** User-defined bet sizing trees via PPT syntax. EV-optimal sizings selected through testing across large board samples, then validated by high-stakes players.

**Strengths:**
- Handles preflop and postflop PLO4 and PLO5
- Multiway support (3+ players)
- Full user control over abstraction granularity
- Battle-tested over 8+ years in the PLO community
- Solutions can be exported and used by third-party tools

**Weaknesses:**
- Heavy abstraction introduces bucketing errors, especially damaging in PLO where equity distributions are clustered
- Hardware requirements are extreme (64--128 GB+ RAM)
- Solve times measured in days to weeks for preflop
- No real-time solving; all computation is offline
- Convergence measured in abstract game only; full-game exploitability unknown

**Computational requirements:** 64--128 GB RAM, multi-core CPU, days to weeks per solve.

### 2. Simple Omaha: Abstraction-Free Postflop Solving

**Algorithm:** Vector-based Nash equilibrium computation without any abstraction or bucketing. Three local algorithm variants (A1, A2, A3) with different RAM/precision tradeoffs.

**Strengths:**
- No abstraction error; exact Nash equilibrium in the defined game tree
- Highest accuracy among PLO postflop solvers
- Supports PLO, PLO5, and other Omaha variants
- Supports pot-limit, no-limit, and fixed-limit betting

**Weaknesses:**
- Postflop only; cannot solve preflop
- Extreme RAM requirements (32 GB minimum, 128 GB recommended)
- Slow for complex trees with many bet sizing options
- Heads-up only

**Computational requirements:** 32--128 GB RAM, local computation only.

### 3. PLO Genius: Cloud-Based Precomputed Solutions

**Algorithm:** Undisclosed internally, likely CFR-based with significant abstraction. Solutions are precomputed and stored in a cloud database.

**Strengths:**
- Massive solution library (7.9M+ precomputed spots)
- No local hardware requirements
- Covers PLO4 and PLO5, preflop and postflop
- Aggregate report features for board texture analysis across 22,100 possible flop combinations
- Continuously growing library (~20,000 new spots per week)
- Integrated training and hand review features

**Weaknesses:**
- Cannot solve arbitrary custom spots (limited to precomputed library, with on-demand solving available at higher tiers)
- Abstraction quality is opaque to users
- No multiway postflop solutions
- Dependent on cloud availability

**Computational requirements:** None for user; cloud infrastructure.

### 4. GTO Wizard PLO: Multiway + QRE

**Algorithm:** Neural-network-based solver that solves one street at a time. In 2025, switched from Nash Equilibrium to Quantal Response Equilibrium (QRE) for custom solves.

**QRE innovation:** Traditional Nash solvers neglect "ghostlines" -- decision nodes that shouldn't be reached under optimal play. QRE optimizes responses at every node, including abnormal spots, producing strategies that are more robust against imperfect opponents. This reduces average flop exploitability from 0.17% to 0.12% of pot (25% improvement in Nash Distance).

**Strengths:**
- Multiway PLO solving (3-way pots)
- QRE produces more practically useful strategies than Nash
- Cloud-based, fast custom solves (~3 seconds per street for NLHE; PLO likely slower)
- Integrated training platform

**Weaknesses:**
- QRE is less theoretically grounded than Nash for game-theoretic analysis
- PLO support less mature than NLHE support
- Custom solving requires highest subscription tier

**Computational requirements:** None for user; cloud infrastructure.

### 5. FlopHero: Real-Time Postflop with Precomputed Library

Precomputed solutions for flop and turn, with real-time river solving in seconds. Browser-based, no downloads. Automated hand import and EV loss analysis. Newer solver (2025), less battle-tested, heads-up focused, limited documentation on underlying algorithm.

### 6. PokerRL-Omaha: Academic Deep RL

Open-source (GitHub) extension of PokerRL using SD-CFR and Deep CFR with distributed computing for 2--6 player Omaha. No domain-expert abstraction required. Improved NN agent significantly outperforms earlier baselines, with ~11% faster training convergence. However, no superhuman results demonstrated and not competitive with commercial solvers. Research-grade code requiring multi-GPU distributed setup.

---

## PLO-Specific Challenges

### 1. Equity Distribution Clustering

The most fundamental difference between PLO and NLHE solving is the equity distribution. In NLHE, the best preflop hand (AA) has ~85% equity against a random hand, and equity distributions are spread across a wide range. In PLO:

- The best preflop hand (AAKKds) has only ~65% equity against a random hand
- Most preflop matchups between good hands are in the 55/45 to 60/40 range
- Postflop equities rarely exceed 70/30 even for very strong hands against ranges

This clustering has cascading effects on solver quality:

- **Abstraction errors are amplified.** When two hands have 63% and 64% equity and are bucketed together, the resulting mixed strategy may be appropriate for neither. In NLHE, where the spread is wider, bucketing errors are less consequential.
- **More buckets are needed.** PLO requires 3--5x more buckets than NLHE for equivalent abstraction quality, directly increasing memory and compute requirements.
- **Convergence is slower.** CFR convergence guarantees are O(1/sqrt(T)) in regret, but practical convergence depends on the game's "spread" -- tighter equity distributions require more iterations to distinguish between hands.

### 2. The Two-and-Three Constraint

Players must use exactly 2 of 4 hole cards and exactly 3 of 5 board cards. This creates evaluation complexity:

- Each player has C(4,2) x C(5,3) = 60 possible five-card hands per board
- Hand evaluation is approximately 6x slower per comparison than NLHE (which evaluates a single best-of-7)
- Efficient PLO evaluators exist (PokerHandEvaluator, OMPEval) using perfect hashing, achieving ~3,054 ns per PLO4 evaluation
- The constraint also creates non-intuitive hand strengths that complicate human understanding and abstraction design (e.g., four flush cards on the board does not give a flush to a player without two suited cards matching the board)

### 3. Wraps and Multi-Draw Dynamics

PLO features draws with far more outs than are possible in NLHE:

- A 20-out wrap (the maximum in PLO) has approximately 45% equity on the flop with two cards to come
- Players frequently hold multiple simultaneous draws (wrap + flush draw = "monster draw")
- This means postflop ranges are much "stickier" -- players correctly continue with many more hands on the flop than in NLHE
- Solver trees must accommodate this increased continuing frequency, leading to wider ranges and more complex turn/river subtrees

### 4. Blockers, Nuttedness, and Pot Growth

With four hole cards, blockers are more prevalent and strategically significant: holding the A-suited blocks the nut flush draw for every opponent; holding pairs blocks set combinations. Solver abstractions that ignore blocker effects lose critical strategic information. Blocker-aware features must be incorporated into card abstraction, increasing feature dimensionality.

PLO strategy is heavily nut-oriented because many players hold strong draws simultaneously. Ranges must be classified not just by equity but by "nuttedness" -- how often they make the nuts. This dimension is not captured by simple equity histograms, requiring richer feature spaces. Solvers must distinguish between hands with equal equity but different nut potential.

The pot-limit structure causes exponential pot growth (~20x by the river through three streets of pot-sized betting), making SPR dynamics critical and amplifying the cost of early-street solver errors.

---

## Abstraction Techniques

### Card Abstraction

#### Equity Histogram Clustering (Standard Approach)

The dominant technique for card abstraction in poker solvers, including PLO:

1. For each hand, compute the equity distribution against a uniform range of opponent hands across all possible future boards
2. Represent each hand as a histogram of equity values
3. Use k-means clustering with Earth Mover's Distance (EMD) to group similar equity histograms into buckets
4. All hands in a bucket are played identically

**PLO-specific challenges:**
- 270,725 hands to cluster (vs. 1,326 in NLHE), making the pairwise distance matrix ~10^10 entries
- Standard EMD computation is too slow; fast heuristic approximations are required (Ganzfried & Sandholm 2014)
- Equity histograms are more tightly clustered, requiring more buckets for equivalent differentiation
- Typical PLO bucket counts: 200--1,000 preflop, 500--5,000 on each postflop street (vs. 50--200 / 200--500 for NLHE)

#### Potential-Aware Abstraction

Standard equity-only clustering ignores how hand strength changes across streets. Potential-aware abstraction (Gilpin, Sandholm & Sorensen 2007; Ganzfried & Sandholm 2014) considers the full distribution of future-round strength:

- A hand's bucket assignment on the flop considers not just flop equity but the histogram of turn/river equity distributions
- This captures "potential" -- a hand that is currently weak but has many nut draws is grouped differently from a currently weak hand with no draws
- Essential for PLO, where draw-heavy hands (wraps, flush draws) have very different strategic profiles from made hands with equal current equity

#### Imperfect-Recall Abstraction

To reduce memory, solvers use imperfect recall: a player can be made to "forget" their exact preflop hand on the flop, knowing only their bucket. This is standard in poker abstraction but has specific implications for PLO:

- With 6 sub-hands per player, the information lost per step of imperfect recall is greater
- Abstraction pathologies (Waugh et al. 2009) are theoretically more severe in PLO because the hand space is larger -- finer abstractions can paradoxically produce worse strategies than coarser ones
- No published PLO-specific study of abstraction pathology severity exists

#### Blocker-Aware Features

Standard equity histograms do not capture blocker effects. PLO-specific abstraction should include nut flush blocker indicators, set blockers, and straight blockers. These features increase the clustering problem's dimensionality but are critical for PLO strategic accuracy.

### Board Texture and Action Abstraction

With 22,100 raw flop combinations, board texture grouping is necessary. Boards are classified by suit distribution (monotone ~5%, two-tone ~55%, rainbow ~40%), pairing (~17%), and connectedness. PLO Genius uses aggregate reports across texture groups to identify strategic patterns.

Action abstraction is simpler in pot-limit than no-limit (the pot cap eliminates extreme sizings), but PLO requires granularity in the 50%--100% pot range. Common discretization: {fold/check, call, 33% pot, 50% pot, 67% pot, 100% pot} for bets, {min-raise, pot raise} for raises.

---

## CFR Convergence in PLO

### Theoretical Framework

CFR converges to Nash equilibrium in two-player zero-sum games with a regret bound of O(1/sqrt(T)), where T is the number of iterations. Practical convergence depends on:

1. **Game tree size:** More information sets require more iterations for each to be visited sufficiently
2. **Branching factor:** Higher branching slows per-iteration traversal
3. **Equity spread:** Games with tighter equity distributions converge more slowly because strategies are more "indifferent" between actions

PLO suffers on all three dimensions relative to NLHE.

### CFR Variants Applicable to PLO

| Variant | Year | Key Property | PLO Applicability |
|---------|------|-------------|-------------------|
| Vanilla CFR | 2007 | Full tree traversal | Infeasible for PLO (tree too large) |
| MCCFR | 2009 | Samples actions instead of full traversal | Feasible with abstraction; used by MonkerSolver |
| CFR+ | 2014 | Clips negative regrets; faster convergence | Applicable but still slow for PLO |
| DCFR | 2019 | Discounts early iterations; 2--3x faster than CFR+ | Current best for PLO blueprints |
| Deep CFR | 2019 | Neural network regret approximation; no tabular storage | Theoretically ideal for PLO; demonstrated by PokerRL-Omaha |
| SD-CFR | 2019 | Single network (no average strategy network); lower approximation error | Used in PokerRL-Omaha |
| PDCFR+ | 2024 | Predictive discounting + clipping; fastest tabular convergence | Not yet applied to PLO specifically |
| Deep DCFR+ | 2025 | Neural DCFR with predictive advantage networks | Potential PLO application; demonstrated on NLHE |

### Practical Convergence

MonkerSolver PLO preflop solves typically require 10,000--100,000+ iterations to reach acceptable exploitability in the abstract game. Simple Omaha (abstraction-free) requires fewer iterations but each is more expensive. No published convergence rate comparison between NLHE and PLO for the same algorithm exists.

---

## Deep Learning Approaches Adapted for PLO

### Deep CFR / Single Deep CFR

**Concept:** Replace tabular regret storage with neural networks that map information states to action regrets. At each iteration, sample game trajectories, compute counterfactual regrets, and train networks on the resulting data.

**PLO adaptation (PokerRL-Omaha):**
- Extended PokerRL framework to handle 4-card Omaha
- Upgraded distributed computing scheme for the larger state space
- Improved neural network convergence at training onset, decreasing overall convergence time by ~11%
- The improved NN agent significantly outperforms the original NN agent in PLO head-to-head play
- Still far from superhuman; primarily a research demonstration

### ReBeL / TurboReBeL

**Concept:** Recursive Belief-based Learning combines self-play RL with search, maintaining public belief states (PBS) and using CFR within each PBS to compute strategies.

**PLO relevance:** ReBeL was demonstrated on NLHE HU, requiring 4.5 billion samples and 2 million GPU hours. TurboReBeL (2024) reduced this by 250x while maintaining comparable exploitability. Neither has been applied to PLO, but the framework is game-agnostic.

**PLO feasibility assessment:** The PBS space in PLO is much larger (270,725 starting hands vs. 1,326), so even TurboReBeL's 250x speedup may not suffice without additional abstraction. However, combining TurboReBeL with PLO-specific card abstraction is a promising unexplored direction.

### Student of Games

Unified algorithm combining guided search, self-play learning, and game-theoretic reasoning (Schmid et al. 2023, *Science Advances*). Demonstrated on NLHE HU and Scotland Yard. Theoretically applicable to PLO but untested on games with PLO's card complexity; computational demands would be substantially higher.

### Deepsolver (Commercial Neural Approach)

First commercial neural-network-powered solver, combining CFR accuracy with NN speed by solving one street at a time. Currently NLHE only; PLO support announced. Achieves 0.59% average Nash Distance on NLHE. If the per-street approach scales to PLO's hand space, it could offer significant speed advantages.

---

## Open Problems

### 1. Full-Game PLO Solving

No system exists that solves PLO from preflop through river without severe abstraction compromises. Preflop solvers (MonkerSolver) use heavy bucketing. Postflop solvers (Simple Omaha, PioSolver PLO) require preflop ranges as input. Bridging the two without abstraction loss at the preflop-to-flop transition is unsolved.

### 2. Abstraction Quality Measurement

In NLHE, abstraction quality can be assessed by computing exploitability in the abstract game and estimating the gap to the full game. In PLO, even abstract-game exploitability computation is expensive, and full-game exploitability is completely intractable. There is no reliable method to assess how much strategic quality is lost by PLO card abstraction.

### 3. Multiway PLO

PLO is predominantly played 6-max. Current solver support for multiway is limited to pre-solved 3-way spots (GTO Wizard, Vision). Multiplayer Nash equilibrium has fundamental theoretical issues (non-uniqueness, questionable strategic value), and the computational cost of multiway PLO solving is prohibitive. No published multiway PLO equilibrium computation exists for more than 3 players.

### 4. PLO Hi/Lo

The split-pot variant where the pot is divided between the best high hand and the best low hand (if qualifying) doubles the strategic dimension. Players must simultaneously reason about high and low equity, with scooping (winning both) as the primary objective. No solver handles PLO Hi/Lo with meaningful accuracy beyond very simplified abstractions.

### 5. Exploitative Play in PLO

Equilibrium strategies are less valuable in PLO than NLHE because human deviations from optimal play are larger and more exploitable. Developing practical exploitation algorithms for PLO -- adapting strategies in real time based on opponent tendencies -- is an open problem with high commercial value.

### 6. PLO Equity Calculation Speed

While efficient evaluators exist (PokerHandEvaluator: ~3,054 ns per PLO4 hand), the 6x per-comparison overhead compounds across the millions of equity calculations required during solving. Further speedups in PLO equity evaluation (perhaps via neural equity approximation) would directly benefit solver performance.

### 7. Superhuman PLO Agent

No published system claims superhuman PLO play. The combination of abstraction limitations, computational intractability, and lack of standardized evaluation benchmarks means this remains a major open challenge. Statistical evaluation is also harder -- PLO's higher variance (tighter equity distributions, larger pots) requires 10,000+ hands minimum for significance, compared to ~5,000 for NLHE.

---

## Relevance to Myosu

### Architecture Stress Test

PLO is the natural stress test for any solver architecture designed for NLHE:

- Same game structure (betting rounds, community cards, hand rankings) but dramatically larger information set space
- Forces evaluation of whether architectures can scale beyond NLHE without unacceptable abstraction degradation
- Exposes whether a system's computational bottleneck is the solving algorithm (which may scale) or the abstraction layer (which may not)

### Mapping to Autoresearch Config

| Config Parameter | PLO Implication |
|-----------------|-----------------|
| `bucket_count` | Must be 3--5x higher than NLHE for equivalent quality. PLO preflop: 500--1,000 buckets. Postflop: 1,000--5,000 per street. |
| `opponent_model` | Less critical for PLO because human deviations are larger; Nash/QRE strategies exploit less but also leak less |
| `strategy` | Deep CFR or neural-guided MCCFR preferred over tabular CFR for PLO due to state space size |
| `success_threshold` | Should be higher for PLO (more hands needed for statistical significance) |
| `degenerate_tolerance` | PLO solutions are more prone to degenerate strategies in low-frequency nodes; tighter tolerance needed |
| Method family selection | Neural methods (Deep CFR, SD-CFR) should rank higher for PLO than tabular methods |

### Subnet Competition Implications

- **Compute intensity:** PLO solving requires significantly more resources than NLHE. Solver nodes need 64--128 GB RAM minimum.
- **Abstraction quality as differentiator:** In PLO, the quality of card abstraction may matter more than the solving algorithm itself. This creates an interesting dimension for subnet competition where participants compete on abstraction design rather than pure compute.
- **Evaluation difficulty:** Comparing PLO strategies requires more hands for statistical significance (10,000+ minimum) due to higher variance. Evaluation harness must account for this.
- **Market demand:** PLO is the second most popular online poker variant, making it commercially relevant.

### Recommended Solver Architecture for PLO

1. **Blueprint computation:** MCCFR with DCFR discount schedule operating on potential-aware, blocker-enriched card abstraction with 500+ preflop buckets and 2,000+ postflop buckets per street.
2. **Neural value estimation:** Deep CFR or SD-CFR for blueprint computation at scale, avoiding tabular storage limitations.
3. **Real-time subgame solving:** Essential for PLO. Blueprint quality alone is insufficient due to abstraction error amplification from clustered equities. Depth-limited solving with neural leaf evaluation (DeepStack/Libratus lineage).
4. **QRE target:** Consider QRE over Nash for postflop solving, following GTO Wizard's 2025 adoption. QRE produces more robust strategies at low-frequency nodes, which are particularly problematic in PLO.
5. **Evaluation:** High sample counts (10,000+ hands minimum) with variance reduction techniques (AIVAT or similar).

---

## Key Papers and References

| Year | Authors / Source | Title / Description | Venue |
|------|-----------------|---------------------|-------|
| 2007 | Zinkevich, Johanson, Bowling, Piccione | "Regret Minimization in Games with Incomplete Information" (CFR) | NeurIPS |
| 2007 | Gilpin, Sandholm, Sorensen | "Potential-Aware Automated Abstraction of Sequential Games" | AAAI |
| 2009 | Waugh, Zinkevich, Johanson, Isbell, Bowling | "Abstraction Pathologies in Extensive Games" | AAMAS |
| 2013 | Johanson, Burch, Valenzano, Bowling | "Measuring the Size of Large No-Limit Poker Games" | Tech report (UAlberta) |
| 2014 | Ganzfried, Sandholm | "Potential-Aware Imperfect-Recall Abstraction with EMD" | AAAI |
| 2014 | Tammelin | "Solving Large Imperfect Information Games Using CFR+" | arXiv |
| 2017 | Moravcik et al. | "DeepStack: Expert-Level AI in HUNL Poker" | *Science* |
| 2017 | Brown, Sandholm | "Safe and Nested Subgame Solving for IIGs" | NeurIPS |
| 2018 | Brown, Sandholm | "Superhuman AI for HU NL Poker: Libratus" | *Science* |
| 2019 | Brown, Sandholm | "Superhuman AI for Multiplayer Poker" (Pluribus) | *Science* |
| 2019 | Brown, Sandholm | "Solving IIGs via Discounted Regret Minimization" (DCFR) | AAAI |
| 2019 | Brown et al. | "Deep Counterfactual Regret Minimization" | ICML |
| 2019 | Steinberger | "Single Deep CFR" | OpenReview |
| 2020 | Brown et al. | "Combining Deep RL and Search for IIGs" (ReBeL) | NeurIPS |
| 2020 | Steinberger | "Unlocking Potential of Deep Counterfactual Value Networks" (DREAM) | arXiv |
| 2020--2022 | diditforlulz273 | PokerRL-Omaha: SD-CFR and Deep CFR for Omaha | GitHub (open-source) |
| 2023 | Schmid et al. | "Student of Games" | *Science Advances* |
| 2024 | Zhang et al. | "Dynamic Discounted CFR" (DDCFR) | ICLR (Spotlight) |
| 2024 | Liu et al. | "Minimizing Weighted Counterfactual Regret with Optimistic OMD" (PDCFR+) | IJCAI (Oral) |
| 2024 | TurboReBeL authors | "TurboReBeL: 250x Accelerated Belief Learning" | OpenReview |
| 2025 | GTO Wizard | Quantal Response Equilibrium adoption for poker solving | Blog / commercial |
| 2025 | arXiv 2511.08174 | "Deep (Predictive) Discounted CFR" (Deep DCFR+ / Deep PDCFR+) | arXiv |
| 2016--2026 | MonkerWare | MonkerSolver: industry-standard PLO preflop/postflop solver | Commercial |
| 2024--2026 | SimplePoker | Simple Omaha: abstraction-free postflop PLO solver | Commercial |
| 2022--2026 | PLO Genius | Cloud-based PLO4/PLO5 solver (7.9M+ solutions) | Commercial |
| 2025--2026 | FlopHero | Real-time PLO solver with precomputed library | Commercial |
| 2025--2026 | Deepsolver | Neural-network-powered solver (NLHE; PLO announced) | Commercial |

### Additional Resources

- [PokerHandEvaluator](https://github.com/HenryRLee/PokerHandEvaluator): Efficient C/C++ PLO4/PLO5/PLO6 hand evaluation using perfect hashing
- [OMPEval](https://github.com/zekyll/OMPEval): Fast C++ poker evaluator and equity calculator with Omaha support
- [poker-hand-clustering](https://github.com/sammiya/poker-hand-clustering): k-means clustering for poker hands using EMD
- [MonkerSolver syntax reference](https://www.monkerware.com/syntax.html): Custom abstraction configuration for PLO
- [Cardquant PLO strategy](https://cardquant.com/pot-limit-omaha-poker-plo-strategy-new-generation/): Structural approach to PLO that prioritizes conceptual understanding over solver imitation

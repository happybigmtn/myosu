# Short Deck Hold'em (6+): Solving Strategies Research Report

## Executive Summary

Short Deck Hold'em (also called Six Plus or 6+) is a 36-card variant of Texas Hold'em that removes all 2s through 5s, producing a game with roughly 50-100x smaller card state space than standard NLHE while preserving identical betting structure complexity. The reduced deck compresses hand equities (AA has ~63% vs random, compared to ~85% in NLHE), reverses key hand rankings (flush > full house), and creates a fundamentally more action-heavy game with higher variance.

From a solving perspective, Short Deck occupies an attractive middle ground: the game tree is substantially smaller than NLHE, making it more tractable for CFR-based approaches, yet large enough that full enumeration remains infeasible for heads-up play. The 36-card deck yields 630 raw starting hand combinations (vs 1,326 in NLHE) and ~171 strategically distinct hands after suit isomorphism. All standard CFR variants (Vanilla, CFR+, DCFR, MCCFR, Deep CFR) apply directly with a modified hand evaluator. Commercial solvers GTO+ and GTO Wizard provide Short Deck solutions; PioSolver does not. Open-source implementations (TexasSolver, TexasHoldemSolverJava) support Short Deck natively.

The primary research gap is that no published academic work targets Short Deck specifically. All available solving approaches are adaptations of NLHE techniques. Short Deck's tighter equity distributions demand finer-grained card abstractions than NLHE despite the smaller card space, and the universal ante structure (vs blinds) requires distinct preflop solution methodology. For Myosu, Short Deck serves as an ideal intermediate complexity benchmark between fully-solved toy games and intractable NLHE 6-max.

---

## Game Complexity Analysis: 36-Card Deck Effects on Combinatorics

### Deck and Starting Hand Reduction

The removal of 16 cards (2-5 in each suit) produces cascading effects on every combinatorial dimension:

| Metric | Standard (52-card) | Short Deck (36-card) | Reduction Factor |
|--------|-------------------|---------------------|-----------------|
| Total cards | 52 | 36 | 1.44x |
| Ranks | 13 | 9 | 1.44x |
| Starting hand combos (raw) | C(52,2) = 1,326 | C(36,2) = 630 | 2.10x |
| Strategically distinct hands | ~169 | ~171 | ~1x (more suits matter) |
| Possible flops (after 4 cards dealt, HU) | C(48,3) = 17,296 | C(32,3) = 4,960 | 3.49x |
| Possible 5-card boards | C(50,5) = 2,118,760 | C(34,5) = 278,256 | 7.61x |
| 5-card hand combos | C(52,5) = 2,598,960 | C(36,5) = 376,992 | 6.89x |

The reduction in board possibilities (~7.6x) is more significant than the starting hand reduction (~2.1x), meaning the postflop game tree shrinks proportionally more than the preflop tree.

### Suit Isomorphism Differences

In standard NLHE, preflop suit isomorphism collapses 1,326 raw combos into ~169 strategic equivalence classes. In Short Deck, the isomorphism is slightly less aggressive: because only 9 cards exist per suit (vs 13), each known suited card removes a proportionally larger fraction of its suit. The result is ~171 strategic classes -- nearly the same count despite fewer raw combos, meaning each class contains fewer hands on average and suit information carries more weight.

### Game Tree Size Estimates

| Metric | NLHE HU Estimate | Short Deck HU Estimate |
|--------|-----------------|----------------------|
| Game tree nodes | ~10^161 | ~10^60 (estimated) |
| Information sets (no abstraction) | ~10^161 | ~10^58-60 |
| Card state space reduction factor | -- | ~50-100x smaller |
| Action tree (identical bet structure) | Same | Same |

The game tree reduction comes entirely from the card dimension. The action tree (fold/check/call/bet/raise with continuous sizing) is identical to NLHE and dominates the overall complexity. This means Short Deck's primary tractability advantage is in card abstraction quality, not action abstraction.

### Card Removal (Blocker) Effects

Each known card in Short Deck removes 1/36 of the remaining deck vs 1/52 in NLHE -- a 44% stronger blocker effect per card. This amplifies several strategic phenomena:

- Holding one suited card reduces same-suit flush draws by 1/9 of the suit vs 1/13 in NLHE.
- Board texture reads are more precise: fewer possible opponent holdings means narrower ranges.
- Pocket pairs as straight blockers become a critical strategic consideration -- with 36% of flops and 72% of turns offering at least one possible straight, blocking specific straight combinations is a core skill.

---

## Hand Ranking Changes: How Modified Rankings Affect Strategy

### The Flush > Full House Reversal

The most consequential ranking change: flushes outrank full houses. The mathematical basis:

| Hand | 5-Card Combos (52-card) | 5-Card Combos (36-card) | Direction |
|------|------------------------|------------------------|-----------|
| Flush (excl. SF) | 5,108 | ~504 | 10x rarer |
| Full House | 3,744 | ~3,744* | ~Same |
| Three of a Kind | 54,912 | ~13,104 | Less common |
| Straight | 10,200 | ~6,120 | More common per-board |

*Full house count in 36-card 5-card hands: with 9 ranks and 4 suits, C(9,1)*C(4,3)*C(8,1)*C(4,2) = 9*4*8*6 = 1,728 (corrected for 5-card). The key insight: flush combinations drop by ~75% because C(9,5) = 126 per suit vs C(13,5) = 1,287 per suit. Full houses become relatively more common because the compressed rank space (9 ranks vs 13) makes pairing more frequent.

### Strategic Implications of Ranking Changes

The flush > full house rule creates several cascade effects:

1. **Flush draws are premium draws.** A flush draw in Short Deck has ~5 outs (vs ~9 in NLHE) and completes ~30% by the river (vs ~35%), but the completed hand is now the 4th-strongest category. Suited hands gain significant preflop value.

2. **Full houses are devalued.** Set mining is still profitable (~17% flop set frequency vs ~12% in NLHE), but the set-to-full-house pathway no longer guarantees a near-nut hand. Full houses lose to flushes.

3. **Straight prevalence reshapes ranges.** With only 6 distinct straights possible (A-6-7-8-9 through T-J-Q-K-A) and highly connected boards, straights dominate the mid-strength range. Open-ended straight draws complete ~45% by the river (vs ~32% in NLHE).

4. **Equity convergence.** Hand equities cluster tightly: AA has ~63% equity vs a random hand (vs ~85% in NLHE). Connected suited hands like T9s have 40-45% equity against premium pairs. This creates fundamentally more multiway action.

### Trips vs Straights Debate

Most commercial venues rank straights above trips (the PokerStars/GGPoker/Triton standard), despite straights being mathematically more common than trips in a 36-card deck. This is a pragmatic choice: the alternative ranking confuses NLHE players transitioning to Short Deck. Solver implementations must support both ranking systems as a configuration option.

---

## Current Best Approaches for Solving Short Deck

### 1. Discounted CFR (DCFR) with Modified Evaluator

**Algorithm:** Discounted CFR (Brown & Sandholm, 2019) applies non-uniform weighting to past iterations, dramatically accelerating convergence. The regret and strategy updates discount older iterations using parameters (alpha, beta, gamma) that control how aggressively early iterations are forgotten.

**Application to Short Deck:** DCFR is the de facto standard algorithm in open-source Short Deck solvers (TexasSolver, TexasHoldemSolverJava). The only required modification is swapping the hand evaluator to one that implements Short Deck rankings (flush > full house, A-6-7-8-9 wheel).

**Strengths:**
- Fastest convergence among tabular CFR variants on poker instances. Achieves target exploitability in ~970 iterations where CFR+ requires ~471,000 iterations.
- Directly applicable with evaluator swap -- no algorithmic changes needed.
- Smaller game tree means each iteration is cheaper than NLHE.

**Weaknesses:**
- Fixed discounting parameters may not be optimal for Short Deck's different equity distribution.
- Still requires explicit card abstraction for full-game (preflop-to-river) solving.

**Compute:** TexasSolver (C++) solves postflop Short Deck spots in seconds to minutes on consumer hardware. Full preflop-to-river blueprint requires days on multi-core machines.

### 2. Dynamic Discounted CFR (DDCFR)

**Algorithm:** Published at ICLR 2024 (Spotlight), DDCFR replaces DCFR's fixed discounting scheme with a learned, dynamic policy. It formulates the CFR iteration process as an MDP and uses evolution strategies (ES) to optimize the discounting schedule per-iteration.

**Application to Short Deck:** DDCFR's dynamic schedule could be particularly valuable for Short Deck because the tighter equity distributions may benefit from different discounting behavior than NLHE. The learned policy generalizes across game sizes, meaning a policy trained on smaller poker variants could transfer to Short Deck.

**Strengths:**
- Strictly dominates fixed-schedule DCFR across all tested games.
- Automatically adapts discounting to game-specific properties.
- Code available (github.com/rpSebastian/DDCFR).

**Weaknesses:**
- Requires a meta-learning phase to train the discounting policy.
- Marginal improvement over well-tuned DCFR in practice.

**Compute:** Meta-learning overhead is modest; per-iteration cost identical to DCFR.

### 3. Monte Carlo CFR (MCCFR)

**Algorithm:** MCCFR samples subsets of the game tree per iteration rather than performing full traversal, reducing per-iteration cost at the expense of higher variance in convergence.

**Application to Short Deck:** MCCFR's sampling advantage is less pronounced in Short Deck than NLHE because the game tree is already smaller. However, for full preflop-to-river solving with multiple bet sizes, MCCFR remains practical.

**Strengths:**
- Sub-linear memory scaling (only visited states need storage).
- Retains theoretical convergence guarantees.
- Most practical method for full-game blueprint computation.

**Weaknesses:**
- Higher variance per iteration than full-traversal methods.
- Convergence can be slower in terms of wall-clock time for smaller games where full traversal is feasible.

**Compute:** Suitable for consumer hardware. The 2025 survey (arxiv:2509.23747) shows MCCFR achieves Top-1 accuracy of 1.000 with KL divergence of 0.015, the cleanest GTO convergence among tested methods.

### 4. Deep CFR

**Algorithm:** Replaces tabular regret storage with neural networks that approximate cumulative regrets and average strategy, enabling generalization across similar states without explicit card abstraction.

**Application to Short Deck:** Deep CFR's abstraction-free approach is compelling for Short Deck because the tighter equity distributions make traditional bucket-based abstraction lossy. Neural networks can learn the nuanced equity relationships directly.

**Strengths:**
- Eliminates need for hand-crafted card abstraction.
- Scales to the full game without explicit state enumeration.
- Naturally handles Short Deck's different equity landscape.

**Weaknesses:**
- Slower convergence than tabular MCCFR for games small enough to enumerate.
- Requires significant GPU resources for training.
- Less mature tooling for Short Deck specifically.

**Compute:** GPU-intensive. Training a full Short Deck blueprint would require multi-GPU setups for days, though the smaller game tree reduces this vs NLHE.

### 5. GPU-Accelerated CFR

**Algorithm:** Recent work (arxiv:2408.14778, 2024) reformulates CFR as dense/sparse matrix operations parallelizable on GPUs, achieving 200-400x speedup over CPU implementations.

**Application to Short Deck:** Short Deck's smaller game tree fits more easily into GPU VRAM, making GPU-accelerated CFR particularly attractive. A 24GB GPU (RTX 4090) could potentially hold a significant fraction of the Short Deck game tree in memory.

**Strengths:**
- Orders of magnitude speedup over CPU implementations.
- Enables rapid iteration on abstraction and betting configurations.
- Short Deck's size is a sweet spot for GPU memory constraints.

**Weaknesses:**
- Higher memory usage than CPU variants (dense representation).
- Implementation complexity (CUDA/CuPy).
- Speedup diminishes for very large games that exceed VRAM.

**Compute:** RTX 4090 (24GB VRAM) demonstrated 401x speedup over Python, 204x over C++ baselines.

### 6. Real-Time Subgame Solving (ReBeL / Pluribus Architecture)

**Algorithm:** Compute a coarse preflop-to-river blueprint offline, then solve specific subgames in real-time during play with higher resolution. ReBeL (Brown et al., 2020) unifies this with learned value functions; Pluribus (2019) uses depth-limited search with a population of blueprint policies.

**Application to Short Deck:** The smaller game tree allows computing a higher-resolution blueprint than feasible for NLHE, reducing the burden on real-time search. The blueprint could potentially cover turn and river with near-exact solutions, requiring real-time solving only for novel preflop/flop spots.

**Strengths:**
- Best theoretical framework for near-optimal play.
- Smaller blueprint + cheaper subgame solving = more practical for Short Deck.
- Directly applicable (evaluator swap only).

**Weaknesses:**
- Complex engineering (blueprint + real-time search + value network).
- Overkill for Short Deck postflop-only solving (commercial solvers handle this).
- No published Short Deck-specific implementation.

**Compute:** Blueprint: days on multi-core CPU. Real-time search: milliseconds to seconds per decision.

---

## Abstraction Techniques for Short Deck

### Card Abstraction

The standard approach clusters strategically similar hands into "buckets" to reduce the information set space. For Short Deck:

**Equity-Based Bucketing (EHS/EHS2):**
- Expected Hand Strength (EHS) groups hands by win probability against a random hand.
- EHS2 (potential-aware) additionally considers the distribution of future equity outcomes.
- Short Deck's tighter equity distributions mean EHS clusters are denser and less separable -- the same number of buckets captures less strategic distinction.

**Earth Mover's Distance (EMD) Clustering:**
- The state of the art for NLHE: build equity histograms for each hand showing the distribution of equity on future streets, then cluster using k-means with EMD as the distance metric.
- For Short Deck, equity histograms are narrower (less variance in outcomes), requiring more buckets to achieve the same strategic resolution.
- The PAAEMD algorithm (Ganzfried et al., 2014) and its successor KrwEmd (2024, arxiv:2403.11486) are directly applicable.

**Practical Bucket Counts:**

| Street | NLHE Typical Buckets | Short Deck Recommended | Rationale |
|--------|---------------------|----------------------|-----------|
| Preflop | 8-20 | 10-25 | Tighter equities need finer granularity |
| Flop | 50-200 | 50-200 | Similar range despite fewer raw combos |
| Turn | 100-500 | 80-300 | Fewer possible boards, slightly coarser OK |
| River | 100-500 | 80-300 | Same logic as turn |

The counterintuitive finding: Short Deck may need *more* buckets per raw hand than NLHE on early streets, despite having fewer hands. The reason is that equity distributions are compressed -- a wider bucket in equity space captures proportionally more strategic variation.

### Action Abstraction

Action abstraction is identical to NLHE: discretize the continuous bet sizing space into a finite number of sizes (e.g., 33%, 50%, 75%, 100%, 150% pot). Short Deck's ante structure creates larger initial pots, which shifts optimal bet sizing distributions:

- Preflop open raises are typically smaller relative to the pot (the pot already contains significant dead money from antes).
- Postflop bet sizing tends toward polarized (small or all-in) due to tighter equity margins.
- All-in preflop as an open raise is GTO-correct at shallow stacks (40-50 ante), a strategy that does not exist in standard NLHE blind structures.

### Suit Isomorphism

Standard suit isomorphism (treating hearts/diamonds/clubs/spades as interchangeable when no flush draw is present) applies identically to Short Deck. On the flop, boards with 0, 1, 2, or 3 suits can be collapsed:

- Monotone flop (3 suited): all 4 suit permutations are isomorphic -> 1 representative.
- Two-tone flop (2 suits): 6 permutations collapse to a smaller set.
- Rainbow flop (3 suits): highest reduction.

The savings are the same percentage as NLHE, applied to a smaller base.

---

## Commercial and Open-Source Solver Landscape

### Commercial Solvers

| Solver | Short Deck Support | Notes |
|--------|--------------------|-------|
| GTO Wizard | Yes | Cloud-based, pre-solved solutions for Short Deck. Supports preflop and postflop. Subscription model. |
| GTO+ | Yes (dedicated version) | Standalone desktop solver, one-time license. Dedicated Short Deck (6+) variant available. |
| PioSolver | No | Focused exclusively on standard Texas Hold'em. No Short Deck evaluator. |
| SimplePostflop | Yes | Short Deck mode available. Desktop solver. |
| PokerStove / Equilab | No native support | Standard evaluators; require modification for Short Deck rankings. |

GTO Wizard is the most comprehensive commercial option, offering both pre-solved strategy libraries and training tools for Short Deck. GTO+ is the primary option for users who want to run their own postflop solves with custom configurations.

### Open-Source Solvers

| Project | Language | Short Deck | Algorithm | Status |
|---------|----------|------------|-----------|--------|
| TexasSolver | C++ | Yes | DCFR | Active, high performance |
| TexasHoldemSolverJava | Java | Yes | CFR++ / DCFR | Active, cross-language bindings |
| wasm-postflop | Rust/WASM | No | DCFR | Suspended (Oct 2023) |
| opensolver | Rust | No | DCFR | Active, UPI compatible |
| Noam Brown's poker_solver | Python | No (NLHE river only) | CFR variants | Reference implementation |

**TexasSolver** is the standout open-source option: it matches or exceeds PioSolver performance on some benchmarks, is fully open-source, supports Short Deck natively, and runs on all major platforms. Its C++ implementation is 5x faster than the Java version (TexasHoldemSolverJava) with 1/3 the memory. Both solvers support configurable CFR variants (Vanilla, CFR+, DCFR) and multi-threaded solving.

### Hand Evaluator Implementations

No major hand evaluation library (PokerHandEvaluator, Cactus Kev, treys) ships with Short Deck support out of the box. The PH Evaluator project has an open issue for Short Deck support that remains unresolved -- generating the modified lookup tables (flush and non-flush) for a 9-rank deck is non-trivial. TexasSolver and TexasHoldemSolverJava include their own internal Short Deck evaluators.

For Myosu, this means building or adapting a hand evaluator is a prerequisite. The core changes: (1) remove ranks 2-5 from the deck, (2) reorder flush above full house, (3) handle A-6-7-8-9 as the lowest straight, (4) regenerate lookup tables for the 36-card space.

---

## Open Problems

### 1. No Published Short Deck-Specific Research

As of March 2026, no peer-reviewed academic paper targets Short Deck Hold'em solving specifically. All solver approaches are adaptations of NLHE techniques. This means:
- No benchmark exploitability numbers exist for Short Deck.
- No public comparison of abstraction quality across methods for the 36-card deck.
- The optimal bucket count / EMD parameters for Short Deck's equity distributions are unknown.

### 2. Ante Structure Preflop Solving

The button-blind-with-antes structure (Triton format) changes preflop dynamics fundamentally vs standard blinds. Dead money from antes creates larger initial pots, incentivizing wider play and making preflop all-in a viable open-raise at shallow depths. Existing preflop charts (RangeConverter) exist for specific configurations, but no systematic study of how ante structure affects GTO preflop strategy has been published.

### 3. Abstraction Quality Under Equity Compression

Short Deck's tighter equity distributions (hands cluster closer to 50% equity) mean that standard NLHE abstraction bucket boundaries may be suboptimal. The research question: does Short Deck require proportionally more buckets to achieve the same strategic resolution, or does the smaller raw hand space compensate?

### 4. Multiway Short Deck

Short Deck is commonly played 5-6 handed with an ante structure that incentivizes multiway pots. Multiway solving faces the same theoretical barriers as NLHE 6-max (non-unique equilibria, PPAD-hardness), compounded by Short Deck's tighter equities making multiway pots even more common. No Pluribus-style multiway solver has been applied to Short Deck.

### 5. Exploitability Bounds

Because the Short Deck game tree is smaller than NLHE, tighter exploitability bounds should be computationally feasible. Computing the exploitability of a Short Deck HU strategy (via best-response computation) would be significantly cheaper than for NLHE HU, enabling more rigorous solver quality evaluation. This has not been done publicly.

### 6. Cross-Variant Transfer Learning

Can a neural network trained on NLHE equity relationships transfer to Short Deck with fine-tuning? The rank-compressed deck and altered hand rankings suggest significant distributional shift, but the underlying poker mechanics are identical. This transfer learning question is unexplored.

---

## Relevance to Myosu

### Why Short Deck Matters for the Subnet

Short Deck serves multiple roles in Myosu's game-solving architecture:

1. **Intermediate Complexity Benchmark.** Between fully-solved toy games (Kuhn, Leduc) and intractable NLHE 6-max, Short Deck HU offers a game large enough to stress-test real solving infrastructure while small enough to compute meaningful exploitability bounds. This makes it the ideal validation target for solver quality.

2. **Evaluator Correctness Test.** The modified hand rankings (flush > full house, A-6-7-8-9 wheel) create a non-trivial hand evaluator implementation challenge. Getting this wrong silently corrupts all solver output. Short Deck forces the framework to support configurable hand rankings from day one.

3. **Ante Structure Support.** The button-blind-with-antes format requires the solver framework to handle ante-based pot calculations, not just blind-based. This generalizes the framework for other ante-based games.

4. **Commercial Relevance.** Short Deck is actively played at the highest stakes (Triton Poker Series) and on major online platforms (GGPoker, PokerStars). Solving quality for Short Deck has direct commercial value.

5. **Faster Iteration.** The ~50-100x smaller card state space means blueprint computation, abstraction experiments, and exploitability measurement all run proportionally faster. This enables rapid prototyping of solver improvements before scaling to NLHE.

### Architecture Requirements

| Requirement | Detail |
|------------|--------|
| Configurable hand evaluator | Must support pluggable ranking systems (flush > FH, trips vs straight toggle) |
| Ante structure support | Preflop pot calculation: sum(antes) + button_blind = starting pot |
| 36-card deck primitives | Deck, shuffle, deal must support arbitrary card removal |
| Abstraction pipeline | EMD clustering must work with Short Deck equity distributions |
| Exploitability computation | Best-response oracle for Short Deck HU as a quality metric |

### Recommended Implementation Order

1. Implement Short Deck hand evaluator with configurable rankings (flush > FH by default, trips-vs-straight toggle).
2. Adapt CFR pipeline (DCFR preferred) with the modified evaluator.
3. Solve Short Deck HU postflop spots and validate against GTO+/GTO Wizard solutions.
4. Compute full preflop-to-river HU blueprint.
5. Measure exploitability of the blueprint via best-response computation.
6. Use exploitability as the quality benchmark for abstraction and algorithm experiments.

---

## Key Papers and References

### Foundational CFR and Poker AI

| Year | Paper / Resource | Contribution |
|------|-----------------|--------------|
| 2007 | Zinkevich et al., "Regret Minimization in Games with Incomplete Information" (NIPS) | Introduced CFR |
| 2014 | Tammelin, "Solving Large Imperfect Information Games Using CFR+" | CFR+ with non-negative regrets, solved Limit HU |
| 2017 | Brown & Sandholm, "Superhuman AI for heads-up no-limit poker: Libratus" (Science) | Blueprint + subgame solving architecture |
| 2019 | Brown & Sandholm, "Superhuman AI for multiplayer poker" (Science) | Pluribus: 6-player NLHE, depth-limited search |
| 2019 | Brown & Sandholm, "Solving Imperfect-Information Games via Discounted Regret Minimization" (AAAI) | DCFR: fastest tabular CFR variant on poker |
| 2019 | Brown et al., "Deep Counterfactual Regret Minimization" (ICML) | Neural CFR without explicit abstraction |
| 2020 | Brown et al., "Combining Deep Reinforcement Learning and Search for Imperfect-Information Games" (NeurIPS) | ReBeL: unified RL+search for imperfect info |

### Recent Advances (2023-2026)

| Year | Paper / Resource | Contribution |
|------|-----------------|--------------|
| 2024 | Zhang et al., "Dynamic Discounted CFR" (ICLR Spotlight) | Learned dynamic discounting schedule, dominates fixed DCFR |
| 2024 | "A Survey on Game Theory Optimal Poker" (arxiv:2401.06168) | Comprehensive survey of GTO poker techniques |
| 2024 | "Signal Observation Models and Historical Information Integration in Poker Hand Abstraction" (arxiv:2403.11486) | KrwEmd: improved imperfect-recall card abstraction |
| 2024 | "GPU-Accelerated Counterfactual Regret Minimization" (arxiv:2408.14778) | 200-400x GPU speedup for CFR |
| 2025 | "Beyond Game Theory Optimal: Profit-Maximizing Poker Agents" (arxiv:2509.23747) | MCCFR vs Deep CFR vs NFSP comparison |
| 2025 | "Comparative analysis of extensive form zero sum game algorithms" (Nature Scientific Reports) | 10-algorithm comparison across poker variants |
| 2025 | "Robust Deep Monte Carlo CFR" (arxiv:2509.00923) | Improved neural CFR stability |

### Short Deck Specific Resources

| Resource | Type | Notes |
|----------|------|-------|
| TexasSolver (github.com/bupticybee/TexasSolver) | Open-source solver | C++, Short Deck native support, DCFR |
| TexasHoldemSolverJava (github.com/bupticybee/TexasHoldemSolverJava) | Open-source solver | Java, Short Deck, CFR++/DCFR |
| GTO+ Short Deck (gtoplus.com/shortdeck/) | Commercial solver | Dedicated 6+ version |
| GTO Wizard (gtowizard.com) | Commercial platform | Short Deck pre-solved solutions |
| RangeConverter Short Deck Charts (rangeconverter.com) | Strategy resource | 24 preflop charts for 5-max 50a |
| Cardquant Short Deck Math series | Strategy analysis | Board probability and blocker analysis |
| PokerNews Short Deck Strategy (pokernews.com) | Reference | Odds, probabilities, hand rankings |
| Dr. Brian's Short Deck Hand Probabilities (drbrian.space) | Mathematical analysis | Complete 5-card hand probability tables |

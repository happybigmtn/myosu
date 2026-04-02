# Solving Strategies for Riichi Mahjong (立直麻雀)

**Date:** 2026-03-30
**Scope:** Comprehensive survey of solving methods and AI research, 2015--2026
**Status:** Research report (no implementation)

---

## Executive Summary

Riichi Mahjong (Japanese Mahjong) stands as one of the most complex imperfect-information games actively studied in AI research. With 136 tiles, four players, enormous hidden information (information sets estimated at 10^48), a combinatorial yaku-based scoring system, and deeply intertwined offensive/defensive dynamics, it exceeds the complexity of heads-up poker by many orders of magnitude. Unlike two-player zero-sum games where CFR guarantees convergence to Nash equilibrium, four-player mahjong breaks those theoretical guarantees, forcing the field toward deep reinforcement learning as the dominant paradigm.

The historical progression runs from early rule-based systems and Monte Carlo simulations (Mizukami & Tsuruoka, 2015) through the watershed moment of Microsoft Research Asia's Suphx (Li et al., 2020), which achieved 10-dan on Tenhou -- placing it above 99.99% of ranked human players. Suphx introduced oracle guiding (training with perfect information features, then gradually dropping them out), global reward prediction via GRUs, and run-time policy adaptation via parametric Monte Carlo. In 2023, Tencent AI Lab's LuckyJ surpassed Suphx with a stable rank of 10.68 dan, using a neural-based CFR algorithm augmented with game tree search features, reaching 10-dan in only 1,321 games versus Suphx's 5,373. The open-source Mortal project (Equim-chan, 2022--present) provides a reproducible, community-accessible deep RL agent written in Rust with Python neural network inference. On the academic front, Tjong (Li et al., 2024) introduced transformer-based hierarchical decision-making with a "fan backward" reward shaping technique, and recent work on Mix-PPO (2024), LsAc*-MJ (2024), and MPPO-based styled agents (2025) continues to push the boundary on resource efficiency, reward engineering, and agent diversity.

Despite these advances, Riichi Mahjong remains unsolved. No Nash equilibrium exists for the four-player game, online search remains rudimentary compared to poker, multi-hand game-level strategy (placement awareness across an 8+ hand hanchan) is poorly modeled, and compute requirements remain substantial. For Myosu, Riichi Mahjong represents a tier-1 challenge: the largest player base of any non-poker game in the survey, rich existing infrastructure for evaluation (Tenhou, Mahjong Soul), and a domain where deep RL with domain-specific reward shaping is the clear architectural choice.

---

## Game Complexity Analysis

### Tile Space and Structure

| Property | Value |
|----------|-------|
| Total tiles | 136 (34 unique x 4 copies) |
| Suited tiles | 108 (3 suits x 9 ranks x 4 copies) |
| Honor tiles | 28 (4 winds x 4 + 3 dragons x 4) |
| Red dora (standard) | 3 (one 5 per suit) |
| Players | 4 |
| Hand size | 13 tiles (14 on draw) |
| Dead wall | 14 tiles |
| Dora indicators | 1--5 (base + kan dora) |

### Information Geometry

Each player observes their own 13-tile hand, all public discards (ordered rivers for each player), open melds, dora indicators, and point standings. The hidden state comprises the three opponents' hands (39 tiles) and the remaining wall (typically 50--70 tiles at game start, decreasing per draw). Critically, a player cannot distinguish which hidden tiles are in opponents' hands versus the wall, making the information set size astronomical.

| Metric | Estimate | Source |
|--------|----------|--------|
| Initial hand combinations C(136,13) | ~10^16 | Combinatorial |
| Strategic equivalence classes | ~10^9 | Suit isomorphism reduction |
| Information sets per hand | ~10^15--10^20 | Suphx paper estimates |
| Average information set size | ~10^48 | Suphx paper; cf. HULHE ~10^3 |
| Game tree nodes per hand | ~10^20--10^30 | Literature consensus |
| Full hanchan game tree | Intractable | Sequence of 8+ correlated hands |
| Branching factor (discard) | ~10--14 unique options | From 14-tile hand |
| Branching factor (total) | ~34 effective | Discard + binary claim decisions |

### Why Riichi Mahjong Is Hard

1. **Four-player dynamics.** Nash equilibria lack the clean two-player guarantees. Placement-based payoffs (uma) create non-zero-sum incentives where implicit cooperation against the leader emerges naturally.

2. **Massive hidden information.** With 39 opponent tiles and 50+ wall tiles unknown, belief tracking is combinatorially explosive. This dwarfs poker's hidden information by many orders of magnitude.

3. **Complex scoring.** The yaku system (40+ valid patterns), fu calculation, and limit hand tiers create a nonlinear, discontinuous reward landscape. A single tile can swing hand value from 1,000 points to mangan (8,000+).

4. **Temporal depth.** A hanchan spans 8+ hands with placement-dependent incentives. Optimal play in hand 7 depends on cumulative point standings, remaining hands, and dealer/non-dealer status.

5. **Defensive complexity.** Betaori (pure defense), suji reasoning, kabe analysis, and discard reading form an intricate defensive meta-game that has no analog in poker.

6. **Sparse rewards.** Most hands end in draws (ryuukyoku). When wins occur, the signal is delayed across many sequential decisions, creating a severe credit assignment problem.

---

## Historical Progression

### Phase 1: Rule-Based and Statistical Systems (pre-2018)

Early mahjong AI relied on hand-coded heuristics encoding expert knowledge: tile efficiency (shanten minimization), basic suji/kabe defense, and simple expected value calculations. Notable work includes:

- **Bakuuchi** and other Japanese research prototypes: combined rule-based discard selection with basic probability tables.
- **Mizukami & Tsuruoka (2015):** "Building a Computer Mahjong Player Based on Monte Carlo Simulation." Used Monte Carlo sampling to evaluate discard choices by simulating future game states. Achieved competent but sub-professional play. This work established the viability of simulation-based approaches for mahjong's enormous state space.
- **Kaneko (2017):** Feature-based evaluation functions trained on professional game records via supervised learning. Demonstrated that supervised learning from expert data could produce reasonable policies but plateaued well below professional level.
- **Wang et al. (2021):** Combined domain knowledge (tile counting, scoring rules) with game-tree search for Chinese mahjong, showing that hybrid approaches could outperform pure heuristics.

These systems had a common ceiling: they could not learn the subtle trade-offs between offense and defense, lacked game-level strategy, and struggled with the nonlinear scoring system.

### Phase 2: Suphx and the Deep RL Breakthrough (2019--2020)

Microsoft Research Asia's Suphx (Li et al., 2020, published in Nature) was the watershed moment. Key facts:

- **Achievement:** Stable 10-dan on Tenhou, above 99.99% of ranked human players. First AI to demonstrably surpass most professional players.
- **Training pipeline:** Supervised pre-training on top human player data from Tenhou, followed by distributed self-play RL using policy gradient with importance sampling.
- **Compute:** 44 GPUs per agent (4 Titan XP parameter servers + 40 Tesla K80 self-play workers), 2 days training per agent, 1.5 million games per agent. Five separate models (discard, riichi, chi, pon, kan).
- **Reached 10-dan:** After 5,373 ranked games on Tenhou.

Suphx introduced three innovations specific to mahjong's challenges (detailed below in Current Best Approaches).

### Phase 3: NAGA and Commercial AI (2018--present)

NAGA, developed at Dwango's DMV (Django Media Village) lab, took a different path: supervised learning from high-ranking player records as the primary training signal, with RL fine-tuning. NAGA consists of four CNN models (discard, calling, riichi, kan) and achieved 8-dan on Tenhou initially, with a later bot (NAGA25) reaching 10-dan. NAGA became commercially significant as a paid game analysis service, allowing players to review their games against the AI's recommendations. This commercial viability demonstrated the market for mahjong AI tools.

### Phase 4: LuckyJ, Mortal, and the Modern Era (2023--2026)

The period from 2023 onward has seen rapid diversification:

- **LuckyJ (Tencent AI Lab, 2023):** Reached 10-dan on Tenhou with a stable rank of 10.68, surpassing both Suphx and all human players with 1000+ games. Used neural-based CFR with game tree search features. Achieved this in only 1,321 games.
- **Mortal (Equim-chan, 2022--present):** Open-source deep RL agent for riichi mahjong. Written in Rust (emulator) + Python (neural networks). Achieves up to 40K hanchans/hour. Versions: 3.0/3.1 (Jan 2023), 4.0 (Nov 2023), 4.1 (Aug 2024). Widely used for game analysis via mjai-reviewer.
- **Tjong (Li et al., 2024):** Transformer-based architecture with hierarchical decision-making and fan backward reward shaping. 15M parameters, trained on 0.5M games over 7 days with 2 GPUs. Top 1% on Botzone.
- **LsAc*-MJ (2024):** Low-resource RL model using LSTM networks with optimized A2C algorithm, targeting data-scarce mahjong variants.
- **Mix-PPO (2024):** Novel PPO variant for four-player Chinese mahjong, published in IEEE Transactions on Games.
- **MJ_RM (2025):** Res2Net-LSTM architecture with improved distributed PPO and three-stage reward mechanism for Chinese Standard Mahjong.
- **MPPO styled agents (2025):** Mixed Proximal Policy Optimization for creating mahjong bots with distinct play styles via learning from demonstration.
- **Mxplainer (2025--2026):** Parameterized search algorithm that imitates black-box mahjong agents for interpretability, achieving 92%+ top-3 action prediction accuracy.

---

## Current Best Approaches

### Suphx (Microsoft Research Asia)

**Architecture:** Five deep CNNs, one per decision type (discard, riichi, chi, pon, kan). State representation uses multiple 34x1 channels encoding tile sets, discard sequences, accumulated scores, and categorical features. The discard model is the most complex (50-layer ResNet in some accounts).

**Key innovations:**

1. **Oracle guiding.** The core insight: train an "oracle agent" that has access to perfect information (all hidden tiles), then gradually drop out the perfect features during training so the agent transitions to operating on imperfect information only. This is implemented via a dropout schedule on oracle features. The oracle becomes a teacher that guides the normal agent toward better policies without the student ever directly accessing hidden information at test time.

2. **Global reward prediction (GRP).** Uses a gated recurrent unit (GRU) to predict the final game-level outcome (placement) from intermediate states. This provides a denser reward signal than the sparse hand-level outcomes, addressing the credit assignment problem across the many decisions within a hand and across hands in a hanchan.

3. **Run-time policy adaptation (pMCPA).** At the beginning of each round, Suphx performs parametric Monte Carlo simulations to adapt its policy to the specific tile distribution it has been dealt. This is a lightweight form of online search that adjusts the pre-trained policy using the observed private hand, improving play without the full cost of game-tree search.

**Strengths:** Superhuman performance. Well-documented. Established the deep RL paradigm for mahjong.

**Weaknesses:** Requires substantial compute (44 GPUs, 2 days per agent). Five separate models create engineering complexity. No open-source release. Published in 2020; surpassed by LuckyJ in 2023.

### LuckyJ (Tencent AI Lab)

**Architecture:** Neural-based CFR algorithm augmented with game tree search results as features. The system learns policies that incorporate both equilibrium-seeking (via CFR) and learned evaluation (via neural networks).

**Key innovations:**

1. **Neural CFR for mahjong.** Adapts counterfactual regret minimization -- traditionally limited to two-player games -- to the four-player setting by using neural network function approximation and game tree search features. While CFR lacks convergence guarantees in multiplayer games, the neural approximation provides practical effectiveness.

2. **Balanced attack and defense.** LuckyJ reportedly achieves notably balanced offensive and defensive play, with a particularly low dealt-in (houjuu) rate while maintaining high win rate and hand value.

**Performance:** Stable rank 10.68 dan on Tenhou (highest recorded for any AI or human). Reached 10-dan in only 1,321 games -- vastly more efficient than Suphx (5,373) or NAGA (26,598).

**Weaknesses:** Proprietary. Limited published architectural details. No open-source release.

### Mortal (Open Source)

**Architecture:** End-to-end deep learning model using model-free reinforcement learning. Rust-based game emulator provides high-throughput simulation (40K hanchans/hour). Python neural network inference. Compatible with mjai protocol for interoperability with Tenhou and Mahjong Soul logs.

**Key features:**

1. **Reproducibility.** Fully open-source (AGPL-3.0 license), the most accessible strong mahjong AI for research and analysis.
2. **Ecosystem.** mjai-reviewer allows turn-by-turn analysis of game logs. Integrations exist with Mahjong Soul, Tenhou, and Riichi City via tools like Akagi.
3. **Iterative improvement.** Multiple major versions (3.x, 4.x) with progressive improvements in play strength.

**Performance:** Strong play at high dan levels. Widely used by the English-speaking mahjong community for game analysis. Exact Tenhou ranking less publicized than Suphx/LuckyJ.

**Weaknesses:** Detailed architectural documentation (layer counts, parameter sizes) is not fully public. Performance likely below LuckyJ/Suphx at the very top level.

### NAGA (Dwango/DMV)

**Architecture:** Four CNN models (discard, calling, riichi, kan). Policy-based approach with supervised learning from high-ranking player records as the primary signal, augmented with RL fine-tuning.

**Key features:**

1. **Data-driven.** Leverages large datasets of expert human play rather than relying purely on self-play.
2. **Commercial viability.** Subscription-based game analysis service demonstrates real market demand for mahjong AI tools.
3. **Human-like play.** Training from expert data produces play patterns recognizable to human players, potentially more useful for analysis/coaching.

**Performance:** NAGA25 bot reached 10-dan on Tenhou. Currently ranked among the top three mahjong AIs alongside LuckyJ and Suphx.

**Weaknesses:** Supervised learning ceiling -- may inherit human biases and fail to discover novel strategies that pure RL would find. Proprietary.

### Tjong (Transformer-Based, 2024)

**Architecture:** Transformer-based with self-attention mechanisms. 15M parameters. Hierarchical decision-making decomposes choices into action decision (chi/pon/kan/pass/win/discard) and tile decision (which tile), reducing the output dimension and simplifying learning.

**Key innovations:**

1. **Fan backward technique.** Retroactively allocates the scoring of winning hands back to every decision in the game, creating a dense reward signal from the final outcome. This avoids training a separate value network (cf. Suphx's GRP) by directly reshaping the reward.
2. **Transformer architecture.** Self-attention captures tile pattern relationships more naturally than CNNs, potentially enabling better long-range dependency modeling within and across turns.

**Performance:** Outperformed CNN, MLP, RNN, ResNet, and ViT baselines by up to 230% score differential. Top 1% on Botzone leaderboard. Action decision accuracy 94.63%, claim decision accuracy 98.55%, discard decision accuracy 81.51%.

**Weaknesses:** Tested primarily on Botzone (Chinese Official Mahjong), not Tenhou (Riichi Mahjong). 81.51% discard accuracy suggests significant room for improvement at the hardest decision. Not yet compared head-to-head with Suphx/LuckyJ/Mortal on riichi rules.

### Kanachan (Open Source, In Development)

**Architecture:** Transformer-based framework for riichi mahjong. Uses curriculum learning: trains mappings from easiest objectives to hardest, transferring the encoder between stages while replacing only the decoder. Goal is to exceed NAGA and Suphx performance.

**Status:** Active development on GitHub. Not yet benchmarked against top systems. Represents the emerging trend of transformer architectures for mahjong.

---

## Mahjong-Specific Techniques

### Shanten Counting

Shanten (向聴数, "hearing count") is the minimum number of tile changes needed to reach tenpai. It is the foundational metric for hand-building strategy. Computing shanten is non-trivial due to the combinatorial structure of melds:

- **Naive approach:** Enumerate all possible decompositions of the hand into partial melds, compute the minimum tiles needed to complete each. Exponential in hand size.
- **Fast algorithms:** Yao et al. (2021) proposed a knowledge-based octree search method that is typically 100x faster than baseline, correctly handles all edge cases including chiitoitsu and kokushi. Multiple open-source implementations exist (e.g., tomohxx/shanten-number on GitHub).
- **Incremental update:** Since only one tile changes per turn (draw then discard), shanten can be maintained incrementally rather than recomputed from scratch.

### Tile Efficiency (Hai Kouritsu / Ukeire)

Tile efficiency extends shanten by counting acceptance -- how many unseen tiles would improve the hand. For a hand at shanten N, tile efficiency measures the number of tiles that would reduce it to shanten N-1. Advanced efficiency considers:

- **Weighted acceptance:** Not just count but probability-weighted by remaining unseen copies.
- **Post-advance quality:** After drawing a tile that advances shanten, what is the resulting hand's tile acceptance? This two-level lookahead is standard in strong play.
- **EV-weighted efficiency:** Incorporating hand value. A discard that maximizes tile acceptance may produce a cheap hand, while a slightly less efficient discard preserves a path to mangan.

### Defensive Algorithms (Betaori)

Betaori (ベタオリ, "flat fold") is the strategy of abandoning one's hand to minimize deal-in probability. Formalization involves:

1. **Genbutsu (100% safe tiles):** Tiles present in a riichi declarer's discard river are provably safe against that player due to the furiten rule.
2. **Suji (筋) reasoning:** Based on the prevalence of ryanmen (two-sided) waits. If a player discarded 4m, then 1m and 7m are suji-safe against ryanmen waits involving 4m. Six suji lines per suit (1-4, 2-5, 3-6, 4-7, 5-8, 6-9), 18 total. Base deal-in probability for a tile against ryanmen is approximately 1/18 per suji line.
3. **Kabe (壁) analysis:** When all four copies of a tile are visible, certain waits become impossible. Three-copy kabe provides partial safety.
4. **Safety ranking:** Genbutsu > honor tiles (already visible copies) > kabe tiles > suji tiles > other tiles. The probability difference between safety ranks is approximately 50% per step.
5. **Probabilistic integration:** Modern AI systems integrate all defensive signals into a single deal-in probability estimate per tile rather than applying heuristic rules sequentially.

### Push/Pull Decision Framework (Oshi-Hiki)

The push/pull decision -- whether to continue building one's hand or switch to defense -- is the strategic crux of riichi mahjong. Key factors:

- **Attacker's hand value and wait quality.** Higher expected value and wider waits justify pushing.
- **Threat assessment.** Number of riichi declarations, suspected tenpai opponents, visible dangerous indicators.
- **Shanten distance.** Pushing from iishanten (2 away) against a declared riichi is much riskier than pushing from tenpai.
- **Point standings and game context.** Trailing in the last hand of south round changes the calculus dramatically.
- **AI vs. human behavior.** Research shows AI declares riichi ~21.65% of the time versus ~17.30% for humans, suggesting humans are suboptimally conservative in many riichi-eligible situations.

### Riichi Declaration Strategy

Modern theory holds that nearly all ryanmen tenpai with a closed hand should declare riichi, with exceptions for:
- Hands already at haneman+ value (riichi adds marginal value).
- Situations requiring defensive flexibility.
- Bad waits (kanchan, tanki) where the locked hand and revealed intent outweigh the 1-han gain.
- Late-game point situations where the 1,000-point bet is significant.

Riichi approximately doubles a hand's expected value for a modest reduction in win rate, making it generally +EV.

---

## Multi-Agent Challenges

### Four-Player Non-Zero-Sum Dynamics

Riichi Mahjong with placement bonuses (uma: typically +30/+10/-10/-30) is strictly non-zero-sum when viewed through the lens of final placements. This creates several phenomena that two-player game theory does not address:

1. **Implicit anti-leader coalitions.** When one player leads significantly, the other three have aligned incentives to prevent that player from winning hands, even at cost to their own hand-building. This emergent cooperation is not modeled by standard self-play RL.

2. **Asymmetric risk tolerance.** A player in 4th place late in the game needs to take high-variance gambles (pushing for expensive hands), while the 1st-place player benefits from conservative, low-variance play. Optimal play is fundamentally position-dependent.

3. **Dealer advantage exploitation.** The dealer (oya) benefits from renchan (repeat) mechanics, creating incentives for non-dealers to end the dealer's streak quickly, even with cheap hands.

4. **Placement boundary effects.** Points near a placement boundary (e.g., just above/below 2nd vs 3rd) create discontinuous incentive shifts. A 100-point difference can be worth 20,000+ points in uma.

### CFR Limitations in 4-Player Games

CFR converges to Nash equilibrium in two-player zero-sum games but loses all convergence guarantees in multiplayer settings. In four-player mahjong:

- The game tree complexity of two-player mahjong alone reaches 10^64 (vs. 10^18 for Limit Hold'em).
- Four-player mahjong is even larger, with the additional combinatorial explosion of three opponents' hidden states.
- LuckyJ's neural CFR approach works in practice but lacks theoretical backing for the four-player case.
- CFR-p (2023) explored CFR with hierarchical policy abstraction for two-player mahjong, but extending to four players remains an open problem.

### Opponent Modeling

Unlike poker where opponent modeling focuses on betting patterns, mahjong opponent modeling involves:

- **Discard reading (yomitsuki):** Inferring opponents' hand structure and tenpai wait patterns from their discard order, timing, and open melds.
- **Style detection:** Identifying aggressive vs. defensive opponents, riichi tendencies, calling frequencies.
- **Belief state maintenance:** Tracking probability distributions over opponents' hidden tiles given all observed information.

Current top AI systems (Suphx, LuckyJ, Mortal) primarily use self-play rather than explicit opponent modeling, potentially leaving performance on the table against exploitable opponents.

---

## Open Problems

### 1. Multi-Hand Game-Level Strategy

All current top systems optimize primarily at the hand level. While Suphx's global reward prediction begins to address this, true game-level strategy -- adjusting play across a hanchan based on evolving point standings, remaining hands, and placement targets -- remains underdeveloped. Human professionals excel at this meta-strategic layer.

### 2. Efficient Online Search

Suphx's run-time policy adaptation (pMCPA) operates only at the beginning of each round. True per-decision online search -- analogous to poker's real-time solving -- is largely absent from mahjong AI. The information set size (~10^48) makes full MCTS infeasible, but targeted search with learned value functions could improve critical decisions.

### 3. Compute Accessibility

Suphx required 44 GPUs for 2 days per agent. LuckyJ's requirements are undisclosed but presumably comparable. Recent work (Tjong, LsAc*-MJ, MJ_RM) attempts to reduce compute requirements, but achieving top-tier performance on consumer hardware remains an open challenge. Mortal's open-source approach is the closest to accessible, but training from scratch still requires significant resources.

### 4. Reward Engineering

The sparse reward problem -- most hands end in draws, and the connection between individual discard decisions and final outcomes is tenuous -- drives much of the difficulty. Fan backward (Tjong), reward variance reduction (RVR), and global reward prediction (Suphx) are partial solutions, but no consensus best practice exists.

### 5. Theoretical Foundations for 4-Player Games

No equilibrium concept fully captures optimal play in four-player mahjong. Nash equilibrium is computationally intractable and may not be the right solution concept due to correlated equilibria and implicit coalitions. This is a fundamental game theory problem, not just an engineering challenge.

### 6. Rule Set Fragmentation

Riichi mahjong has multiple rule sets (Tenhou, WRC, M-League, EMA) with subtle differences in red dora, uma/oka, yakuman stacking, and abortive draw conditions. AI systems trained on one rule set may perform suboptimally on another. A general-purpose riichi AI that handles rule set variation gracefully does not yet exist.

### 7. Interpretability and Human Coaching

Beyond Mxplainer's initial work, making mahjong AI decisions interpretable to human learners is largely unsolved. NAGA's game analysis service is commercially successful but provides limited insight into why the AI recommends a particular action. The gap between "what to do" and "why to do it" limits the coaching value.

### 8. Styled and Diverse Agents

The 2025 MPPO work shows that creating agents with distinct play styles (aggressive, defensive, balanced) is non-trivial. Standard RL converges to a single policy. Generating a population of diverse, high-quality agents -- valuable for both training robustness and human entertainment -- remains nascent.

---

## Relevance to Myosu

### Solver Architecture

Riichi Mahjong is a **tier-1 challenge** for Myosu's game-solving subnet, requiring a fundamentally different architecture from CFR-based poker solving:

| Approach | Viability | Notes |
|----------|-----------|-------|
| CFR / CFR+ | 1/5 | No convergence in 4-player games; information sets too large |
| Deep RL (policy gradient) | 5/5 | Proven by Suphx, LuckyJ, Mortal; the dominant paradigm |
| Neural CFR | 3/5 | LuckyJ shows promise but lacks theoretical grounding |
| MCTS + neural evaluation | 4/5 | Viable for search-enhanced play; underexplored for riichi |
| Supervised learning + RL | 4/5 | NAGA's approach; requires large expert datasets |
| Transformer-based RL | 4/5 | Tjong and Kanachan show promise; natural fit for sequential data |

**Recommended architecture for Myosu:** Deep RL with policy gradient as the backbone, incorporating:
- Oracle guiding (Suphx-style) for training efficiency.
- Fan backward or GRP-style reward shaping for the sparse reward problem.
- Hierarchical decision-making (Tjong-style) for tractable action spaces.
- Optional MCTS enhancement for critical decisions at inference time.
- Rust-based emulation (Mortal-style) for training throughput.

### Evaluation Framework

Tenhou's dan ranking system provides the gold-standard evaluation methodology:
- Stable rank over 1000+ games provides a reliable performance measure.
- Existing AI baselines (Suphx 10.0, LuckyJ 10.68) provide clear targets.
- Alternative: round-robin Elo-style tournaments with diverse opponents (human replays, existing AI agents, self-play variants).

### Engineering Requirements

1. **Scoring engine.** A complete and correct riichi mahjong scoring engine (yaku recognition, fu calculation, limit hand detection, payment distribution) is a substantial engineering prerequisite. The yaku system has numerous edge cases (double-counted winds, open vs. closed han values, chiitoitsu fu exception, pinfu tsumo exception).
2. **Game simulator.** High-throughput mahjong simulation (Mortal achieves 40K hanchans/hour in Rust) is essential for RL training.
3. **State encoding.** Multiple 34x1 channels representing tile distributions, discard histories, open melds, dora, scores, and round context. Suphx's encoding scheme is well-documented and serves as a solid baseline.
4. **Rule set specification.** Must commit to a canonical rule set early. Tenhou rules are the most benchmarked and have the richest AI comparison data.

### Market Opportunity

Riichi mahjong has the largest player base of any non-poker game in the Myosu survey:
- Tenhou: 350,000+ registered accounts.
- Mahjong Soul (Yostar): millions of players globally, growing rapidly.
- M-League (Japan): professional league with broadcast viewership rivaling esports.
- Growing Western competitive scene (WRC, EMA tournaments).
- Established commercial precedent: NAGA's paid analysis service demonstrates willingness to pay for AI tools.

### Compute Budget Estimate

Based on published training requirements:
- Suphx: ~44 GPUs x 2 days per agent x 5 agents = ~440 GPU-days total.
- Tjong: 2 GPUs x 7 days = 14 GPU-days (but weaker performance).
- Mortal: undisclosed, but designed for reasonable-scale training.
- Estimated Myosu target: 50--200 GPU-days on modern hardware (A100/H100) for a competitive agent, assuming architectural improvements over the 2020-era Suphx pipeline.

---

## Key Papers and References

| Year | Authors / System | Title / Description | Venue | Contribution |
|------|-----------------|---------------------|-------|-------------|
| 2015 | Mizukami & Tsuruoka | Building a Computer Mahjong Player Based on Monte Carlo Simulation | CIG 2015 | First systematic MC approach for mahjong AI |
| 2017 | Kaneko | Evaluation Functions for Mahjong | IPSJ | Feature-based supervised learning for tile evaluation |
| 2020 | Li et al. (Suphx) | Suphx: Mastering Mahjong with Deep Reinforcement Learning | Nature | Superhuman mahjong AI; oracle guiding, GRP, pMCPA |
| 2021 | Yao et al. | A Fast Algorithm for Computing the Deficiency Number of a Mahjong Hand | Chinese Journal of Electronics | 100x faster shanten computation via octree search |
| 2021 | Zhao et al. | Mahjong AI: Algorithmic Foundations | Survey | Comprehensive survey of mahjong AI approaches |
| 2022 | Zhao (Meowjong) | Building a 3-Player Mahjong AI using Deep RL | IEEE CoG 2022 | First sanma (3-player) AI with deep RL |
| 2022 | Equim-chan (Mortal) | Mortal: A Fast and Strong AI for Riichi Mahjong | GitHub / Open Source | Open-source deep RL agent; Rust emulator |
| 2023 | LuckyJ (Tencent) | Neural CFR with Game Tree Search for Mahjong | TechNode report | New Tenhou record: stable rank 10.68 dan |
| 2023 | CFR-p | CFR with Hierarchical Policy Abstraction for Two-Player Mahjong | arXiv:2307.12087 | CFR adapted to mahjong with abstraction |
| 2024 | Li et al. (Tjong) | Tjong: A Transformer-Based Mahjong AI via Hierarchical Decision-Making and Fan Backward | CAAI Trans. Intell. Tech. | Transformer architecture; fan backward reward shaping |
| 2024 | LsAc*-MJ | Low-Resource Consumption RL Model for Mahjong | Int. J. Intell. Sys. | LSTM + optimized A2C for data-scarce settings |
| 2024 | Mix-PPO | Imperfect Information Game of Four-Player Mahjong Based on Mix-PPO | IEEE Trans. Games | Novel PPO variant for 4-player Chinese mahjong |
| 2024 | IJCAI Mahjong Competition | Exploring AI Application in Complex Real-World Games | IJCAI 2024 Proceedings | Annual mahjong AI competition on Botzone |
| 2025 | MJ_RM | Efficient Mahjong Model with Distributed PPO and Res2Net-LSTM | Eng. Appl. AI (ScienceDirect) | Three-stage reward mechanism; resource-efficient training |
| 2025 | MPPO Styled Agents | Elevating Styled Mahjong Agents with Learning from Demonstration | arXiv | Creating diverse-style agents via LfD |
| 2025--2026 | Mxplainer | Explain and Learn Insights by Imitating Mahjong Agents | MDPI Algorithms | Interpretable approximation of black-box mahjong AI |

### Online Resources

- **Mortal GitHub:** https://github.com/Equim-chan/Mortal
- **Mortal Documentation:** https://mortal.ekyu.moe/
- **Kanachan GitHub:** https://github.com/Cryolite/kanachan
- **Botzone Platform:** https://botzone.org.cn/
- **NAGA Official:** https://dmv.nico/en/articles/mahjong_ai_naga/
- **Suphx (MSRA):** https://www.microsoft.com/en-us/research/project/suphx-mastering-mahjong-with-deep-reinforcement-learning/
- **Tenhou Dan Rankings:** https://riichi.wiki/Tenhou.net_ranking
- **IJCAI 2025 Mahjong AI Competition:** https://www.botzone.org.cn/static/gamecontest2025a.html

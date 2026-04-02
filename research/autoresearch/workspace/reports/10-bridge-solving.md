# Solving Strategies for Contract Bridge

**Date:** 2026-03-30
**Scope:** Comprehensive survey of solving methods, 2001--2026
**Status:** Research report (no implementation)

---

## Executive Summary

Contract Bridge is widely considered the most strategically complex card game and remains one of the last major games where AI has not conclusively surpassed top human players across all phases of play. Unlike poker, which is purely adversarial between individuals, bridge is a *cooperative-adversarial* game: two partnerships of two players each compete, with partners required to coordinate through a constrained communication channel (the bidding auction) while holding private hands. This dual nature -- cooperation within teams, competition between teams -- places bridge in a distinct and harder class of imperfect-information problems.

The game comprises two fundamentally different phases demanding separate algorithmic treatment. The *card play* phase (declarer play and defense) has seen the most progress: NukkAI's NooK system defeated eight world champions in 2022 in a play-only evaluation, and the Perfect Information Monte Carlo (PIMC) + double-dummy solver (DDS) pipeline has been the standard workhorse since Ginsberg's GIB (2001). The *bidding* phase remains the frontier: it is a cooperative signaling problem under adversarial interference where partners must jointly infer each other's hands through a vocabulary of ~38 legal calls. Recent work (2024) combining supervised pretraining, deep RL self-play, and belief Monte Carlo search has achieved +0.98 IMPs/deal over WBridge5, the strongest traditional bridge engine. A 2025 algorithm (LeadGenius) addresses the opening-lead problem specifically, outperforming WBridge5 and expert benchmarks via neural-guided priority sampling with PIMC.

Despite this progress, no AI system has demonstrated consistent superiority over elite human pairs in full bridge (bidding + play + defense) under tournament conditions. Bridge remains an open grand challenge for AI, with the partnership coordination problem, convention emergence, and defensive signaling as key unsolved dimensions.

---

## Game Complexity Analysis

### Two-Phase Structure

| Phase | Nature | Key Challenge | Information |
|-------|--------|---------------|-------------|
| **Bidding (Auction)** | Cooperative signaling + adversarial interference | Partners communicate hand shape/strength through ~38 legal calls; opponents can overcall, double, preempt | Each player sees only own 13 cards |
| **Card Play** | Sequential trick-taking | Declarer plans across 13 tricks; defenders coordinate without explicit communication | Declarer sees 26/52 cards (own + dummy); defenders see 13 + dummy |

### State Space Metrics

| Metric | Estimate | Notes |
|--------|----------|-------|
| Possible deals | ~5.36 x 10^28 | C(52,13) x C(39,13) x C(26,13) |
| Meaningful bidding auctions | ~10^5 -- 10^7 | Highly variable; constrained by monotonicity rules |
| Game tree nodes (bidding + play) | ~10^40+ | Comparable to chess in raw tree size |
| Information sets (play phase, per defender) | ~10^12 -- 10^15 | Reduced substantially by dummy exposure |
| Action space per bidding turn | Up to 38 | 35 bids + pass + double + redouble |
| Action space per play turn | 1--13 | Must-follow-suit constraint narrows choices |
| Double-dummy solutions per deal | 20 | 4 declarers x 5 strains |

### Partnership Dynamics: The Unique Challenge

Bridge is a **two-team zero-sum extensive-form game** where each team member holds private information and cannot communicate except through public game actions. This structure maps to the Team-Maxmin Equilibrium with Coordination (TMECor) solution concept in game theory. TMECor assumes teams can jointly correlate strategies before play (via convention agreements) but cannot communicate during play -- precisely matching bridge's tournament rules.

Computing TMECor is fundamentally harder than computing Nash equilibrium in two-player games because each player on a team has different information and must use public actions to signal to teammates. The joint action space is combinatorial and exponential in team size, and standard equilibrium-finding algorithms struggle to scale.

---

## Historical Progression

### Pre-AI Era: Rule-Based Systems (1960s--1990s)

Early bridge programs were entirely rule-based, encoding bidding conventions and play heuristics as production rules. Programs like Bridge Baron (1980s) used hierarchical task network (HTN) planning for declarer play, decomposing complex plays like finesses, squeezes, and endplays into primitive card-play actions. These systems played at a weak club level, primarily limited by rigid bidding and inability to reason about hidden hands.

### 2001: GIB -- Monte Carlo + Double-Dummy

Matthew Ginsberg's GIB (Ginsberg's Intelligent Bridgeplayer) introduced the paradigm that still dominates bridge AI. GIB won the World Computer Bridge Championship in 1998 and 1999. Its architecture involved five key technical contributions:

1. **Partition search**: A novel algorithm for restricting the game tree during play.
2. **Monte Carlo sampling**: Generate N random deals consistent with all observed information (bidding, cards played). For each sample, solve perfectly using a DDS, then aggregate results to select the best action.
3. **Achievable sets**: A correction for PIMC's strategy fusion problem -- rather than asking "which card wins the most tricks across all sampled worlds," GIB asks "which card achieves a result in the achievable set for each world."
4. **Alpha-beta on distributive lattices**: An extension of standard pruning beyond total orderings.
5. **Squeaky wheel optimization**: Approximate solutions for cardplay planning.

GIB used 50 Monte Carlo samples with 90 seconds per deal. It played at a strong amateur level but was notably weak in bidding and defensive signaling.

### 2001--2015: WBridge5 and Jack Dominate

**WBridge5** (Yves Costel, France) became the most decorated computer bridge program, winning the World Computer Bridge Championship 8 times (2005, 2007, 2008, 2012, 2014, 2016, 2017, 2018). Its architecture is not publicly documented in detail but uses a sophisticated hand-crafted bidding system with MC + DDS for card play. WBridge5's bidding is considered the strongest among traditional programs.

**Jack** (Hans Kuijf, Netherlands) won 10 of 12 finals from 2001 to 2012, claiming championships in 2001--2004, 2006, 2009, 2010, and 2012. Jack claims to contain "the fastest double dummy solver in the world" and features over 100 adjustable bidding conventions. Like WBridge5, its bidding system is hand-engineered.

Both programs play at an estimated strong club to weak expert level -- competent but below top professionals, with bidding quality being the primary differentiator.

### 2016--2019: Neural Networks Enter Bidding

Yeh and Lin (2016) published the first deep RL approach to bridge bidding, using policy gradient methods to learn bidding decisions from self-play. This demonstrated feasibility but under simplified conditions (restricted bids, passive opponents). Rong et al. (2019) extended this with a dual-network architecture: a policy network for bid selection and an estimation network to predict unseen hands, pretrained on human expert data and refined through RL self-play.

### 2020: Recursive Monte Carlo Search (NukkAI)

Bruno Bouzy, Alexis Rimbaud, and Veronique Ventos at NukkAI introduced Recursive Monte Carlo (RMC) search for bridge card play. Instead of using DDS as the terminal evaluator in Monte Carlo sampling, RMC uses a weaker Monte Carlo player as the evaluator, then recurses: a level-(N+1) RMC player uses level-N RMC as its simulator. This removes the dependency on domain-specific perfect solvers while improving play quality with additional compute. RMC outperformed standard PIMC + DDS at higher computation budgets.

### 2020: The Alpha-Mu Algorithm

Cazenave and Ventos introduced the alpha-mu (αμ) search algorithm, an anytime heuristic search for incomplete-information games that addresses two fundamental weaknesses of PIMC:

1. **Strategy fusion**: PIMC erroneously assumes it can choose different strategies in different sampled worlds, when in reality it must choose one action across all possible worlds consistent with its information.
2. **Non-locality**: Actions in PIMC are evaluated independently per sample, ignoring correlations across information sets.

αμ searches over information sets rather than individual game states, producing strategies that are valid under imperfect information. It was specifically designed for and tested on bridge.

### 2022: NukkAI Defeats World Champions

In March 2022, NukkAI's NooK system defeated eight world champion bridge players in a controlled match in Paris, winning 67 of 80 rounds (83%). This was a landmark achievement, but with critical caveats:

- **Play only**: Bidding was fixed by human experts. NooK played only the card-play phase (declarer play).
- **No defense**: NooK was always declarer, not defender.
- The event demonstrated superhuman card play but did not address bidding or defensive play.

NooK's architecture is a hybrid combining:
- **Probabilistic Inductive Logic Programming (PILP)**: Developed over 20 years by Stephen Muggleton at Imperial College London. Rules of the game are hard-coded; the system learns probabilistic decision rules that are interpretable.
- **Deep reinforcement learning**: For refining play strategies through self-play.
- **Monte Carlo simulation**: Constrained by game rules and human knowledge about signaling conventions.
- **Symbolic reasoning**: Enables "white-box" explainability -- NooK can articulate why it made each decision.

This neurosymbolic approach contrasts sharply with the pure deep learning methods used in poker AI, and NukkAI has emphasized explainability as a core design principle.

### 2023--2024: The Bidding Frontier

**Qiu et al. (2024)** -- *Bridge Bidding via Deep RL and Belief Monte Carlo Search* (IEEE/CAA JAS): The strongest published bidding result. The pipeline:
1. Supervised learning policy network pretrained on expert bidding data.
2. RL policy network initialized from supervised model, refined through self-play.
3. Value network trained to predict game outcomes (IMP scores).
4. Belief network trained to predict other players' hands from bidding sequences.
5. Belief Monte Carlo search at inference time: sample hands from the belief model, evaluate candidate bids using the value network, select the bid with highest expected value.

Achieved +0.98 IMPs/deal over WBridge5, surpassing the previous SOTA of +0.85 IMPs/deal.

**Kita et al. (2024)** -- *A Simple, Solid, and Reproducible Baseline for Bridge Bidding AI* (IEEE CoG 2024): Demonstrated that a straightforward combination of supervised pretraining + PPO + fictitious self-play can outperform WBridge5 without complex architectures. The neural network uses a 480-dimensional binary input vector and 4-layer MLP (1024 neurons per layer, ReLU activations) with separate policy and value heads. Code and models released as open source.

### 2025: LeadGenius -- Solving the Opening Lead

The opening lead is uniquely difficult: the defender must choose a card before dummy is exposed, with only their own hand and the bidding auction as information. LeadGenius (2025, published in *Information Sciences*) introduces a four-part algorithm:
1. **Hand probability model**: Neural network trained via supervised learning to predict opponent/partner hand distributions.
2. **Bidding information extractor**: Analyzes the auction to identify key constraints (e.g., opener has 5+ hearts, responder has 13+ HCP).
3. **Priority-based sampling with filtering**: Generates candidate hands biased toward likely holdings, filtered by extracted constraints.
4. **PIMC search**: Evaluates each candidate opening lead across sampled worlds using double-dummy solving.

LeadGenius outperforms WBridge5 and exceeds expert-level benchmarks on opening lead accuracy.

---

## Current Best Approaches

### 1. PIMC + Double-Dummy Solver (Card Play Standard)

**Algorithm**: Sample N random deals consistent with all known information -> solve each deal perfectly via DDS -> aggregate results (majority vote or expected score).

**Double-Dummy Solver (DDS)**: Bo Haglund's open-source DDS (v2.9.0, Apache 2.0 license) is the standard. It uses alpha-beta search with transposition tables and move ordering to solve any single bridge hand for a given declarer and strain in milliseconds. The `CalcDDtable` function computes optimal results for all 20 declarer/strain combinations in a single call. DDS is used by virtually every serious bridge AI program.

**Strengths**: Fast, exact per-sample, well-understood. Produces play quality that is strong for declarer play where the information gap is smaller (26/52 cards visible).

**Weaknesses**: Strategy fusion bias -- treats each sampled world independently, assuming different strategies can be used in each. Particularly harmful for defensive play where information asymmetry is larger. Sampling quality is critical: naive uniform sampling over consistent hands can produce unrealistic distributions.

**Compute**: 50--200 samples per decision, ~0.1--1 second per decision on modern hardware.

### 2. NooK Hybrid Architecture (Declarer Play SOTA)

**Architecture**: PILP (symbolic rules + probabilistic reasoning) + deep RL + constrained Monte Carlo simulation.

**Strengths**: Superhuman declarer play (demonstrated against world champions). Explainable decisions. Efficient -- does not require billions of training games like pure deep learning approaches.

**Weaknesses**: Demonstrated only for declarer play. Defense and bidding remain unaddressed in published results. PILP framework may not scale to the combinatorial complexity of convention learning. NukkAI has pivoted commercially toward defense/scheduling applications, and no follow-up publications on bridge bidding or defense have appeared as of early 2026.

**Compute**: Not publicly disclosed in detail; described as computationally efficient relative to pure DL approaches.

### 3. Deep RL + Belief Monte Carlo Search (Bidding SOTA)

**Architecture** (Qiu et al. 2024): Policy network + value network + belief network + search.

**Strengths**: Best published bidding performance (+0.98 IMPs/deal vs. WBridge5). Belief network enables principled handling of hidden information. Self-play training discovers conventions implicitly.

**Weaknesses**: Conventions learned through self-play are opaque and may not transfer to human partners. The +0.98 IMP advantage over WBridge5 is modest -- top human pairs likely hold a larger edge over WBridge5. The system was tested only against WBridge5, not against human experts. Partnership coordination is implicit rather than explicitly modeled.

**Compute**: Supervised pretraining on large expert databases + RL self-play (days to weeks of GPU time). Inference-time search adds ~1--10 seconds per bid decision.

### 4. Extended PIMC (EPIMC) -- Strategy Fusion Mitigation

**Algorithm** (2024): Modifies PIMC by postponing the point at which imperfect information is resolved to perfect information. Rather than immediately determinizing the game state, EPIMC maintains partial information sets deeper into the search tree, reducing strategy fusion artifacts.

**Strengths**: Principled improvement over PIMC without dramatic computational overhead. Demonstrated improvements in Dark Chess and Dark Hex; directly applicable to bridge defensive play.

**Weaknesses**: Still an approximation. Computational cost grows with the depth at which information is maintained. Not yet specifically evaluated on bridge in published literature.

---

## Bidding AI

### The Communication Problem

Bridge bidding is fundamentally a **cooperative communication problem under adversarial constraints**. Partners must jointly determine the optimal contract (denomination + level) using a vocabulary of ~38 legal calls per turn, where:

- Each bid must be higher than the previous bid (monotonicity constraint severely limits information bandwidth).
- Opponents can interfere by bidding, doubling, or preempting.
- The mapping from bids to hand information is governed by **conventions** -- pre-agreed codebooks like SAYC, 2/1 Game Forcing, Precision, etc.

### Convention Learning

Bidding systems in human bridge encode thousands of rules: Stayman, Jacoby Transfers, Blackwood, negative doubles, Bergen raises, splinter bids, etc. Each rule maps an auction context to a hand description. The fundamental question for AI is whether to:

1. **Hard-code a convention system** and optimize play within it (traditional approach).
2. **Learn conventions from expert data** via supervised learning (current baseline).
3. **Discover conventions through self-play** via multi-agent RL (emerging approach).
4. **Adapt to arbitrary convention systems** at inference time (unsolved).

Self-play convention learning faces a core problem: agents trained together develop idiosyncratic signaling that does not transfer to human partners or to agents trained in different runs. This is the **ad-hoc teamwork** problem -- can an agent coordinate with a novel partner it has never trained with? Current systems cannot do this reliably.

### Bidding Architecture Comparison

| System | Approach | Benchmark | Result |
|--------|----------|-----------|--------|
| WBridge5 | Hand-crafted rules | Human experts | Strong club level |
| Rong et al. (2019) | SL + RL, dual networks | WBridge5 | Competitive |
| Lockhart et al. (2020) | Search + policy iteration | WBridge5 | +0.85 IMPs/deal |
| Qiu et al. (2024) | SL + RL + belief MC search | WBridge5 | +0.98 IMPs/deal |
| Kita et al. (2024) | SL + PPO + FSP (baseline) | WBridge5 | Outperforms |
| AID-RL (2024) | GAI + alternate inference-decision RL | Various | Competitive |

---

## Card Play AI

### Declarer Play

Declarer play is the most tractable component: the declarer sees 26/52 cards (own hand + dummy) and controls both. Standard techniques:

1. **PIMC + DDS**: Sample opponent hands, solve double-dummy, play the card that performs best across samples. This handles finesses, endplays, and many squeeze positions implicitly (the DDS finds them in each sampled world).
2. **HTN Planning**: Bridge Baron used hierarchical planning to decompose complex plays into primitives (cash, finesse, elimination, endplay). Less flexible than MC + DDS but more interpretable.
3. **Neural evaluation**: Replace DDS with a neural network trick estimator for faster evaluation. Stanford CS229 projects (2016) explored training neural networks on double-dummy results; accuracy was reasonable but not competitive with DDS in computation time.

NooK's hybrid approach achieved superhuman declarer play by combining symbolic reasoning (understanding *why* a finesse works in a specific card layout) with probabilistic search.

### Defensive Play

Defensive play is substantially harder: each defender sees only 13 + 13 = 26 cards (own hand + dummy) and must coordinate with their unseen partner. Key challenges:

1. **Defensive signaling**: Partners communicate through card selection -- attitude signals (high = encouraging), count signals (high-low = even count), suit-preference signals. These signals are conventions agreed before play, analogous to bidding conventions but with a much noisier channel (the signal is a single card from a constrained set).
2. **Opening lead**: Made before dummy is exposed, based solely on the defender's hand and the auction. The highest-variance single decision in bridge. LeadGenius (2025) represents the first dedicated AI system for this problem.
3. **Defensive coordination**: The defenders are in a decentralized cooperative control problem -- each must infer partner's hand and intentions from card play while simultaneously signaling their own holdings.

Current defensive AI is the weakest component of all bridge programs. PIMC + DDS underperforms because the strategy fusion problem is most severe when the acting player has the least information.

---

## Partnership Modeling

### Game-Theoretic Framework

Bridge's team structure maps to the **adversarial team game** framework in game theory. The solution concept is TMECor (Team-Maxmin Equilibrium with Coordination):

- Teams can coordinate strategies before play (convention agreements) but not during play.
- Finding TMECor is equivalent to finding Nash equilibrium in a transformed two-player zero-sum game, but the transformation exponentially increases the game size.

### Algorithmic Approaches

**Multi-Player Transformation Algorithm (MPTA)** (2023): Converts adversarial team game trees into two-player zero-sum game trees with theoretical guarantees. Enables application of standard CFR-like algorithms but at exponential cost in team size.

**Team-PSRO** (NeurIPS 2023): Extends Policy-Space Response Oracles (PSRO) from two-player to team games. In each iteration, both teams learn a joint best response to the opponent's meta-strategy via cooperative RL. Converges to TMECor as the best-response quality improves. Tested on Kuhn poker and Liar's Dice (tabular version converges to TMECor) and Google Research Football (deep RL version beats self-play RL). Not yet applied to bridge at scale.

**PRR-TM** (2025): Perfect-recall refinement with teammate modeling. Approximates TMECor by iteratively refining player strategies while modeling teammate behavior. Represents the latest advance in practical team-game solving.

### The Ad-Hoc Teamwork Problem

In tournament bridge, a program must cooperate with partners it may not have trained with -- and against opponents whose conventions it does not know. This maps to **zero-shot coordination** in multi-agent AI. Current approaches fail at this: self-play RL produces conventions that are arbitrary encodings of hand information, uninterpretable to novel partners. Potential solutions include:

- **Convention-conditioned policies**: Train the agent on many different convention systems so it can adapt at inference time.
- **Theory of Mind models**: Maintain a belief over partner's conventions and update it from observed behavior.
- **Common knowledge priors**: Bias convention learning toward natural, human-interpretable signals (e.g., bidding your longest suit first is a nearly universal convention because it is natural).

---

## Open Problems

### 1. Full-Game Superhuman AI
No system has demonstrated superhuman performance across bidding + declarer play + defensive play. NooK achieved superhuman declarer play; bidding SOTA barely exceeds WBridge5 (which itself is below top human level). Defensive play AI remains primitive. Integrating all three phases into a coherent agent that handles information flow from bidding through play is unsolved.

### 2. Convention Emergence and Transfer
Can AI discover optimal bidding conventions through self-play? Can those conventions transfer to human partners? The self-play convention problem is a special case of emergent communication in multi-agent RL, where current methods produce efficient but opaque protocols. Bridge demands interpretable, transferable conventions.

### 3. Defensive Coordination
The decentralized cooperative control problem in defense has no satisfactory solution. Defenders must balance informing their partner (through signals) against revealing information to the declarer. This information-theoretic tradeoff has not been formally optimized in any existing system.

### 4. Robust Opponent Modeling
Current systems assume a fixed opponent model (typically WBridge5 or self-play). In tournament bridge, opponents vary dramatically in style and convention sophistication. Adaptive opponent modeling that updates beliefs about opponent conventions in real-time during an auction is largely unexplored.

### 5. Exploitability Measurement
There is no practical way to measure the exploitability of a bridge strategy. Unlike two-player poker where exploitability can be computed (in principle) as the value of a best response, bridge's team structure makes this computation intractable. This means we lack a ground-truth metric for measuring progress.

### 6. Abstraction for Bidding
The bidding action space is small (~38 actions) but the auction structure is deep and context-dependent. Unlike poker where card and action abstractions are well-studied, bridge bidding abstraction is largely unexplored. What is the right representation of bidding history for neural networks? The 480-dimensional binary vector used by current systems is ad-hoc.

### 7. Online Cheating Detection
Bridge has a significant online cheating problem (partners can communicate illicitly via phone). AI-based cheating detection systems like EDGAR exist but suffer from false positives. Using AI to establish what optimal play looks like -- and flagging deviations that correlate with illicit information -- is an active applied research area.

---

## Relevance to Myosu

### Solver Architecture Assessment

| Factor | Score | Rationale |
|--------|-------|-----------|
| CFR applicability | 2/5 | Standard CFR does not apply to team games. Team-PSRO or MPTA-based approaches are needed, adding complexity. |
| Neural value network potential | 4/5 | Strong for play evaluation; the DDS provides a perfect training oracle. Bidding value estimation is harder. |
| Abstraction necessity | 3/5 | Play phase is moderate size (DDS solves it directly). Bidding is the bottleneck but has small action space. |
| Real-time solving value | 5/5 | PIMC + DDS is the canonical real-time approach and works well for card play. |
| Transferability of techniques | 2/5 | Partnership mechanics are unique to bridge (and spades). Poker techniques do not directly transfer. |

### Subnet Architecture Recommendations

1. **Dual-phase evaluation**: Score bidding and play separately. Play quality can be measured against the double-dummy optimum (DD%). Bidding quality can be measured against expert databases or IMP differential vs. benchmark programs.

2. **DD% as ground truth for play**: For each deal, the double-dummy result provides the theoretical maximum trick count. A solver's play quality = (actual tricks) / (DD optimal tricks), averaged across deals. This is cheap to compute and objective.

3. **Duplicate format eliminates variance**: Bridge's duplicate format removes deal-distribution variance entirely. All solvers play the same deals; the only difference is decision quality. This is ideal for subnet evaluation.

4. **Convention declaration**: Following tournament bridge rules, solvers should declare their bidding system (convention card) in advance. This enables fair evaluation and prevents obfuscation.

5. **Separate bidding and play leaderboards**: Given the maturity gap between play AI (superhuman) and bidding AI (below expert), maintaining separate rankings prevents play-phase dominance from masking bidding weakness.

6. **Defense as a differentiator**: The weakest component of all current bridge AI is defensive play. A subnet that specifically evaluates defensive play quality would identify the most innovative solvers.

### Compute Profile

| Component | Time Budget | Resource |
|-----------|-------------|----------|
| DDS per deal (all 20 solutions) | ~10--100ms | Single CPU core |
| PIMC (100 samples) per play decision | ~1--10s | Single CPU core |
| Bidding neural network inference | ~10--100ms | GPU or CPU |
| Belief Monte Carlo search (bidding) | ~1--10s | GPU + CPU |
| Full deal (bidding + 13 tricks of play) | ~30--120s | Mixed |

Bridge is computationally light compared to NLHE. A full evaluation tournament of 100 deals can be completed in under an hour on modest hardware. This makes bridge an attractive subnet game for high-throughput evaluation.

---

## Key Papers and References

| Year | Authors | Title | Venue | Contribution |
|------|---------|-------|-------|--------------|
| 2001 | Ginsberg | GIB: Imperfect Information in a Computationally Challenging Game | JAIR | MC + DDS paradigm for bridge AI |
| 2006 | Haglund | Double Dummy Solver (DDS) | Open-source | Standard DDS used by all major bridge programs |
| 2010 | Bethe | The State of Automated Bridge Play | NYU Technical Report | Comprehensive survey of pre-DL bridge AI |
| 2016 | Yeh & Lin | Automatic Bridge Bidding Using Deep Reinforcement Learning | IEEE TETCI | First deep RL approach to bridge bidding |
| 2019 | Rong et al. | Competitive Bridge Bidding with Deep Neural Networks | arXiv | Dual-network bidding (policy + hand estimation) |
| 2020 | Bouzy, Rimbaud, Ventos | Recursive Monte Carlo Search for Bridge Card Play | IEEE CoG | RMC eliminates DDS dependency at cost of compute |
| 2020 | Cazenave & Ventos | The αμ Search Algorithm for the Game of Bridge | LNCS | Addresses strategy fusion and non-locality in PIMC |
| 2020 | Dafoe et al. | Open Problems in Cooperative AI | arXiv | Framework for cooperation problems including bridge |
| 2022 | NukkAI | NooK defeats 8 world champions | Challenge event | Superhuman declarer play via hybrid PILP + DL |
| 2023 | McAleer et al. | Team-PSRO for Learning Approximate TMECor | NeurIPS | Team-game equilibrium via cooperative RL |
| 2023 | Carminati et al. | Marriage between Adversarial Team Games and 2-Player Games | ICML | MPTA for team-to-2p game reduction |
| 2024 | Qiu, Wang, You, Zhou | Bridge Bidding via Deep RL and Belief MC Search | IEEE/CAA JAS | SOTA bidding: +0.98 IMPs/deal vs. WBridge5 |
| 2024 | Kita, Koyamada et al. | A Simple, Solid, Reproducible Baseline for Bridge Bidding AI | IEEE CoG | Open-source baseline; SL + PPO + FSP outperforms WBridge5 |
| 2024 | Arjonilla et al. | Perfect Information Monte Carlo with Postponing Reasoning (EPIMC) | ALA Workshop | Strategy fusion mitigation for imperfect-info games |
| 2024 | Zheng et al. | Alternate Inference-Decision RL with GAI for Bridge Bidding | Neural Comp. & Apps | Novel GAI + AID-RL approach |
| 2025 | (Authors TBD) | LeadGenius: Mastering Opening Leads in Bridge | Information Sciences | Neural-guided PIMC for opening lead; beats WBridge5 |
| 2025 | (Authors TBD) | PRR-TM: Finding Equilibria in Adversarial Team Games | IJMLC | Perfect-recall refinement with teammate modeling |

### Software and Resources

| Resource | URL | Notes |
|----------|-----|-------|
| DDS (Double Dummy Solver) | github.com/dds-bridge/dds | Apache 2.0, C++, v2.9.0 |
| endplay (Python bridge tools) | github.com/dominicprice/endplay | DDS bindings + deal generation |
| OpenSpiel (bridge environment) | github.com/google-deepmind/open_spiel | Bridge env in DeepMind's game framework |
| BridgeBase Online (BBO) | bridgebase.com | Largest online bridge platform; GIB engine |
| Funbridge / Argine | funbridge.com | Adaptive AI supporting multiple convention systems |
| Kita et al. baseline code | github.com/harukaki/brl | Open-source bridge bidding RL |
| WBridge5 | wbridge5.com | 8x World Computer Bridge Champion |
| Jack | jackbridge.com | 10x World Computer Bridge Champion (2001--2012) |

---

## World Computer Bridge Championship Winners

| Year | Winner | Runner-up |
|------|--------|-----------|
| 2001--2004 | Jack | Various |
| 2005 | WBridge5 | Jack |
| 2006 | Jack | WBridge5 |
| 2007--2008 | WBridge5 | Jack |
| 2009--2010 | Jack | WBridge5 |
| 2012 | Jack / WBridge5 | Various |
| 2014--2018 | WBridge5 | Various |
| 2023 (unofficial) | Q-Plus 15.3 | WBridge5 |

The championship has been dominated by two programs (Jack and WBridge5) for over two decades. Both use hand-crafted bidding systems with MC + DDS for card play. The persistence of rule-based approaches at the top of competitive computer bridge, long after deep learning transformed other game domains, underscores the unique difficulty of bridge's partnership coordination problem.

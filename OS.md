---
os_kind: "autonomous_company_os"
os_version: "0.3"
last_updated: "2026-03-16"
company_name: "Myosu"
company_stage: "stage_0_bootstrap"
domain_overlay: "game_solving_chain"
primary_mission_doctrine: "specs/031626-myosu-game-solving-chain.md"
hard_invariants_doctrine:
  - "INVARIANTS.md"
kpi_registry_path: "ops/kpi_registry.yaml"
scorecard_path: "ops/scorecard.md"
instrumentation_backlog_path: "ops/instrumentation_backlog.md"
risk_register_path: "ops/risk_register.md"
incident_ledger_path: "ops/incidents/"
decision_log_path: "ops/decision_log.md"
evidence_root: "ops/evidence/"
---

# Autonomous Company OS

This file is the live operating system for the myosu repo.

## Mission

**Build the world's first decentralized game-solving infrastructure.**

묘수 (myosu) — "brilliant move." A permissionless network where anyone can
contribute compute to solve games, anyone can verify the quality of solutions,
and anyone can play against the strongest strategies humanity has ever produced.

We are replacing a $2.5B proprietary solver oligopoly (PioSolver, MonkerSolver,
GTO+, PokerStove) with an open, competitive, token-incentivized market that
produces better strategies at lower cost and extends beyond poker to every
imperfect-information game worth solving.

## Why This Exists

### The problem

Game-Theory Optimal (GTO) strategies for poker and other imperfect-information
games are computed by solvers — software that runs millions of hours of
self-play using algorithms like Counterfactual Regret Minimization (MCCFR).
Today, this market has three structural problems:

1. **Monopoly pricing.** PioSolver charges $250-2,500/license. MonkerSolver
   charges per-variant. GTO+ charges per-feature. A serious poker student pays
   $500-5,000 for tools that cost pennies in compute.

2. **Centralized trust.** Users trust that the solver computed correctly, but
   have no way to verify. A compromised or biased solver could manipulate
   strategies for profit (imagine a solver that subtly favors hands its
   creator knows opponents hold).

3. **Single-game lock-in.** Each solver is built for one game. PioSolver does
   NLHE. MonkerSolver does PLO. No one does backgammon, mahjong, or bridge
   with the same rigor. The algorithms (CFR) generalize, but the products don't.

### The opportunity

```
  CURRENT MARKET                         MYOSU MARKET

  ┌─────────────────┐                   ┌──────────────────────────┐
  │  PioSolver      │                   │  SUBNET 1: NLHE HU       │
  │  $250-2500      │                   │  100 miners competing     │
  │  NLHE only      │                   │  verifiable quality       │
  │  trust-me       │                   │  free to query            │
  └─────────────────┘                   ├──────────────────────────┤
  ┌─────────────────┐                   │  SUBNET 2: NLHE 6-max    │
  │  MonkerSolver   │                   │  50 miners competing      │
  │  €250+          │                   ├──────────────────────────┤
  │  PLO only       │                   │  SUBNET 3: PLO            │
  └─────────────────┘                   │  30 miners competing      │
  ┌─────────────────┐                   ├──────────────────────────┤
  │  Nothing        │                   │  SUBNET 4: Backgammon     │
  │  for backgammon  │                   │  SUBNET 5: Mahjong        │
  │  mahjong, bridge │                   │  SUBNET 6: Bridge         │
  └─────────────────┘                   └──────────────────────────┘
```

Myosu turns game-solving from a product into a protocol. Miners earn tokens
by producing high-quality strategies. Validators earn tokens by honestly
scoring those strategies. Players get to play against — or learn from — the
strongest bots in existence.

### The moat

1. **Network effects.** More miners → better strategies → more players → more
   revenue (gameplay fees, coaching subscriptions) → more emission value → more
   miners. This is a flywheel, not a linear business.

2. **Verifiable quality.** Exploitability is a deterministic, objective metric.
   A solver that claims to be "GTO" can be verified on-chain. No trust required.

3. **Multi-game generalization.** The same infrastructure (chain, Yuma Consensus,
   miner/validator protocol) works for ANY imperfect-information game. Adding a
   new game means implementing one trait (`CfrGame`) — no chain changes, no
   protocol changes, no new infrastructure.

4. **Compute moat.** The more MCCFR iterations a miner runs, the closer to Nash
   equilibrium. Catching up to a miner that has been running for months requires
   months of compute. Early miners have a structural advantage.

5. **Open competition.** Unlike proprietary solvers, anyone can enter. This means
   the solver market is maximally efficient — no rent extraction, no artificial
   scarcity, no vendor lock-in.

## What We Are Building

```
  ┌─────────────────────────────────────────────────────────────────┐
  │                     MYOSU CHAIN (Substrate)                     │
  │                                                                 │
  │  ┌───────────────────────────────────────────────────────────┐ │
  │  │  pallet_game_solver                                       │ │
  │  │                                                           │ │
  │  │  Subnet Registry ──► Neuron Registry ──► Weight Storage   │ │
  │  │       │                    │                    │          │ │
  │  │       ▼                    ▼                    ▼          │ │
  │  │  Emission    ◄──── Yuma Consensus ◄──── Staking           │ │
  │  │  Distribution        (per tempo)         (voting power)   │ │
  │  └───────────────────────────────────────────────────────────┘ │
  └──────────────────────┬──────────────────┬─────────────────────┘
                         │                  │
              ┌──────────▼──────┐  ┌────────▼────────┐
              │    MINERS        │  │   VALIDATORS     │
              │                  │  │                  │
              │  ┌────────────┐ │  │  ┌────────────┐ │
              │  │ robopoker  │ │  │  │ exploit    │ │
              │  │ MCCFR      │ │  │  │ oracle     │ │
              │  │ trainer    │ │  │  │            │ │
              │  └──────┬─────┘ │  │  └──────┬─────┘ │
              │         │       │  │         │       │
              │  ┌──────▼─────┐ │  │         │       │
              │  │ HTTP axon  │◄┼──┼─────────┘       │
              │  │ /strategy  │ │  │  submit_weights │
              │  └────────────┘ │  │  to chain       │
              └─────────────────┘  └─────────────────┘
                       │
                       │ best miner's strategy
                       ▼
              ┌─────────────────┐
              │   GAMEPLAY       │
              │                  │
              │  human vs bot    │
              │  CLI / web       │
              │  coaching        │
              │  tournaments     │
              └─────────────────┘
```

### Layer 1: Chain (on-chain coordination)

A Substrate blockchain forked from Bittensor's subtensor. Stripped of AI-specific
mechanics, rebuilt for game-solving. The chain knows nothing about poker or
backgammon — it only knows subnets, neurons, weights, and emissions.

### Layer 2: Solvers (off-chain compute)

Miners run MCCFR training on their hardware. They serve trained strategies via
HTTP endpoints. The longer they train, the closer to Nash equilibrium, the
higher they score. Robopoker v1.0.0 is the first solver engine; others can be
built by anyone.

### Layer 3: Validators (off-chain quality control)

Validators query miners, measure exploitability (distance from Nash equilibrium),
and submit scores to the chain. Yuma Consensus aggregates validator scores with
stake-weighted median clipping, preventing collusion.

### Layer 4: Gameplay (consumer product)

Humans play against the best miner's strategy. This is the product that makes
the solver market matter. A solver no one plays against is academic. A solver
humans lose to is valuable.

## Revenue Model (Planned)

Stage 0-1 (bootstrap): Token emissions fund miners and validators. No revenue.

Stage 2-3 (growth):
- **Gameplay fees.** Players pay per-session to play against bots.
- **Coaching subscriptions.** Players pay for strategy analysis — "the bot would
  have played X here because Y."
- **Tournament entry.** Human vs bot tournaments with prize pools.
- **Strategy marketplace.** Subnets sell specialized strategies (e.g., "tournament
  ICM-adjusted" or "exploitative vs recreational players").
- **API access.** Third-party platforms pay for real-time strategy queries.

All revenue flows back to the chain as emission funding, replacing inflation
with sustainable economics.

## Target Games (20 games, by priority)

The selection criteria, in order: CFR fit (imperfect information?) → solver gap
(demand exists but no tool?) → market size (players × willingness to pay) →
cultural narrative (does it tell a story for the brand?) → tractability (can
MCCFR converge in reasonable time?).

```
  CIRCLE 1: POKER               CIRCLE 2: ASIAN CARD GAMES     CIRCLE 3: STRATEGY GAMES
  (capture existing $6B          (expand into $5B+ Asian        (prove the platform works
   solver market)                 gambling markets)              for any imperfect-info game)

  ┌──────────────────────┐      ┌──────────────────────┐       ┌──────────────────────┐
  │ 1. NLHE Heads-Up     │      │ 6. Teen Patti        │       │ 12. Stratego          │
  │ 2. NLHE 6-max        │      │ 8. Hwatu / Go-Stop   │       │ 13. OFC Chinese Poker │
  │ 3. PLO               │      │ 9. Mahjong (Riichi)  │       │ 14. Spades            │
  │ 4. NLHE Tournament   │      │ 11. Gin Rummy        │       │ 15. Liar's Dice       │
  │ 5. Short Deck        │      │ 16. Dou Di Zhu       │       │ 20. Backgammon        │
  │                      │      │ 17. Pusoy Dos        │       └──────────────────────┘
  └──────────────────────┘      │ 18. Tien Len         │
                                │ 19. Call Break       │
                                │ 10. Bridge           │
                                │ 7. Hanafuda          │
                                └──────────────────────┘
```

### Prioritized game list

| # | Game | Market | Solver Gap | CFR Fit | Geography | Stage |
|---|------|--------|-----------|---------|-----------|-------|
| **1** | **NLHE Heads-Up** | $6B+ poker market | PioSolver $250+ | Perfect | Global | 0 |
| **2** | **NLHE 6-max** | Most-played online poker format | PioSolver, expensive | Perfect | Global | 1 |
| **3** | **PLO (4-card Omaha)** | $400M-1.2B; MonkerSolver's entire business | Massive — postflop PLO is unsolved | Perfect | Global | 1 |
| **4** | **NLHE Tournament (ICM)** | Most online poker is tournaments | No ICM-aware CFR solver exists | Perfect | Global | 1 |
| **5** | **Short Deck (6+)** | Niche but 30% smaller state space | Limited solver support | Perfect | Asia | 1 |
| **6** | **Teen Patti** | India $2-3B projected by 2028 | No solver exists | Strong (3-card = fast convergence) | India | 2 |
| **7** | **Hanafuda (Koi-Koi)** | Cultural — Nintendo was founded as hanafuda co. | No solver exists | Moderate | Japan | 2 |
| **8** | **Hwatu / Go-Stop** | Part of Korea's $9-10B gambling market | No solver exists | Moderate | Korea | 2 |
| **9** | **Mahjong (Riichi)** | $1.5-2.1B market; 10% CAGR | Zero consumer solvers despite superhuman AI | Good (4-player caveat) | Japan, China | 2 |
| **10** | **Bridge** | 200M+ players worldwide; affluent demographic | No comprehensive solver (bidding unsolved) | Good (partnership) | Global | 2 |
| **11** | **Gin Rummy** | $1.5B Indian rummy market alone (RummyCircle 50M+ users) | No public solver | Good (state space challenge) | India, Global | 2 |
| **12** | **Stratego** | Niche but passionate; DeepNash proved feasibility | DeepNash research-only, not deployed | Moderate (huge state space) | Netherlands, Global | 3 |
| **13** | **OFC Chinese Poker** | High-stakes poker niche; solvers historically $30K+ | Moderate gap, becoming accessible | Moderate (MCTS may be better) | Russia/CIS, Asia | 3 |
| **14** | **Spades** | Large casual US base; millions daily | No solver exists | Moderate (4-player) | US | 3 |
| **15** | **Liar's Dice (full)** | Casual; architecture proof at scale | Academic only | Perfect | Global | 3 |
| **16** | **Dou Di Zhu (斗地主)** | China's #1 card game; Tencent version 600M+ users | No solver exists | Good (3-player asymmetric 2v1) | China | 3 |
| **17** | **Pusoy Dos / Big Two** | Dominant card game in Philippines, HK, Taiwan, Singapore | No solver exists | Good (shedding game) | SE Asia | 3 |
| **18** | **Tien Len (Thirteen)** | Vietnam's national card game; fast-growing online market | No solver exists | Good (similar to Big Two) | Vietnam | 3 |
| **19** | **Call Break** | South Asia's most popular trick-taking game; 10M+ app downloads each | No solver exists | Good (simple, fast convergence) | Nepal, India, Bangladesh | 3 |
| **20** | **Backgammon** | Middle East/Mediterranean; gambling tradition | Essentially solved (XG, GNU BG are superhuman since 1990s) | Mixed (perfect info + doubling cube) | Turkey, Israel, Global | 3 |

### Why this order

**Games 1-5 (Poker):** Capture the existing solver market. Every serious poker
player has either paid for PioSolver or wishes they could afford it. PLO at #3
is the single highest-value solver gap — MonkerSolver's entire business is PLO
solving, and postflop PLO is genuinely unsolved. Tournament/ICM at #4 is a huge
untapped market — no ICM-aware CFR solver exists.

**Games 6-11 (Asian card games):** This is where the 10x opportunity lives.
Teen Patti (#6) is India's massive gambling game with zero solver tooling.
Hwatu (#8) ties to myosu's Korean identity. Mahjong (#9) has a $1.5-2.1B market
with superhuman AI existing in research but zero consumer products. Bridge (#10)
has 200M+ players and no comprehensive solver. Gin Rummy (#11) taps the $1.5B
Indian rummy market.

**Games 7 + 8 (Hanafuda + Hwatu):** The same card deck, different games,
different countries (Japan + Korea). This pairing makes the multi-game story
feel curated. Nintendo was founded in 1889 as a hanafuda company — there's a
narrative thread connecting traditional Japanese gaming to a decentralized
game-solving protocol named in Korean.

**Games 12-20 (Platform proof):** Each game validates a different aspect of the
architecture. Stratego (#12) proves CFR works for massive state spaces.
Dou Di Zhu (#16) proves 3-player asymmetric games work. Backgammon (#20) is
deliberately last because it's essentially solved — existing tools are cheap
and superhuman. The solver gap is minimal.

### Geographic coverage

```
  CHINA           KOREA        JAPAN         INDIA          SE ASIA        WEST
  ┌──────────┐   ┌────────┐   ┌─────────┐   ┌──────────┐   ┌──────────┐   ┌──────────┐
  │Dou Di Zhu│   │Hwatu   │   │Mahjong  │   │Teen Patti│   │Pusoy Dos │   │Bridge    │
  │(600M+    │   │(Go-Stop│   │(Riichi) │   │(500M+   │   │(Big Two) │   │Spades    │
  │ users)   │   │  )     │   │Hanafuda │   │ Diwali)  │   │Tien Len  │   │Stratego  │
  │          │   │        │   │(Koi-Koi)│   │Call Break│   │(Vietnam) │   │Liar's    │
  │          │   │        │   │         │   │Gin Rummy │   │          │   │ Dice     │
  └──────────┘   └────────┘   └─────────┘   └──────────┘   └──────────┘   │Backgammon│
                                                                           │OFC       │
                 ◄─── 6 POKER VARIANTS SPAN ALL GEOGRAPHIES ──►            └──────────┘
  NLHE HU, NLHE 6max, PLO, NLHE Tournament, Short Deck, OFC
```

Combined addressable market across all 20 games: $10B+ annually. No competitor
addresses more than 2 of these games. myosu addresses all 20 with one protocol.

## Presentation Layer — Making Text Beautiful

The terminal is not a limitation. It is a medium.

Photography gained its identity when artists stopped trying to imitate painting
and embraced what the camera could do that paint could not. Terminal interfaces
gain their identity when designers stop trying to imitate GUIs and embrace what
text can do that pixels cannot: density, composability, universality, and the
radical equality of human and machine as players.

### The core insight: agents are first-class players

Every game on myosu must be playable by both humans and agents through the
same interface. This is not a technical constraint — it is the product thesis.
When an agent sits at a poker table, it sees the same text representation a
human sees and responds with the same commands. The interface IS the protocol.

This means:
- No mouse-dependent interactions. Everything is keyboard-native.
- No visual information that can't be expressed as text.
- Input is natural language or structured commands — `call`, `raise 15`,
  `discard 3m`, `bid two fives`, `challenge`.
- The game's text representation must be complete enough that an LLM can
  play from the text alone, with zero additional context.

If an LLM can read the game state and produce a valid action from text, the
interface is correct. If it can't, the interface is broken.

### Design principles

**1. The conversation metaphor.**

The game interface is a conversation between the player and the game engine.
Not a dashboard. Not a simulation. A conversation.

```
  ── Hand #47 · NLHE Heads-Up · Pot 12 BB ─────────────────────

  Board:  T♠  7♥  2♣

  You (BB):   A♠ K♥     94 BB
  Bot (SB):   ·· ··     94 BB

  Bot raises to 6 BB.

  > call

  You call 6 BB. Pot is now 12 BB.

  ── Turn: 9♦ ─────────────────────────────────────────────────

  Bot checks.

  > raise 8

  You raise to 8 BB.
```

The player types. The game responds. Like a text adventure, but for strategy
games. The aesthetic is closer to a literary experience than a video game.

**2. Information density as beauty.**

A terminal can display more decision-relevant information per square inch than
any GUI. A well-designed TUI shows everything the player needs — hand, board,
pot, stacks, action history, odds — in 20 lines. No scrolling. No tabs. No
hidden panels.

```
  ┌─ Riichi Mahjong · East 1 · Round 3 ──────────────────────┐
  │                                                            │
  │  Player 2:  ░░░░░░░░░░░░░  discards: 1m 9p 5s Nw        │
  │  Player 3:  ░░░░░░░░░░░    discards: 2m 3p 7s            │
  │  Player 4:  ░░░░░░░░░░░░░  discards: 4p                  │
  │                                                            │
  │  Your hand:                                                │
  │  [1m][2m][3m] [5p][6p][7p] [3s][4s] [9s][9s] [E][E]     │
  │                                                            │
  │  Draw: [5s]   Riichi: no   Points: 25000                  │
  │                                                            │
  │  > discard 4s                                              │
  └────────────────────────────────────────────────────────────┘
```

**3. Color as meaning, never decoration.**

The terminal palette is limited. This is a feature. Every color must carry
semantic meaning:

| Color | Meaning |
|-------|---------|
| White | Neutral text, game narration |
| Green | Your actions, positive outcomes, money won |
| Red | Opponent actions, negative outcomes, money lost |
| Yellow | Warnings, time pressure, important state changes |
| Blue | System information, pot size, stack sizes |
| Dim/Gray | Historical actions, folded players, metadata |
| Bold | Current decision point, your cards, action required |

No gradients. No background colors on text. No blinking. Restraint is the
aesthetic.

**4. Unicode as design system.**

Box-drawing characters, card suits, and mathematical symbols are the only
visual elements. They must be used consistently:

```
  Cards:    A♠ K♥ Q♦ J♣ T♠ 9♥        (rank + suit symbol)
  Hidden:   ·· ··                      (middle dot pairs)
  Tiles:    [1m] [2p] [3s] [Ew]       (bracketed shorthand)
  Dice:     ⚀ ⚁ ⚂ ⚃ ⚄ ⚅             (unicode dice faces)
  Borders:  ─ │ ┌ ┐ └ ┘ ├ ┤ ┬ ┴ ┼   (single-line box drawing)
  Dividers: ── section ──              (em-dash runs)
```

No ASCII art for cards. No elaborate box borders. The characters themselves
are the design.

**5. Silence as rhythm.**

Not every action needs a response. Blank lines create rhythm. The game
breathes between streets, between hands, between sessions.

```
  Bot folds.
  You win 6 BB.

                                               Session: +14 BB over 47 hands


  ── Hand #48 ─────────────────────────────────────────────────

  ...
```

The whitespace is intentional. It gives the player time to process.

**6. Free-form input with intelligent parsing.**

Players should be able to type naturally:

```
  > fold                    → Fold
  > call                    → Call
  > raise 15                → Raise to 15 BB
  > raise to 15             → Raise to 15 BB
  > r 15                    → Raise to 15 BB
  > all in                  → Shove
  > shove                   → Shove
  > ?                       → Show available actions
  > help                    → Show help
  > history                 → Show hand history
  > what should I do?       → Trigger coaching mode (query best miner)
  > analyze                 → Show GTO analysis of current spot
```

Invalid input is never an error — it's a prompt for clarification:

```
  > raise
  Raise to how much? (min 4 BB, max 94 BB)
  > 15
  You raise to 15 BB.
```

**7. The editor metaphor.**

For advanced users, the interface should feel like a code editor:

- Readline keybindings (Ctrl-A, Ctrl-E, Ctrl-W, etc.)
- Command history (up/down arrows)
- Tab completion for actions
- `/` prefix for meta-commands (`/quit`, `/stats`, `/settings`)
- Configurable prompt character

The terminal is the IDE. The game is the program. The player is the programmer.

### Agent integration

When an agent plays, the interface is identical. The agent receives the same
text output a human would see and produces the same text commands:

```
  Agent input (what the agent sees):
  ── Hand #47 · NLHE Heads-Up · Pot 12 BB ─
  Board: T♠ 7♥ 2♣
  You (BB): A♠ K♥  94 BB
  Bot (SB): ·· ··  94 BB
  Bot raises to 6 BB.
  >

  Agent output (what the agent types):
  call
```

This means:
- Game state rendering MUST be parseable by an LLM
- Action format MUST be unambiguous from text alone
- No hidden state that only a GUI could convey
- The same game client binary works for human play, agent play, and
  agent-vs-agent evaluation

An agent can join any table by piping stdin/stdout. A human can watch an
agent play in real-time by reading its terminal output. Two agents can play
each other with zero additional infrastructure — just pipe one's stdout to
the other's stdin.

### Implementation: ratatui + crossterm

The TUI framework is ratatui (Rust) with crossterm for terminal manipulation.
This is already battle-tested in the Rust ecosystem (lazygit, bottom, gitui).

The key architectural decision: **immediate-mode rendering with a conversation
log**. The screen is divided into:

```
  ┌─ Game State ──────────────────────────────────────────────┐
  │                                                            │
  │  [compact game state: board, hands, stacks, pot]          │
  │                                                            │
  ├─ Conversation ────────────────────────────────────────────┤
  │                                                            │
  │  [scrollable log of actions and narration]                │
  │  [most recent at bottom]                                  │
  │                                                            │
  ├─ Input ───────────────────────────────────────────────────┤
  │  > _                                                       │
  └────────────────────────────────────────────────────────────┘
```

The game state panel is fixed-height and always visible. The conversation
panel scrolls. The input line is always at the bottom. This layout works for
every game — poker, mahjong, backgammon, Liar's Dice, bridge, Teen Patti.
Only the game state panel changes per game.

### The test of beauty

A myosu game interface is beautiful when:
1. A screenshot is readable at 50% zoom
2. A player can play a complete hand without touching the mouse
3. An LLM can play from the text representation alone
4. A spectator can understand the game state from the conversation log
5. The interface looks intentional, not accidental — every character is placed
   with purpose

The terminal aesthetic is not retro nostalgia. It is the most direct path
between a game's information and a player's mind. No chrome. No animation.
No distraction. Just the game.

## Company Stages

### Stage 0: Bootstrap (current)
**Objective:** Prove the vertical slice works end-to-end on devnet.

Exit criteria:
- Chain produces blocks with game-solving pallet
- One poker subnet with miners and validators
- Human plays one hand against trained bot
- Liar's Dice validates multi-game architecture

### Stage 1: Launch
**Objective:** Mainnet with multiple poker subnets and real token economics.

Exit criteria:
- Mainnet launch with genesis validators
- 3+ poker variant subnets running
- 10+ miners competing per subnet
- Token economics sustain miner incentives without inflation death spiral
- Web-based gameplay interface live

### Stage 2: Platform
**Objective:** Expand beyond poker. Third-party game engines.

Exit criteria:
- 2+ non-poker game subnets (backgammon + mahjong)
- Game Engine SDK published for third-party developers
- Strategy marketplace operational
- Revenue from gameplay fees exceeds emission costs

### Stage 3: Ecosystem
**Objective:** Self-sustaining economy. Global game-solving infrastructure.

Exit criteria:
- 10+ game subnets across 5+ game categories
- Thousands of miners globally
- Coaching and tournament products generating revenue
- Third-party applications building on the protocol
- Bridge to existing gaming platforms

## Rule 0

If the repo lacks trustworthy operating truth for a critical decision, the
first job is to install that truth before adding scope.

1. challenge the requirement
2. delete unnecessary work before automating it
3. simplify the process before scaling it
4. shorten the path from signal -> decision -> action

Operational meaning:
- if a metric does not drive a decision, remove it
- if a review loop does not change priorities, remove it
- if a process can be cut, cut it before instrumenting it
- if a report exists without a default action, it is noise

## Doctrine Hierarchy

1. Mission doctrine — `specs/`, `ralph/SPEC.md`
2. Hard invariants — `INVARIANTS.md`
3. Company operating system — `OS.md`
4. KPI registry and scorecard — `ops/kpi_registry.yaml`, `ops/scorecard.md`
5. Reference context — `ops/risk_register.md`, `ops/decision_log.md`
6. Runtime truth — `state/`

## North Star

| field | value |
|---|---|
| metric | `solver_exploitability_convergence` |
| definition | best miner's strategy exploitability in milli-big-blinds/hand, approaching zero as the network matures |
| formula | `min(exploitability_scores) across active miners per subnet` |
| source | validator consensus on exploitability measurement |
| cadence | per-tempo evaluation |
| why it matters | a game-solving chain is only valuable if it produces strategies that approach Nash equilibrium |

Leading indicator:

| metric | formula | why |
|---|---|---|
| `active_miners_per_subnet` | count of miners that served queries in last epoch | no miners = no strategies = chain is dead |
| `validator_agreement_rate` | % of validator pairs with <10% score divergence on same miner | disagreement means scoring is broken (INV-003) |
| `gameplay_sessions_per_day` | count of human vs bot sessions completed | no players = no product = no revenue path |

## Guardrails

| metric | green | yellow | red | default_action |
|---|---|---|---|---|
| `false_green_proof_count` | 0 | 1 | >1 | stop completion claims, repair proof honesty |
| `validator_determinism_divergence` | <1e-6 | 1e-6 to 1e-3 | >1e-3 | freeze emissions, investigate INV-003 |
| `solver_gameplay_crate_separation` | no cross-dep | — | any direct dep | revert, refactor into engine crate |
| `emission_accounting_balance` | sum distributions == block emission | — | any imbalance | halt emissions, audit |

## Active Functions

| Function | Mandate | Primary outputs |
|---|---|---|
| CEO / Strategy | keep the repo aimed at chain launch and solver quality; decide stage transitions; guard against scope creep | priorities, stage call, bottleneck decisions |
| Security / Risk | guard chain consensus, solver verification, gameplay fairness, token economics safety | audit roadmap, risk register, no-ship escalation |
| Execution / Dev | convert priorities into landed, verified code without lying about completion | delivery proof, closure truth, completion and blockage quality |
| Product | shape the gameplay experience and solver marketplace around real player needs | UX decisions, game variant prioritization, revenue model validation |

Functions intentionally dormant until Stage 1:
- Growth / Marketing
- Revenue / Commercial
- Finance
- Support / CX

## Reference Downstream Lanes

- `codexpoker` — existing poker platform that will consume myosu solver output
- game communities — poker, backgammon, mahjong, bridge players
- third-party developers — future game engine SDK consumers

## Competitive Landscape

| Competitor | Strengths | Weaknesses | Our advantage |
|---|---|---|---|
| PioSolver | Market leader, fast, trusted | Expensive ($249-2500), NLHE only, closed | Open, multi-game, verifiable |
| MonkerSolver | PLO support, multi-way | Complex, expensive, slow | Better UX, cheaper, multi-game |
| GTO+ | Affordable ($75), good UX | Limited features, NLHE only | Full GTO, multi-game platform |
| PokerStove | Free, open source | Equity calculator only, not a solver | Full MCCFR solver, not just equity |
| Bittensor (SN) | Proven subnet infra, TAO economics | AI-focused, no game support | Purpose-built for games, verifiable quality |

No one is building decentralized game-solving infrastructure. The closest
competitor is running a Bittensor subnet, but that costs $1-2M in locked TAO
and gives us no control over chain parameters or game-specific economics.

## Technical Moat (Why Forks Are Hard)

Forking myosu's chain is trivial. Forking the solver ecosystem is hard:

1. **Abstraction tables.** NLHE clustering (13M isomorphisms → 500 abstract
   buckets) takes days of compute. A fork starts with empty abstraction tables
   and must recompute them.

2. **Trained strategies.** Miners with months of MCCFR training have strategies
   that converge toward Nash. A fork starts with random strategies.

3. **Validator reputation.** Validators build bond EMA over time. A fork starts
   with no trust history.

4. **Game engine ecosystem.** Each new game engine (backgammon, mahjong) adds
   to the platform's value. A fork must reimplement all game engines.

The chain is commodity infrastructure. The network of trained solvers is the moat.

## Bootstrap Exit Criteria

Myosu remains in stage 0 until ALL of the following are true:
- Substrate chain compiles and produces blocks on local devnet
- Game-solving pallet integrated at index 7 with Yuma Consensus
- At least one poker subnet registers and runs solver evaluation
- One miner produces a strategy profile from robopoker MCCFR
- One validator computes exploitability and submits weights
- Yuma Consensus distributes emissions proportional to quality
- One human can play a hand of poker against the trained bot
- Liar's Dice validates multi-game architecture (zero existing code changes)
- All 6 invariants pass (INV-001 through INV-006)

## Current Priority Order

1. **Fork robopoker** — add serde feature + NlheEncoder constructor (RF-01, RF-02)
2. **Fork subtensor** — strip to minimal chain, get blocks producing (CF-01..05)
3. **Game engine traits** — re-export robopoker CFR traits with wire serialization (GT-01..05)
4. **Poker engine** — wrap NlheSolver, checkpoint, query, exploitability (PE-01..04)
5. **Game-solving pallet** — subnet registry, neurons, weights, Yuma, emission (GS-01..10)
6. **Miner binary** — register, train, serve axon, checkpoint (MN-01..05)
7. **Validator binary** — register, stake, query, score, submit weights (VO-01..06)
8. **Gameplay CLI** — human vs bot poker (GP-01..04)
9. **Multi-game proof** — Liar's Dice validates architecture (MG-01..04)

## Severity and Response Classes

| Severity | Meaning | Default response |
|---|---|---|
| `S0` | Chain consensus compromised, token accounting broken, or exploitability scoring non-deterministic | freeze emissions, preserve evidence, halt all non-critical work |
| `S1` | Critical capability broken but chain consensus intact (e.g., miner can't register, validator can't submit) | freeze risky changes, mitigate first |
| `S2` | Serious degradation (e.g., gameplay latency >2s, checkpoint corruption) | elevated repair priority |
| `S3` | Contained defect or debt item | backlog with owner |

## No-Ship Conditions

Do not advertise a capability as ready if:
- named proof is not trustworthy (INV-002)
- validator determinism is violated (INV-003)
- solver/gameplay separation is breached (INV-004)
- emission accounting has any imbalance
- Yuma Consensus output diverges from subtensor for identical inputs

## Human-Management Guidance

Dormant at zero humans. This operating system is for an unattended software
kernel augmented by malinka's autonomous development loop. When humans join:
- First hire: Substrate engineer (accelerates chain fork)
- Second hire: Poker domain expert (validates solver quality)
- Third hire: Growth (community building, game partnerships)

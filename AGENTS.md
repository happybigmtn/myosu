---
os_kind: autonomous_kernel
os_version: "2.0"
last_updated: "2026-03-16"
system: myosu
state: stage_0
domain: game_solving_chain
mission_doctrine: specs/031626-myosu-game-solving-chain.md
invariants: INVARIANTS.md
kpi: ops/kpi_registry.yaml
scorecard: ops/scorecard.md
risk: ops/risk_register.md
decisions: ops/decision_log.md
evidence: ops/evidence/
---

# myosu kernel

묘수. Decentralized game-solving protocol. Permissionless strategy computation
for imperfect-information games. Miners produce Nash-approximate strategies via
MCCFR. Validators measure exploitability. Yuma Consensus distributes emissions
to the strongest solvers. Agents and humans play through the same text interface.

## system architecture

```
┌──────────────────────────────────────────────────────────┐
│ CHAIN (Substrate)                                        │
│                                                          │
│  subnet_registry ──► neuron_registry ──► weight_storage  │
│       │                    │                   │         │
│       ▼                    ▼                   ▼         │
│  emission_dist  ◄─── yuma_consensus  ◄─── staking       │
│  (per tempo)         (median clip)        (voting power) │
└────────────────────┬─────────────────┬───────────────────┘
                     │                 │
          ┌──────────▼─────┐  ┌────────▼────────┐
          │ MINERS          │  │ VALIDATORS       │
          │                 │  │                  │
          │ MCCFR trainer   │  │ exploit oracle   │
          │ HTTP /strategy  │◄─┤ submit_weights   │
          └────────┬────────┘  └──────────────────┘
                   │
                   ▼
          ┌─────────────────┐
          │ GAMEPLAY         │
          │                  │
          │ stdin/stdout     │
          │ agent = human    │
          └─────────────────┘
```

| layer | function | actors |
|-------|----------|--------|
| chain | on-chain coordination: subnets, neurons, weights, emissions | pallet_game_solver |
| solvers | off-chain compute: MCCFR training, strategy serving | miners |
| validation | off-chain quality: exploitability scoring, weight submission | validators |
| gameplay | output surface: text interface, agent-native, stdin/stdout | humans, agents |

## market position

| incumbent | price | games | verification | status |
|-----------|-------|-------|--------------|--------|
| PioSolver | $249-2500 | NLHE only | trust-me | monopoly |
| MonkerSolver | €250+ | PLO only | trust-me | monopoly |
| GTO+ | $75 | NLHE only | trust-me | budget |
| Suphx (MSFT) | not deployed | Riichi only | research | locked |
| DeepNash (DeepMind) | not deployed | Stratego only | research | locked |

Protocol replaces all of the above. Exploitability is deterministic and
on-chain verifiable. New game = one trait implementation. No chain changes.

## structural advantages

| advantage | mechanism | fork resistance |
|-----------|-----------|-----------------|
| compute moat | months of MCCFR iterations compound toward Nash | fork starts at random strategy |
| abstraction tables | 13M isomorphisms clustered into 500 buckets (days of compute) | fork starts with empty tables |
| bond EMA | validator reputation accumulates over epochs | fork starts with zero trust |
| game engines | each `CfrGame` impl adds to platform value | fork must reimplement all games |
| network effects | miners → strategies → players → revenue → miners | fork has zero liquidity |

Chain is commodity infrastructure. Trained solver network is the moat.

## revenue model

| stage | source | mechanism |
|-------|--------|-----------|
| 0-1 | token emission | inflation funds miners and validators |
| 2+ | gameplay fees | per-session play against solvers |
| 2+ | coaching | strategy analysis: "solver plays X here because Y" |
| 2+ | tournaments | human vs solver with prize pools |
| 2+ | strategy marketplace | specialized strategies (ICM-adjusted, exploitative) |
| 2+ | API access | third-party platforms query strategies |

Emission-funded bootstrap transitions to revenue-funded sustainability.

## game targets

Selection: CFR fit → solver gap → market size → geographic coverage → tractability.

| # | game | market | solver gap | geography | stage |
|---|------|--------|-----------|-----------|-------|
| 1 | NLHE Heads-Up | $6B+ poker | PioSolver $250+ | global | 0 |
| 2 | NLHE 6-max | most-played format | expensive | global | 1 |
| 3 | PLO | $400M-1.2B | postflop unsolved | global | 1 |
| 4 | NLHE Tournament/ICM | most poker is MTT | no ICM-CFR solver | global | 1 |
| 5 | Short Deck | niche, 30% smaller state | limited | Asia | 1 |
| 6 | Teen Patti | India $2-3B by 2028 | none | India | 2 |
| 7 | Hanafuda (Koi-Koi) | cultural (Nintendo origin) | none | Japan | 2 |
| 8 | Hwatu / Go-Stop | Korea $9-10B gambling | none | Korea | 2 |
| 9 | Mahjong (Riichi) | $1.5-2.1B, 10% CAGR | zero consumer solvers | Japan, China | 2 |
| 10 | Bridge | 200M+ players | bidding unsolved | global | 2 |
| 11 | Gin Rummy | $1.5B Indian rummy | none | India | 2 |
| 12 | Stratego | DeepNash proved feasibility | research-only | global | 3 |
| 13 | OFC Chinese Poker | high-stakes niche | solvers were $30K+ | Russia/CIS | 3 |
| 14 | Spades | millions daily US | none | US | 3 |
| 15 | Liar's Dice | architecture proof | academic only | global | 3 |
| 16 | Dou Di Zhu (斗地主) | 600M+ users (Tencent) | none | China | 3 |
| 17 | Pusoy Dos / Big Two | dominant in PH/HK/TW/SG | none | SE Asia | 3 |
| 18 | Tien Len | Vietnam national game | none | Vietnam | 3 |
| 19 | Call Break | 10M+ app downloads each | none | Nepal, India | 3 |
| 20 | Backgammon | gambling tradition | solved since 1990s | Middle East | 3 |

```
CHINA         KOREA      JAPAN       INDIA        SE ASIA      WEST
┌─────────┐  ┌───────┐  ┌────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐
│Dou Di   │  │Hwatu  │  │Mahjong │  │Teen     │  │Pusoy Dos│  │Bridge   │
│Zhu      │  │       │  │Hanafuda│  │Patti    │  │Tien Len │  │Spades   │
│(600M+)  │  │       │  │        │  │Call Brk │  │         │  │Stratego │
│         │  │       │  │        │  │Gin Rummy│  │         │  │Liar's   │
└─────────┘  └───────┘  └────────┘  └─────────┘  └─────────┘  │Dice     │
                                                               │Backgamm.│
              ◄── 6 poker variants span all geographies ──►    │OFC      │
                                                               └─────────┘
```

Combined addressable market: $10B+ annually. No competitor addresses >2 games.

## presentation layer

### invariant

Agents and humans interact through the same text interface. If an LLM cannot
play from the rendered game state, the interface is broken.

### constraints

| constraint | rationale |
|------------|-----------|
| no mouse dependency | agents have no cursor |
| no visual-only information | agents parse text |
| input = natural language or structured commands | `call`, `raise 15`, `discard 3m` |
| game state must be LLM-complete | zero additional context needed |
| monochrome + 1 accent color max | density over decoration |
| no gradients, no backgrounds, no blinking | restraint is the aesthetic |

### color semantics

| color | meaning |
|-------|---------|
| white | narration |
| green | player action, positive outcome |
| red | opponent action, negative outcome |
| yellow | warning, time pressure |
| blue | system info, pot, stacks |
| dim | history, folded, metadata |
| bold | decision point, player cards |

### notation

```
cards:   A♠ K♥ Q♦ J♣ T♠ 9♥
hidden:  ·· ··
tiles:   [1m] [2p] [3s] [Ew]
dice:    ⚀ ⚁ ⚂ ⚃ ⚄ ⚅
borders: ─ │ ┌ ┐ └ ┘
```

### layout

```
┌─ state ────────────────────────────────────────┐
│ [compact: board, hands, stacks, pot]           │
├─ log ──────────────────────────────────────────┤
│ [scrollable action history]                    │
├─ input ────────────────────────────────────────┤
│ > _                                            │
└────────────────────────────────────────────────┘
```

State panel: fixed height, always visible, game-specific.
Log panel: scrollable conversation. Input: readline, tab-complete, `/commands`.

Layout is game-agnostic. State panel is the only per-game customization.

### agent protocol

Agent receives rendered text on stdin. Agent writes action on stdout.
Two agents play each other via pipe. Zero additional infrastructure.

```
agent_1 | game_engine | agent_2     # agent vs agent
human   | game_engine | agent       # human vs agent
```

### design language rules

1. no marketing language — if it sounds like a pitch, delete it
2. every section controls behavior — if it doesn't change a decision, remove it
3. tables > lists > paragraphs
4. one concept per block
5. visuals = system diagrams only
6. monochrome + 1 accent max
7. agents over users — system actors, not personas
8. deterministic tone — no hedging, no speculation
9. metrics map to actions — otherwise invalid
10. repo = runtime state, not documentation

## system states

```
stage_0 ──► stage_1 ──► stage_2 ──► stage_3
bootstrap    launch      platform    ecosystem
```

### stage_0: bootstrap

System produces verified solver output on devnet.

| exit condition | measured by |
|----------------|-------------|
| chain produces blocks with game-solving pallet | CF-05 smoke test |
| poker subnet registered, miners scored by validators | GS-09 integration test |
| miner produces MCCFR strategy profile | PE-01 training test |
| validator computes exploitability, submits weights | VO-06 loop test |
| Yuma distributes emissions proportional to quality | GS-05 Yuma test vectors |
| human plays one hand against trained solver | GP-02 game loop test |
| Liar's Dice validates multi-game (zero code changes) | MG-03 zero-change test |
| all 6 invariants pass | INV-001 through INV-006 |

### stage_1: launch

Mainnet. Multiple poker subnets. Real token economics.

| exit condition | measured by |
|----------------|-------------|
| mainnet with genesis validators | block production on public network |
| 3+ poker variant subnets | subnet count query |
| 10+ miners competing per subnet | active_miners_per_subnet metric |
| token economics sustain incentives | emission vs stake ratio |

### stage_2: platform

Non-poker games. Third-party engines. Revenue.

| exit condition | measured by |
|----------------|-------------|
| 2+ non-poker game subnets | subnet game_type diversity |
| game engine SDK published | external developer adoption |
| strategy marketplace operational | transaction count |
| revenue exceeds emission cost | financial model |

### stage_3: ecosystem

Self-sustaining. Global infrastructure.

| exit condition | measured by |
|----------------|-------------|
| 10+ game subnets across 5+ categories | subnet registry |
| thousands of miners globally | neuron count |
| coaching + tournament revenue | financial model |
| third-party applications | API consumer count |

## doctrine hierarchy

| priority | source | controls |
|----------|--------|----------|
| 1 | `specs/`, `ralph/SPEC.md` | what system must become |
| 2 | `INVARIANTS.md` | what must never be violated |
| 3 | `OS.md` | how the system decides |
| 4 | `ops/kpi_registry.yaml`, `ops/scorecard.md` | what is green/yellow/red |
| 5 | `ops/risk_register.md`, `ops/decision_log.md` | context for decisions |
| 6 | `state/` | whether the kernel is behaving |

## rule 0

System lacks trustworthy operating truth for a decision → install truth first.

1. challenge the requirement
2. delete before automating
3. simplify before scaling
4. shorten signal → decision → action path

Metric without action → delete. Review without priority change → delete.
Report without default action → noise.

## north star

| field | value |
|-------|-------|
| metric | `solver_exploitability_convergence` |
| definition | min(exploitability) across active miners per subnet (mbb/hand) |
| target | → 0 |
| source | validator consensus |
| cadence | per tempo |

### leading indicators

| metric | formula | action if red |
|--------|---------|---------------|
| `active_miners_per_subnet` | miners serving in last epoch | investigate incentive model |
| `validator_agreement_rate` | % pairs with <10% score divergence | investigate INV-003 |
| `gameplay_sessions_per_day` | completed human vs bot sessions | investigate product surface |

## guardrails

| metric | green | red | action |
|--------|-------|-----|--------|
| `false_green_proof_count` | 0 | >1 | halt claims, repair proof |
| `validator_determinism` | <1e-6 | >1e-3 | freeze emissions |
| `solver_gameplay_separation` | no cross-dep | any dep | revert |
| `emission_balance` | distributions == block_emission | imbalance | halt, audit |

## severity

| level | condition | response |
|-------|-----------|----------|
| S0 | consensus compromised, token accounting broken, scoring non-deterministic | freeze emissions, preserve evidence, halt |
| S1 | critical capability broken, consensus intact | freeze risky changes, mitigate |
| S2 | serious degradation | elevated repair priority |
| S3 | contained defect | backlog with owner |

## no-ship

System capability not ready if:
- proof not trustworthy (INV-002)
- validator determinism violated (INV-003)
- solver/gameplay separation breached (INV-004)
- emission accounting imbalanced
- Yuma output diverges from subtensor for identical inputs

## active functions

| function | mandate | output |
|----------|---------|--------|
| strategy | stage transitions, scope control, bottleneck decisions | priorities |
| security | consensus, verification, fairness, economics | audit, risk, no-ship |
| execution | land verified code, honest completion claims | proof, closure |
| product | gameplay surface, marketplace, game prioritization | UX decisions |

Dormant until stage_1: growth, revenue, finance, support.

## competitive landscape

| competitor | games | price | verification | gap |
|------------|-------|-------|--------------|-----|
| PioSolver | 1 | $249-2500 | none | multi-game, verifiable, open |
| MonkerSolver | 1 | €250+ | none | multi-game, verifiable, open |
| GTO+ | 1 | $75 | none | multi-game, verifiable |
| Bittensor SN | 0 games | $1-2M TAO lock | AI metrics | purpose-built for games |

No entity builds decentralized game-solving infrastructure.

## current priority

| # | work | AC prefix | blocking |
|---|------|-----------|----------|
| 1 | fork robopoker: serde + encoder constructor | RF-01..02 | GT-02, PE-01 |
| 2 | fork subtensor: strip to minimal chain | CF-01..05 | everything |
| 3 | game engine traits: re-export + wire | GT-01..05 | PE, MN, VO, GP |
| 4 | poker engine: solver, query, exploit | PE-01..04 | MN, VO, GP |
| 5 | game-solving pallet: Yuma, subnets, staking | GS-01..10 | MN, VO |
| 6 | miner: train, serve, checkpoint | MN-01..05 | VO, GP |
| 7 | validator: score, submit weights | VO-01..06 | GP |
| 8 | gameplay: human vs solver | GP-01..04 | stage_0 exit |
| 9 | multi-game proof: Liar's Dice | MG-01..04 | stage_0 exit |


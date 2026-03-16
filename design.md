---
design_system: myosu_interface
version: "1.0"
last_updated: "2026-03-16"
renderer: ratatui + crossterm
palette: monochrome + 1 semantic accent
input: stdin (agent-compatible)
output: stdout (LLM-parseable)
---

# myosu interface system

## 0. thesis

Interfaces are not dashboards. They are artifacts of a live system
converging toward equilibrium. Every screen is a statement of truth.
Every input is an intervention. Agents and humans see the same thing.

If an LLM cannot play from the rendered output, the interface is broken.
If a screenshot cannot stand alone, the screen is too bland.

## 1. invariants

| rule | violation |
|------|-----------|
| one screen = one dominant statement | redesign |
| no mouse dependency | agents have no cursor |
| no visual-only information | agents parse text |
| text carries meaning without color | color is semantic overlay, not structure |
| input = plaintext commands | `call`, `raise 15`, `discard 3m`, `challenge` |
| game state must be LLM-complete | zero context beyond what's rendered |

## 2. layout

Every screen. Every game. No exceptions.

```
╔══════════════════════════════════════════════════════════════╗
║ HEADER                                                       ║
╠══════════════════════════════════════════════════════════════╣
║                                                              ║
║ DECLARATION                                                  ║
║                                                              ║
╠══════════════════════════════════════════════════════════════╣
║                                                              ║
║ STATE                                                        ║
║ (game-specific — the only panel that changes per game)       ║
║                                                              ║
╠══════════════════════════════════════════════════════════════╣
║                                                              ║
║ LOG                                                          ║
║ (scrollable action history)                                  ║
║                                                              ║
╠══════════════════════════════════════════════════════════════╣
║ > _                                                          ║
╚══════════════════════════════════════════════════════════════╝
```

| panel | height | content | scrolls |
|-------|--------|---------|---------|
| header | 1 line | system path + game identity | no |
| declaration | 1-2 lines | dominant truth about current state | no |
| state | 4-12 lines | game-specific: cards, board, stacks, tiles, dice | no |
| log | flex | action history, most recent at bottom | yes |
| input | 1 line | readline with tab-complete | no |

The state panel is the ONLY component that varies per game.
Everything else is universal.

## 3. color

Monochrome default. Color is semantic, never decorative.

| token | hex | use |
|-------|-----|-----|
| `fg` | #c0c0c0 | default text |
| `fg.bright` | #ffffff | emphasis, player cards, active decision |
| `fg.dim` | #606060 | history, folded, metadata |
| `converge` | #00cc66 | positive: player action, win, convergence |
| `diverge` | #cc3333 | negative: opponent action, loss, violation |
| `unstable` | #ccaa00 | warning: time pressure, instability |
| `focus` | #4488cc | system info: pot, stacks, subnet data |
| `protocol` | #8844cc | rare: protocol identity, myosu branding |

One accent per screen. If multiple colors appear, one must dominate.

## 4. typography

Terminal monospace only. No font selection — the user's terminal font IS
the typeface. Design for:

| property | value |
|----------|-------|
| min width | 60 columns |
| target width | 80 columns |
| max width | 120 columns (no wrap) |
| line height | 1 (terminal default) |
| indent | 2 spaces for nested data |

### text transforms

| pattern | use |
|---------|-----|
| `ALLCAPS` | declarations, headers, field names |
| `lowercase` | values, actions, game narration |
| `Title Case` | never |
| `camelCase` | never |

## 5. notation — typed object system

Every game object is one of 6 types. Each type has a fixed rendering pattern
that works in any terminal, any locale, any column width. No CJK characters.
No ambiguous symbols. The type IS the visual language.

### object types

| type | pattern | examples | notes |
|------|---------|----------|-------|
| **card** | `Rank``Suit` | `A♠` `K♥` `Q♦` `J♣` `T♠` `9♥` | western deck: rank + suit glyph |
| **tile** | `[``id``]` | `[1m]` `[9p]` `[Ew]` `[Dr]` | bracketed = tile (mahjong, dominoes) |
| **flower** | `{``month``.``type``}` | `{jan.R}` `{feb.B}` `{mar.L}` `{dec.R}` | hwatu/hanafuda: month.type (R=ribbon, B=bright, L=leaf, S=seed) |
| **die** | `[``face``]` | `[3]` `[5]` `[1]` | bracketed numeral = die face |
| **hidden** | `··` or `░` | `·· ·· ··` `░░░░░░░` | dots = known count, bars = unknown count |
| **piece** | `○` `●` or letter | `○3` `●5` `○bar` | backgammon, stratego pieces |

### why this works

- **cards** are bare (no brackets) — `A♠ K♥` is instantly recognizable
- **tiles** are bracketed — `[1m]` visually groups the id, distinguishes from cards
- **flowers** are braced — `{jan.R}` is unambiguous, English month names, no kanji
- **dice** are bracketed numerals — `[3]` reads as "a die showing 3"
- **hidden** uses two visual weights — `··` for "I know how many" vs `░` for "unknown blob"

### hwatu / hanafuda notation

The original `[松1]` notation fails: kanji breaks column alignment, is
culturally specific to Japanese (hwatu is Korean), and many terminals
render CJK at double-width unpredictably.

Instead, each flower card is `{month.type}`:

| month | code | bright (B) | ribbon (R) | seed (S) | leaf (L) |
|-------|------|-----------|-----------|---------|---------|
| January | jan | crane | red poem | — | — |
| February | feb | bush warbler | red poem | — | — |
| March | mar | curtain | red poem | — | — |
| April | apr | cuckoo | red | — | — |
| May | may | bridge | red | — | — |
| June | jun | butterflies | blue | — | — |
| July | jul | boar | red | — | — |
| August | aug | moon | geese | — | — |
| September | sep | sake cup | blue | — | — |
| October | oct | deer | blue | — | — |
| November | nov | rain-man | swallow | lightning | — |
| December | dec | phoenix | — | — | — |

Each month has 4 cards. The type suffix distinguishes them:
`{jan.B}` = January bright (crane), `{jan.R}` = January ribbon,
`{jan.S}` = January seed, `{jan.L}` = January leaf.

This renders cleanly at fixed width (7 chars per card), reads in English,
and is culturally neutral between Korean and Japanese variants.

### dou di zhu / big two notation

Chinese card games use the western deck. No special notation needed.
Jokers: `BJ` (black joker), `RJ` (red joker).

### multiplayer seating

Games with 3+ players show seats in fixed order relative to the player:

| players | seat order (top to bottom) |
|---------|---------------------------|
| 2 | you, solver |
| 3 (DDZ) | you (landlord/peasant), peasant 1, peasant 2 |
| 4 (mahjong) | east (you), south, west, north (wind order) |
| 4 (bridge) | north (dummy), east, south (you), west |
| 4 (spades) | you, left opponent, partner, right opponent |

The acting player is always labeled. Non-acting players show card count
and discard history. The player's own hand is always fully expanded.

### formatting constants

```
QUANTITY    142bb  (big blinds, lowercase unit)
POINTS     25000  (integer, no separator)
RATIO      97.4%  (one decimal)
EMPTY      --
SEPARATOR  ───
```

## 6. declarations

The declaration panel makes a statement. Not a label. Not a status.
A statement about what the system is doing right now.

### declaration vocabulary

| state | declaration |
|-------|-------------|
| game active, normal | `YOU ARE FACING THE STRONGEST AVAILABLE STRATEGY` |
| game active, your turn | `THE SYSTEM AWAITS YOUR DECISION` |
| game active, opponent turn | `SOLVER IS COMPUTING` |
| convergence | `STRATEGIES ARE CONVERGING` |
| divergence | `CONSENSUS IS BREAKING` |
| no miners | `NO MINERS ARE SERVING QUERIES` |
| stale data | `MINER STRATEGY IS STALE` |
| incident | `EMISSION ACCOUNTING IS BROKEN` |
| victory | `YOU TOOK 14bb FROM THE SOLVER` |
| defeat | `THE SOLVER TOOK 8bb FROM YOU` |
| session end | `SESSION COMPLETE` |
| fallback active | `PLAYING AGAINST RANDOM STRATEGY (MINER UNREACHABLE)` |

Declarations are ALLCAPS. No punctuation. No adjectives unless functional.

## 7. input protocol

```
> fold                     fold
> call                     call current bet
> raise 15                 raise to 15bb
> r 15                     raise to 15bb
> shove                    all-in
> check                    check
> discard 3m               discard 3-man tile
> draw                     draw from wall/pile
> riichi                   declare riichi
> tsumo                    declare self-draw win
> ron                      declare win off discard
> bid 3 fours              bid three fours (liar's dice)
> challenge                challenge previous bid
> play A♠ K♥               play card combination
> double                   offer doubling cube
> accept                   accept double
> pass                     pass turn
> ?                        show legal actions
> /stats                   session statistics
> /history                 full hand/game history
> /analyze                 query solver for GTO analysis
> /quit                    exit
```

Invalid input → clarification prompt. Never an error.

```
> raise
raise to how much? (min 4bb, max 94bb)
> 15
you raise to 15bb.
```

## 8. game screens

### 8.1 NLHE HEADS-UP

```
MYOSU / NLHE-HU / HAND 47

YOU ARE FACING THE STRONGEST AVAILABLE STRATEGY

  board    T♠  7♥  2♣  ·  ·
  you      A♠ K♥                94bb   BB
  solver   ·· ··                94bb   SB
  pot      12bb

───

  solver raises to 6bb
  you call. pot 12bb.
  ─── flop: T♠ 7♥ 2♣
  solver checks.

> raise 8
```

### 8.2 RIICHI MAHJONG

```
MYOSU / RIICHI / EAST 1 ROUND 3

THE SYSTEM AWAITS YOUR DECISION

  east (you)   [1m][2m][3m] [5p][6p][7p] [3s][4s] [9s][9s] [E][E]
  draw         [5s]
  south        ░░░░░░░░░░░░░   discards: 1m 9p 5s Nw
  west         ░░░░░░░░░░░     discards: 2m 3p 7s
  north        ░░░░░░░░░░░░░   discards: 4p
  riichi       no              points: 25000    dora: [3m]

───

  south discards Nw
  west discards 7s
  north discards 4p
  you draw [5s]

> discard 4s
```

### 8.3 TEEN PATTI

```
MYOSU / TEEN PATTI / ROUND 12

THE SYSTEM AWAITS YOUR DECISION

  you       A♥ K♠ J♦         seen     pot: 240
  solver    ·· ·· ··         blind    stake: 40

───

  solver posts blind 20
  you see cards (cost 40)
  solver raises blind to 40

> raise 80
```

### 8.4 LIAR'S DICE

```
MYOSU / LIARS DICE / ROUND 3

THE SYSTEM AWAITS YOUR DECISION

  you       [3] [5] [6] [1] [4]      5 dice
  solver    ·· ·· ·· ··              4 dice remaining
  last bid  three fives (solver)

───

  you bid two threes
  solver bids two fives
  you bid three fours
  solver bids three fives

> challenge
```

### 8.5 DOU DI ZHU (斗地主)

```
MYOSU / DOU DI ZHU / ROUND 7

YOU ARE THE LANDLORD

  you (landlord)   3♦ 5♥ 5♠ 7♦ 8♣ 9♥ T♠ J♦ Q♥ K♣ A♠ 2♥ 2♦ BJ RJ
  kitty            [shown: 7♦ 2♥ RJ]
  peasant 1        ░░░░░░░░░░░░░░░░░   14 cards
  peasant 2        ░░░░░░░░░░░░░░       11 cards
  last play        pair 8♣ 8♠ (peasant 1)

───

  peasant 2 passes
  peasant 1 plays pair 8♣ 8♠

> play 2♥ 2♦
```

### 8.6 BRIDGE

```
MYOSU / BRIDGE / BOARD 14

CONTRACT: 3NT BY SOUTH (YOU)

  north (dummy)   A♠ Q♠ 7♠ · K♥ T♥ 4♥ · A♦ 9♦ · J♣ 8♣ 5♣
  east            ░░░░░░░░░░░░░
  south (you)     K♠ J♠ · A♥ 8♥ 3♥ · Q♦ J♦ 7♦ 3♦ · A♣ 6♣
  west            ░░░░░░░░░░░░░
  trick 4 of 13   won: 2    needed: 9     led by: west

───

  west leads 5♦
  north plays 9♦
  east plays T♦

> play Q♦
```

### 8.7 BACKGAMMON

```
MYOSU / BACKGAMMON / GAME 3

SOLVER IS COMPUTING

  24 23 22 21 20 19   18 17 16 15 14 13
  ·  ·  ·  ·  ○3 ·   ·  ○5 ·  ·  ·  ●2
  ─────────────────   ─────────────────
  ·  ·  ·  ·  ●3 ·   ·  ●5 ·  ·  ·  ○2
  1  2  3  4  5  6    7  8  9  10 11 12

  you (○)    pip: 167    bar: 0    off: 0
  solver (●) pip: 167    bar: 0    off: 0
  dice       [3] [5]    cube: 1

───

  solver rolls [4] [2]
  solver moves 8/4 6/5
  you roll [3] [5]

> move 24/20 13/10
```

### 8.8 HWATU / GO-STOP

```
MYOSU / HWATU / ROUND 5

THE SYSTEM AWAITS YOUR DECISION

  hand     {jan.B} {feb.L} {mar.R} {apr.S} {may.S} {jul.R} {oct.L}
  field    {jan.R} {feb.S} {mar.L} {nov.S}
  capture  {jan.S}{jan.L} {feb.R}{feb.B}         score: 3
  solver   ░░░░░░░░░                              score: 1
  deck     26 remaining

───

  solver plays {nov.R}, captures {nov.S}
  you draw {jan.B}

> play {jan.B}
```

### 8.9 GIN RUMMY

```
MYOSU / GIN RUMMY / HAND 8

THE SYSTEM AWAITS YOUR DECISION

  hand       3♥ 4♥ 5♥ · 8♦ 9♦ · J♣ Q♣ K♣ · 2♠
  deadwood   2 (gin possible if discard 2♠)
  discard    [7♠] (top)
  solver     ░░░░░░░░░░   10 cards
  stock      22 remaining

───

  solver draws from stock
  solver discards 7♠

> draw discard
```

### 8.10 BRIDGE BIDDING PHASE

```
MYOSU / BRIDGE / BOARD 14 / AUCTION

THE SYSTEM AWAITS YOUR BID

  you (south)   K♠ J♠ · A♥ 8♥ 3♥ · Q♦ J♦ 7♦ 3♦ · A♣ 6♣
  hcp: 15       distribution: 2-3-4-4

  AUCTION

  west    north   east    south
  --      --      --      ?

───

  bidding opened to you

> bid 1nt
```

### 8.11 COACHING / ANALYZE OUTPUT

```
MYOSU / NLHE-HU / HAND 47 / ANALYSIS

SOLVER RECOMMENDS RAISING

  board    T♠  7♥  2♣
  you      A♠ K♥         94bb   BB
  pot      12bb

  EQUILIBRIUM MIX

  action      frequency    EV
  raise 8bb   47.2%        +2.8bb
  raise 12bb  18.6%        +2.4bb
  call        22.1%        +1.9bb
  check       12.1%        +1.1bb

───

  your hand is in the top 8% of possible holdings
  solver exploitability: 13.2 mbb/h (miner 12)

> raise 8
```

### 8.12 GAME SELECTION

```
MYOSU / LOBBY

SELECT A GAME

  AVAILABLE SUBNETS

  id  game         miners  best_exploit   your_balance
  1   nlhe-hu      12      13.2 mbb/h     1000bb
  2   nlhe-6max    18      15.8 mbb/h     1000bb
  3   plo          12      --             1000bb
  4   teen-patti    4      8.1 mbb/h      --
  5   liars-dice    2      0.02           --

───

  type subnet id to join, or:
  /new       create practice session
  /spectate  watch agent vs agent

> 1
```

## 9. operational screens

### 9.1 NETWORK CONSOLE

```
MYOSU / NETWORK

THE SYSTEM IS PRODUCING VALID STRATEGIES

  chain      devnet          block: 812944     finality: OK
  invariants PASS            uptime: 14d 3h

───

SUBNET FIELD

  id  game        miners  best_exploit   agreement  status
  1   nlhe-hu     12      13.2 mbb/h     97.4%      ACTIVE
  2   nlhe-6max   18      15.8 mbb/h     95.1%      ACTIVE
  3   plo         12      --             --          BOOTSTRAP
  4   teen-patti   4      8.1 mbb/h      99.2%      ACTIVE

───

SIGNAL

  14:22:01 subnet 1 weights finalized
  14:22:03 subnet 2 validator divergence 3.2e-4
  14:22:05 subnet 3 miner 7 registered

[enter] inspect  [q] quit
```

### 9.2 MINER INSPECTION

```
MYOSU / MINER 12

THIS AGENT PRODUCES THE BEST KNOWN STRATEGY

  subnet         nlhe-hu
  stake          40.22
  exploitability 13.2 mbb/h
  trend          converging (-0.8 last epoch)
  latency        84ms
  checkpoint     18s ago

───

EPOCH HISTORY

  epoch  exploitability  delta
  1482   13.2            -0.8
  1481   14.0            -0.5
  1480   14.5            -0.3
  1479   14.8            -0.2
  1478   15.0            -0.4

───

SIGNAL

  14:22:08 score accepted
  14:21:59 strategy checkpoint saved
  14:21:41 queried by validator 3

[r] refresh  [q] back
```

### 9.3 VALIDATOR DIVERGENCE

```
MYOSU / CONSENSUS

CONSENSUS IS BREAKING

  threshold  1e-6
  observed   3.2e-4
  subnet     2 (nlhe-6max)
  severity   S1

───

VALIDATOR FIELD

  id  score_m12  score_m08  divergence  status
  v1  0.912      0.887      2.0e-3      OUTLIER
  v2  0.910      0.885      1.0e-4      OK
  v3  0.911      0.886      2.0e-4      OK

───

DEFAULT ACTION

  1. freeze emissions on subnet 2
  2. capture scoring inputs from all validators
  3. compare encoder hashes across validators

[q] exit
```

### 9.4 SESSION SUMMARY

```
MYOSU / SESSION

SESSION COMPLETE

  game      nlhe-hu
  hands     47
  result    +14bb
  bb/hand   +0.30

───

HAND DISTRIBUTION

  won at showdown    12
  won without SD     18
  lost at showdown    8
  lost without SD     9

  best hand    +22bb (hand 31, A♠ A♥ vs K♠ K♦)
  worst hand   -16bb (hand 19, bluff caught)

───

  solver source: miner 12 (13.2 mbb/h exploitability)

[enter] new session  [q] quit
```

## 10. anti-patterns

| pattern | why it fails |
|---------|-------------|
| colorful dashboards | decoration, not information |
| decorative ASCII art | noise |
| animated terminals | agents can't parse animation |
| vague labels ("status", "info") | not a statement |
| friendly UX copy ("great job!") | not a system |
| oversized whitespace | density is the medium |
| progress bars | show the number, not a bar |
| loading spinners | show "COMPUTING" text |
| modal dialogs | breaks stdin/stdout pipe model |
| mouse-only interactions | agents have no cursor |

## 11. agent protocol

```
human   ──stdin──►  myosu-play  ──stdout──►  human
agent   ──stdin──►  myosu-play  ──stdout──►  agent

agent_a | myosu-play --pipe | agent_b     # agent vs agent
```

The `--pipe` flag strips all formatting (borders, color codes) and outputs
pure structured text. Agent sees:

```
MYOSU/NLHE-HU/HAND47
board: Ts 7h 2c
you: As Kh 94bb BB
solver: -- -- 94bb SB
pot: 12bb
action: solver raises 6bb
>
```

Agent writes: `call`

Same binary. Same protocol. Zero additional infrastructure.

### pipe output for non-poker games

```
MYOSU/RIICHI/EAST1R3
hand: 1m 2m 3m 5p 6p 7p 3s 4s 9s 9s Ew Ew
draw: 5s
south: 13 tiles discards: 1m 9p 5s Nw
west: 11 tiles discards: 2m 3p 7s
north: 13 tiles discards: 4p
riichi: no points: 25000 dora: 3m
>
```

```
MYOSU/LIARS-DICE/R3
you: 3 5 6 1 4 (5 dice)
solver: 4 dice
last_bid: three fives (solver)
>
```

```
MYOSU/HWATU/R5
hand: jan.B feb.L mar.R apr.S may.S jul.R oct.L
field: jan.R feb.S mar.L nov.S
capture: jan.S jan.L feb.R feb.B score: 3
solver: 9 cards score: 1
deck: 26
>
```

### /history output

```
MYOSU / SESSION / HISTORY

RECENT HANDS

  hand  cards     street     action     result
  47    A♠ K♥     river      raise      +22bb
  46    Q♠ 5♦     preflop    fold       -1bb
  45    K♣ K♦     turn       call       +8bb
  44    7♥ 2♠     preflop    fold       -0.5bb
  43    A♦ T♣     flop       raise      -12bb

  showing 5 of 47. type /history all for full log.
```

### invalid input handling

```
> xyz
unknown action. type ? for options.

> raise abc
raise amount must be a number. raise to how much? (min 4bb, max 94bb)
> 15
you raise to 15bb.

> discard
which tile? your hand: [1m][2m][3m] [5p][6p][7p] [3s][4s] [9s][9s] [E][E]
> 4s
you discard [4s].
```

Invalid input is NEVER an error message. It is a clarification prompt that
shows what the system needs. The system remains in the same state — no
action taken, no penalty, just a narrower question.

### miner unreachable fallback

```
MYOSU / NLHE-HU / HAND 12

PLAYING AGAINST RANDOM STRATEGY (MINER UNREACHABLE)

  board    ·  ·  ·  ·  ·
  you      J♠ T♠                100bb   BB
  solver   ·· ··                100bb   SB
  pot      3bb

───

  miner 12 unreachable (timeout 500ms)
  fallback: uniform random over legal actions
  solver raises to 6bb

> call
```

The declaration changes color to `unstable` (yellow) when fallback is active.
The signal log shows the timeout reason so the player knows quality is degraded.

### /stats output

```
MYOSU / SESSION STATS

  game         nlhe-hu
  hands        47
  result       +14bb
  bb/hand      +0.30

  STREAKS

  current      W3
  best         W7 (hands 22-28)
  worst        L4 (hands 8-11)

  POSITION

  as BB        +0.42 bb/hand (24 hands)
  as SB        +0.17 bb/hand (23 hands)

  solver source: miner 12 (13.2 mbb/h)
```

## 12. build checklist

Before shipping any screen:

| check | fail action |
|-------|-------------|
| makes a clear statement? | add declaration |
| exactly one dominant idea? | split or merge |
| all data tied to action? | remove inert data |
| any text decorative? | delete it |
| feels like a system? | remove polish |
| screenshot-worthy? | increase density or contrast |
| LLM can play from output? | restructure for parseability |
| works at 60 columns? | reflow |
| works without color? | restructure (color is overlay) |

## 13. what makes this distinctive

Most terminal apps look the same: box-drawing borders, tables, dim text on
dark background. lazygit, bottom, k9s — competent, interchangeable. myosu
must be instantly recognizable.

### the signature elements

**A. Declarations as hero text.**

No other terminal app uses full-width declarative statements as the visual
anchor. `STRATEGIES ARE CONVERGING` is not a status bar — it is the first
thing the eye reads. It functions like a newspaper headline: it tells you
the story before you read the data. This is the single most distinctive
element. It must be preserved in every screen.

**B. The separator rhythm.**

The `───` separator is not a border. It is a breath. It appears between
state and log, between log sections, between hands. The rhythm of
separator-data-separator-data creates a visual cadence unique to myosu.
Most TUIs use box-drawing borders to contain content. myosu uses horizontal
rules to pace it.

**C. Two-space indent as information hierarchy.**

All data in the state and log panels is indented 2 spaces from the left edge.
Headers (SUBNET FIELD, SIGNAL) are flush left. This creates a consistent
left-margin rhythm that reads differently from the flush-left tables of most
terminal apps.

**D. The `/command` metachannel.**

Game actions are bare words: `call`, `raise 15`, `discard 3m`.
System actions use a `/` prefix: `/stats`, `/analyze`, `/quit`.
This split makes the interface feel like two layers: the game conversation
(foreground) and the system shell (background). No other game interface
has this layered input model.

**E. The `--pipe` protocol.**

The ability to strip all formatting and pipe to another process is not just
a feature — it is the identity claim. "This interface is a protocol, not a
product." No other game platform treats its UI as a machine-readable API.

### the anti-lazygit test

If a screenshot of myosu could be mistaken for lazygit or htop, the screen
has failed. Check:
- Is there a declaration? (lazygit has none)
- Is the dominant visual element text, not borders? (lazygit is border-heavy)
- Does the screen make a claim about the system's state? (dashboards don't)
- Could an LLM read it and take an action? (monitoring tools can't)

## 14. final rule

If an interface feels like SaaS → remove polish.
If it feels like docs → remove explanation.
If it feels like a toy → remove decoration.

It should feel like inevitable infrastructure already in motion.

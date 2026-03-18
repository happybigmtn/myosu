# Riichi Mahjong (立直麻雀)

## Game Identity

| Property | Value |
|----------|-------|
| Full Name | Riichi Mahjong (立直麻雀 / Japanese Competition Mahjong) |
| Variants | EMA (European), WRC (World Riichi Championship), Tenhou rules, M-League rules |
| Players | 4 |
| Information | Imperfect (hidden hands, unknown wall, discards visible) |
| Stochasticity | Stochastic (tile draws from wall, dora indicators) |
| Zero-Sum | Yes (total points fixed per game) |
| Solved Status | Unsolved; superhuman AI demonstrated (Suphx, 2020) |

## Overview

Riichi Mahjong (also called Japanese Mahjong) is a four-player tile game originating from Chinese Mahjong, refined in Japan with distinctive rules including the riichi declaration, dora bonus tiles, and a structured yaku (scoring pattern) system. Players draw and discard tiles to assemble a complete hand of 14 tiles. The game combines pattern recognition, probability calculation, and defensive play. Unlike Chinese variants, Riichi Mahjong requires at least one qualifying yaku to win and features a complex scoring system based on han (doubles) and fu (base points).

**Players:** 4 (3-player variant exists but is non-standard)
**Tiles:** 136 tiles (34 unique tiles, 4 copies each)
**Objective:** Score the most points across a series of hands by assembling winning tile combinations

## Equipment

### Tile Set (136 Tiles)

**Suited Tiles (108 tiles):** Three suits, numbered 1-9, four copies each:
- **Manzu (萬子 / Characters):** 1m-9m (36 tiles)
- **Pinzu (筒子 / Circles/Dots):** 1p-9p (36 tiles)
- **Souzu (索子 / Bamboo):** 1s-9s (36 tiles)

**Honor Tiles (28 tiles):**
- **Kazehai (風牌 / Wind tiles):** East (東), South (南), West (西), North (北) — 4 copies each (16 tiles)
- **Sangenpai (三元牌 / Dragon tiles):** White (白 Haku), Green (發 Hatsu), Red (中 Chun) — 4 copies each (12 tiles)

**Terminal tiles (端牌):** 1 and 9 of each suit (1m, 9m, 1p, 9p, 1s, 9s)
**Simple tiles (中張牌):** 2-8 of each suit

### Red Dora (赤ドラ) — Optional but Standard
Most modern rule sets replace one 5 in each suit with a red-colored variant (red five). Each red five acts as a permanent dora, adding +1 han. Typically: one red 5m, one red 5p, one red 5s (3 red dora total).

### Other Equipment
- Point sticks (点棒 Tenbou): 10,000 / 5,000 / 1,000 / 100 denominations
- Dice (2)
- Wind marker or round indicator

## Setup

### Point Distribution
Each player starts with 25,000 points (some rule sets use 30,000). Point sticks represent these points.

### Seating and Winds
1. Determine seating randomly. Each seat is assigned a wind: East (東), South (南), West (西), North (北), proceeding counter-clockwise.
2. The **East** player is the dealer (親 Oya) for the first hand.
3. Wind assignments rotate counter-clockwise after each hand where the dealer does not win (or does not achieve tenpai in a draw).

### Building the Wall
1. Each player creates a row of 17 face-down tile stacks, 2 tiles high (34 tiles per side).
2. The four rows form a square "wall."
3. The dealer rolls two dice. The sum determines the wall break point (counted counter-clockwise from the dealer's wall).
4. The last 7 stacks (14 tiles) from the break point become the **dead wall** (王牌 Wanpai). The top tile of the 3rd stack from the end of the dead wall is flipped face-up as the **dora indicator**.

### Dealing
Starting from the break point, each player draws 4 tiles at a time, cycling counter-clockwise, until all players have 12 tiles. Then each player draws 1 more tile for 13 total. The dealer draws a 14th tile to begin play.

## Game Flow

### Game Structure
A full game (半荘 Hanchan) consists of at least 8 hands across two rounds:
- **East Round (東場 Tonba):** 4 hands (East 1 through East 4)
- **South Round (南場 Nanba):** 4 hands (South 1 through South 4)

The dealer rotates when the dealer does not win the hand and is not tenpai at an exhaustive draw. When the dealer wins or is tenpai at a draw, the hand is replayed with the same dealer (this is called a "renchan" / 連荘, and a honba counter is added).

### Hand Structure
Each hand proceeds as follows:

1. **Dealer's first discard:** The dealer (14 tiles) discards one tile to begin.
2. **Draw-Discard cycle:** Counter-clockwise from the dealer, each player:
   - Draws one tile from the wall (starting from the break point, proceeding clockwise)
   - Optionally declares a win (Tsumo) if the drawn tile completes their hand
   - Discards one tile face-up in front of them (their discard river / 河 Kawa)
3. **Calls:** After any player's discard, other players may call (see Actions).
4. **Hand end:** The hand ends when a player wins (Tsumo or Ron), or the wall is exhausted (draw / 流局 Ryuukyoku).

### Turn Order After Calls
When a tile is called (Chi, Pon, Kan), the calling player's turn is next, and play continues counter-clockwise from them. Skipped players lose their turn.

### Call Priority
When multiple players want to call the same discard:
1. **Ron** (win) takes highest priority
2. **Pon/Kan** takes priority over Chi
3. **Chi** has lowest priority (and can only be called by the next player in turn order)

If multiple players call Ron simultaneously, the player closest to the discarder (counter-clockwise) wins.

## Actions

### Draw and Discard (ツモ切り / Tsumo-giri)
The default action: draw a tile from the wall, optionally rearrange your hand, discard one tile.

### Tsumo (ツモ / Self-draw Win)
Declare a win using the tile you just drew. Your hand must:
1. Be a complete winning shape (see Winning Conditions)
2. Contain at least one qualifying yaku
3. Not be in furiten (see Furiten)

### Ron (ロン / Win by Discard)
Declare a win using another player's discard. Same requirements as Tsumo, plus you must not be in furiten.

### Chi (チー / Sequence Call)
Claim the previous player's (only the player to your left) discard to complete a sequence (shuntsu) in your hand. Reveal the completed sequence face-up. Your hand becomes "open" (鳴き Naki). Discard one tile from your hand.

### Pon (ポン / Triplet Call)
Claim any player's discard to complete a triplet (koutsu). Reveal the triplet face-up. Your hand becomes open. Discard one tile.

### Kan (カン / Quad Call)
Form a set of four identical tiles. There are three types:

- **Daiminkan (大明槓 / Open Kan):** Claim a discard to complete a quad when you hold three matching tiles. The quad is revealed. Draw a replacement tile from the dead wall. Flip a new dora indicator.
- **Shouminkan (小明槓 / Added Kan):** Add a drawn tile to an existing open triplet (Pon). The quad is revealed. Draw a replacement tile. Flip a new dora indicator. Other players may declare Ron on the added tile (chankan).
- **Ankan (暗槓 / Closed Kan):** Declare a quad from four tiles all in your closed hand. The quad is placed face-down (middle tiles revealed, outer tiles face-down). Draw a replacement. Flip a new dora indicator. Your hand remains closed.

### Riichi (リーチ / Ready Declaration)
When your hand is closed (no open calls) and tenpai (one tile away from winning):
1. Declare "Riichi" and place a 1,000-point stick on the table as a bet.
2. Turn your discard sideways to indicate the riichi declaration.
3. Your hand is now locked — you cannot change your hand composition. You must discard every drawn tile unless it completes your hand.
4. If you win, you collect the riichi bet(s) on the table. If you lose, the riichi stick goes to the winner.
5. Riichi grants the "Riichi" yaku (1 han) and access to ura-dora.

### Double Riichi (ダブルリーチ / Double Riichi)
If a player's initial dealt hand is already tenpai (before any calls are made by any player), they may declare Double Riichi on their first discard. Worth 2 han instead of 1.

## Winning Conditions

### Standard Winning Hand (Agari / 和了)
A complete hand consists of exactly 14 tiles arranged as:
- **4 mentsu (groups) + 1 jantai (pair):**
  - Mentsu can be: Shuntsu (sequence of 3 consecutive same-suit tiles), Koutsu (triplet), or Kantsu (quad, counts as a triplet for hand structure but with 4 tiles)
  - The pair (jantai / atama) is 2 identical tiles

### Special Winning Shapes
- **Chiitoitsu (七対子 / Seven Pairs):** 7 distinct pairs (14 tiles). No mentsu structure needed.
- **Kokushi Musou (国士無双 / Thirteen Orphans):** One of each terminal and honor tile (13 unique tiles) plus one duplicate of any of them.

### Requirements to Win
1. Complete hand shape (above)
2. At least one valid yaku
3. Not in furiten (for Ron only)

### Furiten (振聴 / Sacred Discard)
A player is in furiten and CANNOT declare Ron if:
- **Permanent Furiten:** Any tile that would complete their hand exists in their own discard river.
- **Temporary Furiten:** A tile that would complete their hand was discarded by another player since their last turn, and they did not call Ron on it. This resets when they make their next discard.
- **Riichi Furiten:** After declaring Riichi, if any winning tile is discarded and the player does not (or cannot) call Ron, they enter permanent furiten for the rest of the hand.

A player in furiten may still win by Tsumo (self-draw), provided they have a valid yaku.

### Exhaustive Draw (流局 / Ryuukyoku)
When the wall is exhausted (all tiles drawn except the dead wall), the hand ends in a draw:
- Players reveal whether they are tenpai (one tile away from a complete hand) or noten (not tenpai).
- Tenpai players receive points from noten players: 3,000 points total is redistributed (split among noten players to tenpai players).
- If the dealer is tenpai, the hand is replayed (renchan).

## Scoring

### Overview
Scoring is determined by two values: **han** (翻, doublings from yaku and dora) and **fu** (符, base points from hand composition). These feed into a formula or lookup table.

### Fu Calculation

| Source | Fu |
|--------|----|
| **Base** | 20 (open hand / tsumo) or 30 (closed ron) |
| **Tsumo win** | +2 (except for pinfu tsumo, which is 20 fu flat) |
| **Open Triplet of simples (2-8)** | +2 |
| **Closed Triplet of simples** | +4 |
| **Open Triplet of terminals/honors** | +4 |
| **Closed Triplet of terminals/honors** | +8 |
| **Open Quad of simples** | +8 |
| **Closed Quad of simples** | +16 |
| **Open Quad of terminals/honors** | +16 |
| **Closed Quad of terminals/honors** | +32 |
| **Pair of dragons** | +2 |
| **Pair of seat wind** | +2 |
| **Pair of round wind** | +2 |
| **Kanchan wait (middle of sequence)** | +2 |
| **Penchan wait (edge: 1-2 waiting on 3, or 8-9 waiting on 7)** | +2 |
| **Shanpon wait (dual triplet wait)** | +0 |
| **Tanki wait (pair wait)** | +2 |
| **Ryanmen wait (two-sided: e.g., 4-5 waiting on 3 or 6)** | +0 |

Fu is totaled and rounded up to the nearest 10. Minimum 30 fu for any hand.

**Exceptions:**
- Chiitoitsu (Seven Pairs) is always 25 fu, regardless of composition.
- Pinfu + Tsumo = 20 fu (the +2 for tsumo is waived).
- Open Pinfu (all sequences, open hand, valueless pair) = 30 fu.

### Basic Points Formula
```
Basic Points = fu x 2^(2 + han)
```
This is capped at 2,000 basic points (Mangan). For 5+ han, fu is irrelevant.

### Limit Hands

| Han | Name | Basic Points | Non-Dealer Ron | Dealer Ron | Non-Dealer Tsumo (each) | Dealer Tsumo (each) |
|-----|------|-------------|----------------|------------|--------------------------|---------------------|
| 3 (70+ fu) / 4 (40+ fu) / 5 | **Mangan** (満貫) | 2,000 | 8,000 | 12,000 | 2,000/4,000 | 4,000 all |
| 6-7 | **Haneman** (跳満) | 3,000 | 12,000 | 18,000 | 3,000/6,000 | 6,000 all |
| 8-10 | **Baiman** (倍満) | 4,000 | 16,000 | 24,000 | 4,000/8,000 | 8,000 all |
| 11-12 | **Sanbaiman** (三倍満) | 6,000 | 24,000 | 36,000 | 6,000/12,000 | 12,000 all |
| 13+ | **Yakuman** (役満) | 8,000 | 32,000 | 48,000 | 8,000/16,000 | 16,000 all |

For non-dealer Tsumo, the format is "non-dealer pays / dealer pays."

### Common Scoring Table (Non-Dealer Ron)

| Han \ Fu | 30 | 40 | 50 | 60 | 70 |
|----------|-----|-----|-----|-----|-----|
| **1** | 1,000 | 1,300 | 1,600 | 2,000 | 2,300 |
| **2** | 2,000 | 2,600 | 3,200 | 3,900 | 4,500 |
| **3** | 3,900 | 5,200 | 6,400 | 7,700 | Mangan |
| **4** | 7,700 | Mangan | Mangan | Mangan | Mangan |

### Dora (ドラ / Bonus Tiles)
Dora tiles add +1 han each but do not count as yaku (a hand still needs at least one real yaku):
- **Omote Dora (表ドラ):** The dora indicator on the dead wall shows which tiles are dora. The dora tile is the NEXT tile in sequence (e.g., indicator shows 3m, dora is 4m; indicator shows 9m, dora is 1m; indicator shows North, dora is East; indicator shows Chun, dora is Haku).
- **Ura Dora (裏ドラ):** Revealed only by riichi winners. The tile beneath each dora indicator is flipped, and its corresponding "next" tile is an additional dora.
- **Kan Dora (カンドラ):** Each Kan declaration flips an additional dora indicator, adding more dora to the game.
- **Red Dora (赤ドラ):** Red five tiles are permanent dora (+1 han each).

## Yaku List

### 1 Han Yaku

| Yaku | Japanese | Closed/Open | Description |
|------|----------|-------------|-------------|
| **Riichi** | 立直 | Closed only | Declare riichi when tenpai with a closed hand |
| **Ippatsu** | 一発 | Closed only | Win within one turn cycle after declaring riichi (before any calls interrupt) |
| **Menzen Tsumo** | 門前清自摸和 | Closed only | Win by self-draw with a fully closed hand |
| **Pinfu** | 平和 | Closed only | All sequences, valueless pair, two-sided wait |
| **Tanyao** | 断么九 | Either (open tanyao allowed in most rule sets) | Hand contains only simple tiles (2-8), no terminals or honors |
| **Iipeikou** | 一盃口 | Closed only | Two identical sequences in the same suit |
| **Yakuhai** | 役牌 | Either | Triplet or quad of: any dragon, seat wind, or round wind. Each qualifying set is 1 han |
| **Haitei** | 海底摸月 | Either | Win on the very last tile drawn from the wall |
| **Houtei** | 河底撈魚 | Either | Win on the very last discard of the hand |
| **Rinshan Kaihou** | 嶺上開花 | Either | Win on the replacement tile drawn after a Kan |
| **Chankan** | 槍槓 | Either | Win by Ron on a tile added to an open triplet (Shouminkan) |

### 2 Han Yaku

| Yaku | Japanese | Closed/Open | Description |
|------|----------|-------------|-------------|
| **Double Riichi** | ダブル立直 | Closed only | Riichi declared on the player's very first discard (before any calls) |
| **Chanta** | 混全帯么九 | 2 closed / 1 open | Every mentsu and the pair contains a terminal or honor |
| **Sanshoku Doujun** | 三色同順 | 2 closed / 1 open | Same sequence (e.g., 1-2-3) in all three suits |
| **Ittsu** | 一気通貫 | 2 closed / 1 open | 1-2-3, 4-5-6, 7-8-9 all in the same suit |
| **Toitoi** | 対々和 | Either | All four mentsu are triplets (or quads) |
| **Sanankou** | 三暗刻 | Either | Three closed triplets (the fourth may be open) |
| **Sanshoku Doukou** | 三色同刻 | Either | Same-number triplet in all three suits |
| **Sankantsu** | 三槓子 | Either | Three quads |
| **Chiitoitsu** | 七対子 | Closed only | Seven distinct pairs |
| **Honroutou** | 混老頭 | Either | Only terminals and honors (all triplets + pair, or seven pairs) |
| **Shousangen** | 小三元 | Either | Two dragon triplets + dragon pair |
| **Double Yakuhai** | — | Either | Two separate qualifying yakuhai sets (e.g., two different dragon triplets) |

### 3 Han Yaku

| Yaku | Japanese | Closed/Open | Description |
|------|----------|-------------|-------------|
| **Honitsu** | 混一色 | 3 closed / 2 open | Only one suit plus honor tiles |
| **Junchan** | 純全帯么九 | 3 closed / 2 open | Every mentsu and pair contains a terminal (no honors) |
| **Ryanpeikou** | 二盃口 | Closed only | Two sets of identical sequences (effectively like chiitoitsu made of sequences) |

### 6 Han Yaku

| Yaku | Japanese | Closed/Open | Description |
|------|----------|-------------|-------------|
| **Chinitsu** | 清一色 | 6 closed / 5 open | Entire hand is one suit only (no honors) |

### Yakuman (役満 / Limit Hands)

| Yakuman | Japanese | Description |
|---------|----------|-------------|
| **Kokushi Musou** | 国士無双 (Thirteen Orphans) | One of each terminal and honor (13 tiles) + any duplicate |
| **Suuankou** | 四暗刻 (Four Closed Triplets) | Four concealed triplets + pair |
| **Daisangen** | 大三元 (Big Three Dragons) | Triplets of all three dragons |
| **Shousuushii** | 小四喜 (Little Four Winds) | Triplets of three winds + pair of the fourth wind |
| **Daisuushii** | 大四喜 (Big Four Winds) | Triplets of all four winds (double yakuman in some rules) |
| **Tsuuiisou** | 字一色 (All Honors) | Entire hand is honor tiles only |
| **Chinroutou** | 清老頭 (All Terminals) | Entire hand is terminal tiles only (1s and 9s) |
| **Ryuuiisou** | 緑一色 (All Green) | Hand composed only of 2s, 3s, 4s, 6s, 8s of souzu and Hatsu |
| **Chuuren Poutou** | 九蓮宝燈 (Nine Gates) | 1-1-1-2-3-4-5-6-7-8-9-9-9 of one suit + any tile of that suit |
| **Suukantsu** | 四槓子 (Four Quads) | Four quads declared |
| **Tenhou** | 天和 (Heavenly Hand) | Dealer wins on initial 14-tile deal |
| **Chiihou** | 地和 (Earthly Hand) | Non-dealer wins on first draw (before any calls) |

### Nagashi Mangan (流し満貫 / Special Draw)
If the hand ends in an exhaustive draw and a player's entire discard river consists only of terminals and honors, and none of their discards were called by other players, they receive Mangan payment.

## Special Rules

### Abortive Draws (途中流局)
The hand may be aborted (redealt) under these conditions:
- **Kyuushu Kyuuhai (九種九牌):** On a player's first turn (before any calls), if they have 9 or more different terminal/honor tiles, they may declare an abortive draw.
- **Suufon Renda (四風連打):** All four players discard the same wind tile on their first turn.
- **Suucha Riichi (四家立直):** All four players declare riichi.
- **Suukaikan (四開槓):** Four kans are declared across different players (not by one player).
- **Sanchahou (三家和):** Three players declare Ron on the same discard.

### Honba (本場 / Repeat Counter)
Each renchan (dealer repeat) adds a 300-point bonus per honba counter (100 per honba from each non-winner in tsumo, or 300 from the Ron victim). The honba count resets when a non-dealer wins.

### Uma (ウマ / Placement Bonus)
At the end of the game, point adjustments are made based on final placement. Common: +30/+10/-10/-30 (or +20/+10/-10/-20).

### Oka (オカ / Starting Point Bonus)
If the starting point is 25,000 but the target is 30,000, the difference (5,000 per player = 20,000 total) goes to the first-place finisher.

## Key Strategic Concepts

### Tile Efficiency (牌効率 / Hai Kouritsu)
Maximizing the number of tiles that improve your hand (acceptance count) drives early-hand decisions. Discard tiles that keep the most pathways to tenpai open.

### Defense (守備 / Shuubi)
When an opponent declares riichi or appears close to winning, defensive play becomes critical. Safe tiles include: tiles the tenpai player has already discarded (suji/sujiyomi reasoning), honor tiles already visible, and tiles within the opponent's discard patterns.

### Push/Pull (押し引き / Oshi-Hiki)
The decision to continue building your hand aggressively (push) vs. abandoning your hand to play defensively (pull) depends on: your hand value, remaining tiles in the wall, number of riichi declarations, and your current point standing.

### Hand Value Assessment
A cheap hand (1 han 30 fu = 1,000 points) may not justify the risk of dealing into an opponent's expensive hand. Aim for hands worth Mangan or higher when possible.

### Dora Usage
Dora tiles massively increase hand value but do not provide yaku. Build hands that incorporate dora while maintaining at least one yaku.

### Riichi Judgment
Riichi adds han, grants ippatsu chance, and reveals ura-dora, but locks your hand and announces tenpai to opponents. Declare riichi when your wait is good (many remaining tiles) and your hand benefits from the extra han.

### Furiten Awareness
Track your own discards meticulously. Discarding a winning tile early makes that entire wait unavailable via Ron for the rest of the hand.

## Common Terminology

| Term (Japanese) | Romaji | Definition |
|----------------|--------|------------|
| 麻雀 | Mahjong | The game itself |
| 立直 | Riichi | Ready declaration; also the name of the game variant |
| 和了 | Agari | Winning / completing a hand |
| ツモ | Tsumo | Self-draw (win by drawing your own tile) |
| ロン | Ron | Win by claiming another player's discard |
| 鳴き | Naki | Calling / opening your hand (Chi, Pon, Kan) |
| チー | Chi | Sequence call (from the player to your left only) |
| ポン | Pon | Triplet call (from any player) |
| カン | Kan | Quad declaration |
| 聴牌 | Tenpai | One tile away from a complete hand |
| 振聴 | Furiten | "Sacred discard" — cannot Ron when a winning tile is in your discards |
| 翻 | Han | Doublings (from yaku and dora) |
| 符 | Fu | Base points (from hand composition) |
| ドラ | Dora | Bonus tiles that add han |
| 裏ドラ | Ura-dora | Hidden dora revealed after riichi win |
| 役 | Yaku | Scoring pattern / qualifying combination |
| 役満 | Yakuman | Maximum-value hand |
| 満貫 | Mangan | 5-han limit hand (8,000 / 12,000 points) |
| 親 | Oya | Dealer (East position) |
| 子 | Ko | Non-dealer |
| 連荘 | Renchan | Dealer repeat (extra hand) |
| 本場 | Honba | Repeat counter adding bonus points |
| 河 | Kawa | Discard river (a player's discards in order) |
| 王牌 | Wanpai | Dead wall (14 tiles reserved for dora and kan draws) |
| 半荘 | Hanchan | Half-game (East + South rounds) |
| 東風戦 | Tonpuusen | East-only game (4 hands minimum) |
| 流局 | Ryuukyoku | Exhaustive draw |
| 点棒 | Tenbou | Point sticks |

## State Space Analysis

### Information Sets
- Tile combinations: C(136,13) for initial hand ≈ 10^16.
- With suit isomorphism and tile equivalences: reduced but still enormous.
- Observable information: own hand, all discards (public), called melds (public), dora indicators.
- Hidden information: other players' hands, remaining wall tiles.
- Information set complexity: estimated 10^48 or greater (Suphx paper estimates).

### Game Tree Size
| Metric | Estimate |
|--------|----------|
| Distinct tile hands (13 tiles) | ~10^16 (raw), ~10^9 (strategic equivalence classes) |
| Game tree nodes per hand | ~10^20-10^30 |
| Information sets per hand | ~10^15-10^20 |
| Full game (East+South) tree | Intractable (sequence of 8+ hands) |
| Branching factor | ~34 (discard choice) × claiming decisions |

### Action Space
- **Discard choice**: choose 1 of 14 tiles to discard (13 hand + 1 drawn).
  - 34 distinct tiles, but typically 10-14 unique options per turn.
- **Claim decisions**: chi, pon, kan, ron — each binary (yes/no) when applicable.
- **Riichi declaration**: binary when eligible.
- **Kan declaration**: when holding 4 identical tiles, optionally declare.
- Effective branching factor: ~14 (discard) + binary claim decisions.

## Key Challenges for AI/Solver Approaches

### 1. Four-Player Non-Cooperative Game
4-player mahjong is strictly non-zero-sum when considering placement bonuses (uma). Even with fixed total points, the payoff structure (placement-based) creates complex multi-agent dynamics:
- Implicit cooperation against the leader.
- Risk management relative to placement.
- Different incentives for 1st vs 2nd vs 3rd vs 4th.

### 2. Enormous Hidden Information
Each player has 13 hidden tiles. With 4 players, 39 tiles are hidden (plus the remaining wall). Reasoning about opponents' hands requires sophisticated probabilistic inference.

### 3. Discard Reading (Yomitsuki)
Expert mahjong players read opponents' discards to infer:
- What tiles they're likely holding.
- What they're likely waiting for (tenpai patterns).
- Whether they've declared riichi (and what wait pattern is likely).
This is a complex inference problem requiring integration of multiple signals.

### 4. Defensive Play (Betaori)
When an opponent declares riichi or appears dangerous:
- Which tiles are "safe" to discard?
- Trading off offense (continuing to build hand) vs defense (avoiding deal-in).
- Suji (numerical deduction) and kabe (wall/barrier) techniques for safety analysis.

### 5. Long-Term Game Strategy
A full game spans 8+ hands. Optimal play changes based on:
- Point standing relative to other players.
- Remaining hands in the game.
- Proximity to placement thresholds.
- Dealer advantages (can repeat if winning).

### 6. Dora and Red Dora
Dora tiles add significant value (1 han each). Strategy must account for:
- Dora indicator reveals.
- Red dora in hand composition.
- Risk of discarding dora (may enable opponent's high-scoring hand).

## Known Solver Results

### Suphx (Microsoft Research Asia, 2020)
The landmark AI result for Riichi Mahjong:
- Achieved stable 10-dan on Tenhou (top ~0.002% of players, above most professional players).
- Architecture: deep reinforcement learning with 5 oracle-guided models:
  1. Discard model.
  2. Riichi model.
  3. Chi model.
  4. Pon model.
  5. Kan model.
- Key innovations:
  - **Oracle guiding**: during training, uses "oracle features" (knowledge of hidden tiles) to guide policy learning, then removes oracle features at test time.
  - **Global reward prediction**: predicts game-level outcomes (placement), not just hand-level.
  - **Run-length encoding of game state**: efficient representation of the 136-tile state.
- Published at *Nature* (Li et al., "Suphx: Mastering Mahjong with Deep Reinforcement Learning," 2020).

### NAGA (dwango/N lab, 2021+)
- Japanese commercial mahjong AI.
- Reported performance comparable to or exceeding Suphx on Tenhou.
- Uses supervised learning on professional player data plus reinforcement learning fine-tuning.
- Available as a subscription service for post-game analysis.

### Mortal (Open Source, 2023+)
- Open-source mahjong AI achieving high-level play.
- Based on deep RL with Monte Carlo sampling.
- Available on GitHub, providing a reproducible baseline.

### Earlier Work
- **Bakuuchi** (various Japanese researchers): early rule-based and statistical mahjong AI.
- **Mizukami et al. (2015)**: Monte Carlo simulation for mahjong decision-making.
- **Kaneko (2017)**: supervised learning from professional games.

## Key Papers

| Year | Paper | Contribution |
|------|-------|--------------|
| 2015 | Mizukami & Tsuruoka, "Building a Computer Mahjong Player Based on Monte Carlo Simulation" | MC-based mahjong AI |
| 2017 | Kaneko, "Evaluation Functions for Mahjong" | Feature-based evaluation |
| 2020 | Li et al., "Suphx: Mastering Mahjong with Deep Reinforcement Learning" | Superhuman mahjong AI (*Nature*) |
| 2021 | Zhao et al., "Mahjong AI: Algorithmic Foundations" | Survey of mahjong AI approaches |
| 2023 | Mortal open-source project | Reproducible high-level mahjong AI |

## Relevance to Myosu

### Solver Applicability
Riichi Mahjong is a **tier-1 challenge** for imperfect-information game solving:
- **CFR**: not directly applicable. 4-player, non-zero-sum (placement-based), enormous information sets.
- **Deep RL**: the proven approach (Suphx). Policy gradient methods with oracle guiding.
- **MCTS with information set sampling**: viable for real-time play, used in some Japanese implementations.
- **Supervised learning + RL fine-tuning**: NAGA's approach, leveraging large databases of professional play.
- **Hybrid approaches**: combining search (MCTS) with learned policies and value functions.

### Architecture Ranking Factors
| Factor | Score | Notes |
|--------|-------|-------|
| CFR applicability | 1/5 | 4-player non-zero-sum makes CFR unsuitable |
| Neural value network potential | 5/5 | Essential; proven by Suphx/NAGA |
| Abstraction necessity | 3/5 | Tile structure provides natural abstraction (suit isomorphism) |
| Real-time solving value | 4/5 | MCTS enhances RL policies |
| Transferability of techniques | 4/5 | Multi-agent RL transfers to other multiplayer games |

### Myosu Subnet Considerations
- **High market value**: Riichi Mahjong is immensely popular in Japan, with professional leagues (M-League), online platforms (Tenhou, Mahjong Soul), and growing global interest.
- **Compute requirements**: Suphx required significant GPU training time. Solver nodes need GPU access.
- **Evaluation methodology**: Tenhou dan ranking provides an objective evaluation framework. Alternatively, round-robin tournaments with Elo rating.
- **Game oracle**: tile wall generation, scoring calculation (han/fu), and payment computation are non-trivial to implement correctly. The yaku recognition engine alone is a significant piece of software.
- **Rule set standardization**: Tenhou rules vs WRC rules vs M-League rules differ in details (red dora, uma, oka, etc.). Must specify one canonical rule set.
- **Training data availability**: large databases of high-level games available from Tenhou and other platforms.

### Recommended Approach for Myosu
1. Use deep RL as the primary solving approach (following Suphx architecture).
2. Supplement with MCTS for search-enhanced real-time play.
3. Use Tenhou-equivalent evaluation (simulated games against diverse opponents) for strategy quality assessment.
4. Implement a complete scoring engine as the oracle — this is a prerequisite and a substantial engineering effort.
5. Consider Riichi Mahjong as the flagship non-poker game — it has the largest player base and the most sophisticated existing AI results.

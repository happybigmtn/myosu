# Multi-Game Architecture Proof via Liar's Dice

Status: Completed 2026-03-29.

This ExecPlan is a living document. Maintained per `genesis/PLANS.md`.

Provenance: New plan. Stage-0 exit criteria require "Liar's Dice validates multi-game architecture." No prior genesis plan covered this.

## Purpose / Big Picture

Myosu claims to be a multi-game platform, but only NLHE poker has an engine. Stage-0 exit requires proving that a second game can be added without modifying existing code. This plan implements `myosu-games-liars-dice` as a minimal game engine that: implements the `CfrGame` trait, registers via `GameRegistry`, and can be trained and scored using the same pipeline as poker.

After this plan, `cargo test -p myosu-games-liars-dice` passes, the game is
registered in the registry, and the trait compliance proof demonstrates that a
second game can land as an additive crate without needing new edits to
`myosu-games` or `myosu-games-poker`.

## Progress

- [x] (2026-03-28) Confirmed Liar's Dice is in GameRegistry (GameType::LiarsDice) but no engine crate exists.
- [x] (2026-03-29) Created `myosu-games-liars-dice` with a minimal 2-player,
  1-die state machine, CFR-facing types, and crate-local tests.
- [x] (2026-03-29) Implemented `CfrGame` plus `CfrInfo`/`CfrPublic`/`CfrSecret`
  surfaces for the minimal Liar's Dice proof game.
- [x] (2026-03-29) Implemented a basic `GameRenderer` for shell/TUI integration
  and verified the crate with `cargo test` and `clippy`.
- [x] (2026-03-29) Added a minimal `LiarsDiceSolver` with executable MCCFR smoke
  proving training epochs, encounter population, and averaged opening-policy
  extraction in the new crate.
- [x] (2026-03-29) Added a downstream registry-resolution proof in
  `myosu-games-liars-dice` showing `GameType::LiarsDice` resolves through the
  shared `GameRegistry` without new edits to `myosu-games`.
- [x] (2026-03-29) Added crate-local exact best-response and exploitability
  scoring for the repeated-turn toy game instead of relying on the generic
  robopoker path that panicked here.
- [x] (2026-03-29) Closed the additive-boundary proof honestly: the Liar's
  Dice crate still resolves through the shared registry, depends on
  `myosu-games`, and does not depend on `myosu-games-poker`; the branch-level
  "zero modifications total" wording was retired because earlier shared-crate
  edits already existed before this slice.

## Surprises & Discoveries

- Observation: the retained plan overstated how much existing code needed to
  move for the first slice. `GameType::LiarsDice` and registry descriptions were
  already present in `myosu-games`, so the first honest proof surface was an
  additive crate plus workspace membership rather than any registry surgery.
  Evidence: `crates/myosu-games/src/traits.rs` and `crates/myosu-games/src/registry.rs`
  already carry `GameType::LiarsDice` and a built-in descriptor.
- Observation: the original "zero modifications" proof command is not honest on
  the current branch because `myosu-games` already contains earlier plan-006
  edits unrelated to this slice.
  Evidence: `git diff --stat -- crates/myosu-games crates/myosu-games-poker`
  reports preexisting changes in `myosu-games`, so the right near-term claim is
  "this slice added a new crate without needing further edits there," not "HEAD
  is clean relative to those crates."
- Observation: the first attempt at a convergence proof was too ambitious for
  the current generic robopoker exploitability path. Calling
  `Solver::exploitability()` on the repeated-turn toy game panics inside
  `Profile::external_evalue()` when control returns to the hero later in the
  subtree.
  Evidence: `cargo test -p myosu-games-liars-dice --quiet` failed with `BR is
  available Edge` from `rbp_mccfr` until the proof surface was narrowed to
  epochs, encounter population, and policy extraction.
- Observation: the scoring gap did not require reopening `rbp_mccfr`. The toy
  game's information structure is small enough that a crate-local exact
  best-response evaluator can score the averaged policy directly from the new
  game crate.
  Evidence: `LiarsDiceSolver` now computes exact best-response values and
  exploitability with fresh passing tests in `myosu-games-liars-dice`.
- Observation: the last open proof item was really about the additive boundary,
  not a pristine shared-crate git state the current branch no longer has.
  Evidence: `myosu-games-liars-dice/Cargo.toml` depends on `myosu-games` but
  not `myosu-games-poker`, the crate-local manifest regression now locks that
  boundary, and `cargo tree -p myosu-games-liars-dice` stays free of any poker
  crate edge.

## Decision Log

- Decision: Implement 2-player, 1-die, 6-face Liar's Dice as the minimal variant.
  Rationale: Smallest meaningful game tree for MCCFR. Full Liar's Dice (multiple dice) is exponentially larger and unnecessary for architecture proof.
  Inversion: If we implement full multi-dice, training time dominates and the architecture proof is obscured by solver complexity.
  Date/Author: 2026-03-28 / Genesis

- Decision: Liar's Dice crate depends only on myosu-games (traits) and rbp-mccfr (MCCFR engine). It does NOT depend on myosu-games-poker.
  Rationale: Proves that new games don't need poker-specific code. Clean dependency boundary.
  Date/Author: 2026-03-28 / Genesis

- Decision: The first plan-012 slice stops at additive crate proof, CFR-facing
  type fit, and basic renderer proof rather than jumping straight to MCCFR
  convergence.
  Rationale: That is the fastest honest way to validate the architecture seam.
  Solver convergence can follow once the second game's trait fit is proven.
  Date/Author: 2026-03-29 / Genesis

- Decision: Treat the second plan-012 solver slice as MCCFR training proof, not
  exploitability proof.
  Rationale: The current repeated-turn toy game can be trained and queried
  honestly, but the generic exploitability path in `rbp_mccfr` is not yet a
  truthful executable proof for this state machine.
  Date/Author: 2026-03-29 / Genesis

- Decision: Add a crate-local exact exploitability routine instead of trying to
  force the generic robopoker path to fit this toy game immediately.
  Rationale: The state space is tiny, the information boundary is explicit, and
  the resulting proof is additive to `myosu-games-liars-dice` rather than
  entangling plan 012 with upstream solver surgery.
  Date/Author: 2026-03-29 / Genesis

## Failure Modes

| Codepath | Realistic failure | Handling |
|---|---|---|
| CfrGame trait impl | Trait requires methods that are poker-specific | If so, the trait design needs fixing -- this IS the proof |
| MCCFR training | Training runs but exploitability proof panics on repeated hero turns | Keep the stage-0 proof at epochs, encounter population, and averaged policy until the upstream scoring path is adapted honestly |
| GameRenderer impl | Renderer trait assumes poker-specific state | If so, generalize the trait (this is a valid finding) |
| Registry resolution | Custom game type doesn't resolve correctly | Fix registry -- extensibility must work |

## Outcomes & Retrospective

The first plan-012 slice is real now. `myosu-games-liars-dice` exists as a new
additive crate with a compact state machine, `CfrGame`/`CfrInfo`-compatible
types, and a basic `GameRenderer` implementation. It started as the first
executable answer to the question "can a second game land without reaching back
into poker code?" and it now finishes that proof on the honest branch surface:
yes, at the crate/interface layer, without adding a poker dependency.

The second slice made that proof operational rather than merely structural.
`LiarsDiceSolver` now executes MCCFR training in the new crate and exposes
averaged policy extraction for a real Liar's Dice information set. The
important correction was narrowing the claim when the generic exploitability
path proved untruthful for this repeated-turn toy game, then re-expanding it in
an honest way with a crate-local exact scorer. Stage 0 now has both a solver
smoke and an executable exploitability score for the second game, without
pretending the generic robopoker path already handled this variant.

The registry side is now executable too. Instead of reopening `myosu-games`,
the new crate proves from downstream that `liars_dice` still resolves through
the shared `GameRegistry` as a built-in 2-player game with the expected
descriptor. That keeps the multi-game proof additive: the second game crate can
validate the shared registry seam without needing another round of shared-crate
surgery. The final closure was to make the additive boundary itself executable:
the crate now carries a manifest regression that proves it still depends on
`myosu-games` and does not depend on `myosu-games-poker`, while `cargo tree`
confirms the same dependency shape externally. That replaces the earlier
branch-cleanliness fiction with a truthful boundary proof and closes plan 012.

## Context and Orientation

Liar's Dice is a bluffing game. In the minimal variant (2 players, 1 die each, 6 faces):
- Each player rolls a hidden die.
- Players take turns claiming how many dice of a face value exist between both players' dice.
- Each claim must be higher than the previous (either more dice or higher face value).
- A player can challenge ("liar!") instead of raising.
- If challenged: if the claim is true, challenger loses; if false, claimant loses.

Game tree: ~100 terminal nodes. Information sets: ~50. Perfect for MCCFR convergence proof.

```text
GAME STATE MACHINE

  Roll phase          Bidding phase           Resolution
  +--------+         +----------+            +----------+
  | Each   |         | Player A |--bid------>| Player B |
  | player |-------->| or B     |            | or A     |
  | rolls  |         | bids     |<--bid------| bids     |
  | 1 die  |         |          |            |          |
  +--------+         |          |--challenge->| resolve  |
                      +----------+            +----------+
                                                   |
                                              claim true?
                                              /         \
                                         yes /           \ no
                                      challenger        claimant
                                        loses            loses
```

Files to create:
- `crates/myosu-games-liars-dice/Cargo.toml`
- `crates/myosu-games-liars-dice/src/lib.rs`
- `crates/myosu-games-liars-dice/src/game.rs` -- CfrGame implementation
- `crates/myosu-games-liars-dice/src/renderer.rs` -- GameRenderer implementation

Dependencies: `myosu-games` (traits), `rbp-core` and `rbp-mccfr` (MCCFR engine), `serde`, `ratatui` (renderer).

## Milestones

### Milestone 1: Game state machine

Implement Liar's Dice state: roll, bid, challenge, resolve. Define LiarsDiceAction, LiarsDiceState, and LiarsDiceInfo (information set).

Proof command:

    cargo check -p myosu-games-liars-dice

Status:
- Completed on 2026-03-29 via additive crate implementation and passing crate tests.

### Milestone 2: CfrGame trait implementation

Implement `CfrGame` for `LiarsDiceGame`: `root()`, `actions()`, `apply()`, `info()`, `is_terminal()`, `utility()`. Verify the trait bounds match what MCCFR expects.

Proof command:

    cargo test -p myosu-games-liars-dice cfr_game --quiet

Status:
- Completed on 2026-03-29 at the trait-fit level; solver convergence remains pending.

### Milestone 3: MCCFR training and scoring smoke

Train Liar's Dice with MCCFR, verify epochs advance, encounter tables populate,
and averaged policy can be extracted for a real opening information set. This
proves the trait implementation is executable under the shared solver engine.
Then compute exact best-response values and exploitability with a crate-local
scorer tailored to the tiny repeated-turn game.

Proof command:

    cargo test -p myosu-games-liars-dice --quiet

Status:
- Completed on 2026-03-29 as a truthful training/policy smoke rather than an
  exploitability-decrease proof, then extended the same day with a crate-local
  exact exploitability routine.

### Milestone 4: GameRenderer implementation

Implement `GameRenderer` for Liar's Dice with basic TUI display: current bids, player's die, legal actions.

Proof command:

    cargo test -p myosu-games-liars-dice renderer --quiet

Status:
- Completed on 2026-03-29 with a basic shell/TUI renderer and focused renderer tests.

### Milestone 5: Additive-boundary proof

Verify the honest boundary instead of a branch-cleanliness fiction: the
Liar's Dice crate must resolve through the shared registry, depend on
`myosu-games`, and avoid any `myosu-games-poker` dependency.

Proof command:

    cargo test -p myosu-games-liars-dice --quiet
    cargo tree -p myosu-games-liars-dice

Status:
- Completed on 2026-03-29 via the crate-local registry proof, manifest
  regression, and dependency-tree verification that the additive crate stays
  off the poker boundary.

## Plan of Work

1. Create crate with Cargo.toml and lib.rs.
2. Implement game state machine.
3. Implement CfrGame trait.
4. Implement GameRenderer.
5. Test MCCFR convergence.
6. Close the additive-boundary proof on the honest branch surface.

## Concrete Steps

From `/home/r/coding/myosu`:

    # Verify GameType::LiarsDice exists in registry
    grep -rn "LiarsDice" crates/myosu-games/src/

    # After creation:
    cargo test -p myosu-games-liars-dice --quiet

## Validation and Acceptance

Accepted when:
- Liar's Dice crate compiles and tests pass
- CfrGame trait implementation enables MCCFR training
- MCCFR training produces a usable averaged policy for at least one real info set
- The toy game can be scored via an executable exact exploitability routine
- GameRenderer displays game state in TUI
- The additive crate resolves through `myosu-games` and does not depend on
  `myosu-games-poker`

## Idempotence and Recovery

New crate is additive. If trait doesn't fit, the finding itself is valuable (trait needs generalization).

## Interfaces and Dependencies

Depends on: 006 (game trait boundaries locked).
Blocks: 013 (integration tests may include multi-game).

```text
myosu-games (traits)       rbp-mccfr (engine)
        \                    /
         v                  v
  myosu-games-liars-dice
  (CfrGame + GameRenderer)
         |
         v
  MCCFR training + exact scoring proof
  (epochs + encounters + policy + exploitability)
```

# Build Core Game Logic for the Canonical Ten

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

`PLANS.md` is checked into the repository root and this document must be maintained in accordance with it. This plan assumes `execplans/EXECPLAN_CANONICAL_GAME_TRUTH_LAYER.md` has landed the canonical type names used below.

## Purpose / Big Picture

After this plan is implemented, the canonical-ten portfolio games will have deterministic core game logic instead of only representative strategy distributions. A human or agent will be able to ask for a canonical state, inspect exhaustive legal actions, apply a canonical action, and verify the next state and payoff or continuation state for each of the ten games.

This is the missing step between "we can return a rule-aware recommendation" and "we can play and validate the game." It makes the future network bootstrap stronger because miners and validators can share not only a response format but a replayable game contract.

## Progress

- [x] (2026-04-10T00:00Z) Identified the canonical-ten set from the existing strong-engine reference-family plan: `nlhe-six-max`, `hanafuda-koi-koi`, `riichi-mahjong`, `bridge`, `gin-rummy`, `stratego`, `ofc-chinese-poker`, `dou-di-zhu`, `backgammon`, and `cribbage`.
- [x] (2026-04-10T00:00Z) Confirmed that Myosu currently has shared cards, rollout, and evaluator utilities plus rule-aware family modules, but not full deterministic state-transition engines for the canonical ten.
- [x] (2026-04-10T00:00Z) Add a reusable concrete dispatch API that returns canonical snapshots, legal actions, transitions, and terminal payoffs.
- [x] (2026-04-10T00:00Z) Implement bounded Bridge and Cribbage core state machines with illegal-action and deterministic-transition tests.
- [x] (2026-04-10T00:00Z) Implement bounded Backgammon, Gin Rummy, and OFC core state machines with illegal-action and deterministic-transition tests.
- [x] (2026-04-10T00:00Z) Implement bounded Dou Di Zhu, Hanafuda Koi-Koi, and Riichi Mahjong core state machines with illegal-action and deterministic-transition tests.
- [x] (2026-04-10T00:00Z) Wire eight canonical-ten games through the shared core dispatch API.
- [x] (2026-04-10T00:00Z) Implement bounded Stratego and NLHE 6-Max core state machines with illegal-action and deterministic-transition tests.
- [x] (2026-04-10T00:00Z) Implement the ten game cores behind that dispatch API.
- [x] (2026-04-10T00:00Z) Add per-game illegal-action, deterministic-transition, and payoff tests.
- [x] (2026-04-10T00:00Z) Add a canonical-ten core harness that derives its game list from Rust rather than a hard-coded shell list.
- [x] (2026-04-10T00:00Z) Wire the core logic into the canonical truth layer from `EXECPLAN_CANONICAL_GAME_TRUTH_LAYER.md`.
- [x] (2026-04-10T00:00Z) Extend the same bounded core surface to the remaining twelve research games: `nlhe-heads-up`, `plo`, `nlhe-tournament`, `short-deck`, `teen-patti`, `hwatu-go-stop`, `spades`, `liars-dice`, `pusoy-dos`, `tien-len`, `call-break`, and `hearts`.

## Surprises & Discoveries

- Observation: `plans/041026-strong-game-specific-engines.md` already created a practical first-ten cut by naming one reference game per major family. Reusing that cut is better than inventing a new arbitrary ten.
  Evidence: That plan's reference engines are Bridge, Dou Di Zhu, Riichi Mahjong, Hanafuda Koi-Koi, Gin Rummy, Stratego, Backgammon, Open Face Chinese Poker, Cribbage, and NLHE 6-Max.

- Observation: The current portfolio family modules are useful but too shallow to serve as replayable engines. For example, `crates/myosu-games-portfolio/src/engines/backgammon.rs` has a small `BackgammonRaceState` heuristic, and `crates/myosu-games-portfolio/src/engines/cribbage.rs` checks one three-card run. That is enough for a recommendation baseline, not enough for legal game play.
  Evidence: Those modules return `EngineAnswer` distributions and do not expose `apply(action) -> next_state`.

- Observation: The research rule files are detailed enough to define honest first core states, but many full games are unsolved or too large for exact solving. The plan should implement bounded, representative core logic with explicit limitations instead of pretending to solve full Bridge, Riichi, Stratego, or NLHE 6-max.
  Evidence: The rule files under `research/game-rules/` identify game tree sizes and solved status for these games.

## Decision Log

- Decision: Implement deterministic core logic in additive modules under `crates/myosu-games-portfolio/src/core/` first, not by replacing `crates/myosu-games-portfolio/src/engines/`.
  Rationale: The existing engines are already wired into miner, validator, play, and strength harnesses. A parallel core module lets auto-loop add replayability without breaking the current strategy surface.
  Date/Author: 2026-04-10 / Codex

- Decision: The first core state for each game may be a bounded representative scenario, but it must still enforce real legal actions and deterministic transitions for that scenario.
  Rationale: Full-game implementations for all ten are too large for one safe pass. A bounded state with honest rules is useful; a display-only placeholder is not.
  Date/Author: 2026-04-10 / Codex

- Decision: Core game logic must reject illegal actions with typed errors instead of panicking.
  Rationale: The play harness and validators need to prove illegal-action handling. Bitino's engine spec rejects invalid actions before settlement; Myosu's core game logic should do the same before strategy scoring.
  Date/Author: 2026-04-10 / Codex

## Outcomes & Retrospective

The core API and all ten bounded game cores are implemented, and the same family modules now cover all 22 research games. Bridge, Spades, Call Break, and Hearts share a deterministic trick-taking play state with follow-suit enforcement and trick resolution. Cribbage models a pegging scenario with count, fifteen, pair, and three-card-run scoring; full keep/discard hand scoring is deferred. Backgammon models bar-entry priority and bounded checker movement. Gin Rummy models discard-step legality with knock/gin eligibility. OFC models row capacity and terminal foul detection. Dou Di Zhu, Pusoy Dos, and Tien Len share a climbing state with pass control plus single/pair/straight/bomb legality. Hanafuda Koi-Koi and Hwatu Go-Stop share month-matching capture logic with yaku-gated continuation/stop actions. Riichi Mahjong models a discard decision with tenpai-gated riichi declaration. Stratego models a compact hidden-identity movement/combat scenario with commitment reveal, miner-versus-bomb, spy-versus-marshal, and rank comparison adjudication. The poker-like core now covers heads-up NLHE, NLHE 6-Max, PLO, tournament NLHE, short deck, and Teen Patti through bounded betting states. Liar's Dice now has a bounded bid/challenge state with deterministic terminal challenge resolution.

## Context and Orientation

The existing portfolio crate is `crates/myosu-games-portfolio`. It already has these helpful modules:

- `src/cards.rs` for standard decks, short decks, and Hanafuda modeling.
- `src/combinatorics.rs` for bounded iteration helpers.
- `src/rollout.rs` for deterministic sampling.
- `src/eval/poker.rs`, `src/eval/trick_taking.rs`, and `src/eval/meld.rs` for local evaluators.
- `src/engines/` for rule-aware strategy distributions.
- `src/protocol.rs` for `PortfolioAction`.
- `src/state.rs` for the compact `PortfolioChallenge` envelope.

This plan adds core game state machines. A state machine is code that knows the current phase, can enumerate every legal action, can apply one legal action to produce a next state, and can say whether the game is terminal. A terminal state is a state where the hand or scenario is complete and payoffs can be computed.

Use the term "bounded representative scenario" honestly. It means a small, deterministic subset of the game chosen to exercise legal actions and scoring without implementing the full real-world game. For example, a Bridge scenario may start in the play phase with a fixed three-card hand and a led suit; it should enforce following suit and trick winner logic in that scenario, but it need not implement every bidding convention.

## Plan of Work

Create a new module tree:

- `crates/myosu-games-portfolio/src/core/mod.rs`
- `crates/myosu-games-portfolio/src/core/model.rs`
- `crates/myosu-games-portfolio/src/core/poker_like.rs`
- `crates/myosu-games-portfolio/src/core/hanafuda.rs`
- `crates/myosu-games-portfolio/src/core/mahjong.rs`
- `crates/myosu-games-portfolio/src/core/trick_taking.rs`
- `crates/myosu-games-portfolio/src/core/gin_rummy.rs`
- `crates/myosu-games-portfolio/src/core/stratego.rs`
- `crates/myosu-games-portfolio/src/core/ofc.rs`
- `crates/myosu-games-portfolio/src/core/shedding.rs`
- `crates/myosu-games-portfolio/src/core/backgammon.rs`
- `crates/myosu-games-portfolio/src/core/cribbage.rs`

In `core/model.rs`, define a small dispatch API that can work with the canonical truth model:

- `CoreGameState` with `game: ResearchGame`, `phase: String`, `actor: Option<u8>`, `public_state: serde_json::Value`, `private_state_commitments: Vec<String>`, `legal_actions: Vec<CoreAction>`, `terminal: bool`, and `payoff: Option<Vec<i64>>`.
- `CoreAction` with `action_id: String`, `display_label: String`, and `params: serde_json::Value`.
- `CoreTransition` with `before: CoreGameState`, `action: CoreAction`, and `after: CoreGameState`.
- `CoreGameError` with variants for unsupported game, unknown action, illegal action, invalid params, and non-terminal payoff.
- `pub fn bootstrap_state(game: ResearchGame) -> Result<CoreGameState, CoreGameError>`.
- `pub fn legal_actions(state: &CoreGameState) -> &[CoreAction]`.
- `pub fn apply_action(state: &CoreGameState, action_id: &str, params: serde_json::Value) -> Result<CoreTransition, CoreGameError>`.

Implement `From<CoreGameState> for CanonicalStateSnapshot` and `From<CoreAction> for CanonicalActionSpec` conversions against the canonical truth layer types. Do not add a parallel local snapshot model in the portfolio crate.

Implement each game core as follows.

For `nlhe-six-max`, model a bounded six-seat hand state with blinds, stacks, hole cards represented by commitments, public board cards, pot, current bet, and action to the next player. Legal actions for the first scenario should include fold, call/check, and raise-to if stack and minimum raise allow it. The transition must update stack, committed amount, pot, and next actor deterministically. If the scenario reaches showdown in a later milestone, use `crates/myosu-games-portfolio/src/eval/poker.rs` for ranking. Do not call this heads-up robopoker; this is the portfolio 6-max core.

For `hanafuda-koi-koi`, model a small turn with hand cards, field cards, captured piles, draw pile commitment, and yaku state. Legal actions must include only captures that match a month on the field or a discard-to-field action when no capture exists. `koi-koi` and `stop-round` are legal only after the acting player has a supported yaku. Use `cards.rs` Hanafuda helpers and explicitly list supported yaku in tests.

For `riichi-mahjong`, model a discard decision with a 14-tile hand, visible discards, dora indicator, and danger metadata. Legal actions must include discard actions for tiles in the hand. `declare-riichi` is legal only if the supported shanten calculation says the hand is tenpai and the player has not already declared riichi. If full yaku scoring is not implemented, do not expose `ron` or `tsumo` as legal in the representative state.

For `bridge`, model a play-phase trick state with seat, partnership, trump denomination, led suit, current trick cards, and the acting hand. Legal actions must enforce following suit. Applying an action adds a card to the current trick, advances actor, and computes the trick winner when four cards are present. A bidding state can be deferred, but if a bidding state is added it must enforce increasing bids.

For `gin-rummy`, model a turn with hand, discard pile top, stock commitment, deadwood count, and draw source. Legal actions must include draw-stock, draw-discard, discard-card, knock, and gin only when the hand qualifies. Applying draw-discard then discarding the same card in the same turn must be rejected.

For `stratego`, model a compact board with coordinates, water squares, own pieces, public opponent piece identities where revealed, and unknown opponent commitments where hidden. Legal actions must reject water movement, immobile bomb/flag moves, and non-scout long moves. Combat must implement at least rank comparison, miner versus bomb, and spy attacking marshal. If a hidden opponent piece is attacked, reveal it in the public state after combat.

For `ofc-chinese-poker`, model the front, middle, and back rows, row capacities, dealt cards, and remaining draw commitment. Legal actions must place a dealt card into a row that has capacity. Applying a placement updates row state and, when all 13 cards are placed, marks the hand terminal and detects fouling if front beats middle or middle beats back. Use `eval/poker.rs` where possible.

For `dou-di-zhu`, model a climbing trick with current lead combination, actor hand, landlord role, pass count, and legal combination classes. Legal actions must include pass and any combination that matches and beats the current lead, plus bombs/rocket if supported. Applying two consecutive passes should return lead control to the last player who played. Keep Pusoy Dos and Tien Len out of this canonical-ten plan; their same-family differences are covered by the all-corpus strength harness.

For `backgammon`, model a compact race/contact state with 24 points, bar counts, borne-off counts, dice, cube owner/value, and actor. Legal actions must enumerate legal checker moves for the current dice, including bar entry before other moves and bearing off only when all checkers are in the home board. Doubling can be represented as a legal action only before a dice move and only if the cube is available.

For `cribbage`, model both a keep/discard decision and a pegging decision. The first implementation may choose one bootstrap state, but tests must cover scoring helpers for fifteens, pairs, and runs. In pegging, legal actions must reject any card that would push the running count over 31. Applying a legal card updates the running count and awards immediate pegging points.

After each module is implemented, wire it through `core/mod.rs` dispatch and expose a helper example:

- `crates/myosu-games-portfolio/examples/core_manifest.rs`
- `crates/myosu-games-portfolio/examples/core_roundtrip.rs`

`core_manifest` should print one line for every canonical-ten game. `core_roundtrip` should accept a game slug, choose the highest-probability current portfolio action if it can decode to a core action, apply it, and print a deterministic result.

## Concrete Steps

From the repository root, start by confirming current portfolio tests pass or record existing failures:

    SKIP_WASM_BUILD=1 cargo test --quiet -p myosu-games-portfolio
    SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-portfolio --example strength_manifest -- table

Add `pub mod core;` to `crates/myosu-games-portfolio/src/lib.rs` only after the module compiles.

Implement `core/model.rs` first and write tests that use a tiny fake state/action without game-specific rules. This keeps the dispatch error handling stable before game complexity arrives.

Then implement one game at a time in this order:

1. `bridge`, because follow-suit legality is already supported by `eval/trick_taking.rs`.
2. `cribbage`, because scoring a small run and pegging cap is locally bounded.
3. `backgammon`, because its compact race-state current module is small.
4. `gin-rummy`, because `eval/meld.rs` already exists.
5. `ofc-chinese-poker`, because row-capacity and foul detection are self-contained.
6. `dou-di-zhu`, because combination comparison can be added without touching other shedding games.
7. `hanafuda-koi-koi`, because card modeling exists but yaku legality must be explicit.
8. `riichi-mahjong`, because shanten and riichi legality are easy to overclaim.
9. `stratego`, because hidden identity and combat need careful public/private split.
10. `nlhe-six-max`, because betting transitions and multiway action order touch the most poker-specific assumptions.

For each game, add tests named in this shape:

    core::<game>::tests::<game>_bootstrap_state_has_legal_actions
    core::<game>::tests::<game>_rejects_illegal_action
    core::<game>::tests::<game>_transition_is_deterministic
    core::<game>::tests::<game>_terminal_payoff_is_valid_when_terminal

Add examples after at least Bridge and Cribbage are green, then keep extending the examples as each game is wired.

Expected command shape after implementation:

    SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-portfolio --example core_manifest
    SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-portfolio --example core_roundtrip -- bridge

Expected output shape:

    CORE_GAME slug=bridge phase=play actor=0 actions=2 state=ok
    CORE_ROUNDTRIP game=bridge action_id=bridge.play.double_dummy terminal=false payoff=none transition=ok

## Validation and Acceptance

This plan is accepted when:

- `crates/myosu-games-portfolio/src/core/` exists and is exported.
- Every canonical-ten game has a bootstrap core state with at least one legal action.
- Every canonical-ten game rejects at least one illegal action before transition.
- Every canonical-ten game has a deterministic transition test that applies the same action twice from the same state and gets the same next state.
- Terminal payoff tests exist for games that can reach terminal in the bounded scenario; non-terminal games clearly return `payoff=None` instead of fabricating a score.
- The `core_manifest` example prints exactly ten `CORE_GAME` lines.
- The `core_roundtrip` example runs for every canonical-ten slug.
- The canonical truth layer can convert each core state to a canonical snapshot.

Run this validation set from the repository root:

    SKIP_WASM_BUILD=1 cargo test --quiet -p myosu-games-portfolio core
    SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-portfolio --example core_manifest
    for game in nlhe-six-max hanafuda-koi-koi riichi-mahjong bridge gin-rummy stratego ofc-chinese-poker dou-di-zhu backgammon cribbage; do SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-portfolio --example core_roundtrip -- "$game"; done
    SKIP_WASM_BUILD=1 cargo test --quiet -p myosu-games-portfolio
    SKIP_WASM_BUILD=1 cargo clippy -p myosu-games-portfolio --all-targets -- -D warnings
    cargo fmt --check -p myosu-games-portfolio

The final loop must not print `unsupported`, `static-baseline`, `panic`, or `illegal action accepted`.

## Idempotence and Recovery

Keep the existing `src/engines/` strategy path green while adding `src/core/`. If a game core becomes large or blocks the rest of the plan, keep the prior games committed and record a partial implementation note in `Progress`, `Surprises & Discoveries`, and the module docs for that game.

Generated example output should write only to stdout unless an output directory is explicitly passed. If an example needs artifacts, write them under `target/e2e/core-<game>/`.

If any rule implementation is uncertain, implement the narrower rule and state the limitation in code comments, tests, and this plan. Do not silently accept illegal actions to make a harness pass.

## Interfaces and Dependencies

Prefer existing helpers in `crates/myosu-games-portfolio/src/cards.rs`, `eval/`, `rollout.rs`, and `combinatorics.rs`. Add new dependencies only if they are deterministic, offline, and materially reduce rule complexity.

The public operator interfaces from the prior solver plan should stay stable:

- `myosu-miner --game <slug> --query-file ... --response-file ...`
- `myosu-validator --game <slug> --checkpoint ... --query-file ... --response-file ...`
- `myosu-play --game <slug> --smoke-test`

This plan adds core examples and internal APIs; it should not break miner, validator, or play CLIs.

## Artifacts and Notes

The important artifacts are:

- `crates/myosu-games-portfolio/src/core/`
- `crates/myosu-games-portfolio/examples/core_manifest.rs`
- `crates/myosu-games-portfolio/examples/core_roundtrip.rs`
- Core tests for all ten games
- Ten `CORE_ROUNDTRIP` outputs

Change note, 2026-04-10 / Codex: Initial plan written to turn the existing canonical-ten rule-aware reference engines into deterministic state-transition cores.

Change note, 2026-04-10 / Codex: Implemented the ten bounded core engines plus `core_manifest` and `core_roundtrip`; the Phase 2 core logic gate passes locally.

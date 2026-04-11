# Build a Canonical Game Truth Layer

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

`PLANS.md` is checked into the repository root and this document must be maintained in accordance with it. The reader should be able to start from only this file and the current working tree.

## Purpose / Big Picture

After this plan is implemented, Myosu will have one canonical game truth layer for decision games in the same spirit that Bitino has one canonical wager truth layer for casino games. A strategy query, a validator challenge, a renderer button, a pipe-mode action, and a replay log will all agree on the same game id, state id, and legal action id before any solver-specific code runs. The transition record shape can be defined here, but real transition records are produced after the core game logic plan lands deterministic `apply_action()` implementations.

The user-visible result is not a new game by itself. The result is that an operator or agent can run a canonical registry and snapshot command, see the canonical-ten game catalog, generate a canonical snapshot and strategy binding for a representative decision in each game, and verify that the strategy response and rendered legal actions came from the same typed state rather than from duplicated string labels. Full transition traces are produced by the core game logic and play harness plans after `apply_action()` exists.

## Progress

- [x] (2026-04-10T00:00Z) Researched Bitino's active canonical wager design in `/home/r/Coding/bitino/genesis`, `/home/r/Coding/bitino/execplans`, `/home/r/Coding/bitino/specs/100426-canonical-wager-system.md`, and `/home/r/Coding/bitino/crates/bitino-canonical`.
- [x] (2026-04-10T00:00Z) Compared that design with Myosu's current `myosu-games`, `myosu-games-portfolio`, `myosu-play`, miner, validator, and strength harness surfaces.
- [x] (2026-04-10T00:00Z) Decided that the first canonical truth pass should not replace the existing portfolio strategy surface; it should wrap and verify it.
- [x] (2026-04-10T00:00Z) Add canonical game truth types to `crates/myosu-games/src/canonical.rs`.
- [x] (2026-04-10T00:00Z) Add a registry/adapter crate at `crates/myosu-games-canonical`.
- [x] (2026-04-10T00:00Z) Wire canonical snapshots and strategy bindings through portfolio strength queries for the canonical ten.
- [x] (2026-04-10T00:00Z) Add registry, roundtrip, and drift-guard tests.
- [ ] Update docs and harnesses so future game implementations use canonical action/state IDs instead of local display strings.

## Surprises & Discoveries

- Observation: Bitino's useful pattern is not its casino domain; it is the shape of the boundary. `bitino-canonical` owns wager ids, typed params, snapshots, board/stream projections, exposure, wire helpers, and proof linkage. Myosu needs the same boundary for game decisions and strategy responses.
  Evidence: `/home/r/Coding/bitino/specs/100426-canonical-wager-system.md` says each game module exports a registry, snapshot, projection, wire, engine payload, and fairness dossier. `/home/r/Coding/bitino/crates/bitino-canonical/src/model.rs` defines shared `CanonicalWagerSpec`, `LiveContract`, `CanonicalExposure`, and trace ids.

- Observation: Myosu already has a broad all-corpus strategy-response adapter, but that adapter is not yet a deterministic game engine truth layer. `crates/myosu-games-portfolio/src/protocol.rs` defines `PortfolioAction`; `crates/myosu-games-portfolio/src/state.rs` defines a compact `PortfolioChallenge`; and `crates/myosu-games-portfolio/src/engine.rs` returns a strategy distribution. None of those currently records a replayable state transition or terminal payoff for the canonical ten.
  Evidence: `tests/e2e/research_strength_harness.sh` proves rule-aware strategy outputs, while `crates/myosu-games-portfolio/src/engines/*.rs` mostly return fixed or heuristic action distributions.

- Observation: The active repository has substantial unrelated dirty work from the prior all-corpus solver pass. This plan must be additive and must not revert files outside `execplans/` while it is being authored.
  Evidence: `git status --short` before this file was written showed modified game, miner, validator, play, spec, and harness files plus deleted historical plan files.

## Decision Log

- Decision: Use "canonical game truth" to mean the typed game id, ruleset version, public state snapshot, private-state commitment hashes, legal action specs, chosen action, transition result, strategy response binding, and validation metric for one decision or hand.
  Rationale: Bitino's word "wager" is casino-specific. Myosu's analogous unit is a decision/action in an imperfect-information game.
  Date/Author: 2026-04-10 / Codex

- Decision: Put shared data types in `crates/myosu-games/src/canonical.rs`, and put the cross-game registry/adapters in a new `crates/myosu-games-canonical` crate.
  Rationale: `crates/myosu-games` is already the common low-level crate used by game crates. If it imports every game crate, it will create dependency cycles. A separate aggregator crate can depend on the dedicated game crates and the portfolio crate without forcing those crates to depend on the aggregator.
  Date/Author: 2026-04-10 / Codex

- Decision: The first canonical migration batch is the canonical ten portfolio games from `plans/041026-strong-game-specific-engines.md`: `nlhe-six-max`, `hanafuda-koi-koi`, `riichi-mahjong`, `bridge`, `gin-rummy`, `stratego`, `ofc-chinese-poker`, `dou-di-zhu`, `backgammon`, and `cribbage`.
  Rationale: These ten were already chosen as one reference game per major family. They are the best initial target for auto-loop work because they maximize rules coverage without forcing all 22 research games into the first canonical truth migration.
  Date/Author: 2026-04-10 / Codex

- Decision: Keep heads-up NLHE, Liar's Dice, and Kuhn on their dedicated crates during this plan.
  Rationale: They already have dedicated solver and roundtrip surfaces. The immediate missing layer is the portfolio canonical truth layer; dedicated-game canonical adapters can be added after the shape is proven.
  Date/Author: 2026-04-10 / Codex

## Outcomes & Retrospective

Phase 1 code is implemented. `myosu-games` now owns the shared canonical structs and hash helpers, while `myosu-games-canonical` owns the canonical-ten registry, portfolio strength-query adapters, `canonical_manifest`, and `canonical_snapshot` examples. All ten canonical-ten games produce `CANONICAL_SNAPSHOT` output against the existing rule-aware portfolio strength route.

## Context and Orientation

Bitino is the reference for the pattern. In Bitino, `crates/bitino-canonical/src/lib.rs` declares `PHASE_ONE_GAMES`, and each per-game module such as `crates/bitino-canonical/src/games/roulette.rs` exports a registry, canonical snapshot, board and stream projections, wire action helpers, engine payload construction, and proof linkage. The shared model in `crates/bitino-canonical/src/model.rs` gives every exposed wager one canonical identity before it touches wire, engine, presentation, exposure, replay, or audit paths.

Myosu's corresponding pieces currently live in several places:

- `crates/myosu-games/src/traits.rs` defines `GameType`, `GameConfig`, `GameParams`, `StrategyQuery`, and `StrategyResponse`.
- `crates/myosu-games/src/registry.rs` lists built-in game types.
- `crates/myosu-games-portfolio/src/game.rs` defines `ResearchGame`, the all-research-game manifest, and slug/chain-id mapping.
- `crates/myosu-games-portfolio/src/protocol.rs` defines `PortfolioAction`, `PortfolioStrategyQuery`, and `PortfolioStrengthQuery`.
- `crates/myosu-games-portfolio/src/state.rs` defines the current compact `PortfolioChallenge`.
- `crates/myosu-games-portfolio/src/engine.rs` dispatches a challenge to a rule-aware strategy distribution.
- `crates/myosu-games-portfolio/src/renderer.rs` renders a static portfolio decision snapshot for `myosu-play`.
- `crates/myosu-play/src/cli.rs` and `crates/myosu-play/src/main.rs` expose every research game through `--game` and smoke-test mode.
- `tests/e2e/research_strength_harness.sh` proves strategy outputs for every research game but does not prove replayable transitions.

In this plan, "canonical action id" means a stable, game-scoped string such as `bridge.play.follow_suit_high` or `backgammon.move.bear_off`. It is not presentation copy. Renderers may display it, and pipe mode may accept a human alias, but internal validation must decode to the canonical id.

In this plan, "state snapshot" means a serializable public view of the current game state plus hashes of hidden/private state. The snapshot should be enough for an agent to know all legal actions available to the acting player, and enough for a validator to know what was scored, without leaking hidden information that the acting player should not see.

In this plan, "truth trace" means the future replay record for one decision or round. It records the starting state hash, acting player, legal actions, selected action, strategy response hash, next state hash, terminal payoff if any, and a ruleset version. A validator can re-run the transition and confirm the same hashes after the core transition API exists.

## Plan of Work

First, add a shared canonical model to `crates/myosu-games/src/canonical.rs` and re-export it from `crates/myosu-games/src/lib.rs`. Keep this module generic and free of dependencies on `myosu-games-portfolio`, poker, Liar's Dice, or Kuhn. Define these data types:

- `CanonicalGameSpec` with `game_type: GameType`, `slug: String`, `chain_id: String`, `ruleset_version: u32`, `display_name: String`, `default_players: u8`, and `rule_file: Option<String>`.
- `CanonicalActionSpec` with `game_id: String`, `action_id: String`, `family: String`, `display_label: String`, `legal_phases: Vec<String>`, and `params_schema: serde_json::Value`.
- `CanonicalStateSnapshot` with `game_id: String`, `ruleset_version: u32`, `trace_id: String`, `phase: String`, `actor: Option<u8>`, `public_state: serde_json::Value`, `private_state_commitments: Vec<String>`, `legal_actions: Vec<CanonicalActionSpec>`, and `terminal: bool`.
- `CanonicalStrategyBinding` with `query_hash: String`, `response_hash: String`, `checkpoint_hash: Option<String>`, `engine_tier: String`, `engine_family: String`, and `quality_summary: Option<String>`.
- `CanonicalTransitionTrace` with `trace_id`, `game_id`, `ruleset_version`, `state_hash_before`, `action_id`, `action_params`, `state_hash_after`, `strategy_binding`, and `payoff: Option<Vec<i64>>`.
- `CanonicalTruthError` for malformed actions, hash mismatches, unsupported games, and non-terminal payoff requests.

Use a deterministic hash helper, for example `canonical_hash<T: Serialize>(value: &T) -> Result<String, CanonicalTruthError>`, built on an existing dependency if one is already in the workspace. If no stable hash dependency exists in `myosu-games`, add `sha2` or `blake3` only after checking `Cargo.toml` for existing usage. The hash must serialize with a deterministic encoding. If JSON is used, restrict it to values built by the canonical model and write tests that compare exact hash strings for a small fixture.

Next, create a new crate at `crates/myosu-games-canonical`. This crate is the cross-game aggregator. It should depend on `myosu-games` and `myosu-games-portfolio`. Do not make `myosu-games-portfolio` depend on `myosu-games-canonical`; that would invert the dependency boundary. Add the crate to the workspace in `Cargo.toml`.

Inside `crates/myosu-games-canonical/src/lib.rs`, define:

- `pub const CANONICAL_TEN: [ResearchGame; 10]`.
- `pub fn canonical_ten_specs() -> Vec<CanonicalGameSpec>`.
- `pub fn canonical_game_spec(game: ResearchGame) -> Option<CanonicalGameSpec>`.
- `pub fn canonical_action_specs(game: ResearchGame) -> Option<Vec<CanonicalActionSpec>>`.
- `pub fn canonical_bootstrap_snapshot(game: ResearchGame) -> Result<CanonicalStateSnapshot, CanonicalTruthError>`.
- `pub fn canonical_bootstrap_strategy_binding(game: ResearchGame) -> Result<CanonicalStrategyBinding, CanonicalTruthError>`.

The first pass should adapt the current `PortfolioSolver::strength_query` and `PortfolioSolver::strength_quality` output. That means the initial snapshots can be representative decision snapshots rather than full game trees. The important rule is that every legal action in `StrategyResponse<PortfolioAction>` must map to exactly one `CanonicalActionSpec` and every renderer action label must come from that same mapping.

Then add per-family adapter modules inside `crates/myosu-games-canonical/src/games/`:

- `poker_like.rs` for `nlhe-six-max`
- `hanafuda.rs` for `hanafuda-koi-koi`
- `mahjong.rs` for `riichi-mahjong`
- `trick_taking.rs` for `bridge`
- `gin_rummy.rs`
- `stratego.rs`
- `ofc.rs`
- `shedding.rs` for `dou-di-zhu`
- `backgammon.rs`
- `cribbage.rs`

Do not duplicate scoring code in these modules. They should call or wrap existing portfolio helpers. For example, the Bridge adapter should use the legal actions already produced by `crates/myosu-games-portfolio/src/engines/trick_taking.rs`, then attach canonical action ids and state metadata. The richer state-transition logic belongs in `execplans/EXECPLAN_CANONICAL_TEN_CORE_GAME_LOGIC.md`.

Add executable examples to the new crate:

- `crates/myosu-games-canonical/examples/canonical_manifest.rs`
- `crates/myosu-games-canonical/examples/canonical_snapshot.rs`

`canonical_manifest` should print one stable line per game, including the slug, chain id, ruleset version, rule file, legal action count, and whether a bootstrap snapshot can be built. `canonical_snapshot` should accept a game slug, then print a stable `CANONICAL_SNAPSHOT` line with `game`, `trace_id`, `state_hash`, legal action count, `query_hash`, and `response_hash`.

Finally, add tests in the new crate. The tests should fail if the canonical-ten list has anything other than ten games, if a canonical action id is duplicated within a game, if a strategy response action lacks a canonical action spec, if a renderer completion label cannot decode to a canonical action id, or if a snapshot/binding hash changes when the same fixture is rebuilt twice in the same process.

## Concrete Steps

Run these commands from the repository root to orient yourself before editing:

    rg -n "pub enum GameType|pub struct StrategyResponse" crates/myosu-games/src
    rg -n "pub enum ResearchGame|ALL_PORTFOLIO_ROUTED_GAMES|PortfolioAction|PortfolioChallenge" crates/myosu-games-portfolio/src
    rg -n "strength_query|strength_roundtrip|quality_summary" crates/myosu-games-portfolio crates/myosu-miner crates/myosu-validator crates/myosu-play tests/e2e

Create `crates/myosu-games/src/canonical.rs` and update `crates/myosu-games/src/lib.rs` to re-export it. Add focused unit tests in the same module for serialization, hash stability, duplicate-action rejection helper behavior, and terminal payoff optionality.

Create `crates/myosu-games-canonical/Cargo.toml`, `src/lib.rs`, `src/games/mod.rs`, and the per-family adapter files listed above. Add the crate to the workspace `members` in the root `Cargo.toml`.

Add the two examples:

    crates/myosu-games-canonical/examples/canonical_manifest.rs
    crates/myosu-games-canonical/examples/canonical_snapshot.rs

The expected manifest command should look like this after implementation:

    SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-canonical --example canonical_manifest

Expected output shape:

    CANONICAL_GAME slug=nlhe-six-max chain_id=nlhe_6max ruleset_version=1 actions=3 snapshot=ok
    CANONICAL_GAME slug=bridge chain_id=bridge ruleset_version=1 actions=3 snapshot=ok
    CANONICAL_GAME slug=cribbage chain_id=cribbage ruleset_version=1 actions=3 snapshot=ok

The expected snapshot command should look like this:

    SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-canonical --example canonical_snapshot -- bridge

Expected output shape:

    CANONICAL_SNAPSHOT game=bridge trace_id=bridge:bootstrap-v1 state_hash=<hex> actions=3 query_hash=<hex> response_hash=<hex>

The exact hash values are allowed to differ during implementation until the model stabilizes. Once the model stabilizes, add a fixture test that pins at least one hash for a tiny test value, not for every game snapshot.

## Validation and Acceptance

This plan is accepted when all of the following are true:

- `crates/myosu-games/src/canonical.rs` exists and is re-exported by `myosu-games`.
- `crates/myosu-games-canonical` exists in the workspace and compiles.
- `CANONICAL_TEN` contains exactly the ten portfolio reference games listed in this plan.
- `canonical_manifest` prints ten `CANONICAL_GAME` lines and every line ends with `snapshot=ok`.
- `canonical_snapshot` prints a `CANONICAL_SNAPSHOT` line for every canonical-ten game.
- Every canonical-ten strategy response action maps to exactly one canonical action id.
- Every canonical-ten renderer completion label maps back to one canonical action id.
- Duplicate canonical action ids fail a test.
- A malformed action id fails before any game adapter tries to apply it.

Run this validation set from the repository root:

    SKIP_WASM_BUILD=1 cargo test --quiet -p myosu-games
    SKIP_WASM_BUILD=1 cargo test --quiet -p myosu-games-canonical
    SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-canonical --example canonical_manifest
    for game in nlhe-six-max hanafuda-koi-koi riichi-mahjong bridge gin-rummy stratego ofc-chinese-poker dou-di-zhu backgammon cribbage; do SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-canonical --example canonical_snapshot -- "$game"; done
    SKIP_WASM_BUILD=1 cargo clippy -p myosu-games -p myosu-games-canonical --all-targets -- -D warnings
    cargo fmt --check -p myosu-games -p myosu-games-canonical

If unrelated workspace changes cause a command to fail outside these crates, capture the failing output in `Surprises & Discoveries`, fix only failures caused by this plan, and leave unrelated dirty work untouched.

## Idempotence and Recovery

All work in this plan is additive. Re-running the examples should not mutate source files. If generated artifacts are needed during implementation, write them under `target/e2e/canonical-truth/` or another `target/` subdirectory.

If a new crate causes a dependency cycle, do not move game-specific adapters into `myosu-games`. Instead, keep the shared data model in `myosu-games` and move cross-game aggregation into the new crate. The intended dependency direction is `myosu-games` at the base, game crates above it, and `myosu-games-canonical` as an optional aggregator above the game crates.

If hash stability is difficult because of JSON map ordering, switch the canonical serializable fields to structs or ordered maps. Do not accept nondeterministic hashes in canonical snapshots or bindings.

## Interfaces and Dependencies

The shared canonical module in `crates/myosu-games/src/canonical.rs` should expose plain serializable Rust structs and small helper functions. It should not import `myosu-games-portfolio` or any dedicated game crate.

The aggregator crate `crates/myosu-games-canonical` should be allowed to import `myosu-games-portfolio` because it is a proof and registry layer. It should not be required by `myosu-play`, `myosu-miner`, or `myosu-validator` until its examples and tests are green.

The first version may define the `CanonicalTransitionTrace` type, but the truth-layer examples and gate should stop at snapshots and strategy bindings. Real transition traces are accepted only after the core game logic plan provides `apply_action()` and deterministic next-state hashes. A plan that only wraps the current `PortfolioRenderer::pipe_output()` string is not accepted.

## Artifacts and Notes

The important artifacts produced by this plan are:

- `crates/myosu-games/src/canonical.rs`
- `crates/myosu-games-canonical/`
- `crates/myosu-games-canonical/examples/canonical_manifest.rs`
- `crates/myosu-games-canonical/examples/canonical_snapshot.rs`
- Ten `CANONICAL_SNAPSHOT` example outputs, one per canonical-ten game

Change note, 2026-04-10 / Codex: Initial plan written after researching Bitino's canonical wager/truth layer and comparing it with Myosu's current all-corpus strategy adapter.

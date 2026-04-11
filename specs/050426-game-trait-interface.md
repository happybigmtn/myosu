# Specification: Game Trait Interface

Status: Draft
Date: 2026-04-05
Depends-on: robopoker fork (`happybigmtn/robopoker`) providing `rbp-mccfr` CFR traits
Supersedes: `specs/031626-02a-game-engine-traits.md` (original draft)

## Objective

Define the multi-game abstraction layer that makes myosu a game-agnostic solving
platform. The dedicated game implementations (poker, Liar's Dice, Kuhn) and the
research portfolio rule-aware surface satisfy a shared trait and wire shape. The
solver (miner), validator, and gameplay layers dispatch by game selection while
keeping per-game wire parsing in the game crates.

The interface has three concerns:

1. **CFR trait surface** -- thin re-exports of robopoker's `rbp-mccfr` traits
   (`CfrGame`, `CfrTurn`, `CfrInfo`, `CfrEdge`, `Profile`, `Encoder`) plus
   myosu additions for game identification and configuration.
2. **Wire protocol** -- `StrategyQuery<I>` / `StrategyResponse<E>` generics with
   per-game binary codecs for miner-validator transport.
3. **Rendering** -- `GameRenderer` trait for TUI/pipe output, decoupled from
   solver logic.

## Evidence Status

All items below are verified against code in the repository as of 2026-04-05.

### Core trait crate: `myosu-games`

| Item | Location | Status |
|------|----------|--------|
| CFR trait re-exports from `rbp-mccfr` | `crates/myosu-games/src/traits.rs:8-9` | Verified |
| `GameType` enum (dedicated games plus all research portfolio built-ins and `Custom(String)`) | `traits.rs` | Verified |
| `GameConfig` with `GameParams` typed variants | `traits.rs:17-57`, `traits.rs:158-181` | Verified |
| `StrategyQuery<I>` / `StrategyResponse<E>` generics | `traits.rs:187-241` | Verified |
| `StrategyResponse::is_valid()` epsilon check (0.001) | `traits.rs:220-226` | Verified |
| `GameRegistry` and `GameDescriptor` | `crates/myosu-games/src/registry.rs` | Verified |
| `#[non_exhaustive]` on `GameType` and `GameParams` | `traits.rs:65`, `traits.rs:161` | Verified |
| `Custom(String)` / `Custom(serde_json::Value)` escape hatches | `traits.rs:77`, `traits.rs:180` | Verified |
| `GameType::from_bytes` / `to_bytes` for on-chain encoding | `traits.rs:96-128` | Verified |
| `proptest` roundtrip tests for serialization | `traits.rs:473-498` | Verified |

### Rendering trait: `myosu-tui`

| Item | Location | Status |
|------|----------|--------|
| `GameRenderer` trait (`Send`, renderer + pipe contract) | `crates/myosu-tui/src/renderer.rs` | Verified |
| Methods: `render_state`, `desired_height`, `declaration`, `completions`, `parse_input`, `clarify`, `pipe_output`, `game_label`, `context_label` | same file | Verified |

### Implemented games

| Game | Crate | CFR | Wire codec | Renderer | Solver |
|------|-------|-----|------------|----------|--------|
| NLHE Poker | `myosu-games-poker` | robopoker `NlheProfile` + `NlheEncoder` | `wire.rs` (bincode, 1 MB limit) | `renderer.rs` (`NlheRenderer`) | `PokerSolver` (MCCFR) |
| Liar's Dice | `myosu-games-liars-dice` | `LiarsDiceSolver<N>` | per-crate encode/decode | `renderer.rs` | `LiarsDiceSolver<N>` (generic tree count) |
| Kuhn Poker | `myosu-games-kuhn` | analytical (closed-form Nash) | per-crate encode/decode | `renderer.rs` | analytical |
| Research portfolio games | `myosu-games-portfolio` | rule-aware portfolio reference engines | per-crate encode/decode | `PortfolioRenderer` | `PortfolioSolver` |

### Adding a new game (proven path)

Liar's Dice, Kuhn Poker, and the research portfolio surface followed this path
without changing existing dedicated game crates:

1. Implement CFR traits (`CfrGame`, `CfrTurn`, `CfrInfo`, `CfrEdge`).
2. Add a `GameType` variant and `GameParams` variant.
3. Implement `GameRenderer` for TUI/pipe display.
4. Implement wire codec encode/decode functions.
5. Register in `GameRegistry::supported()`.

No existing game code needs modification. The `#[non_exhaustive]` attribute on
`GameType` and `GameParams` ensures downstream consumers already handle unknown
variants.

### Validator consumption pattern

The validator (`crates/myosu-validator/src/validation.rs`) imports game-specific
decoders (`decode_strategy_query`, `decode_strategy_response`) and solver types
per game, dispatching on `GameSelection`. The generic `StrategyResponse<E>` type
flows across the boundary.

INV-004 (no dependency path between `myosu-play` and `myosu-miner`) is enforced
by `cargo tree` check in CI.

## Acceptance Criteria

### AC-GTI-01: CFR trait re-exports are the sole solver interface

All solver and validator code depends on the re-exported CFR traits from
`myosu-games`. No crate outside a `myosu-games-*` implementation crate imports
`rbp-mccfr` directly.

### AC-GTI-02: `GameType` byte encoding roundtrips

`GameType::from_bytes(game_type.to_bytes())` returns the original value for all
variants, including `Custom`. Verified by `proptest` in `traits.rs`.

### AC-GTI-03: `StrategyResponse` validity invariant

`StrategyResponse::is_valid()` returns `true` iff action probabilities sum to
within 0.001 of 1.0, or the action list is empty (terminal state).

### AC-GTI-04: New games require zero changes to existing game code

Adding a full game requires only: new `GameType`/`GameParams` variants, a new
crate implementing CFR traits + wire codec + renderer, and a `GameRegistry`
entry. Bootstrap-only portfolio games may initially use `GameParams::Custom`
metadata behind the same `StrategyQuery` / `StrategyResponse` contract.

### AC-GTI-05: Wire codec decode limits

Each game's wire codec enforces a maximum decode size (poker: 1 MB via
`MAX_DECODE_BYTES`). No unbounded deserialization.

### AC-GTI-06: `GameRenderer` is decoupled from solving

`GameRenderer` lives in `myosu-tui`, not in `myosu-games`. Game implementation
crates may provide renderer adapters by depending on `myosu-tui`; solver,
miner, and validator logic stay decoupled from the TUI shell.

### AC-GTI-07: `#[non_exhaustive]` extensibility

`GameType` and `GameParams` are `#[non_exhaustive]`. Downstream match arms must
include a wildcard or `Custom` handler, preventing breakage when new variants
land.

### AC-GTI-08: Serialization roundtrip for all config types

`GameConfig`, `GameType`, and `StrategyResponse` survive JSON (serde)
serialization roundtrips. Verified by `proptest` generators
`arb_game_type`, `arb_game_config`, `arb_strategy_response`.

## Verification

| Check | Command | Pass condition |
|-------|---------|----------------|
| Trait tests | `cargo test -p myosu-games` | All pass, including proptest roundtrips |
| Poker wire codec | `cargo test -p myosu-games-poker -- wire` | Encode/decode roundtrip |
| Liar's Dice wire codec | `cargo test -p myosu-games-liars-dice -- wire` | Encode/decode roundtrip |
| Kuhn wire codec | `cargo test -p myosu-games-kuhn -- wire` | Encode/decode roundtrip |
| Portfolio wire codec | `cargo test -p myosu-games-portfolio -- wire` | Encode/decode roundtrip |
| Portfolio corpus + solver manifest | `cargo run -p myosu-games-portfolio --example bootstrap_manifest -- table` | 22 research game identities expose route, rule file, chain id, player count, and solver family; harness checks the dedicated routes and Hearts/Cribbage split mapping |
| Dedicated solver harnesses | `bash tests/e2e/research_games_harness.sh` | NLHE heads-up, Liar's Dice, and the exact Kuhn benchmark produce checkpoint/query/response artifacts with exact-match scores; the two research-dedicated games also expose `STRENGTH` roundtrip artifacts |
| Portfolio corpus + solver harness | `bash tests/e2e/research_portfolio_harness.sh` | 20 portfolio-routed games produce checkpoint/query/response artifacts, exact-match scores, and `STRENGTH` roundtrips from the manifest-derived game list; scoped checkpoints reject cross-game query artifacts; strength manifest entries must be `engine_tier=rule-aware` |
| Direct rbp-mccfr imports scoped to game implementation crates | `rg 'use rbp_mccfr' crates/` | Matches only in shared trait or `myosu-games-*` implementation crates |
| INV-004 isolation | CI `cargo tree` check | No path between `myosu-play` and `myosu-miner` |
| Registry completeness | `GameRegistry::supported()` roundtrips through `GameType::from_bytes` | Dedicated games plus all research portfolio built-ins |

## Open Questions

1. **Minimum viable encoder size for poker**: The full NLHE encoder consumes
   7--11 GB RAM (138M entries). What is the smallest encoder that produces
   strategy quality acceptable for on-chain validation? This affects miner
   hardware requirements and devnet feasibility.

2. **Formalizing `Custom(String)` game types**: The `Custom` variant currently
   serves as an unvalidated escape hatch. Should custom game registration
   require an on-chain governance proposal, or remain permissionless? What
   validation (if any) should `Custom` identifiers undergo (length limits,
   character set, uniqueness)?

3. **Object safety for dynamic dispatch**: The current CFR traits are not
   object-safe (they use associated types and generics). If miners need to
   support multiple game types in a single process, a trait-object or
   enum-dispatch layer may be needed above the CFR traits. The current
   per-game-binary validator model avoids this but limits runtime flexibility.

4. **Robopoker fork tracking (INV-006)**: All downstream changes to the
   `happybigmtn/robopoker` fork must be documented in
   `docs/robopoker-fork-changelog.md`. The current fork baseline is v1.0.0.
   Process for upstreaming changes or rebasing on new upstream releases is
   undefined.

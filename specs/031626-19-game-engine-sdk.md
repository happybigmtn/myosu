# Specification: Game Engine SDK — Third-Party Game Development Kit

Source: CEO review — platform framing for the 20-game roadmap
Status: Draft
Date: 2026-03-17
Depends-on: GT-01..05 (traits), MG-01..04 (multi-game architecture), 031626-13 (n-player)
Blocks: Stage 2 (third-party game engines)

## Purpose

Myosu's current specs describe a **tool**: the myosu team builds 20 game
engines. This spec reframes myosu as a **platform**: anyone can register a
game, implement its solver, and participate in the incentive network.

The difference is the difference between iPhone and App Store. The chain,
the validator protocol, and the TUI shell are infrastructure. Games are
applications that plug into that infrastructure.

This spec defines the developer experience for adding a new game to myosu:
the SDK crate, the `myosu init` scaffolding tool, the testing harness, the
registration flow, and the documentation. The goal: **a competent Rust
developer can add a new game to myosu in one afternoon.**

```
CURRENT (TOOL)                        TARGET (PLATFORM)

myosu team builds 20 games      →    myosu team builds 5 games
community builds 0 games        →    community builds 15+ games
API is internal                  →    API is the primary product surface
TUI is the product               →    TUI is one client of many
adding a game requires           →    adding a game requires implementing
  deep knowledge of internals         one trait + running a test harness
```

## The "30-Minute Game" Story

A developer wants to add Kuhn Poker (simplest non-trivial poker variant,
12 info sets) to myosu. Their experience:

```bash
# 1. Scaffold (30 seconds)
cargo install myosu-sdk
myosu init --game kuhn-poker
cd myosu-games-kuhn-poker

# 2. Implement (20 minutes)
# Edit src/game.rs — implement CfrGame trait
# Edit src/encoder.rs — implement Encoder trait (trivial for small games)
# Edit src/renderer.rs — implement GameRenderer trait

# 3. Test (5 minutes)
myosu test              # runs trait compliance + exploitability convergence
myosu train --iters 1000  # train locally, verify convergence
myosu play              # play locally against trained bot

# 4. Register (5 minutes)
myosu register --chain ws://localhost:9944 --game-type kuhn-poker
# → creates subnet on devnet, registers game type
```

### When things go wrong

The 30-minute story assumes no bugs. Here's what happens when there are:

```bash
$ myosu test
running 6 trait compliance checks...
  ✓ root is chance or player node
  ✓ legal actions nonempty except terminal
  ✓ apply changes state
  ✓ terminal states have utility
  ✗ FAIL: payoff is not zero-sum
    at terminal state after [Bid(2,3), Challenge]:
    player 0 utility: +1.0
    player 1 utility: +0.5
    sum: 1.5 (expected: 0.0)
    → fix: ensure utility(state, 0) + utility(state, 1) == 0
  ✓ information sets distinct

1 check failed. fix the issue above and run `myosu test` again.
```

Error messages must:
1. Name the specific check that failed
2. Show the concrete values that violated the invariant
3. Tell the developer exactly what to fix
4. Reference the method name (`utility`) so they know where to look

## Scope

### AC-SDK-01: SDK Crate (`myosu-sdk`)

- Where: `crates/myosu-sdk/ (new)`
- How: A meta-crate that re-exports everything a game developer needs:

  ```rust
  // crates/myosu-sdk/src/lib.rs

  // Core — always available. No TUI dependency.
  pub use myosu_games::{CfrGame, CfrEdge, CfrTurn, CfrInfo, Profile, Encoder};
  pub use myosu_games::{NPlayerGame, NPlayerTurn};  // from 031626-13
  pub use myosu_games::{GameConfig, GameType, GameParams, ExploitMetric};
  pub use myosu_games::wire::{WireStrategy, WireSerializable};

  // TUI rendering — opt-in via `myosu-sdk = { features = ["tui"] }`
  #[cfg(feature = "tui")]
  pub use myosu_tui::GameRenderer;

  pub mod testing;   // trait compliance test harness
  pub mod scaffold;  // project template generation
  ```

  Developers depend on `myosu-sdk` instead of individual crates. The SDK
  provides a stable API surface while internal crate boundaries can shift.

  The `tui` feature is optional because many SDK consumers (miners,
  validators, headless bots) don't need terminal rendering. The scaffold
  tool generates `features = ["tui"]` by default so game developers get
  the full experience, but miner operators can omit it.

- Required tests:
  - `sdk::tests::all_traits_importable`
  - `sdk::tests::kuhn_poker_compiles_against_sdk`
- Pass/fail:
  - `use myosu_sdk::CfrGame` compiles
  - A minimal game (Kuhn Poker) implements all required traits using only SDK imports

### AC-SDK-02: Scaffold Tool (`myosu init`)

- Where: `crates/myosu-sdk/src/scaffold/ (new)`
- How: `myosu init --game <name>` generates a new crate with:

  ```
  myosu-games-<name>/
  ├── Cargo.toml          # depends on myosu-sdk with features = ["tui"]
  ├── src/
  │   ├── lib.rs          # re-exports
  │   ├── game.rs         # CfrGame impl (stub with TODOs)
  │   ├── encoder.rs      # Encoder impl (stub)
  │   ├── renderer.rs     # GameRenderer impl (stub, gated behind tui feature)
  │   └── tests.rs        # trait compliance tests (pre-written)
  └── README.md           # game-specific documentation template
  ```

  `renderer.rs` is gated behind `#[cfg(feature = "tui")]` in `lib.rs`.
  Developers who only want to build a solver can remove the `tui` feature
  from Cargo.toml and skip the renderer entirely.

  For solver-only scaffolding: `myosu init --game <name> --no-tui`
  generates without the renderer stub or TUI dependency.

  The stubs compile but panic at runtime with clear messages:
  ```rust
  fn legal_actions(state: &Self::State) -> Vec<Self::Action> {
      todo!("implement legal_actions for your game")
  }
  ```

  For n-player games: `myosu init --game <name> --players 4` generates
  `NPlayerGame` stubs instead of `CfrGame`.

- Required tests:
  - `scaffold::tests::generates_compilable_crate`
  - `scaffold::tests::generated_tests_fail_with_todo`
  - `scaffold::tests::nplayer_flag_uses_correct_trait`
  - `scaffold::tests::refuses_to_overwrite_existing_directory`
- Pass/fail:
  - Generated crate compiles with `cargo check`
  - `cargo test` runs but fails with `not yet implemented` messages
  - `--players 4` generates `NPlayerGame` imports
  - Existing directory → clear error "directory already exists, use --force to overwrite"

### AC-SDK-03: Trait Compliance Test Harness

- Where: `crates/myosu-sdk/src/testing/ (new)`
- How: Pre-built test functions that validate a game engine implementation:

  ```rust
  pub fn assert_game_valid<G: CfrGame>() {
      assert_root_is_chance_or_player::<G>();
      assert_legal_actions_nonempty_except_terminal::<G>();
      assert_apply_changes_state::<G>();
      assert_terminal_has_utility::<G>();
      assert_payoff_is_zero_sum::<G>();
      assert_info_sets_distinct_for_different_observations::<G>();
  }

  pub fn assert_solver_converges<G, E>(
      encoder: &E,
      max_iters: usize,
      target_exploit: f64,
  ) where G: CfrGame, E: Encoder<G> {
      // Train MCCFR for max_iters, assert exploitability < target
  }

  pub fn assert_wire_serialization_roundtrips<G: WireSerializable>() {
      // Generate random info sets and edges, serialize/deserialize
  }

  // N-player variant — uses NPlayerGame trait, no zero-sum check
  pub fn assert_nplayer_game_valid<G: NPlayerGame>() {
      assert_root_is_chance_or_player_np::<G>();
      assert_legal_actions_nonempty_except_terminal_np::<G>();
      assert_apply_changes_state_np::<G>();
      assert_terminal_has_utility_for_all_players::<G>();
      assert_num_players_consistent::<G>();
      // Note: no zero-sum check — n-player games may have non-zero-sum
      // payoffs (e.g., all players lose in Mahjong when one wins big)
  }
  ```

  Developers call these in their `tests.rs`:
  ```rust
  #[test]
  fn game_is_valid() {
      myosu_sdk::testing::assert_game_valid::<KuhnPokerGame>();
  }

  #[test]
  fn solver_converges() {
      myosu_sdk::testing::assert_solver_converges::<KuhnPokerGame, _>(
          &KuhnEncoder, 10_000, 0.001
      );
  }
  ```

- Required tests:
  - `testing::tests::rps_passes_all_compliance_checks`
  - `testing::tests::broken_game_fails_zero_sum_check`
  - `testing::tests::convergence_test_detects_non_convergence`
- Pass/fail:
  - Rock-Paper-Scissors (existing robopoker reference) passes all checks
  - A deliberately broken game (non-zero-sum payoffs) fails the zero-sum check
  - A random (non-converging) "strategy" fails the convergence check

### AC-SDK-04: Game Registration Flow

- Where: `crates/myosu-sdk/src/register/ (new)`, `crates/myosu-chain/pallets/game-solver/ (extend)`
- How: On-chain game type registration via extrinsic:

  ```rust
  // On-chain: pallet_game_solver
  #[pallet::call]
  fn register_game_type(
      origin: OriginFor<T>,
      game_type: BoundedVec<u8, ConstU32<64>>,  // max 64 bytes, e.g., "kuhn-poker"
      num_players: u8,
      exploit_metric: ExploitMetricConfig,
      wasm_hash: Option<H256>,         // optional: hash of game engine WASM
  ) -> DispatchResult;
  ```

  `game_type` is bounded to 64 bytes (sufficient for any game name in
  ASCII). Uses Substrate's `BoundedVec` to enforce at the type level —
  oversized inputs are rejected before touching storage.

  The pallet stores the game type metadata. Anyone can then create a
  subnet for this game type via the existing `create_subnet` extrinsic.

  For Phase 0-1, game registration is permissioned (governance or sudo).
  For Stage 2+, permissionless registration with a burn cost (prevents
  spam).

  The SDK provides a CLI command:
  ```bash
  myosu register --chain ws://localhost:9944 \
      --game-type kuhn-poker \
      --players 2 \
      --exploit-unit "exploit" \
      --exploit-baseline 1.0
  ```

  The CLI validates chain connectivity before submitting the extrinsic.
  If the chain is unreachable, it prints: "cannot connect to
  ws://localhost:9944 — is the node running?" and exits with code 1.
  Connection timeout is 5 seconds.

- Required tests:
  - `register::tests::register_new_game_type`
  - `register::tests::duplicate_game_type_fails`
  - `register::tests::create_subnet_for_registered_game`
  - `register::tests::permissionless_registration_burns_tokens`
  - `register::tests::insufficient_balance_rejects_registration`
- Pass/fail:
  - New game type appears in `GameRegistry` after registration
  - Duplicate game type name returns clear error
  - Subnet creation succeeds for registered game type

### AC-SDK-05: Developer Documentation

- Where: `docs/sdk/ (new)`
- How: A developer guide covering:

  1. **Quickstart** — the 30-minute game story end-to-end
  2. **Trait reference** — `CfrGame`, `Encoder`, `GameRenderer`, `NPlayerGame`
  3. **Testing guide** — how to use the compliance harness
  4. **Registration guide** — how to register and create a subnet
  5. **Examples** — Kuhn Poker (2-player), Liar's Dice (2-player), Tic-Tac-Toe (proof that even perfect-info games work, though trivially)
  6. **FAQ** — common pitfalls, performance tips, abstraction guidance

  The documentation is the product. If the docs aren't good enough
  for a developer to succeed without asking for help, the SDK has failed.

- Required tests: N/A (documentation)
- Pass/fail:
  - A developer unfamiliar with myosu can implement Kuhn Poker by
    following the quickstart guide without reading any source code
  - All code examples in the docs compile and pass tests

---

## How this changes the project framing

### OS.md impact

The multi-game roadmap (20 games) shifts from "myosu team builds all 20"
to "myosu team builds Stage 0-1 games (poker variants per 031626-14 +
Liar's Dice proof per MG-01..04) + community builds Stage 2-3 games
using the SDK."

### Stage 2 exit criteria update

Current: "Game Engine SDK published for third-party developers"
Updated: "SDK published, 2+ community-built game engines running on
mainnet subnets"

### Revenue impact

New revenue stream: **SDK ecosystem fees.** Game engines that generate
gameplay revenue share a percentage with the protocol. This aligns the
platform with game developer success.

## Implementation order

1. SDK-01 (meta-crate) — foundation, fast
2. SDK-03 (test harness) — highest developer value
3. SDK-02 (scaffold) — convenience, can ship after
4. SDK-04 (registration) — needs chain, Phase 1
5. SDK-05 (docs) — parallel with all above

## Decision log

- 2026-03-17: SDK as a platform play, not just internal tooling. Rationale:
  20 games is too many for one team. The community must build most of them.
  The SDK makes that possible.
- 2026-03-17: Scaffold generates stubs that compile but panic. Rationale:
  "it compiles" is the minimum bar for a useful template. Panics give clear
  error messages pointing at what to implement.
- 2026-03-17: Permissioned game registration for Phase 0-1, permissionless
  for Stage 2+. Rationale: prevent spam subnets during bootstrap while the
  economics are still being tuned.
- 2026-03-17: Kuhn Poker as the reference example. Rationale: 12 info sets,
  known exact solution, implementable in 30 minutes. If the SDK can't make
  Kuhn Poker easy, it can't make anything easy.

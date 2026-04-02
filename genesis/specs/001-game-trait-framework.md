# Specification: Game Trait Framework

Source: Reverse-engineered from crates/myosu-games (traits.rs, registry.rs, lib.rs)
Status: Draft
Depends-on: none

## Purpose

The game trait framework provides a game-agnostic abstraction layer for
imperfect-information game solving. It defines the shared vocabulary that all
game implementations, solvers, miners, and validators use to represent game
trees, strategy queries, and strategy responses. Without this layer, each game
would require bespoke integration into the miner, validator, and gameplay
surfaces, making multi-game support impractical.

The primary consumers are game implementation crates (poker, Liar's Dice, future
games) and the downstream binaries (miner, validator, play) that operate on
game-agnostic strategy interfaces.

## Whole-System Goal

Current state: The framework exists and is actively used by two game
implementations (NLHE poker, Liar's Dice) and all downstream binaries.

This spec adds: Nothing new. This documents the existing behavioral contract.

If all ACs land: Any new game can be integrated into the system by implementing
the CFR trait family and registering a game type, without modifying miner,
validator, or gameplay code.

Still not solved here: Game-specific rendering, wire encoding, and artifact
formats remain the responsibility of individual game crates.

## Scope

In scope:
- Game type enumeration and byte-level serialization
- Game configuration with per-game parameter variants
- Strategy query and response generic types
- CFR trait re-exports from the robopoker dependency
- Game registry for built-in and custom game descriptors

Out of scope:
- Game-specific state machines (poker hands, dice rolls)
- Wire protocol encoding for network transport
- TUI rendering of game state
- Solver training algorithms

## Current State

The crate exists at crates/myosu-games with two modules (traits, registry) and a
public re-export surface of 13 types. It depends on rbp-core and rbp-mccfr from
the robopoker fork for the CFR trait family (CfrGame, CfrTurn, CfrEdge, CfrInfo,
Encoder, Profile, Probability, Utility).

GameType is a non-exhaustive enum with four variants: NlheHeadsUp, NlheSixMax,
LiarsDice, and Custom(String). GameParams is a non-exhaustive tagged enum with
matching parameter shapes per game type. Both support serde serialization with
snake_case renaming.

GameType supports byte-level round-trip serialization via from_bytes/to_bytes
using canonical byte strings (b"nlhe_hu", b"nlhe_6max", b"liars_dice", or
arbitrary UTF-8 for Custom).

StrategyQuery and StrategyResponse are generic over their information set and
edge types respectively. StrategyResponse validates that action probabilities sum
to approximately 1.0 within an epsilon of 0.001.

GameRegistry is a stateless unit struct providing three built-in descriptors
(NlheHeadsUp 2-player, NlheSixMax 6-player, LiarsDice 2-player) and the
ability to describe any GameType including Custom variants. Custom games default
to 2 players.

## What Already Exists

| Sub-problem | Existing code / flow | Reuse / extend / replace | Why |
|---|---|---|---|
| CFR trait family | rbp-core, rbp-mccfr re-exports | Reuse | Robopoker provides battle-tested CFR abstractions |
| Game type enum | GameType with 4 variants + byte serde | Reuse | Already supports built-in and custom games |
| Strategy generics | StrategyQuery<I>, StrategyResponse<E> | Reuse | Game-agnostic query/response with validation |
| Game registry | GameRegistry with 3 built-in descriptors | Extend | New built-in games require adding variants |
| Game config | GameConfig + GameParams | Extend | New games require new GameParams variants |

## Non-goals

- Prescribing how games implement their CFR game trees.
- Defining wire encoding formats for network transport of strategies.
- Providing solver training logic or exploitability computation.
- Managing game-specific artifacts (encoders, abstraction bundles).
- Dictating how the chain represents game types on-chain.

## Behaviors

The framework exposes a set of types that downstream crates compose:

A game implementation crate implements CfrGame, CfrTurn, CfrEdge, and CfrInfo
from the re-exported robopoker traits, then uses GameType and GameConfig to
identify itself within the system.

GameType serializes to and from canonical byte strings. The from_bytes parser
recognizes three fixed byte patterns and falls back to UTF-8 parsing for Custom
variants, returning None on invalid UTF-8. The to_bytes method produces the
canonical form. num_players returns the default player count for each variant
(2 for heads-up and Liar's Dice, 6 for six-max, 2 for custom).

GameConfig bundles a GameType, player count, and GameParams into a single
configuration object. A convenience constructor produces a heads-up NLHE config
from a stack size.

StrategyResponse holds a vector of (edge, probability) pairs. The is_valid
method checks that probabilities sum to within 0.001 of 1.0. The
probability_for method returns the probability for a specific action or 0.0 if
the action is absent.

GameRegistry returns the three built-in game descriptors via supported(). The
describe method accepts any GameType and returns a descriptor, marking Custom
variants as non-builtin. The from_bytes method combines byte parsing with
descriptor lookup.

## Acceptance Criteria

- A new game type can be added to the registry without modifying downstream
  binaries that operate on game-agnostic interfaces.
- GameType round-trips through from_bytes and to_bytes for all built-in variants
  and for arbitrary valid UTF-8 custom names.
- StrategyResponse correctly identifies valid probability distributions within
  the 0.001 epsilon tolerance.
- StrategyResponse reports the correct probability for a queried action,
  returning 0.0 for absent actions.
- GameRegistry describes all built-in games with correct player counts and marks
  Custom variants as non-builtin.
- All custom types serialize and deserialize via serde with snake_case field
  naming.

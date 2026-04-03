# ADR 004: Enum Dispatch For Multi-Game Surfaces

- Status: Accepted
- Date: 2026-04-02
- Deciders: Myosu maintainers
- Consulted: `AGENTS.md`, `specs/031626-02a-game-engine-traits.md`, `specs/031626-06-multi-game-architecture.md`, `THEORY.MD`
- Informed: game-engine, miner, validator, and gameplay contributors
- Related: `crates/myosu-games/src/traits.rs`, `crates/myosu-games/src/registry.rs`, `crates/myosu-games-poker/`, `crates/myosu-games-liars-dice/`, `crates/myosu-play/src/cli.rs`

## Context

This is a retroactive record of the multi-game dispatch choice already visible
in the live game seam.

Robopoker's core CFR traits are not object-safe in the way a classic
`Box<dyn Trait>` architecture would need. The repo's own guidance records that
the inherited solver/game traits require `Copy` and `Sized`, which blocks a
trait-object strategy for the core CFR interfaces. At the same time, Myosu must
support more than one game without reopening poker-specific code each time.

The live repo already resolves this with explicit game enums, a shared game
registry, and game-specific crates that share transport and discovery seams
without pretending the underlying solver types are interchangeable trait
objects.

## Decision

Myosu uses enum dispatch and explicit registry seams for multi-game support
instead of trait-object erasure over the CFR core.

In practice:

- `GameType` and `GameParams` are the top-level cross-crate discriminants
- built-in games live in separate crates with their own concrete solver types
- miner, validator, and gameplay code branch explicitly on executable game
  choice where behavior truly differs
- shared abstractions stay at the query/response, registry, and renderer
  surfaces rather than forcing a fake uniform solver object model

## Alternatives Considered

### Option A: Enum dispatch plus shared registry

This won because it matches the object-safety limits of the upstream traits
while still giving Myosu an additive multi-game architecture.

### Option B: Trait objects over the core CFR interfaces

This was rejected because the upstream trait bounds do not support a truthful
`dyn`-based design for the actual solver/game surfaces.

### Option C: No shared game seam at all

This was rejected because miner, validator, and gameplay still need a stable
way to identify games, transport requests, and discover built-in vs custom
variants.

## Consequences

### Positive

- New games can be added additively through explicit variants and crates.
- Dispatch logic is visible and honest instead of hidden behind object-erasure
  gymnastics that the upstream traits cannot support.
- Cross-crate callers share one vocabulary for game identity.

### Negative

- Callers must write explicit matches for real behavior differences.
- Adding a new built-in game still requires touching enum/registry surfaces.

### Follow-up

- Keep shared seams narrow and transport-focused.
- Resist generic "one solver interface for every game" abstractions unless the
  underlying solver traits actually make that honest.

## Reversibility

Moderate.

If upstream traits ever become object-safe or Myosu adopts a different game
engine boundary, dispatch could move toward a trait-object or plugin model. But
that would require updating miner, validator, and gameplay call sites together.
The decision should only be reopened when a new abstraction reduces explicit
branching without hiding meaningful game-specific truth.

## Validation / Evidence

- `crates/myosu-games/src/traits.rs` defines the shared `GameType`,
  `GameParams`, and transport surfaces.
- `crates/myosu-games/src/registry.rs` resolves built-in and custom games
  through explicit enum cases.
- The separate poker and liar's-dice crates prove the current additive
  multi-game seam works without trait-object dispatch over the CFR core.

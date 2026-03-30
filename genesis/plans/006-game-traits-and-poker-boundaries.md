# Harden Game Traits and Poker Engine Boundaries

Status: Completed 2026-03-29.

This ExecPlan is a living document. Maintained per `genesis/PLANS.md`.

Provenance: Replaces `archive/genesis_1774729423/plans/006-game-traits-and-poker-boundaries.md` which was a bootstrap-only plan producing spec/review files. This plan targets implementation: hardening the already-working game crates with boundary enforcement, robopoker fork documentation, and multi-game extensibility proof.

## Purpose / Big Picture

The game crates (`myosu-games`, `myosu-games-poker`) already work and are CI-gated. This plan hardens their boundaries: enforcing solver/gameplay separation (INV-004), documenting the robopoker fork divergence (INV-006), and verifying that the trait surface supports adding a second game without modifying existing code.

After this plan, the game crate API surface is locked for stage-0, the robopoker fork has a CHANGELOG, and adding Liar's Dice (plan 012) requires zero changes to `myosu-games` or `myosu-games-poker`.

## Progress

- [x] (2026-03-28) Audited game crate structure, public API surface, and test coverage.
- [x] (2026-03-29) Added executable INV-004 enforcement via
  `crates/myosu-play/tests/invariants.rs`, proving no `myosu-play` ->
  `myosu-miner` or `myosu-miner` -> `myosu-play` dependency path.
- [x] (2026-03-29) Documented the local robopoker fork divergence from `v1.0.0`
  in `docs/robopoker-fork-changelog.md` using the pinned workspace rev.
- [x] (2026-03-29) Locked the variant-growth API seam in
  `crates/myosu-games/src/traits.rs` with `#[non_exhaustive]` on `GameType` and
  `GameParams`.
- [x] (2026-03-29) Added explicit custom-game extensibility proof in
  `crates/myosu-games/src/registry.rs`, showing `GameType::Custom(...)` can be
  described and round-tripped without registry edits.
- [x] (2026-03-29) Verified the property-based/shared game tests and added
  wire-edge coverage for empty responses, zero-probability edges, truncated
  payloads, maximum-width info keys, and unicode custom-game roundtrips.

## Surprises & Discoveries

- Observation: `myosu-games/src/traits.rs` re-exports from `rbp_core` and `rbp_mccfr`. These upstream types are the trait foundation.
  Evidence: `pub use rbp_core::*` and `pub use rbp_mccfr::*` in traits.rs.
- Observation: GameRegistry hardcodes 3 supported games but `GameType::Custom(String)` allows runtime extension.
  Evidence: `crates/myosu-games/src/registry.rs` lines showing `supported()` returns 3 entries.
- Observation: `myosu-play/src/main.rs` is 31,337 lines -- suspicious for a CLI entry point. May contain embedded test data or generated code.
  Evidence: `wc -l crates/myosu-play/src/main.rs`.
- Observation: literal INV-004 currently holds, but the stricter "gameplay shares
  only game crates" goal is still pressured because `myosu-play` depends on
  `myosu-chain-client`, which pulls in chain/runtime crates transitively.
  Evidence: `cargo tree -p myosu-play --edges normal` on 2026-03-29.
- Observation: the public growth seam is concentrated in enum variants, not in
  struct construction. `GameType` and `GameParams` are the stage-0 types most
  likely to gain new variants, while `GameConfig`, `StrategyQuery`, and
  `StrategyResponse` are better left directly constructible.
  Evidence: public-surface audit on 2026-03-29 across `crates/myosu-games`.
- Observation: the current poker wire contract is intentionally still
  unbounded-size bincode with fixed-int encoding and trailing-byte rejection.
  That is acceptable for plan 006 test coverage, but decode-size caps and
  stronger malformed-input hardening belong to plan 008 instead of being hidden
  inside this boundary slice.
  Evidence: `crates/myosu-games-poker/src/wire.rs` audit on 2026-03-29.

## Decision Log

- Decision: This plan targets implementation hardening, not bootstrap spec generation.
  Rationale: Prior plan 006 produced spec/review files. The game crates already work. What's needed is boundary enforcement and extensibility proof, not more documentation.
  Inversion: If we write more specs instead of enforcing boundaries, INV-004 and INV-006 remain unproven.
  Date/Author: 2026-03-28 / Genesis

- Decision: Do not modify the robopoker upstream types. Document what we re-export and why.
  Rationale: INV-006 requires tracking v1.0.0 baseline. Any changes to the fork need CHANGELOG entries.
  Date/Author: 2026-03-28 / Genesis

## Failure Modes

| Codepath | Realistic failure | Handling |
|---|---|---|
| INV-004 check | New crate accidentally depends on both miner and play | `cargo tree` test in CI catches this |
| API locking | `#[non_exhaustive]` breaks downstream code | Only apply to enums that will gain variants; test compile of dependents |
| Wire encoding | New edge case in serde_json/bincode causes silent data loss | Add property-based roundtrip tests for all public types |
| Fork divergence | Robopoker fork has undocumented changes | `git diff v1.0.0..HEAD` in fork repo to enumerate all changes |

## Outcomes & Retrospective

Plan 006 is materially complete at the current stage-0 truth boundary:

- literal INV-004 has executable enforcement
- INV-006 now has a repo-local fork divergence record
- enum growth seams are locked with `#[non_exhaustive]`
- custom-game extensibility has an explicit regression test
- wire/shared serialization edges now have focused coverage

The honest residual is still architectural, not test-shaped: `myosu-play`
retains a transitive chain/runtime dependency through `myosu-chain-client`.
That pressure is documented, but it is not silently being treated as part of
this completed boundary-hardening slice.

## Context and Orientation

```text
GAME CRATE DEPENDENCY GRAPH

  myosu-play (binary)
       |
       v
  myosu-games-poker (NLHE engine)
       |         |
       v         v
  myosu-games   myosu-tui
  (traits +     (shell +
   registry)     renderer)
       |
       v
  rbp-core, rbp-mccfr (robopoker fork)
  rbp-nlhe, rbp-cards, rbp-gameplay
```

Key files:
- `crates/myosu-games/src/traits.rs` -- GameConfig, GameType, GameParams, StrategyQuery/Response
- `crates/myosu-games/src/registry.rs` -- GameRegistry, GameDescriptor
- `crates/myosu-games-poker/src/lib.rs` -- re-exports all poker modules
- `crates/myosu-games-poker/src/wire.rs` -- serialization codecs
- `crates/myosu-games-poker/src/solver.rs` -- PokerSolver (MCCFR wrapper)

INV-004 (Solver-Gameplay Separation): `myosu-miner` and `myosu-play` must never
depend on each other. The stricter design target is still that gameplay should
share only `myosu-games` and `myosu-games-poker`, but the executable invariant
for this slice is the literal no-cross-dependency rule.

INV-006 (Robopoker Fork Coherence): The fork at `happybigmtn/robopoker` must track v1.0.0 baseline with documented changes.

## Milestones

### Milestone 1: INV-004 enforcement test

Add a test script (or cargo test) that runs `cargo tree -p myosu-play` and
`cargo tree -p myosu-miner` to verify no cross-dependency. Do not overclaim the
stricter game-only dependency target until `myosu-play` no longer pulls chain
state through `myosu-chain-client`.

Proof command:

    cargo test -p myosu-play inv_004 --quiet

### Milestone 2: Robopoker fork CHANGELOG

Create or verify CHANGELOG.md in the robopoker fork documenting all changes from v1.0.0 baseline. If the fork is local, document the diff summary in this repo at `docs/robopoker-fork-changelog.md`.

Proof command:

    test -s docs/robopoker-fork-changelog.md || echo "MISSING"

### Milestone 3: Public API audit

Review all `pub` items in `myosu-games` and `myosu-games-poker`. Add `#[non_exhaustive]` to `GameType`, `GameParams`, and any enum that will gain variants. Ensure no internal types are accidentally public.

Proof command:

    grep -rn "pub enum\|pub struct\|pub trait\|pub fn" crates/myosu-games/src/ | wc -l
    grep -rn "#\[non_exhaustive\]" crates/myosu-games/src/ | wc -l

### Milestone 4: Extensibility proof

Write a test in `myosu-games` that registers a `GameType::Custom("test-game".into())` via GameRegistry, verifies it's described correctly, and demonstrates that no modification to existing code is needed.

Proof command:

    cargo test -p myosu-games custom_game --quiet

### Milestone 5: Wire encoding edge cases

Add property-based roundtrip tests for all wire-encoded types including: empty strategy response, maximum-size info key, unicode in custom game type, and zero-probability edges.

Proof command:

    cargo test -p myosu-games-poker wire --quiet
    cargo test -p myosu-games serialization_roundtrip --quiet

## Plan of Work

1. Add INV-004 enforcement test.
2. Document robopoker fork divergence.
3. Audit and lock public API surface.
4. Write extensibility proof test.
5. Add wire encoding edge cases.

## Concrete Steps

From `/home/r/coding/myosu`:

    cargo tree -p myosu-play --edges normal 2>/dev/null | head -20
    cargo test -p myosu-games -p myosu-games-poker --quiet

## Validation and Acceptance

Accepted when:
- INV-004 enforcement test passes (no miner<->play dependency)
- Robopoker fork CHANGELOG exists
- All public enums that gain variants have `#[non_exhaustive]`
- Custom game registration test passes
- Wire encoding roundtrip tests pass for edge cases

Non-goal for this slice:
- proving that `myosu-play` is chain-agnostic. That remains future boundary
  cleanup once gameplay discovery/query seams are narrowed further.

## Idempotence and Recovery

All changes are additive (new tests, new attributes, new docs). No behavioral changes to existing code.

## Interfaces and Dependencies

Depends on: none (game crates work today).
Blocks: 008 (artifact hardening), 009 (TUI productization), 012 (multi-game proof).

```text
myosu-games traits.rs
  (audit pub surface, add #[non_exhaustive])
         |
         v
myosu-games registry.rs
  (extensibility proof test)
         |
         v
myosu-games-poker wire.rs
  (edge-case roundtrip tests)
         |
         v
docs/robopoker-fork-changelog.md
  (INV-006 compliance)
```

# Harden Game Traits and Poker Module Boundaries

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This plan follows `genesis/PLANS.md`.

## Purpose / Big Picture

The gameplay crates are the most production-ready part of the repo, but boundary contracts between generic traits and poker-specific adapters are still partly implicit. This plan makes those interfaces explicit, test-anchored, and safe for multi-game extension.

## Progress

- [x] (2026-03-28 21:30Z) Audited crate boundaries and verified test-rich gameplay surfaces.
- [ ] Freeze `myosu-games` as the stable generic contract (`traits` + `registry` + wire-safe core DTOs).
- [ ] Isolate poker-only request/query conversion logic behind explicit adapter boundaries.
- [ ] Add explicit error mapping for invalid snapshots/cards/amounts at crate boundaries.
- [ ] Add contract tests proving query/response behavior stays stable across `myosu-games` and `myosu-games-poker`.
- [ ] Update crate READMEs to document public APIs and extension points for future games.

## Surprises & Discoveries

- Observation: most behavioral coverage already exists, but API-surface intent is not centralized.
  Evidence: `crates/myosu-games/src/traits.rs`, `registry.rs`, `crates/myosu-games-poker/src/request.rs`, `robopoker.rs`.
- Observation: spec text implies some generic surfaces (`wire`, exploit adapters) that are currently poker-local.
  Evidence: `genesis/ASSESSMENT.md` against current crate layout.

## Decision Log

- Decision: keep `myosu-games` minimal and avoid moving poker-specific wire/exploit concerns into it during this plan.
  Rationale: preserves a small reusable core and avoids premature abstraction.
  Inversion (failure mode): pushing poker details into generic crate will make multi-game support harder, not easier.
  Date/Author: 2026-03-28 / Genesis

- Decision: treat README/API docs as part of acceptance criteria.
  Rationale: current strength is code quality; discoverability must match it.
  Inversion (failure mode): undocumented boundaries create accidental API drift across crates.
  Date/Author: 2026-03-28 / Genesis

## Failure Modes

| Codepath | Realistic failure | Handling |
|---|---|---|
| Snapshot -> request conversion | Invalid card or amount silently coerced | Keep strict `StrategyRequestError` mapping and add boundary tests |
| Query -> response mapping | Missing action distribution normalization | Test normalization invariants in `robopoker.rs` |
| Registry usage | Unknown game type misclassified as builtin | Add explicit registry tests for unknown/custom names |

## Outcomes & Retrospective

- Pending implementation.

## Context and Orientation

Owned files in this plan:
- `crates/myosu-games/src/lib.rs`
- `crates/myosu-games/src/traits.rs`
- `crates/myosu-games/src/registry.rs`
- `crates/myosu-games/README.md`
- `crates/myosu-games-poker/src/lib.rs`
- `crates/myosu-games-poker/src/request.rs`
- `crates/myosu-games-poker/src/robopoker.rs`
- `crates/myosu-games-poker/README.md`

Not owned here:
- Artifact checkpoint format hardening (`008`)
- TUI and interaction design (`009`)

## Milestones

### Milestone 1: Generic contract freeze

Document and lock public `myosu-games` trait and registry surfaces.

Proof command:

    cargo test -p myosu-games --quiet serialization_roundtrip
    cargo test -p myosu-games known_game_types --quiet

### Milestone 2: Poker adapter boundary

Constrain conversion logic to `request.rs` and query handling to `robopoker.rs` with explicit API docs.

Proof command:

    rg -n "pub fn (from_snapshot|query|query_with_encoder|answer)" crates/myosu-games-poker/src/request.rs crates/myosu-games-poker/src/robopoker.rs

### Milestone 3: Error contract hardening

Guarantee invalid snapshots/cards/amounts surface typed errors with stable messages.

Proof command:

    cargo test -p myosu-games-poker request_rejects_bad_cards --quiet
    cargo test -p myosu-games-poker request_rejects_non_positive_amounts --quiet

### Milestone 4: Cross-crate contract proofs

Add tests proving generic query/response semantics remain compatible with poker adapters.

Proof command:

    cargo test -p myosu-games-poker answer_roundtrips_through_wire_query --quiet
    cargo test -p myosu-games strategy_query_response_roundtrip --quiet

### Milestone 5: API docs as contract

Update crate READMEs and fail CI if core examples become stale.

Proof command:

    test -s crates/myosu-games/README.md
    test -s crates/myosu-games-poker/README.md

## Plan of Work

1. Freeze generic API surface.
2. Isolate poker adapters.
3. Strengthen error handling tests.
4. Finalize documentation contracts.

## Concrete Steps

From `/home/r/coding/myosu`:

    sed -n '1,260p' crates/myosu-games/src/traits.rs
    sed -n '1,280p' crates/myosu-games-poker/src/request.rs
    cargo test -p myosu-games -p myosu-games-poker --quiet

## Validation and Acceptance

Accepted when:
- generic and poker crate boundaries are explicit
- boundary error tests pass
- README/API guidance reflects actual public surfaces

## Idempotence and Recovery

- All proof commands are repeatable.
- If a boundary move regresses tests, revert only the affected crate and replay milestone-level changes.

## Artifacts and Notes

- Update `outputs/games/traits/review.md` and `outputs/games/poker-engine/review.md` with boundary changes.

## Interfaces and Dependencies

Depends on: `002-spec-corpus-normalization.md`
Blocks: `008-artifact-wire-checkpoint-hardening.md`, `009-play-tui-productization.md`

```text
myosu-games (generic traits + registry)
              |
              v
myosu-games-poker adapters (request/robopoker)
              |
              v
stable query-response contract tests
```

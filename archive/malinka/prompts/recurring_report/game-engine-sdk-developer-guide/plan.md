# Write developer guide: 30-Minute Game Lane — Plan

Lane: `game-engine-sdk-developer-guide`

Goal:
- Write developer guide: 30-Minute Game

Child work item of plan: Game Engine SDK Development

Objective:
Produce a step-by-step guide for implementing Kuhn Poker from scaffold to trait compliance pass

Owned surfaces:
- `crates/myosu-sdk/docs/30-minute-game.md`

Proof commands:
- `test -f crates/myosu-sdk/docs/30-minute-game.md`

Required durable artifacts:
- `spec.md`
- `review.md`

Context:
- Plan file:
- `genesis/plans/013-game-engine-sdk.md`

Child work item: `game-engine-sdk-developer-guide`

Full plan context (read this for domain knowledge, design decisions, and specifications):

# Game Engine SDK Development

**Plan ID:** 013
**Status:** New
**Priority:** HIGH — enables third-party game registration

`genesis/PLANS.md` at `genesis/PLANS.md` governs this document.

---

## Purpose / Big Picture

After this plan lands, `crates/myosu-sdk/` will provide a complete developer SDK for registering new games on the Myosu platform. A developer can implement a new game (say, Kuhn Poker or Liar's Dice) by implementing the `myosu_games::CfrGame` and `myosu_games::Encoder` traits, run the scaffold tool, pass the trait compliance test harness, and register the game — all without reading the chain source code.

---

## Progress

- [ ] Create `crates/myosu-sdk/` with scaffold tool
- [ ] Implement `cargo myosu-sdk scaffold` command
- [ ] Implement trait compliance test harness
- [ ] Implement game registration flow
- [ ] Write developer guide: "30-Minute Game" (implement Kuhn Poker in 30 min)
- [ ] Verify developer can implement Kuhn Poker without reading chain source

---

## Surprises & Discoveries

*(To be written during implementation)*

---

## Decision Log

- Decision: Permissioned registration initially, permissionless after Phase 3.
  Rationale: Early games must be vetted for correctness before being admitted to the incentive market.
  Date/Author: 2026-03-21 / Interim CEO

---

## Outcomes & Retrospective

*(To be written at plan completion)*

---

## Milestones

### M1: Create `crates/myosu-sdk/` with scaffold tool
The SDK provides a CLI that generates a new game crate with the right trait impls.

Proof: `cargo install --path crates/myosu-sdk` works; `myosu-sdk scaffold --game "liars-dice" --output /tmp/new-game` produces a valid crate.

### M2: Implement trait compliance test harness
The harness verifies a game impl satisfies all trait requirements without running the full chain.

Proof: `myosu-sdk test --crate /tmp/new-game` runs the trait compliance tests and reports pass/fail per AC.

### M3: Implement game registration flow
Register a game with the chain via the SDK.

Proof: `myosu-sdk register --game liars-dice --crate /tmp/new-game --rpc ws://localhost:9944` submits the registration tx.

### M4: Developer guide — "30-Minute Game"
Write the guide that walks a developer through implementing Kuhn Poker using the SDK.

Proof: An engineer unfamiliar with Myosu internals reads the guide, implements Kuhn Poker, and passes the trait compliance harness in under 30 minutes.

### M5: Verify "30-Minute Game" is achievable
Test the guide by having an actual developer (or a model acting as one) attempt the implementation.

Proof: The guide is testable: it includes checkable steps; after following all steps, `myosu-sdk test --crate kuhn-poker` reports all tests passing.

---

## Context and Orientation

Target SDK API:
```
myosu-sdk scaffold --game <name> --output <path>
  → Creates crates/<name>/ with src/lib.rs implementing CfrGame + Encoder

myosu-sdk test --crate <path>
  → Runs trait compliance test harness
  → Reports: CfrGame:info_set_count, Encoder:serialize, Encoder:deserialize

myosu-sdk register --game <name> --crate <path> --rpc <url>
  → Submits game registration tx to chain
```

The test harness implements `specs/031626-19-game-engine-sdk.md`'s AC-SDK-03 trait compliance test. The zero-sum check is skipped for n-player games (documented exception).

---

## Validation

- `cargo build -p myosu-sdk` passes
- `myosu-sdk scaffold --game kuhn-poker --output /tmp/test-game` produces a crate
- `myosu-sdk test --crate /tmp/test-game` runs the harness
- `myosu-sdk register` submits a registration tx
- Developer guide exists and is testable


Workflow archetype: report

Review profile: standard

Active plan:
- `genesis/plans/001-master-plan.md`

Active spec:
- `genesis/SPEC.md`

AC contract:
- Where: Developer guide walking through Kuhn Poker implementation using the SDK
- How: Produce a step-by-step guide for implementing Kuhn Poker from scaffold to trait compliance pass
- Required tests: test -f crates/myosu-sdk/docs/30-minute-game.md
- Verification plan: Guide exists with checkable steps; following all steps produces a crate that passes myosu-sdk test
- Rollback condition: Guide deleted or steps no longer produce a passing crate

Proof commands:
- `test -f crates/myosu-sdk/docs/30-minute-game.md`

Artifacts to write:
- `spec.md`
- `review.md`

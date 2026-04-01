# Implement cargo myosu-sdk scaffold command Lane — Fixup

Fix only the current slice for `game-engine-sdk-scaffold-command`.

Current Slice Contract:
Plan file:
- `genesis/plans/013-game-engine-sdk.md`

Child work item: `game-engine-sdk-scaffold-command`

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


Workflow archetype: implement

Review profile: foundation

Active plan:
- `genesis/plans/001-master-plan.md`

Active spec:
- `genesis/SPEC.md`

AC contract:
- Where: Scaffold subcommand that generates a new game crate with CfrGame + Encoder trait impls
- How: Generate a valid Rust crate with trait stub implementations from a game name
- Required tests: myosu-sdk scaffold --game liars-dice --output /tmp/new-game && cargo check --manifest-path /tmp/new-game/Cargo.toml
- Verification plan: scaffold produces a crate that passes cargo check; generated lib.rs contains CfrGame and Encoder impl stubs
- Rollback condition: scaffold output fails cargo check or is missing trait impls

Proof commands:
- `cargo build -p myosu-sdk`
- `myosu-sdk scaffold --game liars-dice --output /tmp/new-game`
- `cargo check --manifest-path /tmp/new-game/Cargo.toml`

Artifacts to write:
- `spec.md`
- `review.md`


Verification artifact must cover
- summarize the automated proof commands that ran and their outcomes

Priorities:
- unblock the active slice's first proof gate
- stay within the named slice and touched surfaces
- preserve setup constraints before expanding implementation scope
- keep implementation and verification artifacts durable and specific
- do not create or rewrite `promotion.md` during Fixup; that file is owned by the Review stage
- do not hand-author `quality.md`; the Quality Gate rewrites it after verification

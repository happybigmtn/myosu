# Implement Encoder and GameRenderer Lane — Fixup

Fix only the current slice for `liars-dice-proof-encoder-renderer`.

Current Slice Contract:
Plan file:
- `genesis/plans/014-liars-dice-proof.md`

Child work item: `liars-dice-proof-encoder-renderer`

Full plan context (read this for domain knowledge, design decisions, and specifications):

# Liar's Dice Architecture Proof

**Plan ID:** 014
**Status:** New
**Priority:** HIGH — proves multi-game architecture

`genesis/PLANS.md` at `genesis/PLANS.md` governs this document.

---

## Purpose / Big Picture

After this plan lands, Liar's Dice will be implemented as a second game engine and the multi-game architecture will be verified. Per the zero-change verification principle from `specs/031626-06-multi-game-architecture.md`, adding Liar's Dice must not require changing any shared code — not the chain, not the TUI, not the SDK, not the miner.

---

## Progress

- [ ] Implement `crates/myosu-games-liars-dice/`
- [ ] Implement `CfrGame` for Liar's Dice (147,420 terminal states)
- [ ] Implement `Encoder` and `GameRenderer`
- [ ] Register in game registry
- [ ] Verify zero-change: no shared code modified
- [ ] Verify Liar's Dice strategy query works via TUI

---

## Surprises & Discoveries

*(To be written during implementation)*

---

## Decision Log

- Decision: Liar's Dice state space is 147,420 terminal states.
  Rationale: This was calculated exactly in `specs/031626-06-multi-game-architecture.md`. Use this as the validation target.
  Date/Author: 2026-03-21 / Interim CEO

- Decision: Liar's Dice uses variable-length bid history — use `cfr_games::Copy` workaround.
  Rationale: Per `specs/031626-06-multi-game-architecture.md`, the variable-length bid history doesn't satisfy `CfrGame`'s `Copy` bound. Use the documented workaround.
  Date/Author: 2026-03-21 / Interim CEO

---

## Outcomes & Retrospective

*(To be written at plan completion)*

---

## Milestones

### M1: Implement Liar's Dice game engine
Create `crates/myosu-games-liars-dice/`. Implement `CfrGame` for 2-player Liar's Dice with 147,420 terminal states.

Proof: `cargo test -p myosu-games-liars-dice -- terminal_state_count` asserts exactly 147,420.

### M2: Implement `Encoder` and `GameRenderer`
Implement bincode serialization and TUI rendering for Liar's Dice.

Proof: `cargo test -p myosu-games-liars-dice -- roundtrip` passes; TUI renders dice state correctly.

### M3: Register in game registry
Add Liar's Dice to `myosu-games` game registry alongside NLHE.

Proof: `cargo test -p myosu-games -- game_registry` lists both NLHE and Liar's Dice.

### M4: Zero-change verification
Verify that no shared code was modified.

Proof: `git diff --stat main -- crates/myosu-chain/ crates/myosu-tui/ crates/myosu-sdk/ crates/myosu-miner/ crates/myosu-validator/` returns no changes.

### M5: Liar's Dice strategy query via TUI
Play Liar's Dice against the best bot via the TUI.

Proof: `cargo run -p myosu-play -- --game liars-dice` lets a human play one complete Liar's Dice hand.

---

## Context and Orientation

Liar's Dice game rules (simplified 2-player):
- Each player has hidden dice
- Players bid on the total count of a specific face
- A player can either raise or challenge
- Challenging reveals dice and determines winner

State space:
- 6 faces × 5 dice = 30 possible private states per player
- But order doesn't matter → C(5+6-1, 5) = C(10, 5) = 252 possible private states per player
- Total: 252 × 252 = 63,504 private state pairs
- Bid history adds depth but is bounded by game termination
- Exact count: 147,420 terminal states

---

## Validation

- `cargo test -p myosu-games-liars-dice -- terminal_state_count` asserts 147,420
- `cargo test -p myosu-games-liars-dice` all pass
- `cargo test -p myosu-games -- game_registry` shows both NLHE and Liar's Dice
- `git diff --stat main -- shared_dirs` returns empty
- `cargo run -p myosu-play -- --game liars-dice` plays Liar's Dice


Workflow archetype: implement

Review profile: standard

Active plan:
- `genesis/plans/001-master-plan.md`

Active spec:
- `genesis/SPEC.md`

AC contract:
- Where: Encoder and GameRenderer implementations within the liars-dice crate
- How: Implement bincode serialization roundtrip and TUI rendering for Liar's Dice game state
- Required tests: cargo test -p myosu-games-liars-dice -- roundtrip
- Verification plan: Roundtrip serialization test passes; TUI renders dice state correctly
- Rollback condition: Roundtrip test fails or TUI rendering produces incorrect output

Proof commands:
- `cargo test -p myosu-games-liars-dice -- roundtrip`

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

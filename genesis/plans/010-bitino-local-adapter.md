# Bitino Local Adapter and Same-TUI Pilot

This ExecPlan is a living document. The sections `Progress`,
`Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must
be kept up to date as work proceeds. This document must be maintained in
accordance with root `PLANS.md`.

## Purpose / Big Picture

After this plan is complete, the sibling Bitino repo can load a promoted Myosu
policy bundle and render one solver-backed table through the existing Bitino
TUI shell. This is the first offline same-TUI pilot from the active root master
plan: bundle-backed, local, and auditable, but not yet funded or live-miner
backed.

## Requirements Trace

- R1: `../bitino/crates/bitino-policy-canonical/` exists and can deserialize
  and verify Myosu policy bundles
- R2: Bitino gets explicit solver-backed `GameId` values rather than reusing
  deterministic ids
- R3: `bitino-play` can load a local promoted bundle and produce an
  `InteractivePresentation` for one solver-backed table
- R4: The rendered session exposes bundle/provenance metadata in the same TUI
  shell, not a separate demo UI
- R5: The pilot stays offline and bundle-backed; no funded settlement or live
  miner discovery is required

## Scope Boundaries

This plan crosses into the sibling `../bitino/` repo. It does not add funded
settlement, does not require live Myosu miner discovery, and does not modify
Bitino house-server economics. Its job is the offline local adapter and first
same-TUI table proof only.

## Progress

- [ ] Add `../bitino/crates/bitino-policy-canonical/` and wire it into the
  Bitino workspace
- [ ] Add explicit solver-backed `GameId` values to
  `../bitino/crates/bitino-engine/src/types.rs`
- [ ] Add a local bundle-backed table adapter in
  `../bitino/crates/bitino-play/src/`
- [ ] Render one solver-backed table through the existing Bitino TUI shell
- [ ] Verify the session exposes policy provenance and bundle identity

## Surprises & Discoveries

None yet.

## Decision Log

- Decision: The first Bitino milestone is offline and bundle-backed.
  Rationale: This isolates presentation, provenance, and action-sampling truth
  from funded settlement and live-miner sourcing.
  Date/Author: 2026-04-11 / active root master plan

- Decision: Solver-backed Bitino tables use explicit new `GameId` values.
  Rationale: Reusing deterministic ids like `HoldemHeadsUp` would silently
  conflate deterministic and solver-backed semantics.
  Date/Author: 2026-04-11 / active root master plan

## Outcomes & Retrospective

None yet.

## Context and Orientation

This plan depends on promoted Myosu-side bundle surfaces being real first. The
relevant locations are:

- `crates/myosu-games-canonical/src/` in Myosu for the policy-bundle contract
- `outputs/solver-promotion/<game>/` in Myosu for durable promoted bundles
- `../bitino/crates/bitino-wire/src/interactive.rs` for
  `InteractivePresentation`
- `../bitino/crates/bitino-play/src/tui/mod.rs` for the existing TUI shell
- `../bitino/crates/bitino-engine/src/types.rs` for `GameId`
- `../bitino/crates/bitino-play/src/agent/state.rs` for local session/catalog
  state

## Plan of Work

1. Keep the promoted-game artifact layout in Myosu stable under
   `outputs/solver-promotion/<game>/`.
2. Add `../bitino/crates/bitino-policy-canonical/` to deserialize and verify a
   Myosu `CanonicalPolicyBundle`.
3. Extend `../bitino/crates/bitino-engine/src/types.rs` with solver-backed
   `GameId` values such as `SolverHoldemHeadsUp` and `SolverLiarsDice`.
4. Add a local bundle-backed table adapter under
   `../bitino/crates/bitino-play/src/` that turns a verified policy bundle into
   `InteractivePresentation`.
5. Add one smoke proof that renders a solver-backed table round through the
   existing Bitino TUI shell.

## Implementation Units

### Unit 1: Bitino policy-bundle adapter

Goal: Let Bitino load and verify promoted Myosu bundles locally.
Requirements advanced: R1, R2.
Dependencies: Plans 001, 005, and 006.
Files to create or modify: `../bitino/crates/bitino-policy-canonical/`,
`../bitino/crates/bitino-engine/src/types.rs`.
Tests to add or modify: Bundle-parse and verification tests inside the new
crate.
Approach: Deserialize the Myosu bundle shape, verify it locally, and convert it
into a Bitino-local session model without linking Myosu crates directly.
Specific test scenarios:
- promoted bundle JSON loads and verifies
- invalid bundle hash is rejected
- solver-backed `GameId` values parse and roundtrip

### Unit 2: Same-TUI local table proof

Goal: Render one solver-backed table through the existing Bitino TUI shell.
Requirements advanced: R3, R4, R5.
Dependencies: Unit 1.
Files to create or modify: `../bitino/crates/bitino-play/src/solver_tables.rs`,
`../bitino/crates/bitino-play/src/agent/state.rs`,
`../bitino/crates/bitino-play/src/tui/mod.rs`.
Tests to add or modify: One local smoke proof for the solver-backed table path.
Approach: Convert a verified bundle into `InteractivePresentation`, then route
it through the normal ready-room/session shell.
Specific test scenarios:
- local promoted NLHE bundle renders as a solver-backed table session
- session metadata exposes bundle id, artifact hash, and benchmark label
- no funded settlement or house-server dependency is required

## Concrete Steps

Use the Myosu repo root for bundle-producing commands and the sibling Bitino
repo root for adapter commands.

## Verification

Run the commands below once promoted bundles exist:

    cargo run -p myosu-games-poker --example nlhe_policy_bundle -- \
      --output outputs/solver-promotion/nlhe-heads-up/bundle.json
    cargo run -p myosu-games-liars-dice --example liars_dice_policy_bundle -- \
      --output outputs/solver-promotion/liars-dice/bundle.json
    cd ../bitino
    cargo test -p bitino-policy-canonical --quiet
    cargo test -p bitino-play --features tui --quiet
    cargo run -q -p bitino-play -- --headless 1 \
      --game solver_holdem_heads_up \
      --policy-bundle ../myosu/outputs/solver-promotion/nlhe-heads-up/bundle.json

## Acceptance Criteria

- Bitino can load and verify promoted Myosu bundles locally.
- Solver-backed tables use explicit new `GameId` values.
- One solver-backed table renders through the existing Bitino TUI shell.
- The pilot stays offline and bundle-backed.

## Validation and Acceptance

1. The new Bitino policy crate loads and verifies promoted bundle JSON.
2. `bitino-play` renders a solver-backed table through the existing TUI shell.
3. Session or round metadata exposes bundle identity and provenance.

## Idempotence and Recovery

The local adapter is additive and safe to rerun. If the bundle contract changes
later, the fix should be a versioned adapter update plus regenerated promoted
bundle outputs, not silent semantic reuse of deterministic game ids.

## Artifacts and Notes

Expected Myosu inputs per promoted game:

    outputs/solver-promotion/<game>/bundle.json
    outputs/solver-promotion/<game>/benchmark-summary.json
    outputs/solver-promotion/<game>/sampling-proof-sample.json

Expected Bitino additions:

    ../bitino/crates/bitino-policy-canonical/src/model.rs
    ../bitino/crates/bitino-play/src/solver_tables.rs

## Interfaces and Dependencies

This plan depends on the promoted Myosu bundle builders and the sibling Bitino
presentation surface. It should keep the dependency direction simple: Myosu
produces bundles and Bitino consumes them.

Revision note (2026-04-11 / reconciliation pass): restored the grounded
sibling-repo adapter milestone from the active root master plan after
reconciling the corpus against the inspected Bitino presentation and `GameId`
surfaces.

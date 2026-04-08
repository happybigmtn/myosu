# Architecture Decision Records

This directory holds Architecture Decision Records for choices that constrain
the shape of Myosu and are expensive to reverse later.

ADRs complement `SPEC.md`, `THEORY.MD`, and `ops/decision_log.md`. They do not
replace those documents. Specs describe the target system, theory captures the
stage narrative, and ADRs record why a load-bearing architectural choice was
made.

## Goal

Use an ADR when Myosu is making or reaffirming a technical decision that:

- changes protocol shape, crate boundaries, trust boundaries, or operator
  behavior
- affects runtime APIs, storage, wire formats, or consensus/incentive logic
- constrains future options or is difficult to reverse once deployed
- should be understandable without re-reading large narrative documents

Do not write an ADR for a local refactor, a one-off operational response, or a
small implementation detail that does not change the system shape.

## Location And Naming

- ADRs live in `docs/adr/`
- File names use `NNN-short-kebab-title.md`
- `000-template.md` is reserved for the canonical template
- Numbers are assigned in increasing order and are never reused or renumbered
- `stage-2-roadmap.md` is a supporting roadmap document, not a numbered ADR

## Status Vocabulary

Use one of these status values near the top of every ADR:

- `Proposed`: under consideration and not yet the active repo position
- `Accepted`: the active architectural decision for the repo
- `Deprecated`: no longer preferred, but not yet formally replaced everywhere
- `Superseded by ADR-XXX`: replaced by a newer accepted ADR

Retroactive ADRs for decisions that already shape the current repo should
normally use `Accepted` and describe the decision as it stands today.

## Authoring Flow

1. Copy `docs/adr/000-template.md` to the next available numbered filename.
2. Fill in the context, decision, alternatives, consequences, and reversibility
   sections before or alongside the implementation that depends on the choice.
3. Link the ADR to the relevant specs, plans, code surfaces, and risks so the
   reasoning stays discoverable.
4. Land the ADR in the same change as the decision when practical. If the
   decision already exists, write a retroactive ADR that records the current
   state truthfully.

## When To Write One

Write an ADR before implementation when a change would:

- introduce or remove a major dependency or fork strategy
- change the protocol contract, persistence format, or chain behavior
- redefine a cross-crate seam used by miner, validator, gameplay, or chain
- alter security, privacy, determinism, or operator trust assumptions
- commit the project to a future migration path that will be costly to unwind

If the answer to "will a future contributor ask why the system is shaped this
way?" is yes, the change probably needs an ADR.

## Updating And Superseding

- Accepted ADRs are historical records. Do not rewrite them to hide the old
  decision.
- If the decision changes, create a new ADR and mark the old one as
  `Superseded by ADR-XXX`.
- Minor factual corrections, broken links, or clearer references can be edited
  in place as long as the original decision remains intact.
- Review accepted ADRs when a spec, invariant, or live proof shows the recorded
  assumptions are no longer true.

## Review Cadence

Review ADRs whenever a major architectural lane reopens and at least once per
quarter during active development. The purpose of review is to catch drift
between the recorded decision and the live repo, not to reopen settled choices
without cause.

## Index

- `000-template.md`: canonical ADR template for new records
- `001-single-token-emission.md`: single-token staking and emission model for stage-0
- `002-substrate-fork-strategy.md`: fork subtensor into an owned Substrate chain rather than deploy as a native subnet
- `003-robopoker-fork.md`: pin a narrow robopoker fork close to `v1.0.0` and wrap it locally
- `004-enum-dispatch-games.md`: use enum dispatch and registry seams for multi-game support
- `005-swap-interface-abstraction.md`: keep a stage-0 pallet-local swap seam with a noop runtime implementation
- `006-commit-reveal-v2.md`: retain hash-based commit-reveal v2 as the only live weight-hiding path
- `007-checkpoint-versioning.md`: frame solver checkpoints with explicit magic and version headers
- `008-future-token-economics-direction.md`: proposed post-stage-0 token-economics direction and migration away from the noop swap
- `009-polkadot-sdk-migration-feasibility.md`: hold the opentensor polkadot-sdk fork through stage-0 until the fork-only patch set and the 4-authority finality proof are understood
- `010-inv-004-stage0-ci-enforcement.md`: keep the solver-gameplay boundary enforced by CI and invariant tests for stage-0
- `011-emission-dust-policy.md`: close the stage-0 coinbase split remainder in the validator bucket instead of dropping it as dust
- `stage-2-roadmap.md`: supporting roadmap for the next major architecture decisions after stage 1

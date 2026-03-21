# Agent Integration Adapter

## Purpose

This document translates the reviewed `play:tui` and `agent:experience`
artifacts into the next honest product-frontier move. It is an adapter between
reviewed lane outputs and the next Fabro/Raspberry family, not a new product
spec.

## Inputs Consulted

- `README.md`
- `SPEC.md`
- `PLANS.md`
- `AGENTS.md`
- `specs/031626-00-master-index.md`
- `specs/031826-fabro-primary-executor-decision.md`
- `fabro/programs/myosu-product.yaml`
- `outputs/play/tui/spec.md`
- `outputs/play/tui/review.md`
- `outputs/agent/experience/spec.md`
- `outputs/agent/experience/review.md`
- `outputs/games/traits/implementation.md`
- `crates/myosu-games/Cargo.toml`
- `crates/myosu-tui/src/pipe.rs`
- `crates/myosu-tui/src/schema.rs`

## Product Frontier Truth

The checked-in product frontier has two reviewed bootstrap lanes:

- `play:tui`
- `agent:experience`

`fabro/programs/myosu-product.yaml` already models the dependency correctly:

- `play:tui` depends on reviewed `games:traits` and `tui:shell`
- `agent:experience` depends on `play@reviewed` and the presence of
  `docs/api/game-state.json`

That means product is no longer waiting on another bootstrap-ready review lane.
The reviewed surfaces now exist and the frontier question is what to implement
first.

## What Is Already Real

Trusted or reviewed surfaces already present in the tree:

- `docs/api/game-state.json`
- `crates/myosu-tui/src/schema.rs`
- `crates/myosu-tui/src/pipe.rs`
- `outputs/play/tui/review.md`
- `outputs/agent/experience/review.md`

These are enough to define the product-owned implementation backlog without
reopening upstream design.

## What Is Still Missing

The missing surfaces are implementation-owned:

- `crates/myosu-play/` does not exist
- `crates/myosu-tui/src/agent_context.rs` does not exist
- `crates/myosu-tui/src/journal.rs` does not exist
- `crates/myosu-tui/src/narration.rs` does not exist
- no CLI wiring exists for `--context`, `--narrate`, reflection prompts, or a
  spectator relay

These are product deliverables, not new upstream doctrine gaps.

## Superseded Blocker

`outputs/agent/experience/review.md` still lists `robopoker` absolute path
dependencies as a high blocker. That blocker is now superseded by live repo
state:

- `crates/myosu-games/Cargo.toml` uses pinned git-rev dependencies to
  `happybigmtn/robopoker`
- `outputs/games/traits/implementation.md` records the completed
  path-to-git migration

This changes the product decision. The frontier no longer needs another
portability unblock before starting product implementation.

## Recommended Next Family

Product should open an implementation family next, not another upstream
unblock.

Recommended order:

1. `play:tui` implementation family first
2. `agent:experience` implementation family second, starting as soon as
   `play:tui` Slice 1 creates `crates/myosu-play/` and a binary host for
   `--pipe`

Why this order:

- `agent:experience` owns agent context, narration, reflection, and spectator
  behavior, but most user-visible entrypoints hang off `myosu-play`
- `play:tui` reviewed artifacts already say the lane is ready for
  implementation-family work immediately
- the earliest useful agent slices become much safer once `myosu-play` exists,
  even though `agent_context.rs` and `journal.rs` can be prepared inside
  `myosu-tui`

## Recommended Supervisory Gating

- Pattern the product implementation family after
  `fabro/programs/myosu-games-traits-implementation.yaml`
- First milestone: `outputs/play/tui/implementation.md` plus
  `outputs/play/tui/verification.md`
- Gate `agent:experience` implementation on `play:tui` having a built binary
  skeleton, not on `chain:runtime`
- Allow Phase 0 lobby data to be stubbed, as already accepted in the reviewed
  `agent:experience` lane
- Defer chain-backed discovery and WebSocket spectator transport to later
  product or chain slices

## Decision

The honest adapter outcome is:

- do not reopen product for another upstream unblock
- do not invent new bootstrap lanes
- start product implementation with `play:tui`, then carry
  `agent:experience` implementation immediately behind it

## Verification Notes

I could not re-run `cargo test` in this sandbox because the inherited cargo
target path was read-only and workspace-local targets hit disk quota in `/tmp`.
This adapter therefore relies on checked-in reviewed artifacts plus direct
source inspection of the current tree.

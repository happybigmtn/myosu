# `agent:experience` Product Adapter

## Purpose

This artifact adapts the reviewed product-lane outputs into the next honest
control-plane move. It answers one question: now that `play:tui` and
`agent:experience` both have reviewed artifacts, should `myosu-product` wait
for another upstream unblock or promote product into implementation-family
work?

## Inputs Reviewed

- `README.md`
- `SPEC.md`
- `PLANS.md`
- `AGENTS.md`
- `specs/031626-00-master-index.md`
- `specs/031826-fabro-primary-executor-decision.md`
- `fabro/programs/myosu-product.yaml`
- `fabro/workflows/README.md`
- `outputs/play/tui/spec.md`
- `outputs/play/tui/review.md`
- `outputs/agent/experience/spec.md`
- `outputs/agent/experience/review.md`
- `outputs/games/traits/implementation.md`
- `outputs/games/traits/verification.md`
- concrete code surfaces under `crates/myosu-tui/src/` plus the absence of
  `crates/myosu-play/` and `crates/myosu-games-poker/`

## Current Product Truth

`fabro/programs/myosu-product.yaml` is a reviewed frontier only. It owns two
lanes, `play:tui` and `agent:experience`, and both lanes already satisfy the
current milestone contract by producing `spec.md` and `review.md`.

The product code itself is still mostly absent, which means the frontier is no
longer blocked on doctrine. It is blocked on implementation work:

- `crates/myosu-play/` does not exist
- `crates/myosu-games-poker/` does not exist
- `crates/myosu-tui/src/agent_context.rs` does not exist
- `crates/myosu-tui/src/journal.rs` does not exist
- `crates/myosu-tui/src/narration.rs` does not exist
- `crates/myosu-tui/src/screens/spectate.rs` does not exist

## What the Reviewed Artifacts Mean Together

`outputs/play/tui/review.md` makes the strongest product claim: the lane is
ready for an implementation-family workflow immediately, and its first slice is
the `myosu-play` binary skeleton. That slice is the first executable product
surface for the whole frontier.

`outputs/agent/experience/review.md` also says to proceed to an
implementation-family workflow, but its own review makes the sequencing clear:
Slices 3 through 9 depend on `play:tui` Slice 1 because the `myosu-play`
binary is the vehicle for `--pipe`, `--context`, `--narrate`, lobby, and
spectator wiring. Only the early `myosu-tui` additions such as
`agent_context.rs` and `journal.rs` are independent.

`fabro/workflows/README.md` says interface bringup such as `play:tui` belongs
in the `implement/` family when the work is still pure code/UI bringup. That
matches the current product state exactly: there is no persistent runtime to
stabilize yet, only missing crates and missing modules.

One earlier blocker cited by both product reviews is now stale. The reviews
flagged `myosu-games`'s absolute-path robopoker dependency as a HIGH blocker.
`outputs/games/traits/verification.md` shows that this was already reduced in
the first `games:traits` implementation slice: `crates/myosu-games/Cargo.toml`
now uses pinned git dependencies and `cargo fetch` succeeded. Product should
not keep waiting on that already-reduced blocker.

## Adapter Decision

`myosu-product` needs an implementation family next, not another upstream
unblock.

The first product implementation lane should be `play:tui`, not
`agent:experience`, for one reason: it creates the binary and runtime surface
that the rest of product extends. `agent:experience` is a real downstream lane,
but it is an extension of the play surface, not the first executable product
entrypoint.

That yields the honest sequence:

1. Seed a lane-scoped `play:tui` implementation family in `fabro/`.
2. Deliver `play:tui` Slice 1 (`myosu-play` binary skeleton + shell wiring).
3. Seed `agent:experience` implementation-family work immediately after that,
   with optional overlap on the independent `myosu-tui` slices only.

## Recommended Control-Plane Follow-On

The next contributor should add product implementation surfaces that mirror the
existing `games:traits` pattern rather than inventing another bootstrap pass.

Recommended first additions:

1. `fabro/workflows/implement/play-tui.fabro`
2. `fabro/run-configs/implement/play-tui.toml`
3. a lane-scoped implementation manifest mirroring
   `fabro/programs/myosu-games-traits-implementation.yaml`, but targeting
   `outputs/play/tui/`

That implementation family should consume:

- `outputs/play/tui/spec.md`
- `outputs/play/tui/review.md`

and produce:

- `outputs/play/tui/implementation.md`
- `outputs/play/tui/verification.md`

Once `play:tui` Slice 1 is verified, mirror the same pattern for
`agent:experience`.

## What Not To Do

Do not create another upstream-unblock lane for product. The reviewed upstream
inputs already exist, and the earlier robopoker portability blocker has already
been reduced.

Do not wait for `chain:runtime` before starting product implementation. The
reviewed `play:tui` slices are explicitly ordered so local training mode lands
before chain-connected play.

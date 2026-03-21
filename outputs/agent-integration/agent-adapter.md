# Agent Integration Adapter

Date: 2026-03-20

## Purpose

This adapter turns the reviewed `agent:experience` lane into the next honest
Fabro/Raspberry move for the product frontier.

The decision target is narrow: after the last ready product lane completed its
bootstrap review, should product pause for another upstream unblock, or should
it open an implementation family next?

## Inputs Used

- `README.md`
- `SPEC.md`
- `PLANS.md`
- `AGENTS.md`
- `specs/031626-00-master-index.md`
- `specs/031826-fabro-primary-executor-decision.md`
- `fabro/programs/myosu.yaml`
- `fabro/programs/myosu-product.yaml`
- `fabro/programs/myosu-games-traits-implementation.yaml`
- `fabro/run-configs/product/play-tui.toml`
- `fabro/run-configs/product/agent-experience.toml`
- `fabro/run-configs/implement/game-traits.toml`
- `fabro/workflows/bootstrap/play-tui.fabro`
- `fabro/workflows/bootstrap/agent-experience.fabro`
- `fabro/workflows/implement/game-traits.fabro`
- `outputs/play/tui/spec.md`
- `outputs/play/tui/review.md`
- `outputs/agent/experience/spec.md`
- `outputs/agent/experience/review.md`
- `outputs/games/traits/review.md`
- `outputs/games/traits/implementation.md`
- `outputs/tui/shell/review.md`
- `Cargo.toml`
- `crates/myosu-games/Cargo.toml`
- `crates/myosu-tui/src/`

## Execution Surface Checked

`agent:experience` is already wired into the product frontier as unit `agent`,
lane `experience`, in `fabro/programs/myosu-product.yaml`. Its checked-in run
config, `fabro/run-configs/product/agent-experience.toml`, still points to the
bootstrap workflow `fabro/workflows/bootstrap/agent-experience.fabro` and
produces only `outputs/agent/experience/spec.md` and
`outputs/agent/experience/review.md`.

The same product program contains only one other unit: `play:tui`. Both product
lanes are review-shaped today. There is no checked-in product implementation
program, no `fabro/workflows/implement/play-tui.fabro`, and no
`fabro/workflows/implement/agent-experience.fabro`.

The only checked-in implementation-family pattern in the repo today is
`games:traits`, via `fabro/programs/myosu-games-traits-implementation.yaml`.
That is the nearest honest template for what product should do next.

No repo-local `.raspberry/` state exists in this worktree, so this adapter uses
committed manifests, curated outputs, and fresh proof commands as the current
truth source.

## Decision

Product should open an implementation family next. It does not need another
upstream-unblock cycle first.

## Why This Is Implementation Next

1. `outputs/agent/experience/review.md` explicitly says to proceed to an
   implementation-family workflow next.
2. `outputs/play/tui/review.md` says `play:tui` is ready for an
   implementation-family workflow immediately.
3. The strongest blocker still named by those product reviews, robopoker
   absolute-path coupling, is no longer live. `crates/myosu-games/Cargo.toml`
   already uses pinned git rev dependencies, and
   `outputs/games/traits/implementation.md` records that migration.
4. The remaining gaps are product-owned code surfaces, not missing upstream
   doctrine:
   `crates/myosu-play/`, `crates/myosu-games-poker/`,
   `crates/myosu-tui/src/agent_context.rs`,
   `crates/myosu-tui/src/journal.rs`,
   `crates/myosu-tui/src/narration.rs`, and later spectator modules.
5. `AGENTS.md` says not to widen the bootstrap manifest and says new execution
   work should land as Fabro assets plus Raspberry program updates. A separate
   product implementation family follows that doctrine better than reopening the
   bootstrap frontier.

## Fresh Verification

Fresh proof from this run:

- `CARGO_TARGET_DIR=/tmp/myosu-cargo-target cargo test -p myosu-games`
  passed with 10 unit tests and 4 doctests.
- `CARGO_TARGET_DIR=/tmp/myosu-cargo-target cargo test -p myosu-tui`
  passed with 84 unit tests and 1 doctest.
- `crates/myosu-games/Cargo.toml` pins both `rbp-core` and `rbp-mccfr` to
  `https://github.com/happybigmtn/robopoker` at
  `04716310143094ab41ec7172e6cea5a2a66744ef`.
- `crates/myosu-play/` is still missing.
- `crates/myosu-games-poker/` is still missing.
- `crates/myosu-tui/src/agent_context.rs`,
  `crates/myosu-tui/src/journal.rs`, and
  `crates/myosu-tui/src/narration.rs` are still missing.
- `Cargo.toml` still sets workspace `edition = "2024"` and there is still no
  checked-in `rust-toolchain.toml`.

## Recommended Control-Plane Shape

Match the existing `games:traits` implementation-family pattern.

Add:

- `fabro/programs/myosu-product-implementation.yaml`
- `fabro/workflows/implement/play-tui.fabro`
- `fabro/run-configs/implement/play-tui.toml`
- `fabro/workflows/implement/agent-experience.fabro`
- `fabro/run-configs/implement/agent-experience.toml`

Use the existing product output roots rather than creating a new taxonomy:

- `outputs/play/tui/implementation.md`
- `outputs/play/tui/verification.md`
- `outputs/agent/experience/implementation.md`
- `outputs/agent/experience/verification.md`

## First Honest Product Slices

`play:tui`

- Slice 1: create `crates/myosu-play`
- wire the minimal shell-facing binary
- prove `cargo build -p myosu-play`

`agent:experience`

- Slice 1: add `crates/myosu-tui/src/agent_context.rs`
- Slice 2: add `crates/myosu-tui/src/journal.rs`

After `play:tui` Slice 1 lands:

- `agent:experience` Slice 3: `--context` wiring
- `agent:experience` Slice 4: `reflect>` prompt

## Caveats To Carry Forward

- `outputs/agent/experience/review.md` still names the old robopoker blocker.
  Treat that as stale bootstrap review language, not as a current gate.
- No repo-local `.raspberry/` state is present, so control-plane readiness is
  currently inferred from committed artifacts and fresh proof rather than live
  Raspberry state.
- The workspace toolchain policy is still unresolved, but it is not specific
  enough to delay the first product implementation round.

## Bottom Line

The reviewed `agent:experience` slice did its job. Product is past the point
where another upstream review loop is the honest next move. The honest next
move is a product implementation family.

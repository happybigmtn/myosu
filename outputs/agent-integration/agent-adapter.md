# Agent Integration Adapter

## Purpose

This artifact adapts the reviewed `agent:experience` lane into the current
Myosu product frontier so the next operator does not have to re-derive what
belongs in product, what is already trusted, and what sequence is honest for
implementation.

The adapter consumes the reviewed lane artifacts at:

- `outputs/play/tui/spec.md`
- `outputs/play/tui/review.md`
- `outputs/agent/experience/spec.md`
- `outputs/agent/experience/review.md`

It also reconciles those artifacts against the current workspace state,
`fabro/programs/myosu-product.yaml`, and the current doctrine in `README.md`,
`SPEC.md`, `PLANS.md`, `AGENTS.md`,
`specs/031626-00-master-index.md`, and
`specs/031826-fabro-primary-executor-decision.md`.

## Product Boundary After `agent:experience` Review

The product frontier currently has two reviewed lanes:

1. `play:tui` owns the existence of the player-facing product binary and the
   baseline gameplay loop.
2. `agent:experience` owns the agent-specific extensions that sit on top of
   that gameplay loop.

The durable boundary is:

- `play:tui` owns `myosu-play`, the NLHE renderer, local training mode, the
  solver-advisor panel, and the baseline `--pipe` transport.
- `agent:experience` owns the agent context file, append-only journal,
  reflection prompt, narration mode, pipe-mode lobby behavior, and spectator
  relay/screen.
- `tui:shell` remains the trusted rendering/input shell underneath both lanes.
- `games:traits` remains the trusted game-interface layer underneath both
  lanes.

This means product does not need a third reviewed contract lane before real
implementation starts. The product contract surface is already present; what is
missing is code.

## Live Workspace Truth

The current workspace confirms the following:

- `crates/myosu-tui/` exists and `cargo test -p myosu-tui` passes when run with
  a writable local `CARGO_TARGET_DIR`. Result on 2026-03-21: 82 tests passed,
  2 ignored, 1 doctest passed.
- `crates/myosu-games/` exists and `cargo test -p myosu-games` passes when run
  with the same writable local `CARGO_TARGET_DIR`. Result on 2026-03-21:
  10 unit tests passed, 4 doctests passed.
- `crates/myosu-games/Cargo.toml` now uses pinned robopoker git dependencies,
  not absolute filesystem paths.
- `crates/myosu-play/` is still absent.
- `crates/myosu-games-poker/` is still absent.
- `crates/myosu-tui/src/agent_context.rs`,
  `crates/myosu-tui/src/journal.rs`,
  `crates/myosu-tui/src/narration.rs`,
  `crates/myosu-play/src/spectate.rs`, and
  `crates/myosu-tui/src/screens/spectate.rs` are still absent.

The important adapter consequence is that the largest live product blocker is
now the missing `myosu-play` and gameplay crates, not an unresolved upstream
dependency contract.

## Cleared vs Active Blockers

### Cleared since the existing lane reviews were written

- The older `play:tui` and `agent:experience` reviews both carry forward the
  earlier warning that `myosu-games` still used absolute robopoker path
  dependencies.
- That warning is now stale. `crates/myosu-games/Cargo.toml` points to
  `git = "https://github.com/happybigmtn/robopoker"` with pinned `rev`
  entries for both `rbp-core` and `rbp-mccfr`, and the crate tests pass.

### Still active

- `play:tui` Slice 1 has not happened yet because `crates/myosu-play/` does
  not exist.
- `play:tui` Slice 2 has not happened yet because `crates/myosu-games-poker/`
  does not exist.
- `agent:experience` Slices 3, 6, 7, 8, and 9 cannot finish until `myosu-play`
  exists, because their public surface is CLI wiring or spectator plumbing.
- The future chain-connected product path is still blocked on chain/runtime
  work, but that does not block the first product implementation family.

## Honest Product Sequencing

The right sequencing is not "finish `play:tui`, then start
`agent:experience`." The honest product sequencing is a small coupled
implementation family where the foundational slices come first and the
agent-facing slices layer on as soon as their host surface exists.

### Phase A: start product implementation immediately

- `play:tui` Slice 1: create `crates/myosu-play/` and wire a minimal
  `myosu-play --train` skeleton to `myosu-tui`.
- `agent:experience` Slice 1: add `crates/myosu-tui/src/agent_context.rs`.
- `agent:experience` Slice 2: add `crates/myosu-tui/src/journal.rs`.

These three slices do not require chain runtime work. They are the earliest
honest implementation-family start for product.

### Phase B: prove the product host surface

- `play:tui` Slice 2: create `crates/myosu-games-poker/` with an
  `NlheRenderer` and `pipe_output()`.

Once this lands, the product frontier has a real gameplay host for the agent
lane instead of only a shell contract.

### Phase C: wire the first agent-visible behavior

- `agent:experience` Slice 3: wire `--context` through `myosu-play` into
  `PipeMode`.
- `agent:experience` Slice 4: add the `reflect>` prompt and journal append on
  hand completion.

These are the first slices where the agent lane becomes externally visible to a
real `myosu-play` binary.

### Phase D: enrich the agent experience after local play exists

- `play:tui` Slice 3: training table and heuristic bot.
- `agent:experience` Slice 5: narration engine.
- `agent:experience` Slice 6: `--narrate` CLI wiring.
- `agent:experience` Slice 7: pipe-mode lobby / game selection.

The narration and lobby slices are more valuable once the local product loop
can actually play hands.

### Phase E: add spectator surfaces after the local session loop is stable

- `agent:experience` Slice 8: local spectator relay in `myosu-play`.
- `agent:experience` Slice 9: spectator screen in `myosu-tui`.

The relay/screen pair is product-facing, but it is not the first thing product
needs to prove.

## Product Decision This Adapter Supports

This adapter supports a product-level decision of:

**Start a product implementation family next. Do not wait for another reviewed
upstream unblock.**

That judgment is honest because:

- the product program already has reviewed artifacts for both owned lanes
- the trusted upstream lanes (`games:traits`, `tui:shell`) are present and
  passing
- the previously cited robopoker path blocker is no longer live
- the next missing work is implementation work inside product-owned crates

The remaining caveat is not "wait." The caveat is "sequence the product family
correctly," with `play:tui` Slice 1 as the first host-surface commit and
`agent:experience` Slices 1-2 allowed to begin in parallel.

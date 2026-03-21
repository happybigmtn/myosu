# `agent-integration` Agent Adapter

## Purpose

This note adapts the reviewed `agent:experience` lane into the next honest
product action. The question is whether product still needs another upstream
unblock or whether it should move into an implementation-family workflow.

## Inputs Used

- `README.md`
- `SPEC.md`
- `PLANS.md`
- `AGENTS.md`
- `specs/031626-00-master-index.md`
- `specs/031826-fabro-primary-executor-decision.md`
- `outputs/agent/experience/spec.md`
- `outputs/agent/experience/review.md`
- `outputs/play/tui/spec.md`
- `outputs/play/tui/review.md`
- `outputs/games/traits/implementation.md`
- `outputs/games/traits/verification.md`
- the current repo state in `Cargo.toml`, `crates/myosu-games/Cargo.toml`,
  `crates/myosu-tui/src/lib.rs`, `crates/myosu-tui/src/pipe.rs`, and
  `fabro/programs/myosu-product.yaml`

## Current Repo Truth

1. `agent:experience` already reached the correct lane judgment:
   implementation-family next. Its blocker list is no longer fully current,
   though. The review still warns about robopoker absolute-path coupling, but
   `crates/myosu-games/Cargo.toml` now pins `rbp-core` and `rbp-mccfr` to
   `https://github.com/happybigmtn/robopoker` at rev
   `04716310143094ab41ec7172e6cea5a2a66744ef`.

2. The trusted upstream crates are healthy in the current workspace:
   - `CARGO_TARGET_DIR=/tmp/myosu-agent-integration-target cargo test -p myosu-games --quiet`
     passed with 10 unit tests and 4 doctests.
   - `CARGO_TARGET_DIR=/tmp/myosu-agent-integration-target cargo test -p myosu-tui --quiet`
     passed with 82 tests, 2 ignored tests, and 1 doctest.

3. The product control plane stops at reviewed artifacts. `fabro/programs/myosu-product.yaml`
   only defines `spec.md` and `review.md` artifacts and only manages the
   `reviewed` milestone for both `play:tui` and `agent:experience`.

4. The remaining gaps are product implementation gaps, not missing doctrine:
   - `Cargo.toml` still comments out `crates/myosu-play`.
   - `crates/myosu-play/` does not exist.
   - `crates/myosu-games-poker/` does not exist.
   - `crates/myosu-tui/src/lib.rs` exports no `agent_context`, `journal`, or
     `narration` modules.
   - `crates/myosu-tui/src/pipe.rs` is still the minimal stdin/stdout bridge,
     with no `--context`, `reflect>`, or narration wiring.

## Decision

**Product needs an implementation family next. It does not need another
upstream unblock before the first honest product slice.**

## Why This Is the Honest Next Move

- The Phase 0 upstream contract lanes that product depends on are already in
  place and proven enough to build on: `games:traits` is now portable and
  tested, and `tui:shell` is green.
- The main blocker that remains for agent-facing product work is internal to
  product: the absence of `myosu-play` and the first concrete play runtime.
- Chain-backed discovery, live subnet metrics, and miner-axon spectator
  transport are later-phase enhancements. They are not required for the first
  local product slice.
- Another review-only pass would mostly restate known missing files.

## Recommended Family Shape

Create a dedicated product implementation family rather than widening the
existing bootstrap manifest.

Recommended surfaces:

- `fabro/run-configs/implement/play-tui.toml`
- `fabro/workflows/implement/play-tui.fabro`
- `fabro/prompts/implement/play-tui-implement.md`
- `fabro/prompts/implement/play-tui-fix.md`
- `fabro/prompts/implement/play-tui-review.md`
- `outputs/play/tui/implementation.md`
- `outputs/play/tui/verification.md`
- `fabro/run-configs/implement/agent-experience.toml`
- `fabro/workflows/implement/agent-experience.fabro`
- `fabro/prompts/implement/agent-experience-implement.md`
- `fabro/prompts/implement/agent-experience-fix.md`
- `fabro/prompts/implement/agent-experience-review.md`
- `outputs/agent/experience/implementation.md`
- `outputs/agent/experience/verification.md`

The program shape can stay lane-centric, mirroring the existing
`games:traits` implementation family, or use one dedicated product
implementation manifest with separate `play` and `agent` units. The important
part is the milestone contract: reviewed input, then implemented output, then
verified output.

## Recommended Execution Order

1. **Start `play:tui` implementation Slice 1 first.**
   The first gate is to create `crates/myosu-play`, add it to the workspace,
   and prove `cargo build -p myosu-play`.

2. **Then run `agent:experience` Slice 1-2 as soon as the family exists.**
   `agent_context.rs` and `journal.rs` can live inside `myosu-tui` without
   waiting for chain integration or spectator transport.

3. **Gate `agent:experience` Slice 3+ on `myosu-play` buildability.**
   `--context`, `--narrate`, the reflection prompt, and the lobby wiring all
   need the `myosu-play` CLI entrypoint.

4. **Keep spectator and live lobby data as follow-on slices.**
   The spectator relay and miner-backed lobby data should stay behind the first
   local product loop. If needed, the lobby can start with stub data exactly as
   the reviewed `agent:experience` lane already recommended.

## Reopen Conditions

Reopen the "upstream unblock" question only if one of these proves true during
implementation:

- `myosu-play` cannot be added without changing trusted upstream contracts in
  `myosu-tui` or `myosu-games`
- the current `GameRenderer` / `PipeMode` contract cannot support the first
  NLHE product slice
- `agent_context` or `journal` work uncovers a missing durable surface in the
  trusted shell that the current review artifacts did not account for

Until one of those conditions appears, the honest control-plane move is
implementation family, not another unblock pass.

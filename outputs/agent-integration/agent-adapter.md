# `agent-integration` Adapter

## Purpose

This artifact adapts the reviewed product-lane contracts into the next honest
execution move for the product frontier. The question is narrow:

- after `outputs/play/tui/review.md` and `outputs/agent/experience/review.md`
- and after confirming the current tree still matches those reviews

should Myosu pause for another upstream unblock, or should product open an
implementation-family workflow next?

## Inputs Consumed

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

## Current Product Truth

| Surface | Reviewed judgment | Integration consequence |
|---|---|---|
| `play:tui` | **KEEP**; ready for implementation-family workflow | Product already has one lane whose next honest step is code delivery |
| `agent:experience` | **KEEP**; proceed to implementation-family workflow | The lane is design-complete; only part of it depends on `play:tui` Slice 1 |
| `myosu-product.yaml` | Both reviewed product lanes already exist under one frontier | No new product bootstrap lane is needed before execution begins |
| Current tree | `crates/myosu-play/`, `crates/myosu-games-poker/`, `agent_context.rs`, `journal.rs`, `narration.rs`, and spectator files are all absent | The reviewed artifacts are still truthful and current |

## Adapter Decision

**Decision: product needs an implementation family next, not another upstream
unblock.**

The first implementation family should be **`play:tui`**, not a combined
play-plus-agent delivery lane. `play:tui` owns the first shared product seam:
the `myosu-play` binary, the concrete poker renderer, and the `--pipe` runtime
surface that `agent:experience` extends.

That means the integration adapter for product is:

1. Open a `play:tui` implementation-family workflow using the existing
   `implement/` family pattern already proven by
   `fabro/programs/myosu-games-traits-implementation.yaml`.
2. Treat `agent:experience` as the immediate downstream consumer of that work,
   not as a reason to delay execution.
3. Once `play:tui` Slice 1 lands, either:
   - open a dedicated `agent:experience` implementation-family workflow, or
   - intentionally widen the product implementation frontier if the control
     plane needs one shared product-delivery surface.

## Why Another Upstream Unblock Is Not The Next Move

The reviewed artifacts do document real blockers, but none of them justify
stopping product execution before the first implementation slice.

### 1. The biggest near-term dependency is product-owned, not upstream-owned

`agent:experience` Slice 3 and later require the `myosu-play` binary to exist.
That binary belongs to `play:tui` Slice 1. This is a product sequencing issue,
not a missing upstream review.

### 2. The `robopoker` path dependency is real, but it blocks later slices

`play:tui/review.md` and `agent:experience/review.md` both preserve the
`robopoker` absolute-path risk in `myosu-games`. That is still an upstream
maintenance task, but it does not block the first honest product bringup:

- `play:tui` Slice 1 is binary skeleton + shell wiring
- `play:tui` Slice 2 is hardcoded `NlheRenderer`
- `agent:experience` Slices 1-2 are `agent_context.rs` + `journal.rs`

Those slices can begin without waiting for a separate upstream-unblock-only
round.

### 3. Chain restart is not on the critical path for Phase 0 product work

Both product reviews treat chain-backed play, live subnet discovery, and miner
integration as later slices. The first product implementation family is local
training + pipe + renderer work.

## Recommended Execution Order

### Phase A: open `play:tui` implementation family now

Start with the smallest shared seam:

- create `crates/myosu-play/`
- create `crates/myosu-games-poker/`
- prove the shell render loop and `--pipe` entrypoint compile

This is the first product move that creates new executable surface instead of
restating existing design.

### Phase B: begin the non-CLI-dependent agent slices immediately after

Once product execution is active, the next smallest agent slices are:

- `crates/myosu-tui/src/agent_context.rs`
- `crates/myosu-tui/src/journal.rs`

These do not require chain work and do not need the full spectator stack.

### Phase C: consume the new play seam from `agent:experience`

After `play:tui` Slice 1 exists:

- wire `--context`
- wire `reflect>`
- wire `--narrate`
- add lobby behavior in pipe mode

After `play:tui` has a stable data-dir/session convention:

- add `SpectatorRelay`
- add `SpectateScreen`

## What Would Reopen This Adapter

Revisit this decision only if one of these becomes true:

- `play:tui` Slice 1 cannot be started without first resolving the
  `robopoker` dependency in `myosu-games`
- the `GameRenderer` or `PipeMode` contract changes underneath the reviewed
  product specs
- Raspberry needs a different control-plane shape than one lane-scoped
  implementation family followed by another

Absent one of those conditions, the next honest frontier move is to begin
product implementation, led by `play:tui`.

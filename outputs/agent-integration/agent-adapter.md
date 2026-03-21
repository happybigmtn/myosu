# `agent-integration` Adapter

**Frontier**: `product`  
**Date**: 2026-03-20

## Purpose

This artifact translates the reviewed `agent:experience` lane into the next
executable product move.

The product frontier now has both reviewed bootstrap lanes checked in:

- `outputs/play/tui/spec.md`
- `outputs/play/tui/review.md`
- `outputs/agent/experience/spec.md`
- `outputs/agent/experience/review.md`

The question is no longer "what should product own?" The question is "what is
the smallest honest implementation sequence that turns those reviewed contracts
into working code?"

## Reviewed Inputs Used

- `README.md`
- `SPEC.md`
- `PLANS.md`
- `AGENTS.md`
- `specs/031626-00-master-index.md`
- `specs/031826-fabro-primary-executor-decision.md`
- `outputs/play/tui/spec.md`
- `outputs/play/tui/review.md`
- `outputs/agent/experience/spec.md`
- `outputs/agent/experience/review.md`
- `fabro/programs/myosu-product.yaml`

## Current Product Substrate

The trustworthy substrate is stronger than the earlier bootstrap reviews alone
suggested.

### Trusted now

| Surface | Current evidence | Why it matters |
|---|---|---|
| `crates/myosu-games/` | `cargo test -p myosu-games` passed on 2026-03-20: 10 unit tests + 4 doctests | Product-facing play code can depend on the shared game trait surface now |
| `crates/myosu-tui/` | `cargo test -p myosu-tui` passed on 2026-03-20: 82 tests passed, 2 TTY-only tests ignored | Shell, renderer contract, pipe mode, schema, and navigation state machine remain trustworthy |
| `docs/api/game-state.json` + `crates/myosu-tui/src/schema.rs` | Present and covered by the `myosu-tui` test suite | The machine-readable agent surface already exists |
| `crates/myosu-tui/src/pipe.rs` | Present, minimal, and tested | Agent integration should extend this surface, not redesign it |
| `crates/myosu-tui/src/screens.rs` | `Lobby` and `Spectate` states already exist in the navigation state machine | Spectator and lobby work have a shell foothold even though the full product flow is not implemented |

### Still missing

| Surface | Current state | Consequence |
|---|---|---|
| `crates/myosu-play/` | Missing | No consumer-facing product binary exists yet |
| `crates/myosu-games-poker/` | Missing | No concrete NLHE renderer exists for the trusted shell to host |
| `crates/myosu-tui/src/agent_context.rs` | Missing | No persistent agent identity/memory surface yet |
| `crates/myosu-tui/src/journal.rs` | Missing | No append-only agent journal yet |
| `crates/myosu-tui/src/narration.rs` | Missing | No `--narrate` mode yet |
| Product implementation family assets in `fabro/` | Missing | Product has reviewed bootstrap lanes, but no checked-in implement loop like `games:traits` has |

## What Changed Since the Earlier `agent:experience` Review

The largest blocker named in `outputs/agent/experience/review.md` is now stale:
the `robopoker` absolute-path coupling in `crates/myosu-games/` has already
been fixed in the working tree.

`crates/myosu-games/Cargo.toml` now uses pinned git dependencies:

- `rbp-core = { git = ".../happybigmtn/robopoker", rev = "04716310143094ab41ec7172e6cea5a2a66744ef" }`
- `rbp-mccfr = { git = ".../happybigmtn/robopoker", rev = "04716310143094ab41ec7172e6cea5a2a66744ef" }`

That matters because it changes the product decision. The product frontier is
not waiting on another upstream portability unblock before it can start.

## Adapter Decision

Product needs an implementation family next.

Not another upstream bootstrap lane. Not another review-only pass. The missing
thing is the checked-in delivery loop that can turn the reviewed `play:tui` and
`agent:experience` contracts into `implementation.md` and `verification.md`
artifacts.

## Smallest Honest Product Sequence

### Step 1: promote `play:tui` into the first product implementation lane

Start with `play:tui`, not `agent:experience`.

Reason:

- `play:tui` owns the missing `myosu-play` binary
- `play:tui` owns the missing poker renderer crate
- `agent:experience` slices 3 and later explicitly depend on that binary
- the first product proof should be "the product can render and host a game",
  not "the agent layer has richer metadata over a nonexistent binary"

The approved first slice remains the one already captured in
`outputs/play/tui/spec.md`:

- create `crates/myosu-play/`
- wire a minimal `--train` flow into the trusted `myosu-tui` shell
- prove the render loop compiles and runs

### Step 2: once `myosu-play` exists, promote `agent:experience`

After the binary skeleton exists, `agent:experience` becomes a normal
implementation continuation rather than a blocked contract lane.

Recommended early slice order:

1. `agent_context.rs`
2. `journal.rs`
3. `--context` wiring through `pipe.rs` and the `myosu-play` CLI
4. `reflect>` prompt after hand completion

This keeps agent work attached to a real product binary instead of creating an
agent-only abstraction layer with no live host.

### Step 3: hold later product slices behind the right gates

These remain deferred, but they are not reasons to delay the implementation
family itself:

- `--narrate` depends on real gameplay state being available through the binary
- lobby detail can use Phase 0 stub data before chain discovery exists
- spectator relay should wait until the play binary owns a real session loop
- chain-connected play remains a later phase gated by `chain:runtime`

## Practical Control-Plane Handoff

The current `fabro/` tree already proves the pattern to copy:

- bootstrap family for contract lanes
- implement family for trusted code lanes

`games:traits` already has:

- `fabro/workflows/implement/game-traits.fabro`
- `fabro/run-configs/implement/game-traits.toml`
- `fabro/programs/myosu-games-traits-implementation.yaml`

Product needs the same kind of handoff next, starting with a lane-scoped
implementation program for `play:tui`. After that first product implementation
slice lands, `agent:experience` should get its own implementation loop or join
the same product delivery family with lane-specific milestones.

## Recommended Product Decision

### Choose implementation family next

The honest next product move is:

1. seed a `play:tui` implementation-family workflow/program
2. execute `play:tui` Slice 1
3. use that new binary surface to begin `agent:experience` implementation

### Do not spend the next slice on another upstream unblock

The remaining blockers are mostly product-local:

- missing product crates
- missing product implementation workflow assets
- missing CLI wiring

Those are implementation problems, not frontier-selection problems.

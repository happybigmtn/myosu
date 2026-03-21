# Agent Integration Adapter

## Purpose

This note adapts the reviewed `agent:experience` lane into the next honest
product move. It reconciles the reviewed artifacts with the current repository
state and turns that into a clear sequencing rule for the product frontier.

## Inputs Used

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
- the current worktree under `crates/`, `docs/`, `fabro/`, and `outputs/`

## Reconciled Current Truth

| Surface | Current repo truth | Adapter consequence |
|---|---|---|
| Agent JSON contract | `docs/api/game-state.json` exists and `crates/myosu-tui/src/schema.rs` is implemented. `cargo test -p myosu-tui schema::tests` passed with 12 tests. | The machine-readable agent surface is trusted enough to implement against. |
| Pipe transport skeleton | `crates/myosu-tui/src/pipe.rs` exists and `cargo test -p myosu-tui pipe::tests` passed with 5 tests. | The text transport contract already exists as a shell-level boundary. |
| Screen routing groundwork | `crates/myosu-tui/src/screens.rs` already models `Lobby`, `Game`, and `Spectate` transitions. | The product shell already has a place for spectator and lobby flows, even though the concrete screens are not built. |
| Gameplay binary | `crates/myosu-play/` does not exist and the workspace still comments out `crates/myosu-play` in `Cargo.toml`. | There is still no executable host for `--pipe`, `--context`, `--narrate`, or `--spectate`. |
| Poker renderer | `crates/myosu-games-poker/` does not exist. | There is still no concrete `GameRenderer` for the agent surfaces to attach to. |
| Agent persistence and narration modules | `crates/myosu-tui/src/agent_context.rs`, `narration.rs`, and `journal.rs` do not exist. | The reviewed `agent:experience` contract still represents real implementation work, not already-landed code. |
| Prior robopoker portability blocker | `crates/myosu-games/Cargo.toml` now uses git-pinned `rbp-core` and `rbp-mccfr`, and `cargo test -p myosu-games` passed with 10 unit tests and 4 doctests. | The older “absolute path dependency” blocker in neighboring reviews is stale. It is no longer a reason to delay product execution. |
| Product implementation-family assets | Search of `fabro/programs/`, `fabro/run-configs/`, and `fabro/workflows/` found no product implementation-family manifest. Only `myosu-games-traits-implementation.yaml` exists today. | The next honest control-plane addition is a product implementation family, not another review-only pass. |

## Adapter Decision

Product should move into an implementation family next.

The first slice in that family should be `play:tui`, not `agent:experience`.
`agent:experience` is an adapter on top of a gameplay binary plus a concrete
renderer. Today, both are missing:

- no `crates/myosu-play`
- no `crates/myosu-games-poker`
- no workspace membership for `myosu-play`

That means the correct sequence is:

1. land `play:tui` Slice 1 and Slice 2 so the product frontier has a real
   executable surface and a concrete poker renderer
2. resume with `agent:experience` implementation slices on top of that binary
   and renderer

This is not “another upstream unblock.” The reviewed upstream contracts already
exist and pass targeted verification in the current worktree. The remaining
hard gaps are now inside the product frontier itself.

## Product Sequencing Rule

### Step 1: Start `play:tui` implementation family

Required outcome:

- create `crates/myosu-play`
- create `crates/myosu-games-poker`
- add `crates/myosu-play` back to the workspace
- prove a real `myosu-play --train` and `myosu-play --pipe` entry surface

Why this goes first:

- `outputs/play/tui/review.md` already judged the lane implementation-ready
- `outputs/agent/experience/review.md` still depends on the `myosu-play` binary
  for slices 3 and beyond
- the current repo no longer has the previously cited external robopoker path
  blocker

### Step 2: Start `agent:experience` implementation family

First honest slices after the binary exists:

- `agent_context.rs`
- `journal.rs`
- `--context` flag wiring
- `reflect>` prompt in pipe mode

Second-wave slices once the renderer is real:

- `narration.rs`
- `--narrate`
- lobby-driven game selection in pipe mode
- spectator relay and spectator screen

### Step 3: Reopen upstream only if a new hard dependency appears

Do not insert another upstream unblock by default. Only reopen upstream if the
first `play:tui` implementation slice uncovers a new hard dependency that is
not already represented in the reviewed artifacts.

## Evidence Run During Integration

The following commands were run against the current worktree using a writable
target dir under `/tmp`:

    CARGO_TARGET_DIR=/tmp/myosu-agent-integration-target cargo test -p myosu-games
    CARGO_TARGET_DIR=/tmp/myosu-agent-integration-target cargo test -p myosu-tui schema::tests
    CARGO_TARGET_DIR=/tmp/myosu-agent-integration-target cargo test -p myosu-tui pipe::tests

Observed result:

- `myosu-games`: 10 unit tests passed, 4 doctests passed
- `myosu-tui schema::tests`: 12 tests passed
- `myosu-tui pipe::tests`: 5 tests passed

## What This Adapter Changes

This adapter does not change the `agent:experience` lane contract. It changes
how the product frontier should act on that contract:

- before this note: product still had an open choice between another upstream
  unblock and an implementation-family move
- after this note: the honest next step is a product implementation family,
  beginning with `play:tui`

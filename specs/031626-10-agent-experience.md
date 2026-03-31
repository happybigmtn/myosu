# Specification: Agent Experience — Shared Text Interface and Machine Contract

Source: `OS.md` presentation-layer doctrine, `crates/myosu-tui/src/schema.rs`,
`crates/myosu-tui/src/pipe.rs`, `crates/myosu-play/src/main.rs`,
`crates/myosu-play/src/live.rs`
Status: Draft
Date: 2026-03-29
Depends-on: TU-01..12 (TUI implementation), GP-01..04 (gameplay),
CC-01 (shared chain client), LI-01..06 (launch integration)

## Purpose

Define the canonical agent-facing contract for Stage 0 and the immediate
follow-on stages. `OS.md` states the doctrine: agents and humans are supposed
to inhabit one text interface. This spec turns that doctrine into an executable
contract grounded in the code that now exists, instead of the older
transport-table-only story.

At the current Stage 0 truth boundary, the agent experience is not a separate
application stack. It is the same gameplay surface exposed in two forms:

- the shared plain-text pipe protocol used by `myosu-play pipe`
- the machine-readable game-state schema carried by `myosu-tui::schema`

The same game state, legal actions, and recommendation metadata that humans see
in the TUI must remain available to non-human actors without requiring them to
re-derive legality or hidden game context.

## Whole-System Goal

Current state:
- `myosu-play train` runs the interactive text UI
- `myosu-play pipe` emits plain-text agent frames and accepts stdin actions
- `myosu-tui::schema` defines a game-agnostic JSON contract with exhaustive
  `legal_actions`
- `myosu-play` can optionally discover a miner from chain state and query a
  live strategy endpoint over HTTP
- the repo does **not** yet ship a standalone WebSocket gameplay server,
  packaged Python SDK, or separate agent daemon

This spec adds:
- the truthful Stage 0 contract for agent interaction
- transport invariants that all future surfaces must preserve
- a separation between what is implemented now and what remains future scope
- acceptance criteria for pipe-mode parity, schema parity, and live-query
  provenance metadata

If all ACs land:
- an agent can play through the same state machine as a human using `pipe`
- no agent needs to guess legal actions or hidden context
- live miner advice, when present, is surfaced as provenance metadata instead
  of hidden implementation detail
- future HTTP/WebSocket/SDK surfaces have a concrete compatibility target

Still not solved here:
- persistent multi-session gameplay server
- authenticated remote agent sessions
- packaged Python/Rust SDK artifacts for external distribution
- spectator-agent orchestration and journaling surfaces described in `OS.md`

## Why This Spec Exists As One Unit

- The agent contract is easy to overclaim. `OS.md` lists multiple transports,
  but the code today implements only part of that story. This spec exists to
  lock the current truth before later transports drift.
- Pipe output, legal-action exhaustiveness, and live-query provenance all touch
  the same user-visible contract. Splitting them would make it easier for the
  docs to promise parity that the code does not actually preserve.
- The gameplay binary already carries both human and agent paths. The real
  design question is not "which app talks to agents?" but "what invariants must
  survive across the interfaces we already have and the ones we add later?"

## Scope

In scope:
- the Stage 0 pipe transport exposed by `myosu-play pipe`
- the game-state JSON schema in `myosu-tui::schema`
- the legal-action completeness contract
- plain-text frame formatting and metadata lines
- live miner query provenance appended to pipe output
- the relationship between interactive TUI mode and agent mode
- compatibility rules for future HTTP/WebSocket/SDK surfaces

Out of scope:
- implementing a standalone HTTP or WebSocket gameplay server in this slice
- packaging external SDK crates or wheels
- agent memory, journals, or reflection state beyond the current render frame
- remote auth, rate limiting, or tenancy policy

## Current State

The agent experience already exists in a narrow but honest form:

- `crates/myosu-play/src/main.rs` exposes `train` and `pipe` subcommands
- `crates/myosu-tui/src/pipe.rs` handles frame emission and stdin reads
- `crates/myosu-tui/src/schema.rs` defines the machine-readable state contract
- `crates/myosu-games-poker/src/renderer.rs` renders recommendation-bearing
  pipe frames for NLHE
- `crates/myosu-play/src/live.rs` performs best-effort live miner health and
  strategy queries over HTTP and folds the result into agent-visible metadata

The important honesty constraint is that the repo does **not** yet expose the
full OS transport table as first-class runtime surfaces. Pipe mode is real now.
The schema is real now. Live HTTP strategy querying is real now. The rest is
design intent until implemented.

## What Already Exists

| Sub-problem | Existing code / flow | Reuse / extend / replace | Why |
|-------------|----------------------|--------------------------|-----|
| Plain-text agent IO | `myosu-tui::PipeMode` | reuse | Already emits deterministic metadata + state lines |
| Exhaustive action contract | `myosu-tui::schema::GameState` | reuse | Central machine contract for all games |
| NLHE agent frames | `myosu-games-poker::NlheRenderer::pipe_output*` | reuse | Current Stage 0 reference implementation |
| Local agent command parsing | `myosu-play::pipe_response` | extend | Human/agent action path already converges here |
| Live miner advice | `myosu-play::live` | reuse with caution | Useful provenance surface, but best-effort only |
| Chain-visible miner discovery | `myosu-play` + shared chain client | extend | Optional bridge from gameplay to live solver queries |

## Ownership Map

| Component | Status | Location |
|-----------|--------|----------|
| Agent JSON schema | Implemented | `crates/myosu-tui/src/schema.rs` |
| Pipe transport | Implemented | `crates/myosu-tui/src/pipe.rs` |
| Pipe command handling | Implemented | `crates/myosu-play/src/main.rs` |
| Live miner strategy query | Implemented | `crates/myosu-play/src/live.rs` |
| NLHE agent rendering | Implemented | `crates/myosu-games-poker/src/renderer.rs` |
| Future HTTP/WebSocket gameplay transport | Planned | no canonical runtime surface yet |
| External SDK packaging | Planned | no canonical package yet |

## Architecture / Runtime Contract

```text
human keyboard                    agent stdin/stdout
     |                                  |
     v                                  v
  myosu-play train                  myosu-play pipe
         \                            /
          \                          /
           v                        v
                 GameRenderer
                      |
                      v
               myosu-games-poker
                      |
                      v
              game state + advice
                      |
        +-------------+--------------+
        |                            |
        v                            v
  TUI shell render              pipe/state schema
        |                            |
        v                            v
   human-visible frame         agent-visible contract
```

Optional live-query enrichment:

```text
chain ws discovery --> discovered miner --> HTTP /health + /strategy
                                             |
                                             v
                               recommendation + provenance metadata
                                             |
                                             v
                                     appended to pipe state
```

## Invariants

### INV-AX-01: One decision surface

Humans and agents may use different transports, but they must not receive
different game truth. If a human-visible decision can be made, the same
decision must be reachable from the agent surface without reverse-engineering
layout-only signals.

### INV-AX-02: Exhaustive legal actions

The agent contract must enumerate all legal actions. The agent never computes
legality from board state or stack math. If the schema or pipe surface omits a
legal action, the interface is wrong.

### INV-AX-03: Plain-text pipe

Pipe mode is a transport contract, not a prettified TUI capture. It must not
emit ANSI escapes, cursor control, box drawing, or alternate-screen behavior.

### INV-AX-04: Provenance over magic

When advice is coming from an artifact, fallback generator, or live miner
query, the agent-visible surface must say so. Recommendations are useful only
if their origin is inspectable.

### INV-AX-05: Future transports preserve schema semantics

Any future HTTP, WebSocket, or SDK surface must preserve the semantics already
present in `myosu-tui::schema` and the pipe output. New transports may add
session management and streaming, but they may not weaken legal-action
exhaustiveness or hide source metadata.

## Acceptance Criteria

### AC-AX-01: Pipe mode is a first-class Stage 0 transport

- Where:
  - `crates/myosu-play/src/main.rs`
  - `crates/myosu-tui/src/pipe.rs`
- Requirement:
  - `myosu-play pipe` must emit initial state, accept one action per input
    line, emit action acknowledgment, and print the next state after each
    state-advancing move.
  - Empty lines may be ignored. `/quit` must always be available.
- Proof:
  - `cargo test -p myosu-play pipe_response --quiet`
  - `cargo test -p myosu-tui pipe --quiet`

### AC-AX-02: Pipe frames stay plain-text and self-describing

- Where:
  - `crates/myosu-tui/src/pipe.rs`
  - `crates/myosu-games-poker/src/renderer.rs`
- Requirement:
  - Each emitted frame carries shell metadata plus a game-state line.
  - Pipe output must stay free of ANSI escapes.
  - Idle and active-hand states must both be representable.
- Proof:
  - `cargo test -p myosu-tui pipe_output_no_ansi --quiet`
  - `cargo test -p myosu-games-poker pipe_output_stays_plain_text --quiet`

### AC-AX-03: The machine contract enumerates legal actions exhaustively

- Where:
  - `crates/myosu-tui/src/schema.rs`
  - `crates/myosu-play/src/main.rs`
- Requirement:
  - `GameState.legal_actions` remains exhaustive.
  - Invalid actions must return the current legal-action set rather than
    forcing the caller to reconstruct it.
  - Pipe-mode invalid input must expose the same action set that a schema
    client would consume.
- Proof:
  - `cargo test -p myosu-tui legal_actions_exhaustive --quiet`
  - `cargo test -p myosu-play pipe_response_rejects_invalid_input --quiet`

### AC-AX-04: Live-query provenance is agent-visible, not hidden

- Where:
  - `crates/myosu-play/src/live.rs`
  - `crates/myosu-play/src/main.rs`
- Requirement:
  - When a live miner query succeeds, the agent-visible state must expose that
    a live query occurred and identify the advice source.
  - Failure to query a live miner must degrade to metadata, not undefined
    behavior or silent state mutation.
- Proof:
  - `cargo test -p myosu-play pipe_output_carries_live_query_success_metadata --quiet`
  - `cargo test -p myosu-play pipe_output_carries_live_query_failure_metadata --quiet`

### AC-AX-05: The smoke path proves agent-play completeness

- Where:
  - `crates/myosu-play/src/main.rs`
- Requirement:
  - The smoke path must prove that a deterministic sequence of valid inputs can
    carry the state from preflop to terminal completion through the same
    action-handling surface the pipe transport uses.
- Proof:
  - `cargo run -p myosu-play --quiet -- --smoke-test`

### AC-AX-06: Future transports are compatibility targets, not promises

- Where:
  - this spec, `OS.md`, future transport implementations
- Requirement:
  - HTTP REST, WebSocket, Python SDK, and Rust in-process traits remain valid
    design targets, but they must not be documented elsewhere as implemented
    Stage 0 runtime surfaces until code and proof commands exist.
  - The compatibility bar for those future transports is the current pipe +
    schema contract.

## Transport Table

The truthful transport matrix at this stage is:

| transport | state in repo today | note |
|-----------|---------------------|------|
| stdin/stdout pipe | implemented | primary Stage 0 agent interface |
| JSON game schema | implemented | canonical machine contract |
| live HTTP miner query | implemented as downstream enrichment | gameplay client to miner, not a standalone gameplay API |
| chain WebSocket discovery | implemented as optional dependency of gameplay discovery | not a dedicated agent-session transport |
| gameplay HTTP REST server | planned | OS intent only today |
| gameplay WebSocket server | planned | OS intent only today |
| packaged Python SDK | planned | OS intent only today |
| Rust in-process strategy trait | partial | trait-level/game crate seams exist, but not a full packaged agent SDK |

## Data Contract

The canonical machine-readable contract is `myosu_tui::schema::GameState`.
Important fields:

- `game_type`
- `hand_number`
- `phase`
- `state`
- `legal_actions`
- `meta`

The crucial design rule is that `state` may vary by game, but the surrounding
shape cannot become game-specific in a way that forces transport consumers to
special-case the transport itself. New games extend the payload, not the
existence of legality or provenance metadata.

## Failure Modes

| Codepath | Realistic failure | Handling |
|---|---|---|
| Pipe output | ANSI or layout characters leak into frames | plain-text tests fail |
| Action handling | invalid input hides the real legal action set | rejection path must enumerate legal actions |
| Live query | miner endpoint hangs or returns malformed HTTP | timeouts and parse errors degrade to explicit metadata |
| Future docs | OS transport table gets mistaken for implemented runtime truth | this canonical spec and acceptance criteria keep the boundary honest |

## Plan of Work

1. Treat pipe mode plus schema as the canonical Stage 0 agent surface.
2. Keep legal-action completeness and provenance metadata executable.
3. Add future transports only against this compatibility target.
4. Refuse doc claims that present planned transports as already shipped.

## Validation and Acceptance

Accepted when:
- the canonical spec points at real files instead of aspirational interfaces
- pipe mode, schema legality, and live-query provenance are all proven
- future transports are clearly described as compatibility targets, not current
  runtime surfaces

## Idempotence and Recovery

This spec is documentation-only. If future code extends the transport surface,
update the transport table and ACs rather than loosening the Stage 0 claims.

# `agent-integration` Adapter

## Purpose

This artifact translates the already-reviewed `agent:experience` lane into a
product-frontier execution decision. It answers one practical question:

Should Myosu product work branch into an implementation family now, or does the
frontier still need another upstream unblock first?

The answer from the current tree is: **product needs an implementation family
next**. The remaining hard gaps are product-owned missing crates and CLI wiring,
not upstream bootstrap uncertainty.

## Inputs Inspected

- `README.md`
- `SPEC.md`
- `PLANS.md`
- `AGENTS.md`
- `specs/031626-00-master-index.md`
- `specs/031826-fabro-primary-executor-decision.md`
- `specsarchive/031626-10-agent-experience.md`
- `specsarchive/031626-17-spectator-protocol.md`
- `fabro/programs/myosu-product.yaml`
- `fabro/programs/myosu-games-traits-implementation.yaml`
- `fabro/run-configs/product/agent-experience.toml`
- `fabro/workflows/bootstrap/agent-experience.fabro`
- `outputs/agent/experience/spec.md`
- `outputs/agent/experience/review.md`
- `outputs/play/tui/spec.md`
- `outputs/play/tui/review.md`
- `outputs/games/traits/review.md`
- `outputs/games/traits/implementation.md`
- `outputs/games/traits/verification.md`
- `docs/api/game-state.json`
- `crates/myosu-games/Cargo.toml`
- `crates/myosu-tui/src/schema.rs`
- `crates/myosu-tui/src/pipe.rs`
- `crates/myosu-tui/src/renderer.rs`

## Verified Repository State

### Product Control Plane

- `fabro/programs/myosu-product.yaml` already owns the two reviewed product
  lanes: `play:tui` and `agent:experience`.
- The product program currently stops at `reviewed` milestones. There is no
  product-scoped implementation family yet.
- The only implementation-family program in the repo today is
  `fabro/programs/myosu-games-traits-implementation.yaml`, which provides the
  pattern to copy.

### Code Truth Relevant to Agent Experience

- `docs/api/game-state.json` exists and remains the shared machine-readable
  contract for agents and spectators.
- `crates/myosu-tui/src/schema.rs` exists and is implemented.
- `crates/myosu-tui/src/pipe.rs` exists and provides the current `PipeMode`
  skeleton.
- `crates/myosu-tui/src/renderer.rs` exists and defines the `GameRenderer`
  contract.
- `crates/myosu-play/` does not exist.
- `crates/myosu-games-poker/` does not exist.
- `crates/myosu-tui/src/agent_context.rs`, `narration.rs`, and `journal.rs`
  do not exist.

### Upstream Blocker Status

- `crates/myosu-games/Cargo.toml` no longer uses absolute robopoker paths.
  It already points to pinned git revisions for `rbp-core` and `rbp-mccfr`.
- This means the old “robopoker path dependency” blocker in product review
  artifacts is now a **stale review claim**, not a current upstream blocker.

## Proof Commands Run for This Adapter

These were run in this worktree with `CARGO_TARGET_DIR=/tmp/myosu-cargo-target`
to avoid the read-only shared target path:

- `cargo test -p myosu-games`
  Result: passed; 10 unit tests and 4 doctests.
- `cargo test -p myosu-tui schema::tests`
  Result: passed; 12 schema tests.
- `cargo test -p myosu-tui pipe::tests`
  Result: passed; 5 pipe tests.

These proofs are enough for the adapter decision because they validate the two
trusted upstream surfaces the product lanes actually build on:

- `games:traits` compiles and tests cleanly with the git-pinned robopoker fork.
- `myosu-tui` still has a working schema surface and pipe skeleton.

## Adapter Decision

### Decision

**Proceed to a product implementation family next. Do not wait for another
upstream unblock.**

### Why

The unresolved gaps are now product-owned:

1. `play:tui` still lacks the `myosu-play` crate and the
   `myosu-games-poker` crate.
2. `agent:experience` still lacks its persistence, narration, reflection, and
   spectator modules.
3. The remaining cross-lane dependency inside product is `myosu-play` Slice 1,
   not another bootstrap or platform restart.

The previous upstream concern about robopoker portability was valid when the
bootstrap reviews were written, but the `games:traits` implementation and
verification artifacts show that concern has already been reduced.

## Product Implementation Sequencing

### Critical Path

1. Start `play:tui` implementation with Slice 1:
   create `crates/myosu-play/`, wire the shell, and prove
   `cargo build -p myosu-play`.
2. Once `myosu-play` exists, `agent:experience` Slice 3 and later become
   straightforward because the CLI flags (`--pipe`, `--context`, `--narrate`,
   `--spectate`) finally have a home.

### Parallel Work That Is Safe Before `myosu-play`

`agent:experience` Slices 1 and 2 can run before the play binary exists:

- `crates/myosu-tui/src/agent_context.rs`
- `crates/myosu-tui/src/journal.rs`

Those slices only extend `myosu-tui` and do not require a live play binary.

### Later-Phase Work

- `agent:experience` Slice 7 lobby discovery can begin with stubbed local data
  and should not wait for chain integration.
- Spectator relay and spectator screen should wait until the play binary exists.
- Live chain-backed lobby discovery and WebSocket spectator transport remain
  future soft dependencies on `chain:runtime`.

## Recommended Product Implementation Family

Mirror the `games:traits` implementation-family pattern rather than inventing a
new control-plane shape.

### Recommended New Control-Plane Files

- `fabro/programs/myosu-product-implementation.yaml`
- `fabro/run-configs/implement/play-tui.toml`
- `fabro/run-configs/implement/agent-experience.toml`
- `fabro/workflows/implement/play-tui.fabro`
- `fabro/workflows/implement/agent-experience.fabro`

### Recommended Output Artifacts

- `outputs/play/tui/implementation.md`
- `outputs/play/tui/verification.md`
- `outputs/agent/experience/implementation.md`
- `outputs/agent/experience/verification.md`

### Recommended Program Shape

- `play:tui` implementation lane:
  own the first critical-path slice and produce the first product
  implementation proof.
- `agent:experience` implementation lane:
  use the reviewed lane artifacts as slice authority and avoid a hard manifest
  dependency on `play:tui` verification, because Slices 1 and 2 can land before
  `myosu-play` exists.

The slice-level dependency should stay in the curated artifacts:

- Slices 1-2 can proceed immediately.
- Slice 3 and later wait on `myosu-play` Slice 1.

## Honest Gaps To Preserve

- `outputs/agent/experience/spec.md` and `review.md` are directionally correct
  but contain stale quantitative claims:
  old test counts and the now-resolved robopoker path blocker.
- `outputs/play/tui/review.md` is directionally correct but still describes the
  robopoker path issue as a blocker rather than a risk already reduced by
  `games:traits`.
- Those artifacts do not need a new bootstrap run before implementation begins,
  but they should be normalized during the first product implementation-family
  pass so Raspberry is not left with stale blocker language.

## Bottom Line

`agent:experience` does not need another upstream unblock to justify product
work. The correct next move is to create a product implementation family, start
with `play:tui` Slice 1 on the critical path, and treat early
`agent:experience` persistence slices as optional parallel work.

# `agent-integration` Review

## Judgment

**KEEP — product should open an implementation-family workflow next.**

The reviewed `agent:experience` lane did what it needed to do: it clarified
the contract, identified the missing agent-facing surfaces, and exposed the
real dependency on `play:tui`. After checking the current repository state,
the remaining work is product implementation work, not another upstream
unblock.

## Findings

### 1. The `agent:experience` blocker list is directionally right but partly stale

`outputs/agent/experience/review.md` still names robopoker absolute-path
coupling as a high blocker. That blocker is already reduced in the repo:
`crates/myosu-games/Cargo.toml` now uses pinned git dependencies, and
`outputs/games/traits/implementation.md` plus
`outputs/games/traits/verification.md` record that migration as complete.

**Impact:** product should not wait on another `games:traits` unblock pass
before starting its first implementation slice.

### 2. The product control plane ends at review stage

`fabro/programs/myosu-product.yaml` manages only `spec.md` and `review.md`
artifacts for `play:tui` and `agent:experience`, both at the `reviewed`
milestone. There is no product implementation or verification contract yet.

**Impact:** the next honest gap is a missing implementation family, not missing
product doctrine.

### 3. The first real product blocker is internal to product

The repository still has no `crates/myosu-play/` and no
`crates/myosu-games-poker/`. `Cargo.toml` still comments out the
`crates/myosu-play` workspace member. Inside `myosu-tui`, `lib.rs` has no
agent-specific modules, and `pipe.rs` is still the minimal pipe driver with no
context, narration, or reflection support.

**Impact:** `play:tui` Slice 1 is the first true product gate, and
`agent:experience` Slice 3+ should stay behind it.

## Verification Performed

The current upstream crate state was checked directly in this workspace.

- `CARGO_TARGET_DIR=/tmp/myosu-agent-integration-target cargo test -p myosu-games --quiet`
  passed with 10 unit tests and 4 doctests.
- `CARGO_TARGET_DIR=/tmp/myosu-agent-integration-target cargo test -p myosu-tui --quiet`
  passed with 82 tests, 2 ignored tests, and 1 doctest.

These checks are enough to show that the product frontier is no longer blocked
by the old robopoker portability issue and that the trusted shell surface is
still healthy.

## Recommendation

**Recommendation: open the product implementation family now.**

The first execution order should be:

1. `play:tui` implementation Slice 1: create `myosu-play`, add the workspace
   member, and prove `cargo build -p myosu-play`.
2. `agent:experience` implementation Slice 1-2: add agent context and journal
   persistence inside `myosu-tui`.
3. `agent:experience` implementation Slice 3+: wire `--context`,
   `reflect>`, and `--narrate` after `myosu-play` exists.

Chain-backed lobby data and spectator-over-axon remain later slices. They are
not reasons to delay the first local product implementation loop.

## Risks to Preserve

- Do not reintroduce local robopoker path dependencies when creating
  `myosu-play` or `myosu-games-poker`.
- Treat `GameRenderer` and the existing `PipeMode` contract as the frozen
  Phase 0 interface unless implementation proves an actual gap.
- Keep the first agent slices small and local to `myosu-tui`; do not let the
  implementation family balloon into chain discovery or miner transport before
  the local play loop exists.

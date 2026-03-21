# `agent-integration` Review

**Lane**: `agent-integration`  
**Frontier**: `product`  
**Date**: 2026-03-20  
**Judgment**: **KEEP product moving; promote product into an implementation family next**

---

## 1. Decision

### Decision: implementation family next, not another upstream unblock

`agent:experience` was the last remaining ready reviewed lane in the product
frontier. With that review complete, the product frontier is no longer blocked
on missing contract artifacts. It is now blocked on missing implementation
machinery and missing product crates.

The right next move is to start a product implementation family beginning with
`play:tui`, then follow immediately with `agent:experience` once the product
binary exists.

---

## 2. Why this is the honest call

### 2.1 Product already has the reviewed artifacts it needed

`myosu-product.yaml` owns two seeded lanes:

- `play:tui`
- `agent:experience`

Both now have reviewed artifacts under `outputs/`:

- `outputs/play/tui/spec.md`
- `outputs/play/tui/review.md`
- `outputs/agent/experience/spec.md`
- `outputs/agent/experience/review.md`

That means the frontier-selection question has been answered. Product has a
reviewed contract for both of its current lanes.

### 2.2 The shared upstream substrate is currently healthy

Fresh evidence from 2026-03-20:

- `CARGO_TARGET_DIR=/tmp/myosu-agent-integration-games cargo test -p myosu-games`
  passed with 10 unit tests and 4 doctests.
- `CARGO_TARGET_DIR=/tmp/myosu-agent-integration-tui cargo test -p myosu-tui`
  passed with 82 tests and 2 ignored real-TTY tests.

This confirms the trusted product substrate is still intact:

- game traits are working
- the shell is working
- the renderer contract is working
- pipe mode exists
- the schema exists

### 2.3 The earlier highest-severity blocker is stale now

`outputs/agent/experience/review.md` called out `robopoker` absolute-path
coupling as a HIGH blocker. That is no longer current.

`crates/myosu-games/Cargo.toml` now points to pinned git dependencies for
`rbp-core` and `rbp-mccfr`, and the crate test suite passes against that
configuration. So product no longer needs to wait on that upstream fix before
starting its first implementation slice.

---

## 3. What is still blocked

These blockers are real, but they are product-local implementation blockers,
not reasons to send the frontier back for more upstream bootstrap work.

### 3.1 `myosu-play` does not exist

`crates/myosu-play/` is still absent. This blocks:

- the product CLI
- `--pipe` flag exposure at the binary level
- `--context` / `--narrate` wiring
- spectator relay ownership in a real executable

Owner: `play:tui`

### 3.2 `myosu-games-poker` does not exist

`crates/myosu-games-poker/` is still absent. This blocks the first concrete
NLHE renderer and therefore blocks proving that the trusted shell can host a
real game.

Owner: `play:tui`

### 3.3 Agent-specific modules are still missing

These files are still absent from `crates/myosu-tui/src/`:

- `agent_context.rs`
- `journal.rs`
- `narration.rs`

Owner: `agent:experience`

### 3.4 Product has no implementation-family assets yet

Unlike `games:traits`, product currently has no checked-in implement loop:

- no `fabro/workflows/implement/play-tui.fabro`
- no `fabro/run-configs/implement/play-tui.toml`
- no lane-scoped product implementation manifest

This is the main control-plane gap.

---

## 4. Recommended sequencing

### First slice: `play:tui` Slice 1

This should be the first product implementation slice because it unlocks the
binary and renderer surfaces that `agent:experience` needs.

The reviewed slice already exists in `outputs/play/tui/spec.md`:

- create `crates/myosu-play/`
- wire a minimal `--train` entry into `myosu-tui`
- prove the render loop builds and runs

### Second slice: early `agent:experience`

Once `myosu-play` exists, begin the smallest agent slices:

1. `agent_context.rs`
2. `journal.rs`
3. `--context` wiring
4. `reflect>` prompt

This keeps agent integration attached to a real executable product surface.

### Later slices stay gated correctly

These should remain later product work, not reasons to delay the
implementation-family handoff:

- narration mode
- richer lobby data
- spectator relay
- chain-connected play APIs

---

## 5. Residual risks

### Risk 1: stale reviewed blockers cause the wrong frontier choice

If operators keep treating the resolved `robopoker` path issue as active, the
product frontier will waste time asking for another upstream unblock instead of
starting delivery.

### Risk 2: `agent:experience` is promoted before `play:tui`

That would create pressure to design richer agent surfaces before the product
has a host binary. It would likely produce partial work in `myosu-tui` without
an end-to-end product proof.

### Risk 3: product copies the wrong implementation pattern

The safe pattern already exists in `games:traits`: reviewed contract first,
then lane-scoped implement/fix/verify loop with `implementation.md` and
`verification.md`. Product should copy that pattern instead of inventing a new
one-off workflow.

---

## 6. Recommendation

**Promote product into an implementation family next.**

Concretely:

1. create the first product implementation-family assets for `play:tui`
2. execute `play:tui` Slice 1
3. follow with `agent:experience` implementation once the binary skeleton is
   live

**Do not spend the next slice on another upstream unblock.** The current
frontier evidence says product is ready to build.

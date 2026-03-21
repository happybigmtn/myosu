# Agent Integration Review

Date: 2026-03-20

## Scope

This review checks whether the completed `agent:experience` review changes the
next move for the product frontier. The decision target is narrow:
implementation family next, or another upstream unblock first.

## Judgment

**KEEP - Product should branch into an implementation-family next.**

`agent:experience` is the last ready product lane and its reviewed artifacts,
combined with the current tree, do not justify another upstream-unblock loop.
They justify opening the first product implementation family.

## Findings

### 1. The product frontier has only two lanes, and both are already reviewed

`fabro/programs/myosu-product.yaml` contains only:

- unit `play`, lane `tui`
- unit `agent`, lane `experience`

Both units already have their reviewed artifacts checked in under
`outputs/play/tui/` and `outputs/agent/experience/`.

This means `agent:experience` really is the last ready product review lane in
the checked-in product frontier.

### 2. The checked-in product control plane is still review-shaped

Both product run configs still point at bootstrap workflows:

- `fabro/run-configs/product/play-tui.toml` ->
  `fabro/workflows/bootstrap/play-tui.fabro`
- `fabro/run-configs/product/agent-experience.toml` ->
  `fabro/workflows/bootstrap/agent-experience.fabro`

Those workflows only require `spec.md` and `review.md`. They do not produce
`implementation.md` or `verification.md`.

There is no checked-in product implementation program and there are no
`implement/` workflows for either product lane. By contrast, `games:traits`
already has a complete implementation-family surface:

- `fabro/programs/myosu-games-traits-implementation.yaml`
- `fabro/run-configs/implement/game-traits.toml`
- `fabro/workflows/implement/game-traits.fabro`

The missing control-plane work is therefore implementation-family work, not
another bootstrap review pass.

### 3. Both reviewed product lanes already point to implementation next

The product review artifacts are directionally aligned:

- `outputs/play/tui/review.md` says the lane is ready for an
  implementation-family workflow immediately.
- `outputs/agent/experience/review.md` says to proceed to an
  implementation-family workflow next.

There is no checked-in product review artifact asking for a design reset,
boundary rewrite, or another upstream contract review before product can move.

### 4. The strongest cited upstream blocker is already resolved in the tree

The stale blocker named by both product reviews is robopoker absolute-path
coupling in `myosu-games`. That blocker is no longer live.

Current proof:

- `crates/myosu-games/Cargo.toml` uses pinned git rev dependencies for
  `rbp-core` and `rbp-mccfr`
- `outputs/games/traits/implementation.md` records that migration as the first
  implemented slice for `games:traits`
- `CARGO_TARGET_DIR=/tmp/myosu-cargo-target cargo test -p myosu-games`
  passed in this run with 10 unit tests and 4 doctests

This does not mean the older product reviews were wrong when written. It means
their strongest blocker language is now stale and should not control the next
frontier decision.

### 5. The current TUI upstream is stronger than the old bootstrap review record

`outputs/tui/shell/review.md` still carries reopen language around proof gaps.
The current tree is stronger than that snapshot:

- `CARGO_TARGET_DIR=/tmp/myosu-cargo-target cargo test -p myosu-tui`
  passed in this run with 84 unit tests and 1 doctest
- the run included the headless event-path test
  `events::tests::headless_stream_delivers_tick_key_resize_and_update`

This is a posture update, not a claim that the old review should be discarded.
It is evidence that product should not wait on a stale bootstrap snapshot when
the present tree is already stronger.

### 6. The real remaining blockers are product-owned implementation gaps

Current missing surfaces:

- `crates/myosu-play/`
- `crates/myosu-games-poker/`
- `crates/myosu-tui/src/agent_context.rs`
- `crates/myosu-tui/src/journal.rs`
- `crates/myosu-tui/src/narration.rs`

Those are not upstream-unblock problems. They are exactly the missing code and
control-plane surfaces a product implementation family exists to build.

### 7. AGENTS doctrine argues against reopening bootstrap

`AGENTS.md` says:

- the current bootstrap program intentionally stays narrow
- do not widen the bootstrap manifest
- new execution work should land as Fabro assets plus Raspberry program updates

Starting a separate product implementation family is consistent with that
doctrine. Reopening bootstrap for product would pull in the wrong direction.

### 8. Live Raspberry state is absent locally

The manifests point at repo-local `.raspberry/` state files, but no
`.raspberry/` directory exists in this worktree.

That means this review had to rely on:

- checked-in program manifests
- checked-in curated artifacts
- fresh proof commands against the current tree

This is an honesty note, not a blocker. It does mean the implementation family
should continue to prefer durable checked-in artifacts until the Fabro-to-
Raspberry run-truth bridge is present locally.

## Recommendation

Start a product implementation family next, following the existing
`games:traits` implementation-family pattern.

Recommended first round:

1. `play:tui` Slice 1: create `crates/myosu-play` and wire the shell-facing
   binary skeleton.
2. `agent:experience` Slice 1 and Slice 2: add `agent_context.rs` and
   `journal.rs` inside `myosu-tui`.
3. After `myosu-play` exists, continue `agent:experience` Slice 3 and Slice 4
   for `--context` wiring and the `reflect>` prompt.

Recommended control-plane additions:

- `fabro/programs/myosu-product-implementation.yaml`
- `fabro/workflows/implement/play-tui.fabro`
- `fabro/run-configs/implement/play-tui.toml`
- `fabro/workflows/implement/agent-experience.fabro`
- `fabro/run-configs/implement/agent-experience.toml`

Recommended durable artifact additions:

- `outputs/play/tui/implementation.md`
- `outputs/play/tui/verification.md`
- `outputs/agent/experience/implementation.md`
- `outputs/agent/experience/verification.md`

## Residual Risks

### Stale review language

Some product review artifacts still name blockers that the tree has already
cleared. The new implementation family should not blindly preserve those stale
gates.

### Internal sequencing still matters

`agent:experience` is not fully independent. Its CLI-facing slices still depend
on `play:tui` creating `myosu-play` first. That is an implementation sequencing
rule, not a reason to reopen upstream review.

### Toolchain portability remains unresolved

The workspace still uses Rust 2024 edition and there is no checked-in
`rust-toolchain.toml`. That remains a repo-wide portability risk, but it is not
the honest reason to keep product out of implementation mode.

### No live Raspberry state in the worktree

Until `.raspberry/` state exists locally, program posture will continue to be
inferred from checked-in manifests and durable outputs.

## Verdict

Product does not need another upstream unblock first. It needs implementation-
family control-plane surfaces and the first small round of real product code.

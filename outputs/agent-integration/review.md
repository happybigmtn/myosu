# Agent Integration Review

## Judgment

**KEEP — product needs an implementation-family next, not another upstream
unblock.**

The `agent:experience` reviewed artifacts are strong enough to make the next
frontier decision. The honest move is to start a product implementation family
now, with `play:tui` first and `agent:experience` immediately after the binary
scaffold exists.

## Findings

### 1. The prior external robopoker blocker is no longer active

Older reviewed artifacts in neighboring lanes still describe `myosu-games` as
blocked by absolute filesystem paths into a local `robopoker` checkout. That is
no longer true in the current tree.

Current evidence:

- `crates/myosu-games/Cargo.toml` now uses git-pinned `rbp-core`
- `crates/myosu-games/Cargo.toml` now uses git-pinned `rbp-mccfr`
- `CARGO_TARGET_DIR=/tmp/myosu-agent-integration-target cargo test -p myosu-games`
  passed with 10 unit tests and 4 doctests

Consequence:

- product does not need to pause for that upstream unblock before beginning its
  own implementation family

### 2. The agent-facing contract surfaces are real and locally verified

Current evidence:

- `docs/api/game-state.json` exists
- `crates/myosu-tui/src/schema.rs` exists and
  `cargo test -p myosu-tui schema::tests` passed with 12 tests
- `crates/myosu-tui/src/pipe.rs` exists and
  `cargo test -p myosu-tui pipe::tests` passed with 5 tests
- `crates/myosu-tui/src/screens.rs` already models `Spectate` as a screen state

Consequence:

- `agent:experience` is not waiting on more review-only specification work
- the product frontier has enough trusted contract surface to start coding

### 3. The remaining hard gaps are product-local

Current evidence:

- `crates/myosu-play/` is absent
- `crates/myosu-games-poker/` is absent
- `Cargo.toml` still comments out `crates/myosu-play`
- `crates/myosu-tui/src/agent_context.rs` is absent
- `crates/myosu-tui/src/narration.rs` is absent
- `crates/myosu-tui/src/journal.rs` is absent
- `crates/myosu-play/src/spectate.rs` cannot exist yet because `crates/myosu-play/` is absent

Consequence:

- the real dependency edge is inside product: `agent:experience` needs
  `play:tui`'s executable host before its later slices make sense

### 4. Product is reviewed, but not yet equipped with an implementation-family surface

Current evidence:

- `fabro/programs/myosu-product.yaml` supervises only the reviewed contract
  lanes `play:tui` and `agent:experience`
- search across `fabro/programs/`, `fabro/run-configs/`, and
  `fabro/workflows/` found no checked-in product implementation-family manifest
- the only existing implementation-family manifest today is
  `fabro/programs/myosu-games-traits-implementation.yaml`

Consequence:

- the next control-plane move is to add and run product implementation-family
  assets, not to author another upstream review pass

## Decision

Start a product implementation family next.

Recommended order:

1. `play:tui` implementation family
2. `agent:experience` implementation family

Why this order is correct:

- `play:tui` owns the missing binary and renderer
- `agent:experience` owns the pipe enrichments and persistence layers that sit
  on top of that binary
- current upstream truth is strong enough that another non-product unblock would
  be lower leverage than simply starting product implementation

## Residual Risks

### Review drift must not be mistaken for a live blocker

Some older lane reviews still mention the resolved robopoker path issue and
older test counts. That drift should be corrected when those lanes are next
touched, but it should not be treated as a reason to delay product execution.

### `agent:experience` should not skip over `play:tui`

Although some early `agent:experience` slices touch only `myosu-tui`, the lane
cannot honestly prove its CLI and spectator claims without the `myosu-play`
binary that `play:tui` owns.

### Spectator work remains second-wave product scope

`crates/myosu-tui/src/screens.rs` already reserves a `Spectate` screen state,
but there is still no relay, no screen implementation, and no binary wiring.
That work should stay behind the first executable gameplay slice.

## Final Frontier Call

Product does **not** need another upstream unblock before moving. Product
**does** need an implementation family next.

The first honest executable slice is:

- seed and run `play:tui` implementation assets that create `myosu-play`

The immediate follow-on slice is:

- seed and run `agent:experience` implementation assets that add context,
  reflection, and later narration/spectator behavior on top of that binary

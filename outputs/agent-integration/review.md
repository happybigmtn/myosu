# Agent Integration Review

## Judgment

**KEEP — product needs an implementation family next, not another upstream
unblock.**

The reviewed product frontier is now far enough along to leave bootstrap mode.
`play:tui` and `agent:experience` both have checked-in reviewed artifacts, and
the remaining gaps are missing product implementation surfaces rather than
missing upstream doctrine.

## Evidence

- `fabro/programs/myosu-product.yaml` contains only two product lanes:
  `play:tui` and `agent:experience`
- `outputs/play/tui/review.md` explicitly says the lane is ready for an
  implementation-family workflow
- `outputs/agent/experience/review.md` explicitly says the lane should proceed
  to implementation-family work
- `Cargo.toml` still comments out `crates/myosu-play`, and the crate does not
  exist
- `crates/myosu-tui/src/schema.rs` and `crates/myosu-tui/src/pipe.rs` exist,
  but `agent_context.rs`, `journal.rs`, and `narration.rs` do not
- `crates/myosu-games/Cargo.toml` now points to pinned git revs for
  `happybigmtn/robopoker`, matching
  `outputs/games/traits/implementation.md`

## Findings

### 1. The previously cited `robopoker` portability blocker is stale

The older `agent:experience` review still warns that `myosu-games` depends on
absolute `robopoker` paths. The live tree no longer matches that claim.
Portability is not the honest reason product is paused today.

### 2. The real blocker is missing product code, not missing upstream design

The product frontier still lacks the concrete crates and modules that reviewed
artifacts were preparing to unblock:

- `crates/myosu-play/`
- `crates/myosu-tui/src/agent_context.rs`
- `crates/myosu-tui/src/journal.rs`
- `crates/myosu-tui/src/narration.rs`

That is implementation work. It does not justify another upstream-unblock
detour.

### 3. Runtime truth in this checkout is artifact-based, not Raspberry-state-based

The checked-in manifests point at `.raspberry/...` state files, but no such
state files are present in this worktree. The reliable local truth for this
slice is therefore the curated `outputs/` surface plus live source inspection.

### 4. Local proof reruns were not possible in this sandbox

Direct `cargo test` reruns were blocked here by a read-only inherited cargo
target path and then by `/tmp` disk quota when forcing a local target dir. That
does not invalidate the reviewed artifacts, but it means this integration
review should not claim fresh proof execution.

## Recommendation

- Create or queue a product implementation-family program patterned after
  `fabro/programs/myosu-games-traits-implementation.yaml`
- Run `play:tui` implementation Slice 1 first so `crates/myosu-play/` exists
  and the product has a real CLI host
- Start `agent:experience` implementation as soon as that first `play:tui`
  milestone lands
- Treat chain-backed lobby data and WebSocket spectator transport as later
  slices, not reasons to delay the first product implementation family

## Keep / Reopen / Reset

- `play:tui`: **KEEP**
- `agent:experience`: **KEEP**
- Product frontier next action: **IMPLEMENT NEXT**
- Another upstream unblock: **NOT CURRENTLY JUSTIFIED**

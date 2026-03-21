# Agent Integration Review

## Judgment

**KEEP — product should move to an implementation-family workflow next.**

The reviewed `agent:experience` artifacts do not reveal a new upstream contract
gap. They reveal that the product frontier has reached the end of its reviewed
bootstrap surface and now needs product implementation sequencing.

## What Was Reviewed

The review reconciled:

- root doctrine: `README.md`, `SPEC.md`, `PLANS.md`, `AGENTS.md`
- canonical doctrine entrypoints:
  `specs/031626-00-master-index.md` and
  `specs/031826-fabro-primary-executor-decision.md`
- product program control plane: `fabro/programs/myosu-product.yaml`
- reviewed product lane artifacts:
  `outputs/play/tui/spec.md`,
  `outputs/play/tui/review.md`,
  `outputs/agent/experience/spec.md`, and
  `outputs/agent/experience/review.md`
- source archive specs that those artifacts derive from:
  `specsarchive/031626-10-agent-experience.md` and
  `specsarchive/031626-17-spectator-protocol.md`
- current workspace truth in `crates/myosu-games/`, `crates/myosu-tui/`,
  `docs/api/game-state.json`, and the absence of `crates/myosu-play/`

## Evidence That Product Is Past the Review Stage

1. `myosu-product.yaml` already owns both reviewed product lanes:
   `play:tui` and `agent:experience`.
2. `agent:experience` explicitly depends on the `play` unit's reviewed
   milestone, and that reviewed milestone already exists.
3. The trusted leaf crates still hold:
   `myosu-games` passed `cargo test -p myosu-games` on 2026-03-21 with a local
   writable `CARGO_TARGET_DIR`, and `myosu-tui` passed
   `cargo test -p myosu-tui` on 2026-03-21 with the same setup.
4. The earlier robopoker absolute-path warning in the product reviews is now
   stale. `myosu-games` depends on pinned robopoker git revisions, and the
   crate tests pass.
5. The largest remaining product gaps are missing crates and modules owned by
   product itself:
   `crates/myosu-play/`,
   `crates/myosu-games-poker/`,
   `crates/myosu-tui/src/agent_context.rs`,
   `crates/myosu-tui/src/journal.rs`,
   `crates/myosu-tui/src/narration.rs`,
   `crates/myosu-play/src/spectate.rs`, and
   `crates/myosu-tui/src/screens/spectate.rs`.

## Why This Is Not "Wait for Another Upstream Unblock"

An upstream unblock would be the right answer if the next product step were
still prevented by an unresolved trusted-lane contract. That is no longer the
case.

### Not a live upstream blocker anymore

- `games:traits` already cleared the robopoker path-coupling issue.
- `tui:shell` remains a trusted upstream with passing tests.

### Still blocked, but by product implementation itself

- `agent:experience` cannot expose `--context`, `--narrate`, lobby behavior,
  or spectator wiring until `play:tui` creates `myosu-play`.
- `play:tui` cannot prove local gameplay until it creates the missing binary
  and renderer crates.

Those are implementation-family facts, not reasons to reopen upstream review.

## Required Next Move

The next honest move is to seed a **product implementation family** rather than
another review-only lane.

Recommended first sequence:

1. `play:tui` Slice 1: create `crates/myosu-play/` and prove a minimal
   `myosu-play --train` host surface.
2. In parallel, `agent:experience` Slice 1 and Slice 2:
   `agent_context.rs` and `journal.rs`.
3. `play:tui` Slice 2: create `crates/myosu-games-poker/` with an
   `NlheRenderer`.
4. Then resume `agent:experience` Slice 3 and Slice 4:
   `--context` wiring and `reflect>` prompt.

This sequencing matches the reviewed lane contracts and uses the already trusted
upstream surfaces without waiting for chain/runtime work that is only needed
for later phases.

## Risks to Preserve in the Implementation Family

1. Treat the existing `GameRenderer` trait as the stable product seam. The
   first product commits should conform to it rather than reopening shell
   architecture.
2. Keep `agent:experience` scoped to agent-facing extensions on top of
   `myosu-play`; do not let it absorb the gameplay host responsibilities owned
   by `play:tui`.
3. Preserve the now-cleared robopoker git-pin state. New product crates should
   not reintroduce absolute path dependencies.
4. Keep spectator work late in the family. It is valuable, but it should not
   outrun the existence of a local playable session loop.

## Residual Gaps

Two things are still worth calling out even though they do not change the main
judgment:

- `outputs/play/tui/review.md` and `outputs/agent/experience/review.md` still
  mention the older robopoker absolute-path blocker. Future refreshes should
  remove or annotate that stale warning.
- There is not yet a checked-in product implementation-family manifest
  equivalent to `fabro/programs/myosu-games-traits-implementation.yaml`. That
  is the next control-plane asset product needs, but it is downstream of this
  review decision, not a reason to defer it.

## Final Decision

**Product needs an implementation family next, not another upstream unblock.**

The frontier is review-complete enough to start real product code delivery. The
first implementation family should be anchored on `play:tui` Slice 1 and
`agent:experience` Slices 1-2, with later agent wiring following once
`myosu-play` exists.

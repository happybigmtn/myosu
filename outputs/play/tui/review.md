# `play:tui` Lane Review

**Lane**: `play:tui`
**Date**: 2026-03-29

## Judgment Summary

**Judgment: KEEP — Implemented and stage-0-real**

This review is no longer a pre-implementation baseline. The lane now exists as
real code across `myosu-play`, `myosu-tui`, and `myosu-games-poker`, and the
most important stage-0 claims are executable:

- `myosu-play train` provides the interactive TUI path
- `myosu-play pipe` provides the plain-text agent path
- the smoke path proves one complete hand from preflop to terminal completion
- live miner discovery/query metadata can be surfaced without replacing the
  local artifact-backed or generated fallback path

The honest remaining gap is not "build the lane." It is productization and
future transport expansion beyond the current TUI + pipe contract.

## Verified Today

Fresh proof on 2026-03-29:

```bash
SKIP_WASM_BUILD=1 cargo run -p myosu-play --quiet -- --smoke-test
```

This passes and proves:

- the binary boots on the current workspace line
- the smoke driver starts from `PREFLOP`
- the scripted action path reaches `STATE complete`
- the same action-handling surface used by pipe mode advances the hand

The plain non-`SKIP_WASM_BUILD` route is still environment-gated here because
`myosu-play` pulls the chain runtime into the build and this machine does not
have `wasm32-unknown-unknown` installed.

Supporting repo truth also exists in the code:

- `crates/myosu-play/src/main.rs` exposes `train`, `pipe`, and `--smoke-test`
- `crates/myosu-tui/src/pipe.rs` owns the plain-text transport contract
- `crates/myosu-games-poker/src/renderer.rs` owns the NLHE renderer and pipe
  state formatting
- `crates/myosu-play/src/live.rs` owns best-effort live miner HTTP query
  enrichment

## Surface Assessment

| Surface | Status | Rationale |
|---------|--------|-----------|
| `crates/myosu-play/` | **KEEP** | Real binary, real smoke proof, real train/pipe modes |
| `crates/myosu-tui/` | **KEEP** | Shared shell and pipe transport are active dependencies |
| `crates/myosu-games-poker/` renderer path | **KEEP** | NLHE render + pipe output are on the live path |
| Artifact-backed advice | **KEEP WITH CAUTION** | Real, but still one of several advice origins |
| Live miner query enrichment | **KEEP WITH CAUTION** | Real and useful, but best-effort and optional |
| Future server/SDK transports | **REOPEN LATER** | Still design targets, not shipped stage-0 surfaces |

## What Changed Since The Old Review

The stale review said the lane was "ready to build" and listed missing crates.
That is no longer true. The crates exist, the smoke path exists, and the lane
has already been integrated into the completed stage-0 stack documented in the
master plan.

The right question now is narrower:

- does the current TUI + pipe surface stay honest?
- does the smoke path stay green?
- do live miner query and artifact-backed advice remain provenance-visible?

## Residual Risks

- Pipe and schema parity still need to stay synchronized as transports evolve.
- Live miner query remains a best-effort enrichment path, so failures must keep
  degrading into explicit metadata rather than silent behavior changes.
- Productization work like broader transport exposure, onboarding polish, and
  packaging remains out of scope for this lane review.

## Recommendation

Treat `play:tui` as implemented and trustworthy on the current stage-0 line.
Do not reopen it as a missing-build lane. The right follow-on work is:

1. preserve the smoke proof
2. preserve pipe/schema legality and provenance guarantees
3. treat new transports as compatibility work, not as a reason to re-describe
   the existing lane as incomplete

# ADR 006: Commit-Reveal V2 As The Only Live Weight-Hiding Path

- Status: Accepted
- Date: 2026-04-02
- Deciders: Myosu maintainers
- Consulted: `ops/decision_log.md`, `AGENTS.md`, `THEORY.MD`, `specs/031626-03-game-solving-pallet.md`
- Informed: validator, pallet, and chain-client contributors
- Related: `crates/myosu-chain/pallets/game-solver/src/macros/dispatches.rs`, `crates/myosu-chain/pallets/game-solver/src/subnets/weights.rs`, `crates/myosu-chain/pallets/game-solver/src/tests/stage_0_flow.rs`

## Context

This is a retroactive record of the stage-0 weight-submission decision captured
in the 2026-03-17 decision log and later hardened by stage-0 proofs.

Myosu needs validators to hide weight vectors long enough to reduce easy
copying, but the inherited CRV3 timelock path depends on `pallet_drand` and
other surfaces that the stage-0 fork strips out. At the same time, the repo
needed a live proof that the remaining path was real, not just a compile-time
artifact left behind after the fork reduction.

The current pallet already embodies the reduced design: direct `set_weights`
is blocked when commit-reveal is enabled, while `commit_weights` and
`reveal_weights` remain as the surviving weight-hiding route.

## Decision

Myosu stage 0 supports only hash-based commit-reveal v2 for hidden weight
submission.

This covers:

- keeping `commit_weights` and `reveal_weights` as the live hiding mechanism
- rejecting direct `set_weights` when a subnet has commit-reveal enabled
- removing CRV3 timelock behavior from the stage-0 contract
- treating commit timing and reveal windows as part of the validator-facing
  chain contract

## Alternatives Considered

### Option A: Commit-reveal v2 only

This won because it preserves a real hiding mechanism that still works under
the stripped stage-0 fork.

### Option B: Keep CRV3 timelock encryption

This was rejected because the required drand/timelock surfaces are not present
in the stage-0 fork.

### Option C: Disable hidden submission entirely

This was rejected because it would leave validator weight vectors immediately
copyable and would undercut the intended consensus mechanics.

## Consequences

### Positive

- The live stage-0 chain has exactly one truthful weight-hiding path.
- Validator behavior is simpler to document and test than a mixed v2/v3 system.
- The fork no longer carries a dead timelock dependency chain just to preserve
  a feature it cannot actually run.

### Negative

- Validators must respect reveal windows and subnet rate limits.
- A pure hash-and-reveal scheme does not provide the stronger secrecy properties
  that a functioning timelock path might have offered.

### Follow-up

- Keep stage-0 proofs focused on the actual commit/reveal window behavior.
- Do not reintroduce alternate hidden-submission paths without a new ADR and a
  real runtime dependency story.

## Reversibility

Moderate.

Myosu could adopt a stronger hiding mechanism later, but doing so would require
new runtime dependencies, client behavior, and operator guidance. The decision
should be reopened only when a future-stage design can prove that v2 is no
longer sufficient for validator behavior or incentives.

## Validation / Evidence

- `crates/myosu-chain/pallets/game-solver/src/macros/dispatches.rs` exposes the
  live `commit_weights` and `reveal_weights` dispatchables and guards
  `set_weights` behind `CommitRevealEnabled`.
- `crates/myosu-chain/pallets/game-solver/src/subnets/weights.rs` carries the
  commit-count and reveal-window enforcement logic.
- `crates/myosu-chain/pallets/game-solver/src/tests/stage_0_flow.rs` contains
  the focused proof that commit-reveal v2 is the only surviving live
  weight-hiding path.

# `games:traits` Lane Review

**Lane**: `games:traits`
**Date**: 2026-03-29

## Judgment Summary

**Judgment: KEEP — Hardened boundary, not a bootstrap placeholder**

The lane is trustworthy as a stage-0 leaf crate and the surrounding boundary
work has moved beyond simple portability fixes. The current live truth is:

- the crate stays small and auditable
- the public growth seam is intentional
- the custom-game extension boundary has explicit proof
- no gameplay/miner cross-dependency is being hidden as "close enough"

## Verified Today

Fresh proof on 2026-03-29:

```bash
cargo test -p myosu-games --quiet
```

Results:

- `20` unit/integration tests passed
- `4` doctests passed

## Surface Assessment

| Surface | Status | Rationale |
|---------|--------|-----------|
| `src/lib.rs` re-export entry point | **KEEP** | Thin and stable |
| `src/traits.rs` | **KEEP** | Still the narrow shared game contract |
| variant-growth seam | **KEEP** | Intentionally locked for additive growth |
| custom-game extensibility | **KEEP** | Explicitly proven rather than assumed |
| gameplay/miner boundary doctrine | **KEEP WITH CAUTION** | Literal invariant is enforced, broader architectural pressure is still documented |

## What Changed Since The Old Review

The stale review treated this lane as a trusted but still modest bootstrap
crate and even pointed at the wrong follow-on plan. Since then:

- plan `006` was completed
- executable INV-004 enforcement landed outside the crate in
  `crates/myosu-play/tests/invariants.rs`
- the robopoker fork divergence now has a repo-local changelog
- enum growth seams were locked with `#[non_exhaustive]`
- custom-game extensibility was turned into a concrete regression instead of a
  narrative claim

## Residual Risks

- The crate still intentionally re-exports robopoker traits, so fork coherence
  remains part of the maintenance burden.
- The stricter long-term architectural target is still better than the literal
  invariant: gameplay should eventually share only the game crates and not feel
  pressure from chain-client transitive dependencies.

## Recommendation

Keep this lane trusted and stop framing it as awaiting foundational proof. The
right follow-on work is preserving the narrow API and the executable boundary
proofs, not reopening structural bootstrap questions that are already settled.

# Chain Pallet Review

**Lane**: `chain:pallet`
**Date**: 2026-03-29

## Judgment Summary

**Judgment: KEEP — Simplified, proven, and on the live stage-0 path**

The pallet lane is no longer in "restart" territory. The active
`pallet-game-solver` surface has been reduced, aligned with the stripped
runtime, and backed by focused regression proofs. The important stage-0 truth
now is:

- the pallet owns the live runtime-facing game-solver identity
- the inherited subtensor RPC/reporting baggage has been aggressively removed
- commit-reveal-v2 behavior is explicitly proven
- the remaining stage-0 DTO/reporting surface is tested through
  `stage_0_flow`

## Verified Today

Fresh proof on 2026-03-29:

```bash
cargo test -p pallet-game-solver stage_0_flow --quiet
```

Result:

- `16 passed, 0 failed`

This keeps the focused stage-0 pallet proof family green on the current
workspace line.

## Surface Assessment

| Surface | Status | Rationale |
|---------|--------|-----------|
| core pallet config and runtime-facing types | **KEEP** | They back the live stage-0 runtime surface |
| commit-reveal-v2 path | **KEEP** | Explicitly proven and now the only honest commit/reveal story |
| reduced RPC/reporting DTOs | **KEEP** | Smaller, more truthful outward contract |
| stage-0 swap seam | **KEEP** | Narrow local seam replaced the broader inherited swap contract |
| dead aggregate/custom RPC helpers | **RESET LANDED** | Removed from live runtime/RPC exposure |
| swap-era no-op liquidity plumbing | **RESET LANDED** | Removed from the pallet path |

## What Changed Since The Old Review

The stale review still described the pallet as a compile-restored restart lane
that might or might not become the sole runtime identity. That is outdated.
Since then:

- plan `005` was closed
- the pallet-side reporting surface was simplified in multiple verified slices
- the runtime/RPC surface was collapsed down to the surviving stage-0 contract
- the pallet now depends on a smaller local stage-0 swap seam instead of the
  full inherited swap interface

## Residual Risks

- The proof family is intentionally focused. It proves the surviving stage-0
  contract, not every historical subtensor-era path.
- Legacy gated tests and removed surfaces outside the active path can still
  carry stale assumptions.
- The upstream `trie-db v0.30.0` future-incompat warning remains a repo-wide
  notice, not a pallet-specific regression.

## Recommendation

Treat the pallet lane as complete on the active stage-0 surface. The right
follow-on work is not more inherited-surface cleanup inside this review; it is
preserving `stage_0_flow` as the honest pallet gate while downstream doctrine
and integration surfaces catch up.

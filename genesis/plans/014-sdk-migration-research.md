# 014: Upstream SDK Migration Research Gate

## Objective

Determine whether myosu should migrate from the opentensor polkadot-sdk fork
to upstream polkadot-sdk, and estimate the effort required.

## Context

The current chain depends on `opentensor/polkadot-sdk` at a specific git rev.
This fork may contain subtensor-specific patches. The risks of staying on the fork:
- Upstream security patches may not be backported
- Build toolchain may diverge from upstream Rust/wasm targets
- Contributor friction from non-standard SDK source

The risks of migrating:
- Subtensor-specific patches may be load-bearing
- `substrate_fixed` integration may require adjustment
- Storage layout compatibility unknown

## Acceptance Criteria

- A research document `specs/sdk-migration-research.md` that:
  - Identifies all divergences between the opentensor fork and upstream polkadot-sdk
    at the pinned rev
  - Categorizes each divergence as: myosu-relevant, subtensor-specific, or trivial
  - Estimates migration effort in terms of files to change
  - Identifies blockers (if any divergence is required for myosu)
  - Recommends: migrate now, migrate later, or stay on fork
- If the recommendation is "migrate now," a follow-up implementation plan
  is drafted (but not necessarily executed in this phase)

## Verification

The plan is closed when `specs/sdk-migration-research.md` exists with a clear
recommendation. No code changes required.

## Dependencies

- None. This research can proceed in parallel with all other plans.
- Should not be started before Plan 002 (dead code removal) to avoid
  auditing dependencies against the duplicate pallet.

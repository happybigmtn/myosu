# 014 — Token Economics Research Gate

## Objective

Complete the token economics decision that has been deferred since stage-0 began. This is a research gate, not an implementation plan. The deliverable is a reviewed decision document, not code.

## Context

The current stage-0 model uses `Stage0NoopSwap` — an identity stub that treats all token conversions as 1:1 with zero fees. This is correct for a single-token local devnet but is explicitly not production-ready.

ADR 008 (`docs/adr/008-future-token-economics-direction.md`, 14.5K) already exists with status `Proposed`. It documents the repo-local recommendation but says: `Deciders: pending maintainer review required by specs/050426-token-economics.md`.

IMPLEMENTATION_PLAN.md `F-003` tracks this blocker. The spec requires review by at least two contributors with token-economics context before the task closes.

**Design axes to decide (from ADR 008):**
1. Single vs dual token model
2. AMM type (if dual token)
3. Fee model
4. Registration cost mechanism
5. Emission schedule and halving
6. Staking mechanics
7. Cross-subnet token flow
8. Governance token utility

**This plan is feasibility-unresolved because it depends on external human review.** The repo cannot self-close this gate.

## Acceptance Criteria

- ADR 008 status changes from `Proposed` to `Accepted` (or `Rejected` with alternative)
- At least two named reviewers with token-economics context have signed off (recorded in the ADR or decision log)
- Each of the 8 design axes has a concrete recommendation with rationale
- Migration path from NoOpSwap to chosen model is sketched (not implemented)
- IMPLEMENTATION_PLAN.md `F-003` is resolved
- ops/decision_log.md records the decision

## Verification

This is a review-gated task. Verification is:
- ADR 008 has status != `Proposed`
- Decision log entry exists
- Named reviewers are recorded

No code changes required. No test commands.

## Dependencies

- None (this runs independently of all phases)
- **Blocker:** External human review. Cannot be self-closed by automated work.

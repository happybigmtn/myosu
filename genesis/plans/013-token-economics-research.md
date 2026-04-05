# 013: Token Economics Research Gate

## Objective

Determine whether the stage-0 single-token identity-swap model should evolve
into a dual-token AMM model (inherited from subtensor) or a different economic
design for post-stage-0 network operation.

## Context

The current `NoOpSwap` stub is an intentional simplification: all swaps are
1:1 identity, there are no fees, no liquidity pools, and no price discovery.
This is correct for a devnet where token value is not meaningful.

The inherited subtensor code contains a full constant-product AMM with:
- Alpha (subnet-specific) / TAO (network-wide) token pairs
- Pool liquidity management
- Fee collection
- Price slippage protection

The question is whether myosu should:
1. Activate the inherited AMM (reuse subtensor's dual-token model)
2. Design a simpler economic model (single token with emission decay)
3. Defer token economics entirely to a later phase

This is a **research gate**, not an implementation plan. The output is a
decision document, not code.

## Acceptance Criteria

- A research document `specs/token-economics-research.md` that:
  - Describes the current identity-swap model and its limitations
  - Analyzes the inherited AMM model's complexity vs. benefit
  - Proposes 2-3 alternative economic models
  - Recommends one model with rationale
  - Identifies the implementation cost of the recommendation
  - Lists open questions that require simulation or game-theoretic analysis
- The document is self-contained (no external repo references)
- The recommendation explicitly addresses: Is the NoOpSwap stub a temporary
  bridge or the final stage-0 economic model?

## Verification

The plan is closed when `specs/token-economics-research.md` exists and
contains a clear recommendation with rationale. No code changes required.

## Dependencies

- None. This research can proceed in parallel with all other plans.
- The research output may influence post-stage-0 planning but does not
  block any stage-0 work.

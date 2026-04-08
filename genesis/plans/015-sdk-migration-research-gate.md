# 015 — Polkadot SDK Migration Research Gate

## Objective

Determine whether and when to migrate from the opentensor polkadot-sdk fork (rev `71629fd`) to upstream polkadot-sdk. This is a research gate, not an implementation plan.

## Context

ADR 009 (`docs/adr/009-polkadot-sdk-migration-feasibility.md`, 8.8K) already exists. Key findings:

- The opentensor fork has 21 fork-only commits against upstream
- These patches touch consensus-critical paths: `sc-grandpa`, `sc-babe`, txpool, `frame-system`, `frame-support` dispatch, `sc-cli`, `sc-keystore`
- Migration effort is "moderate but consensus-path heavy"
- WORKLIST.md `CHAIN-SDK-001` recommends starting with patch classification, not a blind dependency swap

**This plan is feasibility-unresolved because:**
1. The 21 fork-only patches have not been individually classified as "needed by myosu" vs "subtensor-specific"
2. Upstream `stable2506` (or later) may have incorporated some of these patches
3. The effort to backport needed patches is unknown

## Acceptance Criteria

- Each of the 21 opentensor fork-only commits is classified as:
  - "Needed by myosu" (with rationale)
  - "Subtensor-specific" (safe to drop)
  - "Uncertain" (needs deeper investigation)
- A recommended migration timeline is proposed: "now", "after stage-0", "after stage-1", or "never (maintain fork)"
- If "now" or "after stage-0": a rough effort estimate and risk assessment
- If "never": rationale for maintaining the fork permanently
- ADR 009 is updated with the classification results
- WORKLIST.md `CHAIN-SDK-001` is resolved

## Verification

This is a research task. Verification is:
- ADR 009 updated with patch classification table
- Each of 21 commits has a classification
- Decision log entry exists

No code changes required. No test commands.

## Dependencies

- None (this runs independently of all phases)
- **Note:** If the classification reveals that all 21 patches are subtensor-specific, migration becomes a straightforward dependency swap. If consensus-critical patches are needed, migration becomes a multi-week effort requiring extensive testing.

# Genesis Report

Generated: 2026-04-11
Corpus refresh from: `trunk @ 4e0b37fbaa` plus current working tree

## Summary

This review was an independent Codex pass over the repo and the generated
corpus. It did not use sub-agents or the sibling Bitino repo. The corpus needed
real corrections, not just polish:

- the repo facts around validator scoring, NLHE training, and dependency-audit
  scope were materially sharper than the generated prose
- the plan set had a structural mismatch with the live repo checks
- the original Bitino plans were not grounded enough for execution ordering and
  had to be reconciled to the active root master plan

The revised corpus now acts as a subordinate decomposition of the active root
master plan in [plans/001-master-plan.md](/home/r/Coding/myosu/plans/001-master-plan.md).
It keeps the same product direction, preserves the Myosu-side grounding work,
and sequences the sibling `../bitino/` adapter only after the policy/promotion
gates are in place.

## Major Findings

### Finding 1: The validator path is still a determinism proof, not an independent quality oracle

`crates/myosu-validator/src/validation.rs` still scores the observed miner
response against the validator's own checkpoint-loaded expectation. That is
good enough for determinism and transport proofs and not good enough to justify
promotion claims by itself.

### Finding 2: NLHE promotion cannot honestly rely on the repo-owned sparse artifacts

`crates/myosu-miner/src/training.rs` rejects positive-iteration poker training
when `postflop_complete=false`, and the checked-in poker artifact summaries in
`crates/myosu-games-poker/src/artifacts.rs` still report sampled postflop
coverage only. That means a truthful `promotable_local` NLHE plan must depend
on a stronger pinned dossier path, not on the sparse bootstrap bundle.

### Finding 3: The corpus had two plan-shape failures

The generated numbered plans used the new ExecPlan sections, but:

- they did not satisfy the repo's live legacy plan-quality checker
- plans `010` and `011` were built around `../bitino/`, which this review was
  not allowed to inspect and which is outside the repo-relative plan boundary

Both issues are now corrected in the revised corpus.

### Finding 4: Security debt is broader than the generated prose implied

The active worklist tracks 12 advisories under `SEC-001`, but the live CI audit
gate currently suppresses 19 advisory IDs. The direct `bincode 1.3.3` usage in
the game codecs remains the most relevant owned-code item.

### Finding 5: The terminal product surfaces are credible and worth building on

`myosu-play`, `myosu-miner`, `myosu-validator`, and `myosu-keys` are already
structured enough to justify a promotion-driven product pass. This is not a
repo that needs a new UI shell first; it needs stronger solver provenance and a
stable downstream policy-bundle contract.

## Recommended Direction

Execute the promotion stream in this order:

1. policy bundle contract
2. promotion ledger and manifest gate
3. NLHE dossier/benchmark unblock
4. NLHE and Liar's Dice promotion
5. cribbage deepening
6. Bitino local adapter and same-TUI pilot

Security triage stays parallel. The practical change from the original corpus
is that sibling-repo Bitino work is no longer treated as the starting point. It
is a later, grounded milestone once the Myosu-side policy and promotion
surfaces are in place.

## Top Next Priorities

1. **Plan 001**: define the canonical policy-bundle contract in
   `myosu-games-canonical`.
2. **Plan 002**: build the promotion ledger and manifest gate.
3. **Plan 004**: unblock truthful NLHE provenance with a pinned dossier path.
4. **Plans 005-006**: promote NLHE and Liar's Dice only when the declared bar
   is actually met.
5. **Plan 008**: triage the audit suppressions, especially direct `bincode`
   usage.

## User Challenges

### Challenge 1: Treating sibling-repo Bitino implementation as an immediate first-class step without grounding

The root product direction still points toward a same-TUI Bitino consumer, but
the corrected corpus should not front-load `../bitino/` work ahead of policy
and promotion grounding. The honest near-term contract is: Myosu stabilizes
bundle/proof artifacts first, then the sibling Bitino adapter consumes them in
the later pilot milestone described by the active master plan.

## Not Doing List

| Item | Why it stays out of this corpus |
|------|----------------------------------|
| Production deployment work | The current repo truth is still local/operator proof, not public deployment. |
| Polkadot SDK migration execution | Important, but not blocking the next product-learning loop. |
| Funded settlement implementation | The repo still uses stage-0 economics and has no stable same-TUI consumer contract yet. |
| Ungrounded Bitino repo edits before policy/promotion gates | Still not an honest starting point. |
| Broad multi-game deepening | One portfolio proof is enough before widening again. |

## Decision Audit Trail

| Decision | Classification | Rationale |
|----------|----------------|-----------|
| Keep the promotion-driven direction | Mechanical | It matches the repo's actual leverage and the root master plan. |
| Gate same-TUI work behind policy/promotion grounding, then follow the active Bitino adapter milestone | User Challenge | The original cross-repo plans were front-loaded and under-grounded; the corrected sequence keeps them, but later and against inspected surfaces. |
| Require compatibility headings in the numbered plans | Mechanical | The live repo still enforces the older plan-quality headings. |
| Keep security triage parallel instead of first | Taste | Reasonable alternatives exist, but promotion evidence is the clearer immediate product loop. |
| Require stronger NLHE evidence than the sparse bootstrap artifacts | Mechanical | The sparse artifact path is explicitly blocked for positive-iteration training. |

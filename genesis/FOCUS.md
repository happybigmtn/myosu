# Focus Brief

## Raw Focus String

> focus on what we need to do to make the 001-master-plan come into reality

## Normalized Focus Themes

1. **Canonical policy bundle contract**: add a durable Myosu-side policy bundle
   and sampling-proof surface in `crates/myosu-games-canonical/`.
2. **Promotion ledger and benchmark gates**: create one machine-checked ledger
   that says which games are only routed, benchmarked, or promotable.
3. **Dedicated-game promotion**: move `nlhe-heads-up` and `liars-dice`
   forward only when their evidence is stronger than the current validator
   same-checkpoint self-check.
4. **Portfolio deepening**: prove the promotion pipeline can extend beyond the
   dedicated games by deepening one portfolio-routed game, with cribbage still
   the default candidate.
5. **Same-TUI adapter preparation**: make Myosu export a stable, self-contained
   bundle/proof contract and then feed it into the later sibling Bitino local
   adapter milestone.

## Repo Surfaces Implied by the Focus

| Surface | Repo location | What the focus requires |
|---------|---------------|------------------------|
| Canonical policy types | `crates/myosu-games-canonical/src/` | New `policy.rs` module with bundle verification and deterministic sampling |
| Promotion ledger | `ops/solver_promotion.yaml` | One entry per research game with evidence-backed tiering |
| Promotion manifest | `crates/myosu-games-canonical/examples/promotion_manifest.rs` | Table or machine-readable manifest that joins ledger claims with code support |
| Poker benchmark dossier | `crates/myosu-games-poker/src/artifacts.rs`, `benchmark.rs` | Hash-pinned artifact and benchmark summaries that can reference stronger external artifacts |
| Liar's Dice benchmark dossier | `crates/myosu-games-liars-dice/src/` | Exact-exploitability-backed checkpoint summary for policy-bundle provenance |
| Portfolio deepening | `crates/myosu-games-portfolio/src/core/cribbage.rs`, `examples/` | Labeled scenario pack and benchmark surface for one portfolio game |
| Promotion harness | `tests/e2e/promotion_manifest.sh` | CI-runnable gate that fails on aspirational ledger claims |
| Same-TUI adapter inputs | `outputs/solver-promotion/`, `crates/myosu-games-canonical/examples/` | Stable promoted bundle/proof samples consumed by the later Bitino local adapter |

## Out-of-Scope Dependency Guardrail

This review pass was limited to `/home/r/Coding/myosu`. That means:

- the sibling `../bitino/` repo was not inspected in this pass
- ungrounded Bitino implementation could not honestly be treated as the first
  executable step here
- the sequencing guardrail is "Myosu policy/promotion first, Bitino adapter
  second", not "never touch Bitino"

That boundary is preserved as an explicit sequencing constraint in the report
and the revised plan set.

## Repo-Wide Risks That Still Matter

1. **MINER-QUAL-001 remains a direct blocker**: the repo-owned poker bootstrap
   artifacts are still postflop-sampled, so truthful NLHE promotion requires a
   stronger pinned dossier path.
2. **SEC-001 still matters even if it is not first**: the current CI audit gate
   suppresses 19 advisories, including direct `bincode 1.3.3` usage in the game
   codecs.
3. **CHAIN-SDK-001 is still real but not on the critical path**: the
   `opentensor/polkadot-sdk` fork remains pinned and consensus-path heavy.
4. **Token economics are still stage-0 only**: `Stage0NoopSwap` is good enough
   for local proofs but not for any funded product claim.

## Main Questions the Focus Must Answer

1. What is the minimal policy-bundle contract Myosu can verify and sample
   deterministically today without pretending the validator already provides an
   independent quality oracle?
2. What evidence is required before `nlhe-heads-up` can honestly claim
   `promotable_local` rather than merely “bundle-capable”?
3. How does Liar's Dice package its existing exact-exploitability surface into
   the same promotion pipeline without inventing a parallel contract?
4. Which single portfolio-routed game can deepen the pipeline with the least
   ambiguity, and is cribbage still the right default after reviewing the
   actual code?
5. What repo-local export artifacts must exist before a downstream same-TUI
   consumer can begin work without crate-level coupling to Myosu?

## Priority Ordering After Review

1. Canonical policy bundle contract
2. Promotion ledger and benchmark gate surface
3. NLHE benchmark/dossier unblock
4. NLHE promotion with truthful evidence
5. Liar's Dice promotion with exact-exploitability provenance
6. Security-debt triage in parallel
7. Cribbage deepening
8. Bitino local adapter and same-TUI pilot
9. Same-TUI pilot checkpoint

The practical change from the original corpus is that “Bitino implementation”
is no longer treated as the opening move. The focus still points toward the
same product direction, but the actionable sequence is to make the Myosu side
stable first and then use that contract in the sibling Bitino pilot.

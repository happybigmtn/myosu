# Expand CI Proof Gates to Chain and Control-Plane Surfaces

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This plan follows `genesis/PLANS.md`.

## Purpose / Big Picture

Current CI only gates gameplay crates. This plan extends CI to chain/runtime/node, doctrine consistency checks, and focused invariant proofs so regressions are caught before merge.

## Progress

- [x] (2026-03-28 21:46Z) Confirmed `.github/workflows/ci.yml` only gates active gameplay crates.
- [ ] Split CI into jobs: gameplay, chain-core compile, doctrine checks, and focused invariant proofs.
- [ ] Add runtime/node/pallet check commands to CI with clear failure localization.
- [ ] Add canonical-spec integrity checks (non-empty required files, no duplicate-active mirrors).
- [ ] Add per-plan proof convention checks for `genesis/plans/*.md` structure.
- [ ] Add CI summary outputs mapping failing jobs to owning genesis plans.

## Surprises & Discoveries

- Observation: CI uses strong targeted gameplay tests but has zero automated checks for runtime/node/pallet.
  Evidence: `.github/workflows/ci.yml`.
- Observation: doctrine drift is currently detectable with simple file checks but not automated.
  Evidence: empty/duplicate specs found in assessment.

## Decision Log

- Decision: keep chain checks as compile/targeted-test gates first; full end-to-end devnet smoke is a later gate.
  Rationale: fast, reliable, and merge-friendly baseline.
  Inversion (failure mode): if CI starts with brittle full-devnet runs, developers will bypass checks.
  Date/Author: 2026-03-28 / Genesis

- Decision: include doctrine and plan-structure checks in CI.
  Rationale: planning/control drift is currently a top project risk.
  Inversion (failure mode): code-only CI will allow documentation contradictions to reappear.
  Date/Author: 2026-03-28 / Genesis

## Failure Modes

| Codepath | Realistic failure | Handling |
|---|---|---|
| CI chain job | Long compile causes timeout and flaky status | Split by crate and cache aggressively |
| Doctrine check | False positives due glob drift | Keep explicit required-file list in script |
| Plan checks | New plan missing proof command passes unnoticed | Add regex check for milestone + proof command blocks |

## Outcomes & Retrospective

- Pending implementation.

## Context and Orientation

Owned files in this plan:
- `.github/workflows/ci.yml`
- `.github/workflows/chain.yml` (new)
- `fabro/checks/` scripts used by CI
- `genesis/plans/001-master-plan.md` (plan index references if needed)

Not owned here:
- Runtime/node/pallet implementation details (`003`-`005`)

## Milestones

### Milestone 1: Multi-job CI split

Refactor CI into explicit jobs for gameplay, chain-core, doctrine, and plan-shape checks.

Proof command:

    rg -n "jobs:|gameplay|chain|doctrine|plans" .github/workflows/ci.yml .github/workflows/chain.yml

### Milestone 2: Chain compile gates

Add `cargo check` gates for runtime, node, and pallet.

Proof command:

    cargo check -p myosu-chain-runtime --features fast-runtime
    cargo check -p myosu-chain --features fast-runtime
    cargo check -p pallet-game-solver

### Milestone 3: Doctrine integrity gate

Fail CI when canonical specs are missing/empty or duplicate-active mirrors reappear.

Proof command:

    for f in specs/031626-00-master-index.md specs/031626-07-tui-implementation.md specs/031626-10-agent-experience.md; do test -s "$f"; done

### Milestone 4: Genesis plan quality gate

Fail CI when a genesis plan has no milestones or no proof commands.

Proof command:

    for f in genesis/plans/*.md; do rg -q "^### Milestone" "$f" && rg -q "Proof command:" "$f"; done

### Milestone 5: CI ownership mapping

Add summary output mapping each failing job to plan IDs.

Proof command:

    rg -n "Plan 00[1-9]|Plan 01[0-6]" .github/workflows/ci.yml .github/workflows/chain.yml || true

## Plan of Work

1. Split CI jobs by ownership domain.
2. Add chain compile gates.
3. Add doctrine and plan-structure checks.
4. Add ownership mapping in CI summaries.

## Concrete Steps

From `/home/r/coding/myosu`:

    sed -n '1,260p' .github/workflows/ci.yml
    rg -n "cargo check -p myosu-chain|pallet-game-solver|031626-07|031626-10|genesis/plans" .github/workflows/*.yml fabro/checks/* || true

## Validation and Acceptance

Accepted when:
- CI gates gameplay + chain-core + doctrine + plan-shape checks
- failures are attributable to owning plans/modules

## Idempotence and Recovery

- CI config updates are deterministic.
- If a new gate is too flaky, downgrade it to non-blocking for one cycle with a dated TODO and owner.

## Artifacts and Notes

- Update `outputs/operations/scorecard/review.md` with CI coverage deltas.

## Interfaces and Dependencies

Depends on: `002-spec-corpus-normalization.md`, `003-chain-runtime-reduction.md`, `004-node-devnet-minimalization.md`, `005-pallet-game-solver-simplification.md`
Blocks: `011-security-observability-release.md`

```text
source changes
    |
    v
CI jobs
  + gameplay
  + chain-core
  + doctrine
  + plan-shape
    |
    v
merge decision with plan ownership mapping
```

# 180-Day Turnaround Master Plan

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This plan follows `genesis/PLANS.md` (copied from repository `PLANS.md`) and is the dependency map for all numbered genesis plans.

## Purpose / Big Picture

In 180 days, Myosu must move from “promising but inconsistent prototype” to a provable, coherent stage-0 platform: trusted local gameplay value, a de-risked chain core, aligned doctrine, and CI-backed proof gates.

The operator-visible outcome is simple: every claim in docs can be validated by commands in CI or repeatable local proofs.

## Progress

- [x] (2026-03-28 20:30Z) Completed repo-wide assessment and generated genesis plan corpus skeleton.
- [ ] Execute Phase 0 stabilization plans (days 1-30) and close doctrine/CI/reliability gaps.
- [ ] Execute Phase 1 foundation plans (days 31-90) and establish durable module boundaries.
- [ ] Execute Phase 2 growth plans (days 91-150) and deliver integrated user + protocol features.
- [ ] Execute Phase 3 polish plans (days 151-180) and complete release-readiness and governance hardening.

## Surprises & Discoveries

- Observation: Product-facing crates are far better tested than chain/runtime/node surfaces.
  Evidence: `.github/workflows/ci.yml` only gates `myosu-games`, `myosu-tui`, `myosu-games-poker`, and `myosu-play`.
- Observation: Active `specs/` contains empty canonical files and duplicate mirrors, causing control-plane ambiguity.
  Evidence: `specs/031626-07-tui-implementation.md`, `specs/031626-10-agent-experience.md`, duplicate numbered/non-numbered files.

## Decision Log

- Decision: Run a four-phase turnaround (stabilization -> foundation -> growth -> polish) with explicit plan dependencies.
  Rationale: Sequence risk reduction before feature acceleration.
  Inversion (failure mode): If growth starts before stabilization, integration work will amplify doctrine and CI drift.
  Date/Author: 2026-03-28 / Genesis

- Decision: Keep 10 active execution workstreams plus 5 dropped-plan provenance records.
  Rationale: Satisfies carry-forward requirements without letting obsolete plans remain silently active.
  Inversion (failure mode): Silent deletion of old plans will recreate trust debt in documentation.
  Date/Author: 2026-03-28 / Genesis

## Outcomes & Retrospective

- Pending execution. This section must be updated after each phase closeout.

## Context and Orientation

Active numbered execution plans:
- `002-spec-corpus-normalization.md`
- `003-chain-runtime-reduction.md`
- `004-node-devnet-minimalization.md`
- `005-pallet-game-solver-simplification.md`
- `006-game-traits-and-poker-boundaries.md`
- `007-miner-validator-bootstrap.md`
- `008-artifact-wire-checkpoint-hardening.md`
- `009-play-tui-productization.md`
- `010-ci-proof-gates-expansion.md`
- `011-security-observability-release.md`

Dropped-plan provenance records:
- `012-dropped-031826-clean-up-myosu-for-fabro-primary-executor.md`
- `013-dropped-031826-bootstrap-fabro-primary-executor-surface.md`
- `014-dropped-031926-design-myosu-fabro-workflow-library.md`
- `015-dropped-031926-decompose-myosu-into-raspberry-programs.md`
- `016-dropped-031926-iterative-execution-and-raspberry-hardening.md`

## Milestones

### Milestone 1: Phase 0 Stabilization (Days 1-30)

Deliver doctrine integrity, CI coverage expansion, and chain/runtime risk reduction.

Included plans: 002, 003, 004, 010, 012, 013, 014, 015, 016

### Milestone 2: Phase 1 Foundation (Days 31-90)

Lock stable crate boundaries and establish miner/validator execution scaffolding.

Included plans: 005, 006, 007

### Milestone 3: Phase 2 Growth (Days 91-150)

Ship integrated product+protocol improvements around artifacts, advisor quality, and operator workflows.

Included plans: 008, 009

### Milestone 4: Phase 3 Polish (Days 151-180)

Harden security, observability, release governance, and no-ship checks.

Included plans: 011

## Plan of Work

Execute plans in dependency order, not numeric order when prerequisites require it:
1. 002 -> 010 (control-plane and proof reliability first)
2. 003 -> 004 -> 005 (chain runtime/node/pallet stabilization)
3. 006 -> 008 -> 009 (game/product quality and artifact trust)
4. 007 after 003/004/005 stabilize runtime interfaces
5. 011 after all critical paths have measurable proof surfaces

## Concrete Steps

From repo root (`/home/r/coding/myosu`):

    # Phase 0 kickoff checks
    cargo check -p myosu-games -p myosu-games-poker -p myosu-tui -p myosu-play
    cargo test -p myosu-play smoke_report_proves_preflop_to_flop_progression --quiet

    # Track plan status transitions in each numbered plan file
    rg -n "^- \[[ x]\]" genesis/plans/*.md

## Validation and Acceptance

Master plan is accepted when:
- All active numbered plans (002-011) are completed with proof commands recorded.
- Dropped-plan provenance files (012-016) exist and explain disposition.
- `genesis/GENESIS-REPORT.md` summarizes outcomes and unresolved risks.

Proof command:

    ls genesis/plans/00{2..9}-*.md genesis/plans/01{0..6}-*.md

## Idempotence and Recovery

- Running the same plan validation commands repeatedly is safe.
- If a downstream plan is blocked, update its `Progress` and `Decision Log`; do not rewrite history in master.

## Artifacts and Notes

- Assessment baseline: `genesis/ASSESSMENT.md`
- Durable spec baseline: `genesis/SPEC.md`
- Design baseline: `genesis/DESIGN.md`

## Interfaces and Dependencies

ASCII dependency view:

```text
002 ---> 010 ---> 003 ---> 004 ---> 005 ---> 007
  \                               \
   \--> 006 -----------------------> 008 ---> 009 ---> 011

012/013/014/015/016 are provenance-only and do not block execution
```


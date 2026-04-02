# Genesis ExecPlan Conventions

This file governs the numbered execution plans under `genesis/plans/`. For the
repository-wide ExecPlan contract, see the root `PLANS.md`.

## Plan Format

Every numbered plan follows this skeleton:

```md
# NNN - <Short, action-oriented title>

## Purpose / Big Picture

## Context and Orientation

## Architecture

## Progress
- [x] (pre-satisfied) M1. ...
- [ ] M2. ...
  - Surfaces: `path/a`, `path/b`
  - What exists after: ...
  - Why now: ...
  - Proof: `specific command`
  - Tests: `specific test command`

## Surprises & Discoveries

## Decision Log
- Decision: ...
  - Why: ...
  - Failure mode: ...
  - Mitigation: ...
  - Reversible: yes/no

## Validation and Acceptance

## Outcomes & Retrospective
_Updated after milestones complete._
```

## Numbering

- `001` is always the master plan (180-day roadmap).
- `002`--`NNN` are individual workstreams ordered by dependency and priority.
- Plans are never renumbered once published. Dropped plans are marked
  `Status: Dropped` with rationale. New plans get the next available number.

## Quality Requirements

- 3--8 milestones per plan.
- 80--200 lines per plan unless there is a compelling reason.
- Concrete repo-relative file paths in Surfaces.
- Specific proof commands (not vague `cargo test`).
- At least one failure scenario per plan.
- Pre-satisfied milestones marked `(pre-satisfied)` with verification evidence.

## Relationship to Prior Plans

The `genesisarchive/plans/` directory contains 21 plans from the previous genesis
run (2026-03-28/30). This genesis run carries forward the verified work,
consolidates redundant plans, and drops plans whose purpose is fully satisfied.

Disposition of prior plans:

| Prior Plan | Disposition | Reason |
|-----------|-------------|--------|
| 002 Spec Corpus Normalization | Dropped | Fully verified -- specs are normalized |
| 003 Chain Runtime Reduction | Carried forward as 003 | Stubs exist; surface unreduced |
| 004 Node Devnet Minimalization | Merged into 003 | Same workstream |
| 005 Pallet Game-Solver Simplification | Merged into 003 | Same workstream |
| 006 Game Traits and Poker Boundaries | Dropped | Fully verified |
| 007 Miner-Validator Bootstrap | Dropped | Fully verified |
| 008 Artifact Wire Checkpoint Hardening | Dropped | Fully verified |
| 009 Play TUI Productization | Dropped | Fully verified |
| 010 CI Proof Gates Expansion | Carried forward as 004 | Needs genesis plan sync |
| 011 Security Observability Release | Carried forward as 005 | Stub only |
| 012 Multi-Game Architecture Proof | Dropped | Fully verified |
| 013 Integration Test Harness | Carried forward as 006 | Partially verified |
| 014 OS Refresh and Operator Docs | Carried forward as 007 | Doctrine staleness |
| 015--019 | Dropped | Meta/process plans, fully satisfied or not applicable |
| 020 Second-Game Subnet Proof | Dropped | Fully verified |
| 021 Operator Hardening | Dropped | Fully verified |

## CI Integration

The CI script `.github/scripts/check_plan_quality.sh` validates that genesis
plans in the `0[0-2][0-9]-*.md` range (excluding 001) contain:

- At least one `### Milestone` heading
- At least one `Proof command:` line

The CI script `.github/scripts/check_stage0_repo_shape.sh` requires specific
genesis plan files to exist. When plans are renumbered, this script must be
updated in the same commit.

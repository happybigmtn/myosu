# 002 - CI and Genesis Plan Sync

## Purpose / Big Picture

The CI pipeline hardcodes references to specific genesis plan filenames. This
plan synchronizes CI scripts with the new 13-plan corpus (renumbered from the
prior 21-plan set). Without this sync, CI breaks on the first push.

## Context and Orientation

`.github/scripts/check_stage0_repo_shape.sh` requires:
- `genesis/plans/002-spec-corpus-normalization.md`
- `genesis/plans/010-ci-proof-gates-expansion.md`
- `genesis/plans/020-second-game-subnet-execution-proof.md`

`.github/scripts/check_plan_quality.sh` validates plans in `0[0-2][0-9]-*.md`
range contain `### Milestone` headings and `Proof command:` lines.

These scripts must be updated to match the new plan numbering.

## Architecture

No architectural changes. This is a CI configuration sync.

## Progress

- [x] (pre-satisfied) M1. Genesis plans written with new numbering scheme
  - Surfaces: `genesis/plans/`
  - What exists after: Plans 001--013 with consistent numbering
  - Why now: Foundation for all subsequent work
  - Proof: `ls genesis/plans/0*.md | wc -l`
  - Tests: N/A

### Milestone 2: Update repo shape check

- [ ] M2. Update `check_stage0_repo_shape.sh` to reference new plan filenames
  - Surfaces: `.github/scripts/check_stage0_repo_shape.sh`
  - What exists after: Script references plans that actually exist
  - Why now: CI will fail without this
Proof command: `bash .github/scripts/check_stage0_repo_shape.sh`
  - Tests: `bash .github/scripts/check_stage0_repo_shape.sh`

### Milestone 3: Update plan quality check format

- [ ] M3. Ensure all plans have `### Milestone` headings and `Proof command:` lines
  - Surfaces: `genesis/plans/002-*.md` through `genesis/plans/013-*.md`
  - What exists after: Plan quality check passes for all plans
  - Why now: Gate for all subsequent CI runs
  - Proof: `bash .github/scripts/check_plan_quality.sh`
  - Tests: `bash .github/scripts/check_plan_quality.sh`

### Milestone 4: Verify full CI passes

- [ ] M4. Push branch and verify all 7 CI jobs pass
  - Surfaces: `.github/workflows/ci.yml`
  - What exists after: Green CI on trunk with new genesis corpus
  - Why now: Gate for all subsequent work
  - Proof: `gh run list --branch trunk --limit 1`
  - Tests: All 7 CI jobs green

## Surprises & Discoveries

- The plan quality script uses `has_milestone()` matching `^### Milestone `.
  Plans must use this exact heading format.
- The plan quality script uses `has_proof_command()` matching `^Proof commands?:`.
  Plans must include `Proof command:` or `Proof commands:` as a line prefix.

## Decision Log

- Decision: Update CI scripts rather than preserving old plan filenames.
  - Why: The old numbering (002--021) is confusing when 12 plans are dropped.
  - Failure mode: Missing a CI reference to an old filename.
  - Mitigation: `rg "genesis/plans" .github/` to find all references.
  - Reversible: yes

## Validation and Acceptance

All 7 CI jobs pass with the new genesis plan corpus.

## Outcomes & Retrospective
_Updated after milestones complete._

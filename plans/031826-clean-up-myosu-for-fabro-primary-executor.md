# Clean Up Myosu for Fabro Primary Execution

This ExecPlan is a living document. The sections `Progress`,
`Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must
be kept up to date as work proceeds.

`PLANS.md` is checked into the repository root and this document must be
maintained in accordance with it. This plan depends on
`specs/031826-fabro-primary-executor-decision.md` and
`specs/031826-myosu-fabro-primary-executor-migration.md`.

## Purpose / Big Picture

After this slice lands, Myosu will have a clean Fabro-aligned planning surface.
A contributor will be able to find current durable repo strategy in `specs/`,
find live implementation slices in `plans/`, and still recover the earlier
product corpus from `specsarchive/`. This does not finish the Fabro migration,
but it removes the biggest source-of-truth ambiguity that would otherwise block
Fabro from becoming the primary executor.

## Progress

- [x] (2026-03-19 03:17Z) Reviewed `/home/r/coding/fabro/PLANS.md`,
  `/home/r/coding/fabro/SPEC.md`, and both Fabro plans under
  `/home/r/coding/fabro/plans/`.
- [x] (2026-03-19 03:17Z) Reviewed `OS.md` and the full legacy `specs/`
  heading inventory to understand Myosu's current doctrine and duplicate-spec
  problem.
- [x] (2026-03-19 03:17Z) Identified that current repo doctrine still points to
  `specs/031626-00-master-index.md` and `ralph/IMPLEMENT.md`, so the first
  cleanup slice must preserve old paths where practical.
- [x] (2026-03-19 03:17Z) Archived the legacy spec tree unchanged by renaming
  `specs/` to `specsarchive/`.
- [x] (2026-03-19 03:17Z) Added root `PLANS.md` and `SPEC.md` using the Fabro
  framework as the new canonical rules for plans and specs.
- [x] (2026-03-19 03:17Z) Created a new `specs/` set consisting of a canonical
  index, a repo-strategy decision spec, and a Fabro migration spec.
- [x] (2026-03-19 03:17Z) Created a new `plans/` set consisting of this cleanup
  plan and a follow-on plan for retargeting the remaining doctrine and
  operational surfaces.
- [x] (2026-03-19 04:09Z) Follow-on slice rewrote `OS.md`, `AGENTS.md`, and
  the active Fabro migration specs so the repo points at the new Fabro-aligned
  doctrine surface and treats deleted Malinka files as gone.
- [x] (2026-03-19 04:10Z) Archived the detailed contents of
  `ralph/IMPLEMENT.md` and `ops/malinka-capabilities.md`, then replaced the
  live filenames with historical stubs that redirect to Fabro/Raspberry.

## Surprises & Discoveries

- Observation: the legacy spec tree contains both zero-padded and non-padded
  filenames for overlapping topics.
  Evidence: `specsarchive/` now contains pairs such as
  `031626-04a-miner-binary.md` and `031626-miner-binary.md`.

- Observation: the current doctrine still depends on the path
  `specs/031626-00-master-index.md`.
  Evidence: `OS.md`, `project.yaml`, and `AGENTS.md` all reference that path,
  so preserving it in the new canonical set avoids immediate breakage.

- Observation: the worktree already contains user changes in `AGENTS.md`,
  `project.yaml`, `ops/malinka-capabilities.md`, and `ralph/IMPLEMENT.md`.
  Evidence: `git status --short` before edits showed those files as modified.

## Decision Log

- Decision: preserve the stable path `specs/031626-00-master-index.md` inside
  the new canonical `specs/` set.
  Rationale: several doctrine files already point there, and preserving the
  path keeps the first cleanup slice small and non-destructive.
  Date/Author: 2026-03-19 / Codex

- Decision: archive the full old `specs/` tree unchanged instead of deleting or
  partially rewriting it.
  Rationale: the historical corpus still contains product reasoning that would
  be expensive to reconstruct and is still useful during migration.
  Date/Author: 2026-03-19 / Codex

- Decision: keep the first cleanup slice documentation-only and avoid touching
  already-dirty doctrine/config files.
  Rationale: the worktree already contains user edits in key files, and the
  immediate goal is to establish the new canonical planning surface safely.
  Date/Author: 2026-03-19 / Codex

- Decision: seed one follow-on plan immediately instead of stopping with only
  the cleanup slice.
  Rationale: Fabro adoption needs a visible next step, and a fresh contributor
  should be able to continue from the new structure without re-deriving the
  next migration target.
  Date/Author: 2026-03-19 / Codex

## Outcomes & Retrospective

The repository now has the minimum Fabro-aligned doctrine surface it was
missing:

- root `PLANS.md`
- root `SPEC.md`
- a fresh `specs/` directory with current canonical docs
- a fresh `plans/` directory with current executable slices
- `specsarchive/` preserving the older corpus

The main remaining gap is now smaller doctrine retargeting. `project.yaml` and
`WORKFLOW.md` are gone, and `ralph/IMPLEMENT.md` plus
`ops/malinka-capabilities.md` are now explicit archive stubs. The next slice
must remove the remaining stale references and promote the Fabro/Raspberry
surfaces as the only active path.

## Context and Orientation

Before this cleanup, Myosu had a large `specs/` directory containing product
vision, architecture, acceptance criteria, and executor-specific notes all in
one place. The repository did not have root `PLANS.md` or `SPEC.md`, which
meant there was no small, current, Fabro-friendly explanation of how durable
specs should differ from live implementation plans.

The relevant files are:

- `OS.md`, which still describes the product mission, stage gates, priorities,
  and no-ship conditions
- `INVARIANTS.md`, which still defines hard system boundaries
- `specsarchive/`, which now contains the full earlier spec corpus
- `SPEC.md` and `PLANS.md`, which are the new canonical rules for current work

A **durable spec** describes a boundary that should still make sense months
later. A **plan** is a living document that tracks the next bounded slice while
work is underway. This cleanup establishes those two layers explicitly.

## Milestones

### Milestone 1: Audit the old doctrine against the Fabro framework

Read the Fabro framework files and Myosu's current doctrine to identify what
must be preserved, what is duplicated, and what paths are already depended on
by the repo. The proof is a clear migration decision about what becomes the new
canonical layer and what moves to archive.

### Milestone 2: Create the new canonical planning surface

Archive the old spec tree, add root `PLANS.md` and `SPEC.md`, and create a new
minimal `specs/` set plus a new `plans/` set. The proof is that a newcomer can
list the repo root and see a clear Fabro-aligned doctrine surface without
losing the historical corpus.

### Milestone 3: Seed the next migration slice

Write a follow-on ExecPlan that explains how to retarget the remaining doctrine
and operational entrypoints. The proof is that the repo cleanup does not end in
a dead-end; it points directly to the next bounded slice.

## Plan of Work

First, inspect the Fabro framework and Myosu doctrine so the cleanup reflects
actual repository dependencies rather than a guessed structure. Next, rename
the old `specs/` directory to `specsarchive/` so history is preserved as-is.
Then add `PLANS.md` and `SPEC.md` at the repository root using the Fabro
framework.

After the framework files exist, create a new canonical `specs/` directory.
Seed it with a small index file that keeps the existing
`specs/031626-00-master-index.md` path alive, a decision spec that records the
repo-strategy change, and a migration spec that explains how Fabro becomes the
primary executor. Finally, create a new `plans/` directory with this cleanup
plan and a follow-on migration plan for the next slice.

## Concrete Steps

Work from the repository root.

1. Audit the current and target doctrine surfaces.

       sed -n '1,260p' /home/r/coding/fabro/PLANS.md
       sed -n '1,260p' /home/r/coding/fabro/SPEC.md
       for f in /home/r/coding/fabro/plans/*.md; do echo "===== ${f##*/} ====="; sed -n '1,220p' "$f"; done
       rg -n '^(#|##|###) ' OS.md
       for f in specs/*.md; do printf '\n--- %s ---\n' "${f##*/}"; rg -n '^(#|##|###) ' "$f"; done

2. Preserve the old corpus and create the new root directories.

       mv specs specsarchive
       mkdir -p specs plans

3. Add the new framework and seeded docs.

       test -f PLANS.md
       test -f SPEC.md
       test -f specs/031626-00-master-index.md
       test -f specs/031826-fabro-primary-executor-decision.md
       test -f specs/031826-myosu-fabro-primary-executor-migration.md
       test -f plans/031826-clean-up-myosu-for-fabro-primary-executor.md
       test -f plans/031826-bootstrap-fabro-primary-executor-surface.md

4. Verify the resulting tree.

       find . -maxdepth 2 \( -path './.git' -o -path './target' \) -prune -o \
         \( -path './specs' -o -path './plans' -o -path './specsarchive' \) -print

## Validation and Acceptance

Acceptance is complete when all of the following are true:

- `PLANS.md` and `SPEC.md` exist at the repo root
- `specsarchive/` contains the full older spec corpus
- `specs/` contains a canonical index plus current decision and migration specs
- `plans/` contains this cleanup plan and a follow-on plan
- the legacy path `specs/031626-00-master-index.md` resolves to the new
  canonical index

Run:

    test -d specsarchive && test -d specs && test -d plans
    test -f PLANS.md && test -f SPEC.md
    rg --files specs plans specsarchive | sort

Expect `rg --files` to show files under all three directories, with the old
corpus only under `specsarchive/` and the fresh Fabro-aligned docs under
`specs/` and `plans/`.

## Idempotence and Recovery

This cleanup is mostly additive after the one directory rename. Re-running the
file-creation steps is safe because the canonical files are written to stable
paths. If the rename already happened, do not move `specsarchive/` again;
instead confirm the archive exists and continue with the new root directories.

If a later slice needs to roll back, the safest approach is not to delete the
new docs. Instead, keep both surfaces and explicitly mark which one is
canonical. The archive exists specifically to make recovery safe.

## Artifacts and Notes

Important resulting paths:

    PLANS.md
    SPEC.md
    specs/031626-00-master-index.md
    specs/031826-fabro-primary-executor-decision.md
    specs/031826-myosu-fabro-primary-executor-migration.md
    plans/031826-clean-up-myosu-for-fabro-primary-executor.md
    plans/031826-bootstrap-fabro-primary-executor-surface.md
    specsarchive/

## Interfaces and Dependencies

This slice changes only documentation and repository layout. The important
interfaces are path-level interfaces:

- `SPEC.md` defines the durable-spec contract for current work
- `PLANS.md` defines the ExecPlan contract for current work
- `specs/` is the canonical durable doctrine directory
- `plans/` is the canonical live-plan directory
- `specsarchive/` is the preserved historical doctrine directory

Revision Note: Initial draft created while landing the repository-cleanup slice
so the new doctrine surface is itself documented by the same framework it
introduces.

# Specification: Fabro as Primary Executor for Myosu

Status: Accepted
Date: 2026-03-18
Type: Decision Spec
Supersedes: the assumption that `ralph/IMPLEMENT.md` and the legacy `specs/`
tree are the primary planning surface for Myosu

## Decision

Myosu adopts the Fabro documentation model as its canonical planning framework.
Durable architectural, migration, and repo-strategy decisions now live in
root-level `SPEC.md`-governed files under `specs/`. Bounded implementation
slices now live in root-level `PLANS.md`-governed files under `plans/`. The
prior spec corpus is preserved in `specsarchive/` as historical context, not as
the canonical source for new work.

Fabro becoming the primary executor also means future operational work should
be shaped for Fabro-native supervision and run truth rather than for a
Malinka-only task loop. `project.yaml` and `WORKFLOW.md` have already been
deleted. `ralph/IMPLEMENT.md` remains as historical compatibility context until
follow-on migration slices either retire it or reduce it to archive-only
status.

## Why Now

- The current repo has strong product intent but no small canonical planning
  surface for Fabro.
- The legacy spec tree mixes durable product doctrine, duplicate documents,
  greenfield assumptions, and Malinka deployment notes.
- Contributors currently have to infer whether a document is a durable
  architecture boundary, a live task tracker, or historical context.
- Fabro cannot safely become the primary executor while the repo still points
  first to the older planning model.

## Alternatives Considered

- Keep the current tree as canonical and add `PLANS.md` / `SPEC.md` later.
  This was rejected because it leaves the same ambiguity in place and makes
  future automation read from two competing sources of truth.
- Rewrite every legacy spec into the new framework in one pass.
  This was rejected because it is too wide for the first cleanup slice and
  risks losing useful historical reasoning while the repo is still in flux.
- Delete the old spec corpus entirely.
  This was rejected because the old documents still contain product,
  sequencing, and operational context that will be needed during migration.

## Consequences

- New durable work begins in `specs/` and `plans/`, not in `specsarchive/`.
- The stable path `specs/031626-00-master-index.md` stays alive as the new
  doctrine entrypoint so existing references do not break immediately.
- Follow-on slices must retarget `OS.md`, `AGENTS.md`, and the remaining
  historical `ralph` surfaces to the new canonical Fabro/Raspberry surfaces.
- The deleted Malinka control files are not to be recreated. Remaining
  historical documents are reference-only unless explicitly re-adopted.

## What Is Now Superseded

- The old `specs/` directory as the canonical place for new planning work.
- The assumption that Myosu's primary execution story should stay centered on
  the Malinka `ralph/IMPLEMENT.md` loop.
- The habit of using large product specs as both durable architecture docs and
  live implementation checklists.

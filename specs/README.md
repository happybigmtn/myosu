# Specs Surface

`specs/` is still a live repo surface.

Use the directory like this:

- `specs/050426-*.md` capture the current stage-0 contracts, evidence, and
  review-aligned acceptance criteria for the checked-in repo.
- `specs/031626-*.md` remain active bootstrap doctrine and product specs; they
  are still referenced by `AGENTS.md` and the repo mission doctrine.
- `specsarchive/` and other historical planning surfaces are reference-only.

When behavior changes, update the matching spec in the same slice instead of
leaving drift for a later cleanup pass.

Durable design decisions belong in `docs/adr/`. Execution state belongs in
`IMPLEMENTATION_PLAN.md`, `REVIEW.md`, `ARCHIVED.md`, `WORKLIST.md`, and the
Fabro/Raspberry manifests under `fabro/`.

# Retire Malinka and Cut Over to No-Autodev Doctrine

Status: Completed locally on 2026-03-30.

This ExecPlan is a living document. Maintained per `genesis/PLANS.md`.

## Purpose / Big Picture

The operator has decided to stop treating Malinka and autodev as part of Myosu's active future. That choice needs to become concrete in the repo: Malinka artifacts must either be deleted or archived as historical, and any live doctrine that still assumes autodev must be removed.

After this plan, the active repo story assumes no Malinka, no autodev, and no hidden dependency on Malinka-era control files. Historical Malinka material may remain only under clearly historical locations such as `archive/`, not as active working surfaces.

## Progress

- [x] (2026-03-28) Confirmed that Malinka artifacts and Malinka-era specs still exist in the live repo.
- [x] (2026-03-30) Inventoried the remaining legacy executor surface in the
  working tree. The active repo-root footprint is concentrated in the
  `malinka/` tree (~4.4 MB across blueprints, plan mappings, programs,
  prompts, run configs, and workflows), while active-doc drift still survives
  in `genesis/ASSESSMENT.md`, `fabro/programs/README.md`, and
  `fabro/workflows/README.md`.
- [x] (2026-03-30) Classified the first cleanup targets. The root `malinka/`
  tree should be archived rather than normalized in place; active-doc
  references in `genesis/` and `fabro/` should be rewritten in place; the
  already-historical enhancement specs can remain under `specsarchive/`.
- [x] (2026-03-30) Archived the root `malinka/` tree under `archive/malinka/`,
  removing it from the active repo root without deleting the historical
  material.
- [x] (2026-03-30) Scrubbed the remaining active-doc references in
  `genesis/ASSESSMENT.md`, `fabro/programs/README.md`, and
  `fabro/workflows/README.md`. The active-doc sweep over `README.md`, `OS.md`,
  `AGENTS.md`, `genesis/ASSESSMENT.md`, `fabro/`, and `outputs/` is now clean.

## Surprises & Discoveries

- Observation: Earlier doctrine cleanup removed some Malinka-only control files, but not the full Malinka footprint.
  Evidence: `malinka/` still exists at the repo root with plan mappings and a blueprint.
- Observation: The generated assessment still treated the Malinka enhancement spec as active.
  Evidence: `genesis/ASSESSMENT.md` classified `031626-99-malinka-enhancements` as active, low priority.

## Decision Log

- Decision: Prefer deletion over normalization for Malinka-era active surfaces.
  Rationale: The repo doctrine says deleted Malinka-only control files should not be recreated; carrying the remaining artifacts as live surfaces would contradict that.
  Inversion: Keeping Malinka files in place but "ignoring" them invites future synth runs and contributors to treat them as current again.
  Date/Author: 2026-03-28 / Codex

- Decision: Historical preservation, when needed, belongs under `archive/`, not in the active repo root.
  Rationale: The operator wants Malinka gone from the active working surface, not cosmetically demoted.
  Date/Author: 2026-03-28 / Codex

## Failure Modes

| Codepath | Realistic failure | Handling |
|---|---|---|
| Artifact cleanup | Delete something still needed for Fabro/Raspberry | Inventory first; only remove Malinka-specific surfaces after reference sweep |
| Spec cleanup | Historical rationale is lost | Move truly historical items into `archive/` with a short manifest note when deletion would erase useful provenance |
| Reference sweep | Hidden autodev assumptions remain | Run repo-wide `rg` proofs after each cleanup slice |

## Outcomes & Retrospective

The first useful `015` discovery is that the real problem is smaller and more
concrete than "the whole repo still thinks Malinka is active." The active
root-level legacy surface is now mostly the `malinka/` directory itself plus a
small number of active-doc references. That means the cleanup can proceed in
two deliberate phases: rewrite the active docs now, then archive the root
legacy tree cleanly instead of churning through every historical reference at
once.

That shape held all the way through completion. The right end state was not
"erase every historical mention." It was "remove the active root-level legacy
executor surface and keep the active doctrine clean." The archive now carries
the preserved material, while the live repo surface no longer advertises that
execution model as current.

## Context and Orientation

Known likely targets:
- `malinka/`
- `specs/031626-99-malinka-enhancements.md`
- `specs/031626-malinka-enhancements.md`
- any active docs that still say `autodev`
- any generated plan/report text that still treats Malinka as current

## Milestones

### Milestone 1: Inventory and classify

Produce a complete inventory of legacy-executor references and classify each as
delete, archive, or rewrite.

Proof command:

    rg -n "malinka|autodev" README.md OS.md AGENTS.md genesis fabro outputs || true
    find malinka -maxdepth 3 -type f 2>/dev/null | sort || true

### Milestone 2: Remove active Malinka surfaces

Delete or archive the `malinka/` tree and any remaining root-level
legacy-executor-only specs so they no longer exist as active repo-root working
surfaces.

Proof command:

    test ! -d malinka
    test ! -f specs/031626-99-malinka-enhancements.md
    test ! -f specs/031626-malinka-enhancements.md

### Milestone 3: Scrub active doctrine

Rewrite remaining active docs so they assume no legacy executor path.

Proof command:

    rg -n "malinka|autodev" README.md OS.md AGENTS.md genesis/ASSESSMENT.md fabro outputs || true

## Plan of Work

1. Inventory references.
2. Delete or archive Malinka artifacts.
3. Rewrite active docs to remove autodev assumptions.

## Concrete Steps

From `/home/r/coding/myosu`:

    rg -n "malinka|autodev" . -g '!archive/**' -g '!target/**' || true
    find malinka -maxdepth 3 -type f 2>/dev/null | sort || true

## Validation and Acceptance

Accepted when:
- `malinka/` is gone from the active repo root.
- Malinka-only specs are removed from the active `specs/` surface.
- Active doctrine no longer refers to autodev as a current execution mode.
- The remaining references are either this plan itself or clearly historical
  archive surfaces.

## Idempotence and Recovery

Deletion is irreversible, so archive first when there is any doubt about provenance value. Once archived, the active repo should still contain no Malinka working surfaces.

## Interfaces and Dependencies

Depends on: 014.
Blocks: 016, 018, and 019, because future doctrine and synth governance should not keep inheriting Malinka assumptions.

```text
malinka/
specs/*malinka*
docs mentioning autodev
        |
        v
archive/ (optional)
        |
        v
clean active doctrine
```

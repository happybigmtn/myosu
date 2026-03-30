# Rationalize Fabro Workflow and Program Surfaces

Status: Completed locally on 2026-03-30.

This ExecPlan is a living document. Maintained per `genesis/PLANS.md`.

## Purpose / Big Picture

The repo now contains a real but uneven Fabro surface: multiple workflows, run configs, program manifests, and output roots exist, but their trust level and intended use are not obvious. This plan classifies those surfaces so contributors can tell what is live, what is secondary, and what is only a draft.

After this plan, `fabro/` will read like an intentional execution substrate instead of a pile of artifacts from multiple planning eras.

## Progress

- [x] (2026-03-28) Confirmed that `fabro/` contains a live bootstrap path plus several broader planning, services, maintenance, and review-promote surfaces.
- [x] (2026-03-30) Classified the checked-in workflow families, run-config
  families, and program manifests using a common active/secondary framing.
- [x] (2026-03-30) Updated the Fabro READMEs so those classifications are
  explicit and include concrete examples of active and secondary files.
- [x] (2026-03-30) Made the delete/archive decision for this phase: none of the
  checked-in Fabro surfaces are misleading enough to remove right now once they
  are labeled clearly, so no Fabro workflow/program files were archived or
  deleted in this slice.

## Surprises & Discoveries

- Observation: The live bootstrap path is relatively clean; the surrounding Fabro surface is the confusing part.
  Evidence: `fabro/programs/myosu-bootstrap.yaml` is easy to explain, while the broader set of manifests and run-config families needs interpretation.

## Decision Log

- Decision: Prefer explicit classification over premature deletion for Fabro surfaces.
  Rationale: Unlike Malinka, these files are already part of the active execution substrate; the immediate problem is ambiguity, not illegitimacy.
  Date/Author: 2026-03-28 / Codex

## Failure Modes

| Codepath | Realistic failure | Handling |
|---|---|---|
| Classification | A useful but secondary workflow gets deleted too early | Label first, delete only after one more review pass |
| READMEs | Readmes describe categories but not file-level examples | Include concrete examples of one active and one secondary file in each README |

## Outcomes & Retrospective

This plan closed without needing destructive cleanup. The repo's Fabro surface
was uneven, not illegitimate. Once the program, workflow, and run-config
READMEs all used the same explicit classification scheme, the checked-in
execution substrate became much easier to explain without deleting useful
secondary surfaces prematurely.

## Context and Orientation

Relevant surfaces:
- `fabro/programs/`
- `fabro/workflows/`
- `fabro/run-configs/`
- `outputs/`

## Milestones

### Milestone 1: Surface inventory and classification

Create a table in the relevant README(s) that classifies each family or manifest.

Proof command:

    find fabro -maxdepth 3 -type f | sort

### Milestone 2: Explain active versus secondary

Update README-level docs so a newcomer can tell which Fabro surfaces are part of the active operator loop.

Proof command:

    sed -n '1,220p' fabro/programs/README.md
    sed -n '1,220p' fabro/workflows/README.md

### Milestone 3: Archive or delete misleading leftovers

If any Fabro surfaces are no longer defensible even as secondary or experimental files, archive or delete them.

Proof command:

    rg -n "active|secondary|experimental|historical" fabro/programs/README.md fabro/workflows/README.md

## Plan of Work

1. Inventory the Fabro surface.
2. Classify each family.
3. Archive or delete only after classification.

## Concrete Steps

From `/home/r/coding/myosu`:

    find fabro -maxdepth 3 -type f | sort
    sed -n '1,200p' fabro/programs/README.md
    sed -n '1,200p' fabro/workflows/README.md

## Validation and Acceptance

Accepted when:
- The Fabro surface is intentionally classified.
- READMEs explain the active versus secondary split.
- No obviously misleading Fabro surfaces remain unlabeled.

## Idempotence and Recovery

Classifications can be revised as the control plane evolves. Preserve clear labels even when surfaces move between categories.

## Interfaces and Dependencies

Depends on: 016.
Blocks: 018, because Genesis adjudication should reference the classified Fabro surface, not an ambiguous one.

```text
fabro/programs/
fabro/workflows/
fabro/run-configs/
        |
        v
classified execution substrate
```

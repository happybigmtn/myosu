# Specification: Myosu Fabro Primary Executor Migration

Status: Draft
Date: 2026-03-18
Type: Migration / Port Spec
Supersedes: the repo strategy where Myosu product doctrine and implementation
execution are centered on the legacy Malinka / `ralph` surface

## Purpose / User-Visible Outcome

After this migration, a contributor can enter the Myosu repository and find one
Fabro-aligned doctrine surface for durable decisions and one Fabro-aligned
planning surface for live implementation slices. The product mission remains
the same, but the repo becomes much easier for Fabro to supervise: the old spec
corpus is archived, current direction is concentrated in a smaller set of
living documents, and legacy Malinka-specific entrypoints become explicit
bridges instead of unspoken defaults.

The immediate user-visible result is cleaner operator truth for humans and
agents. A newcomer no longer needs to infer which of several overlapping spec
files is current. They can read `SPEC.md`, `PLANS.md`, the new `specs/`, and
the new `plans/`, then continue from there.

## Whole-System Goal

Current state:

- Myosu already has a strong product mission in `OS.md`, `INVARIANTS.md`, and
  the old spec corpus.
- The repository already contains real code, but many legacy specs still speak
  as if the repo were greenfield.
- The old `specs/` tree mixed multiple roles at once: product architecture,
  migration intent, live build sequencing, and Malinka deployment notes.
- Root-level `PLANS.md` and `SPEC.md` did not exist before the cleanup slice.
- `project.yaml` and `WORKFLOW.md` have now been deleted.
- `OS.md`, `AGENTS.md`, and `ralph/IMPLEMENT.md` still preserve parts of the
  older Malinka / `ralph` framing.

This migration adds:

- a canonical Fabro-aligned doctrine layer in `specs/`
- a canonical ExecPlan layer in `plans/`
- a preserved historical archive in `specsarchive/`
- a phased path to retarget Myosu's operational entrypoints toward Fabro as the
  primary executor

If this migration lands:

- new work starts from `SPEC.md`, `PLANS.md`, `specs/`, and `plans/`
- the pre-Fabro spec corpus remains available without blocking current work
- Myosu's build-process doctrine becomes legible to Fabro and to human
  contributors without implicit chat context

Still not solved here:

- a full rewrite of all older product specs into the new framework
- the exact Fabro program manifests and run configs that will replace the
  remaining `ralph` loop
- the final retirement date for every Malinka-specific file in the repo

12-month direction:

- Myosu runs its real chain, miner, validator, gameplay, and operational work
  through Fabro-native supervisory surfaces
- legacy Malinka-only doctrine survives only as historical reference or thin
  compatibility glue

## Current State

- `OS.md` remains the best single narrative of Myosu's mission, stage gates,
  priorities, and no-ship doctrine.
- `OS.md` and `AGENTS.md` now describe Fabro/Raspberry as the active path, but
  `ralph/IMPLEMENT.md` and `specsarchive/` still survive as historical context.
- `ralph/IMPLEMENT.md` still exists as a historical implementation contract
- the old spec corpus contains duplicate filenames, overlapping coverage, and
  both product and executor docs in the same directory
- the repo already includes copied or in-progress chain/runtime code, so some
  older "greenfield" wording is no longer literally true

## Target Architecture

Myosu should have four doctrine layers:

1. Stable operator doctrine that remains product-facing:
   `OS.md`, `INVARIANTS.md`, and `ops/`
2. Durable Fabro-aligned planning doctrine:
   `SPEC.md` plus a small canonical set of current specs in `specs/`
3. Live bounded implementation slices:
   `PLANS.md` plus current ExecPlans in `plans/`
4. Historical context:
   `specsarchive/` and any future archive directories

The key rule is that archived product reasoning may remain readable, but new
durable direction must be restated in the new canonical layer instead of being
left trapped in the archive.

Fabro as the primary executor means the repo eventually needs a Fabro-native
operational surface as well: program manifests, run configs, or other
supervisory truth that describe Myosu's real workstreams without requiring the
old Malinka loop as the first-class entrypoint.

## What Ports Directly

- the Myosu mission, stage model, priorities, and no-ship criteria from `OS.md`
- hard invariants from `INVARIANTS.md`
- risk, KPI, scorecard, and evidence surfaces from `ops/`
- the product intent behind the old chain, miner, validator, gameplay, and
  multi-game specs

## What Ports Selectively

- acceptance-criteria detail from the old spec corpus into smaller capability
  specs or into ExecPlans when the detail is implementation-slice specific
- recurring-lane ideas from the deleted Malinka configuration into future
  Fabro-native supervisory manifests if they still prove useful
- useful Malinka deployment lessons from
  `specsarchive/031626-99-malinka-enhancements.md` into future Fabro execution
  policy where those lessons still apply

## What Does Not Port

- duplicate filenames and overlapping spec variants from the old corpus
- the habit of using one giant spec tree as both durable architecture and live
  implementation tracker
- the assumption that every current contributor must start by reading the
  archived corpus in full
- Malinka-only RESULT/BLOCKED workflow semantics as the long-term primary
  control surface

## Transitional Bridges

During migration, Myosu may temporarily keep:

- `ralph/IMPLEMENT.md` as a compatibility tracker
- `AGENTS.md` and `OS.md` references that have not yet been retargeted

These bridges are acceptable only while the new doctrine surface is being
adopted. They must be explicit bridges, not silent competing sources of truth.

## Migration Phases

### Phase 1: Clean up the canonical doctrine surface

Archive the legacy spec tree unchanged, add root `SPEC.md` and `PLANS.md`,
create a new canonical `specs/` set, and seed `plans/` with the first Fabro
aligned implementation slices.

### Phase 2: Retarget doctrine and repo configuration

Update `OS.md`, `AGENTS.md`, and the remaining historical `ralph` references so
they point to the new doctrine surface first and treat legacy `ralph` artifacts
as reference-only.

### Phase 3: Introduce Fabro-native operational truth for Myosu workstreams

Define the Fabro-native program, manifest, and run-config surfaces that will
supervise real Myosu work such as chain runtime, miner, validator, gameplay,
and operations.

### Phase 4: Freeze legacy Malinka-first surfaces

Keep any remaining legacy files only as compatibility or historical reference,
not as the primary place where execution behavior is designed.

## Parity Gates

Before Phase 2 is considered complete:

- root `PLANS.md` and `SPEC.md` exist
- `specsarchive/` preserves the previous corpus unchanged
- a new canonical index exists at `specs/031626-00-master-index.md`
- at least one decision spec, one migration spec, and one live ExecPlan exist

Before Phase 3 is considered complete:

- `OS.md` and `AGENTS.md` reference the new doctrine surface as canonical
- stale references to deleted Malinka control files are gone
- current contributors can find the active repo strategy without relying on
  `specsarchive/`

Before Phase 4 is considered complete:

- at least one real Myosu workstream is supervised through Fabro-native
  program truth
- `ralph/IMPLEMENT.md` is compatibility-only rather than the primary planning
  contract

## Non-goals

- rewriting the entire Myosu product corpus into Fabro format in one slice
- changing Myosu's chain, miner, validator, or gameplay architecture as part of
  the cleanup itself
- deleting the old spec corpus
- removing every Malinka-oriented file before a Fabro-native replacement exists

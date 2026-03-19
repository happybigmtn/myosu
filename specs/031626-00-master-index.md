# Specification Index: Myosu Fabro-Aligned Doctrine

Status: Active
Date: 2026-03-18
Type: Orientation / Index
Supersedes: the earlier `specs/` directory as the canonical planning surface

## Purpose / User-Visible Outcome

After the repository cleanup for Fabro adoption, a contributor can start with
this file, `SPEC.md`, and `PLANS.md` and understand where current doctrine
lives. The older spec corpus is still available in `specsarchive/`, but new
durable decisions now live in `specs/` and new implementation slices now live
in `plans/`.

## Canonical Documents

- `SPEC.md` defines how to write durable decision, migration, and capability
  specs in this repository.
- `PLANS.md` defines how to write living executable implementation plans.
- `specs/031826-fabro-primary-executor-decision.md` records the repo-strategy
  decision that makes Fabro the primary execution framework for Myosu.
- `specs/031826-myosu-fabro-primary-executor-migration.md` defines the target
  architecture for the doctrine and execution-surface migration.
- `plans/031826-clean-up-myosu-for-fabro-primary-executor.md` captures the
  cleanup slice that created this structure.
- `plans/031826-bootstrap-fabro-primary-executor-surface.md` defines the next
  slice that retargets the remaining doctrine and operational entrypoints.

## Whole-System Goal

Myosu is still building the same product described in `OS.md`: a
Substrate-based game-solving chain with off-chain miners, off-chain validators,
and gameplay surfaces that let humans and agents interact with solver output.
The first market remains poker, and the longer-term goal remains a multi-game
protocol for imperfect-information games.

The doctrine cleanup does not change that product direction. It changes how the
repository records durable boundaries and bounded implementation slices so
Fabro can become the primary executor without forcing contributors to navigate
an ambiguous mix of legacy planning formats.

## Current Durable Priorities

- Preserve the product mission, stage gates, invariants, and no-ship doctrine
  already recorded in `OS.md`, `INVARIANTS.md`, and `ops/`.
- Make Fabro-style `SPEC.md` + `PLANS.md` the canonical way to drive new work.
- Preserve the pre-Fabro spec corpus as historical context instead of deleting
  it.
- Use the new doctrine surface to prepare follow-on migration of
  `OS.md`, `AGENTS.md`, and remaining `ralph` references away from
  Malinka-era assumptions.
- Resume chain, miner, validator, gameplay, and launch work under the new
  Fabro-aligned planning surface once the repo-level migration is stable.

## Archived Legacy Corpus

`specsarchive/` contains the earlier Myosu design corpus, including duplicate
filenames, greenfield product specs, and Malinka-oriented deployment notes.
That material remains useful reference while the migration is in progress, but
it is not the place to author new canonical direction.

## How To Use This Index

Start with `SPEC.md` if you are choosing a durable boundary. Start with
`PLANS.md` if you are defining or executing the next bounded slice. Update
`specs/` and `plans/` for current work. Update `specsarchive/` only when you
are explicitly preserving or annotating historical material.

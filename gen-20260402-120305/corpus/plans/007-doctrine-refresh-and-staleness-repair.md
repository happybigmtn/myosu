# 007 - Doctrine Refresh and Staleness Repair

## Purpose / Big Picture

Several doctrine documents reference files that may not exist, contain stale
claims, or are empty stubs. This plan audits every doctrine file and repairs
staleness so that a new contributor can trust the documentation.

## Context and Orientation

Documentation staleness from ASSESSMENT.md:
- `OS.md` references root `DESIGN.md` which may not exist
- `INVARIANTS.md` references `ops/` files that may not exist
- `COMPLETED.md` is an empty stub
- `docs/execution-playbooks/` has partially stale content
- Some playbooks reference outdated surfaces

## Architecture

No architectural changes. This is a documentation audit and repair.

## Progress

### Milestone 1: Audit all doctrine file cross-references

- [ ] M1. Verify every file path referenced in doctrine documents actually exists
  - Surfaces: `README.md`, `SPEC.md`, `INVARIANTS.md`, `OS.md`, `AGENTS.md`, `PLANS.md`, `THEORY.MD`
  - What exists after: Every cross-reference in doctrine files points to an
    existing file. Broken references are either fixed or removed.
  - Why now: New contributors will follow broken links and lose trust.
Proof command: `rg -o '\b(ops|genesis|specs|docs|crates)/[a-zA-Z0-9/_.-]+' README.md SPEC.md INVARIANTS.md OS.md AGENTS.md PLANS.md | while IFS=: read -r doc ref; do test -e "$ref" || echo "BROKEN: $doc -> $ref"; done`
  - Tests: No broken references found

### Milestone 2: Fill or remove COMPLETED.md

- [ ] M2. Either populate COMPLETED.md with verified completion records or remove it
  - Surfaces: `COMPLETED.md`
  - What exists after: File either has real content or is deleted.
  - Why now: Empty stub misleads contributors.
Proof command: `test -s COMPLETED.md || ! test -e COMPLETED.md`
  - Tests: File is non-empty or absent

### Milestone 3: Refresh execution playbooks

- [ ] M3. Update playbooks to reflect current crate structure and commands
  - Surfaces: `docs/execution-playbooks/*.md`
  - What exists after: Every command in every playbook actually works when run.
  - Why now: Playbooks are operator-facing. Stale commands waste operator time.
Proof command: Extract commands from playbooks and dry-run them
  - Tests: Manual verification of extracted commands

### Milestone 4: Sync OS.md references

- [ ] M4. Update OS.md to reference only files that exist
  - Surfaces: `OS.md`
  - What exists after: All `DESIGN.md`, `ops/`, and plan references in OS.md
    point to real files.
  - Why now: OS.md is the primary orientation document.
Proof command: `rg -o '\b[a-zA-Z0-9/_.-]+\.(md|toml|yaml|sh)' OS.md | while read ref; do test -e "$ref" || echo "BROKEN: $ref"; done`
  - Tests: No broken references

## Surprises & Discoveries

- The `THEORY.MD` file uses `.MD` extension (uppercase). This is inconsistent
  with all other markdown files but is not breaking.
- Multiple `outputs/*/review.md` files describe surfaces as "not yet built" when
  the code already exists. These should be treated as stale output artifacts,
  not doctrine.

## Decision Log

- Decision: Repair references rather than restructure doctrine hierarchy.
  - Why: The hierarchy (specs > invariants > OS.md > ops) is sound. The problem
    is stale references, not structural.
  - Failure mode: Doctrine structure itself is wrong.
  - Mitigation: Assessment found the structure works. Only references are stale.
  - Reversible: yes

## Validation and Acceptance

1. Zero broken cross-references in doctrine files.
2. `COMPLETED.md` is either populated or removed.
3. All playbook commands are valid.
4. `OS.md` references only existing files.

## Outcomes & Retrospective
_Updated after milestones complete._

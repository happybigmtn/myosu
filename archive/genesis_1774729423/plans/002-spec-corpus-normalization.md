# Normalize Active Spec Corpus and Doctrine Surface

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This plan follows `genesis/PLANS.md`.

## Purpose / Big Picture

The repository currently has conflicting control-plane documents: empty canonical specs, duplicate mirror specs, and legacy assumptions mixed with active doctrine. This plan restores a single trustworthy spec surface so every implementation plan has a stable source.

After this plan, contributors should be able to answer “which file is canonical?” without interpretation.

## Progress

- [x] (2026-03-28 20:32Z) Identified empty canonical files and duplicate legacy mirror specs.
- [ ] Replace empty canonical specs (`031626-07`, `031626-10`) with implementation-ready content.
- [ ] Remove or archive duplicate mirror specs from active `specs/` namespace.
- [ ] Resolve duplicated incentive docs (`031626-11` vs `031626-12`) and filename/content mismatch.
- [ ] Update master index and README pointers so they only reference canonical files.

## Surprises & Discoveries

- Observation: `specs/README.md` claims legacy archive semantics while living in the active `specs/` directory.
  Evidence: `specs/README.md` content versus actual active usage.
- Observation: two incentive docs are byte-identical with different filenames and intent labels.
  Evidence: matching hashes for `specs/031626-11-agent-coordination-mechanism.md` and `specs/031626-12-nlhe-incentive-mechanism.md`.

## Decision Log

- Decision: Keep numbered `031626-*` canonical and move non-numbered mirrors out of active path.
  Rationale: Lowest-risk normalization that preserves provenance.
  Inversion (failure mode): Keeping both active will reintroduce contradictory planning context.
  Date/Author: 2026-03-28 / Genesis

- Decision: Treat `031626-12` as the canonical incentive file and repurpose or retire `031626-11`.
  Rationale: Filename-to-content alignment is stronger for `031626-12`.
  Inversion (failure mode): If both remain active duplicates, downstream plans will diverge on naming and ownership.
  Date/Author: 2026-03-28 / Genesis

## Outcomes & Retrospective

- Pending implementation.

## Context and Orientation

Primary files:
- `specs/031626-00-master-index.md`
- `specs/031626-07-tui-implementation.md`
- `specs/031626-10-agent-experience.md`
- `specs/031626-11-agent-coordination-mechanism.md`
- `specs/031626-12-nlhe-incentive-mechanism.md`
- all non-numbered mirrors in `specs/031626-*.md`

## Milestones

### Milestone 1: Restore missing canonical spec content

Fill `031626-07` and `031626-10` with concrete architecture, scope, and acceptance criteria.

Proof command:

    test -s specs/031626-07-tui-implementation.md
    test -s specs/031626-10-agent-experience.md

### Milestone 2: Eliminate active duplicates

Move or mark non-numbered mirrors as archived and update pointers.

Proof command:

    ls specs/031626-*.md | rg -v '031626-[0-9]{2}.*\.md|031626-00-master-index\.md|README\.md' || true

### Milestone 3: Resolve incentive doc collision

Leave one canonical incentive spec and add explicit supersession note in the other file.

Proof command:

    sha256sum specs/031626-11-agent-coordination-mechanism.md specs/031626-12-nlhe-incentive-mechanism.md
    rg -n 'Superseded|Canonical' specs/031626-11-agent-coordination-mechanism.md specs/031626-12-nlhe-incentive-mechanism.md

### Milestone 4: Rebuild master index references

Update `031626-00` links/dependencies to match the normalized corpus.

Proof command:

    rg -n '031626-07|031626-10|031626-11|031626-12' specs/031626-00-master-index.md

### Milestone 5: Doctrine sanity gate

Run a consistency check that every canonical spec file exists and is non-empty.

Proof command:

    for f in specs/031626-00-master-index.md specs/031626-0{1,2,3,4,5,6,7,8,9}-*.md specs/031626-1{0,1,2,3,4,5,6,7,8,9}-*.md; do [ -s "$f" ] || echo "EMPTY: $f"; done

## Plan of Work

1. Author missing content for empty canonical specs from current implementation realities.
2. Decide canonical file per duplicated topic and add supersession markers.
3. Move duplicate mirrors out of active namespace or mark clearly as archived.
4. Rewire index references and dependency notes.
5. Re-run consistency checks and fix any broken references.

## Concrete Steps

From `/home/r/coding/myosu`:

    rg --files specs | sort
    wc -l specs/*.md | sort -n
    sha256sum specs/*.md | sort

Then apply targeted edits and rerun proof commands per milestone.

## Validation and Acceptance

Accepted when:
- No empty canonical spec remains.
- No active duplicate spec topic remains without explicit supersession notes.
- Master index points only to canonical active files.

## Idempotence and Recovery

- This plan is idempotent if file moves are additive and supersession notes are explicit.
- Recovery path: if a file is wrongly archived, restore from git and re-run index validation.

## Artifacts and Notes

- Output artifact: `outputs/strategy/planning/spec.md` should be updated after normalization.
- Assessment source: `genesis/ASSESSMENT.md`.

## Interfaces and Dependencies

```text
Legacy mirrors + empty canon
        |
        v
Canonical numbered specs only
        |
        v
Master index + plans consume one source of truth
```

Depends on: none.
Blocks: 003, 005, 007, 011.


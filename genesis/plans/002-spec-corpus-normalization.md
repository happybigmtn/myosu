# Normalize Active Spec Corpus and Doctrine Surface

Status: Completed 2026-03-29.

This ExecPlan is a living document. Maintained per `genesis/PLANS.md`.

Provenance: Enhanced from `archive/genesis_1774729423/plans/002-spec-corpus-normalization.md`. Changes: added content-quality gates beyond file-existence checks, added output doc freshness milestones, added explicit file manifest.

## Purpose / Big Picture

The `specs/` directory currently contains 37 files: 20 numbered canonical specs (2 empty), 13 non-numbered mirror copies, 1 index, 1 malinka-enhancements, 1 myosu-game-solving-chain overview, and 1 README. Two canonical specs are 0 bytes. Two others are byte-identical duplicates with different names. Downstream plans reference missing content.

After this plan, every file in `specs/` is either a canonical numbered spec with real content, or archived. The master index (`031626-00`) references only files that exist and are non-empty. Output artifacts in `outputs/` reflect current code reality.

## Progress

- [x] (2026-03-28) Identified empty canonical files, duplicate mirrors, and stale output artifacts.
- [x] (2026-03-29) Restored the empty canonical TUI spec by promoting the live
  mirror content into `specs/031626-07-tui-implementation.md`.
- [x] (2026-03-29) Filled `specs/031626-10-agent-experience.md` with the honest
  Stage 0 agent contract grounded in `OS.md`, `myosu-tui::schema`,
  `myosu-tui::pipe`, and `myosu-play`.
- [x] (2026-03-29) Resolved the `031626-11` / `031626-12` collision by keeping
  `12` as the active NLHE incentive spec and replacing `11` with an explicit
  supersession note plus the narrower future agent-coordination topic.
- [x] (2026-03-29) Archived the remaining 13 non-index mirror/doctrine specs to
  `specsarchive/`, leaving the active `specs/` namespace aligned with the
  master-index canonicals.
- [x] (2026-03-29) Updated `031626-00-master-index.md` to keep the active index
  aligned with the canonical files, including the explicit supersession status
  for `031626-11`.
- [x] (2026-03-29) Refreshed the stale review artifacts in
  `outputs/play/tui/review.md`, `outputs/chain/pallet/review.md`,
  `outputs/chain/runtime/review.md`, and `outputs/games/traits/review.md`.
- [x] (2026-03-29) Ran the consistency gate: every master-index target in
  `specs/` is present and non-empty, there are no extra `031626-*` files left
  in active `specs/`, and the refreshed review docs are present.

## Surprises & Discoveries

- Observation: `specs/031626-tui-implementation.md` (non-numbered) has 21,985 bytes of real content while `specs/031626-07-tui-implementation.md` (numbered canonical) is 0 bytes.
  Evidence: `ls -la specs/031626-tui-implementation.md` vs `specs/031626-07-tui-implementation.md`.
- Observation: `specs/031626-11-agent-coordination-mechanism.md` and `specs/031626-12-nlhe-incentive-mechanism.md` are byte-identical (25,106 bytes each).
  Evidence: `sha256sum` produces identical hashes.
- Observation: the filename heuristic was weaker than the master index. Files
  like `02a/02b/04a/04b` look non-canonical to a simple regex, but the master
  index treats them as real split canonical specs. The honest cleanup rule is
  "archive anything not backed by the master index," not "archive anything that
  fails a filename pattern."
  Evidence: master-index reconciliation on 2026-03-29 after the first archive
  pass temporarily over-moved `02a/02b/04a/04b`.
- Observation: the live Stage 0 agent surface is stronger than the original
  plan assumed. The repo already ships `myosu-play pipe`, the shared
  `GameState` schema, and optional live miner HTTP query enrichment.
  Evidence: `crates/myosu-play/src/main.rs`, `crates/myosu-play/src/live.rs`,
  `crates/myosu-tui/src/schema.rs`, and `crates/myosu-tui/src/pipe.rs`.

## Decision Log

- Decision: Copy content from non-numbered `031626-tui-implementation.md` into empty `031626-07-tui-implementation.md` rather than writing from scratch.
  Rationale: The content already exists in the right directory; it's just in the wrong file.
  Date/Author: 2026-03-28 / Genesis

- Decision: Make `031626-12-nlhe-incentive-mechanism.md` canonical; add supersession header to `031626-11`.
  Rationale: Filename-to-content alignment is stronger for 12 (NLHE incentive mechanism is the actual topic).
  Date/Author: 2026-03-28 / Genesis

- Decision: Archive non-numbered mirrors rather than delete.
  Rationale: `git rm` plus move to `specsarchive/` preserves provenance while cleaning the active namespace.
  Date/Author: 2026-03-28 / Genesis

## Failure Modes

| Codepath | Realistic failure | Handling |
|---|---|---|
| Spec content copy | Content from mirror is stale vs current code | Cross-check against actual crate implementations before committing |
| Index rewiring | Master index references file that was archived | Run proof command (milestone 6) before merge |
| Output refresh | Review claims functionality that no longer exists | Read the actual source files before updating reviews |

## Outcomes & Retrospective

This plan is now complete. The active spec corpus is master-index-clean, the
empty placeholder docs are gone, the fake 11/12 duplication is gone, the
master index points at live non-empty canonicals, and the stale review
artifacts have been rewritten against the current completed-core truth.

## Context and Orientation

The `specs/` directory at `/home/r/coding/myosu/specs/` is the canonical specification surface. `SPEC.md` (root) defines spec types: decision, migration, and capability specs. The master index at `specs/031626-00-master-index.md` maps 20 spec files to AC (Acceptance Criteria) prefixes.

Files requiring action:
- none on the active canonical namespace; follow-on spec work is new doctrine,
  not cleanup debt

## Milestones

### Milestone 1: Restore TUI spec content

Copy the existing content from `specs/031626-tui-implementation.md` (non-numbered, 21,985 bytes) into the empty canonical `specs/031626-07-tui-implementation.md`. Verify content aligns with actual TUI implementation in `crates/myosu-tui/`.

Proof command:

    test -s specs/031626-07-tui-implementation.md && wc -c < specs/031626-07-tui-implementation.md

Expected: file exists and is >20,000 bytes.

### Milestone 2: Restore agent experience spec content

Write implementation-ready content for `specs/031626-10-agent-experience.md` based on the agent transport table in `OS.md` (lines 341-369) and the pipe mode implementation in `crates/myosu-tui/src/pipe.rs`.

Proof command:

    test -s specs/031626-10-agent-experience.md && wc -c < specs/031626-10-agent-experience.md

Expected: file exists and is >5,000 bytes.

### Milestone 3: Resolve incentive doc collision

Add a supersession header to `specs/031626-11-agent-coordination-mechanism.md` pointing to `031626-12`. Differentiate 11's content to cover agent coordination specifically (multi-agent gameplay protocol) rather than duplicating NLHE incentive mechanism.

Proof command:

    head -5 specs/031626-11-agent-coordination-mechanism.md | grep -q "Superseded\|superseded\|See 031626-12"
    ! cmp -s specs/031626-11-agent-coordination-mechanism.md specs/031626-12-nlhe-incentive-mechanism.md

Expected: supersession marker present, files are no longer byte-identical.

### Milestone 4: Archive non-numbered mirrors

Move all non-numbered `specs/031626-*.md` files (those without a two-digit prefix after 031626-) to `specsarchive/`. Preserve `specs/031626-00-master-index.md` and `specs/README.md`.

Proof command:

    ls specs/031626-*.md | grep -v '031626-[0-9][0-9]' | grep -v 'README' | wc -l

Expected: 0 (no non-numbered mirrors remain in active specs/).

### Milestone 5: Update master index

Edit `specs/031626-00-master-index.md` so every referenced file exists and is non-empty. Remove references to archived mirrors.

Proof command:

    grep -oP '031626-\d+-[a-z-]+\.md' specs/031626-00-master-index.md | while read f; do test -s "specs/$f" && echo "OK: $f" || echo "MISSING: $f"; done

Expected: all OK, no MISSING.

### Milestone 6: Refresh stale output artifacts

Update `outputs/play/tui/review.md`, `outputs/chain/pallet/review.md`, `outputs/chain/runtime/review.md`, and `outputs/games/traits/review.md` to reflect current code state. Read the actual source files before rewriting.

Proof command:

    for f in outputs/play/tui/review.md outputs/chain/pallet/review.md outputs/chain/runtime/review.md outputs/games/traits/review.md; do test -s "$f" && echo "OK: $f"; done

## Plan of Work

1. Copy TUI spec content from non-numbered mirror to canonical file.
2. Write agent experience spec from OS.md and pipe.rs.
3. Add supersession marker to 031626-11, differentiate content.
4. Move non-numbered mirrors to specsarchive/.
5. Update master index.
6. Refresh output artifacts by reading current source.

## Concrete Steps

From `/home/r/coding/myosu`:

    # Survey current state
    wc -c specs/031626-*.md | sort -n
    ls specs/031626-*.md | grep -v '031626-[0-9][0-9]' | grep -v README

## Validation and Acceptance

Accepted when:
- No empty canonical spec remains in `specs/`.
- No active duplicate remains without supersession marker.
- Master index references only existing, non-empty files.
- Output reviews match current code.

## Idempotence and Recovery

File moves are additive (to specsarchive/). Content writes are overwrite-safe. If a file is wrongly archived, restore from git.

## Interfaces and Dependencies

Depends on: none.
Blocks: 003, 005, 007, 010, 011.

```text
specs/ (37 files, messy)
        |
        v
specs/ (20 canonical, all non-empty)
specsarchive/ (13 mirrors archived)
outputs/ (reviews refreshed)
```

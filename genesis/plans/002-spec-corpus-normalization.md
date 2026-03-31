# Normalize Active Spec Corpus and Doctrine Surface

Status: Completed locally on 2026-03-30 after the reopened freshness pass
finished syncing the canonical corpus and downstream doctrine surfaces to the
current stage-0 repo truth.

This ExecPlan is a living document. Maintained per `genesis/PLANS.md`.

Provenance: Enhanced from `archive/genesis_1774729423/plans/002-spec-corpus-normalization.md`. Changes: added content-quality gates beyond file-existence checks, added output doc freshness milestones, added explicit file manifest.

## Purpose / Big Picture

The original `002` problem was structural: the active `specs/` namespace had
empty canonical files, duplicate canonicals, and mirror copies that made the
source of truth ambiguous. That part is now fixed. The reopened problem is
freshness: the namespace is clean, but the canonical master index and some
downstream Genesis summaries were still describing pre-implementation reality
after the repo had already moved much further.

After the reopened `002` pass, the canonical spec surface should be both clean
and current. The master index (`031626-00`) should reference only real files
and describe the repo as it exists now, not as it existed before stage-0
execution work landed. Downstream Genesis and doctrine summaries should stop
claiming the already-fixed empty/duplicate-spec problems are still current.

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
- [x] (2026-03-30) Re-audited the active canonical namespace after `010`
  closure and found a second-order drift problem: the empty and duplicate
  files are gone, but `specs/031626-00-master-index.md` still describes a
  greenfield repo and Genesis doctrine docs still claim `031626-07` /
  `031626-10` are empty plus `031626-11` / `031626-12` are duplicates.
- [x] (2026-03-30) Updated the canonical master index to current repo truth:
  local stage-0 loop, shared chain client, miner, validator, live gameplay
  surface, and the additive Liar's Dice proof are now reflected directly in
  `specs/031626-00-master-index.md`.
- [x] (2026-03-30) Synced the stale downstream doctrine surfaces in
  `genesis/plans/001-master-plan.md`, `genesis/GENESIS-REPORT.md`,
  `genesis/ASSESSMENT.md`, `ops/no-ship-ledger.md`, and `THEORY.MD` so the
  repo no longer claims the already-fixed empty/duplicate-spec problems are
  still current.
- [x] (2026-03-30) Audited the next four numbered canonicals with the biggest
  current-state drift (`031626-04a`, `04b`, `05`, and `06`) and rewrote their
  `Current state`, `What already exists`, and ownership sections to the real
  crate/module layout now living under `crates/myosu-miner/`,
  `crates/myosu-validator/`, `crates/myosu-play/`, and
  `crates/myosu-games-liars-dice/`.
- [x] (2026-03-30) Audited the next chain/operator canonicals with the clearest
  pre-implementation drift (`031626-01`, `03`, `08`, `09`, and `18`) and
  rewrote their current-state and ownership sections to the real stage-0 chain,
  pallet, artifact, launch-proof, and RPC surfaces now living under
  `crates/myosu-chain/`, `crates/myosu-games-poker/`, `crates/myosu-miner/`,
  `crates/myosu-validator/`, and `docs/execution-playbooks/`.
- [x] (2026-03-30) Audited the remaining obvious library/UI canonicals with
  stale "missing crate" language (`031626-02a`, `02b`, and `07`) and rewrote
  their current-state and ownership sections to the live `myosu-games`,
  `myosu-games-poker`, and `myosu-tui` module layout.
- [x] (2026-03-30) Added explicit current-truth framing to the later
  future-facing canonicals that still risked reading like silent current
  promises (`031626-12`, `15`, `16`, and `19`), and corrected the obvious
  validator/lobby path drift inside `031626-16`.
- [x] (2026-03-30) Audited the last ambiguous future-facing canonicals
  (`031626-13`, `14`, and `17`) and kept them as design docs while adding the
  current-truth disclaimers and future-surface wording needed to stop them from
  sounding like already-landed stage-0 features.
- [ ] Audit the remaining numbered specs for content freshness beyond the
  master index so the canonical corpus describes current implementation state,
  not just current filenames.

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

The reopened work changes the remaining scope. The namespace itself is clean,
but the canonical corpus still needs freshness work. This slice updated the
master index and downstream Genesis/doctrine summaries to the current repo
state, then refreshed the miner/validator/gameplay/multi-game canonicals that
had the clearest crate-layout drift. The next slice then refreshed the
chain/pallet/abstraction/launch/operator-RPC canonicals to the live local-loop
truth. The remaining work is a narrower pass over whatever numbered contents
still describe clearly older pre-implementation reality. The latest slice
showed that some later-numbered docs were not "wrong implementation docs" so
much as "future design docs missing present-tense disclaimers"; those now say
so explicitly.

At this point reopened `002` is mostly down to editorial judgment rather than
bulk factual cleanup. The active canonicals now broadly separate live stage-0
implementation truth from future design intent instead of blending the two.

## Context and Orientation

The `specs/` directory at `/home/r/coding/myosu/specs/` is the canonical specification surface. `SPEC.md` (root) defines spec types: decision, migration, and capability specs. The master index at `specs/031626-00-master-index.md` maps 20 spec files to AC (Acceptance Criteria) prefixes.

Files requiring action in the reopened freshness slice:
- `specs/031626-00-master-index.md`
- `specs/031626-01-chain-fork-scaffold.md`
- `specs/031626-02a-game-engine-traits.md`
- `specs/031626-02b-poker-engine.md`
- `specs/031626-03-game-solving-pallet.md`
- `specs/031626-04a-miner-binary.md`
- `specs/031626-04b-validator-oracle.md`
- `specs/031626-05-gameplay-cli.md`
- `specs/031626-06-multi-game-architecture.md`
- `specs/031626-07-tui-implementation.md`
- `specs/031626-08-abstraction-pipeline.md`
- `specs/031626-09-launch-integration.md`
- `specs/031626-18-operational-rpcs.md`
- `genesis/plans/001-master-plan.md`
- `genesis/GENESIS-REPORT.md`
- `genesis/ASSESSMENT.md`
- `ops/no-ship-ledger.md`
- `THEORY.MD`

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
- The master index and Genesis/doctrine summaries do not claim already-fixed
  empty/duplicate-spec issues are still open.

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

Revision note (2026-03-30): Reopened after `010` closed because the namespace
cleanup was done but freshness drift remained. The master index and Genesis
doctrine now reflect current repo truth, and the remaining `002` work is a
broader per-spec freshness pass rather than file-structure cleanup.

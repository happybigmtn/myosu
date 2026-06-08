# ADR 015: MCCFR review gate (turn INV-006 advisory enforcement into a blocking gate)

- Status: Accepted
- Date: 2026-06-08
- Deciders: codexworker (NEM-007 row in `nemesis/IMPLEMENTATION_PLAN.md`)
- Consulted: `INVARIANTS.md` INV-006, `docs/adr/003-robopoker-fork.md`, `docs/robopoker-fork-changelog.md`, `ops/decision_log.md`, plan rows NEM-002 / NEM-005 / NEM-007
- Informed: myosu operators, future MCCFR contributors, robopoker fork reviewers
- Related: `.github/workflows/ci.yml` `robopoker-fork-coherence` job, `tests/e2e/mccfr_review_gate.sh`, `INVARIANTS.md` INV-006, `docs/robopoker-fork-changelog.md`

## Context

`INV-006` in `INVARIANTS.md` requires the robopoker fork (`happybigmtn/robopoker`)
to track `v1.0.0` as its proven MCCFR baseline, and to require review of any
core MCCFR algorithm change before it lands in the workspace. The invariant
statement, in the current `INVARIANTS.md`:

> The robopoker fork (`happybigmtn/robopoker`) must track v1.0.0 as its
> baseline. The pinned workspace rev and repo-local fork changelog must
> document every downstream change with rationale. **Core MCCFR algorithm
> changes require review.**

The current enforcement is purely structural:

- The workspace pin is in `crates/myosu-validator/Cargo.toml` and
  `crates/myosu-games-liars-dice/Cargo.toml` (`rev = "0471631..."`, five
  sub-crates: `rbp-cards`, `rbp-gameplay`, `rbp-mccfr`, `rbp-nlhe`,
  `rbp-core`).
- The repo-local fork changelog `docs/robopoker-fork-changelog.md` records
  the diverging commits beyond `v1.0.0` and a one-line functional summary.
- The `robopoker-fork-coherence` CI job in `.github/workflows/ci.yml`
  (line 35) runs `.github/scripts/check_robopoker_fork_status.sh` to
  report the divergence between the pinned rev and the upstream
  default branch.

The CI job runs with `continue-on-error: true` (line 39 of `ci.yml`),
which means a fork-coherence failure does not block the PR. The
`check_robopoker_fork_status.sh` script itself only emits a
`::warning title=Robopoker fork coherence::` annotation on divergence;
it does not classify the diff as MCCFR-relevant, does not require a
reviewer sign-off, and does not fail the job on a regression.

The NEM-002 finding in `nemesis/nemesis-audit.md` originally
classified this as a process gap:

> **Finding NEM-002** ... INV-006: Robopoker Fork Coherence.
> A silent MCCFR correctness change could ship without review.
> Enforcement is advisory (`continue-on-error: true`).

The problem the gate has to solve is not "the fork must never
advance" (the changelog already documents the narrow serde-only
divergence at the current pin) — the problem is "a future commit on
the fork that touches a proven MCCFR correctness surface
(regret update, averaging formula, sampling method) must be loud
and must require review, not silent." The current advisory
enforcement catches this only by accident: a future fork commit
that touches `rbp-mccfr/src/cfr.rs` would show up in the
`check_robopoker_fork_status.sh` `fork commits ahead of baseline`
count and in the changelog divergence summary, but the gate
emits a `::warning` annotation and the CI job continues with a
green check, so the MCCFR-relevant change can land in trunk
without any reviewer interaction at all.

The chosen direction in this ADR is the one the NEM-007 plan row
proposed: a written proposal, a checklist template, a list of
MCCFR-relevant change criteria, and a CI job that proves the
ADR/changelog/INVARIANTS trio stay in sync. The ADR does **not**
flip the existing `robopoker-fork-coherence` job to a blocking
gate yet — that is the "proposed process hardening" half of the
NEM-007 scope boundary, and turning it blocking is a follow-on
that requires operator consensus on the reviewer pool and the
required-change threshold.

If the project did nothing:

- A future robopoker fork commit that changes a regret-update
  formula or the MCCFR averaging rule would land in the
  workspace pin via a normal `cargo update` flow, the
  `check_robopoker_fork_status.sh` script would emit a
  `::warning` annotation, the CI job would stay green (because
  of `continue-on-error: true`), and the change would ship
  without any reviewer seeing the divergence.
- The repo-local `docs/robopoker-fork-changelog.md` would be
  updated only when the next worker happens to refresh the
  `pinned fork rev` line, which is a manual discipline that
  has no automated enforcement.
- INV-006's "require review" clause would remain an
  unaudited aspiration, and a future maintainer reading
  `INVARIANTS.md` would have no concrete signal for what
  "review" means in this context.

## Decision

We land this ADR (`docs/adr/015-mccfr-review-gate.md`) together
with three operational changes that make the proposed blocking
gate reviewable today:

1. **Document the four MCCFR-relevant change criteria** (the
   "what counts as MCCFR" half of the gate). The criteria are
   listed in `## MCCFR-Relevant Change Criteria` below. The
   criteria are the only definition of "MCCFR-relevant" that
   the gate uses, and any future commit that touches one of
   these surfaces must follow the review checklist in
   `## Review Checklist Template` below.
2. **Document the review checklist template** that a reviewer
   must complete when an MCCFR-relevant change lands. The
   checklist is in `## Review Checklist Template` below and is
   meant to be copy-pasted into the PR description when a fork
   commit triggers it.
3. **Add a CI proof harness** `tests/e2e/mccfr_review_gate.sh`
   that asserts the ADR, the `INVARIANTS.md` INV-006
   cross-reference, the `docs/robopoker-fork-changelog.md`
   checklist pointer, the four MCCFR-relevant change criteria,
   and the review checklist template are all present and
   consistent. The harness is the auditable surface for "is the
   gate actually documented," and is wired into a new
   `mccfr-review-gate` CI job in `.github/workflows/ci.yml`.

The CI enforcement of "block on MCCFR-relevant change" is left
as a follow-on that requires operator consensus on the reviewer
pool and the required-change threshold. The reason for keeping
the change advisory for now is documented in
`## Alternatives Considered` below.

### MCCFR-Relevant Change Criteria

A robopoker fork commit counts as "MCCFR-relevant" for the
purposes of INV-006 if it touches any of the following
surfaces. The list is intentionally narrow (it covers only the
proven MCCFR correctness surfaces in the v1.0.0 baseline) and is
the **only** definition the gate uses. A commit that touches
none of these surfaces is not MCCFR-relevant, even if it lives
in the `rbp-mccfr` crate (e.g. a new benchmark harness in
`rbp-mccfr/benches/` that does not change the algorithm is
not MCCFR-relevant).

The four criteria are:

1. **Regret update formula.** Any change to a function or
   inline expression in `rbp-mccfr` whose body computes a
   `regret`, a counterfactual value, or the cumulative-regret
   aggregation (e.g. `update_regrets`, `cfr_step`, the
   `regrets[a] += ...` lines, the `regret_matching_plus`
   averaging rule, the `pruned` regret-mask path). This is
   the single highest-risk surface in the fork — a silent
   rounding change here is a silent policy-quality change
   downstream.
2. **Averaging / sampling formula.** Any change to a function
   in `rbp-mccfr` whose body computes the MCCFR averaging
   weight (e.g. `compute_average`, `linear_cfr_average`,
   `discounted_average`, the `t`/`t+1` weight lookup, the
   sample-weight renormalization in `sample_external_regrets`).
   The averaging rule determines how much weight a given
   iteration's strategy carries in the final policy; a silent
   change here is a silent policy-quality change.
3. **Sampling method.** Any change to the regret-sampling
   branch selection in `rbp-mccfr` (e.g. the
   `sample_action` / `sample_chance` helpers, the
   `sample_outcome` call sites, the regret-matching
   `argmax`/`softmax` selection, the
   explore-with-probability-epsilon branches). A change to
   the sampling method is a change to which counterfactual
   regret is observed next, and a silent change here can
   shift the convergence target.
4. **Public MCCFR-API signature change.** Any change to a
   `pub fn` in the `rbp-mccfr` crate's `lib.rs` or
   `cfr.rs` whose signature a Myosu crate calls into
   (currently `Myosu` calls `Blueprint::regret`,
   `Blueprint::sample`, `Trainable::train` /
   `Trainable::train_chunk`, and a handful of `Regret`
   accessors; a signature change to any of these is a
   public-API change in the `rbp-mccfr` surface that
   `myosu-validator` and `myosu-games-liars-dice` consume).

A commit that touches any of the four surfaces is
MCCFR-relevant and must follow the review checklist below.
A commit that touches none of the four surfaces (e.g. a new
`benches/mccfr_bench.rs`, a docstring-only change in
`rbp-mccfr`, a `Cargo.toml` feature-gate change that does
not alter an MCCFR formula) is **not** MCCFR-relevant and
does not require the review checklist.

The criteria are intentionally enumerated in this ADR so a
future reviewer can grep `MCCFR-Relevant Change Criteria`
straight to the list. A future ADR may extend the criteria
(e.g. to cover a new MCCFR variant), and a future ADR may
narrow them (e.g. if a new wrapper layer isolates a
criterion from a Myosu crate's call site).

### Review Checklist Template

When a robopoker fork commit touches one of the four
MCCFR-relevant surfaces above, the PR author must copy-paste
the following checklist into the PR description and check
every box before the PR is mergeable. The checklist is the
auditable surface for "did a reviewer actually look at this,"
and the `mccfr-review-gate` CI job verifies the template
itself is present in this ADR (not that every PR carries
the checklist — that is a process discipline, not a CI
gate, until the blocking-gate follow-on lands).

```text
MCCFR Review Checklist (NEM-007 / ADR 015)
==========================================

This PR touches one or more MCCFR-relevant surfaces in the
robopoker fork. The reviewer must check every box before
merging.

[ ] 1. Identify which of the four MCCFR-relevant criteria the
       commit touches:
       ( ) Regret update formula
       ( ) Averaging / sampling formula
       ( ) Sampling method
       ( ) Public MCCFR-API signature change

[ ] 2. Cite the upstream `v1.0.0` baseline for the touched
       surface (file path + function name) so a reviewer can
       `git diff v1.0.0..<pinned-rev> -- <path>` and confirm
       the change.

[ ] 3. Summarize the algorithmic change in one paragraph
       (what is the new formula, what is the old formula,
       what is the numerical impact on a known reference
       scenario).

[ ] 4. Cite the proof surface that demonstrates the change
       does not regress the proven baseline. Acceptable
       surfaces:
       - a new `crates/myosu-games-poker` or
         `crates/myosu-games-liars-dice` unit test that
         runs the touched surface against a fixed
         reference and asserts the output is unchanged
         (within the documented INV-003 epsilon)
       - a `docs/robopoker-fork-changelog.md` update
         with a new entry under "Changes Since v1.0.0"
         that names the commit and the proof surface
       - a new `tests/e2e/` harness that exercises the
         touched surface end-to-end and asserts the
         result against a pinned reference

[ ] 5. If the commit changes a public MCCFR-API signature,
       update `crates/myosu-validator/src/score.rs`,
       `crates/myosu-games-liars-dice/src/solver.rs`, and
       any other call site to match the new signature, and
       add a unit test that covers the new signature.

[ ] 6. If the commit is a numerical change to a regret
       update or averaging formula, run
       `cargo test -p myosu-games-poker --quiet
       benchmark::tests::sparse_bootstrap_checkpoint_*`
       and confirm the reference-pack benchmark is still
       within the documented epsilon of the v1.0.0
       reference. Cite the test name and the result
       (mean L1 distance, exact-action-match ratio).

[ ] 7. Update `docs/robopoker-fork-changelog.md` to add
       the new commit under "Changes Since v1.0.0" with
       a one-line summary and a pointer to the proof
       surface from box 4.

[ ] 8. Confirm the workspace pin in
       `crates/myosu-validator/Cargo.toml` and
       `crates/myosu-games-liars-dice/Cargo.toml`
       (the five `rev = "..."` lines under `rbp-cards`,
       `rbp-gameplay`, `rbp-mccfr`, `rbp-nlhe`,
       `rbp-core`) all advance to the new rev in the
       same commit, and that
       `docs/robopoker-fork-changelog.md` "pinned fork
       rev" line matches.

[ ] 9. If the commit is a security-driven pin change
       (per `ops/cve-tracking-process.md`), also update
       the stage-0 security audit snapshot and confirm
       the SEC-001 allowlist still passes
       (`cargo audit -D warnings` exits 0).

[ ] 10. Sign off in the PR description with a one-line
        statement that links this checklist to the
        touched commit SHA. The sign-off is auditable
        from the PR history even if the checklist is
        later edited.
```

A reviewer who checks any box in section 1 of the checklist
without completing the matching sections 2-10 has not
completed the MCCFR review. The CI `mccfr-review-gate` job
verifies the template is present in this ADR; the per-PR
completion discipline is a process contract until the
follow-on blocking-gate ADR lands.

## Alternatives Considered

### Option A: Preferred choice — land this ADR + checklist + proof harness, leave `continue-on-error: true` advisory enforcement in place for now.

Why this option won:

- The four MCCFR-relevant change criteria are the actual
  missing artifact: without a written list of "what counts
  as MCCFR," the future blocking-gate has no enforceable
  boundary, and a reviewer cannot tell whether a given
  commit is in scope.
- The review checklist template is the auditable surface
  for "did a reviewer actually look at this" without
  requiring the operator to staff a reviewer pool today.
- The proof harness `tests/e2e/mccfr_review_gate.sh`
  makes the ADR/checklist/INVARIANTS trio a CI-enforced
  invariant, so the documented policy cannot silently
  drift away from the implementation.
- Flipping the existing `robopoker-fork-coherence` job to
  a blocking gate requires a reviewer pool and a
  required-change threshold that the project does not
  have today. Land that as a follow-on once the operator
  pool is staffed, and the criteria in this ADR will be
  the input to the blocking rule.

### Option B: Rejected alternative — flip `robopoker-fork-coherence` to `continue-on-error: false` immediately in this same change.

Why this option was not chosen:

- The current `check_robopoker_fork_status.sh` script
  reports fork-coherence divergences (changelog mismatch,
  upstream-ahead-of-pinned) as `::warning` annotations.
  Switching to a blocking gate would block every PR that
  touches the workspace pin, including the routine
  security-driven pin changes documented in
  `ops/cve-tracking-process.md`, and the project does not
  have a staffed reviewer pool to unblock them.
- Without the four MCCFR-relevant change criteria
  documented in this ADR, a blocking gate would block
  every divergence (including `Cargo.toml` feature-gate
  changes, docstring-only updates, and benchmark
  additions), which is the wrong failure mode for an
  INV-006 enforcement gate. The criteria have to land
  first, so the gate can be precise.
- The blocking follow-on requires operator consensus on
  the reviewer pool and the required-change threshold,
  neither of which is in scope for this row.

### Option C: Rejected alternative — move the four MCCFR-relevant change criteria into `docs/robopoker-fork-changelog.md` instead of this ADR.

Why this option was not chosen:

- The changelog is the artifact that records what
  changed; the ADR is the artifact that records the
  decision (and the criteria) that decides what counts
  as MCCFR. Mixing the two would mean a future reader
  who updates the changelog (e.g. on a security-driven
  pin change) would have to re-derive the criteria
  from the historical record, instead of grepping
  `docs/adr/015-mccfr-review-gate.md` for the current
  list.
- The criteria need a stable pointer from
  `INVARIANTS.md` INV-006 (this ADR cross-references
  the criteria, and the INV-006 row points to the
  ADR), and a stable pointer from the
  `mccfr-review-gate` CI job. Anchoring the criteria
  in the changelog would make both pointers
  brittle (the changelog is rewritten on every pin
  change).

## Consequences

### Positive

- The four MCCFR-relevant change criteria are the
  single, stable, grep-able definition of "what counts
  as MCCFR" in this repo, replacing the
  unaudited-aspiration reading of `INVARIANTS.md` INV-006
  with a concrete, ADRed list.
- The review checklist template is the auditable surface
  for "did a reviewer look at this" without requiring a
  staffed reviewer pool today.
- The `mccfr-review-gate` CI job and the
  `tests/e2e/mccfr_review_gate.sh` proof harness make
  the ADR/checklist/INVARIANTS trio a CI-enforced
  invariant, so the documented policy cannot silently
  drift away from the implementation.
- The follow-on blocking-gate ADR has a concrete input
  (the four criteria) and a concrete process contract
  (the checklist) to draw on, which removes the biggest
  open question for the follow-on.

### Negative

- The CI enforcement of "block on MCCFR-relevant
  change" is **not** in this ADR; it is a follow-on.
  Until that follow-on lands, the gate is documented
  but advisory. The NEM-007 scope boundary
  intentionally accepts this trade-off.
- The four MCCFR-relevant change criteria are
  enumerated by hand from the v1.0.0 baseline; if a
  future robopoker release adds a new MCCFR variant
  (e.g. a new sampling scheme in a new module), the
  criteria must be extended by a follow-on ADR.
  Maintaining the list is a small but real
  follow-on cost.
- The proof harness
  (`tests/e2e/mccfr_review_gate.sh`) is a
  string-presence gate (it greps the ADR text and
  the `INVARIANTS.md` INV-006 section); it does not
  statically analyze the fork. A future worker who
  wanted a stricter gate would have to build a
  cross-crate grep that compares the v1.0.0 baseline
  to the current pin on the four criterion surfaces.
  That is out of scope for NEM-007.

### Follow-up

- **MCCFR blocking gate follow-on.** A future ADR
  (target slug: `docs/adr/016-mccfr-blocking-gate.md`)
  must define (a) the reviewer pool (a list of GitHub
  handles in `ops/reviewer-roster.md` that can sign
  off on the review checklist), (b) the
  required-change threshold (which subset of the
  four criteria trigger a required review, and
  which subset are advisory), and (c) the CI change
  that flips the `robopoker-fork-coherence` job to
  `continue-on-error: false` once the reviewer pool
  is staffed. This ADR is the input to that
  follow-on, not the gate itself.
- **MCCFR criterion maintenance.** Every robopoker
  release that adds a new MCCFR module or a new
  sampling scheme must extend the criteria list in
  `## MCCFR-Relevant Change Criteria` of this ADR
  (via a follow-on ADR), and the
  `tests/e2e/mccfr_review_gate.sh` proof harness
  must be updated to grep for the new criterion
  phrase.
- **`docs/robopoker-fork-changelog.md` update.** The
  changelog currently ends at the "Remaining
  Obligation" section. This ADR adds a "Review
  Checklist Pointer" link to the checklist template
  in this ADR, so a future worker who updates the
  pin can find the checklist by following the
  changelog link.

## Reversibility

Moderate. The four MCCFR-relevant change criteria are
written in this ADR; a future ADR can replace them
with a different list, but every change must be a
new ADR (the changelog and the proof harness both
reference the criteria by their exact phrase in
this ADR). The review checklist template is also
greppable by phrase in the proof harness, so
changing the template requires updating both the
ADR and the proof harness in the same change. The
proof harness is CI-enforced; flipping the
`robopoker-fork-coherence` job to a blocking gate
(Option B) is a separate, easier-to-reverse
change because the only thing that has to be
reverted is a single `continue-on-error` flag in
`ci.yml`. A proof trigger that would justify
reopening this decision: a future robopoker
release that adds a new MCCFR variant which is
not covered by the four criteria above, and which
the operator pool agrees is too narrow to be
advisory.

## Validation / Evidence

- The new `tests/e2e/mccfr_review_gate.sh` proof
  harness is the executable end-to-end gate. The
  harness asserts:
  - (a) `docs/adr/015-mccfr-review-gate.md` exists
    and is the ADR that documents this decision.
  - (b) `INVARIANTS.md` INV-006 section ends with a
    cross-reference to this ADR.
  - (c) `docs/robopoker-fork-changelog.md` has a
    `Review Checklist Pointer` link to this ADR.
  - (d) All four MCCFR-relevant change criteria
    phrases appear in the ADR (greppable strings:
    `Regret update formula`, `Averaging / sampling
    formula`, `Sampling method`, `Public MCCFR-API
    signature change`).
  - (e) The review checklist template is present
    in the ADR with all ten numbered items.
  - (f) The `mccfr-review-gate` CI job is wired in
    `.github/workflows/ci.yml`.
  - (g) The NEM-007 row is `[x]` in both
    `IMPLEMENTATION_PLAN.md` and
    `nemesis/IMPLEMENTATION_PLAN.md` with a
    cross-reference to this ADR.
- The new `mccfr-review-gate` CI job in
  `.github/workflows/ci.yml` installs no extra
  tooling (the proof harness is pure bash + grep)
  and runs `bash tests/e2e/mccfr_review_gate.sh`
  on every PR and push to `trunk` / `main`.
- The existing `robopoker-fork-coherence` CI job
  continues to run with
  `continue-on-error: true` (this ADR explicitly
  does not flip it to blocking; see
  `## Alternatives Considered` Option B).
- The repo-local
  `docs/robopoker-fork-changelog.md` gains a
  one-line "Review Checklist Pointer" link to this
  ADR; no other change to the changelog.

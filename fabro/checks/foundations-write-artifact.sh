#!/usr/bin/env bash
set -euo pipefail

artifact="${1:?usage: foundations-write-artifact.sh <foundation-plan|review>}"
out_dir="outputs/foundations"
mkdir -p "$out_dir"

case "$artifact" in
  foundation-plan)
    cat > "$out_dir/foundation-plan.md" <<'EOF'
# Bootstrap Foundations Frontier and Revalidate Raspberry Truth

This ExecPlan is a living document. The sections `Progress`,
`Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must
be kept up to date as work proceeds.

`PLANS.md` is checked into the repository root and this document must be
maintained in accordance with it. This plan depends on `README.md`,
`SPEC.md`, `PLANS.md`, `AGENTS.md`,
`specs/031626-00-master-index.md`, and
`specs/031826-fabro-primary-executor-decision.md`.

## Purpose / Big Picture

After this slice lands, Myosu has a dedicated foundations frontier that captures
the current Fabro/Raspberry control-plane truth in durable artifacts and proves
the local `execute/status/watch` path against a small, isolated lane instead of
widening the existing bootstrap programs. A contributor should be able to read
these two artifacts, inspect the historical `games:multi-game` false-submit,
run the foundations lane, and understand whether current detached execution is
trustworthy enough for the next frontier expansion.

## Progress

- [x] (2026-03-21 02:55Z) Read `README.md`, `SPEC.md`, `PLANS.md`, `AGENTS.md`,
  `specs/031626-00-master-index.md`, and
  `specs/031826-fabro-primary-executor-decision.md`.
- [x] (2026-03-21 02:55Z) Re-read the active Fabro/Raspberry control-plane
  surfaces under `fabro/programs/`, `fabro/run-configs/`, `fabro/workflows/`,
  and the existing reviewed `outputs/` artifacts.
- [x] (2026-03-21 02:55Z) Verified that the current `myosu-platform`,
  `myosu-product`, and `myosu-recurring` programs render truthful `plan` and
  `status` views from current artifact state rather than from the stale
  historical `games:multi-game` false-submit.
- [x] (2026-03-21 02:55Z) Inspected the historical `games:multi-game`
  false-submit run directory and confirmed the raw detached-run failure still
  exists on disk as evidence.
- [x] (2026-03-21 02:55Z) Added an isolated `myosu-foundations` program plus a
  small script-backed workflow so Myosu can probe current detached execution
  truth without depending on external LLM networking.
- [ ] Execute `raspberry execute --manifest fabro/programs/myosu-foundations.yaml`
  and record the resulting run-truth evidence in this plan and in
  `outputs/foundations/review.md`.
- [ ] Decide whether the foundations probe is strong enough to close the
  current false-submit concern or whether Fabro/Raspberry still needs another
  hardening slice in the sibling repository.

## Surprises & Discoveries

- Observation: the repo-local command examples for Raspberry were stale.
  Evidence: `cargo --manifest-path /home/r/coding/fabro/Cargo.toml run ...`
  failed in this environment, while the already-built
  `/home/r/coding/fabro/target-local/debug/raspberry` binary worked.

- Observation: the historical `games:multi-game` false-submit is still present
  as raw Fabro evidence, but it is not currently poisoning refreshed
  `raspberry status` output for `myosu-platform`.
  Evidence: the run directory
  `~/.fabro/runs/20260319-01KM2BS4ASVRXVT2ND1GVVMKJ0/` still contains
  `status.json` with `submitted` plus a `detach.log` CLI parse failure, while
  `raspberry status --manifest fabro/programs/myosu-platform.yaml` reports
  `complete=3 ready=0 running=0 blocked=0 failed=0`.

- Observation: `.raspberry/` state is rebuilt on demand from the checked-in
  program manifests and current artifact truth.
  Evidence: this worktree started without `.raspberry/`, and the first
  `raspberry status` call recreated `.raspberry/myosu-platform-state.json`.

## Decision Log

- Decision: keep the foundations probe isolated in `myosu-foundations.yaml`
  instead of widening `myosu-bootstrap.yaml`.
  Rationale: `AGENTS.md` explicitly asks the bootstrap manifest to stay narrow.
  A separate program gives us a truthful canary lane without perturbing the
  trusted bootstrap frontier.
  Date/Author: 2026-03-21 / Codex

- Decision: make the foundations workflow script-backed rather than LLM-backed.
  Rationale: this slice is about control-plane truth, not model quality. A
  script-backed lane removes network/provider variance so a detached-run failure
  points at Fabro/Raspberry plumbing, not at external API access.
  Date/Author: 2026-03-21 / Codex

- Decision: treat the historical `games:multi-game` false-submit as durable
  evidence that still needs review, but not as current proof that status/watch
  are wrong today.
  Rationale: current `plan`, `status`, and `watch` surfaces render from current
  artifact state and do not surface the stale submitted run.
  Date/Author: 2026-03-21 / Codex

## Outcomes & Retrospective

This slice is complete only when the dedicated foundations lane has been run
through Raspberry and the resulting artifacts capture both the historical
failure evidence and the current live execution truth. Until that happens, the
new foundations surface is only staged, not proven.

## Context and Orientation

The active control-plane package lives under `fabro/`. The relevant files for
this slice are:

- `fabro/programs/myosu-foundations.yaml` — isolated Raspberry program for this
  frontier
- `fabro/run-configs/bootstrap/foundations.toml` — Fabro run config describing
  the scope and required artifacts
- `fabro/workflows/bootstrap/foundations.fabro` — small workflow used as the
  live execute/status/watch canary
- `fabro/checks/foundations-write-artifact.sh` — script that writes the durable
  foundations artifacts
- `fabro/checks/foundations-verify.sh` — script that verifies those artifacts
  exist and contain the expected headings
- `outputs/foundations/foundation-plan.md` and `outputs/foundations/review.md`
  — the durable outputs owned by this frontier

The historical run evidence lives outside the repo under the user's Fabro run
store:

- `~/.fabro/runs/20260319-01KM2BS4ASVRXVT2ND1GVVMKJ0/` — stale detached
  `games:multi-game` false-submit
- `~/.fabro/runs/20260319-01KM2CGPHAJ95J38TQ7SPN46NZ/` — direct foreground
  `games:multi-game` run with normal manifest/run files

In Raspberry terms, a **program** is a supervised frontier, a **lane** is one
dispatchable stream inside that frontier, an **artifact** is a durable file
under `outputs/`, and a **milestone** is a named set of artifacts that proves
progress.

## Plan of Work

First, keep the foundations frontier small and explicit: one program, one lane,
two durable artifacts. The new lane should not change ownership of any
existing outputs and should not widen `myosu-bootstrap.yaml`.

Next, use the local built Raspberry binary to render `plan`, `status`, and
`watch` over the current Myosu programs so the plan reflects actual observed
truth, not stale assumptions. Capture whether the historical false-submit still
affects current observer surfaces.

Then, dispatch `myosu-foundations.yaml` through `raspberry execute` and inspect
the resulting Fabro run directory. The acceptance signal is a normal detached
run with the usual metadata files (`manifest.json`, `run.toml`, `run.pid`,
`status.json`, and progress events), not another submitted-only ghost run.

Finally, tighten both durable artifacts so they record the observed live
behavior, the remaining historical cleanup gap, and the next smallest honest
actions for Myosu or the sibling Fabro repository.

## Concrete Steps

From the repository root, run:

    /home/r/coding/fabro/target-local/debug/raspberry plan --manifest fabro/programs/myosu-platform.yaml
    /home/r/coding/fabro/target-local/debug/raspberry status --manifest fabro/programs/myosu-platform.yaml
    timeout 3 /home/r/coding/fabro/target-local/debug/raspberry watch --manifest fabro/programs/myosu-platform.yaml --interval-ms 500
    /home/r/coding/fabro/target-local/debug/raspberry plan --manifest fabro/programs/myosu-foundations.yaml
    /home/r/coding/fabro/target-local/debug/raspberry execute --manifest fabro/programs/myosu-foundations.yaml
    /home/r/coding/fabro/target-local/debug/raspberry status --manifest fabro/programs/myosu-foundations.yaml

For the historical false-submit evidence, inspect:

    sed -n '1,80p' ~/.fabro/runs/20260319-01KM2BS4ASVRXVT2ND1GVVMKJ0/status.json
    sed -n '1,80p' ~/.fabro/runs/20260319-01KM2BS4ASVRXVT2ND1GVVMKJ0/detach.log

## Validation and Acceptance

This slice is accepted when all of the following are true:

- `outputs/foundations/foundation-plan.md` and `outputs/foundations/review.md`
  both exist and describe the same frontier truth.
- `raspberry plan/status/watch` on `myosu-platform.yaml` reflect the current
  completed platform artifacts and do not regress to the stale detached run.
- `raspberry execute --manifest fabro/programs/myosu-foundations.yaml`
  produces a real detached Fabro run with normal metadata files, not a
  submitted-only directory.
- The review artifact clearly says whether the historical `games:multi-game`
  false-submit still requires upstream cleanup even if current observer
  surfaces are healthy.

## Idempotence and Recovery

The foundations workflow is safe to rerun. It only writes the two files under
`outputs/foundations/`. If the lane needs to be rerun from a clean control
state, use a disposable checkout and remove only the foundations output files
plus `.raspberry/myosu-foundations-state.json` before dispatching again.

Do not mutate historical Fabro run directories unless the sibling Fabro repo
has an explicit cleanup or tombstone flow. Those directories are evidence for
this slice.

## Artifacts and Notes

Expected current observer transcript:

    Program: myosu-platform
    Counts: complete=3 ready=0 running=0 blocked=0 failed=0

Historical false-submit evidence:

    {
      "status": "submitted",
      "updated_at": "2026-03-19T06:16:00.090006038Z"
    }

    error: unexpected argument '/home/r/coding/myosu/fabro/programs/../run-configs/platform/multi-game.toml' found

## Interfaces and Dependencies

The foundations frontier ends with these checked-in interfaces:

- `fabro/programs/myosu-foundations.yaml`
- `fabro/run-configs/bootstrap/foundations.toml`
- `fabro/workflows/bootstrap/foundations.fabro`
- `fabro/checks/foundations-write-artifact.sh`
- `fabro/checks/foundations-verify.sh`

No production crate APIs change in this slice. The only runtime dependency is
the local built Raspberry/Fabro pair in `/home/r/coding/fabro/target-local/debug/`.
EOF
    ;;
  review)
    cat > "$out_dir/review.md" <<'EOF'
# Foundations Frontier Review

**Lane:** `foundations:foundations`
**Kind:** platform
**Owning program:** `myosu-foundations`
**Output root:** `outputs/foundations/`
**Date:** 2026-03-21

## Keep / Reopen / Reset Judgment

**Judgment: KEEP WITH A REOPENED HISTORICAL CLEANUP GAP**

Keep the current Fabro/Raspberry observer path. On 2026-03-21, fresh
`raspberry plan`, `status`, and `watch` calls over the checked-in Myosu
programs report the current artifact-backed truth and do not surface the stale
`games:multi-game` false-submit as a running or blocked lane.

Reopen the historical cleanup question, not the whole control plane. The raw
false-submit evidence still exists under `~/.fabro/runs/` as a submitted-only
detached run with no manifest or pid. That evidence should stay visible in this
review until there is a clear upstream cleanup or tombstone policy.

No broad reset is warranted today. The existing bootstrap, platform, product,
and recurring programs are all currently rendering coherent status from the
checked-in artifacts in this worktree.

## What Is Trustworthy Enough To Keep

### Current Raspberry Program Truth

- `raspberry plan --manifest fabro/programs/myosu-platform.yaml` reports all
  three platform lanes complete.
- `raspberry status --manifest fabro/programs/myosu-platform.yaml` reports
  `complete=3 ready=0 running=0 blocked=0 failed=0`.
- `timeout 3 raspberry watch ...` repeats the same completed state rather than
  inventing a stale running or submitted lane.

These are the surfaces that matter for the current frontier task because the
problem statement is about whether `execute/status/watch` truth is trustworthy
again.

### Current Artifact Backing

- `outputs/games/multi-game/spec.md`
- `outputs/games/multi-game/review.md`
- `outputs/games/poker-engine/spec.md`
- `outputs/games/poker-engine/review.md`
- `outputs/sdk/core/spec.md`
- `outputs/sdk/core/review.md`

Those artifacts are present and sufficient for Raspberry to mark the platform
frontier complete today.

### Isolated Foundations Canary

The new `myosu-foundations` program is the right kind of proving lane for this
moment: it is isolated, script-backed, and artifact-oriented. It tests detached
execution plumbing without conflating Fabro control-plane health with external
LLM/provider availability.

## What Must Be Reopened

### Historical False-Submit Cleanup Policy

The old detached `games:multi-game` run still exists on disk:

- `~/.fabro/runs/20260319-01KM2BS4ASVRXVT2ND1GVVMKJ0/status.json`
- `~/.fabro/runs/20260319-01KM2BS4ASVRXVT2ND1GVVMKJ0/detach.log`

Evidence captured from those files:

    {
      "status": "submitted",
      "updated_at": "2026-03-19T06:16:00.090006038Z"
    }

    error: unexpected argument '/home/r/coding/myosu/fabro/programs/../run-configs/platform/multi-game.toml' found

That historical evidence no longer appears to poison current Myosu status
surfaces, but it is still a real orphaned run record. The reopened question is
not "does Myosu need a reset?" The reopened question is "what is the official
cleanup/tombstone story for stale submitted-only detached runs in Fabro?"

### Local Operator Command Surface

The old repo-local command examples used a stale cargo invocation shape.
Those examples were corrected in `README.md` and `fabro/README.md` to point at
the verified local Raspberry binary.

## What Should Be Reset

Nothing in the checked-in Myosu programs needs a broad reset right now.

Do not reset:

- `myosu-bootstrap`
- `myosu-platform`
- `myosu-product`
- `myosu-recurring`

The historical false-submit evidence is not itself a reason to throw away the
current artifact-backed state of those frontiers.

## Immediate Next Actions

1. Run `raspberry execute --manifest fabro/programs/myosu-foundations.yaml`
   and confirm the resulting detached run has normal Fabro metadata files.
2. Re-check `raspberry status --manifest fabro/programs/myosu-foundations.yaml`
   and record whether the lane cleanly transitions to `complete`.
3. If the foundations run detaches cleanly, treat the current false-submit as a
   historical cleanup item rather than as a current blocker for Myosu control
   truth.
4. If the foundations run reproduces a submitted-only ghost run, stop and fix
   the sibling Fabro/Raspberry repo before dispatching more Myosu frontiers.

## Readiness Verdict

**Ready for a live execute probe now.**

The foundation lane is intentionally small enough that a failure will be
diagnostic. If it succeeds, Myosu has a fresh truthful detached-run proof in
this worktree. If it fails, the failure is narrow enough to carry back into the
sibling Fabro repo without ambiguity about whether the issue belonged to a
particular product lane.
EOF
    ;;
  *)
    echo "unknown artifact: $artifact" >&2
    exit 1
    ;;
esac

# Give the detached run enough lifetime for status/watch probes to see progress.
sleep 2

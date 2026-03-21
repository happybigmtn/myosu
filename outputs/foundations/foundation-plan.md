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
the current Fabro/Raspberry control-plane truth in durable artifacts and turns
the frontier’s “fix defects only when real execution finds them” rule into a
reproducible lane. A contributor can inspect the historical `games:multi-game`
false-submit, compare it with current `plan/status/watch`, and rerun the small
foundations probe without widening the trusted bootstrap manifest.

The user-visible outcome for this slice is not “everything is fixed.” The
user-visible outcome is sharper than that: current observer surfaces are mostly
truthful again, and the remaining defects are now pinned to concrete reproducible
execution facts instead of vague suspicion.

## Progress

- [x] (2026-03-21 02:55Z) Read `README.md`, `SPEC.md`, `PLANS.md`,
  `AGENTS.md`, `specs/031626-00-master-index.md`, and
  `specs/031826-fabro-primary-executor-decision.md`.
- [x] (2026-03-21 02:55Z) Re-read the active Fabro/Raspberry package under
  `fabro/programs/`, `fabro/run-configs/`, `fabro/workflows/`, and the
  existing reviewed `outputs/` artifacts.
- [x] (2026-03-21 02:55Z) Verified that current `raspberry plan`, `status`,
  and `watch` on `myosu-platform`, `myosu-product`, and `myosu-recurring`
  report the current artifact-backed truth and do not surface the stale
  historical `games:multi-game` false-submit as active work.
- [x] (2026-03-21 02:55Z) Inspected the historical `games:multi-game`
  false-submit under
  `~/.fabro/runs/20260319-01KM2BS4ASVRXVT2ND1GVVMKJ0/` and confirmed it still
  exists as raw evidence (`status = submitted`, detach CLI parse failure).
- [x] (2026-03-21 02:58Z) Fixed stale operator examples in `README.md` and
  `fabro/README.md`; the old cargo invocation no longer matches the current
  toolchain and local workspace permissions.
- [x] (2026-03-21 02:58Z) Added an isolated `myosu-foundations` program plus a
  script-backed workflow, a repo-local Fabro wrapper, and a dedicated
  foundations output root.
- [x] (2026-03-21 02:59Z) Ran `raspberry execute` against the new foundations
  program with a relative manifest path and observed a real launch defect:
  Raspberry could not spawn a usable Fabro run from that path shape.
- [x] (2026-03-21 03:01Z) Proved the local wrapper can launch a detached Fabro
  run when given an absolute run-config path, a repo-local `--run-dir`, and a
  writable `FABRO_LOG_DIR`.
- [x] (2026-03-21 03:06Z) Re-ran `raspberry execute` with an absolute manifest
  path, explicit lane selection, and the repo-local Fabro wrapper. Raspberry
  truthfully reported a submitted run:
  `01KM75Q2QBHW7XN46707XJJ00M`.
- [x] (2026-03-21 03:06Z) Verified that `raspberry status` and `watch`
  truthfully showed the foundations lane as `running` at stage `Plan` with the
  correct Fabro run id.
- [x] (2026-03-21 03:06Z) Observed the detached worker disappear before
  completing `Plan`; `raspberry status` now truthfully reports the lane as
  failed with `failure_kind = transient_launch_failure` and
  `error = tracked run remained active after its worker process disappeared`.
- [x] (2026-03-21 03:08Z) Wrote the required durable artifacts manually from
  the observed repo state because the failed foundations run never reached the
  file-writing stages.
- [ ] Fix the sibling Fabro/Raspberry defects that this slice isolated:
  relative-manifest path handling for execute, and the disappearing detached
  worker in the repo-local run-dir path.

## Surprises & Discoveries

- Observation: the repo-local Raspberry command examples were stale.
  Evidence: `cargo --manifest-path /home/r/coding/fabro/Cargo.toml run ...`
  failed in this environment, while the already-built
  `/home/r/coding/fabro/target-local/debug/raspberry` binary worked.

- Observation: the historical `games:multi-game` false-submit is still present
  as raw Fabro evidence, but it is not currently poisoning refreshed
  `raspberry status` output for `myosu-platform`.
  Evidence: `~/.fabro/runs/20260319-01KM2BS4ASVRXVT2ND1GVVMKJ0/status.json`
  still says `submitted`, while
  `raspberry status --manifest fabro/programs/myosu-platform.yaml` reports
  `complete=3 ready=0 running=0 blocked=0 failed=0`.

- Observation: invoking `raspberry execute` with a relative manifest path still
  leaves execute on shaky ground in this repo.
  Evidence: the first foundations execute probe failed before a usable run was
  created; separate direct wrapper testing showed that a relative run-config
  path plus `directory = "../../.."` collapses to an empty work dir inside the
  current Fabro path resolver (`Failed to set working directory to : No such
  file or directory (os error 2)`).

- Observation: the installed `~/.fabro/bin/fabro` binary is not a workable
  execution target inside this sandbox when it writes to its default global log
  and run locations.
  Evidence: direct launch attempts hit read-only filesystem errors until the
  wrapper moved `FABRO_LOG_DIR` and `--run-dir` into repo-local writable paths.

- Observation: with an absolute manifest path and the repo-local wrapper,
  `raspberry execute` is capable of truthful submission and truthful failure
  reporting again.
  Evidence: `raspberry execute --lane foundations:foundations ...` returned
  `run_id=01KM75Q2QBHW7XN46707XJJ00M exit_status=0`, `raspberry status` then
  reported `running` at stage `Plan`, and later transitioned to `failed` with
  `tracked run remained active after its worker process disappeared`.

## Decision Log

- Decision: keep the foundations probe isolated in `myosu-foundations.yaml`
  instead of widening `myosu-bootstrap.yaml`.
  Rationale: `AGENTS.md` explicitly asks the bootstrap manifest to stay narrow.
  A separate program gives us a truthful canary lane without perturbing the
  trusted bootstrap frontier.
  Date/Author: 2026-03-21 / Codex

- Decision: make the foundations workflow script-backed rather than LLM-backed.
  Rationale: this slice is about control-plane truth, not model quality. A
  script-backed lane removes external model/provider variance so a failure
  points at Fabro/Raspberry plumbing.
  Date/Author: 2026-03-21 / Codex

- Decision: use a workspace-local Fabro wrapper for the execute probe.
  Rationale: the current sandbox does not allow detached runs to write into the
  default global Fabro locations. The wrapper injects a repo-local `--run-dir`
  and `FABRO_LOG_DIR`, which keeps the probe inside writable paths while
  preserving Fabro CLI behavior.
  Date/Author: 2026-03-21 / Codex

- Decision: invoke the foundations execute probe with an absolute manifest path.
  Rationale: current relative-manifest execution still interacts badly with the
  run-config path resolver and `directory = "../../.."`.
  Date/Author: 2026-03-21 / Codex

- Decision: treat “truthful failed run” as sufficient for the first honest
  reviewed slice.
  Rationale: the frontier task explicitly allows a truthful failure as an
  acceptable conversion target. The key outcome is that current status/watch
  now describe the failure instead of silently hiding it behind a poisoned
  submitted state.
  Date/Author: 2026-03-21 / Codex

## Outcomes & Retrospective

This slice established four durable truths.

First, current observer surfaces over the existing Myosu frontiers are healthier
than the historical `games:multi-game` false-submit suggests. `myosu-platform`,
`myosu-product`, and `myosu-recurring` all render coherent current state from
checked-in artifacts.

Second, the historical false-submit still matters, but now as evidence rather
than as live program truth. It remains on disk and should still inform upstream
cleanup work.

Third, `raspberry execute` is not “fully fixed” in this environment, but it is
no longer a black box. We now have a reproducible path that reaches truthful
submission, truthful running state, and truthful failure state.

Fourth, the remaining blockers are sharp enough to send upstream:
relative-manifest execution path handling and detached worker disappearance
during the foundations `Plan` stage.

## Context and Orientation

The active control-plane package lives under `fabro/`. The files added for this
slice are:

- `fabro/programs/myosu-foundations.yaml`
- `fabro/run-configs/bootstrap/foundations.toml`
- `fabro/workflows/bootstrap/foundations.fabro`
- `fabro/checks/foundations-write-artifact.sh`
- `fabro/checks/foundations-verify.sh`
- `fabro/checks/fabro-local-dispatch.sh`

The durable outputs owned by the frontier are:

- `outputs/foundations/foundation-plan.md`
- `outputs/foundations/review.md`

The repo-local detached run evidence for the execute probe lives at:

- `.raspberry/runs/foundations/`

The historical stale `games:multi-game` evidence lives at:

- `~/.fabro/runs/20260319-01KM2BS4ASVRXVT2ND1GVVMKJ0/`

In Raspberry terms, a **program** is a supervised frontier, a **lane** is one
dispatchable stream inside that frontier, an **artifact** is a durable file
under `outputs/`, and a **milestone** is a named set of artifacts that proves
progress.

## Plan of Work

Keep the foundations frontier intentionally small: one program, one lane, two
artifacts, and one reproducible execute probe. Do not widen the existing
bootstrap frontier.

Use the foundations program to separate three questions that were previously
mixed together:

1. Are current `plan/status/watch` surfaces generally truthful on the checked-in
   Myosu programs?
2. Does the historical `games:multi-game` false-submit still poison current
   control-plane state?
3. Can current `raspberry execute` produce a truthful detached run state in
   this worktree?

The answer after this slice is:

1. yes, mostly
2. no, not on refreshed current state
3. partially: it can submit and report truth, but the detached worker still
   disappears during execution

The next work should therefore go upstream into Fabro/Raspberry rather than
into more Myosu-local frontier expansion.

## Concrete Steps

From the repository root, the key commands for this slice were:

    /home/r/coding/fabro/target-local/debug/raspberry plan --manifest fabro/programs/myosu-platform.yaml
    /home/r/coding/fabro/target-local/debug/raspberry status --manifest fabro/programs/myosu-platform.yaml
    timeout 3 /home/r/coding/fabro/target-local/debug/raspberry watch --manifest fabro/programs/myosu-platform.yaml --interval-ms 500

    /home/r/coding/fabro/target-local/debug/raspberry plan --manifest fabro/programs/myosu-foundations.yaml

    /home/r/coding/fabro/target-local/debug/raspberry execute --manifest /home/r/.fabro/runs/20260320-01KM74SBWAM4KBQHZJYKJ5AEZX/worktree/fabro/programs/myosu-foundations.yaml --lane foundations:foundations --fabro-bin /home/r/.fabro/runs/20260320-01KM74SBWAM4KBQHZJYKJ5AEZX/worktree/fabro/checks/fabro-local-dispatch.sh

    /home/r/coding/fabro/target-local/debug/raspberry status --manifest /home/r/.fabro/runs/20260320-01KM74SBWAM4KBQHZJYKJ5AEZX/worktree/fabro/programs/myosu-foundations.yaml
    /home/r/coding/fabro/target-local/debug/raspberry watch --manifest /home/r/.fabro/runs/20260320-01KM74SBWAM4KBQHZJYKJ5AEZX/worktree/fabro/programs/myosu-foundations.yaml --interval-ms 500

For the historical false-submit evidence:

    sed -n '1,80p' ~/.fabro/runs/20260319-01KM2BS4ASVRXVT2ND1GVVMKJ0/status.json
    sed -n '1,80p' ~/.fabro/runs/20260319-01KM2BS4ASVRXVT2ND1GVVMKJ0/detach.log

## Validation and Acceptance

This first honest slice is accepted when all of the following are true:

- `outputs/foundations/foundation-plan.md` and `outputs/foundations/review.md`
  both exist and describe the same frontier truth.
- current `raspberry plan/status/watch` over existing Myosu frontiers are shown
  to be artifact-backed and not polluted by the historical stale multi-game run.
- the foundations execute probe reaches a truthful live state transition, even
  if that transition ends in failure rather than success.
- the remaining defects are sharp enough that a future Fabro/Raspberry fix
  slice can start from these artifacts alone.

That bar is met here: the execute probe reached truthful `submitted`, truthful
`running`, and truthful `failed` states, and the remaining defects are named.

## Idempotence and Recovery

The foundations probe is safe to rerun. It only touches:

- `outputs/foundations/`
- `.raspberry/myosu-foundations-state.json`
- `.raspberry/runs/foundations/`

Do not mutate the historical `~/.fabro/runs/20260319-01KM2BS4ASVRXVT2ND1GVVMKJ0/`
evidence unless the sibling Fabro repo grows an explicit cleanup or tombstone
flow. That directory is part of the frontier proof.

## Artifacts and Notes

Historical false-submit evidence:

    {
      "status": "submitted",
      "updated_at": "2026-03-19T06:16:00.090006038Z"
    }

    error: unexpected argument '/home/r/coding/myosu/fabro/programs/../run-configs/platform/multi-game.toml' found

Truthful foundations execute submission:

    Program: myosu-foundations
    Dispatch parallelism: 1
    foundations:foundations [submitted] run_id=01KM75Q2QBHW7XN46707XJJ00M exit_status=0

Truthful foundations failed status:

    foundations:foundations [failed|platform] failure_kind=transient_launch_failure
      error: tracked run remained active after its worker process disappeared

## Interfaces and Dependencies

The foundations frontier ends with these checked-in interfaces:

- `fabro/programs/myosu-foundations.yaml`
- `fabro/run-configs/bootstrap/foundations.toml`
- `fabro/workflows/bootstrap/foundations.fabro`
- `fabro/checks/foundations-write-artifact.sh`
- `fabro/checks/foundations-verify.sh`
- `fabro/checks/fabro-local-dispatch.sh`

No production crate APIs change in this slice. The only external dependency is
the local sibling Fabro checkout at `/home/r/coding/fabro/`, and the remaining
execution defects now clearly belong there rather than in the Myosu repo.

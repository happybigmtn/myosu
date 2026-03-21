# Foundations: Honest Execution Truth for the Current Frontier

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

`PLANS.md` is checked into the repository root and this document must be maintained in accordance with it.

## Purpose / Big Picture

The purpose of this slice is not to invent another abstract frontier map. It is to establish the first honest foundations review for the current Myosu frontier by checking what Raspberry and Fabro actually do when asked to supervise real Myosu work. After this slice, a contributor should be able to see which parts of the current frontier are trustworthy, which ones are only artifact-deep, and which upstream Fabro/Raspberry defects must be fixed before more Myosu control-plane claims are treated as real.

The smallest user-visible outcome is a documented, reproducible truth ladder for the current `games:multi-game` frontier:

- the current `raspberry execute/status/watch` behavior as observed on March 21, 2026 UTC
- the current detached Fabro failure modes
- the current foreground Fabro behavior proving the lane itself can start
- the exact next upstream fixes required before Raspberry can be called trustworthy again

## Progress

- [x] (2026-03-21 03:26Z) Read `README.md`, `SPEC.md`, `PLANS.md`, `AGENTS.md`, `specs/031626-00-master-index.md`, and `specs/031826-fabro-primary-executor-decision.md`.
- [x] (2026-03-21 03:27Z) Audited the live Myosu execution surfaces under `fabro/programs/`, `fabro/run-configs/`, `fabro/workflows/`, `outputs/`, and the current `plans/` frontier plans.
- [x] (2026-03-21 03:28Z) Refreshed Raspberry state files by running `raspberry status` against the active manifests and recorded that the CLI timed out without rendering output even though `.raspberry/*.json` was rewritten.
- [x] (2026-03-21 03:31Z) Reproduced the first detached Fabro failure with `fabro run --detach fabro/run-configs/platform/multi-game.toml`, which failed immediately with `Read-only file system (os error 30)`.
- [x] (2026-03-21 03:32Z) Reproduced a truthful detached failure by rerunning `games:multi-game` with an explicit writable run dir and the relative run-config path; Fabro wrote `status=failed` and the detach log showed `Failed to set working directory to : No such file or directory`.
- [x] (2026-03-21 03:34Z) Reproduced a second detached truth bug by rerunning `games:multi-game` with an explicit writable run dir and an absolute run-config path; Fabro emitted run `01KM77ANMNBBQBECZFYK2Y2FNJ`, then the worker died while the run remained frozen at `starting/sandbox_initializing`.
- [x] (2026-03-21 03:37Z) Reproduced a live foreground run of the same lane with an absolute run-config path and writable run dir; run `01KM77GNB586TZVC2AEKF1TRXE` reached stage `Specify` and wrote real node artifacts before the agent backend stalled.
- [x] (2026-03-21 03:38Z) Verified that `raspberry watch --iterations 1` on the same manifest also timed out without rendering output.
- [ ] Upstream Fabro/Raspberry fixes are still required before `execute/status/watch` truth is trustworthy again for this frontier.
- [ ] After the upstream fixes land, rerun `games:multi-game` through Raspberry dispatch and record the resulting detached run as either complete or truthfully failed.

## Surprises & Discoveries

- Observation: `raspberry status`, `raspberry watch`, and `raspberry execute` can time out without rendering any terminal output even while `.raspberry` state files are being mutated.
  Evidence: `timeout 20 raspberry watch --manifest <abs myosu-platform> --iterations 1` exited `124` with no output; `timeout 60 raspberry execute --manifest <abs myosu-platform> --lane games:multi-game` also exited `124` with no output and did not create a run under the temporary `HOME`.

- Observation: the default detached Fabro run root is not sandbox-safe in this environment.
  Evidence: `fabro --no-upgrade-check run --detach fabro/run-configs/platform/multi-game.toml` failed immediately with `Read-only file system (os error 30)` because the default run root is `~/.fabro/runs`.

- Observation: the relative run-config path is enough to break `directory = "../../.."` resolution inside detached runs.
  Evidence: `fabro --no-upgrade-check run --detach --run-dir /tmp/fabro-multigame-2 fabro/run-configs/platform/multi-game.toml` produced a truthful `status=failed` run whose detach log ended with `Failed to set working directory to : No such file or directory`.

- Observation: an absolute run-config path avoids the empty-working-directory bug but still reveals a detached truth defect.
  Evidence: `fabro --no-upgrade-check run --detach --run-dir /tmp/fabro-multigame-abs-2 $(realpath fabro/run-configs/platform/multi-game.toml)` returned run id `01KM77ANMNBBQBECZFYK2Y2FNJ`, wrote `run.pid`, then left the run stuck at `starting/sandbox_initializing` after the worker disappeared.

- Observation: the `games:multi-game` lane itself is not the first blocker anymore; the supervision path is.
  Evidence: foreground run `01KM77GNB586TZVC2AEKF1TRXE` reached `Specify`, wrote `prompt.md`, `provider_used.json`, and progress events under `/tmp/fabro-foreground-multigame-2/nodes/specify/`.

- Observation: the active agent backend still has local-environment friction unrelated to the Myosu lane definition.
  Evidence: the foreground run's `cli_stdout.log` records repeated `EROFS` failures creating `/home/r/.claude/session-env/...` and repeated `api_retry` events with `error="unknown"`.

- Observation: not all existing Raspberry state files match artifact reality.
  Evidence: `.raspberry/myosu-bootstrap-state.json` currently marks `games:traits` and `tui:shell` as `blocked` even though `outputs/games/traits/review.md` and `outputs/tui/shell/review.md` already exist.

## Decision Log

- Decision: treat this foundations slice as execution-truth review, not as another manifest-authoring pass.
  Rationale: the current frontier already has seeded manifests and reviewed lane artifacts; the missing piece is honest supervision truth.
  Date/Author: 2026-03-21 / Codex

- Decision: use live Myosu execution attempts as the source of truth, even when they expose defects outside this repository.
  Rationale: the frontier tasks explicitly require fixing Raspberry/Fabro only when real Myosu execution discovers the defect.
  Date/Author: 2026-03-21 / User + Codex

- Decision: do not hand-edit `.raspberry` state or curated lane outputs to make the frontier look healthier than it is.
  Rationale: that would recreate the exact false-truth problem this slice is meant to eliminate.
  Date/Author: 2026-03-21 / Codex

- Decision: record the current Fabro path issues as two distinct defects, not one.
  Rationale: the first failure is the unwritable default run root, while the second is the empty working-directory bug triggered by relative run-config paths.
  Date/Author: 2026-03-21 / Codex

- Decision: treat the detached `starting/sandbox_initializing` freeze as a control-plane bug even though the child created `run.pid`.
  Rationale: the worker is dead and the run state never transitions to `failed`, so the run truth remains misleading.
  Date/Author: 2026-03-21 / Codex

## Outcomes & Retrospective

This slice produced the first honest foundations review for the current frontier. The repo now has written evidence that:

- the current Myosu lane surfaces are real enough to exercise
- the `games:multi-game` lane can start under foreground Fabro execution
- the detached Fabro and Raspberry supervisory paths are still not trustworthy in this sandbox without upstream fixes

The important gap is no longer documentation coverage. The important gap is bounded truth: a contributor needs `execute/status/watch` to either report a real run or fail clearly and quickly. The next slice should therefore be upstream fix work plus a rerun, not more local artifact authoring.

## Context and Orientation

The current frontier is anchored by existing reviewed artifacts under `outputs/`, especially:

- `outputs/games/multi-game/spec.md`
- `outputs/games/multi-game/review.md`
- `outputs/games/poker-engine/spec.md`
- `outputs/games/poker-engine/review.md`
- `outputs/sdk/core/spec.md`
- `outputs/sdk/core/review.md`

The active supervisory surfaces are:

- `fabro/programs/myosu-platform.yaml` for the platform frontier
- `fabro/run-configs/platform/multi-game.toml` for the `games:multi-game` lane
- `.raspberry/*.json` for Raspberry's persisted program state

The sibling Fabro repository at `/home/r/coding/fabro` remains the location of the current supervisor implementation, especially:

- `lib/crates/fabro-config/src/run.rs`
- `lib/crates/fabro-cli/src/main.rs`
- `lib/crates/raspberry-supervisor/src/dispatch.rs`
- `lib/crates/raspberry-supervisor/src/manifest.rs`

This plan assumes those upstream files, not the Myosu repository alone, must change before the current frontier can be considered trustworthy.

## Plan of Work

The next honest slice is small and specific.

First, keep the Myosu-side frontier definition stable. The current manifests and outputs are good enough to reproduce the defects, so do not widen or redesign the platform frontier yet.

Second, fix the upstream Fabro/Raspberry path and truth issues that were observed during live Myosu execution:

1. Fabro must stop collapsing relative run-config `directory` values to the empty string when the `.toml` path was supplied relatively from the CLI.
2. Raspberry dispatch must stop depending on Fabro's unwritable default run root in sandboxed environments; it should pass an explicit run dir when one is available or derive a writable one itself.
3. Detached Fabro runs that die after writing `run.pid` but before advancing out of `submitted` or `starting` must be rewritten to `failed` with preserved failure detail.
4. Raspberry `status` and `watch` must return visible output in bounded time for the Myosu platform manifest, rather than silently mutating `.raspberry` files.

Third, rerun `games:multi-game` through Raspberry dispatch after those fixes. The rerun should be judged only by observable behavior: either it produces a live detached run with advancing status, or it fails quickly and truthfully with a usable reason.

## Concrete Steps

Work from the Myosu repository root unless otherwise stated.

1. Reproduce the current detached Fabro default-run-root failure.

       /home/r/.cache/cargo-target/debug/fabro --no-upgrade-check run --detach fabro/run-configs/platform/multi-game.toml

   Expected current result in this sandbox:

       error: Read-only file system (os error 30)

2. Reproduce the truthful detached failure with an explicit writable run dir and the relative run-config path.

       RUN_DIR=/tmp/fabro-multigame-repro
       mkdir -p "$RUN_DIR"
       /home/r/.cache/cargo-target/debug/fabro --no-upgrade-check run --detach --run-dir "$RUN_DIR" fabro/run-configs/platform/multi-game.toml
       sed -n '1,120p' "$RUN_DIR/detach.log"
       sed -n '1,120p' "$RUN_DIR/status.json"

   Expected current result:

       detach.log ends with "Failed to set working directory to : No such file or directory"
       status.json shows "status": "failed"

3. Reproduce the detached stale-starting bug with a writable run dir and an absolute run-config path.

       ABS_CFG=$(realpath fabro/run-configs/platform/multi-game.toml)
       RUN_DIR=/tmp/fabro-multigame-detach-live
       mkdir -p "$RUN_DIR"
       /home/r/.cache/cargo-target/debug/fabro --no-upgrade-check run --detach --run-dir "$RUN_DIR" "$ABS_CFG"
       cat "$RUN_DIR/run.pid"
       sed -n '1,120p' "$RUN_DIR/status.json"

   Expected current result:

       Fabro prints a run id
       run.pid exists
       the worker later disappears while status.json remains stuck at "starting"

4. Reproduce the live foreground run that proves the lane can start.

       ABS_CFG=$(realpath fabro/run-configs/platform/multi-game.toml)
       RUN_DIR=/tmp/fabro-multigame-foreground
       mkdir -p "$RUN_DIR"
       /home/r/.cache/cargo-target/debug/fabro --no-upgrade-check run --run-dir "$RUN_DIR" "$ABS_CFG"

   Expected current result before upstream fixes:

       the run reaches stage "Specify"
       `nodes/specify/` is created
       `cli_stdout.log` shows local Claude hook and API retry problems

5. Re-check Raspberry once the upstream fixes land.

       ABS_MANIFEST=$(realpath fabro/programs/myosu-platform.yaml)
       HOME=/tmp/raspberry-home /home/r/.cache/cargo-target/debug/raspberry status --manifest "$ABS_MANIFEST"
       HOME=/tmp/raspberry-home /home/r/.cache/cargo-target/debug/raspberry watch --manifest "$ABS_MANIFEST" --iterations 1
       HOME=/tmp/raspberry-home /home/r/.cache/cargo-target/debug/raspberry execute --manifest "$ABS_MANIFEST" --fabro-bin /home/r/.cache/cargo-target/debug/fabro --lane games:multi-game

## Validation and Acceptance

This foundations slice is complete today because it has established honest truth. The next execution slice should not be called complete until all of the following are true:

- `raspberry status --manifest <abs myosu-platform>` returns rendered output in bounded time
- `raspberry watch --manifest <abs myosu-platform> --iterations 1` returns rendered output in bounded time
- `raspberry execute --manifest <abs myosu-platform> --lane games:multi-game` either dispatches a real detached run or fails immediately with a clear error
- a detached `games:multi-game` run no longer freezes forever at `submitted` or `starting` after the worker dies
- `.raspberry/myosu-bootstrap-state.json` agrees with the currently present reviewed bootstrap artifacts

## Idempotence and Recovery

The reproduction commands above are safe to rerun because they write their transient Fabro runs under `/tmp/` and only overwrite lane outputs through real Fabro execution. Do not delete reviewed artifacts under `outputs/` to force the frontier forward. If a run hangs or freezes, preserve the run dir and treat it as evidence; the correct recovery is to fix the upstream supervisor or detach behavior and rerun, not to hand-edit state files.

## Artifacts and Notes

The key evidence from this slice is:

- truthful detached failure run dir: `/tmp/fabro-multigame-2`
- detached stale-starting run: `01KM77ANMNBBQBECZFYK2Y2FNJ` under `/tmp/fabro-multigame-abs-2`
- foreground live run: `01KM77GNB586TZVC2AEKF1TRXE` under `/tmp/fabro-foreground-multigame-2`
- stale supervisor state example: `.raspberry/myosu-bootstrap-state.json`

## Interfaces and Dependencies

The next upstream fixes should target these interfaces directly:

- `fabro_config::run::load_run_config` in `/home/r/coding/fabro/lib/crates/fabro-config/src/run.rs`
- `detach_run` and `ensure_detached_child_started` in `/home/r/coding/fabro/lib/crates/fabro-cli/src/main.rs`
- `raspberry_supervisor::dispatch::run_fabro` in `/home/r/coding/fabro/lib/crates/raspberry-supervisor/src/dispatch.rs`
- manifest path resolution in `/home/r/coding/fabro/lib/crates/raspberry-supervisor/src/manifest.rs`

Revision Note: Created on March 21, 2026 UTC after live Myosu execution of the `games:multi-game` frontier exposed detached-run and supervisor-truth defects that are not honestly representable as "done."

# Foundations Lane Review

## Judgment

**Judgment: REOPEN**

The current frontier has enough real evidence to keep the Myosu lane definitions and existing reviewed artifacts, but it does not have trustworthy Raspberry/Fabro supervision truth yet. The honest action is to reopen the foundations frontier around control-plane truth, not around product scope.

## Findings

### 1. `raspberry execute/status/watch` are not currently trustworthy for the live Myosu platform frontier. (Critical)

`raspberry status`, `raspberry watch`, and `raspberry execute` against `fabro/programs/myosu-platform.yaml` all timed out in this environment without rendering usable output. The commands therefore cannot currently serve as operator truth, even before we ask whether their lane classifications are correct.

Evidence:

- `timeout 20 raspberry watch --manifest <abs myosu-platform> --iterations 1` exited `124` with no output.
- `timeout 60 raspberry execute --manifest <abs myosu-platform> --lane games:multi-game` exited `124` with no output and did not create a detached run under the temporary `HOME`.
- the earlier `raspberry status` refresh mutated `.raspberry/*.json` but did not render terminal output before timing out.

Impact:

- The frontier cannot honestly claim `execute/status/watch` truth is repaired.
- Any lane judgment derived only from `.raspberry` state files remains suspect until the CLI surfaces render bounded, visible truth again.

### 2. Fabro's default detached run root is not safe in this sandbox, so Raspberry's current detach path is environment-fragile. (Critical)

Direct detached Fabro execution of `games:multi-game` failed immediately when it tried to use the default `~/.fabro/runs` root.

Evidence:

- `/home/r/.cache/cargo-target/debug/fabro --no-upgrade-check run --detach fabro/run-configs/platform/multi-game.toml`
- Result: `error: Read-only file system (os error 30)`

Impact:

- Any supervisory path that depends on Fabro's default detached run root will fail in this workspace sandbox before real lane work even begins.
- Raspberry needs to pass an explicit writable run dir, or Fabro needs a safer default under sandboxed execution.

### 3. Relative run-config paths can still break detached execution by collapsing `directory = "../../.."` to an empty working directory. (Critical)

After giving Fabro a writable detached run dir, the lane no longer failed on the run root. It failed on path resolution instead.

Evidence:

- Command: `fabro --no-upgrade-check run --detach --run-dir /tmp/fabro-multigame-2 fabro/run-configs/platform/multi-game.toml`
- Detach log ended with: `Failed to set working directory to : No such file or directory (os error 2)`
- `status.json` in `/tmp/fabro-multigame-2` was truthfully rewritten to `failed`

Impact:

- The current relative-path handoff between Raspberry/Fabro and Myosu run configs is still not robust enough for detached execution.
- Upstream path resolution, not the lane's prompt or workflow shape, is the blocker here.

### 4. Detached runs can still die after startup and remain frozen at `starting/sandbox_initializing` instead of transitioning to `failed`. (Critical)

Using an absolute run-config path avoids the empty-working-directory bug, but the next truth defect appears immediately afterward.

Evidence:

- Command: `fabro --no-upgrade-check run --detach --run-dir /tmp/fabro-multigame-abs-2 $(realpath fabro/run-configs/platform/multi-game.toml)`
- Fabro returned run id `01KM77ANMNBBQBECZFYK2Y2FNJ`
- `/tmp/fabro-multigame-abs-2/run.pid` existed, but the recorded PID was no longer alive
- `status.json` remained:

      {
        "status": "starting",
        "reason": "sandbox_initializing",
        ...
      }

Impact:

- This is the current honest version of the earlier false-submit problem: the run looks alive enough to supervise, but the worker is already dead.
- Detached startup still needs a terminal failure transition when the worker disappears after `run.pid` exists.

### 5. Bootstrap state is already inconsistent with artifact reality in at least one checked-in supervisor state file. (High)

Not all persisted Raspberry state currently agrees with the reviewed artifacts already present in the repository.

Evidence:

- `.raspberry/myosu-bootstrap-state.json` marks `games:traits` and `tui:shell` as `blocked`
- `outputs/games/traits/review.md` exists
- `outputs/tui/shell/review.md` exists

Impact:

- `.raspberry` JSON cannot currently be treated as authoritative without cross-checking the artifact surface.
- The supervisor refresh logic still needs a truth pass before more frontiers are declared stable.

### 6. The `games:multi-game` lane itself is executable enough to start; the current blocker is supervision truth, not lane design. (Medium)

A foreground run with a writable run dir and an absolute run-config path reached real workflow execution and stage materialization.

Evidence:

- Command: `fabro --no-upgrade-check run --run-dir /tmp/fabro-foreground-multigame-2 $(realpath fabro/run-configs/platform/multi-game.toml)`
- Run id: `01KM77GNB586TZVC2AEKF1TRXE`
- `state.json` recorded `current_stage_label = "Specify"`
- `progress.jsonl` recorded `WorkflowRunStarted`, `StageStarted`, and `CliEnsureCompleted`
- `nodes/specify/` contained `prompt.md`, `provider_used.json`, and `cli_stdout.log`

Impact:

- The multi-game lane does not need a scope reset.
- The next work should stay focused on Fabro/Raspberry truth and execution plumbing.

### 7. The foreground run exposed local Claude CLI environment problems, but those are secondary to the control-plane defects above. (Medium)

The lane got far enough to reach the agent backend, where the current local environment introduced additional friction.

Evidence from `/tmp/fabro-foreground-multigame-2/nodes/specify/cli_stdout.log`:

- repeated `EROFS` failures creating `/home/r/.claude/session-env/...`
- repeated `api_retry` events with `error="unknown"`

Impact:

- Even after the detach and supervisor truth issues are fixed, this local agent environment may still need adjustment for successful end-to-end runs.
- This does not change the current foundations judgment, because the control-plane defects are strictly earlier and more severe.

## Keep / Reopen / Reset Decision

Keep:

- the current Myosu platform manifests
- the current `games:multi-game` lane definition
- the existing reviewed artifacts under `outputs/games/multi-game/`

Reopen:

- the truth claim that Raspberry supervision is already trustworthy for this frontier
- the detached Fabro start path for Myosu lanes
- the consistency of `.raspberry` state with present artifacts

Reset:

- do **not** reset the lane scope, workflow family, or output artifacts
- do **not** delete or rewrite reviewed artifacts to hide the control-plane gap

## What Must Be True Before This Frontier Can Close

1. `raspberry status --manifest <abs myosu-platform>` returns visible output in bounded time.
2. `raspberry watch --manifest <abs myosu-platform> --iterations 1` returns visible output in bounded time.
3. `raspberry execute --manifest <abs myosu-platform> --lane games:multi-game` dispatches a real detached run or fails immediately with a clear reason.
4. A dead detached worker no longer leaves the run frozen at `submitted` or `starting`.
5. Bootstrap and platform `.raspberry` state agree with the artifact surface actually present in `outputs/`.

## Recommended Next Fixes

1. Fix relative run-config path handling in Fabro so `directory = "../../.."` resolves correctly when the `.toml` path is passed relatively from the CLI.
2. Teach Raspberry dispatch to pass an explicit writable `run_dir` when launching detached Fabro runs, instead of always depending on `~/.fabro/runs`.
3. Harden the detached Fabro wrapper so a worker that dies after creating `run.pid` still rewrites `status.json` to `failed`.
4. Make `raspberry status/watch` return rendered output in bounded time for the Myosu platform manifest before the next frontier round is called healthy.

## Residual Risks

- The current sandbox cannot write to `/home/r/.fabro` or `/home/r/.claude`, so any path that silently assumes those writable defaults will continue to fail here.
- The local Claude CLI environment may still prevent a clean end-to-end completion even after detached-truth issues are fixed.
- Until the upstream fixes land, any "complete" status in `.raspberry` should be treated as advisory rather than authoritative.

# Foundations Frontier Review

**Lane:** `foundations:foundations`
**Kind:** platform
**Owning program:** `myosu-foundations`
**Output root:** `outputs/foundations/`
**Date:** 2026-03-21

## Keep / Reopen / Reset Judgment

**Judgment: KEEP THE CURRENT OBSERVER SURFACES, REOPEN THE EXECUTE PATH**

Keep the current artifact-backed observer path. On 2026-03-21, fresh
`raspberry plan`, `status`, and `watch` calls over the existing checked-in
Myosu programs were coherent and did not surface the stale historical
`games:multi-game` false-submit as active work.

Reopen the execute path. The new foundations probe showed that current
`raspberry execute` can be made truthful again in this worktree, but only with
two explicit workarounds:

1. call Raspberry with an absolute manifest path
2. use a repo-local Fabro wrapper that injects a writable `--run-dir` and
   `FABRO_LOG_DIR`

Even with those workarounds, the detached worker still disappears during the
foundations `Plan` stage, and Raspberry ultimately reports a truthful failure:

    error: tracked run remained active after its worker process disappeared

No broad Myosu reset is warranted.

## What Is Trustworthy Enough To Keep

### Current Observer Truth On Existing Frontiers

These surfaces are trustworthy enough to keep:

- `raspberry plan --manifest fabro/programs/myosu-platform.yaml`
- `raspberry status --manifest fabro/programs/myosu-platform.yaml`
- `raspberry watch --manifest fabro/programs/myosu-platform.yaml`
- the equivalent `status` views over `myosu-product` and `myosu-recurring`

Observed evidence:

- `myosu-platform` reported `complete=3 ready=0 running=0 blocked=0 failed=0`
- `myosu-product` reported `complete=2 ready=0 running=0 blocked=0 failed=0`
- `myosu-recurring` reported `complete=4 ready=0 running=0 blocked=0 failed=0`

This matters because the frontier task was not simply “find any old bad run.”
It was “restore trustworthy current execute/status/watch truth.”

### Historical False-Submit As Evidence, Not Active Truth

The stale `games:multi-game` detached run still exists on disk:

- `~/.fabro/runs/20260319-01KM2BS4ASVRXVT2ND1GVVMKJ0/status.json`
- `~/.fabro/runs/20260319-01KM2BS4ASVRXVT2ND1GVVMKJ0/detach.log`

Its recorded facts remain important:

    {
      "status": "submitted",
      "updated_at": "2026-03-19T06:16:00.090006038Z"
    }

    error: unexpected argument '/home/r/coding/myosu/fabro/programs/../run-configs/platform/multi-game.toml' found

But that stale directory no longer appears to poison refreshed current program
state in this worktree.

### The Isolated Foundations Canary

The new `myosu-foundations` program is worth keeping even though its first live
execute probe failed. It is the right shape for future hardening work:

- one lane
- two durable artifacts
- no production crate churn
- a controlled repo-local run dir under `.raspberry/runs/foundations/`

That gives Fabro/Raspberry a narrow proving ground instead of forcing future
control-plane experiments through `games:multi-game` again.

## What Must Be Reopened

### Relative-Manifest Execute Path

The first foundations execute probe using a relative manifest path failed before
it reached a usable run. The current control plane still mishandles this path
shape in practice.

The best observed explanation from the repo-local reproduction is:

- relative manifest path
- relative resolved run-config path
- `directory = "../../.."` in the run config
- Fabro path normalization collapsing that work dir to an empty string

The corresponding direct reproduction produced:

    Failed to set working directory to : No such file or directory (os error 2)

This is now a concrete upstream bug, not a vague suspicion.

### Detached Worker Disappearance After Truthful Submission

With the absolute-manifest and wrapper workaround, `raspberry execute` did
submit a real run:

    foundations:foundations [submitted] run_id=01KM75Q2QBHW7XN46707XJJ00M exit_status=0

`raspberry status` and `watch` then truthfully reported the lane as running at
stage `Plan`, with the same run id. The repo-local run directory contained the
expected live run files:

- `manifest.json`
- `run.toml`
- `run.pid`
- `sandbox.json`
- `state.json`
- `status.json`
- `progress.jsonl`

But the worker process disappeared before `Plan` completed, and Raspberry
ultimately reported:

    foundations:foundations [failed|platform] failure_kind=transient_launch_failure
      error: tracked run remained active after its worker process disappeared

This is the central reopened defect for the next Fabro/Raspberry slice.

## What Should Be Reset

Nothing else.

Do not reset:

- `myosu-bootstrap`
- `myosu-platform`
- `myosu-product`
- `myosu-recurring`

Do not widen `myosu-bootstrap.yaml`.

Do not throw away the historical `games:multi-game` run evidence.

The only thing that needs a restart is the upstream Fabro/Raspberry hardening
loop for execute-path reliability.

## Concrete Next Actions

1. In the sibling Fabro repo, fix relative-manifest execute handling so
   Raspberry resolves run-config and work-dir paths robustly even when the
   manifest argument is relative.
2. In the sibling Fabro repo, debug why the detached worker disappears after
   `StageStarted` for the foundations `Plan` command stage.
3. Keep the repo-local Fabro wrapper and foundations program in Myosu as the
   repro harness until the upstream fixes land.
4. After the upstream fixes land, rerun:

       /home/r/coding/fabro/target-local/debug/raspberry execute --manifest /home/r/.fabro/runs/20260320-01KM74SBWAM4KBQHZJYKJ5AEZX/worktree/fabro/programs/myosu-foundations.yaml --lane foundations:foundations --fabro-bin /home/r/.fabro/runs/20260320-01KM74SBWAM4KBQHZJYKJ5AEZX/worktree/fabro/checks/fabro-local-dispatch.sh

   and do not close this frontier until the lane either writes both artifacts
   from inside Fabro or produces a different, sharper truthful failure.
5. Only after the foundations canary is stable should the historical
   `games:multi-game` submitted-only run be given a cleanup or tombstone policy.

## Readiness Verdict

**Ready for upstream Fabro/Raspberry hardening, not for closure.**

This frontier now has the right kind of evidence:

- current observer surfaces are mostly healthy
- the stale historical run is isolated as evidence
- the execute path has a reproducible narrow failure

That is enough to move forward honestly, but not enough to call the execute
path “trustworthy again” in the strongest sense. The trustworthy part today is
the reporting, not the detached worker completion path.

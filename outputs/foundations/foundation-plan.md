# Foundations Lane — Foundation Plan

Date: 2026-03-20
Status: active baseline

## Goal

Bootstrap the first honest reviewed slice for the current frontier by making the
local Fabro/Raspberry truth surface dependable again for this worktree, then
recording that baseline as durable repo evidence.

This slice is intentionally narrow. It does not claim Stage 0 product progress.
It claims only that the current checked-in control-plane package can be
evaluated honestly from this repository and that the result matches the curated
artifacts already present under `outputs/`.

## Inputs

- `README.md`
- `SPEC.md`
- `PLANS.md`
- `AGENTS.md`
- `specs/031626-00-master-index.md`
- `specs/031826-fabro-primary-executor-decision.md`
- `plans/031926-iterative-execution-and-raspberry-hardening.md`
- the checked-in Fabro package under `fabro/`
- the current curated outputs under `outputs/`

## Verified Starting Point

Before this slice, the repo already contained reviewed artifacts for the seeded
frontiers, but the local proof path was not trustworthy in this worktree:

- the documented Raspberry commands in `README.md` and `fabro/README.md` used
  invalid Cargo syntax: `cargo --manifest-path ... run ...`
- the local proof scripts inherited a read-only `CARGO_TARGET_DIR`, causing
  otherwise-valid `cargo test` and `cargo check` proofs to fail inside
  Raspberry evaluation
- the historical `games:multi-game` false-submit remained documented in
  `plans/031926-iterative-execution-and-raspberry-hardening.md`, but there was
  no fresh honest foundations artifact tying current local truth together

## Scope of This Slice

This slice owns only the local foundation needed for truthful supervision:

1. Correct the repo-local proof scripts so Myosu-owned checks use a writable
   repo-local cargo target.
2. Correct the checked-in operator docs so the Raspberry invocation path is the
   one that worked in real execution here.
3. Re-run the affected proof and status surfaces until `status` and `watch`
   report the same truthful result for the full Myosu portfolio in this
   worktree.
4. Record the outcome and the remaining external risk in durable artifacts
   under `outputs/foundations/`.

This slice does not attempt to reopen product doctrine, re-run historical
detached `games:multi-game` execution, or edit the sibling Fabro repository.

## Repo Changes Included in This Slice

The local repair is intentionally small:

- `fabro/checks/games-traits.sh`
- `fabro/checks/games-traits-implement.sh`
- `fabro/checks/tui-shell.sh`
  These scripts now force `CARGO_TARGET_DIR` to
  `${MYOSU_CARGO_TARGET_DIR:-$repo_root/.raspberry/cargo-target}` and create
  that directory before running Cargo. This makes the proofs Myosu-owned and
  stable inside this worktree.

- `README.md`
- `fabro/README.md`
  These docs now use the valid Cargo form
  `cargo run --manifest-path ... -p raspberry-cli -- ...`, set
  `CARGO_TARGET_DIR="$PWD/.raspberry/cargo-target"`, and pass an absolute
  manifest path via `MYOSU_MANIFEST="$PWD/fabro/programs/myosu.yaml"`.

## Proof We Must Keep True

From the repository root:

1. `./fabro/checks/games-traits.sh`
2. `./fabro/checks/tui-shell.sh`
3. `./fabro/checks/games-traits-implement.sh`
4. `/home/r/coding/fabro/target-local/debug/raspberry status --manifest "$PWD/fabro/programs/myosu-bootstrap.yaml"`
5. `/home/r/coding/fabro/target-local/debug/raspberry status --manifest "$PWD/fabro/programs/myosu-games-traits-implementation.yaml"`
6. `/home/r/coding/fabro/target-local/debug/raspberry status --manifest "$PWD/fabro/programs/myosu.yaml"`
7. `/home/r/coding/fabro/target-local/debug/raspberry watch --manifest "$PWD/fabro/programs/myosu.yaml" --iterations 1`
8. `/home/r/coding/fabro/target-local/debug/raspberry execute --manifest "$PWD/fabro/programs/myosu.yaml"`

The expected outcomes for this slice are:

- the three proof scripts exit 0
- `myosu-bootstrap` reports `complete=4`
- `myosu-games-traits-implementation` reports `complete=1`
- top-level `myosu` reports `complete=7 ready=0 running=0 blocked=0 failed=0`
- `watch` reports the same top-level truth
- `execute` returns a truthful no-op failure: `Error: no ready lanes selected for execution`

That last result is acceptable here because every seeded frontier is already
complete in the current artifact state. Honest no-op is better than fabricated
dispatch.

## Durable Acceptance for This Foundations Slice

This foundations slice is complete when all of the following are true:

- `outputs/foundations/foundation-plan.md` exists and explains the local truth
  contract
- `outputs/foundations/review.md` exists and records the evidence from the
  rerun
- the local Myosu proof scripts no longer depend on an inherited read-only
  cargo target
- the checked-in operator docs match the invocation form that worked in real
  execution here
- the current worktree can rehydrate `.raspberry/*-state.json` so that
  `raspberry status` and `raspberry watch` agree on the top-level program state

## Remaining Work After This Slice

The next honest work is outside this narrow foundations contract:

- Reproduce the historical `games:multi-game` detach false-submit on the next
  genuinely ready lane, then fix it in the sibling Fabro repository if it still
  occurs.
- Refresh stale operational surfaces such as `ops/scorecard.md` and the
  recurring planning review, which still lag the now-complete seeded frontier
  state.
- Resume product/chain/service execution only when a new lane becomes ready.

## Notes

This plan is deliberately conservative. The control-plane truth is now healthy
enough to support the next honest frontier move, but it does not change the
fact that Stage 0 product exit criteria remain unmet.

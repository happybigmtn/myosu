# Foundations Lane — Review

Date: 2026-03-20
Judgment: KEEP

## Summary

The first honest foundations slice is now in place for this worktree. The local
Fabro/Raspberry control plane can be evaluated truthfully from checked-in repo
surfaces, and the result matches the curated artifacts already present under
`outputs/`.

This review is intentionally narrow. It does not claim product readiness. It
claims that the current frontier package is now locally trustworthy enough to
use as the baseline for the next real execution slice.

## What Was Verified

### 1. The local proof scripts were the immediate blocker, not the underlying crates

The following commands now exit 0 from the repo root after pinning proofs to a
repo-local cargo target:

- `./fabro/checks/games-traits.sh`
- `./fabro/checks/tui-shell.sh`
- `./fabro/checks/games-traits-implement.sh`

Observed results:

- `myosu-games`: 10 unit tests passed, 4 doctests passed
- `myosu-tui`: 82 unit tests passed, 2 ignored TTY-only tests, 1 doctest passed
- `games-traits-implement`: `cargo check -p myosu-games` plus the same passing
  `myosu-games` test suite

Conclusion: the blocked Raspberry proofs were caused by inherited local cargo
target state, not by failing `myosu-games` or `myosu-tui` code in this
worktree.

### 2. Bootstrap and implementation truth now resolve cleanly

Observed commands:

- `/home/r/coding/fabro/target-local/debug/raspberry status --manifest "$PWD/fabro/programs/myosu-bootstrap.yaml"`
- `/home/r/coding/fabro/target-local/debug/raspberry status --manifest "$PWD/fabro/programs/myosu-games-traits-implementation.yaml"`

Observed outcomes:

- `myosu-bootstrap`: `complete=4 ready=0 running=0 blocked=0 failed=0`
- `myosu-games-traits-implementation`: `complete=1 ready=0 running=0 blocked=0 failed=0`

This matches the reviewed and verified artifacts already checked into
`outputs/games/traits/`, `outputs/tui/shell/`, and `outputs/chain/*`.

### 3. Portfolio status and watch now agree on the top-level truth

Observed commands:

- `/home/r/coding/fabro/target-local/debug/raspberry status --manifest "$PWD/fabro/programs/myosu.yaml"`
- `/home/r/coding/fabro/target-local/debug/raspberry watch --manifest "$PWD/fabro/programs/myosu.yaml" --iterations 1`

Observed outcome in both cases:

- `Program: myosu`
- `Counts: complete=7 ready=0 running=0 blocked=0 failed=0`

Per-program child summaries:

- `bootstrap`: complete=4
- `chain-core`: complete=2
- `games-traits-implementation`: complete=1
- `platform`: complete=3
- `product`: complete=2
- `recurring`: complete=4
- `services`: complete=2

This is the first honest reviewed slice for the current frontier because it
demonstrates that the seeded frontier manifests, proofs, and curated artifacts
now converge to the same local truth.

### 4. `execute` now fails truthfully when there is nothing to do

Observed command:

- `/home/r/coding/fabro/target-local/debug/raspberry execute --manifest "$PWD/fabro/programs/myosu.yaml"`

Observed outcome:

- exit status 1
- `Error: no ready lanes selected for execution`

This is acceptable and preferable here. The seeded frontiers are already
complete, so the honest result is a no-op failure, not a fabricated dispatch.

## Repo Surfaces Updated by This Slice

- `fabro/checks/games-traits.sh`
- `fabro/checks/games-traits-implement.sh`
- `fabro/checks/tui-shell.sh`
- `README.md`
- `fabro/README.md`

These are small but material fixes. They align the checked-in operator loop
with the path that actually worked in real Myosu execution in this workspace.

## Residual Risk

One important historical risk remains open:

- the documented `games:multi-game` detach false-submit incident from
  `plans/031926-iterative-execution-and-raspberry-hardening.md` was not
  re-exercised here, because the current top-level portfolio has no ready lanes
  left to dispatch

That means this slice restores trustworthy local `status/watch/execute`
behavior for the current artifact state, but it does not prove that the sibling
Fabro detach path is fixed under a fresh ready-lane execution.

## Recommendation

Keep this foundations slice as the local baseline and use it for the next real
frontier move. When a genuinely ready lane appears again, re-test dispatch
against the sibling Fabro runtime and treat any new false-submit as Fabro work,
not as something to paper over in Myosu doctrine.

# Foundations Lane — Review

**Lane:** `foundations`
**Date:** 2026-03-21
**Judgment:** REOPEN

---

## Why This Slice Must Stay Reopened

The repository has enough real structure to avoid a reset: doctrine is coherent,
the manifests are seeded, curated outputs exist, and the two trusted leaf crates
still test cleanly. But the control-plane truth surface is not yet honest
enough to close. In this workspace, Raspberry mutates state without reliably
rendering operator-visible output, and the `games:multi-game` rerun could not be
driven to a truthful failure or a successful live submission.

This is therefore a reopen, not a keep and not a reset.

---

## Findings

### 1. Critical: `raspberry status` mutates `.raspberry` state but does not complete the human-facing status render

Evidence gathered on 2026-03-21:

    timeout 60 /home/r/.cache/cargo-target/debug/raspberry status --manifest fabro/programs/myosu-bootstrap.yaml
    Result: exit 124, no stdout

Before that run, `.raspberry/myosu-bootstrap-state.json` marked `games:traits`
and `tui:shell` as `blocked`. After the run, the same file showed:

    updated_at: 2026-03-21T04:09:52.410645982Z
    chain:pallet: complete
    chain:runtime: complete
    games:traits: complete
    tui:shell: complete

That means the evaluation engine refreshed truth, but the CLI failed to return
the rendered table. An operator cannot trust a status surface that mutates state
and then stalls before showing what changed.

### 2. Critical: `raspberry execute` did not produce a truthful rerun result for `games:multi-game`

Evidence gathered on 2026-03-21:

    timeout 5 /home/r/.cache/cargo-target/debug/raspberry execute --manifest fabro/programs/myosu-platform.yaml --fabro-bin /home/r/.cache/cargo-target/debug/fabro --lane games:multi-game
    Result: exit 124, no stdout

The platform state file remained unchanged:

    .raspberry/myosu-platform-state.json
    updated_at: 2026-03-21T04:04:01.298858887Z

No new run id was surfaced, no failure was rendered, and no state change proved
that the rerun even reached the dispatch boundary. The `games:multi-game`
false-submit problem is therefore still unresolved in this workspace.

### 3. High: the bootstrap cargo-backed proof scripts were inheriting a stale external target dir

The shell environment in this run exported:

    CARGO_TARGET_DIR=/home/r/coding/myosu/.worktrees/autodev-live/.raspberry/cargo-target

That path is outside this writable worktree, so the original proofs failed with
read-only filesystem errors before they could measure repo truth. This was a
real Myosu-local portability defect, not only a Fabro defect.

This slice fixed the issue by hardening:

- `fabro/checks/games-traits.sh`
- `fabro/checks/tui-shell.sh`

Both now use the current target repo's `.raspberry/cargo-target`.

### 4. Medium: the underlying trusted leaf crates remain healthy once the proof path is corrected

After the proof-script hardening, these checks passed directly in this worktree:

    ./fabro/checks/games-traits.sh
    Result: 10 tests passed, 4 doctests passed

    ./fabro/checks/tui-shell.sh
    Result: 82 tests passed, 2 ignored, 1 doctest passed

    ./fabro/checks/chain-runtime-reset.sh
    Result: exit 0

    ./fabro/checks/chain-pallet-reset.sh
    Result: exit 0

This matters because it narrows the current frontier problem. The repo-local
foundation is not “all broken”; the remaining blocker is concentrated in the
Raspberry/Fabro operator path.

---

## What This Review Accepts As Real Today

These statements are honest in the current worktree:

- The Fabro-first doctrine is in place and internally coherent.
- Reviewed bootstrap-style artifacts exist across bootstrap, platform,
  services, product, and recurring lanes.
- `myosu-games` and `myosu-tui` are still trustworthy leaf crates.
- Bootstrap proof scripts are now portable inside the current repo.
- Raspberry state refresh does happen.

These statements are not yet honest enough to claim:

- "`raspberry status` is trustworthy again"
- "`raspberry execute` can truthfully rerun `games:multi-game`"
- "the false-submit path has been repaired"

---

## Required Next Move

The next move is not more Myosu-local artifact authoring. The next move is to
repair the Raspberry/Fabro CLI render-and-return path in the sibling Fabro repo,
then rerun `games:multi-game` from this Myosu worktree until it lands in one of
two honest states:

1. A live run with an inspectable Fabro run id and run directory.
2. A direct failure with rendered output explaining why the lane did not start.

Anything weaker than that leaves the frontier reopened.

---

## Scope Boundary

The remaining repair is cross-repo. This run could inspect
`/home/r/coding/fabro`, but it could not edit it because that path is outside
the writable roots of the current sandbox. This review therefore records the
blocker and the exact rerun target instead of pretending the Fabro-side defect
was fixed locally.

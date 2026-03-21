# Foundations Lane — Foundation Plan

**Lane:** `foundations`
**Goal:** bootstrap the first honest reviewed slice for the current frontier
**Primary doctrine inputs:** `README.md`, `SPEC.md`, `PLANS.md`, `AGENTS.md`, `specs/031626-00-master-index.md`, `specs/031826-fabro-primary-executor-decision.md`
**Date:** 2026-03-21

---

## Purpose / Operator-Facing Outcome

The `foundations` lane exists to re-establish trust in the Myosu execution
surface before more frontier claims accumulate. The immediate user-visible
outcome is simple: an operator should be able to run the current Fabro and
Raspberry commands, see truthful status, and distinguish reviewed artifact
completion from real live execution.

This is not a greenfield planning lane. It is a truth-restoration lane. The
current doctrine is broadly coherent, the frontier manifests exist, and the
reviewed output roots are populated. The remaining gap is that the control-plane
commands are not yet reliably rendering or rerunning that truth.

---

## Verified Baseline On 2026-03-21

The following facts were verified directly in this worktree:

1. The active doctrine agrees on the Fabro/Raspberry cutover. `README.md`,
   `SPEC.md`, `PLANS.md`, `AGENTS.md`, and
   `specs/031826-fabro-primary-executor-decision.md` all treat Fabro as the
   primary execution substrate and Raspberry as the control plane.

2. The frontier manifests and curated outputs are present. `fabro/programs/`
   contains the bootstrap, chain-core, services, product, platform, recurring,
   and top-level portfolio manifests. `outputs/` contains reviewed artifacts for
   the current bootstrap and downstream bootstrap-style lanes.

3. The local Raspberry state surface exists and is mutable. This worktree now
   contains `.raspberry/myosu-*.json` state files for the seeded programs.

4. The bootstrap cargo-backed proof scripts were not portable at the start of
   this slice because the ambient shell exported:

       CARGO_TARGET_DIR=/home/r/coding/myosu/.worktrees/autodev-live/.raspberry/cargo-target

   In this sandbox that path is read-only, so `./fabro/checks/games-traits.sh`
   and `./fabro/checks/tui-shell.sh` failed before they could measure repo
   truth.

5. The repo-local proof scripts are now hardened to use the current target
   repo's `.raspberry/cargo-target` instead of inheriting the stale outer
   worktree target dir. The proofs now pass in this workspace:

       ./fabro/checks/games-traits.sh
       Result: 10 unit tests passed, 4 doctests passed

       ./fabro/checks/tui-shell.sh
       Result: 82 tests passed, 2 ignored, 1 doctest passed

       ./fabro/checks/chain-runtime-reset.sh
       Result: exit 0

       ./fabro/checks/chain-pallet-reset.sh
       Result: exit 0

6. Raspberry state refresh is active but the human-facing CLI surface is still
   not trustworthy. The following commands produced no stdout and were killed by
   `timeout`, even though they did mutate program state:

       timeout 60 /home/r/.cache/cargo-target/debug/raspberry status --manifest fabro/programs/myosu-bootstrap.yaml
       Result: exit 124, no rendered table

       timeout 60 /home/r/.cache/cargo-target/debug/raspberry status --manifest fabro/programs/myosu-platform.yaml
       Result: exit 124, no rendered table

       timeout 5 /home/r/.cache/cargo-target/debug/raspberry execute --manifest fabro/programs/myosu-platform.yaml --fabro-bin /home/r/.cache/cargo-target/debug/fabro --lane games:multi-game
       Result: exit 124, no rendered submission/failure output

7. The bootstrap state file changed despite the missing CLI output. After the
   stalled `status` run, `.raspberry/myosu-bootstrap-state.json` showed
   `updated_at = 2026-03-21T04:09:52.410645982Z` and all four bootstrap lanes as
   `complete`. That means the underlying evaluation logic advanced, but the CLI
   did not complete the operator-facing render.

---

## What Is Currently Trustworthy

These surfaces are good enough to build on:

- Root doctrine and migration intent.
- Checked-in Fabro workflows, run configs, and program manifests.
- Curated reviewed artifacts under `outputs/`.
- The `myosu-games` and `myosu-tui` leaf crates, as measured by the repaired
  bootstrap proof scripts.
- The chain restart file-presence checks.

---

## What Is Not Yet Trustworthy

These are the current truth gaps:

1. `raspberry plan/status/watch/execute` cannot yet be treated as trustworthy
   operator surfaces from this workspace. They may refresh state, but they do
   not reliably return rendered truth within a bounded time.

2. The `games:multi-game` false-submit problem described in
   `plans/031926-iterative-execution-and-raspberry-hardening.md` has not been
   converted into a truthful failure or a successful live rerun here. The
   current `execute` surface stalled before returning any submission result.

3. The current workspace cannot repair sibling-repo Raspberry/Fabro code
   directly because `/home/r/coding/fabro` is outside the writable roots for
   this run. The remaining repair must therefore be expressed as a concrete next
   slice, not falsely claimed as done.

---

## Smallest Honest Next Slices

### Slice 1 — Keep Repo-Local Proofs Portable

This slice is complete in the current worktree.

`fabro/checks/games-traits.sh` and `fabro/checks/tui-shell.sh` now resolve a
repo-local cargo target directory under `.raspberry/cargo-target`. This keeps
bootstrap proofs aligned with the actual target repo instead of a stale external
autodev worktree.

**Done when:** both scripts exit 0 in the current target repo without requiring
an external `CARGO_TARGET_DIR` override.

### Slice 2 — Repair the Raspberry CLI Render/Return Path in Fabro

This is the first remaining blocker. The sibling Fabro repo must be changed so
that:

- `raspberry status --manifest <program>` returns a rendered table after state
  refresh
- `raspberry plan --manifest <program>` returns grouped summary output after
  evaluation
- `raspberry execute --manifest <program> --lane <lane>` returns a truthful
  submission or failure line instead of stalling indefinitely

The important bar is not only internal state mutation. The CLI must finish the
operator contract.

**Done when:** the three commands above return human-readable output in bounded
time from this Myosu worktree.

### Slice 3 — Re-run `games:multi-game` Through Raspberry After the CLI Repair

Once Slice 2 lands in Fabro, re-run:

    /home/r/.cache/cargo-target/debug/raspberry execute \
      --manifest fabro/programs/myosu-platform.yaml \
      --fabro-bin /home/r/.cache/cargo-target/debug/fabro \
      --lane games:multi-game

This rerun must end in one of two honest states:

- a live Fabro run id whose run directory exists and can be inspected, or
- an immediate failure with stderr/stdout that explains why the lane could not
  start

What is explicitly not acceptable is a silent timeout or a synthetic
`submitted`-looking state with no inspectable run.

### Slice 4 — Re-verify `status` and `watch` Against the Same `games:multi-game` Run

After Slice 3, the operator must be able to render the same lane truth through:

    /home/r/.cache/cargo-target/debug/raspberry status --manifest fabro/programs/myosu-platform.yaml
    /home/r/.cache/cargo-target/debug/raspberry watch --manifest fabro/programs/myosu-platform.yaml --iterations 1

These commands must report the same run truth the execute step created. This is
the control-plane trust gate for the current frontier.

---

## Acceptance For The Foundations Slice

The foundations lane can be called honest when all of the following are true:

1. Repo-local bootstrap proofs run successfully in the current target repo.
2. `raspberry status` and `raspberry plan` return rendered output for the
   current manifests instead of only mutating `.raspberry/*.json`.
3. `raspberry execute` on `games:multi-game` returns a truthful live result
   rather than stalling.
4. The resulting `games:multi-game` run truth is visible consistently through
   `execute`, `status`, and `watch`.
5. The repaired path is reflected in an updated review artifact, not only in
   ephemeral terminal memory.

---

## Artifacts Produced By This Slice

- `outputs/foundations/foundation-plan.md`
- `outputs/foundations/review.md`

The repo-local proof hardening that supported this slice landed in:

- `fabro/checks/games-traits.sh`
- `fabro/checks/tui-shell.sh`

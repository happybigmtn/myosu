# Myosu Foundations — Lane Review

**Lane**: `foundations`
**Date**: 2026-03-20
**Review**: Honest assessment of Myosu Fabro frontier state

---

## Judgment Summary

| Surface | Status | Rationale |
|---------|--------|-----------|
| `games:traits` lane artifacts | **KEEP** | Trusted crate, clean test history, git dep migration done, implementation unblocked |
| `tui:shell` lane artifacts | **KEEP (partial)** | Core modules production-quality; 3 modules have proof gaps but are not blocking other lanes |
| `chain:runtime` lane artifacts | **KEEP (reset context)** | Review correctly diagnosed the transplant failure; restart is the right answer |
| `chain:pallet` lane artifacts | **KEEP (reset context)** | Review correctly diagnosed 50+ errors across 3 waves; restart is the right answer |
| `games:poker-engine` lane artifacts | **KEEP** | Spec is coherent; crate is greenfield, which is expected for bootstrap |
| `games:multi-game` lane artifacts | **REOPEN** | Spec is sound; but the lane has a false-submit history that must be resolved |
| `foundations.fabro` workflow | **FIXED** | Was missing at session start; created in this run |
| Fabro/Raspberry execute/status/watch path | **NOT TRUSTWORTHY** | `games:multi-game` false-submit is the first confirmed instance of claim-without-delivery |

---

## Can `execute/status/watch` Be Trusted Today?

**No — not for lanes that have not produced artifacts via a verified Fabro run.**

The `games:multi-game` false-submit is the evidence. A previous Raspberry dispatch of the `games:multi-game` lane returned success (the lane milestone was marked complete) but the artifacts that the lane's `review.md` says should exist are either absent or were produced by a bootstrap agent running outside the proper Fabro workflow path.

The `foundations.fabro` workflow itself was missing from `fabro/workflows/bootstrap/` at the start of this session — meaning the current Fabro run started without its defined workflow graph. This is a second-order defect: the run started but the workflow it was supposed to execute did not exist on disk.

**The rule going forward**: Trust `execute/status/watch` only when:
1. The lane has a `review.md` with explicit KEEP judgment
2. The artifacts the lane claims to have produced are actually present at the named paths
3. The Fabro run that produced them used the correct workflow graph

---

## Critical Defect: `games:multi-game` False-Submit

**What happened**: Raspberry dispatched the `games:multi-game` lane. The lane's `multi-game.fabro` workflow has 5 nodes (`start → specify → review → polish → verify → exit`) with a `goal_gate=true` on the `verify` node. The `verify` node checks:
```
test -f outputs/games/multi-game/spec.md && test -f outputs/games/multi-game/review.md
```

The `review.md` at `outputs/games/multi-game/review.md` exists and was produced. However, it is unclear whether this was produced by:
- (a) A real Fabro run of `multi-game.fabro` that passed `verify`, or
- (b) A bootstrap agent that ran the spec+review work without the Fabro workflow being present

The `outputs/games/multi-game/spec.md` references `specsarchive/031626-06-multi-game-architecture.md` and other archived specs — it reads like a genuine bootstrap spec. But the `review.md` was clearly produced by an agent with access to the existing artifacts. The critical question is whether `verify` passed legitimately or whether the artifact-check is a no-op when artifacts already exist from a prior run.

**Required action**:
1. Run `fabro inspect` on the previous `games:multi-game` Fabro run and confirm whether `verify` actually ran and passed
2. If `verify` was never executed or always passes when artifacts exist (even from bootstrap), this is a `goal_gate` design flaw
3. After diagnosing, either fix the specific bug or accept that `games:multi-game` needs a clean re-run

**Evidence location**: `outputs/games/multi-game/review.md` lines 118–132 (false-submit judgment), Fabro run directory for `games:multi-game` lane (check `run.toml` graph path and `status.json`)

---

## The Two Explicit Frontier Tasks: Honest Assessment

### Task 1: Fix Raspberry/Fabro Defects Only When Discovered by Real Myosu Execution

**Verdict**: This is the correct operational posture. Do not speculate about Fabro/Raspberry internals.

The `games:multi-game` false-submit is the first real defect surfaced by execution. It was not discovered by reading code — it was discovered by noticing that a lane claimed success but the worktree didn't reflect it. This is exactly the kind of defect this task describes.

**The one exception**: The missing `foundations.fabro` workflow was discovered by reading `run.toml` and confirming the file didn't exist. This is a bootstrap setup defect, not a real execution defect, and was fixed in this session as a prerequisite.

**What "trustworthy" means**: `execute/status/watch` is trustworthy when it returns the actual state of the Fabro run, not a guessed state inferred from scanning run directories. The Fabro docs explicitly say run directory layout is internal — so the current Raspberry coupling to `latest_fabro_run_for_lane()` by directory scanning is a temporary bridge, not a durable foundation.

### Task 2: Convert `games:multi-game` False-Submit into Truthful Failure or Successful Live Run

**Verdict**: The lane needs a clean re-run through the proper Fabro path.

The current state:
- `outputs/games/multi-game/spec.md` exists — appears genuine but provenance is uncertain
- `outputs/games/multi-game/review.md` exists — appears genuine but provenance is uncertain
- `fabro/workflows/bootstrap/multi-game.fabro` exists and is well-formed
- `fabro/run-configs/bootstrap/multi-game.toml` exists (need to verify)
- `crates/myosu-games-liars-dice/` does NOT exist — the crate the lane is supposed to produce

The re-run should:
1. Confirm the current `spec.md` and `review.md` are honest products of `multi-game.fabro` or re-produce them
2. Begin Slice 1 of the implementation: create the `myosu-games-liars-dice` crate skeleton
3. Produce `implementation.md` and `verification.md` artifacts

**The repair prerequisite**: Before re-running `games:multi-game`, the Fabro/Raspberry detach path must be verified to not silently swallow lane failures. This can be confirmed by running a lane that is expected to fail (e.g., the `chain:pallet` restart which is guaranteed to fail on current code) and confirming the failure is reported honestly.

---

## Concrete Risks for the Next Execution Cycle

### Risk 1: False Positives from `goal_gate` When Artifacts Pre-exist
**Location**: `fabro/workflows/bootstrap/multi-game.fabro` line 18

The `verify` node checks `test -f outputs/games/multi-game/spec.md && test -f outputs/games/multi-game/review.md`. If these files already exist (from a bootstrap agent or a prior incomplete run), `verify` passes immediately without running the lane. This makes `goal_gate=true` meaningless for lanes whose artifacts can exist before the run.

**What must happen**: Either delete the existing artifacts before re-running the lane, or change the `verify` script to confirm the artifacts were produced by the current run (e.g., check git log for recent commits to those paths, or check the Fabro run metadata).

### Risk 2: `execute/status/watch` Infers Lane State from Directory Scanning
**Location**: Raspberry `latest_fabro_run_for_lane()` equivalent

The current Raspberry implementation infers which Fabro run corresponds to a lane by scanning `~/.fabro/runs/` and matching `run.toml` contents. This is fragile: it breaks if run directories are moved, renamed, or if multiple runs of the same lane produce different directory names.

**What must happen**: Use `fabro inspect` (the stable Fabro inspection surface) rather than directory scanning. This is the correct long-term fix, but for the next execution cycle, verify that the correct run directory is being used by cross-checking `fabro inspect --run-id <id> --json | jq .status`.

### Risk 3: Autodev-Branch Push Conflict Blocks Integration
**Location**: `detach.log` line 20-26

```
Failed to push autodev-live to origin: non-fast-forward
```

The current run's sandbox is clean (`sandbox_mode = "clean"` in `run.toml`) but the autodev-live branch on the remote has diverged. The integration strategy is `squash` targeting `origin/HEAD`, but the push is failing because the remote branch is ahead.

**What must happen**: Before the next `fabro run` completes and tries to push, either:
- `git pull --ff-only` on the `autodev-live` branch, or
- `git reset --hard origin/HEAD` on `autodev-live`, or
- Change the integration strategy to `rebase` instead of `squash`

This is not a code defect — it is an operator state problem. The divergence happened because a prior Fabro run made commits that were not integrated before the next run started.

### Risk 4: Two Lanes (`chain:runtime`, `chain:pallet`) Are Guaranteed to Fail on Current Code
**Location**: `outputs/chain/runtime/review.md` and `outputs/chain/pallet/review.md`

Both reviews conclude "restart from Phase 0" and "restart from Phase 1". Attempting to run these lanes with `cargo check` or `cargo build` on current code will fail. If these lanes are re-run without first implementing the restart work, they will produce honest failures — which is actually the correct behavior, not a defect.

**What must happen**: Run these lanes only after the restart work is done (workspace wiring for runtime; dep addition + broken-module deletion for pallet). Do not re-run them against the current broken code and expect success.

---

## Lane Trust Summary

| Lane | Trust Level | Milestone | Can Run Now? |
|------|-------------|-----------|--------------|
| `games:traits` | TRUSTED | `reviewed` | YES |
| `tui:shell` | PARTIAL | `specified` | YES (with proof gaps) |
| `chain:runtime` | RESET NEEDED | `runtime_reviewed` (judgment done) | NO — restart first |
| `chain:pallet` | RESET NEEDED | `pallet_reviewed` (judgment done) | NO — blocked on runtime |
| `games:poker-engine` | GREENFIELD | `specified` | YES — implement |
| `games:multi-game` | UNVERIFIED | `specified` | NO — fix false-submit first |
| `foundations` | BOOTSTRAPPING | This run | IN PROGRESS |

---

## Recommended Next Execution Order

1. **`games:traits` implementation lane** — unconditionally ready, zero risk
2. **`games:poker-engine` implementation lane** — greenfield, immediately ready
3. **`chain:runtime` restart lane** — immediately ready but will fail honestly (correct behavior)
4. **`games:multi-game` re-run** — only after false-submit diagnosis and Fabro detach verification
5. **`chain:pallet` restart lane** — only after `chain:runtime` restarts successfully
6. **`tui:shell` implementation lane** — after `games:traits` implementation is done (schema types)

---

## File Reference Index (This Lane)

| File | Role |
|------|------|
| `fabro/workflows/bootstrap/foundations.fabro` | This lane's workflow graph (created in this run) |
| `fabro/run-configs/bootstrap/foundations.toml` | This lane's run config (created in this run) |
| `outputs/foundations/foundation-plan.md` | This lane's plan artifact (this file's companion) |
| `outputs/foundations/review.md` | This file |
| `fabro/workflows/bootstrap/multi-game.fabro` | `games:multi-game` lane workflow |
| `outputs/games/multi-game/review.md` | Source of false-submit judgment |
| `outputs/chain/runtime/review.md` | Source of runtime reset judgment |
| `outputs/chain/pallet/review.md` | Source of pallet reset judgment |
| `outputs/games/traits/review.md` | Source of traits KEEP judgment |
| `outputs/tui/shell/review.md` | Source of tui:shell partial judgment |

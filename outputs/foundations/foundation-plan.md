# Foundations Frontier — Foundation Plan

## Lane Boundary

`foundations` is the **meta-execution frontier** for the myosu Fabro/Raspberry substrate. It owns:

- The trustworthiness of `execute`, `status`, and `watch` truth across all lanes
- The Fabro-to-Raspberry detach path (how Raspberry dispatches Fabro runs and collects results)
- The `games:multi-game` false-submit resolution (convert to truthful failure or live run)
- The defect-fix discipline: only repair Raspberry/Fabro defects when real Myosu execution surfaces them, then rerun the affected frontier to restore trustworthy truth

`foundations` does **not** own:

- Game logic implementation (owned by `games:traits`, `games:poker-engine`, `games:multi-game`)
- Chain/runtime restart (owned by `chain:runtime`, `chain:pallet`)
- Service bringup (owned by `services:miner`, `services:validator-oracle`)

---

## Platform-Facing Purpose

The foundations frontier delivers **trustworthy execution infrastructure**. Without it, every Fabro lane run carries unquantified doubt: did the run actually execute? Did the artifacts come from a real execution or a false-submit? The user-visible outcomes are:

- An operator can run `raspberry execute` and trust the result: success means success, failure means failure, with no false positives
- An operator can run `raspberry status` and get a faithful picture of what actually happened in each lane
- An operator can run `raspberry watch` and see real-time execution events, not a stale or fabricated status
- The `games:multi-game` lane has been resolved: either it ran successfully with real artifacts, or it failed with a truthful error — never a silent false-submit

---

## Current Frontier Tasks

### Task 1: Make `execute/status/watch` Truth Trustworthy

**Problem**: The current Raspberry/Fabro integration cannot be trusted to report what actually happened. The `games:multi-game` bootstrap produced `spec.md` and `review.md` artifacts, but the Fabro dispatch path for that lane is broken — the artifacts were written without a real Fabro execution run behind them. This means the current `execute/status/watch` pipeline has a false-positive defect.

**Evidence**: The `fabro/run-configs/platform/multi-game.toml` targets a workflow at `fabro/workflows/bootstrap/multi-game.fabro`, but the `verify` step checks for pre-existing artifact files (`test -f outputs/games/multi-game/spec.md`) rather than verifying a real Fabro run produced them. The `games:multi-game` artifacts exist, but not from a real execution — they were either manually created or produced by a previous broken dispatch path.

**What must happen**:

1. Diagnose the exact Fabro-to-Raspberry dispatch failure for the `games:multi-game` lane
2. Determine whether the dispatch path fails at the Raspberry→Fabro handoff or at the Fabro execution layer
3. Fix the identified defect (not a workaround — the root cause)
4. Rerun the `games:multi-game` lane with the repaired dispatch path
5. Verify the artifacts produced are from a real Fabro execution, not a false-submit
6. Extend the diagnosis to all other active lanes to confirm or exclude similar false-submit defects

**Proof**: After this task is complete:
- `raspberry status --manifest fabro/programs/myosu.yaml` reports lane states that match what actually happened in Fabro runs
- `raspberry execute --manifest fabro/programs/myosu.yaml --lane games:multi-game` runs the lane and reports success only when the artifacts were actually produced by that run
- No lane in `fabro/programs/myosu.yaml` reports success from a false-submit

---

### Task 2: Resolve `games:multi-game` False-Submit

**Problem**: The `games:multi-game` lane was submitted for execution and produced `spec.md` and `review.md` artifacts, but the submission path was broken — the artifacts came from a false-submit, not a real Fabro execution.

**What must happen**:

Option A — Truthful failure: If the Fabro dispatch is irreparably broken for this lane in the current environment, the lane should report a clear, actionable failure with a diagnosis, not silently produce artifacts.

Option B — Successful live run: If the dispatch is reparable, fix it and rerun the lane so that the artifacts are produced by a real, verifiable Fabro execution.

**Decision criterion**: After diagnosing the dispatch failure (Task 1), if the fix is a single known change (e.g., a correct run-config path, a working graph reference), pursue Option B. If the fix requires architectural changes to the Fabro/Raspberry integration, pursue Option A with a documented repair plan.

**Proof**: After this task is complete:
- `raspberry execute --manifest fabro/programs/myosu.yaml --lane games:multi-game` either fails with a truthful diagnostic, or succeeds with artifacts traceable to a real Fabro run id
- The lane's `review.md` is updated to reflect the actual execution result

---

## Relationship to Other Lanes

| Lane | Relationship |
|------|-------------|
| `games:traits` | Proven trustworthy — bootstrap and implementation both ran successfully via real Fabro executions. Serves as the reference for what a trustworthy lane looks like. |
| `games:multi-game` | The primary evidence of the false-submit defect. The foundations lane must resolve this lane first before the rest of the infrastructure can be trusted. |
| `games:poker-engine` | Bootstrap artifacts exist (`spec.md`, `review.md`); unknown whether they came from a real execution. Must be verified after Task 1 completes. |
| `tui:shell` | Bootstrap artifacts exist; unknown whether from real execution. Must be verified after Task 1 completes. |
| `chain:runtime` | Restart lane. Artifacts are placeholders. Not a false-submit risk — restart lanes start from empty. |
| `chain:pallet` | Restart lane. Artifacts are placeholders. Not a false-submit risk. |

---

## Concrete Steps

### Step 1: Inventory current lane artifact provenance

```bash
# For each lane with existing outputs/*/spec.md and outputs/*/review.md,
# determine whether they came from a real Fabro execution or a false-submit.

# Reference trustworthy: games:traits
# - outputs/games/traits/spec.md: created by bootstrap + implement Fabro runs
# - outputs/games/traits/review.md: created by bootstrap + implement Fabro runs
# - Evidence: cargo test -p myosu-games passes, git log shows Fabro-run commits

# Suspect false-submit: games:multi-game
# - outputs/games/multi-game/spec.md: exists
# - outputs/games/multi-game/review.md: exists
# - Evidence: fabro/run-configs/platform/multi-game.toml uses clean worktree mode
#   with a verify step that checks pre-existing files rather than run-produced files

# Suspect false-submit: games:poker-engine, tui:shell
# - Similar pattern to games:multi-game
```

### Step 2: Diagnose the `games:multi-game` dispatch failure

```bash
# Attempt to run the multi-game lane directly via Fabro
fabro run fabro/run-configs/platform/multi-game.toml

# Capture the full output and error stream
fabro run fabro/run-configs/platform/multi-game.toml 2>&1 | tee /tmp/multi-game-run.log

# Inspect the Fabro run directory for the run
ls -la ~/.fabro/runs/ | tail -5

# Check if the run actually executed the workflow or failed at dispatch
```

### Step 3: Fix the identified dispatch defect

The fix depends on the diagnosis. Common failure modes:

1. **Run-config graph reference mismatch**: The `graph = "..."` path in the TOML does not resolve to an existing `.fabro` file from the `directory` context.
2. **Clean worktree mode conflict**: The `sandbox.local.worktree_mode = "clean"` setting prevents the agent from writing to the working tree, which prevents artifact creation.
3. **Verify step logic bug**: The `verify` node in the workflow checks `test -f outputs/...` but the files may not exist in the clean worktree context.
4. **Raspberry→Fabro handoff**: Raspberry is not correctly translating lane execution into Fabro run invocations.

### Step 4: Rerun `games:multi-game` with repaired path

```bash
# After fixing the defect, rerun the lane
fabro run fabro/run-configs/platform/multi-game.toml

# Verify the artifacts were produced by this run
ls -la outputs/games/multi-game/
git log --oneline -3 outputs/games/multi-game/

# Confirm the run id is recorded in the artifact metadata
```

### Step 5: Verify other lanes for false-submit contamination

```bash
# For each lane with existing artifacts, re-run and compare
for lane in poker-engine tui-shell; do
    echo "=== Verifying $lane ==="
    fabro run fabro/run-configs/platform/${lane}.toml 2>&1 | head -20
done
```

---

## Acceptance Criteria

| # | Criterion | How to Verify |
|---|-----------|---------------|
| AC-01 | No lane reports success from a false-submit | All lanes with `spec.md`/`review.md` artifacts must have been produced by a verifiable Fabro run in the current session |
| AC-02 | `games:multi-game` has a truthful outcome | The lane either fails with a diagnostic or succeeds with artifacts traceable to a real Fabro run id |
| AC-03 | `execute/status/watch` report faithful truth | `raspberry status` for each lane matches what actually happened in the most recent Fabro run |
| AC-04 | Defect-fix discipline is established | No Raspberry/Fabro defect is fixed preemptively; all fixes are triggered by real Myosu execution failures |
| AC-05 | Affected frontiers are rerun after repair | After any Raspberry/Fabro defect is fixed, all frontiers affected by that defect are rerun |

---

## Idempotence

This plan is idempotent: Steps 1–5 can be run multiple times without causing drift. If a lane is already clean (artifacts from a real run), re-running it will produce the same artifacts and exit 0.

If Step 3 finds a dispatch defect that requires a code fix to the Fabro or Raspberry implementation itself, that fix should be applied to the `/home/r/coding/fabro/` source tree first, then this plan resumes from Step 4.

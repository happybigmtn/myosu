# Foundations Frontier — Review

## Keep / Reopen / Reset Judgment

**Judgment: KEEP (infrastructure repair mode)**

The foundations frontier is correctly scoped. The execution substrate is broken in a specific, diagnosable way: the Fabro-to-Raspberry dispatch path for several lanes has produced artifacts without real execution runs. The two current frontier tasks are the right first agenda items.

## Current Trust Assessment of All Lanes

| Lane | Artifacts Exist | Provenance | Status |
|------|----------------|------------|--------|
| `games:traits` | `spec.md`, `review.md`, `implementation.md`, `verification.md` | Real Fabro runs (bootstrap + implement) | **TRUSTED** |
| `games:multi-game` | `spec.md`, `review.md` | **Unknown — suspected false-submit** | UNTRUSTED |
| `games:poker-engine` | `spec.md`, `review.md` | **Unknown — suspected false-submit** | UNTRUSTED |
| `tui:shell` | `spec.md`, `review.md` | **Unknown — suspected false-submit** | UNTRUSTED |
| `chain:runtime` | `spec.md`, `review.md` | Placeholders (restart lane, starts empty) | NOT APPLICABLE |
| `chain:pallet` | `spec.md`, `review.md` | Placeholders (restart lane, starts empty) | NOT APPLICABLE |
| `sdk:core` | `spec.md`, `review.md` | Unknown | UNTRUSTED |
| `agent:experience` | `spec.md`, `review.md` | Unknown | UNTRUSTED |
| `play:tui` | `spec.md`, `review.md` | Unknown | UNTRUSTED |
| `learning:improvement` | `spec.md`, `review.md` | Unknown | UNTRUSTED |
| `operations:scorecard` | `spec.md`, `review.md` | Unknown | UNTRUSTED |
| `security:audit` | `spec.md`, `review.md` | Unknown | UNTRUSTED |
| `strategy:planning` | `spec.md`, `review.md` | Unknown | UNTRUSTED |

**Only `games:traits` has verified provenance.** All other lanes with artifacts are in UNTRUSTED or NOT APPLICABLE status.

---

## Evidence for the False-Submit Hypothesis

### `games:multi-game`

The `fabro/run-configs/platform/multi-game.toml` contains:

```toml
graph = "../../workflows/bootstrap/multi-game.fabro"
directory = "../../.."
[sandbox]
provider = "local"
[sandbox.local]
worktree_mode = "clean"
```

And the workflow's `verify` node:

```
verify  [label="Verify", shape=parallelogram,
         script="test -f outputs/games/multi-game/spec.md && test -f outputs/games/multi-game/review.md",
         goal_gate=true, max_retries=0]
```

**The problem**: `worktree_mode = "clean"` means the agent runs in a clean worktree. The `verify` step checks whether `spec.md` and `review.md` already exist. But if they already exist before the run starts (as they do now), the verify step passes — even though the files were not produced by this Fabro run. This is a classic false-positive gate: it proves the files exist, not that this run produced them.

**Consequence**: The `games:multi-game` artifacts were likely created by a previous agent that ran in a dirty worktree (or manually), and every subsequent run passes the verify gate without actually executing anything.

### `games:poker-engine` and `tui:shell`

Same pattern: `fabro/run-configs/platform/poker-engine.toml` and `fabro/run-configs/platform/tui-shell.toml` (if it exists) would have the same `worktree_mode = "clean"` + pre-existence verify pattern.

### The Root Cause

The `goal_gate = true` + `max_retries = 0` + pre-existence check is a verify step that was designed for lanes where artifacts are pre-committed. It does not verify that the current Fabro run produced the artifacts. This is a design bug in the workflow structure, not just in the run-config.

---

## What the Foundations Lane Must Do First

### Diagnosis (Task 1, Step 2)

Run the `games:multi-game` lane directly via Fabro and capture the full output:

```bash
fabro run fabro/run-configs/platform/multi-game.toml 2>&1 | tee /tmp/multi-game-diagnosis.log
echo "Exit code: $?"
ls -la outputs/games/multi-game/
cat /tmp/multi-game-diagnosis.log
```

**Expected findings** (one of):

1. **Fabro dispatch failure**: The `fabro run` command itself fails before any agent is spawned. This points to a problem in the run-config or graph resolution.
2. **Agent runs but artifacts are pre-existing**: The agent runs, but the `spec.md` and `review.md` files have timestamps predating the run. This confirms the false-submit.
3. **Agent creates artifacts fresh**: The artifacts have current timestamps and the run log shows the agent writing them. This would mean the false-submit hypothesis is wrong and the lane is actually clean.
4. **Agent fails to write artifacts**: The agent runs but fails to create the artifacts (e.g., due to clean worktree mode). This points to a sandbox configuration problem.

---

## Risks the Foundations Lane Must Preserve

1. **No preemptive fixes**: Do not fix any Raspberry/Fabro defect before it is triggered by a real Myosu execution failure. The discipline is: real execution failure → diagnose → fix → rerun.

2. **No false confidence from existing artifacts**: Having `spec.md` and `review.md` files in `outputs/` does not mean a lane is healthy. Provenance matters.

3. **Rerun after every fix**: After any Raspberry/Fabro defect is repaired, all affected frontiers must be rerun. A fix is not complete until the rerun confirms trustworthy truth.

---

## Risks the Foundations Lane Should Reduce

1. **No regression to `games:traits`**: The only trusted lane must not be disturbed by infrastructure work. Any Fabro or Raspberry changes must be tested against `games:traits` first to confirm no regression.

2. **No circular dependency**: Foundations lane cannot depend on any lane that is UNTRUSTED. It can only use `games:traits` as its reference for trustworthy behavior.

3. **No manual artifact creation**: The goal is truthful execution, not just the appearance of good artifacts. Do not manually create or edit artifacts to make a lane look clean.

---

## Is the Foundations Lane Ready for Implementation-Family Workflow Next?

**No — not yet.**

The foundations lane must first complete its diagnostic work. Once the false-submit defects are diagnosed and a repair strategy is established, the lane can proceed to an implementation-family workflow to fix the actual defects.

The prerequisite sequence is:

1. Diagnose `games:multi-game` dispatch failure (this review)
2. Determine if the fix is a run-config correction, a workflow graph correction, or a Raspberry code change
3. Fix the identified defect (may require a Fabro or Raspberry code change in `/home/r/coding/fabro/`)
4. Rerun `games:multi-game` with the repaired path
5. Verify all other UNTRUSTED lanes are either clean or have documented issues
6. Only then can the foundations lane move from "diagnosis" to "implementation"

---

## Relationship to Other Lanes

| Lane | Relationship |
|------|-------------|
| `games:traits` | The only trusted lane. Used as reference for what trustworthy execution looks like. Must not be regressed. |
| `games:multi-game` | Primary evidence of the false-submit defect. First lane to diagnose and repair. |
| All other UNTRUSTED lanes | Must be verified after `games:multi-game` is resolved. |
| Restart lanes (`chain:runtime`, `chain:pallet`) | Not at risk of false-submit (empty start state). Independent of the repair sequence. |

---

## File Reference Index

| File | Role |
|------|------|
| `outputs/foundations/foundation-plan.md` | This lane's implementation plan |
| `outputs/foundations/review.md` | This file |
| `fabro/workflows/bootstrap/multi-game.fabro` | Workflow with the flawed verify step |
| `fabro/run-configs/platform/multi-game.toml` | Run-config with clean worktree mode + pre-existence verify |
| `fabro/programs/myosu.yaml` | Root Raspberry program manifest |
| `outputs/games/traits/spec.md` | Reference for a trustworthy lane's spec |
| `outputs/games/multi-game/spec.md` | Suspected false-submit artifact |
| `outputs/games/multi-game/review.md` | Suspected false-submit artifact |

# foundations — Lane Review

## Judgment Summary

| Surface | Status | Rationale |
|---------|--------|-----------|
| `raspberry execute` (detach path) | **UNVERIFIED** | Detach emits run ID without confirming worker started; false-submit observed for `games:multi-game` |
| `raspberry execute` (foreground path) | **TRUSTED** | Direct Fabro foreground runs produce real manifests, states, and stage labels |
| `raspberry status` | **UNVERIFIED** for detach; **TRUSTED** for foreground | Detach-submitted runs show `submitted` indefinitely with no `run.pid`; foreground runs track correctly |
| `raspberry watch` | **UNVERIFIED** for detach; **TRUSTED** for foreground | Watch shows nothing for false-submit runs; real stage progression for foreground runs |
| `games:multi-game` false-submit | **HONEST FAILURE REQUIRED** | Must be converted to truthful failure or live success; current state is false-submit with no worker |
| `fabro run --detach` | **UNVERIFIED** | Returns run ID without confirming child worker process has started |

---

## Detailed Assessments

### raspberry execute (detach path) — UNVERIFIED

**Observation**: `raspberry execute` for `games:multi-game` returned run ID `01KM2BS4ASVRXVT2ND1GVVMKJ0` but the worker never started. The run directory has `status.json = submitted` and `detach.log` parse failure, but no `run.pid` and no manifest.

**Evidence**:
```
# Run directory state for 01KM2BS4ASVRXVT2ND1GVVMKJ0
status.json = "submitted"      # never progressed
detach.log = parse failure      # worker never started
run.pid = absent              # no worker process
manifest.toml = absent        # no lane manifest
```

**Risk**: Any lane submitted through the detach path cannot be trusted to represent a real running worker. `raspberry status` will report `submitted` forever, and `raspberry watch` will show nothing happening.

**Required Proof**:
1. A Fabro `--detach` submit must produce a `run.pid` in the run directory before the run ID is returned to Raspberry.
2. `raspberry status` for a detached submit must eventually transition from `submitted` to `running` or `completed`, not stay stuck at `submitted`.
3. `raspberry watch` for a detached submit must show real stage progression matching the actual worker state.

---

### raspberry execute (foreground path) — TRUSTED

**Observation**: Direct foreground Fabro runs for `games:multi-game` (`01KM2CGPHAJ95J38TQ7SPN46NZ`) and `validator:oracle` (`01KM2CGPCCC86SHEEQ6QFTRFEM`) produced real manifests, states, and stage labels. These runs completed successfully and their output artifacts are honest.

**Evidence**:
- `fabro inspect 01KM2CGPHAJ95J38TQ7SPN46NZ` shows a real lane execution with manifest, stages, and completion
- `fabro inspect 01KM2CGPCCC86SHEEQ6QFTRFEM` shows a real lane execution with manifest, stages, and completion
- Both produced real artifacts under their respective `outputs/` roots

**Correctness**: Foreground execution is the honest fallback path. Until the detach path is repaired, all important lane submissions should use this path.

---

### raspberry status — UNVERIFIED (detach) / TRUSTED (foreground)

**Observation**: For detach-submitted runs, `raspberry status` reports `submitted` indefinitely even though no worker is running. For foreground runs, `raspberry status` accurately reflects the actual Fabro run state.

**Claim vs Proof**:
- Detach claim: "submitted" means the run was dispatched to Fabro
- Detach proof: The run directory shows `status.json = submitted` but no worker process exists — the status is misleading
- Foreground claim: status reflects actual Fabro run state
- Foreground proof: `fabro inspect <run_id>` confirms the state matches what Raspberry reports

**Risk**: A contributor watching `raspberry status` for a detached submit will believe work is in progress when no worker exists. This blocks honest progress tracking and makes it impossible to trust the control plane for decision-making.

---

### raspberry watch — UNVERIFIED (detach) / TRUSTED (foreground)

**Observation**: For the false-submit run, `raspberry watch` showed no stage progression because no worker was running to generate stages. For foreground runs, `raspberry watch` shows real stage-by-stage truth.

**Claim vs Proof**:
- Detach claim: watch provides real-time visibility into running work
- Detach proof: watch shows nothing because there is no worker to watch — the run exists only as a status.json entry
- Foreground claim: watch shows real stage progression
- Foreground proof: watch output matches `fabro inspect` output for the same run

**Risk**: `raspberry watch` becomes a no-op for detach-submitted runs, which is worse than a visible failure — it presents an illusion of monitoring when there is nothing to monitor.

---

### games:multi-game false-submit — HONEST FAILURE REQUIRED

**Observation**: The `games:multi-game` lane was submitted through `raspberry execute` and returned a run ID, but the worker never started. This is a false-submit, not a lane failure.

**Claim vs Proof**:
- Raspberry claim: the lane was submitted and running
- Actual state: no worker process was started; the lane never ran
- The failure is in the control plane (detach path), not in the `games:multi-game` lane itself

**Risk**: If this false-submit is treated as a lane failure, the `games:multi-game` lane will be incorrectly diagnosed as broken when the actual problem is the control plane submit path.

**Required Action**: Convert this into a truthful result by either:
1. Running `games:multi-game` through direct foreground Fabro and documenting the honest result (success or real failure)
2. Fixing the detach path and rerunning through `raspberry execute` with the repaired path

---

### fabro run --detach — UNVERIFIED

**Observation**: The `--detach` flag in Fabro returns a run ID immediately without waiting to confirm the child worker process has started. The child can exit or fail to start, and the run ID has already been returned.

**Root Cause Hypothesis**: The Fabro detach implementation returns the run ID as soon as the background process is *spawned*, not as soon as the worker is *confirmed running*. The `run.pid` appears in the run directory only after the child process writes it, but the run ID is already returned to the caller before that happens.

**Risk**: Any Fabro or Raspberry code that trusts the returned run ID as evidence of a running worker will be wrong. The run ID only means "a background task was spawned", not "a worker is running".

**Required Proof**:
1. `fabro run --detach` must not return until `run.pid` exists in the run directory, or must return a run ID that is guaranteed to have a confirmed running worker.
2. If the child worker fails to start, the run ID must not be returned (or must be immediately invalidated).

---

## Submit Path Trust Inventory

| Lane | Submit Path | Trust Level | Evidence |
|------|-------------|-------------|----------|
| `games:traits` | Direct Fabro foreground | **TRUSTED** | Real run; real artifacts under `outputs/games/traits/` |
| `tui:shell` | Direct Fabro foreground | **TRUSTED** | Real run; real artifacts under `outputs/tui/shell/` |
| `chain:runtime` | Direct Fabro foreground | **TRUSTED** | Real run; real artifacts under `outputs/chain/runtime/` |
| `chain:pallet` | Direct Fabro foreground | **TRUSTED** | Real run; real artifacts under `outputs/chain/pallet/` |
| `games:multi-game` (first submit) | Raspberry detach | **BROKEN** | Run ID `01KM2BS4ASVRXVT2ND1GVVMKJ0`; no worker started |
| `games:multi-game` (second submit) | Direct Fabro foreground | **TRUSTED** | Run ID `01KM2CGPHAJ95J38TQ7SPN46NZ`; real artifacts |
| `validator:oracle` | Direct Fabro foreground | **TRUSTED** | Run ID `01KM2CGPCCC86SHEEQ6QFTRFEM`; real artifacts |
| `miner:service` | Direct Fabro foreground | **TRUSTED** | Real artifacts under `outputs/miner/service/` |
| `sdk:core` | Direct Fabro foreground | **TRUSTED** | Real artifacts under `outputs/sdk/core/` |

**Conclusion**: The foreground submit path is consistently trustworthy. The detach path is broken. All important lane submissions should use foreground execution until the detach path is repaired.

---

## Proof Deficiency Summary

| Surface | Proof Claim | Actual Proof | Gap Severity |
|---------|-------------|--------------|-------------|
| `fabro run --detach` | Returns run ID for a running worker | Returns run ID before worker confirmed started | **CRITICAL** |
| `raspberry execute` (detach) | Submits lane and starts worker | Submits lane but worker may never start | **CRITICAL** |
| `raspberry status` (detach) | Reports actual run state | Reports `submitted` forever even with no worker | **CRITICAL** |
| `raspberry watch` (detach) | Shows real-time stage progression | Shows nothing because no worker is running | **CRITICAL** |
| `games:multi-game` | Honest lane result | False-submit; no real lane execution occurred | **HIGH** |
| `fabro run --detach` | Blocks until worker confirmed | Returns immediately; worker confirmation is async | **HIGH** |

---

## Recommendation

**Reset the detach path to unverified until the following conditions are met:**

1. **Critical**: A Fabro `--detach` submit must not return a run ID until the worker process has a confirmed `run.pid` in the run directory.
2. **Critical**: `raspberry status` for any detached submit must transition from `submitted` to `running` when the worker confirms startup, not remain `submitted` forever.
3. **High**: `raspberry watch` for any detached submit must show real stage progression matching the actual worker state.
4. **High**: The `games:multi-game` false-submit must be converted to either a truthful lane failure (real error) or a successful live run.

**Interim guidance for contributors:**

- Use direct foreground Fabro for all lane submissions that matter.
- Do not trust `raspberry execute --detach` for any lane until the critical proof gaps above are closed.
- When a lane appears to be stuck at `submitted` in `raspberry status`, verify with `fabro inspect <run_id>` to determine whether a real worker is actually running.
- When a Fabro run fails to start, do not treat it as a lane failure until you have confirmed the lane code itself is the cause (as opposed to a control plane submit-path failure).

The foundations lane is **unblocked for honest work**: the false-submit is documented honestly, the foreground path is trusted, and the detach path is explicitly marked unverified with clear proof requirements for when it can be trusted again.

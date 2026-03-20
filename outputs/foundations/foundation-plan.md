# `foundations` Lane Spec

## Lane Boundary

`foundations` is the **bootstrap review and honest snapshot lane** for the myosu Fabro/Raspberry frontier. It does not own a crate — it owns the first honest assessment of what the current codebase actually contains, which execution paths are trustworthy, and what the next concrete steps are.

`foundations` does **not** own:
- Any product, chain, or service implementation
- Any new Fabro workflow or run config creation
- Any Raspberry manifest changes

## Purpose / User-Visible Outcome

After this lane lands, a contributor can open `outputs/foundations/review.md` and understand:
1. Which crates are actually trustworthy (compiles, tests pass)
2. Which execution paths have been proven vs. which are speculative
3. What the known Raspberry/Fabro defects are
4. What the honest next steps are based on real evidence, not assumptions

## Current State Summary

### Trusted Crate Surfaces

| Crate | Trust Level | Evidence |
|-------|-------------|----------|
| `myosu-games` | **TRUSTED** | Compiles, 10 unit tests pass, 4 doctests pass |
| `myosu-tui` | **TRUSTED** | Compiles, tests pass |
| Chain fork | **UNTRUSTED** | Does not compile; pallet transplant with missing dependencies |

### Fabro/Raspberry Execution Truth

| Path | Status | Evidence |
|------|--------|----------|
| Direct Fabro foreground | **WORKING** | `games:traits` implementation completed via direct `fabro run` |
| Raspberry detach submit | **BROKEN** | `games:multi-game` submitted as `01KM2BS4ASVRXVT2ND1GVVMKJ0` but no `run.pid` or manifest appeared |
| `raspberry execute` | **PARTIAL** | Can submit runs and get run IDs, but detach path produces false-submits |
| `raspberry status/watch` | **UNVERIFIED** | Depends on detach path working correctly |

### Known Defects

1. **Detach false-submit**: `raspberry execute` returns a run id but the worker never actually starts. Evidence: `games:multi-game` run `01KM2BS4ASVRXVT2ND1GVVMKJ0` shows `status.json=submitted` but no `run.pid`.

2. **MiniMax env isolation**: Non-interactive shells don't get `MINIMAX_API_KEY` / `ANTHROPIC_*` env vars because `.bashrc` returns early for non-interactive shells.

3. **Fabro run-truth bridge**: Raspberry infers live Fabro runs by scanning run directories, but Fabro docs say raw run directory layout is internal.

## The `games:multi-game` False-Submit

The `games:multi-game` lane was submitted through `raspberry execute` and received run id `01KM2BS4ASVRXVT2ND1GVVMKJ0`. However:
- No `run.pid` appeared in the run directory
- No manifest was written
- Only `status.json=submitted` and a `detach.log` parse failure were present

This is the **false-submit** referenced in the frontier tasks. It must be converted to either:
- A **truthful failure** (Raspberry correctly reports the run failed)
- A **successful live run** (the lane actually executes)

The direct Fabro foreground fallback succeeded for the same lane (`01KM2CGPHAJ95J38TQ7SPN46NZ`), producing real artifacts.

## Honest Next Steps

### Step 1: Verify the Detach Path

Before running more lanes through Raspberry, verify whether `fabro run --detach` actually starts workers or just writes `status.json=submitted`.

**Proof**: Run a simple lane through `fabro run --detach` and check if `run.pid` appears within 30 seconds.

### Step 2: Convert False-Submit to Truthful Failure or Live Run

The `games:multi-game` lane needs to be re-run through either:
- Direct Fabro foreground (if control-plane truth matters more than supervisory submit path)
- Raspberry with a fixed detach path (if supervisory submit path needs to work)

### Step 3: Trust `execute/status/watch` Before Relying On It

Only after Step 1 confirms the detach path works should `raspberry execute/status/watch` be trusted for lane dispatch.

## Proof / Check Shape

### Bootstrap proof (lane integrity check)

```bash
# Verify myosu-games is still trustworthy
cargo test -p myosu-games

# Verify no unexpected files were created
find outputs/foundations -type f | sort
```

### Milestone checks

| Milestone | Validates |
|-----------|-----------|
| `review.md` exists and is honest | Lane contract |
| `spec.md` exists and is accurate | Lane contract |
| Current branch truth is captured | Foundation |

## Relationship to Other Lanes

| Lane | Relationship |
|------|-------------|
| All lanes | `foundations` provides the honest baseline for all other lanes |
| `games:traits` | The only lane that is fully proven (bootstrap + implementation) |
| `games:multi-game` | The lane with the known false-submit that needs resolution |
| `chain:runtime`, `chain:pallet` | Currently untrusted - restart lanes that don't compile |
| All Fabro/Raspberry lanes | Depend on the detach path being fixed or workarounds being established |

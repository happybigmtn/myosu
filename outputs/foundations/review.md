# `foundations` Lane Review

## Lane Boundary

`foundations` is a **meta-lane** — it does not implement game-solving code. It produces the honest baseline artifacts for the Myosu frontier: an authoritative `foundation-plan.md` and this `review.md`. These artifacts capture the current state of all lanes, the known execution-layer defects in Fabro/Raspberry, and the path to trustworthy run-truth.

## Keep / Reopen / Reset Judgment

**Judgment: KEEP (foundational — must be kept accurate as the frontier evolves)**

The `foundations` lane is not a software feature lane. It is the honest accounting surface for the entire frontier. Its artifacts must be kept accurate. Any lane that claims completion without fixing the `games:multi-game` false-submit is operating on untrustworthy truth.

The foundations lane is never "done" in the sense that other lanes are done. It must be updated whenever:
- A lane completes an honest review
- A new Fabro/Raspberry execution defect is discovered
- A lane's `review.md` is updated with corrected judgments

## Concrete Defects the Implementation Must Preserve or Fix

### Defect 1: `games:multi-game` False Submit (Critical)

**Location**: `outputs/games/multi-game/review.md` claims "KEEP" but the task description explicitly calls this a "false-submit."

**What must happen**: The `games:multi-game` lane must be rerun with real execution. Until that rerun produces an honest `review.md`, the existing one is untrustworthy and must not be used as a basis for downstream lane decisions.

**Trustworthiness criteria**:
```bash
# A lane's status is trustworthy iff ALL of:
# 1. The proof command ran to completion (exit 0 or documented expected failure)
# 2. The Raspberry state was written with the correct status
# 3. The lane's artifacts reflect the actual execution result

# games:multi-game currently fails criterion 1 and 3.
# Until it is rerun, its review.md is not evidence of anything.
```

**Verification after fix**:
```bash
# Must show real execution
cargo test -p myosu-games-liars-dice
# All slice tests must exit 0 (see outputs/games/multi-game/review.md lines 22-56)
```

### Defect 2: No Raspberry State Files

**Location**: `.raspberry/` directory has no state files. `myosu.yaml` specifies `state_path: ../../.raspberry/myosu-state.json` but no such file exists.

**What must happen**: The Raspberry state path resolution must be verified. If the path is incorrect, fix the manifest. If the state has never been persisted, run `raspberry execute --manifest fabro/programs/myosu-bootstrap.yaml` and verify state is written.

**Verification after fix**:
```bash
# After running raspberry execute, this file must exist:
ls -la .raspberry/myosu-state.json  # or myosu-bootstrap-state.json
# And must contain valid JSON with lane status
```

### Defect 3: Two Program Manifests with Unclear Scope

**Location**: `fabro/programs/myosu.yaml` (7 units) vs `fabro/programs/myosu-bootstrap.yaml` (3 units)

**What must happen**: The bootstrap manifest should be explicitly labeled "current operational entrypoint." The full manifest should be labeled "aspirational target." The relationship must be documented in `foundation-plan.md` and referenced from both manifests.

**Verification after fix**:
```bash
# Both manifests should have a comment or label indicating their role
grep -n "operational\|aspirational\|bootstrap" fabro/programs/myosu-bootstrap.yaml
grep -n "operational\|aspirational\|bootstrap" fabro/programs/myosu.yaml
```

## Lane-by-Lane Trust Assessment

| Lane | Trust Level | Basis | Required Action |
|------|-------------|-------|-----------------|
| `games:traits` | **TRUSTED** | `cargo test -p myosu-games` passes, honest review complete | Continue; no action needed |
| `tui:shell` | **TRUSTED** | Review artifacts exist, tests pass | Continue; no action needed |
| `games:multi-game` | **UNTRUSTWORTHY** | False submit; review.md does not reflect real execution | Rerun with real execution before any downstream use |
| `chain:runtime` | **IN PROGRESS** | Bootstrap restart lane; honest review not yet complete | Complete review before chain:pallet can proceed |
| `chain:pallet` | **BLOCKED** | Depends on chain:runtime reviewed | Unblock by completing chain:runtime review |
| `games:poker-engine` | **ASPIRATIONAL** | Not started | Await trusted upstream lanes first |
| `services:miner` | **ASPIRATIONAL** | Not started | Await chain:pallet and games:poker-engine |
| `services:validator-oracle` | **ASPIRATIONAL** | Not started | Await chain:pallet and games:poker-engine |
| `product:play-tui` | **ASPIRATIONAL** | Not started | Await trusted upstream lanes |
| `platform:abstraction` | **ASPIRATIONAL** | Not started | Await chain:pallet |

## Proof Commands

| Context | Command | Expected |
|---------|---------|----------|
| Foundations artifacts exist | `ls outputs/foundations/` | `foundation-plan.md review.md` |
| `games:multi-game` rerun | `cargo test -p myosu-games-liars-dice` | All slice tests exit 0 |
| Raspberry state persists | After `raspberry execute`, `ls .raspberry/` | State JSON file present |
| Bootstrap manifest labeled | `grep "bootstrap\|operational" fabro/programs/myosu-bootstrap.yaml` | Role label found |

## Is the Frontier Ready for a Full Program Run?

**No — not until the following are resolved:**

1. **`games:multi-game` false submit is eliminated**: A real execution must produce a real `review.md`
2. **Raspberry state persistence is verified**: `.raspberry/` state files must exist after runs
3. **`chain:runtime` completes honest review**: The chain restart lanes must produce truthful review artifacts
4. **`chain:pallet` unblocks**: Depends on chain:runtime completion

Until these four items are resolved, any `raspberry execute --manifest fabro/programs/myosu.yaml` run will produce untrustworthy truth. The Fabro/Raspberry execution surface is the foundation of all downstream lane confidence.

## Relationship to Other Lanes

| Lane | Relationship |
|------|-------------|
| All lanes | Depend on foundations for honest baseline truth |
| `games:multi-game` | Must be repaired before depending on it |
| `chain:runtime` | Must complete before `chain:pallet` can unblock |
| `myosu.yaml` program | Aspirational target; `myosu-bootstrap.yaml` is current entrypoint |

## File Reference Index

| File | Role |
|------|------|
| `outputs/foundations/foundation-plan.md` | Honest baseline plan and state documentation |
| `outputs/foundations/review.md` | This file — trust judgments for all lanes |
| `outputs/games/multi-game/review.md` | **UNTRUSTWORTHY** — do not use without rerun |
| `fabro/programs/myosu.yaml` | Aspirational full program manifest (7 units) |
| `fabro/programs/myosu-bootstrap.yaml` | Current operational bootstrap manifest (3 units) |
| `.raspberry/` | Raspberry state directory — **currently empty** |

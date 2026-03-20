# `foundations` Lane Review

## Keep / Reopen / Reset Judgment

**Judgment: KEEP — lane is open, work is in progress**

The foundations lane is not a product lane. It is the execution infrastructure
lane. Its artifacts are not `spec.md` and `review.md` for a crate — they are
workflow fixes, run-config corrections, and confirmed truthfulness of the
`execute/status/watch` pipeline.

The lane is open because:
1. The `games:multi-game` false-submit has been diagnosed but not yet re-run
2. The `multi-game.fabro` workflow graph prompt misassignment has been
   identified but not yet fixed
3. No Raspberry-to-Fabro truthfulness audit has been performed yet

---

## Concrete Risks the Foundations Lane Must Preserve or Reduce

### Risk 1: `games:multi-game` False-Submit Evidence Is Still in `outputs/`
**Location**: `outputs/games/multi-game/spec.md`, `outputs/games/multi-game/review.md`

These artifacts claim the lane is KEEP with zero blockers and 22 proof
commands that would all fail. The `review.md` even says "Yes — with
conditions" and lists prerequisites — but those prerequisites are not met,
and the lane was marked KEEP anyway.

**What must be preserved:**
- The artifacts as evidence of the false-submit (do not delete before re-run)
- The `review.md` judgment text as proof that the false-submit happened

**What must be reduced:**
- The artifacts must be replaced by honest ones from a correctly configured
  re-run, or the existing artifacts must be annotated with a false-submit
  warning header

**Verification:**
After re-run, `fabro run fabro/run-configs/platform/multi-game.toml` must
produce new artifacts that are consistent with actual code state.

### Risk 2: `multi-game.fabro` Uses `review.md` as a Polish Prompt
**Location**: `fabro/workflows/bootstrap/multi-game.fabro` line 17

```fabro
polish  [label="Polish", prompt="@../../prompts/bootstrap/review.md", ...]
```

`review.md` is the prompt for reviewing already-produced artifacts. Using it as
a production prompt at the `polish` node is a category error. This is the
direct cause of the false-submit.

**What must be preserved:**
- Nothing — this is a misconfiguration, not a design choice

**What must be reduced:**
- The `polish` node must either be removed (if this is a bootstrap lane) or
  must use a real production prompt (`implement.md` or equivalent)
- The workflow graph must match the lane category: bootstrap lanes use
  `plan.md` → `review.md` → verify; platform/implement lanes use
  `plan.md` → `implement.md` → `review.md` → verify

**Verification:**
```bash
grep -E 'prompt="' fabro/workflows/bootstrap/multi-game.fabro
# Must not show review.md used as a production step
```

### Risk 3: `games:multi-game` Run-config Is in `platform/` But Workflow Is in `bootstrap/`
**Location**: `fabro/run-configs/platform/multi-game.toml` line 2

```toml
graph = "../../workflows/bootstrap/multi-game.fabro"
```

A platform lane using a bootstrap workflow graph is a structural mismatch.
Bootstrap graphs are for producing `spec.md` + `review.md` from existing code.
Platform graphs are for producing implementation artifacts from a spec.

**What must be preserved:**
- Nothing — the mismatch itself is the risk

**What must be reduced:**
- Either move the workflow to `workflows/platform/` with correct prompt chain,
  or reclassify the lane as bootstrap and fix the workflow graph

**Verification:**
After fix, `fabro run fabro/run-configs/platform/multi-game.toml` uses a
workflow graph in `workflows/platform/`, not `workflows/bootstrap/`.

### Risk 4: Raspberry `execute/status/watch` May Report Stale or False State
**Location**: `fabro/programs/myosu.yaml`, `.raspberry/` state directory

The false-submit of `games:multi-game` means Raspberry may have recorded a
successful lane completion for a lane that never produced working code. This
corrupts the lane state that `status` and `watch` report.

**What must be preserved:**
- The ability to query Fabro run truth directly via `fabro inspect`

**What must be reduced:**
- Any cached Raspberry state that reflects the false-submit
- The gap between what Raspberry reports and what `fabro inspect` confirms

**Verification:**
```bash
raspberry status --manifest fabro/programs/myosu.yaml
fabro inspect <latest-run-id> --format json  # compare lane results
# Both must agree; no false positives
```

### Risk 5: Other Workflow Graphs May Have the Same Prompt Misassignment
**Location**: All `fabro/workflows/**/*.fabro` files

If `multi-game.fabro` has this problem, other graphs may too.

**What must be reduced:**
- Audit all workflow graphs for prompt-to-node-category mismatches
- Fix any graph that uses `review.md` as a production step

**Verification:**
```bash
for graph in fabro/workflows/**/*.fabro; do
  prompts=$(grep -oE 'prompt="@[^"]+"' "$graph" | sort -u)
  echo "=== $graph ==="
  echo "$prompts"
done
# Each graph's prompts must match its category (bootstrap vs platform vs implement)
```

---

## Proof Commands

These are the commands that validate the foundations lane is complete:

| # | Context | Command | Expected |
|---|---------|---------|----------|
| 1 | False-submit re-run | `fabro run fabro/run-configs/platform/multi-game.toml` | Artifacts consistent with code state |
| 2 | Workflow graph fix | `grep 'review.md' fabro/workflows/bootstrap/multi-game.fabro` | No production-step usage |
| 3 | Category alignment | `grep 'workflows/bootstrap' fabro/run-configs/platform/*.toml` | Empty (no platform lanes using bootstrap graphs) |
| 4 | Truth alignment | `raspberry status --manifest fabro/programs/myosu.yaml` | No false-positive lane completions |
| 5 | Graph audit | All workflow graphs use prompts matching their lane category | Exit 0 |

---

## What Is Currently Broken (Unambiguous)

| # | Item | Location | Status |
|---|------|----------|--------|
| 1 | `multi-game.fabro` `polish` node uses `review.md` | `fabro/workflows/bootstrap/multi-game.fabro:17` | BROKEN |
| 2 | `games:multi-game` artifacts are false-positives | `outputs/games/multi-game/` | BROKEN |
| 3 | Platform lane uses bootstrap workflow graph | `fabro/run-configs/platform/multi-game.toml:2` | BROKEN |
| 4 | `games:multi-game` proof commands cannot pass | All 22 commands in `review.md` require missing crate | BROKEN |
| 5 | Raspberry state may reflect false-submit | `.raspberry/` | SUSPECT |

---

## File Reference Index

| File | Role |
|------|------|
| `fabro/workflows/bootstrap/multi-game.fabro` | Workflow graph with prompt misassignment |
| `fabro/run-configs/platform/multi-game.toml` | Run config that uses wrong-category workflow |
| `fabro/prompts/bootstrap/review.md` | Review prompt — correctly used at review step only |
| `fabro/prompts/bootstrap/implement.md` | Implement prompt — misapplied to review step |
| `outputs/games/multi-game/spec.md` | False-positive artifact (needs re-run) |
| `outputs/games/multi-game/review.md` | False-positive artifact (needs re-run) |
| `fabro/workflows/bootstrap/game-traits.fabro` | Reference: correct bootstrap workflow pattern |
| `fabro/programs/myosu.yaml` | Raspberry program manifest (lane truth source) |
| `.raspberry/` | Raspberry local state (may contain false-submit record) |

---

## Is the Execution Substrate Trustworthy?

**No — not yet.**

The `games:multi-game` false-submit is direct evidence that the Fabro workflow
system can produce artifacts that claim success without any underlying code
changes. Until the workflow graph is fixed and the re-run produces honest
artifacts, no Fabro-dispatched lane result can be trusted without independent
verification.

The path to trustworthiness:
1. Fix `multi-game.fabro` workflow graph prompt assignments
2. Re-run `games:multi-game` with truthful prompts
3. Audit all other workflow graphs for the same disease
4. Confirm `raspberry status` and `fabro inspect` agree on lane results
5. Only then: mark the foundations lane complete

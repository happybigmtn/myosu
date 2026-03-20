# `foundations` Lane Review

## Keep / Reopen / Reset Judgment

**Judgment: KEEP (contract-only, pre-implementation)**

The foundations lane is a newly created contract lane. It has `foundation-plan.md`
which defines the honest diagnostic and repair work needed for this slice. It has
not yet been executed — no Fabro run has been dispatched for this lane.

The lane is honest by construction: its job is to fix a false submit and make the
control plane trustworthy. It cannot cause harm by being in KEEP state because it
has no implementation power — only diagnostic and artifact-correction power.

## Implementation Lane Status

**Not applicable — this is a foundations/diagnostic lane, not an implementation lane.**

The lane does not produce code. It produces corrected control-plane artifacts:
- Updated `outputs/games/multi-game/review.md` with a truthful judgment
- Documented root cause of the `games:multi-game` false submit
- Documented state of `execute/status/watch` trustworthiness
- Documented repair actions (if any)

## Immediate Next Step

Run the diagnostic concrete steps from `foundation-plan.md`:

```bash
# Step 1: Confirm the false submit
ls crates/myosu-games-liars-dice/ 2>&1
# Expected: "No such file or directory"

cargo build -p myosu-games-liars-dice 2>&1
# Expected: "package not found"

# Step 2: Check product manifest for multi-game lane
grep -n "multi-game\|liars-dice\|LiarsDice" fabro/programs/myosu-product.yaml
# Expected: no output

# Step 3: Check Raspberry state
cat .raspberry/myosu-state.json 2>/dev/null | python -c "import json,sys; d=json.load(sys.stdin); print(d.get('lanes', {}).get('games:multi-game', 'not found'))" 2>/dev/null
# Expected: "not found" or error

# Step 4: Compare status command vs reality for a known-good lane
raspberry status --manifest fabro/programs/myosu-bootstrap.yaml --lane games:traits
# Compare with: cargo test -p myosu-games (should both report healthy)
```

## Concrete Findings

### Finding 1: `games:multi-game` Is a Contract-Only Lane

**Evidence**: `fabro/programs/myosu-product.yaml` does not define a `games:multi-game`
lane. The `myosu-product` program only has `play:tui` and `agent:experience` lanes.

**What this means**: The `outputs/games/multi-game/` artifacts (spec.md and review.md)
were produced by manual review, not by a Fabro execution lane. There is no
`fabro/run-configs/product/multi-game.toml` and no corresponding workflow graph.

**Impact**: The false submit was structurally guaranteed. Without a lane definition
in the manifest, Raspberry cannot execute or track the lane. The review artifact
exists but has no execution backing.

### Finding 2: `myosu-games-liars-dice` Crate Does Not Exist

**Evidence**:
```bash
$ ls crates/myosu-games-liars-dice/
ls: cannot access 'crates/myosu-games-liars-dice/': No such file or directory

$ grep "myosu-games-liars-dice" Cargo.toml
# (no output — not in workspace members)
```

**What this means**: The `games:multi-game` lane has a review artifact claiming the
implementation is KEEP, but the implementation crate has never existed. The review
judgment is factually incorrect.

**Impact**: The `outputs/games/multi-game/review.md` must be corrected to RESET
before any honest lane work can proceed.

### Finding 3: `games:traits` Lane Is the Trustworthy Reference

**Evidence**: `cargo test -p myosu-games` passes with 10 unit tests and 4 doctests.
The lane has a proper Fabro run config (`fabro/run-configs/bootstrap/game-traits.toml`)
and workflow graph (`fabro/workflows/bootstrap/game-traits.fabro`).

**What this means**: `games:traits` is the model for what an honest bootstrap lane
looks like. `games:multi-game` should follow the same pattern when it is re-opened.

### Finding 4: `execute/status/watch` Trustworthiness Is Unverified

**Evidence**: No diagnostic has been run to confirm that `raspberry status` returns
state consistent with actual code for any lane.

**What this means**: The task's requirement to make `execute/status/watch` truthful
has not been validated. The concrete steps in `foundation-plan.md` must be run to
establish the baseline.

### Finding 5: The Fabro Detach Path Has Not Been Tested for `games:multi-game`

**Evidence**: There is no `games:multi-game` lane in any active program manifest.
Therefore `raspberry execute --manifest fabro/programs/myosu-product.yaml --lane multi-game`
would fail with "lane not found".

**What this means**: The "repaired Fabro detach path" mentioned in the task cannot
be tested until `games:multi-game` has a proper lane definition.

## Risks the Implementation Lane Must Preserve or Reduce

### Risk 1: The `games:multi-game` Review Judgment Was Premature

**Exact location**: `outputs/games/multi-game/review.md` line 5

The judgment says **KEEP** but the implementation crate does not exist. This is
a false positive in the control plane.

**What must be preserved**: The spec at `outputs/games/multi-game/spec.md` is coherent
and should be kept as the lane's target.

**What must be reduced**: The review judgment must be corrected to RESET before the
lane is re-opened for implementation.

### Risk 2: No Execution Backing for `games:multi-game`

**Exact location**: `fabro/programs/myosu-product.yaml`

The product program manifest does not define a `games:multi-game` lane. Without a
lane definition, Fabro cannot execute the lane and Raspberry cannot track it.

**What must be preserved**: The existing spec and review artifacts.

**What must be reduced**: A proper lane definition (run config + workflow + manifest
entry) must be created when the lane is re-opened.

### Risk 3: `execute/status/watch` Truth Is Unverified

**Exact location**: Unknown — no diagnostic has been run

The task requires that `execute/status/watch` truth be trustworthy. This has not
been confirmed.

**What must be preserved**: The existing `games:traits` lane as a known-good reference.

**What must be reduced**: Run the diagnostic steps and document whether the status
commands report truthful state.

## Proof Commands

| Context | Command | Expected |
|---------|---------|----------|
| Diagnose false submit | `ls crates/myosu-games-liars-dice/` | No such file or directory |
| Diagnose false submit | `cargo build -p myosu-games-liars-dice` | package not found |
| Check lane definition | `grep "multi-game" fabro/programs/myosu-product.yaml` | no output |
| Check status command | `raspberry status --manifest fabro/programs/myosu-bootstrap.yaml --lane games:traits` | reports healthy |
| Compare with reality | `cargo test -p myosu-games` | 10 unit + 4 doctest pass |
| Foundation lane | `fabro run fabro/run-configs/foundations/diagnostic.toml` (once created) | diagnostic steps complete |

## File Reference Index

| File | Role |
|------|------|
| `outputs/foundations/foundation-plan.md` | This lane's ExecPlan |
| `outputs/foundations/review.md` | This file |
| `outputs/games/multi-game/review.md` | Contains the false KEEP judgment |
| `outputs/games/multi-game/spec.md` | Lane specification (coherent, should be preserved) |
| `fabro/programs/myosu-product.yaml` | Product program (missing multi-game lane) |
| `fabro/programs/myosu-bootstrap.yaml` | Bootstrap program (games:traits is healthy reference) |
| `crates/myosu-games-liars-dice/` | Does not exist |
| `crates/myosu-games/` | Trusted leaf crate; `GameType::LiarsDice` exists |
| `fabro/run-configs/bootstrap/game-traits.toml` | Model for proper lane run config |
| `fabro/workflows/bootstrap/game-traits.fabro` | Model for proper lane workflow |

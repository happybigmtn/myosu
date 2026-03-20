# Foundations Lane — Honest Execution Foundations

This ExecPlan establishes the honest execution foundations for the myosu Fabro/Raspberry
control plane. It is a living document governed by `PLANS.md`.

Reference: `PLANS.md` governs executable implementation plans. This document must be
maintained in accordance with those rules.

## Purpose / Big Picture

After this lane completes, the Fabro/Raspberry execution model for myosu produces
**truthful completion claims**. No lane may report a milestone as complete unless:

1. The code change actually exists in the worktree
2. The proof command actually executed and passed
3. The `execute/status/watch` surfaces reflect ground truth, not optimistic fiction

The `games:multi-game` lane currently has a curated `review.md` artifact but zero
implemented code — a **false submit**. This lane either converts that to a truthful
failure or a successful live run.

## Progress

- [ ] (2026-03-20) Document current Fabro/Raspberry execution surface and identify
  where `execute/status/watch` truth is not trustworthy
- [ ] (2026-03-20) Determine honest disposition of `games:multi-game` lane
  (truthful failure vs. live implementation)
- [ ] (2026-03-20) If live implementation: add `games:multi-game` lane to
  `fabro/programs/myosu-bootstrap.yaml` with honest proof gates
- [ ] (2026-03-20) If truthful failure: mark `games:multi-game` as closed-failed with
  evidence preserved
- [ ] (2026-03-20) Verify `execute/status/watch` surfaces report correct state for
  all bootstrap lanes
- [ ] (2026-03-20) Produce honest `review.md` artifact for foundations lane

## Surprises & Discoveries

- Observation: `outputs/games/multi-game/review.md` exists with "Judgment: KEEP" but
    `crates/myosu-games-liars-dice/` does not exist in the workspace
  Evidence: `ls crates/` shows only `myosu-games/`, `myosu-tui/`, `myosu-chain/`
  Implication: The lane has a curated artifact but no implemented code — any success
    claim from this lane would be a false positive

- Observation: `fabro/programs/myosu-bootstrap.yaml` has 4 units (games:traits,
    tui:shell, chain:runtime, chain:pallet) but no `games:multi-game` unit
  Evidence: `myosu-bootstrap.yaml` lines 6–121
  Implication: The `games:multi-game` lane exists as a curated artifact but was never
    added to the active program manifest

- Observation: INV-001 (Structured Closure Honesty) and INV-002 (Proof Honesty) are
    exactly the invariants violated by a lane with artifacts but no code
  Evidence: `INVARIANTS.md` lines 5–24

## Decision Log

- Decision: Foundations lane must be created as a first-class lane in the myosu
    bootstrap program before any other new lanes are added
  Rationale: The false-submit pattern (artifacts exist, code does not) undermines all
    downstream trust in the control plane. Honest foundations precede all other work.
  Date/Author: 2026-03-20

- Decision: `games:multi-game` disposition must be resolved before the foundations lane
    is considered complete
  Rationale: Leaving a false-submit in the curated outputs creates a poisoned
    milestone that will confuse future contributors and automation
  Date/Author: 2026-03-20

- Decision: `execute/status/watch` surfaces must be verified as truthful before any
    new lane is approved for the bootstrap program
  Rationale: Without trustworthy execution truth, no completion claim can be trusted
  Date/Author: 2026-03-20

## Outcomes & Retrospective

*To be written after lane completion.*

## Context and Orientation

### What is the Fabro/Raspberry execution model?

**Fabro** is the execution substrate — it runs workflows defined in
`fabro/workflows/`, governed by run configs in `fabro/run-configs/`. It maintains
run state in `.fabro/` directories.

**Raspberry** is the supervisory control plane layered on Fabro. It owns:
- Program manifests (`fabro/programs/*.yaml`) that define units, lanes, milestones,
  and proof gates
- Curated output artifacts under `outputs/` that survive across runs
- State tracking in `.raspberry/` that informs milestone progression

**The `execute/status/watch` surfaces** are the inspection APIs that report what
Fabro runs have done. If these surfaces report "success" when the actual worktree
has no code, the entire control plane is built on fiction.

### Key files for this lane

| File | Role |
|------|------|
| `fabro/programs/myosu-bootstrap.yaml` | Raspberry program manifest for bootstrap lanes |
| `outputs/games/multi-game/review.md` | The false-submit artifact that must be resolved |
| `outputs/games/multi-game/spec.md` | The spec companion to the false-submit review |
| `crates/` | All myosu crates — currently missing `myosu-games-liars-dice/` |
| `INVARIANTS.md` | INV-001 (Structured Closure Honesty), INV-002 (Proof Honesty) |
| `fabro/workflows/` | Fabro workflow definitions |
| `fabro/run-configs/` | Fabro run configurations |

### The false-submit problem

The `games:multi-game` lane produced a `review.md` that says "Judgment: KEEP" and
"Implementation lane unblocked: Yes". But the review itself documents that the
`myosu-games-liars-dice` crate does not exist. This is a textbook false submit:

1. A lane artifact exists (`review.md`)
2. The implementation code does not exist (`crates/myosu-games-liars-dice/` absent)
3. Any claim of "success" or "unblocked" is therefore false

The correct dispositions are:
- **Truthful failure**: close the lane with evidence of what was promised vs. what
  exists
- **Live implementation**: actually implement the crate and rerun

## Plan of Work

### Step 1: Audit the `execute/status/watch` surfaces

Inspect the Fabro run state and Raspberry state to determine what they currently
report about the `games:multi-game` lane. The goal is to understand what the
control plane *thinks* is true vs. what is actually true.

### Step 2: Determine `games:multi-game` honest disposition

Review the `games:multi-game` spec and review. The crate does not exist. The
question is: should this be a truthful failure (lane closed, evidence preserved) or
a live implementation task?

Given that:
- The spec already exists with detailed slice instructions
- The `CfrGame: Copy` constraint is a known technical risk
- The zero-change architectural claim is verifiable

The honest answer depends on whether there is willingness to implement the Liar's
Dice crate in this session. If not, the lane must be closed with a truthful
failure artifact.

### Step 3: Update `myosu-bootstrap.yaml` with honest lane configuration

If continuing with `games:multi-game` implementation: add it to the program manifest
with honest proof gates that require the crate to exist before claiming any
milestone.

If closing with failure: document the closure in the program manifest notes.

### Step 4: Verify all bootstrap lanes report truthful status

Run `raspberry status --manifest fabro/programs/myosu-bootstrap.yaml` and
compare against ground truth in the worktree. Any discrepancy is a defect in the
`execute/status/watch` surfaces.

### Step 5: Produce `outputs/foundations/review.md`

The review artifact must contain an honest judgment: is the execution model
trustworthy, and what specific defects were found and fixed?

## Concrete Steps

### Step 1 audit command

```bash
# Check what Raspberry thinks the lane state is
raspberry status --manifest fabro/programs/myosu-bootstrap.yaml 2>&1 || true

# Check Fabro run history for any games:multi-game runs
ls -la .fabro/ 2>/dev/null || echo "No .fabro/ directory"
ls -la .raspberry/ 2>/dev/null || echo "No .raspberry/ directory"

# Verify ground truth: does the crate exist?
ls crates/ | grep liars-dice || echo "myosu-games-liars-dice NOT FOUND"
```

### Step 2 disposition decision

The disposition is made by human judgment based on:
- Whether the Liar's Dice implementation will be attempted in this session
- Whether the cost of a truthful failure (re-opening later) is acceptable

If implementation is not happening now: produce a `games-multi-game-closed.md`
failure artifact and mark the lane as closed-failed in the program.

### Step 3 update command (if implementing)

```bash
# Add games:multi-game unit to myosu-bootstrap.yaml
# with honest proof gates:
#   - precondition: crates/myosu-games-liars-dice/src/lib.rs exists
#   - proof: cargo build -p myosu-games-liars-dice exits 0
```

### Step 4 verification command

```bash
raspberry status --manifest fabro/programs/myosu-bootstrap.yaml
# Compare each lane's reported state against:
# - games:traits: does crates/myosu-games/ exist? do tests pass?
# - tui:shell: does crates/myosu-tui/src/shell.rs exist?
# - chain:runtime: does crates/myosu-chain/runtime/src/lib.rs exist?
# - chain:pallet: does crates/myosu-chain/pallets/game-solver/src/lib.rs exist?
```

## Validation and Acceptance

The foundations lane is complete when:

1. `outputs/foundations/review.md` exists with an honest judgment
2. `outputs/foundations/foundation-plan.md` exists with current progress (this file)
3. The `games:multi-game` lane disposition is resolved (either implemented or
   truthfully closed-failed)
4. `raspberry status` output matches worktree ground truth for all bootstrap lanes
5. No bootstrap lane has artifacts but missing code (the false-submit pattern)

## Idempotence and Recovery

This lane is idempotent: running it multiple times with the same inputs produces the
same artifacts. If the lane fails partway through, restart from the last incomplete
step.

If `games:multi-game` is closed-failed but later needs to be reopened: create a new
lane artifact `games-multi-game-reopened.md` with fresh disposition and update the
program manifest.

## Artifacts and Notes

*To be populated as lane work proceeds.*

## Interfaces and Dependencies

This lane depends on:

| Dependency | Role |
|------------|------|
| `fabro/programs/myosu-bootstrap.yaml` | Program manifest to inspect and potentially update |
| `outputs/games/multi-game/review.md` | The false-submit artifact to resolve |
| `outputs/games/multi-game/spec.md` | The spec companion |
| `INVARIANTS.md` | INV-001 and INV-002 are the foundation of honest execution |
| `PLANS.md` | Governs this plan's format and maintenance |
| `SPEC.md` | Governs spec authoring conventions |

This lane produces:

| Artifact | Path |
|----------|------|
| Foundation plan | `outputs/foundations/foundation-plan.md` |
| Foundation review | `outputs/foundations/review.md` |
| Optional: closed-failure artifact | `outputs/games-multi-game-closed.md` (if closing) |

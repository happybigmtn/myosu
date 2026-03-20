# Foundations Lane — First Honest Reviewed Slice

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Reference: PLANS.md governs executable implementation plans.

## Purpose / Big Picture

Establish an honest, trustworthy execution foundation for the Myosu frontier. The core problem is that `execute/status/watch` truth has become unreliable due to Raspberry/Fabro defects and a false-submit in the `games:multi-game` lane. This lane bootstraps the first slice where:

1. Defects are only fixed when discovered by **real Myosu execution**, not speculation
2. The `games:multi-game` false-submit becomes either a truthful failure or a successful live run
3. The Fabro detach path is repaired and rerunnable

After this change, a contributor can run `fabro run` and `raspberry execute` and trust the output they see.

## Progress

- [x] (2026-03-20) Assess current `execute/status/watch` truth posture
- [x] (2026-03-20) Identify Raspberry/Fabro defects affecting truth trustworthiness
- [x] (2026-03-20) Locate `games:multi-game` false-submit and assess repair path
- [ ] Establish first honest execution baseline
- [ ] Convert false-submit to truthful outcome
- [ ] Repair Fabro detach path
- [ ] Verify `execute/status/watch` truth is trustworthy post-repair

## Surprises & Discoveries

- Observation: The `games:multi-game` lane produced a submit without a corresponding successful live run — a false-positive in the control plane.
- Evidence: `outputs/games/multi-game/review.md` notes the lane status but execution truth does not match artifact claim.
- Observation: Raspberry/Fabro `execute/status/watch` surfaces are not yet stable enough to serve as single source of truth.
- Evidence: No `.raspberry/` state files found in current worktree; bootstrap program manifest points to state_path that does not exist.
- Observation: The bootstrap program (`myosu-bootstrap.yaml`) defines 3 units (games, tui, chain) with lane-level milestones but no foundations unit.
- Evidence: `fabro/programs/myosu-bootstrap.yaml` lines 1–122 show the program structure; `outputs/` has no `foundations/` directory.

## Decision Log

- Decision: Create a dedicated `foundations` lane as the first honest slice for this frontier.
  Rationale: The task explicitly requires bootstrapping an honest reviewed slice. A foundations lane provides the structural home for truth-injection into the control plane.
  Date/Author: 2026-03-20 / foundations lane bootstrap

- Decision: Fix Raspberry/Fabro defects only when discovered by real Myosu execution.
  Rationale: Speculative fixes without real execution evidence risk introducing new defects. The task gate is explicit on this point.
  Date/Author: 2026-03-20 / foundations lane bootstrap

- Decision: Repair the `games:multi-game` false-submit by converting it to a truthful failure or successful live run.
  Rationale: A false-positive in the control plane undermines trust in all downstream milestone decisions. The repair must be honest, not cosmetic.
  Date/Author: 2026-03-20 / foundations lane bootstrap

## Outcomes & Retrospective

*To be written after first honest slice completes.*

## Context and Orientation

### What is the Foundations Lane?

The foundations lane is the trust anchor for the Myosu control plane. It lives at `outputs/foundations/` and is not yet registered in any program manifest — it is being bootstrapped as a new structural slice.

### Key Files and Paths

| Path | Role |
|------|------|
| `fabro/programs/myosu.yaml` | Repo-wide Raspberry control-plane entrypoint (top-level program) |
| `fabro/programs/myosu-bootstrap.yaml` | Bootstrap program with games/tui/chain units |
| `outputs/games/multi-game/review.md` | Multi-game lane review — contains the false-submit evidence |
| `outputs/games/traits/review.md` | Reference pattern for review document structure |
| `.raspberry/` | Raspberry runtime state (currently absent — needs investigation) |
| `fabro/run-configs/bootstrap/` | Bootstrap run configs (game-traits, tui-shell, chain-runtime, chain-pallet) |

### Key Concepts

**False-submit:** A lane that recorded a submit milestone without a corresponding successful live run. This is a control-plane integrity failure — the artifact claims success that did not occur.

**Fabro detach path:** The mechanism by which a Fabro workflow cleans up after itself. If broken, stale state pollutes subsequent runs and corrupts `execute/status/watch` truth.

**`execute/status/watch` truth:** The output of `raspberry execute --manifest X` and `raspberry status --manifest X` and the watch mode. If these surfaces lie, no milestone decision is trustworthy.

## Plan of Work

The foundations lane proceeds in three phases:

**Phase 1 — Truth Assessment:** Examine the current `execute/status/watch` posture. Determine which defects are real (discovered by actual Myosu execution) vs. theoretical. Identify the exact false-submit in `games:multi-game`.

**Phase 2 — Honest Repair:** Fix only the defects that real execution has revealed. Convert the `games:multi-game` false-submit into either a documented truthful failure (with failure mode understood) or a successful live run (if execution can be recovered).

**Phase 3 — Detach Path Repair:** Repair the Fabro detach path to ensure clean run boundaries. Verify that `execute/status/watch` truth is trustworthy after the repair.

## Concrete Steps

### Phase 1: Truth Assessment

**Step 1.1 — Examine multi-game false-submit**

Read `outputs/games/multi-game/review.md` and `outputs/games/multi-game/spec.md` to understand what the lane claimed vs. what actually executed.

```bash
cat outputs/games/multi-game/review.md
cat outputs/games/multi-game/spec.md
```

**Step 1.2 — Check for .raspberry state**

```bash
ls -la .raspberry/ 2>/dev/null || echo "No .raspberry directory found"
find .raspberry -type f 2>/dev/null | head -20
```

**Step 1.3 — Run execute/status to assess current truth posture**

```bash
cargo run --manifest-path /home/r/coding/fabro/Cargo.toml -p raspberry-cli -- plan --manifest fabro/programs/myosu.yaml 2>&1 | head -50
```

### Phase 2: Honest Repair

**Step 2.1 — Document the false-submit honestly**

If the multi-game lane failed, the review must reflect the failure, not a false success. Update `outputs/games/multi-game/review.md` to reflect the actual execution outcome.

**Step 2.2 — Fix only execution-proven defects**

Do not speculate. Only fix defects that `raspberry execute` or `raspberry status` reveals as actual failures.

### Phase 3: Detach Path Repair

**Step 3.1 — Identify detach failures**

Look for signs of Fabro detach failures: stale lock files, orphaned processes, incomplete cleanup between runs.

**Step 3.2 — Verify truth trustworthiness**

After repairs, run a clean `fabro run` and `raspberry execute` cycle and verify that the output matches the actual execution state.

## Validation and Acceptance

### Acceptance Criteria

1. `outputs/foundations/review.md` exists with honest assessment of current truth posture
2. The `games:multi-game` false-submit has been converted to either:
   - A truthful **failure** with documented root cause, OR
   - A **successful live run** with verifiable execution evidence
3. `execute/status/watch` truth is trustworthy — what the control plane reports matches what actually ran
4. The Fabro detach path is clean — no stale state from previous runs pollutes new executions

### Proof Commands

| Context | Command | Expected |
|---------|---------|----------|
| Foundations lane bootstrap | `ls outputs/foundations/` | `foundation-plan.md`, `review.md` |
| Multi-game lane assessment | `cat outputs/games/multi-game/review.md` | Honest outcome (not false success) |
| Raspberry state check | `ls .raspberry/` | State files present after execute |
| Execute truth check | `raspberry execute --manifest fabro/programs/myosu.yaml` | Output matches actual execution |

## Idempotence and Recovery

- All changes to `outputs/foundations/` are additive and can be rerun without damage
- If a repair attempt fails, document the failure honestly and revert to known-good state
- Do not delete `games:multi-game/review.md` even if the lane failed — the honest failure record is the artifact

## Artifacts and Notes

### Current False-Submit Evidence

The `games:multi-game` lane at `outputs/games/multi-game/` contains:
- `spec.md` — lane specification
- `review.md` — lane review (status to be confirmed as honest)
- `.gitkeep` — directory marker, no actual output artifacts

The presence of only a `.gitkeep` in a lane directory that claims a reviewed milestone is a red flag for false-submit.

### Bootstrap Program Structure

```
myosu-bootstrap.yaml units:
  - games (traits lane)
  - tui (shell lane)
  - chain (runtime lane, pallet lane)
```

No foundations unit exists yet in the bootstrap program. This lane creates that structural home.

## Interfaces and Dependencies

### Inputs (read-only, preserved)

- `README.md` — repo entrypoint and architecture overview
- `SPEC.md` — spec writing rules and doctrine
- `PLANS.md` — executable plan writing rules and doctrine
- `AGENTS.md` — agent roles and responsibilities (to be migrated)
- `specs/031626-00-master-index.md` — canonical doctrine index
- `specs/031826-fabro-primary-executor-decision.md` — Fabro-as-executor decision

### Outputs (created by this lane)

- `outputs/foundations/foundation-plan.md` — this document
- `outputs/foundations/review.md` — honest review of foundations state

### Dependencies

- `fabro/programs/myosu-bootstrap.yaml` — bootstrap program manifest (will need foundations unit addition)
- `outputs/games/multi-game/review.md` — false-submit evidence source
- `.raspberry/` — Raspberry state directory (must be created by execute)

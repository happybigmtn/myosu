# `foundations` Lane Review

## Keep / Reopen / Reset Judgment

**Judgment: REOPEN**

The foundations lane is correctly identified as the execution substrate lane, and the two critical defects (`execute/status/watch` truth untrustworthy, `games:multi-game` false-submit) are correctly diagnosed. However, no concrete implementation slices have been executed yet. The artifacts in this lane are currently a specification exercise, not a reviewed operational state.

The lane must **reopen** until at least one of the critical defects is demonstrably fixed by a real Fabro run with verified truthful reporting.

---

## Implementation Lane Unblocked?

**No — blocked on Slice 1 (fabro inspect stability).**

The foundations lane cannot proceed to implementation until `fabro inspect` produces stable, machine-readable output that Raspberry can consume as the authoritative run-truth source. Without this, every downstream slice (Raspberry status adapter, execute binding, watch, detach) would be built on an untrustworthy foundation.

---

## Concrete Risks the Foundations Lane Must Preserve or Reduce

### Risk 1: Fabro Inspect Output Gap
**Exact location**: Fabro `fabro inspect` command — current output format is not documented as machine-readable stable API.

`fabro inspect` may produce human-readable text output today. If the output format is not stable JSON with run id, status, artifacts, and timestamp, Raspberry cannot reliably consume it.

**What must be preserved**:
- The `fabro inspect` command itself (it exists and is the right abstraction)
- The intent to use Fabro's own inspection surface rather than run-directory scanning

**What must be reduced**:
- Raspberry's current `latest_fabro_run_for_lane()` heuristic that scans `~/.fabro/runs/`
- Any code that reads `run.toml` directly for run state

### Risk 2: games:multi-game Milestone Is Incorrectly Complete
**Exact location**: `fabro/programs/myosu-platform.yaml` lines 22–23

```yaml
- id: multi_game_reviewed
  requires: [multi_game_spec, multi_game_review]
```

The `multi_game_reviewed` milestone is satisfied because `outputs/games/multi-game/spec.md` and `outputs/games/multi-game/review.md` both exist. However, `myosu-games-liars-dice` crate does not exist. The milestone is satisfied on paper but not in reality.

**What must be preserved**:
- The milestone model (artifacts as the unit of lane completion)
- The dependency chain from `poker_engine_reviewed` to `multi_game_reviewed`

**What must be reduced**:
- Milestone satisfaction that depends only on file existence, not content validation
- The `games:multi-game` lane's current milestone state in `myosu-platform.yaml`

### Risk 3: Fabro Workflow Verify Steps Only Check File Existence
**Exact location**: `fabro/workflows/bootstrap/multi-game.fabro` line 18

```fabro
verify  [label="Verify", shape=parallelogram, script="test -f outputs/games/multi-game/spec.md && test -f outputs/games/multi-game/review.md", goal_gate=true, max_retries=0]
```

The `verify` step passes if both artifact files exist, regardless of their content accuracy or whether the underlying code exists.

**What must be preserved**:
- The verify step as a gate before lane completion
- The `goal_gate=true` semantics (the verify step must pass for the run to be considered successful)

**What must be reduced**:
- The `test -f` only check — it should be augmented with content validation
- `max_retries=0` on verify — if verify fails, the lane should not report success

### Risk 4: Raspberry Detach Path Not Explicitly Modeled
**Exact location**: All bootstrap workflow graphs (e.g., `fabro/workflows/bootstrap/game-traits.fabro`)

All bootstrap workflows end at `verify -> exit`. There is no explicit step that commits artifacts to the repo and updates Raspberry milestone state. This means the "detach" behavior is implicit and likely handled by Raspberry scanning for artifacts after the run — which is exactly the behavior that causes false-submits.

**What must be preserved**:
- The artifact-based milestone model (artifacts are the contract)
- The separation between Fabro run state and Raspberry control plane state

**What must be reduced**:
- Implicit artifact commit after run completion — should be explicit in the workflow

### Risk 5: No Content-Level Validation in Bootstrap Workflows
**Exact location**: `fabro/prompts/bootstrap/plan.md`, `fabro/prompts/bootstrap/implement.md`, `fabro/prompts/bootstrap/review.md`

The bootstrap prompts direct the agent to write `spec.md` and `review.md` artifacts but do not include explicit checks that the described code actually exists and passes its tests. An agent can produce a well-written spec for a non-existent crate.

**What must be preserved**:
- The prompt-based bootstrap approach (it works for `games:traits` which has real code)
- The separation between plan/implement/review prompts

**What must be reduced**:
- Prompt guidance that does not include code-presence checks as part of the bootstrap review

---

## Proof Commands

| Context | Command | Expected |
|---------|---------|----------|
| Fabro inspect stability | `fabro inspect --last-run --format json` | Valid JSON with `run_id`, `status`, `artifacts`, `timestamp` |
| Raspberry status truth | `raspberry status --manifest fabro/programs/myosu.yaml --format json` | JSON matching actual Fabro run state from `fabro inspect` |
| games:multi-game honest state | `raspberry status --manifest fabro/programs/myosu-platform.yaml --lane games:multi-game` | `failed` or `blocked` (not `succeeded` with missing code) |
| Fabro detach check | `./fabro/checks/fabro-detach.sh` | Exit 0; confirms outputs/ artifacts match last run |
| Execute truth binding | `raspberry execute --manifest fabro/programs/myosu.yaml --lane games:traits --format json` | `success` field matches `fabro inspect --run-id <id> --status` |

---

## File Reference Index

| File | Role |
|------|------|
| `fabro/workflows/bootstrap/game-traits.fabro` | Reference bootstrap workflow; verify only checks file existence (same pattern as multi-game) |
| `fabro/workflows/bootstrap/multi-game.fabro` | Defective bootstrap workflow; verify passes despite missing `myosu-games-liars-dice` crate |
| `fabro/run-configs/platform/multi-game.toml` | Run config for `games:multi-game`; references the defective workflow |
| `fabro/programs/myosu-platform.yaml` | Platform program manifest; `multi_game_reviewed` milestone incorrectly satisfied |
| `fabro/programs/myosu-bootstrap.yaml` | Bootstrap program manifest; `games:traits` milestone correctly satisfied |
| `fabro/programs/myosu.yaml` | Top-level program-of-programs; used for `execute/status/watch` truth tests |
| `outputs/games/multi-game/spec.md` | Exists but describes a crate that does not exist |
| `outputs/games/multi-game/review.md` | Exists but marks the lane complete when the crate is missing |
| `outputs/games/traits/spec.md` | Reference successful spec; `games:traits` implementation lane confirmed accurate |
| `outputs/games/traits/review.md` | Reference successful review; keep/reopen/reset judgment is file-specific |
| `fabro/checks/games-traits.sh` | Working proof script; model for `fabro-detach.sh` and `fabro-inspect.sh` |
| `outputs/README.md` | Correct outputs convention; `spec.md` + `review.md` for bootstrap lanes |

---

## Relationship to Other Lanes

| Lane | Relationship |
|------|-------------|
| `games:traits` | Trusted. Its successful implementation proves the bootstrap workflow shape is correct. Serves as the reference pattern for what trustworthy lane completion looks like. |
| `games:multi-game` | Directly broken. Must be reset. Currently reports `multi_game_reviewed` milestone satisfied when `myosu-games-liars-dice` crate is missing. |
| `chain:runtime`, `chain:pallet` | Restart lanes. Cannot trust their milestone evaluations until `execute/status/watch` is fixed. Their proof scripts (`chain-runtime-reset.sh`, `chain-pallet-reset.sh`) are present but their status reporting is untrustworthy. |
| `services:miner`, `services:validator-oracle` | Future lanes. Will be dispatched via Raspberry. Cannot be trusted until foundations are repaired. |

---

## Is the Lane Ready for an Implementation-Family Workflow Next?

**No — the foundations lane itself must be bootstrapped first.**

The `foundations` lane is a meta-lane: it fixes the infrastructure that other lanes use. Until `execute/status/watch` truth is established, no implementation-family workflow can be trusted to produce honest results.

The first honest implementation step is Slice 1: confirm that `fabro inspect` produces stable JSON output. If it does not, this must be fixed in Fabro itself before Raspberry can be adapted.

If `fabro inspect` is stable, Slice 2 (Raspberry status adapter) can proceed in parallel with the `games:multi-game` truthful reset (Slice 3).

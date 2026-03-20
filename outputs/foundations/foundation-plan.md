# `foundations` Lane Spec

## Lane Boundary

`foundations` is the **execution truth surface lane** for the myosu frontier. It does not own any product crate — it owns the supervisory infrastructure that makes all other lane status trustworthy:

- The Raspberry program manifest (`fabro/programs/myosu-bootstrap.yaml`) and its lane definitions
- The Fabro run-config surfaces that dispatch work (`fabro/run-configs/bootstrap/`)
- The check scripts that produce proof signals (`fabro/checks/`)
- The `outputs/` artifact conventions that survive across runs
- The execution truth bridge between Fabro run-state and Raspberry program state

`foundations` does **not** own:
- Any game crate (`myosu-games`, `myosu-games-poker`, etc.)
- Any service binary (`myosu-miner`, `myosu-validator`, etc.)
- The chain runtime or pallet code
- Any product feature

## Platform-Facing Purpose

The foundations lane delivers **honest supervisory truth**. The user-visible outcomes are:

- A contributor can run `raspberry status --manifest fabro/programs/myosu-bootstrap.yaml` and see a truthful representation of what each lane has actually produced
- A lane that fails produces a truthful failure record, not a false success
- The Fabro execution plane produces artifacts that survive across runs and are visible to the control plane
- The `games:multi-game` lane's false-submit is resolved into either a truthful failure (because `myosu-games-liars-dice` does not exist) or a successful live run

## How Surfaces Fit Together

```
Fabro execution plane (run state under .fabro/):
  fabro/workflows/bootstrap/*.fabro     ← workflow graphs
  fabro/run-configs/bootstrap/*.toml     ← run configurations
  fabro/checks/*.sh                     ← proof/check scripts

Raspberry control plane (survives across runs):
  fabro/programs/myosu-bootstrap.yaml   ← program manifest (lane definitions)
  .raspberry/myosu-state.json          ← derived state (does not exist yet)
  outputs/                             ← curated lane artifacts (survives)
    <lane>/spec.md                     ← lane contract
    <lane>/review.md                   ← trust assessment
    <lane>/implementation.md           ← what was changed
    <lane>/verification.md             ← proof result

Current execution truth sources:
  - Fabro run directory (per-run, ephemeral)
  - Raspberry program state (cross-run, derived from Fabro)
  - outputs/ curated artifacts (cross-run, human-maintained)
```

## Current Execution Truth State

### What Exists

| Surface | Location | Status |
|---------|----------|--------|
| Program manifest | `fabro/programs/myosu-bootstrap.yaml` | Present, valid YAML, defines 4 units with lanes |
| Run configs | `fabro/run-configs/bootstrap/*.toml` | Present for `game-traits`, `tui-shell`, `chain-runtime-restart`, `chain-pallet-restart` |
| Check scripts | `fabro/checks/*.sh` | Present: `games-traits.sh`, `tui-shell.sh`, `chain-runtime-reset.sh`, `chain-pallet-reset.sh` |
| Workflow graphs | `fabro/workflows/bootstrap/*.fabro` | Not yet inspected |
| Raspberry state | `.raspberry/` | **Does not exist** — no run has produced state yet |
| `outputs/` artifacts | `outputs/*/review.md` | 12 review files exist across multiple lanes |

### What Is Broken

1. **`.raspberry/` state directory missing**: No Fabro run has produced Raspberry-readable state. The `state_path: ../../.raspberry/myosu-state.json` in the program manifest points to a file that has never been written.

2. **`games:multi-game` false-submit**: The `outputs/games/multi-game/spec.md` and `outputs/games/multi-game/review.md` artifacts exist, but the underlying crate `crates/myosu-games-liars-dice/` does not exist. These artifacts were created speculatively — they describe a lane that has never been executed truthfully. The `games:multi-game` lane in the program manifest (`fabro/programs/myosu-bootstrap.yaml`) is not defined, which means it was never in a bootstrap run.

3. **No `foundations` unit in program manifest**: The `fabro/programs/myosu.yaml` has no `foundations` unit. This lane has no formal program entry.

### The `games:multi-game` False-Submit Problem

**Root cause**: The `outputs/games/multi-game/` artifacts were written as if the lane had executed, but:

- `crates/myosu-games-liars-dice/` does not exist in the workspace
- `Cargo.toml` workspace members do not include `myosu-games-liars-dice`
- The `games:multi-game` lane is not defined in `fabro/programs/myosu-bootstrap.yaml`
- No Fabro run has ever dispatched `games:multi-game` work

**Consequence**: The `games:multi-game` artifacts represent a speculative future state, not a truthful execution record. A contributor reading `outputs/games/multi-game/review.md` might reasonably conclude the lane has been attempted, when in fact it has never been dispatched.

**Resolution options**:
- **Option A (Reset)**: Delete `outputs/games/multi-game/` and recreate it only after a truthful successful run or truthful failure
- **Option B (Annotate)**: Keep the artifacts with explicit "FALSE-SUBMIT — never executed" banner until the lane runs truthfully
- **Option C (Trust the spec, distrust the review)**: The spec is correct; the review should be marked "pending execution" rather than "keep"

## Current Proven Status

### `games:traits` — TRUSTED (proven by bootstrap run)

The only lane with honest execution proof:

```bash
$ cargo test -p myosu-games
running 10 tests
  test traits::tests::game_type_from_bytes_custom    ... ok
  test traits::tests::game_config_nlhe_params        ... ok
  test traits::tests::game_type_from_bytes_known     ... ok
  test traits::tests::game_config_serializes          ... ok
  test traits::tests::game_type_to_bytes_roundtrip   ... ok
  test traits::tests::game_type_num_players          ... ok
  test traits::tests::reexports_compile               ... ok
  test traits::tests::strategy_response_probability_for  ... ok
  test traits::tests::strategy_response_validates     ... ok
  test traits::tests::strategy_query_response_roundtrip ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured

running 4 doctests
  traits::GameType::num_players (line 120)  ... ok
  README.md usage example (line 20)         ... ok
  traits::GameType::from_bytes (line 75)   ... ok
  traits::GameType::to_bytes (line 101)    ... ok

test result: ok. 4 passed
```

### `games:multi-game` — UNEXECUTED (false submit)

No Fabro run has ever dispatched this lane. The artifacts are speculative.

### `tui:shell`, `chain:runtime`, `chain:pallet` — UNKNOWN

The check scripts exist but have not been run with Raspberry tracking. The code exists in the expected locations, but no honest execution record exists.

## Smallest Honest Slices

### Slice 1 — Establish Raspberry State Directory

**Problem**: `.raspberry/` does not exist. No Fabro run has ever written state to `../../.raspberry/myosu-state.json`.

**What**: Create the `.raspberry/` directory structure. Verify the program manifest's `state_path` is writable.

**Proof**:
```bash
mkdir -p .raspberry
touch .raspberry/myosu-state.json
# Verify the path matches what the manifest expects
```

### Slice 2 — Annotate `games:multi-game` False-Submit

**Problem**: `outputs/games/multi-game/` artifacts imply execution that never happened.

**What**: Add a prominent header to `outputs/games/multi-game/review.md` and `outputs/games/multi-game/spec.md` marking them as "UNEXECUTED — awaiting first honest run."

**Alternative**: Delete the speculative artifacts and recreate after first honest run.

**Decision needed**: Reset (delete) or Annotate (keep with banner)?

### Slice 3 — Define `foundations` Unit in Program Manifest

**Problem**: The `foundations` lane has no formal entry in `fabro/programs/myosu.yaml`.

**What**: Add a `foundations` unit with a `program` lane to the program manifest. This lane would track its own status.

**Proof**: `fabro/programs/myosu.yaml` contains `foundations` unit after edit.

### Slice 4 — First Honest `games:multi-game` Dispatch

**Problem**: The lane has never been truthfully dispatched.

**What**: Run `fabro run fabro/run-configs/bootstrap/game-traits.toml` equivalent for `games:multi-game`. Observe whether it produces a truthful failure (crate missing) or a live run.

**Decision trigger**: If Fabro reports success but the code doesn't exist, the Fabro dispatch logic itself has a defect. If Fabro reports failure with clear error, the execution truth surface is working.

### Slice 5 — Verify `execute/status/watch` Truth

**Problem**: The task description says "until `execute/status/watch` truth is trustworthy again."

**What**: After slices 1–4, run `raspberry status --manifest fabro/programs/myosu-bootstrap.yaml` and verify the output reflects actual execution state, not speculative artifacts.

## What the Foundations Lane May Change First

The foundations lane **may change first**:

1. **Program manifest additions** — adding new units, lanes, or milestones
2. **Check script corrections** — fixing incorrect proof commands
3. **Artifact annotation** — marking false-submits honestly
4. **Raspberry state initialization** — creating the state directory structure

The foundations lane must **not change first without coordination**:

1. **Deleting other lanes' artifacts** — only annotate, don't destroy without consensus
2. **Changing proof commands for other lanes** — those lanes own their proof profiles
3. **Modifying Fabro run-configs that other lanes depend on** — coordinate with affected lanes

## Dependency Order

```
foundations (this lane)
  │
  ├── Slice 1: Raspberry state directory
  │
  ├── Slice 2: games:multi-game annotation decision
  │
  ├── Slice 3: foundations unit in program manifest
  │
  ├── Slice 4: first honest games:multi-game dispatch
  │
  └── Slice 5: execute/status/watch truth verification
```

## Relationship to Other Lanes

| Lane | Relationship |
|------|-------------|
| `games:traits` | Trusted reference lane. `foundations` must not break its proof commands. |
| `games:multi-game` | Primary beneficiary of `foundations` work. False-submit resolution is the trigger for honest execution. |
| `tui:shell` | Unproven like `games:multi-game`. `foundations` should apply same false-submit audit. |
| `chain:runtime`, `chain:pallet` | Unproven. Same audit applies. |
| All other lanes | All lane artifacts should be audited for false-submit patterns during `foundations` work. |

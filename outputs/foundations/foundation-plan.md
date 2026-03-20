# `foundations` Lane Spec

## Lane Boundary

`foundations` is the **execution substrate lane** for the myosu Fabro/Raspberry control plane. It owns the health and correctness of the infrastructure that all other lanes depend on:

- **Fabro execution plane**: workflow graphs (`fabro/workflows/`), run configs (`fabro/run-configs/`), prompt files (`fabro/prompts/`), and check scripts (`fabro/checks/`)
- **Raspberry control plane**: program manifests (`fabro/programs/*.yaml`), lane milestone definitions, proof profiles, and check definitions
- **Execute/status/watch surfaces**: the Raspberry commands that read Fabro run state and report lane health
- **Fabro-to-Raspberry detach path**: the mechanism by which a completed Fabro run's artifacts are incorporated into the permanent repo structure and reported to Raspberry
- **Outputs structure**: the curated artifact surface under `outputs/` that Raspberry consumes for milestone evaluation

`foundations` does **not** own:

- Any product crate code (those belong to their respective lanes: `games:traits`, `chain:runtime`, etc.)
- The robopoker fork or subtensor fork (those are owned by their respective migration lanes)
- The `specs/`, `plans/`, or `ops/` directory content (those are doctrine/lane artifacts)

`foundations` is unique among lanes: it is the only lane whose customers are other lanes, not end users or validators. Every other lane breaks if the execution substrate is broken.

---

## Platform-Facing Purpose

The foundations lane delivers **trustworthy execution**. The user-visible outcomes are:

- A contributor can run `raspberry execute` and trust the result — the reported success or failure accurately reflects what happened in the Fabro run
- A contributor can run `raspberry status` and trust the lane state — the milestone health signals are derived from real Fabro run truth, not stale or fabricated data
- A contributor can run `raspberry watch` and see live Fabro run progress that actually corresponds to work happening on disk
- When a Fabro run completes, its artifacts are cleanly detached from the run directory and incorporated into `outputs/` with correct path attribution
- The `games:multi-game` lane's current false-submit is resolved: the lane either reports truthful failure or produces a genuinely successful live run

---

## Currently Trusted Inputs

### Fabro Core Assets

| File | Trust Signal |
|------|-------------|
| `fabro/workflows/bootstrap/game-traits.fabro` | Passes `fabro inspect`; run produced verified `outputs/games/traits/spec.md` and `outputs/games/traits/review.md`; `games:traits` implementation lane succeeded |
| `fabro/workflows/bootstrap/tui-shell.fabro` | Passes `fabro inspect`; lane preconditions and proof commands are defined |
| `fabro/run-configs/bootstrap/game-traits.toml` | Parses cleanly with `tomllib`; run succeeded end-to-end |
| `fabro/run-configs/bootstrap/tui-shell.toml` | Parses cleanly with `tomllib`; run configured correctly |
| `fabro/prompts/bootstrap/plan.md` | Produces `spec.md`-shaped output matching `SPEC.md` conventions |
| `fabro/prompts/bootstrap/review.md` | Produces `review.md`-shaped output with keep/reopen/reset judgment |
| `fabro/prompts/bootstrap/implement.md` | Produces `review.md`-shaped output; drives the review step in bootstrap workflows |
| `fabro/prompts/bootstrap/fix.md` | Provides fix-lane guidance for broken bootstrap runs |
| `fabro/checks/games-traits.sh` | Executes `cargo test -p myosu-games`; exit 0 confirmed on HEAD |
| `fabro/checks/tui-shell.sh` | Executes `cargo test -p myosu-tui`; exit 0 confirmed on HEAD |

### Raspberry Program Manifests

| File | Trust Signal |
|------|-------------|
| `fabro/programs/myosu-bootstrap.yaml` | Produces lane milestone state; `games:traits` reviewed milestone confirmed real; `chain:runtime` and `chain:pallet` restart lanes correctly modeled |
| `fabro/programs/myosu.yaml` | Top-level program-of-programs; units and lanes match the frontier structure; `max_parallel: 2` correctly limits concurrent runs |

### Outputs Structure

| File | Trust Signal |
|------|-------------|
| `outputs/README.md` | Correctly describes the outputs convention: `spec.md` + `review.md` for bootstrap lanes, `implementation.md` + `verification.md` for implementation lanes |
| `outputs/games/traits/spec.md` | Produced by successful Fabro run; `games:traits` implementation lane confirmed it as accurate foundation |
| `outputs/games/traits/review.md` | Produced by successful Fabro run; keep/reopen/reset judgment is specific and file-referenced |

---

## Current Broken / Missing Surfaces

### Critical: `execute/status/watch` Truth Is Untrustworthy

**Symptom**: Raspberry's `execute`, `status`, and `watch` commands report lane state that does not match what Fabro runs actually produce. This was identified as the primary blocker for further lane execution.

**Root cause (suspected)**: Raspberry infers live Fabro runs by scanning run directories and matching `run.toml` contents. Fabro's own documentation (`docs/reference/run-directory.mdx`) explicitly warns that raw run-directory layout is an internal detail, not a durable external contract. Raspberry's current `latest_fabro_run_for_lane()` heuristic reads from `~/.fabro/runs/` which is an internal Fabro path, not a stable inspection surface.

**Impact**: Every lane milestone evaluation that depends on `execute/status/watch` is unreliable. The `games:multi-game` false-submit is a downstream symptom of this root cause.

**What must happen**: Replace Raspberry's run-directory scanning with Fabro's stable `fabro inspect` command as the primary run-truth source. This requires:
1. Fabro `fabro inspect` command produces stable machine-readable output (JSON) with run id, status, artifact paths, and timestamps
2. Raspberry's lane state adapter reads from `fabro inspect` rather than scanning `~/.fabro/runs/`
3. The `execute/status/watch` commands are re-bound to the new adapter

### Critical: `games:multi-game` False-Submit

**Symptom**: The `games:multi-game` lane was dispatched by Raspberry (via `myosu-platform.yaml`), a Fabro run completed, and artifacts appeared at `outputs/games/multi-game/spec.md` and `outputs/games/multi-game/review.md`. However, the lane's own review document identifies that the lane's own crate (`myosu-games-liars-dice`) does not exist — the artifacts were produced without the underlying code existing.

**Root cause**: The `multi-game.fabro` workflow's `verify` step only checks file existence (`test -f outputs/games/multi-game/spec.md && test -f outputs/games/multi-game/review.md`). It does not validate that the artifacts are accurate representations of actual code state. A Fabro agent can produce a well-written `spec.md` and `review.md` even when the underlying crate is missing.

**Impact**: The lane is marked as having produced reviewed artifacts when no actual implementation work exists. The `games:multi-game` milestone (`multi_game_reviewed` in `myosu-platform.yaml`) reports success without the lane actually being complete.

**What must happen**: The `games:multi-game` lane must be reset. Either:
- Option A: the lane is re-run with a corrected workflow that includes real code-presence checks before the `review` step succeeds
- Option B: the lane is marked as truthfully failed until the greenfield `myosu-games-liars-dice` crate exists

### High: Fabro Detach Path Is Broken

**Symptom**: When a Fabro run completes, its artifacts (the curated outputs that should survive to `outputs/`) are not cleanly incorporated into the repo. The `games:multi-game` false-submit is evidence of this: artifacts appeared without corresponding code.

**Root cause**: The Fabro-to-Raspberry detach path is not explicitly modeled. The workflow graphs show `verify -> exit` but do not include an explicit `detach` step that confirms artifacts are committed to the repo and Raspberry state is updated accordingly.

**What must happen**: The bootstrap workflow family needs an explicit `detach` step that:
1. Confirms all artifact files exist at the expected `outputs/` paths
2. Confirms the artifact content passes content-level validation (not just file existence)
3. Updates Raspberry's lane state to reflect the completed milestone

---

## Code Boundaries and Deliverables

### Fabro Execution Plane Additions

```
fabro/
  workflows/
    bootstrap/
      foundations.fabro      # Bootstrap workflow for this lane itself
  checks/
    fabro-inspect.sh         # Check: fabro inspect returns valid JSON for last run
    rpi-status.sh            # Check: raspberry status shows correct lane states
    fabro-detach.sh          # Check: outputs/ artifacts match last run's produced files
```

### Raspberry Control Plane Additions

```
fabro/programs/
  myosu-foundations.yaml    # Program manifest for foundations lane
```

### Outputs Structure Additions

```
outputs/
  foundations/
    foundation-plan.md       # This file (spec artifact)
    review.md               # Review artifact
```

---

## Proof / Check Shape

### Foundations Bootstrap Proof

```bash
# Fabro inspect produces valid JSON
fabro inspect --last-run --format json | python -c "import sys, json; json.load(sys.stdin); print('valid')"

# Raspberry status is consistent with Fabro run state
raspberry status --manifest fabro/programs/myosu.yaml --format json | python -c "import sys, json; data = json.load(sys.stdin); assert 'lanes' in data; print('valid')"

# outputs/ artifact paths match what the last Fabro run produced
./fabro/checks/fabro-detach.sh

# Execute/status/watch truth test
raspberry execute --manifest fabro/programs/myosu-platform.yaml --lane games:multi-game --format json
# The 'success' field must match whether the lane's proof commands actually pass
```

### Critical Defect Proofs

```bash
# The execute/status/watch truth test
raspberry status --manifest fabro/programs/myosu.yaml --lane games:traits --format json
# Must show 'games:traits' lane state matching actual Fabro run state
# If fabro inspect shows the run failed, raspberry status must show failed (not stale success)

# games:multi-game false-submit resolution test
raspberry status --manifest fabro/programs/myosu-platform.yaml --lane games:multi-game --format json
# Must show 'failed' OR show the lane succeeded with real code present (myosu-games-liars-dice crate exists)
# Must NOT show 'succeeded' with artifacts but no code
```

---

## Next Implementation Slices (Smallest Honest First)

### Slice 1 — `fabro inspect` Stability (highest priority)

**Problem**: Raspberry cannot reliably read Fabro run state because it scans run directories instead of using `fabro inspect`.

**What must happen**:
1. Verify `fabro inspect --last-run --format json` produces stable, machine-readable output
2. Identify any gaps between what `fabro inspect` returns and what Raspberry's lane adapter needs
3. If `fabro inspect` output is insufficient, file a Fabro issue or implement the missing fields

**Proof**: `fabro inspect --last-run --format json` returns valid JSON with `run_id`, `status`, `artifacts`, and `timestamp` fields.

---

### Slice 2 — Raspberry `status` Adapter to `fabro inspect`

**Problem**: `raspberry status` reads run state incorrectly because it uses run-directory scanning.

**What must happen**: Bind Raspberry's `status` command to `fabro inspect` output instead of `~/.fabro/runs/` scanning.

**Proof**: `raspberry status --manifest fabro/programs/myosu.yaml --format json` output matches `fabro inspect --last-run --format json` for the same run id.

---

### Slice 3 — `games:multi-game` Truthful Reset

**Problem**: The `games:multi-game` lane's artifacts exist but the underlying crate is missing. The lane reported success without real work.

**What must happen**: Determine whether `games:multi-game` should:
- Option A: Re-run with proper proof gates that fail if `myosu-games-liars-dice` crate is missing
- Option B: Mark the lane as blocked/failed until the crate exists

The decision depends on whether the false-submit is a Raspberry-level bug (submitting without checking code presence) or a Fabro workflow design gap (the verify step should have caught missing code).

**Proof**: After reset, `raspberry status --lane games:multi-game` must show an honest state (failed or in-progress) that matches reality.

---

### Slice 4 — Fabro `execute` Truth Binding

**Problem**: `raspberry execute` dispatches a Fabro run but the reported result may not match the actual run outcome.

**What must happen**: Bind `raspberry execute` to authoritative Fabro run ids from `fabro inspect`. The `execute` command should not return until the Fabro run completes and its status is confirmed by `fabro inspect`.

**Proof**: After running `raspberry execute --manifest fabro/programs/myosu.yaml --lane games:traits`, the reported `success` field matches `fabro inspect`'s `status` field for that run id.

---

### Slice 5 — Fabro `watch` Live Progress

**Problem**: `raspberry watch` shows progress that may not correspond to actual work on disk.

**What must happen**: `raspberry watch` should consume Fabro's run event stream (if available) or poll `fabro inspect` at short intervals to render accurate live progress.

**Proof**: While a Fabro run is active, `raspberry watch` output shows step names and timestamps that match the actual work being performed.

---

### Slice 6 — Explicit Fabro-to-Raspberry Detach Step

**Problem**: The workflow graphs end at `verify` without an explicit detach step that commits artifacts to the repo and updates Raspberry state.

**What must happen**: Add a `detach` step to each bootstrap workflow that:
1. Confirms artifact content validity (not just file existence)
2. Commits artifacts to the repo (if using a worktree model)
3. Updates Raspberry's milestone state to `complete`

**Proof**: After a successful Fabro run, `outputs/<lane>/` contains valid artifacts and `raspberry status` shows the lane's milestone as complete.

---

## Dependency Order

```
Slice 1 (fabro inspect stability)
  │
  ├──► Slice 2 (raspberry status adapter) — must precede slices 3-6
  │      │
  │      ▼
  │    Slice 4 (raspberry execute binding)
  │      │
  │      ▼
  │    Slice 5 (raspberry watch live progress)
  │
  ├──► Slice 3 (games:multi-game truthful reset) — independent of slice 2
  │      (can proceed once slice 1 confirms fabro inspect is reliable)
  │
  ▼
Slice 6 (explicit detach step)
  (must be last — depends on slices 1-5 being stable)
```

---

## Relationship to Other Lanes

| Lane | Relationship |
|------|-------------|
| All lanes | Every lane is blocked by the foundations defects. Until `execute/status/watch` truth is fixed, no lane can trust its milestone evaluation. |
| `games:multi-game` | Directly affected by the false-submit. Resolved only after slice 2 (Raspberry status adapter) confirms truthful reporting. |
| `games:traits` | Trusted example lane. Its successful implementation proves the bootstrap workflow shape is correct — the defect is in the Raspberry run-truth layer, not in the Fabro workflow layer. |
| `chain:runtime`, `chain:pallet` | Restart lanes. Their progress depends on `execute/status/watch` being trustworthy so that restart verification can be trusted. |
| `services:miner`, `services:validator-oracle` | Future lanes that will be dispatched via Raspberry. Cannot be trusted until foundations are repaired. |

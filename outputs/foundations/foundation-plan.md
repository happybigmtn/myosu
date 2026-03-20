# Foundations Lane — Execution Plan

This ExecPlan is a living document. The sections `Progress`, `Surprises &
Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up
to date as work proceeds.

`PLANS.md` is checked into the repository root and this document must be
maintained in accordance with it.

## Purpose / Big Picture

The foundations lane exists to make the Fabro/Raspberry execution and
control plane trustworthy enough to serve as the basis for all other Myosu
lanes. Two root defects currently block this:

1. **Run-truth brittleness**: `execute/status/watch` truth in Raspberry is
   derived by scanning raw Fabro run directories — an internal layout that
   Fabro's own docs explicitly warn against. This produces false-positive
   status signals and makes detached-submit handoffs unreliable.

2. **False submit on `games:multi-game`**: A Raspberry-dispatched
   `games:multi-game` run reported success but was a false positive. The
   Fabro detach path is broken, and the lane must either produce a truthful
   failure or a verified successful run.

After this lane lands, `execute/status/watch` reports lane truth derived from
stable Fabro inspection surfaces rather than directory scanning, and every
frontier lane can be rerun with confidence that the outcome reflects real
execution.

## Progress

- [ ] (2026-03-20) Bootstrap the foundations lane with honest `spec.md` and
  `review.md` artifacts.
- [ ] Diagnose the `games:multi-game` false-submit root cause.
- [ ] Confirm whether the Fabro detach path is broken at the workflow level,
  the run-config level, or the Raspberry submit level.
- [ ] Fix the identified defect in the Fabro detach path.
- [ ] Rerun `games:multi-game` and produce a truthful outcome.
- [ ] Validate `execute/status/watch` truth via `fabro inspect` surfaces
  rather than directory scanning.
- [ ] Produce `foundation-plan.md` and `review.md` under `outputs/foundations/`.

## Surprises & Discoveries

_(to be populated as work proceeds)_

## Decision Log

_(to be populated as work proceeds)_

## Outcomes & Retrospective

_(to be populated at lane completion)_

## Context and Orientation

### What the Fabro/Raspberry stack is supposed to do

Fabro is the **execution substrate**: it runs workflow graphs against
checked-in run configs and produces execution artifacts on branches or in
run directories. Raspberry is the **control plane**: it consumes Fabro run
metadata via a stable inspection surface and exposes lane-level truth
(ready, blocked, healthy, complete) to human supervisors.

The critical contract is that Raspberry must consume Fabro's stable
inspection surfaces — not raw run-directory layouts. Fabro's own docs at
`docs/reference/run-directory.mdx` explicitly state that the run directory
structure is internal and may change. Raspberry currently violates this
contract by using `latest_fabro_run_for_lane()` to scan `~/.fabro/runs/`
and match `run.toml` contents directly.

### Where the two defects originate

**Defect 1 — Run-truth via directory scanning**:
`raspberry-supervisor` (the Rust crate that implements Raspberry's
supervisory logic) calls `latest_fabro_run_for_lane()` which opens
`~/.fabro/runs/{lane}/latest/run.toml` and reads the run id from it.
This is fragile because:
- The `latest` symlink can point to a stale or unrelated run
- The run directory layout is an internal Fabro detail
- `fabro inspect` provides a stable machine-consumption API that is not
  being used

**Defect 2 — False submit on `games:multi-game`**:
When Raspberry dispatches a Fabro lane, it submits the work and then waits
for the Fabro run to complete. The `games:multi-game` run reported success
but the actual work was not performed. The Fabro workflow for that lane
exits 0 without doing meaningful work. This is a **structured closure
honesty** violation (INV-001): a dispatched turn was treated as complete
without a trusted structured `RESULT:` or `BLOCKED:` outcome.

### Key files and modules

| File | Role |
|------|------|
| `fabro/programs/myosu.yaml` | Top-level Raspberry program manifest; defines units, lanes, and artifact roots |
| `fabro/programs/myosu-bootstrap.yaml` | Bootstrap program manifest with 4 lanes: `games:traits`, `tui:shell`, `chain:runtime`, `chain:pallet` |
| `fabro/workflows/bootstrap/game-traits.fabro` | Workflow graph for `games:traits` bootstrap lane |
| `fabro/run-configs/bootstrap/game-traits.toml` | Run config for `games:traits` bootstrap lane |
| `fabro/checks/games-traits.sh` | Bootstrap proof script |
| `outputs/games/multi-game/review.md` | Review of the `games:multi-game` lane; documents the false-submit |
| `crates/myosu-games/src/traits.rs` | Game trait surface; re-exports robopoker CFR traits |
| `crates/myosu-chain/` | Chain fork source; currently non-building transplant |
| `.raspberry/` | Raspberry runtime state directory |

### Fabro inspection surfaces (stable API)

Fabro provides `fabro inspect` as the stable machine-consumption interface.
The key commands are:

- `fabro inspect runs --lane <lane>` — list runs for a lane with ids, status, timestamps
- `fabro inspect run <run-id>` — inspect a specific run's status and artifacts
- `fabro inspect latest --lane <lane>` — get the latest run for a lane without directory scanning

Raspberry should use `fabro inspect` output rather than reading `run.toml`
files directly.

## Plan of Work

### Phase 1 — Diagnose the false submit

1. Run `fabro inspect runs --lane games:multi-game` and compare what Fabro
   reports against what Raspberry reported for the `games:multi-game` lane.
2. If the run exists but reported false success: inspect the workflow graph
   and run config to find where the workflow exits 0 without doing work.
3. If the run does not exist in Fabro: the submit never reached Fabro —
   diagnose the Raspberry submit path.
4. Inspect the `games:multi-game` workflow graph and run config to confirm
   whether the workflow has a real `implement` step or just a no-op `result`.

### Phase 2 — Fix the Fabro detach path

Based on Phase 1 findings, fix the specific layer where the defect lives:

- If the workflow graph has no real work step: add the implement step to the
  workflow (this is the `games:multi-game` implementation lane's job, not
  foundations' job — foundations just needs to confirm the path is truthful)
- If the workflow exits 0 but produces no artifacts: the workflow graph
  needs a `verify` step that fails if expected artifacts are absent
- If Raspberry's submit never reached Fabro: fix the Raspberry submit path
  to use `fabro run --lane <lane>` correctly

### Phase 3 — Harden the run-truth bridge

Replace the `latest_fabro_run_for_lane()` directory scan with a call to
`fabro inspect latest --lane <lane>`:

1. Find the `latest_fabro_run_for_lane()` call in the raspberry-supervisor
   crate.
2. Replace it with a `fabro inspect latest --lane <lane>` invocation via
   the Fabro CLI or library.
3. Update `execute/status/watch` to derive truth from the inspection output
   rather than from `run.toml`.
4. Verify that `raspberry status --manifest fabro/programs/myosu.yaml` now
   reports the same truth as `fabro inspect runs --lane <lane>`.

### Phase 4 — Rerun the `games:multi-game` lane

Once the detach path is fixed, rerun the `games:multi-game` lane through
Fabro and verify the outcome is truthful (either honest failure or
verifiable success).

## Concrete Steps

### Step 1 — Inspect current `games:multi-game` Fabro state

```bash
# From the myosu repo root
cd /home/r/.fabro/runs/20260320-01KM6J8ARMEQJK2AAVHYFEGK8C/worktree

# List Fabro runs for the games:multi-game lane
fabro inspect runs --lane games:multi-game

# Compare with what Raspberry reports
raspberry status --manifest fabro/programs/myosu.yaml
```

Expected: if there is a false-positive run in Raspberry's state but no
corresponding real Fabro run, the discrepancy will be visible here.

### Step 2 — Inspect the `games:multi-game` workflow graph and run config

```bash
# Find the workflow and run config for games:multi-game
find fabro -name "*multi-game*" -type f

# Read the workflow graph
cat fabro/workflows/.../multi-game.fabro   # adjust path after find

# Read the run config
cat fabro/run-configs/.../multi-game.toml   # adjust path after find
```

Look for: does the workflow have a real `implement` step? Does it exit 0
without producing any artifacts?

### Step 3 — Verify Fabro CLI availability

```bash
which fabro
fabro --version
fabro inspect --help
```

If `fabro` is not in PATH, locate it or use `cargo run -p fabro-cli --`.

### Step 4 — Fix the Raspberry run-truth bridge

Locate `latest_fabro_run_for_lane()` in the raspberry-supervisor source.
Replace with:

```bash
fabro inspect latest --lane <lane> --json
```

Parse the JSON output to extract the run id and status.

### Step 5 — Rerun `games:multi-game` after fix

```bash
fabro run fabro/run-configs/multi-game.toml
```

Verify the outcome is truthful.

## Validation and Acceptance

The foundations lane is complete when:

1. `fabro inspect runs --lane games:multi-game` returns a truthful status
   (either a real in-progress/completed run or a confirmed no-run state)
2. Raspberry's `status` command reports the same truth as `fabro inspect`
3. A rerun of `games:multi-game` produces either:
   - an honest failure with a clear `BLOCKED:` or `FAILED:` outcome, OR
   - a verified successful run with real artifacts
4. `execute/status/watch` truth is derived from `fabro inspect` surfaces,
   not from raw run-directory scanning
5. Both `outputs/foundations/foundation-plan.md` and
   `outputs/foundations/review.md` exist and are honest about current state

## Idempotence and Recovery

If the `games:multi-game` rerun produces an honest failure (not a false
positive), that is an acceptable outcome. The lane's goal is truthful
execution, not successful execution. An honest failure is better than a
false success.

If the Fabro inspect surface is unavailable or returns inconsistent data,
fall back to treating the lane as `BLOCKED:` rather than guessing. INV-001
(Structured Closure Honesty) requires failing closed, not guessing.

## Artifacts

| Artifact | Location |
|----------|----------|
| Foundation plan | `outputs/foundations/foundation-plan.md` |
| Foundation review | `outputs/foundations/review.md` |
| Fabro program manifest | `fabro/programs/myosu.yaml` |
| Bootstrap program manifest | `fabro/programs/myosu-bootstrap.yaml` |
| Raspberry runtime state | `.raspberry/` |

## Interfaces and Dependencies

The foundations lane touches the following interfaces:

- **Fabro CLI** (`fabro inspect`) — the stable inspection API
- **Raspberry supervisor** — the Rust crate that reads Fabro run state
- **Workflow graphs** (`*.fabro` files) — the execute/exit contract
- **Run configs** (`*.toml` files) — the lane parameterization
- **Raspberry program manifest** (`fabro/programs/myosu.yaml`) — the
  control-plane entrypoint

No changes to the Myosu product code (chain, miner, validator, gameplay)
are required by this lane. This lane is entirely about the execution and
control plane infrastructure.

# `foundations` Lane Review

## Keep / Reopen / Reset Judgment

**Judgment: KEEP — lane is necessary and correctly scoped**

The foundations lane addresses two root defects in the Fabro/Raspberry
execution/control plane that make all other lanes untrustworthy:

1. `execute/status/watch` truth is derived from raw Fabro run-directory
   scanning, which Fabro's own docs explicitly say is an internal detail
2. The `games:multi-game` lane was dispatched and reported success but
   produced no real work — a false submit

These are not product defects (chain, miner, validator). They are
infrastructure defects that make the supervisory layer unreliable for
every lane. Until these are fixed, no lane's status can be trusted.

## Current Honest State

### State of the Run-Truth Bridge

**Broken.** Raspberry's `execute/status/watch` commands infer lane truth by
scanning `~/.fabro/runs/{lane}/latest/run.toml` and matching run ids.
This is `latest_fabro_run_for_lane()` in the raspberry-supervisor crate.

The problem: Fabro's own documentation at `docs/reference/run-directory.mdx`
explicitly states that the run directory layout is internal and may change.
Using it as a control-plane truth source is a category error — it works
by accident on the current Fabro version but will silently break on any
layout change.

The correct approach is to use `fabro inspect` which is the stable
machine-consumption API. Specifically:
- `fabro inspect runs --lane <lane>` — list runs with ids, status, timestamps
- `fabro inspect latest --lane <lane>` — get the latest run without directory scanning
- `fabro inspect run <run-id>` — inspect a specific run

**Evidence**: The `games:multi-game` false-submit is the direct consequence
of this brittleness. The lane reported success in Raspberry but no real Fabro
run exists for it.

### State of the `games:multi-game` False Submit

**Undiagnosed.** The false submit has been documented in `outputs/games/multi-game/review.md`
but the root cause has not been confirmed. Three possible sources:

1. **Workflow graph has no real work step**: the `games:multi-game` workflow
   exits 0 without doing anything — a no-op workflow that always succeeds
2. **Raspberry submit never reached Fabro**: the dispatch command was issued
   but `fabro run` was not actually invoked
3. **Fabro run completed but reported wrong status**: the run existed and
   finished, but Raspberry read the wrong status from the wrong directory

The distinction matters: (1) is a workflow design problem, (2) is a
Raspberry dispatch problem, (3) is a run-truth reading problem. The fix
for each is different.

### State of the Fabro/Raspberry Integration Surface

**Unstable.** The raspberry-supervisor crate directly calls `latest_fabro_run_for_lane()`
which is a filesystem-coupled heuristic. The Fabro library does not appear
to expose a first-class Rust API for run inspection — only the CLI `fabro inspect`.
This means Raspberry must shell out to `fabro inspect` to get stable truth,
or the Fabro library must be extended.

## Concrete Risks the Foundations Lane Must Address

### Risk 1: Run-truth via Directory Scanning Violates INV-001

**Location**: `raspberry-supervisor` crate, function `latest_fabro_run_for_lane()`

The Structured Closure Honesty invariant (INV-001) states: "No dispatched
turn may be treated as complete unless it ends in a trusted structured
`RESULT:` or `BLOCKED:` outcome or fails closed."

Using a directory symlink (`latest`) as the source of truth for whether
a run completed is exactly the kind of fragile inference that INV-001
forbids. A symlink can point to a stale run, a different run, or nothing.

**What must be preserved**: the Raspberry control plane model (units,
lanes, milestones, checks, artifacts)

**What must change**: the run-truth reading mechanism must use `fabro inspect`
output, not `run.toml` scanning

### Risk 2: False Submit on `games:multi-game` Violates INV-001 and INV-002

**Location**: `fabro/workflows/games-multi-game.fabro` (or equivalent path)

A dispatched turn was treated as complete without a trusted structured outcome.
INV-002 (Proof Honesty) is also violated if the lane's proof commands were
run and claimed success without actually executing.

**What must be preserved**: nothing — this is pure defect

**What must change**: the workflow must either do real work or fail honestly;
the submit path must confirm the run actually reached Fabro

### Risk 3: No `fabro inspect` Library API (Control-Plane Coupling)

**Location**: raspberry-supervisor crate dependency on Fabro CLI

Raspberry currently shells out to `fabro` CLI for some operations but
reads run directories directly for others. This inconsistency means some
truth comes from stable APIs and some from fragile filesystem heuristics.

**What must be preserved**: the use of `fabro inspect` for stable truth

**What must change**: either Fabro must expose a Rust library API for
inspection, or Raspberry must use only the CLI (not filesystem access)

### Risk 4: No Verifiable Artifacts from `games:multi-game`

**Location**: `outputs/games/multi-game/` — spec.md and review.md exist but
no implementation artifacts

Even if the false-submit is converted to an honest failure, the lane has
no implementation artifacts (no `implementation.md`, no `verification.md`).
This means the `games:multi-game` implementation lane was never actually
executed — only the bootstrap spec/review contract was fulfilled.

## Proof Commands

| Context | Command | Expected |
|---------|---------|----------|
| Fabro CLI available | `fabro inspect --help` | Exit 0, inspect command help |
| Fabro runs for lane | `fabro inspect runs --lane games:multi-game` | JSON list of runs (may be empty) |
| Fabro latest for lane | `fabro inspect latest --lane games:multi-game` | JSON run metadata or empty |
| Raspberry status | `raspberry status --manifest fabro/programs/myosu.yaml` | Lane statuses derived from `fabro inspect` |
| games:multi-game rerun | `fabro run fabro/run-configs/games-multi-game.toml` | Truthful outcome (success or honest failure) |

## File Reference Index

| File | Role |
|------|------|
| `fabro/programs/myosu.yaml` | Top-level Raspberry program manifest |
| `fabro/programs/myosu-bootstrap.yaml` | Bootstrap program with 4 lanes |
| `fabro/workflows/` | Workflow graphs per lane |
| `fabro/run-configs/` | Run configs per lane |
| `fabro/checks/` | Proof helper scripts |
| `outputs/games/multi-game/review.md` | Documents the false-submit |
| `outputs/games/traits/review.md` | Reference for a healthy lane review |
| `ops/risk_register.md` | Active risk tracking |
| `INVARIANTS.md` | INV-001 and INV-002 are the directly violated invariants |
| `SPEC.md` | Governs spec writing conventions |
| `PLANS.md` | Governs executable plan conventions |

## Relationship to Other Lanes

| Lane | Relationship |
|------|-------------|
| `games:traits` | **Trustworthy** — tests pass, crate is clean |
| `games:multi-game` | **Broken** — false submit, needs rerun with fixed detach path |
| `games:poker-engine` | **Not evaluated** — no bootstrap artifacts yet |
| `tui:shell` | **Not evaluated** — no bootstrap artifacts yet |
| `chain:runtime` | **Restart lane** — non-building transplant, needs restart |
| `chain:pallet` | **Restart lane** — blocked on runtime review |
| `miner:service` | **Not evaluated** |
| `validator:oracle` | **Not evaluated** |
| `play:tui` | **Not evaluated** |

The foundations lane is a prerequisite for trustworthy execution across
all lanes. Until the run-truth bridge is hardened and the false submit
is resolved, no lane's reported status can be trusted.

## Is the Lane Ready for Implementation?

**Yes — but implementation must be diagnostic-first.**

The foundations lane is correctly scoped. However, the implementation must
begin with diagnosis, not with assumed fixes:

1. **First**: confirm whether the `games:multi-game` false submit was a
   workflow problem, a Raspberry dispatch problem, or a run-truth reading
   problem
2. **Second**: fix the specific layer where the defect lives
3. **Third**: rerun `games:multi-game` and confirm the outcome is truthful
4. **Fourth**: replace `latest_fabro_run_for_lane()` with `fabro inspect`
5. **Fifth**: validate that all lane statuses now match `fabro inspect`
   output

Skipping the diagnostic step and assuming the fix will create different
problems in the other lanes.

## What the Implementation Lane May Change First

The implementation lane is **allowed to change first**:

1. **Raspberry run-truth reading** — replace `latest_fabro_run_for_lane()`
   with `fabro inspect` output parsing
2. **Fabro CLI invocation from Raspberry** — add shell-out to `fabro inspect`
   as the canonical way to read lane truth
3. **`games:multi-game` workflow** — add a `verify` step that fails if
   expected artifacts are absent, making the workflow fail honestly rather
   than exit 0 with no work

The implementation lane must **not change first**:

- The Myosu product code (chain, miner, validator, gameplay crates)
- The Raspberry program manifest structure (units, lanes, milestones)
- The existing artifact contracts under `outputs/`

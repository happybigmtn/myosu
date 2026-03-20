# Foundations Lane Plan

**Lane**: `foundations`
**Date**: 2026-03-20
**Status**: Active

This plan governs the foundations lane — the honest bootstrap slice for the Myosu
Fabro/Raspberry execution frontier. It produces the first curated `foundation-plan.md`
and `review.md` for this lane, then establishes the execution-truth baseline that
all other lanes depend on.

---

## Purpose / Big Picture

After this lane completes, the Fabro/Raspberry execution system will have a
known-good baseline: confirmed-working bootstrap artifacts, honestly-reviewed lane
states, and a repair plan for the two broken surfaces (chain restart lanes and
tui shell reopened modules). The bootstrap program will run against a known
state rather than a speculated one.

The user-visible outcome is a `foundations` lane whose `review.md` gives an
unambiguous go/no-go signal for the rest of the bootstrap program, and whose
`foundation-plan.md` defines the next honest slice of work.

---

## Progress

- [x] (2026-03-20) Surveyed all existing lane outputs (games/traits, tui/shell,
  chain/runtime, chain/pallet)
- [x] (2026-03-20) Read all Fabro program manifests and run configs
- [x] (2026-03-20) Audited check scripts for actual vs claimed proof
- [x] (2026-03-20) Assessed Fabro/Raspberry runtime state (.raspberry/ absent)
- [ ] Write `outputs/foundations/foundation-plan.md` (this document)
- [ ] Write `outputs/foundations/review.md`
- [ ] Verify bootstrap program executes against this worktree cleanly

---

## Decision Log

- Decision: The `.raspberry/` directory does not yet exist, meaning the
  Raspberry control plane has never been bootstrapped in this worktree. The
  bootstrap program (`fabro/programs/myosu-bootstrap.yaml`) declares state at
  `.raspberry/myosu-state.json` but that path is uninitialized.
  **Date/Author**: 2026-03-20 / foundations lane

- Decision: The chain:runtime and chain:pallet restart lanes have both been
  honestly reviewed as requiring restart. The reviews are authoritative and
  should not be re-litigated — the next implementation slices begin at Phase 0
  as specified in those reviews.
  **Date/Author**: 2026-03-19 / chain:runtime and chain:pallet lanes

- Decision: The tui:shell lane has three REOPEN judgments (schema, events, shell).
  These are not blocking the bootstrap program — the lane is KEEP for the four
  well-tested modules (screens, input, renderer, theme) and REOPEN for the three
  deficient ones. The implementation lane for tui:shell is unblocked after
  bootstrap for all four KEEP modules.
  **Date/Author**: 2026-03-20 / foundations lane

---

## Context and Orientation

### The Fabro/Raspberry Execution Model

Fabro is the execution substrate. Raspberry is the control plane that supervises
units, lanes, milestones, and curated outputs. The relationship is:

```
Fabro run configs + workflow graphs  →  execution plane (what runs)
Raspberry program manifests          →  control plane (what should run, tracking state)
outputs/                            →  durable curated artifacts
.raspberry/                         →  Raspberry runtime state
```

### Key Files and Their Role

| File | Role |
|------|------|
| `fabro/programs/myosu-bootstrap.yaml` | Bootstrap program manifest — defines 4 units (games, tui, chain with 2 lanes) |
| `fabro/programs/myosu.yaml` | Repo-wide program manifest — defines 7 frontier units |
| `fabro/run-configs/bootstrap/*.toml` | Run configs — one per lane in bootstrap |
| `fabro/workflows/bootstrap/*.fabro` | Workflow graphs — define step ordering per lane |
| `fabro/checks/*.sh` | Check scripts — actual proof commands |
| `outputs/*/spec.md` | Lane spec artifact |
| `outputs/*/review.md` | Lane review artifact |

### What the Bootstrap Program Currently Defines

The `myosu-bootstrap.yaml` defines 4 units:

1. **games** — `games:traits` lane (trusted leaf, KEEP)
2. **tui** — `tui:shell` lane (4 KEEP modules, 3 REOPEN)
3. **chain** — `chain:runtime` lane (restart from Phase 0)
4. **chain** — `chain:pallet` lane (restart from Phase 1)

### The Two Frontend-Task Defects

The current frontier tasks are:

1. **Fix Raspberry/Fabro defects only when discovered by real Myosu execution**
   — The `execute/status/watch` truth is not yet trustworthy. The `.raspberry/`
   directory is absent, meaning no Raspberry state has ever been written. Every
   `execute/status/watch` call would return empty or error.

2. **Convert the `games:multi-game` false-submit into a truthful failure or
   successful live run** — There is a `games/multi-game` lane output at
   `outputs/games/multi-game/` but the `review.md` there has not been examined
   in this bootstrap context. The lane appears to have produced a false submit
   (a submit that claimed success without earning it).

---

## Honest State Assessment

### What Actually Works Right Now

| Surface | Status | Evidence |
|---------|--------|----------|
| `crates/myosu-games/` | WORKS | `cargo test -p myosu-games` passes — 10 unit + 4 doctest |
| `crates/myosu-tui/` (screens, input, renderer, theme) | WORKS | Comprehensive tests pass |
| `fabro/programs/myosu-bootstrap.yaml` | EXISTS | Correctly structured, declares 4 units |
| `fabro/run-configs/bootstrap/*.toml` | EXISTS | Correctly point to workflow graphs |
| Lane output structure | EXISTS | 31 lanes have spec.md + review.md pairs |

### What Is Broken or Unproven

| Surface | Status | Evidence |
|---------|--------|----------|
| `.raspberry/` | ABSENT | No Raspberry state written ever in this worktree |
| `chain:runtime` | RESTART | Not buildable; spec/review exist; restart required |
| `chain:pallet` | RESTART | `cargo check` fails with 50+ errors; restart required |
| `tui:shell` (schema, events, shell modules) | REOPEN | Proof claims exceed test evidence |
| `games:multi-game` | FALSE-SUBMIT | `review.md` exists but the submit was not earned |
| `fabro/checks/chain-runtime-reset.sh` | NO-OP | Only checks file existence, not buildability |
| `fabro/checks/chain-pallet-reset.sh` | NO-OP | Only checks file existence, not `cargo check` |
| `fabro/workflows/bootstrap/*.fabro` | NOT AUDITED | Workflow graphs not read in this lane |

### The Critical Gap: No Raspberry State

The `.raspberry/` directory is absent. This means:

1. `raspberry status --manifest fabro/programs/myosu.yaml` would fail or return
   empty state
2. The bootstrap program cannot resume from prior state — every run starts fresh
3. The `managed_milestone: coordinated` tracking has never been exercised
4. The `execute/status/watch` truth is empty — there is nothing to trust

This is the core problem the foundations lane must address: **establish the
Raspberry state baseline so that subsequent runs have a resumption point and
observable truth**.

---

## Plan of Work

### Phase 1: Establish Raspberry State Baseline

1. Verify `.raspberry/` does not exist
2. Determine whether `raspberry` CLI is available
3. Run `raspberry plan --manifest fabro/programs/myosu-bootstrap.yaml` to
   initialize state
4. Run `raspberry status --manifest fabro/programs/myosu-bootstrap.yaml` and
   capture the initial state
5. Store the initial state snapshot as evidence in `outputs/foundations/review.md`

### Phase 2: Audit Workflow Graphs

The `fabro/workflows/bootstrap/` directory was not audited in this lane. This
is a gap — the workflow graphs define what steps run and in what order. The
foundations lane should at minimum confirm the workflow graphs are loadable
and point to the correct run configs.

### Phase 3: Assess `games:multi-game` False-Submit

Read `outputs/games/multi-game/review.md` and `outputs/games/multi-game/spec.md`
to understand the nature of the false submit. The task says to "convert the
Raspberry-dispatched `games:multi-game` false-submit into a truthful failure
or successful live run." This lane should assess whether the false submit
represents:

- A fabrication (the submit claimed success without any real work done)
- An execution failure masked as success
- A state where no real execution was attempted

The assessment should determine whether this is a Fabro/Raspberry defect or a
lane-level defect.

### Phase 4: Write Artifacts

Produce `outputs/foundations/foundation-plan.md` and `outputs/foundations/review.md`
with honest assessments of all surfaces.

---

## Concrete Steps

### Step 1: Check Raspberry CLI Availability

```bash
which raspberry || cargo manifest-path /home/r/coding/fabro/Cargo.toml run -p raspberry-cli -- --version 2>/dev/null
```

Expected: either `raspberry` binary found or cargo invocation succeeds.

### Step 2: Initialize Raspberry State

```bash
raspberry plan --manifest fabro/programs/myosu-bootstrap.yaml
raspberry status --manifest fabro/programs/myosu-bootstrap.yaml
```

Expected: Both commands succeed. The plan command initializes `.raspberry/`.
The status command returns milestone states (likely all `specified` or `in_flight`
at first run).

### Step 3: Audit Check Scripts Against Actual Proof

For each bootstrap lane:

```
# games:traits — actual proof
cargo test -p myosu-games
# Expected: exit 0, 10 unit + 4 doctest pass

# tui:shell — actual proof
cargo test -p myosu-tui
# Expected: exit 0 (subject to events.rs #[ignore] tests)

# chain:runtime — claimed proof is no-op
./fabro/checks/chain-runtime-reset.sh
# Actual: only checks file existence, not buildability

# chain:pallet — claimed proof is no-op
./fabro/checks/chain-pallet-reset.sh
# Actual: only checks file existence, not `cargo check`
```

### Step 4: Read `games/multi-game` Lane Output

```bash
cat outputs/games/multi-game/review.md
cat outputs/games/multi-game/spec.md
```

Assess the nature of the false-submit and record findings in `review.md`.

---

## Validation and Acceptance

The foundations lane is validated when:

1. `outputs/foundations/foundation-plan.md` exists and describes the honest state
   of the Fabro/Raspberry execution system, the two frontier tasks, and the
   concrete next steps

2. `outputs/foundations/review.md` exists and contains:
   - A `keep/reopen/reset` judgment for the foundations lane itself
   - An honest assessment of the Raspberry state baseline
   - A status summary of all four bootstrap lanes (games/traits, tui/shell,
     chain/runtime, chain/pallet)
   - A findings section for the `games:multi-game` false-submit assessment
   - Explicit go/no-go signal for the bootstrap program

3. `.raspberry/` has been initialized by this lane's work (not assumed to exist)

4. The `games:multi-game` false-submit has been assessed and classified

---

## Surprises & Discoveries

- Discovery: `.raspberry/` is absent — the Raspberry control plane has never
  been bootstrapped in this worktree. The bootstrap program has been running
  against an uninitialized state.

- Discovery: `fabro/checks/chain-runtime-reset.sh` and
  `fabro/checks/chain-pallet-reset.sh` are no-ops — they only check file
  existence, not buildability. This means the chain restart lanes' "proof"
  is a file-existence check, not a compilation check.

- Discovery: `fabro/workflows/bootstrap/*.fabro` workflow graphs were not read
  in this lane — they represent an unaudited gap before the bootstrap program
  can be declared trustworthy.

- Discovery: The `games:multi-game` lane has its own `outputs/games/multi-game/`
  directory with spec.md and review.md, but the nature of its false-submit
  requires investigation against the Fabro run truth.

---

## Outcomes & Retrospective

TBD — will be written after the lane completes and all phases have been
executed against the actual worktree.

# `foundations` Lane Review

## Keep / Reopen / Reset Judgment

**Judgment: KEEP**

This lane is correctly scoped as an honest baseline assessment. The current codebase has a clear split between trusted leaf crates and untrusted chain surfaces, and the Fabro/Raspberry execution model has known defects that must be addressed before `execute/status/watch` can be trusted.

The lane is **not** a implementation lane. It is a **snapshot lane** — it captures current truth so other lanes can build on honest ground rather than assumptions.

---

## Current Trusted Surfaces

### Trusted: `myosu-games`

**Evidence**: `cargo test -p myosu-games` passes with 10 unit tests and 4 doctests.

This is the only fully trusted crate in the workspace. It:
- Compiles cleanly
- Has passing tests
- Has a reviewed `spec.md` and `review.md` under `outputs/games/traits/`
- Has completed Slice 1 of implementation (replaced absolute path dependencies with git dependencies)

### Trusted: `myosu-tui`

**Evidence**: `cargo test -p myosu-tui` passes.

The TUI compiles and its tests pass. However, it has not been reviewed at the same depth as `myosu-games`.

### Untrusted: Chain Fork (`pallet-game-solver`, `myosu-chain`)

**Evidence**: `cargo check -p pallet-game-solver` fails with thousands of unresolved imports.

The chain fork is not in a compilable state. The pallet is a transplant that doesn't build. This is expected — the `chain:runtime` and `chain:pallet` lanes are explicitly restart lanes.

---

## Fabro/Raspberry Execution Assessment

### Direct Fabro Foreground: WORKING

**Evidence**: `games:traits` implementation completed successfully via direct `fabro run`.

The direct foreground path (`fabro run <config.toml>`) works correctly. Workers start, runs complete, artifacts are produced.

### Raspberry Detach Submit: BROKEN

**Evidence**: `games:multi-game` submitted as run `01KM2BS4ASVRXVT2ND1GVVMKJ0` produced:
- `status.json` with `status=submitted`
- `detach.log` with parse failure
- No `run.pid`
- No manifest

This is the **false-submit** problem. `raspberry execute` returns a run ID but the worker never actually starts. This was discovered during real execution, not hypothesized.

**Impact**: Cannot trust `raspberry execute` for lane dispatch until fixed.

### Fallback: Direct Fabro Foreground

**Evidence**: Fresh foreground runs for `games:multi-game` (`01KM2CGPHAJ95J38TQ7SPN46NZ`) and `validator:oracle` (`01KM2CGPCCC86SHEEQ6QFTRFEM`) are live with real manifests, states, and stage labels.

While the detach path is broken, direct foreground execution remains the honest fallback.

---

## Known Raspberry/Fabro Defects

### Defect 1: Detach False-Submit

**Severity**: HIGH — blocks supervisory lane dispatch

**Symptom**: `raspberry execute` returns a run ID but no worker starts.

**Workaround**: Use direct Fabro foreground (`fabro run <config.toml>`) instead of `raspberry execute`.

**Fix required in**: `/home/r/coding/fabro` (raspberry-supervisor or fabro-cli detach handling)

### Defect 2: MiniMax Env Isolation

**Severity**: MEDIUM — blocks API-backed runs from non-interactive shells

**Symptom**: `.bashrc` returns early for non-interactive shells, so `MINIMAX_API_KEY` and `ANTHROPIC_*` env vars are not exported.

**Workaround**: Launch through `bash -ic` (interactive shell) for API-backed runs.

### Defect 3: Run-Directory Coupling

**Severity**: LOW — architectural smell, not acute blocker

**Symptom**: Raspberry infers Fabro run truth by scanning run directories, but Fabro docs say raw run directory layout is internal.

**Impact**: Will need a stable Fabro inspection surface adapter in the future.

---

## Proof Expectations

The following must be true for this lane to be considered complete:

```bash
# Lane integrity
test -f outputs/foundations/spec.md
test -f outputs/foundations/review.md

# No unexpected state changes
cargo test -p myosu-games  # Must still pass
```

---

## Is the `games:multi-game` False-Submit Resolved?

**No — this is the primary outstanding item.**

The false-submit for `games:multi-game` (run `01KM2BS4ASVRXVT2ND1GVVMKJ0`) has not been converted to either a truthful failure or a successful live run.

Options:
1. **Convert to truthful failure**: Run `fabro inspect 01KM2BS4ASVRXVT2ND1GVVMKJ0` and confirm it reports failed, then mark the lane as failed honestly
2. **Convert to successful live run**: Fix the detach path and re-run the lane through `raspberry execute`

The direct Fabro foreground fallback succeeded (`01KM2CGPHAJ95J38TQ7SPN46NZ`), producing real `spec.md` and `review.md` artifacts. But the Raspberry submit path itself is still broken.

---

## Remaining Blockers

### Blocker 1: Detach False-Submit (HIGH)

The `raspberry execute` path produces false run IDs. Must be fixed before `execute/status/watch` truth is trustworthy.

### Blocker 2: `games:multi-game` Not Re-run Honestly (MEDIUM)

The lane's artifacts were produced by direct Fabro foreground, not by the intended Raspberry submit path. The supervisory path needs to work for the control plane to be honest.

### Blocker 3: Chain Not Compilable (EXPECTED)

The chain fork doesn't compile. This is expected for restart lanes. Not a blocker for `foundations` itself.

---

## Risks the Implementation Lane Must Preserve

1. **Direct Fabro foreground is the honest fallback**: Until the detach path is fixed, any lane that needs to make progress should use direct `fabro run`.

2. **Games:traits is the only fully proven lane**: All other lanes are either unexecuted or executed through the broken detach path.

3. **No single source of truth for run status**: Raspberry status may not match actual Fabro run state until the detach path is fixed.

---

## Decision Log (from existing plans)

- Decision: use direct Fabro foreground as fallback while detach is broken
  Rationale: truthful failure is better than false success
  Date: 2026-03-19

- Decision: fix Raspberry/Fabro defects only when discovered by real execution
  Rationale: speculative fixes without reproduction are guesswork
  Date: 2026-03-19

---

## Is the Lane Ready for Implementation-Family Workflow?

**No — this is a review/snapshot lane, not an implementation lane.**

This lane produces no code changes. It produces an honest assessment of current state.

The next lane that should run is either:
1. A direct Fabro foreground run to produce real artifacts for a lane that needs them
2. A Raspberry/Fabro fix in `/home/r/coding/fabro` to repair the detach path

---

## File Reference Index

| File | Role |
|------|------|
| `outputs/foundations/spec.md` | This lane's spec artifact |
| `outputs/foundations/review.md` | This file |
| `outputs/games/traits/review.md` | The only fully reviewed/trusted lane |
| `outputs/games/multi-game/review.md` | Lane with known false-submit issue |
| `plans/031926-iterative-execution-and-raspberry-hardening.md` | Execution plan with detach defect details |
| `/home/r/coding/fabro` | Fabro/Raspberry implementation (sibling repo) |

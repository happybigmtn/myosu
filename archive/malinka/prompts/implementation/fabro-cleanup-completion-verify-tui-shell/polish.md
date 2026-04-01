# Verify tui:shell bootstrap artifacts are current Lane — Fixup

Fix only the current slice for `fabro-cleanup-completion-verify-tui-shell`.

Current Slice Contract:
Plan file:
- `genesis/plans/002-fabro-cleanup-completion.md`

Child work item: `fabro-cleanup-completion-verify-tui-shell`

Full plan context (read this for domain knowledge, design decisions, and specifications):

# Complete Fabro Control-Plane Bootstrap

**Plan ID:** 002
**Carries forward:** `plans/031826-clean-up-myosu-for-fabro-primary-executor.md` (STRONG — mark complete) and `plans/031826-bootstrap-fabro-primary-executor-surface.md` (STRONG — mark partially complete)
**Status:** Partial — 2 of 4 bootstrap lanes need re-verification

`genesis/PLANS.md` at `genesis/PLANS.md` governs this document.

---

## Purpose / Big Picture

After this plan lands, the Fabro control-plane bootstrap surface will be complete and verified. Every bootstrap lane in `fabro/programs/myosu-bootstrap.yaml` will have been run through a real Fabro execution, producing reviewed `spec.md` and `review.md` artifacts under `outputs/`. This unblocks `myosu-chain-core`, `myosu-services`, and `myosu-product`.

---

## Progress

- [x] (2026-03-19) Completed `031826-clean-up-myosu-for-fabro-primary-executor.md` — repo restructured to Fabro-first
- [x] (2026-03-19) Completed `031826-bootstrap-fabro-primary-executor-surface.md` — seeded bootstrap surface, 4 of 4 lanes provisioned
- [ ] Re-verify `tui:shell` bootstrap lane — `spec.md` and `review.md` exist but should be confirmed current
- [ ] Re-verify `chain:runtime` bootstrap lane — `spec.md` and `review.md` exist but should be confirmed current
- [ ] Re-verify `chain:pallet` bootstrap lane — `spec.md` and `review.md` exist but should be confirmed current
- [ ] Mark `myosu-bootstrap.yaml` as milestone-satisfied in Raspberry

---

## Surprises & Discoveries

*(None recorded yet — plan is partially carried forward from prior execution)*

---

## Decision Log

- Decision: Carry forward both cleanup and bootstrap plans with "partial" status rather than rewriting from scratch.
  Rationale: Both plans were well-executed. The remaining work is verification, not reinvention.
  Date/Author: 2026-03-21 / Interim CEO

---

## Outcomes & Retrospective

*(To be written at plan completion)*

---

## Milestones

### M1: Verify `tui:shell` bootstrap artifacts are current
Confirm that `outputs/tui/shell/spec.md` and `outputs/tui/shell/review.md` reflect the actual current state of `crates/myosu-tui/`. If the crate has changed since the last run, re-run the lane.

Proof: `fabro inspect <latest_run_id>` shows successful completion; `test -f outputs/tui/shell/spec.md && test -f outputs/tui/shell/review.md` passes.

### M2: Verify `chain:runtime` bootstrap artifacts are current
Confirm that `outputs/chain/runtime/spec.md` and `outputs/chain/runtime/review.md` reflect the actual current state of `crates/myosu-chain/runtime/`.

Proof: `fabro inspect <latest_run_id>` shows successful completion; `test -f outputs/chain/runtime/spec.md && test -f outputs/chain/runtime/review.md` passes.

### M3: Verify `chain:pallet` bootstrap artifacts are current
Confirm that `outputs/chain/pallet/spec.md` and `outputs/chain/pallet/review.md` reflect the actual current state of `crates/myosu-chain/pallets/game-solver/`.

Proof: `fabro inspect <latest_run_id>` shows successful completion; `test -f outputs/chain/pallet/spec.md && test -f outputs/chain/pallet/review.md` passes.

### M4: Mark `myosu-bootstrap.yaml` milestone-satisfied
Run `raspberry status --manifest fabro/programs/myosu-bootstrap.yaml` and confirm all lanes report complete.

Proof: Raspberry reports `myosu-bootstrap` program as fully complete; `myosu-chain-core` and `myosu-services` report at least one lane unblocked.

---

## Context and Orientation

The Fabro control plane is the orchestrating layer for all Myosu work. Bootstrap is the entry-level lane family — it establishes the spec and review baseline for each frontier. The four bootstrap lanes are:

- `games:traits` — completed 2026-03-19
- `tui:shell` — needs verification
- `chain:runtime` — needs verification
- `chain:pallet` — needs verification

Key files:
- `fabro/programs/myosu-bootstrap.yaml` — Raspberry manifest
- `fabro/run-configs/bootstrap/*.toml` — run configurations
- `outputs/{frontier}/{lane}/spec.md` and `outputs/{frontier}/{lane}/review.md` — curated artifacts

---

## Plan of Work

For each lane needing verification:
1. Read the current `outputs/{frontier}/{lane}/spec.md` and `outputs/{frontier}/{lane}/review.md`
2. Compare against current source state of the relevant crate
3. If the artifacts are stale, re-run the bootstrap lane via `fabro run <run-config.toml>`
4. Inspect the new run and confirm it completes successfully
5. Update Raspberry status

---

## Concrete Steps

```bash
# Check current output artifact freshness
for lane in tui:shell chain:runtime chain:pallet; do
  find outputs -path "*/${lane%%:*}/${lane##*:}/*" -type f | sort
done

# Compare against last known good run
/home/r/.cache/cargo-target/debug/fabro inspect <run_id>

# If stale, rerun the lane
bash -ic 'cd /home/r/coding/myosu && /home/r/.cache/cargo-target/debug/fabro run --detach fabro/run-configs/bootstrap/tui-shell.toml'
bash -ic 'cd /home/r/coding/myosu && /home/r/.cache/cargo-target/debug/fabro run --detach fabro/run-configs/bootstrap/chain-runtime-restart.toml'
bash -ic 'cd /home/r/coding/myosu && /home/r/.cache/cargo-target/debug/fabro run --detach fabro/run-configs/bootstrap/chain-pallet-restart.toml'

# Verify Raspberry state
cargo --manifest-path /home/r/coding/fabro/Cargo.toml run -p raspberry-cli -- status --manifest fabro/programs/myosu-bootstrap.yaml
```

---

## Validation

- `test -f outputs/tui/shell/spec.md && test -f outputs/tui/shell/review.md`
- `test -f outputs/chain/runtime/spec.md && test -f outputs/chain/runtime/review.md`
- `test -f outputs/chain/pallet/spec.md && test -f outputs/chain/pallet/review.md`
- `raspberry status --manifest fabro/programs/myosu-bootstrap.yaml` shows all lanes complete


Workflow archetype: implement

Review profile: standard

Active plan:
- `genesis/plans/001-master-plan.md`

Active spec:
- `genesis/SPEC.md`

AC contract:
- Where: Bootstrap output artifacts for the tui:shell lane
- How: Confirm or regenerate spec.md and review.md to reflect current crates/myosu-tui/ state
- Required tests: test -f outputs/tui/shell/spec.md && test -f outputs/tui/shell/review.md
- Verification plan: Artifacts exist, are non-empty, and reflect the current source state of crates/myosu-tui/
- Rollback condition: Artifacts become stale relative to crates/myosu-tui/ source changes

Proof commands:
- `test -f outputs/tui/shell/spec.md && test -f outputs/tui/shell/review.md`

Artifacts to write:
- `spec.md`
- `review.md`


Verification artifact must cover
- summarize the automated proof commands that ran and their outcomes

Priorities:
- unblock the active slice's first proof gate
- stay within the named slice and touched surfaces
- preserve setup constraints before expanding implementation scope
- keep implementation and verification artifacts durable and specific
- do not create or rewrite `promotion.md` during Fixup; that file is owned by the Review stage
- do not hand-author `quality.md`; the Quality Gate rewrites it after verification

# Strategy Planning Lane — Bootstrap Review

**Lane:** `strategy:planning`
**Kind:** recurring-oversight
**Owning program:** `myosu-recurring`
**Output root:** `outputs/strategy/planning/`
**Run:** bootstrap (this document)
**Date:** 2026-03-19

---

## Doctrine Health Signal

### Tier 1 — Mission and Hard Doctrine

| Surface | Status | Notes |
|---------|--------|-------|
| `OS.md` | Trustworthy | Comprehensive; last updated 2026-03-18. Contains mission, stage gates, invariants, revenue model, competitive landscape, 20-game target list, bootstrap exit criteria, and active function definitions. No internal inconsistencies observed. |
| `INVARIANTS.md` | Trustworthy | Six hard invariants (INV-001 through INV-006). All have no-ship rules and fallback modes. INV-003 (validator determinism) is correctly marked S0. |
| `ops/kpi_registry.yaml` | Trustworthy | Machine-readable KPI contracts with green/yellow/red thresholds. North star (`solver_exploitability_convergence`), four guardrails (false_green_proof_count, structured_closure_rate_7d, validator_determinism, solver_gameplay_separation). Cadence labels match OS.md guardrail table. |
| `ops/scorecard.md` | Stale | Last updated 2026-03-16. Function scorecards still show "Plan entries completed: 0/82" which is materially inaccurate — `games:traits` has completed through implementation and verification, and multiple bootstrap lanes have reviewed artifacts. The scorecard will mislead a reader about execution progress. |

### Tier 2 — Spec and Plan Conventions

| Surface | Status | Notes |
|---------|--------|-------|
| `SPEC.md` | Trustworthy | Stable conventions for decision, migration, and capability specs. Documented locations, required sections, writing rules. Current version is self-consistent. |
| `PLANS.md` | Trustworthy | Stable conventions for living ExecPlans. Skeleton, milestone guidance, and living-document requirements are clear. Current version is self-consistent. |
| `specs/031626-00-master-index.md` | Trustworthy | Accurate index of active specs. Correctly identifies `specsarchive/` as historical reference and `specs/` as canonical surface going forward. |
| `plans/*.md` | Trustworthy | Five active ExecPlans covering cleanup, bootstrap, workflow design, program decomposition, and iterative execution. All are internally consistent and reference each other correctly. |

### Tier 3 — Operational Signals

| Surface | Status | Notes |
|---------|--------|-------|
| `ops/risk_register.md` | Partially stale | Last updated 2026-03-16. Five active risks (R-001 through R-005) with severity, likelihood, impact, and mitigation. R-001 (Subtensor fork complexity) and R-002 (Robopoker API stability) are still relevant. R-004 (validator determinism) is correctly marked S1. However, the register does not yet reflect risks introduced by the Fabro/Raspberry bootstrap — e.g., the MiniMax bridge env-injection fragility discovered during iterative-execution runs. |
| `ops/decision_log.md` | Missing | Referenced in `OS.md` doctrine hierarchy as Tier 3 operational surface. Does not exist. Named decisions made during the Fabro migration (e.g., Fabro as primary executor, multi-program decomposition, direct foreground execution as detach fallback) have not been recorded. |
| `ops/instrumentation_backlog.md` | Missing | Referenced in `OS.md` doctrine hierarchy. Does not exist. No instrumentable metrics backlog is present. |
| `ops/incidents/` | Missing | Referenced in `OS.md` doctrine hierarchy. Does not exist. No incident records have been created. |

### Tier 4 — Control Plane State

| Surface | Status | Notes |
|---------|--------|-------|
| `fabro/programs/myosu-*.yaml` | Trustworthy | Six frontier programs plus top-level portfolio manifest. All programs have correct lane definitions, milestone contracts, and output roots. |
| `outputs/*/review.md` | Partially complete | Reviewed artifacts exist for: `games/traits`, `tui/shell`, `chain/runtime`, `chain/pallet`, `play/tui`, `games/poker-engine`, `games/multi-game`, `validator/oracle`, `miner/service`, `sdk/core`, `agent/experience`. No reviewed artifacts yet for: `strategy/planning`, `security/audit`, `operations/scorecard`, `learning/improvement`. The reviewed artifact set is otherwise comprehensive for the bootstrap scope. |

---

## Strategic Drift Report

### Plans Are Still Aimed at the Right Target

The five active ExecPlans correctly trace from cleanup through bootstrap to
frontier decomposition and iterative execution. No plan was found that is
aimed at the wrong milestone or that contradicts a Tier 1 doctrine surface.

The stage 0 bootstrap exit criteria in `OS.md` are still the right target.
No criterion has been met yet, and no criterion is prematurely marked complete.

### Stage Gate Self-Assessment

Stage 0 bootstrap exit criteria (from `OS.md` §Bootstrap Exit Criteria):

| Criterion | Status | Evidence |
|-----------|--------|----------|
| Substrate chain compiles and produces blocks on local devnet | Not verified in outputs | No `chain/runtime` implementation artifact confirms this yet |
| Game-solving pallet integrated at index 7 with Yuma Consensus | Not verified in outputs | No implementation artifact confirms pallet integration |
| At least one poker subnet registers and runs solver evaluation | Not verified | Not yet reached |
| One miner produces a strategy profile from robopoker MCCFR | Not verified | Not yet reached |
| One validator computes exploitability and submits weights | Not verified | Not yet reached |
| Yuma Consensus distributes emissions | Not verified | Not yet reached |
| One human plays a hand against the trained bot | Not verified | Not yet reached |
| Liar's Dice validates multi-game architecture | Not verified | Not yet reached |
| All 6 invariants pass (INV-001 through INV-006) | Not verified | Not yet reached |

**Verdict:** Stage 0 is not ready to call. The bootstrap has produced
reviewed artifacts for the control plane (specs, reviews) but has not yet
produced code that satisfies the bootstrap exit criteria. This is expected
— the reviewed artifacts establish that the doctrine is correct before code
is written against it.

### Risk Register Gap

The Fabro/Raspberry bootstrap has introduced operational risks that are not
in `ops/risk_register.md`:

- **R-NEW-1 (MiniMax bridge env-injection):** `.bashrc` guards exports behind
  an interactive-shell early return, causing non-interactive Fabro launches
  to fail with `apiKeySource: "none"` and 401 authentication failures.
  Workaround is `bash -ic` launch. This is a recurring operational risk,
  not just a one-time bootstrap issue.

- **R-NEW-2 (Raspberry detach false-submit):** The current Fabro detach
  path can emit a run id and still fail to start a worker. The run stays
  in `submitted` state with no manifest or pid. This poisons Raspberry's
  run-truth state. Discovered and documented in `plans/031926-iterative-execution-and-raspberry-hardening.md`.

Neither of these is in the risk register. They should be added.

---

## Stale or Missing Surfaces Summary

| Surface | State | Action Required |
|---------|-------|-----------------|
| `ops/scorecard.md` | Stale — 3 days out of date with significant bootstrap progress | Refresh to reflect `games:traits` through implementation+verification, `tui/shell`, `chain/runtime`, `chain/pallet` all reviewed |
| `ops/decision_log.md` | Missing | Create and record named decisions from Fabro migration: Fabro as primary executor, multi-program decomposition, direct foreground as detach fallback, MiniMax `bash -ic` workaround |
| `ops/risk_register.md` | Missing new risks | Add R-NEW-1 (MiniMax env-injection) and R-NEW-2 (Raspberry detach false-submit) |
| `ops/instrumentation_backlog.md` | Missing | Create — no instrumentable-metrics backlog exists yet |
| `ops/incidents/` | Missing | Create when first incident occurs |

---

## Next Planning Proposal

**Proposed Slice: Close Tier 3 Ops Surfaces**

Before the next recurring `strategy:planning` run, the following should be
produced by a separate execution lane (not by the strategy lane itself):

1. **Refresh `ops/scorecard.md`** to reflect current bootstrap state:
   - `games:traits` fully implemented and verified
   - `tui/shell`, `chain/runtime`, `chain/pallet` all reviewed
   - `services` and `platform` frontiers marked complete in scorecard
   - Function scorecards updated to reflect actual plan completion state

2. **Create `ops/decision_log.md`** and record at minimum:
   - Decision: Fabro as primary executor (2026-03-18)
   - Decision: Multi-program decomposition (2026-03-19)
   - Decision: Direct foreground execution as detach fallback (2026-03-19)
   - Decision: `bash -ic` launch for MiniMax bridge (2026-03-19)

3. **Add R-NEW-1 and R-NEW-2 to `ops/risk_register.md`**

4. **Create `ops/instrumentation_backlog.md`** — at minimum, a stub that
   names the instrumentable surfaces (Fabro run latency, Raspberry state
   sync lag, lane closure rate) even if no metrics are yet wired up.

This slice is small, bounded, and does not require any code. It closes the
Tier 3 gap that makes the `strategy:planning` lane's recurring runs
produce long reviews full of missing-surface flags. Once the Tier 3 surfaces
are present and current, the recurring `strategy:planning` lane can
focus on its actual mandate: strategic drift detection, not ops archaeology.

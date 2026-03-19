# Strategy Planning Lane — Capability Spec

**Lane:** `strategy:planning`
**Kind:** recurring-oversight
**Owning program:** `myosu-recurring`
**Output root:** `outputs/strategy/planning/`
**Status:** bootstrap-complete (this document)
**Last updated:** 2026-03-19

---

## Purpose / Operator-Facing Outcome

The `strategy:planning` lane is the recurring oversight surface that keeps Myosu's
doctrine aligned with its mission. When it runs, the operator learns whether the
repository's strategic direction is still coherent — whether the mission is still
aimed at the right target, whether stage gates are still meaningful, whether
frontier priorities reflect current evidence, and whether any doctrine surface has
drifted from what was decided.

It does not drive execution. It watches execution and reports when the gap between
what the doctrine says and what the repo is doing becomes material.

---

## Doctrine Surfaces This Lane Reads

The lane reads these surfaces as source truth, in order of authority:

### Tier 1 — Mission and Hard Doctrine (read-only; never writes)

| Surface | Path | What It Provides |
|---------|------|-----------------|
| Mission statement | `OS.md` §Mission | The "why" and the product definition |
| Stage gates | `OS.md` §Company Stages | Exit criteria for stage 0 → 1 → 2 → 3 |
| Hard invariants | `INVARIANTS.md` | Six non-negotiable rules; violations are no-ship events |
| North Star metric | `OS.md` §North Star | `solver_exploitability_convergence`; the single outcome that matters |
| Revenue model | `OS.md` §Revenue Model (Planned) | Planned monetization; informs scope decisions |
| Competitive landscape | `OS.md` §Competitive Landscape | Why no existing tool addresses the same problem |
| Target game list | `OS.md` §Target Games | 20-game priority table; geographic coverage rationale |

### Tier 2 — Spec and Plan Conventions (read-only)

| Surface | Path | What It Provides |
|---------|------|-----------------|
| Spec writing standard | `SPEC.md` | How to author durable decision, migration, and capability specs |
| Plan writing standard | `PLANS.md` | How to author living executable implementation plans |
| Spec index | `specs/031626-00-master-index.md` | Current canonical decision and migration specs; what is active vs. archived |
| Active plans | `plans/*.md` | Current living ExecPlans; what is in flight vs. completed |

### Tier 3 — Operational Signals (read-only; consumed for context)

| Surface | Path | What It Provides |
|---------|------|-----------------|
| KPI registry | `ops/kpi_registry.yaml` | Machine-readable guardrail definitions with green/yellow/red thresholds |
| Scorecard | `ops/scorecard.md` | Human-readable snapshot of current metric states |
| Risk register | `ops/risk_register.md` | Active risks, severity, likelihood, mitigation owners |
| Decision log | `ops/decision_log.md` | Named decisions with rationale, date, and author |
| Incident ledger | `ops/incidents/` | Directory of post-mortem and incident records |

### Tier 4 — Control Plane State (read-only; consumed for context)

| Surface | Path | What It Provides |
|---------|------|-----------------|
| Bootstrap program | `fabro/programs/myosu-bootstrap.yaml` | Which lanes are complete; what bootstrap achieved |
| Frontier programs | `fabro/programs/myosu-*.yaml` | Which frontiers are active, blocked, or complete |
| Reviewed artifacts | `outputs/*/review.md` | Which lane outputs have been reviewed and accepted |
| Fabro execution state | `.raspberry/myosu-*-state.json` | Raspberry-reported state for each frontier |

---

## What This Lane Is Allowed to Propose or Refresh

The lane's mandate is strictly oversight. It may:

1. **Propose new ExecPlans** in `plans/` when a strategic gap is found that no
   current plan addresses. Proposals are written as draft ExecPlans that another
   agent or human can execute. The lane does not execute them.

2. **Propose new decision specs** in `specs/` when a durable architectural or
   strategic choice needs to be recorded. The lane writes the spec skeleton
   and rationale; it does not make the decision — it surfaces the decision
   question with enough context for a human or senior agent to resolve.

3. **Propose updates to Tier 3 operational surfaces** (`ops/scorecard.md`,
   `ops/risk_register.md`, `ops/decision_log.md`) when the current content is
   materially stale or inconsistent with what the lane observes. Updates are
   proposals, not direct edits, unless the change is purely mechanical
   (e.g., updating a last-modified date, correcting a broken link).

4. **Flag drift** — it may call out, in its review artifact, any case where
   the repo's behavior or output artifacts are inconsistent with Tier 1 doctrine.
   It does not fix drift; it names it and escalate via the review artifact.

The lane does **not** directly edit `OS.md`, `INVARIANTS.md`, `SPEC.md`, or
`PLANS.md`. Those are mission-hard surfaces. Changes there require a dedicated
decision spec or migration spec process, not a recurring oversight pass.

---

## Durable Outputs This Lane Writes

Each recurring run produces:

### `outputs/strategy/planning/spec.md` (this document)

The bootstrap run produces the initial spec. Subsequent runs may refresh it when
the strategy planning mandate itself changes — for example, when a new doctrine
surface is added to the repository, or when a new Tier 1 artifact is created
that the lane needs to read.

### `outputs/strategy/planning/review.md`

The per-run review artifact. Produced on every recurring execution. Contains:

- **Doctrine health signal:** Is every Tier 1 and Tier 2 surface present and
  consistent with the last version the lane observed?
- **Strategic drift report:** Are active plans still aimed at the right stage
  gate? Are reviewed lane outputs consistent with declared priorities?
- **Risk drift report:** Are active risks still accurately characterized? Are new
  risks implied by recent progress that are not yet in the register?
- **Next planning proposal:** A short, concrete proposal for the next strategic
  action the repository should take, with enough context to be executable by
  another agent.

---

## What Healthy Recurring Behavior Looks Like

A healthy recurring run of `strategy:planning`:

1. **Completes without surprise.** The lane finds nothing that was not already
   implied by the existing doctrine surfaces. All Tier 1 surfaces are present,
   all Tier 2 surfaces are internally consistent, and the active plans are still
   aimed at the correct next milestone.

2. **Produces a short review.** A healthy review is typically 10–20 lines.
   Long reviews indicate either that the doctrine surfaces are unclear (a
   spec problem) or that real drift has occurred (a execution problem).

3. **Proposes only when warranted.** A healthy lane goes several iterations
   without a new proposal. If every run produces a proposal, either the
   doctrine is too vague or the lane is operating below the doctrine layer
   (doing execution planning instead of strategy oversight).

4. **Marks the reviewed milestone correctly.** The lane's milestone in
   `myosu-recurring.yaml` is `reviewed`. Reaching `reviewed` means the
   review artifact was produced and is consistent with the spec. It does not
   mean the proposals in the review were acted upon — that is a separate
   execution concern.

5. **Does not block other lanes.** `strategy:planning` has no execution
   preconditions beyond the bootstrap program being present. It does not gate
   services, product, or platform progress. Its value is in producing the
   review artifact, not in being a prerequisite.

---

## Smallest Honest Next Implementation Slices

The bootstrap run (this document + the initial review) is Slice 0. It establishes
the contract. The following slices are the honest next steps:

### Slice 1 — First Recurring Run

Run the lane through the bootstrap workflow:

```
fabro run fabro/run-configs/maintenance/strategy-planning.toml
```

Produce the first `review.md`. This establishes the baseline: what the current
doctrine surfaces look like through a strategy lens, and whether any material
drift is already visible.

**Done when:** `outputs/strategy/planning/review.md` exists and is reviewed
against this spec.

### Slice 2 — Close the Missing Ops Surfaces

Several Tier 3 surfaces referenced in `OS.md` doctrine hierarchy do not yet
exist:

- `ops/decision_log.md` — named decisions need to be recorded as they are made
- `ops/instrumentation_backlog.md` — `OS.md` references it but it is not present
- `ops/incidents/` — directory referenced but not present

Slice 2 is not executed by `strategy:planning` itself — it is a proposal
the lane surfaces in its review. A separate lane or agent would execute it.
The `strategy:planning` lane's job is to name the gap and propose the slice.

### Slice 3 — Align the Ops Scorecard with Current Repo State

`ops/scorecard.md` was last updated 2026-03-16. The bootstrap has made
significant progress since then: `games:traits` has completed through
implementation and verification, `tui:shell`, `chain:runtime`, and
`chain:pallet` all have reviewed artifacts, and services and platform
frontiers are now reporting as fully complete. The scorecard's
"Function Scorecards" section still shows `Plan entries completed: 0/82`
which is materially stale.

This slice would produce a refreshed `ops/scorecard.md` that reflects
current repo state. Again, this is a proposal from `strategy:planning`,
not an edit the lane makes directly.

### Slice 4 — Produce a Stage Gate Self-Assessment

The stage 0 exit criteria in `OS.md` are concrete. A recurring
`strategy:planning` run should periodically assess whether any criterion
has been met and recommend a stage transition when appropriate. Slice 4
would produce the first such self-assessment against the bootstrap exit
criteria.

**Done when:** The review artifact contains a named stage gate assessment
with a clear "not yet ready" or "ready to call" signal.

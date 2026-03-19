# `learning:improvement` Lane Specification

## Purpose and Operator-Facing Outcome

`learning:improvement` is the recurring-oversight lane that watches Myosu's execution history and surfaces the highest-signal improvement opportunities back to human operators and downstream lanes.

The lane does not implement improvements — it names them, ranks them by evidence strength, and produces a refreshed improvement log that `strategy:planning` or implementation lanes can consume. Its recurring trigger means it runs on a schedule (not on every commit) so it can accumulate enough execution evidence to separate noise from pattern.

**Operator-visible behavior**: On each run, `review.md` is refreshed with a current trust assessment of all retrospective surfaces. `spec.md` is updated only when the lane definition itself changes. A durable improvement log is written to `outputs/learning/improvement/improvements.md` (see Durable Outputs below).

---

## What the Lane Reads as Doctrine or Source Truth

The lane ingests the following surfaces in priority order:

### Tier 1 — Execution ground truth
- `plans/*.md` Decision Log sections — explicit decisions with date, author, and rationale
- `plans/*.md` Surprises & Discoveries sections — empirical observations from real execution
- `plans/*.md` Outcomes & Retrospective sections — post-slice results vs. stated goals
- `fabro/programs/myosu-*.yaml` — Raspberry manifest state (lane readiness, milestone satisfaction, blocked-by relationships)

### Tier 2 — Reviewed artifact corpus
- `outputs/*/review.md` — per-lane trust assessments; the lane reads these as the audited quality signal
- `outputs/*/spec.md` — per-lane contracts; the lane checks whether spec promises match execution outcomes
- `outputs/*/verification.md` — proof evidence for lanes that have reached verification stage

### Tier 3 — Operational evidence
- `ops/evidence/` — any checked-in run artifacts, debug traces, or incident notes
- `fabro/workflows/*/` — workflow graph structure; changes here indicate lane restructure decisions
- `fabro/run-configs/*/` — run config structure; drift here is an operational smell
- Git log (informative commits only; noisy commits are not signal)

### Tier 4 — Doctrine boundary
- `SPEC.md` (repo root) — defines what a Myosu spec is and how it should be written
- `PLANS.md` (repo root) — defines plan structure and update conventions
- `OS.md` — product mission, stage gates, invariants, no-ship doctrine
- `fabro/programs/myosu-recurring.yaml` — this lane's own manifest; changes here redefine what this lane does

The lane does NOT read external tooling state (e.g., CI results, test覆盖率) unless that state has been explicitly summarized in a Tier 1–3 surface. Raw CI output is not self-documenting enough to serve as doctrine.

---

## What Durable Outputs the Lane Writes or Refreshes

| Output | Purpose | Refresh policy |
|--------|---------|---------------|
| `outputs/learning/improvement/spec.md` | This file; defines the lane contract | Update only when lane definition changes |
| `outputs/learning/improvement/review.md` | Current trust assessment of all retrospective surfaces; improvement signal assessment | **Refresh every run** |
| `outputs/learning/improvement/improvements.md` | Ranked improvement opportunities derived from this run's analysis | **Refresh every run**; append-only (never delete prior entries, but mark stale ones) |
| `outputs/learning/improvement/surface-audit.md` | Per-surface trust log: which surfaces are healthy, which are stale, which are missing | **Refresh every run** |

The improvements log (`improvements.md`) is the primary actionable output. Each entry has:
- **Signal source** — which surface this improvement was derived from
- **Evidence strength** — HIGH (explicit decision with outcome), MEDIUM (surprise/discovery with outcome), LOW (observed without clear outcome yet)
- **Owner** — which lane or operator action owns the improvement
- **Status** — OPEN / IN_PROGRESS / RESOLVED / WONTFIX
- **Last seen** — date of the run that last confirmed this improvement is still relevant

---

## What Healthy Recurring Behavior Means

Healthy recurring behavior for this lane is defined by five observable properties:

### 1. Signal accumulation, not noise multiplication
Each run should surface no more than 5 new improvement entries. If 20 appear, the ingestion logic is undirected and needs tightening. A healthy run surfaces the highest-signal items and suppresses items that were already surfaced in the prior run (mark them as PERSISTED rather than re-entering them).

### 2. Outcome linkage
Every HIGH-signal improvement entry must cite a specific decision log entry or reviewed artifact that produced the signal. Improvements without a named source surface are MEDIUM at best.

### 3. Plan feedback closure
If a prior run surfaced an improvement that was addressed in a plan's Decision Log, the current run should mark that improvement RESOLVED and cite the plan entry. If the improvement was addressed but the Decision Log was not updated, the improvement is marked RESOLVED but a secondary improvement is opened: "Decision Log not updated after improvement closure."

### 4. Stale surface detection
Healthy runs identify at least one surface that has gone stale since the prior run and log it in `surface-audit.md`. A run that finds zero stale surfaces should be suspicious — either the surfaces are genuinely healthy (possible but unlikely), or the lane is not reading enough surfaces.

### 5. No phantom progress
The lane must not mark an improvement RESOLVED without a specific cited resolution event (a merged PR, a closed issue, a decision log entry). An improvement that is "still being worked on" stays OPEN. An improvement that was "mentioned in a recent plan" stays IN_PROGRESS only if there is a linked milestone or slice in that plan.

---

## Smallest Honest Next Implementation Slices

The lane is being bootstrapped now. The following slices define the honest minimum viable recurrence:

### Slice 1: Passive retrospective read (this bootstrap run)
**What**: Ingest `plans/`, `specs/`, `outputs/`, and `ops/` surfaces. Identify the 3 most actionable improvement signals. Write `outputs/learning/improvement/improvements.md` with those 3 entries. Write `outputs/learning/improvement/surface-audit.md` with a per-surface trust assessment.

**What it is NOT**: This slice does not implement any improvement. It does not update any plan. It does not touch any code. It only names and ranks.

**Proof gate**: `outputs/learning/improvement/improvements.md` exists and has exactly 3 entries with signal source, evidence strength, and owner filled in.

### Slice 2: First improvement close loop
**What**: After Slice 1, watch for the first plan that explicitly closes one of the surfaced improvements. When that plan's Decision Log is updated, the next learning:improvement run should mark that improvement RESOLVED in `improvements.md` and verify the Decision Log was updated (if not, open a secondary improvement entry).

**What it is NOT**: The lane does not write the plan's Decision Log. The lane only reads it and reacts.

**Proof gate**: `improvements.md` has at least one RESOLVED entry with a cited plan entry.

### Slice 3: Stale-surface detection hardening
**What**: Add systematic checks for surfaces that were updated in the prior period vs. surfaces that were not. Specifically: flag any `outputs/*/review.md` that has not been refreshed in 4 weeks; flag any `plans/*.md` Decision Log that has no entries in the current month; flag any lane in `fabro/programs/myosu-*.yaml` that has been blocked on the same precondition for more than 2 weeks.

**Proof gate**: `surface-audit.md` has at least one STALE entry with a named surface and age.

### Slice 4: Improvement ranking refinement
**What**: Add an explicit ranking algorithm: sort improvement entries by (evidence_strength × recency). Weight MEDIUM-signal items that have persisted for 3+ runs higher than MEDIUM items just surfaced. De-prioritize items that have been owned by a lane for 2+ runs without movement.

**Proof gate**: The top improvement in `improvements.md` is the one with highest (evidence_strength × recency), verifiable by manual inspection.

### Slice 5: Recurrence scheduling integration
**What**: Wire the lane into Raspberry's recurring dispatch so it runs automatically on a schedule (not just on-demand). This requires `fabro/run-configs/maintenance/learning-improvement.toml` to have a `recurring` block with a `cron` or `interval` field, and the lane's `checks` in `myosu-recurring.yaml` to include a time-based gate.

**Proof gate**: Two back-to-back scheduled runs produce different `review.md` timestamps and different `improvements.md` (at least the `Last seen` field advances).

---

## Dependency on Other Lanes

| Lane / Surface | Dependency type | What is consumed |
|---|---|---|
| `plans/*.md` Decision Logs | Hard upstream | Primary evidence source for HIGH-signal improvements |
| `outputs/*/review.md` | Hard upstream | Per-lane trust signal |
| `fabro/programs/myosu-recurring.yaml` | Self-reference | Lane definition; any change here invalidates this spec |
| `strategy:planning` | Downstream consumer | `improvements.md` is the primary feed for strategy prioritization |
| `ops/evidence/` | Soft upstream | Incident notes and run artifacts; often missing or sparse |

The lane has no hard dependencies on code lanes (`games:traits`, `tui:shell`, etc.). It operates at the doctrine and oversight layer.

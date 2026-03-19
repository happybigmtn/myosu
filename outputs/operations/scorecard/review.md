# `operations:scorecard` Lane Review

**Lane**: `operations:scorecard`
**Frontier**: `recurring`
**Date**: 2026-03-19
**Judgment**: **KEEP lane open; bootstrap the recurring engine starting with Slice 1**

---

## 1. Keep / Reopen / Reset Decision

### Decision: **KEEP** — proceed to bootstrap

This is a fresh lane with no prior implementation attempt. No `outputs/operations/scorecard/` artifacts exist beyond a `.gitkeep`. There is nothing to reopen or reset.

The lane is well-bounded: it reads three existing doctrine surfaces and produces four well-defined outputs. The `ops/` directory surfaces exist and are current enough (2026-03-16) to serve as the initial source of truth.

The lane does **not** require any upstream lane to complete before Slice 1. It is the first recurring oversight lane in `myosu-recurring.yaml` that can be implemented entirely from static files.

---

## 2. Trustworthy Operations Surfaces That Already Exist

### 2.1 `ops/kpi_registry.yaml` — TRUSTED (machine-readable, stable)

This is the most production-ready surface in the ops directory. It is machine-readable, has a consistent schema, and defines exact green/yellow/red thresholds for every KPI.

**Strengths**:
- All KPI IDs are unique and stable
- `cadence` field gives the scorecard lane its heartbeat
- `default_action` provides operator guidance for each threshold violation
- YAML structure is parseable by any standard YAML library

**Weaknesses**:
- No current readings — `structured_closure_rate_7d`, `validator_determinism`, `solver_exploitability_convergence` all return `skipped: no_source` until chain is running
- No machine-readable history; trends must be built by appending to `kpi_trends.yaml` over time

**Verdict**: This surface is the scorecard lane's primary doctrine input. It is trustworthy as a definition file and will become more valuable once the chain provides live readings.

### 2.2 `ops/scorecard.md` — TRUSTED with staleness risk (human-maintained, current)

The scorecard is a well-structured markdown document with three sections: North Star, Guardrails, and Function Scorecards.

**Strengths**:
- Targets are clearly stated per metric
- `false_green_proof_count` is explicitly green (0) — can be evaluated immediately
- Function Scorecards provide qualitative context (stage, top priority, scope risk)
- Last updated 2026-03-16 — 3 days ago at time of writing; not yet stale

**Weaknesses**:
- Function Scorecards are narrative prose — require markdown table parsing to extract machine-readable fields
- No machine-readable timestamp field; staleness must be inferred from `mtime` alone
- "Not measured" status for most guardrails means the lane has nothing to compare against yet

**Verdict**: Trustworthy as a human-maintained snapshot. The lane should parse it literally for Slice 1 and add machine-readable timestamps when producing `scorecard.md`.

### 2.3 `ops/risk_register.md` — TRUSTED with high staleness risk (human-maintained)

Five active risks (R-001 through R-005) with structured fields. The severity scale (S1–S2) and likelihood/impact ratings provide a machine-parseable base.

**Strengths**:
- 5 risks are clearly named and numbered
- Severity, Likelihood, Impact, Mitigation, Owner fields are consistent
- R-004 (Validator Determinism) is marked S1 — correctly identifies the highest-severity risk

**Weaknesses**:
- No machine-readable last_updated timestamp per risk — age must be inferred from the file-level `Last updated: 2026-03-16`
- No `status` field (active/mitigated/closed) — cannot distinguish a risk that was actively managed from one that was forgotten
- No resolution criteria — "mitigation" describes approach but not success metric
- No automated update mechanism; depends entirely on human diligence

**Verdict**: Trustworthy as a snapshot of active risks as of 2026-03-16. The lane should flag any risk older than 30 days as `stale: true` and surface that in `risk_assessment.md`.

### 2.4 `fabro/programs/myosu-recurring.yaml` — TRUSTED (Raspberry program manifest)

The manifest correctly defines the `operations` unit with `spec` and `review` artifacts and the `scorecard` lane as `recurring` kind. The precondition (`bootstrap_program_present`) is satisfied.

**Verdict**: The manifest is correct. No changes needed.

### 2.5 `fabro/run-configs/maintenance/operations-scorecard.toml` — TRUSTED (run config)

The run config correctly references the workflow graph and sets `goal_gate: true` requiring both `spec.md` and `review.md`. The `directory = "../.."` correctly points to the repo root.

**Verdict**: The run config is correct. No changes needed.

---

## 3. Stale or Missing Scorecard Surfaces

### 3.1 Stale: `ops/scorecard.md` Function Scorecards section

The Function Scorecards (Execution/Dev, Strategy, Security) are prose-heavy and not machine-comparable. "Plan entries completed: 0/82" is a number embedded in prose.

**Impact**: The scorecard lane cannot evaluate whether the function scorecards improved or degraded without manually parsing prose.

**Remedy**: When these sections need to be tracked by the scorecard lane, they should be migrated to structured format (YAML or markdown tables). For now, the lane will copy them verbatim into `scorecard.md`.

### 3.2 Missing: Machine-readable history for KPIs

`ops/kpi_registry.yaml` defines KPIs but has no mechanism to store historical readings. The `kpi_trends.yaml` output file does not yet exist.

**Impact**: Without trend data, the scorecard lane cannot show whether a guardrail is improving or degrading — only whether it is currently green/yellow/red.

**Remedy**: Slice 2 of the lane implementation creates `kpi_trends.yaml` and appends readings on each run. This is the highest-value durable output the lane can produce.

### 3.3 Missing: Machine-readable risk status

`ops/risk_register.md` has no per-risk `status` field (active/mitigated/closed) and no machine-readable last-updated date per risk entry.

**Impact**: The scorecard lane cannot distinguish an actively-managed risk from an abandoned one. It can only flag risks by file age.

**Remedy**: `ops/risk_register.md` should be migrated to YAML with per-risk `status` and `updated` fields. The scorecard lane's `risk_assessment.md` output will serve as the machine-readable overlay on top of the human-maintained markdown.

### 3.4 Missing: `outputs/operations/scorecard/` artifact directory

Only a `.gitkeep` exists in `outputs/operations/scorecard/`. No scorecard, trends, risk assessment, or health check artifacts exist yet.

**Impact**: No history, no baseline to compare against.

**Remedy**: Slice 1 of the lane implementation produces the first `scorecard.md` and `health_check.md`.

---

## 4. Next Honest Recurring Implementation Slice

### Slice 1 is the immediate next step

**What to implement**: Doctrine reader + static refresh

**Owned files**:
- A new Fabro prompt or script that reads `ops/scorecard.md`, `ops/kpi_registry.yaml`, `ops/risk_register.md` and produces `outputs/operations/scorecard/scorecard.md` and `outputs/operations/scorecard/health_check.md`

**Observable**:
- `outputs/operations/scorecard/scorecard.md` exists and matches the structure of `ops/scorecard.md`
- `outputs/operations/scorecard/health_check.md` contains `status: pass` and lists all three verified source files

**What this proves**:
- The lane can read all three doctrine surfaces without error
- The lane can write to `outputs/operations/scorecard/` durably
- The `goal_gate` in `operations-scorecard.toml` will pass after this slice completes

**What this does NOT prove** (deferred to later slices):
- KPI threshold evaluation (Slice 2)
- Staleness detection (Slice 3)
- Risk re-evaluation (Slice 4)
- Chain-connected KPI reads (Slice 5)

**Estimated complexity**: Low. This is a file-read + file-write operation with no external dependencies.

---

## 5. Remaining Blockers

### No blockers for Slice 1

All three doctrine files exist and are parseable. The output directory exists (`.gitkeep` present). No upstream lane completion is required.

### Blockers for Slice 5 (Chain-connected KPI reads)

| Blocker | Owner | Description |
|---------|-------|-------------|
| `chain:runtime` restart | `chain:runtime` lane | Must complete before `validator_determinism` and `solver_exploitability_convergence` can be measured |
| `chain:pallet` restart | `chain:pallet` lane | Must complete before on-chain validator state is queryable |

These blockers are correctly reflected in the `ops/scorecard.md` as "Not measured" and in `ops/kpi_registry.yaml` with `cadence: per_tempo`. The scorecard lane handles them correctly by emitting `skipped: no_source` entries.

---

## 6. Lane Readiness

| Dimension | Status | Notes |
|-----------|--------|-------|
| Doctrine surfaces (ops/) | **READY** | All three files exist; kpi_registry.yaml is machine-readable |
| KPI definitions | **READY** | `kpi_registry.yaml` defines all thresholds; cadence values define heartbeat |
| Risk register | **READY** | 5 risks defined; age-based staleness detection is sufficient for now |
| Fabro workflow + run-config | **READY** | `operations-scorecard.fabro` and `operations-scorecard.toml` are correct |
| Raspberry program manifest | **READY** | `myosu-recurring.yaml` correctly registers this lane |
| Output artifacts | **MISSING** | Slice 1 produces first artifacts |
| Recurring engine | **MISSING** | No cron/interval scheduling; first slice is one-shot |
| Chain-connected readings | **BLOCKED** | Slice 5 blocked on chain:runtime restart |

---

## 7. Recommendation

**Proceed to Slice 1 immediately.** The lane is clean, the doctrine surfaces are trustworthy, and there are no blockers.

Slice 1 should be implemented as a Fabro workflow that:
1. Reads all three `ops/` files
2. Validates they are parseable
3. Writes `outputs/operations/scorecard/scorecard.md` (static refresh)
4. Writes `outputs/operations/scorecard/health_check.md` with `status: pass`

Once Slice 1 is complete, the `goal_gate` in `operations-scorecard.toml` will be satisfied and the lane can be marked `reviewed` in Raspberry.

Slice 2 (KPI threshold evaluation + trend append) should follow immediately, as it produces the highest-value durable output (`kpi_trends.yaml`) and requires no additional dependencies.

# `operations:scorecard` Lane Specification

**Lane**: `operations:scorecard`
**Frontier**: `recurring`
**Date**: 2026-03-19
**Status**: Bootstrap — lane boundary defined; no recurring engine implemented yet

---

## 1. Purpose and Operator-Visible Outcome

### Purpose

`operations:scorecard` is the **recurring oversight lane** for Myosu's operational health. It periodically reads the live doctrine surfaces in `ops/` and produces refreshed, machine-comparable scorecard artifacts in `outputs/operations/scorecard/`.

The lane answers the question: *Are the guardrails green, yellow, or red, and what changed since last run?*

### Operator-Visible Outcome (once implemented)

A Raspberry supervisor runs this lane on a cadence (daily). Each run produces:

- A refreshed `scorecard.md` with current vs. target values filled in
- A `kpi_trends.yaml` appending the latest KPI readings with timestamps
- A `risk_assessment.md` re-evaluating each risk entry against current state

The lane does **not** modify `ops/` source files — those remain human-maintained. The lane produces *refreshed copies* in `outputs/operations/scorecard/` that reflect the current state derived from reading the source doctrine.

---

## 2. Lane Boundary

```
                          operations:scorecard (THIS LANE)
                          ┌──────────────────────────────────────────────────┐
                          │                                                      │
doctrine (read)           │  ops/scorecard.md        ops/kpi_registry.yaml    │
  (ops/)      ──────────► │  ops/risk_register.md                              │
                          │                                                      │
                          │  ┌────────────────────────────────────────────┐  │
                          │  │  ScorecardRefreshEngine                      │  │
                          │  │  - parse kpi_registry.yaml                   │  │
                          │  │  - evaluate guardrail thresholds             │  │
                          │  │  - compare to ops/scorecard.md targets       │  │
                          │  │  - detect stale entries (age > cadence × 2)  │  │
                          │  └────────────────────────────────────────────┘  │
                          │                                                      │
                          │  outputs/operations/scorecard/                     │
                          │    scorecard.md        (refreshed)                 │
                          │    kpi_trends.yaml    (appended)                  │
                          │    risk_assessment.md (re-evaluated)               │
                          └──────────────────────────────────────────────────┘
```

**Read-only doctrine inputs (ops/)**:
- `ops/scorecard.md` — North Star metrics, Guardrail status, Function Scorecards; human-maintained
- `ops/kpi_registry.yaml` — machine-readable KPI definitions with green/yellow/red thresholds and formulas
- `ops/risk_register.md` — active risk entries (R-001 through R-005) with severity, likelihood, impact

**Durable outputs**:
- `outputs/operations/scorecard/scorecard.md` — machine-refreshed scorecard derived from source doctrine
- `outputs/operations/scorecard/kpi_trends.yaml` — timestamped KPI readings appended on each run
- `outputs/operations/scorecard/risk_assessment.md` — current risk status derived from risk_register.md
- `outputs/operations/scorecard/health_check.md` — run-level health summary: pass/fail/warn per guardrail

**Not owned by this lane**:
- `ops/scorecard.md`, `ops/kpi_registry.yaml`, `ops/risk_register.md` — human-maintained source of truth; the lane reads but does not edit
- Any chain, game, or service code

---

## 3. Current State of Doctrine Surfaces

### 3.1 `ops/scorecard.md` — Last updated 2026-03-16

**Content**:
- **North Star**: `solver_exploitability_convergence` — target <= 50 mbb/hand, currently "Not measured" (no chain yet)
- **Guardrails**: `false_green_proof_count` (green: 0), `structured_closure_rate_7d` (N/A), `validator_determinism` (N/A), `solver_gameplay_separation` (N/A)
- **Function Scorecards**: Execution/Dev (0/82 plans completed), Strategy (stage_0_bootstrap), Security (0 open findings)

**Health signals**: Green for `false_green_proof_count`; everything else "Not measured". The function scorecards are narrative, not machine-readable.

**Staleness risk**: The file is 3 days old at time of writing. No automated refresh mechanism.

### 3.2 `ops/kpi_registry.yaml` — Machine-readable, complete

**Content**: 1 north_star KPI + 4 guardrail KPIs. Each has `id`, `definition`, `formula`, `source`, `cadence`, `green`, `yellow`, `red`, and `default_action`.

**Status**: This is the most machine-ready surface in the ops directory. The cadence values (`per_tempo`, `daily`, `per_build`) give the scorecard lane its heartbeat.

**Staleness risk**: Low — YAML structure is stable; only manual edits required when new KPIs are added.

### 3.3 `ops/risk_register.md` — Last updated 2026-03-16

**Content**: 5 active risks (R-001 through R-005) with Severity, Likelihood, Impact, Mitigation, Owner fields.

**Status**: Human-maintained prose table. No machine-readable equivalent. The scorecard lane must parse the markdown table to extract risk state.

**Staleness risk**: High — no automation; relies on manual updates after risk state changes.

---

## 4. What Healthy Recurring Behavior Means

The lane is **healthy** when every run:

1. **Reads fresh** — All three doctrine files are parsed without error. If a file is missing or malformed, the run produces a `health_check.md` with `status: warn` and a descriptive message.

2. **Compares to targets** — Each KPI from `kpi_registry.yaml` is evaluated against its green/yellow/red thresholds using the formula. The result is written to `scorecard.md` with explicit status per guardrail.

3. **Appends trends** — A timestamped reading is appended to `kpi_trends.yaml`. If a KPI has no current reading (e.g., `validator_determinism` before chain is running), the entry is `{ "skipped": "no_source", "reason": "..." }`.

4. **Detects staleness** — If any doctrine file has not been modified within `2 × cadence` period, the run writes `health_check.md` with `status: warn` and identifies the stale surface.

5. **Re-evaluates risks** — Each risk entry from `risk_register.md` is included in `risk_assessment.md` with a freshness timestamp. Risks older than 30 days without a status change are flagged `stale: true`.

6. **Never overwrites source** — The lane writes exclusively to `outputs/operations/scorecard/`. `ops/` remains the human-maintained source of truth.

**The lane is not responsible for**:
- Fixing guardrails that are yellow or red
- Closing risks that are resolved
- Updating the function scorecards (those require human judgment)

---

## 5. Owned Files and Outputs

### 5.1 Durable Outputs (written by the lane)

| Output file | Format | Purpose |
|-------------|--------|---------|
| `scorecard.md` | Markdown | Refreshed scorecard — North Star, Guardrails, Function Scorecards with current readings filled in |
| `kpi_trends.yaml` | YAML | Append-only time-series of KPI readings; one run = one appended entry |
| `risk_assessment.md` | Markdown | Risk register re-typed with current staleness flags and age |
| `health_check.md` | YAML | Run-level health: `status: pass\|warn\|fail`; per-guardrail status; stale surface list |

### 5.2 Internal Artifacts (not durable contracts)

| Path | Purpose |
|------|---------|
| `fabro/run-configs/maintenance/operations-scorecard.toml` | Lane run configuration |
| `fabro/workflows/maintenance/operations-scorecard.fabro` | Lane workflow graph |
| `fabro/prompts/maintenance/recurring-inventory.md` | Inventory prompt |
| `fabro/prompts/maintenance/recurring-review.md` | Review prompt |

---

## 6. Dependency on Other Lanes

| Lane | Direction | Nature |
|------|---------|--------|
| `bootstrap` | Precondition | `fabro/programs/myosu-bootstrap.yaml` must exist (checked by Raspberry) |
| `chain:runtime` | Soft upstream | Provides `validator_determinism` and `solver_exploitability_convergence` readings once chain exists |
| `strategy:planning` | Peer | Both are recurring lanes under `myosu-recurring.yaml`; share `ops/` directory but own different outputs |
| `security:audit` | Peer | Peer recurring lane; shares risk_register.md as input |

No other lane blocks this lane from running its first slice. The lane reads static files and produces static outputs. Chain connectivity is only required for KPIs that measure chain-provided signals.

---

## 7. Implementation Slices (Smallest Honest First)

### Slice 1: Doctrine Reader + Static Refresh

**Goal**: Read all three ops doctrine files and produce a static `scorecard.md` copy in `outputs/operations/scorecard/` without any chain connectivity.

**What**:
- Parse `ops/kpi_registry.yaml` → extract all KPI definitions
- Parse `ops/scorecard.md` → extract current readings and targets
- Parse `ops/risk_register.md` → extract risk entries
- Produce `outputs/operations/scorecard/scorecard.md` (static snapshot, same content as source, same structure)
- Produce `outputs/operations/scorecard/health_check.md` with `status: pass` and `sources_verified: [scorecard.md, kpi_registry.yaml, risk_register.md]`

**Proof gate**: Running the lane twice produces identical `scorecard.md`; `health_check.md` shows `status: pass`.

**Blockers**: None.

---

### Slice 2: KPI Threshold Evaluation + Trend Append

**Goal**: Evaluate each guardrail KPI against its green/yellow/red thresholds and append a timestamped reading to `kpi_trends.yaml`.

**What**:
- For each KPI in `kpi_registry.yaml`, evaluate the guardrail formula against current source data
- Since no chain exists yet, `solver_exploitability_convergence`, `validator_determinism`, and `structured_closure_rate_7d` produce `skipped: no_source` entries
- `false_green_proof_count` can be evaluated: source is `ops/scorecard.md` (always 0 currently), so this is green
- Append `{ timestamp, kpi_id, status, value, source }` to `kpi_trends.yaml`

**Observable**: `kpi_trends.yaml` grows one entry per run with correct green/yellow/red evaluation.

**Proof gate**: Two consecutive runs produce two appended entries in `kpi_trends.yaml` with incrementing timestamps; skipped KPIs have `skipped: no_source`.

**Blockers**: None for the evaluation and append logic; chain-dependent KPIs will always return `skipped` until chain:runtime provides the data.

---

### Slice 3: Staleness Detection

**Goal**: Produce `health_check.md` with `status: warn` when any doctrine file is older than `2 × cadence`.

**What**:
- For each source file, check `mtime`
- Map cadence from `kpi_registry.yaml`: `per_tempo` → 5 minutes, `daily` → 2 days, `per_build` → 1 day
- If file age > `2 × cadence`, set `stale: true` for that surface in `health_check.md`
- If any surface is stale, overall `health_check.status` → `warn`
- If a source file is missing, `health_check.status` → `fail`

**Observable**: `health_check.md` correctly identifies stale surfaces after the cadence window passes.

**Proof gate**: Touch a source file to set its mtime to 3 days ago; run the lane; `health_check.md` shows that file as `stale: true` and overall `status: warn`.

**Blockers**: None.

---

### Slice 4: Risk Re-Evaluation + Risk Assessment Output

**Goal**: Produce `risk_assessment.md` with staleness flags for risk entries older than 30 days.

**What**:
- Parse `ops/risk_register.md` for R-001 through R-005
- For each risk, compute `age_days = today - last_updated`
- If `age_days > 30`, set `stale: true`
- Format as `risk_assessment.md` with table: `ID | Severity | Likelihood | Impact | Age Days | Stale | Mitigation`
- Append to `risk_assessment.md` (never overwrite previous assessments; add new section with timestamp)

**Observable**: `risk_assessment.md` shows all 5 risks with age, and flags any older than 30 days.

**Proof gate**: `risk_assessment.md` exists and contains all 5 risk entries with correct age computation; risks older than 30 days are flagged `stale: true`.

**Blockers**: None.

---

### Slice 5: Chain-Connected KPI Reads (Post chain:runtime restart)

**Goal**: Replace `skipped: no_source` entries with real readings from chain RPC once `chain:runtime` provides validator logs and exploitability data.

**What**:
- For `validator_determinism`: query validator logs via RPC, compute max divergence
- For `solver_exploitability_convergence`: query miner strategy scores, compute min exploitability
- For `structured_closure_rate_7d`: query adjudication history, compute ratio
- Update `kpi_trends.yaml` to replace `skipped` with real `{ value, source: chain_rpc_url }`

**Observable**: After chain:runtime restart completes, `kpi_trends.yaml` entries for chain-dependent KPIs contain numeric values instead of `skipped`.

**Proof gate**: After chain:runtime produces a devnet, run the lane; `kpi_trends.yaml` shows numeric values for all KPIs.

**Blockers**: `chain:runtime` must restart and produce a devnet. This is the natural blocker for this slice.

---

## 8. Proof / Check Shape for the Lane

The lane is **proven** when:

```
# Slice 1: Static refresh
test -f outputs/operations/scorecard/scorecard.md
test -f outputs/operations/scorecard/health_check.md
grep "status: pass" outputs/operations/scorecard/health_check.md

# Slice 2: KPI trends append
test -f outputs/operations/scorecard/kpi_trends.yaml
# Two consecutive runs produce two entries with different timestamps

# Slice 3: Staleness detection
# Touch ops/scorecard.md to 3 days ago
# Run lane → health_check.md shows stale: true for that file

# Slice 4: Risk assessment
test -f outputs/operations/scorecard/risk_assessment.md
grep "R-001" outputs/operations/scorecard/risk_assessment.md
# All 5 risks present

# Slice 5: Chain-connected (post chain:runtime)
# chain:runtime devnet running
# kpi_trends.yaml shows numeric values for solver_exploitability_convergence
```

---

## 9. Relationship to Other Recurring Lanes

| Lane | Direction | Shared surface |
|------|-----------|---------------|
| `strategy:planning` | Peer | Both read `ops/scorecard.md`; own different outputs |
| `security:audit` | Peer | Reads `ops/risk_register.md`; owns `outputs/security/audit/` |
| `learning:improvement` | Peer | No shared inputs yet |

All four recurring lanes live under `myosu-recurring.yaml` and are orchestrated by Raspberry. They are independent and can run in parallel.

---

## 10. Phase Ordering

```
Phase 1 (Bootstrap — depends on nothing):
  Slice 1 → Slice 2 → Slice 3 → Slice 4

Phase 2 (Chain-connected — depends on chain:runtime restart):
  Slice 5
```

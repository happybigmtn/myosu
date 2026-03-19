# Security Audit Lane Spec

**Lane**: `security:audit`
**Date**: 2026-03-19
**Status**: Bootstrap — recurring lane contract defined; implementation deferred to maintenance conformance

---

## 1. Operator-Facing Purpose

The `security:audit` recurring lane is the ongoing integrity review body for the Myosu project. It runs on a cadence (default: weekly) and answers one question: **is the system still behaving according to its stated security invariants?**

The lane does not fix code. It produces signed attestations — `review.md` documents — that rate the health of each security surface, flag regressions, and escalate confirmed violations to the ops lane for remediation.

**Operator-visible outcome**: On each run, `outputs/security/audit/review.md` is refreshed with a new dated snapshot. Any `S0` finding triggers an immediate alert. Any `S1` finding is listed with a remediation owner and a fix-by date.

---

## 2. Source Truth — What the Lane Reads as Doctrine

The lane reads the following files as authoritative security ground truth, in priority order:

### Priority 1 — Contract (never stale; require explicit override to change)

| File | Doctrine role |
|------|--------------|
| `INVARIANTS.md` | Hard security rules. Each invariant states a falsifiable claim, a measurement method, and a no-ship severity. The lane verifies the measurement, not the intent. |
| `ops/risk_register.md` | Active risk log. Severity and likelihood scores are the lane's signal prioritization input. R-004 (validator determinism) is the highest-priority item by current severity. |

### Priority 2 — Codebase surfaces under audit

| File / Path | What the lane checks |
|-------------|---------------------|
| `crates/myosu-miner/src/` | Proof-of-training honesty, checkpoint integrity, no secret exfiltration |
| `crates/myosu-validator/src/` | Determinism (INV-003), weight-submission honesty, no spurious slashing |
| `crates/myosu-chain/pallets/game-solver/src/` | External-call hygiene (CEI), storage access control, no reentrancy |
| `crates/myosu-chain/runtime/src/` | Runtime upgrade hygiene, sudo access, genesis initialization |
| `crates/myosu-games-poker/` | MCCFR algorithm integrity, fixed-point arithmetic soundness |
| `fabro/programs/myosu.yaml` + `fabro/programs/myosu-recurring.yaml` | Lane ownership boundaries are respected; no cross-lane coupling without an declared interface contract |
| `outputs/chain/runtime/review.md` | The chain runtime's own security posture (from its lane review) |
| `outputs/chain/pallet/review.md` | The pallet's own security posture |
| `outputs/miner/service/review.md` | The miner service's own security posture |
| `outputs/validator/oracle/review.md` | The validator oracle's own security posture |

### Priority 3 — External / dependency surfaces (read-only, no write authority)

| Surface | Check |
|---------|-------|
| `Cargo.lock` | New dependency additions are reviewed for supply-chain risk |
| `robopoker` fork (RF-02) | CHANGELOG.md reviewed for core MCCFR changes (INV-006) |
| polkadot-sdk git ref | Version pinned in `Cargo.toml`; no unpinned `git = "https://..."` refs |

---

## 3. Durable Outputs — What the Lane Must Write or Refresh

### 3.1 `outputs/security/audit/review.md` (refreshed every run)

The lane's primary output. A dated snapshot with the following sections:

```
## Audit Run: YYYY-MM-DD

### Invariant Health Matrix

| Invariant | Last checked | Status | Notes |
|-----------|-------------|--------|-------|
| INV-001 | 2026-03-19 | PASS / FAIL / DEGRADED | ... |
| INV-002 | 2026-03-19 | PASS / FAIL / DEGRADED | ... |
| INV-003 | 2026-03-19 | PASS / FAIL / DEGRADED | ... |
| INV-004 | 2026-03-19 | PASS / FAIL / DEGRADED | ... |
| INV-005 | 2026-03-19 | PASS / FAIL / DEGRADED | ... |
| INV-006 | 2026-03-19 | PASS / FAIL / DEGRADED | ... |

### Active Risk Health

| Risk | Severity | Trend | Notes |
|------|----------|-------|-------|
| R-001 | S2 | stable / worsening / improving | ... |
| R-002 | S2 | ... | ... |
| R-003 | S2 | ... | ... |
| R-004 | S1 | ... | ... |
| R-005 | S2 | ... | ... |

### New Findings

| ID | Severity | Surface | Description | Remediation owner | Fix-by |
|----|----------|---------|-------------|-------------------|--------|
| SF-01 | S1 | validator scoring | ... | validator-oracle lane | 2026-03-26 |

### Attestation

This review was produced by `security:audit` lane run on YYYY-MM-DD.
Proof-of-run: <git commit hash at time of run>
Source surfaces checked: <list>
No external network access was used during this audit run.
```

### 3.2 `outputs/security/audit/attestation.md` (append-only log)

Each run appends a one-line attestation entry:

```
YYYY-MM-DD | <git-hash> | INV-001:PASS INV-002:PASS INV-003:FAIL ... | 1 new finding (SF-01 S1)
```

This log is never edited after append. It provides an immutable audit trail for compliance.

---

## 4. What Healthy Recurring Behavior Looks Like

### Cadence

Default: **weekly**. Runs every Monday 09:00 UTC. Can be triggered ad-hoc by any `S0` or `S1` event.

### Run contract

Each run must:

1. **Read all Priority 1 and Priority 2 surfaces** listed above at the git commit hash recorded at run start.
2. **Measure each invariant** using its defined measurement method from `INVARIANTS.md`.
3. **Compare to last run's `review.md`** — flag any regression (e.g., a surface that was PASS and is now DEGRADED).
4. **Append to `attestation.md`**.
5. **Refresh `review.md`** with new snapshot.
6. **Alert** if any `S0` finding exists (ops lane notified via direct message).
7. **Escalate** if any `S1` finding is older than its fix-by date (ops lane notified).

### What "healthy" means in steady state

In steady state, a healthy recurring run produces:
- All invariants: `PASS`
- All active risks: `stable` or `improving` trend
- Zero new findings per run
- `attestation.md` entries show consistent green across runs

A run with `DEGRADED` on any invariant is still considered operational but flagged — the ops lane must schedule remediation before the next run.

A run with `FAIL` on any invariant is a **no-ship event** — the ops lane is notified immediately, the affected lane is suspended until the invariant is restored.

---

## 5. Smallest Honest Next Implementation Slices

The lane spec is bootstrapped. The recurring engine (the thing that actually runs on a cadence) does not exist yet. The following slices produce something runnable:

### Slice 0 — Manual audit script (no engine yet)

**Goal**: Produce a runnable shell script that can be executed manually and produces a valid `review.md` snapshot.

**File**: `fabro/checks/security-audit.sh`

**What it does**:
1. Records git commit hash
2. Checks each invariant using its defined measurement (e.g., runs `cargo tree -p myosu-miner --no-dependencies` to verify INV-004 separation, runs deterministic test to verify INV-003)
3. Compares output to previous `review.md` if it exists
4. Writes new `outputs/security/audit/review.md`
5. Appends to `outputs/security/audit/attestation.md`

**Proof**: Running `fabro/checks/security-audit.sh` produces a valid `review.md` and a new `attestation.md` entry. No engine required.

**Blockers**: None.

---

### Slice 1 — Fabro lane wiring

**Goal**: Wire `security-audit.sh` as a `proof` check in the `security:audit` lane in `myosu-recurring.yaml`.

**File**: `fabro/programs/myosu-recurring.yaml` (update `checks` on the `security` unit)

**Change**:
```yaml
checks:
  - label: chain_core_runtime_review_ready
    kind: precondition
    type: file_exists
    path: ../../outputs/chain/runtime/review.md
  - label: security_audit_runs
    kind: proof
    type: command_succeeds
    command: ./fabro/checks/security-audit.sh
```

**Proof**: `cargo run --manifest-path fabro.toml -- check security:audit` exits 0.

**Blockers**: Slice 0 must be complete first.

---

### Slice 2 — Cadence engine

**Goal**: Replace the manual script trigger with a Raspberry-managed recurring execution.

This is owned by the Raspberry/Fabro maintenance lane, not `security:audit`. The `security:audit` lane spec defines the contract; the cadence engine consumes it. This slice is documented here for completeness but is **out of scope for this lane's bootstrap**.

---

## 6. Lane Boundary

### What this lane OWNS

- `outputs/security/audit/spec.md` — this document
- `outputs/security/audit/review.md` — recurring review snapshot (refreshed every run)
- `outputs/security/audit/attestation.md` — append-only attestation log
- `fabro/checks/security-audit.sh` — the audit run script

### What this lane DOES NOT OWN

- Any crate source code
- Any other lane's `spec.md` or `review.md`
- The cadence engine (Raspberry concern)
- Remediation of findings (ops lane concern)

### Interface contracts

| Interface | Type | Defined by | Consumed by |
|-----------|------|-----------|-------------|
| `INVARIANTS.md` | Doctrine input | Myosu project | `security:audit` lane (read-only) |
| `ops/risk_register.md` | Risk input | Myosu project | `security:audit` lane (read-only) |
| `outputs/*/review.md` | Surface health input | Respective lane | `security:audit` lane (read-only) |
| `review.md` refresh | Doctrine output | `security:audit` | Ops lane (reads) |
| `attestation.md` append | Audit trail | `security:audit` | Ops lane, compliance |
| S0/S1 alert | Signal | `security:audit` | Ops lane |

---

## 7. Relationship to Other Lanes

| Lane | Direction | Nature |
|------|-----------|--------|
| `strategy:planning` | Parallel | Both are recurring oversight lanes; share `ops` escalation path |
| `operations:scorecard` | Downstream consumer | Reads `review.md` to drive remediation prioritization |
| `learning:improvement` | Parallel | Both are recurring oversight lanes |
| `chain:pallet` | Upstream surface | The pallet's `review.md` is a Priority 2 input |
| `chain:runtime` | Upstream surface | The runtime's `review.md` is a Priority 2 input |
| `miner:service` | Upstream surface | The miner's `review.md` is a Priority 2 input |
| `validator:oracle` | Upstream surface | The validator's `review.md` is a Priority 2 input |

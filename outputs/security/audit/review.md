# Security Audit Review

**Lane**: `security:audit`
**Date**: 2026-03-19
**Bootstrap run**: Yes — this is the initial audit snapshot produced during lane bootstrap

---

## Audit Run: 2026-03-19

### Invariant Health Matrix

| Invariant | Last checked | Status | Notes |
|-----------|-------------|--------|-------|
| INV-001: Structured Closure Honesty | 2026-03-19 | **DEGRADED** | No adjudicator or supervisor crate exists yet. No `RESULT:`/`BLOCKED:` outcome tracking system has been implemented. The closure contract defined in `WORKFLOW.md` is referenced in the invariant but `WORKFLOW.md` does not exist in the repository. |
| INV-002: Proof Honesty | 2026-03-19 | **DEGRADED** | No proof-gating infrastructure exists. No `false_green_proof_count` measurement is possible because no named proof commands have been implemented. |
| INV-003: Game Verification Determinism | 2026-03-19 | **DEGRADED** | `poker_exploitability()` and `remote_poker_exploitability()` are referenced in `outputs/validator/oracle/spec.md` but the `games:poker-engine` lane has not been implemented. No deterministic test (VO-03) exists. R-004 is the highest-severity active risk (S1). |
| INV-004: Solver-Gameplay Separation | 2026-03-19 | **PASS** | `cargo tree -p myosu-miner -p myosu-play` shows no path dependency between `myosu-miner` and `myosu-play`. The two crates are separate workspace members. The invariant is structurally upheld by workspace isolation. |
| INV-005: Plan And Land Coherence | 2026-03-19 | **DEGRADED** | `IMPLEMENT.md` does not exist. No git land workflow has been established. `WORKFLOW.md` referenced in INV-005 also does not exist. Rollback-on-land-failure behavior is undefined. |
| INV-006: Robopoker Fork Coherence | 2026-03-19 | **DEGRADED** | `happybigmtn/robopoker` fork exists but no `CHANGELOG.md` documents changes from `v1.0.0`. The fork's `Cargo.toml` pins to `robopoker = { git = "..." }` with no tag — version is unpinned. INV-006 enforcement (CHANGELOG.md + git tag pinning) is not implemented. |

### Active Risk Health

| Risk | Severity | Trend | Notes |
|------|----------|-------|-------|
| R-001: Subtensor Fork Complexity | S2 | **stable** | Runtime restart lane is in progress. Minimal runtime strategy is the agreed mitigation. |
| R-002: Robopoker API Stability | S2 | **worsening** | No CHANGELOG.md in fork. No upstream contributions documented. RF-02 (`NlheEncoder::from_dir()`) not confirmed implemented. |
| R-003: Exploitability Computation Cost | S2 | **stable** | No benchmark data yet. Mitigation (sample-based computation) not yet implemented. |
| R-004: Validator Determinism | S1 | **worsening** | `games:poker-engine` lane not started. INV-003 is degraded. R-004 is the most critical active risk. |
| R-005: Miner Gaming | S2 | **stable** | Commit-reveal design referenced in `validator:oracle` spec. Not yet implemented. |

---

## Trustworthy Security Surfaces That Already Exist

### 1. Workspace crate isolation (INV-004 PASS)

`myosu-miner` and `myosu-play` are separate workspace members. `cargo tree` confirms no import path between them. The solver-gameplay separation is structurally enforced by the Rust module system. This is the most mature security control in the project.

### 2. Invariants document (INV-001..INV-006)

`INVARIANTS.md` defines falsifiable security claims with explicit measurement methods, no-ship severities, and fallback modes. This is a strong foundation for recurring audit. The definitions are precise enough to write automated checks against (e.g., `cargo tree` for INV-004, git diff for INV-006 CHANGELOG.md presence).

### 3. Risk register (ops/risk_register.md)

R-004 is correctly identified as S1. The risk register is current (updated 2026-03-16). The severity/likelihood/impact framework is sound and consistent with the invariant no-ship rules.

### 4. Fabro lane ownership boundaries

`fabro/programs/myosu-recurring.yaml` correctly defines `security:audit` as its own unit with `produces: [spec, review]`. The lane interface contract is well-structured. Cross-lane coupling is declared via explicit `depends_on` in other programs.

### 5. Pallet restart boundary definition

`outputs/chain/pallet/spec.md` correctly identifies what is and is not salvageable from the subtensor fork transplant. The no-ship rule mapping (S0 for INV-003 validator disagreement, S1 for INV-001/INV-002/INV-004/INV-005 violations) is explicitly stated.

---

## Stale or Missing Audit Surfaces

### S-01: No proof-of-training or proof-of-closure infrastructure (INV-001, INV-002)

**Severity**: S1
**Surface**: `crates/myosu-miner/`, `fabro/programs/`
**Problem**: INV-001 requires a `RESULT:` or `BLOCKED:` structured outcome system. INV-002 requires named proof commands that actually execute. Neither exists. The adjudicator, supervisor, or delivery contract referenced in `INVARIANTS.md` are not implemented.
**Remediation owner**: `miner:service` lane for the miner-side proof surface; Fabro lane infrastructure for the adjudicator
**Fix-by**: 2026-03-26 (end of first cadence cycle)

### S-02: `games:poker-engine` lane not started — INV-003 degraded, R-004 unmitigated (INV-003)

**Severity**: S1
**Surface**: `crates/myosu-games-poker/`, `games:poker-engine` lane
**Problem**: INV-003 (validator determinism) requires deterministic exploitability computation. The `poker_exploitability()` function referenced in `validator:oracle` spec does not exist in any compiled crate. R-004 (S1) has no mitigation implemented.
**Remediation owner**: `games:poker-engine` lane (not yet started)
**Fix-by**: 2026-04-02 (second cadence cycle — requires lane to be started first)

### S-03: Robopoker fork has no CHANGELOG.md — INV-006 not enforced (INV-006)

**Severity**: S2
**Surface**: `robopoker` fork, `happybigmtn/robopoker`
**Problem**: INV-006 requires `CHANGELOG.md` in the fork documenting all changes from `v1.0.0`. No such file exists. Fork's `Cargo.toml` does not pin to a git tag.
**Remediation owner**: External / RF-02 owner
**Fix-by**: 2026-04-02

### S-04: `WORKFLOW.md` does not exist — INV-001 and INV-005 enforcement references are broken

**Severity**: S1
**Surface**: Repository root
**Problem**: `INV-001` and `INV-005` both reference `WORKFLOW.md` as the enforcement mechanism. `WORKFLOW.md` does not exist in the repository. The closure contract and rollback-on-land-failure behavior are undefined.
**Remediation owner**: Fabro / execution lane
**Fix-by**: 2026-03-26

### S-05: No git land workflow — INV-005 cannot be measured

**Severity**: S2
**Surface**: `fabro/programs/`, git history
**Problem**: INV-005 (plan-land coherence) requires that `IMPLEMENT.md` truth, git land behavior, and task runtime truth do not drift. No `IMPLEMENT.md` exists, no git land enforcement exists, and no measurement mechanism exists for `count of land attempts that leave plan/git/runtime in divergent states`.
**Remediation owner**: Fabro lane
**Fix-by**: 2026-04-09

### S-06: `myosu-chain-client` is referenced but not implemented (interface contract gap)

**Severity**: S2
**Surface**: `crates/myosu-chain-client/`
**Problem**: `outputs/miner/service/spec.md` defines `crates/myosu-chain-client/` as a shared chain interaction primitive consumed by both `miner:service` and `validator:oracle`. This crate does not exist in the workspace. No `Cargo.toml` or source files for it exist.
**Remediation owner**: `miner:service` lane (Slice 1 definition includes this crate)
**Fix-by**: 2026-04-02 (as part of `miner:service` lane slice 1)

### S-07: Validator commit-reveal anti-gaming not implemented

**Severity**: S2
**Surface**: `crates/myosu-validator/src/submitter.rs`
**Problem**: R-005 mitigation (commit-reveal test positions) is the primary defense against miner gaming. `outputs/validator/oracle/spec.md` defines `commit_weights` / `reveal_weights` in the chain interface, but the pallet's `serve_axon` / `set_weights` surfaces have not been verified to support commit-reveal. The subtensor pallet transplant is not yet cleaned to verify this.
**Remediation owner**: `chain:pallet` lane (restart must confirm commit-reveal surface), `validator:oracle` lane
**Fix-by**: 2026-04-09

---

## Next Honest Recurring Implementation Slice

### Immediate (before first cadence cycle ends — 2026-03-26)

1. **S-04 fix**: Create `WORKFLOW.md` stub defining the closure contract and rollback-on-land-failure behavior. Even a minimal version enables INV-001 and INV-005 measurement.
2. **S-01 partial**: `fabro/checks/security-audit.sh` (Slice 0 of the lane spec) must be implemented so the recurring lane can actually run. This is the highest-priority bootstrap deliverable.
3. **S-03 partial**: Establish a CHANGELOG.md in the robopoker fork and pin the `Cargo.toml` dependency to a git tag.

### By second cadence cycle (2026-04-02)

4. **S-02**: `games:poker-engine` lane must be started. This unblocks INV-003 measurement and R-004 mitigation. It is the most critical unstarted lane.
5. **S-06**: `crates/myosu-chain-client/` must be created as a workspace member (part of `miner:service` Slice 1).
6. **S-01 completion**: Adjudicator/proof-gating infrastructure design documented in `security:audit` lane's own `review.md` as a finding with owner.

### By third cadence cycle (2026-04-09)

7. **S-05**: Git land workflow definition (what constitutes a "land" event, how `IMPLEMENT.md` is tracked, how divergence is measured).
8. **S-07**: Commit-reveal surface verification in `chain:pallet` restart confirmation.

---

## Attestation

This review was produced by `security:audit` lane bootstrap run on 2026-03-19.
Proof-of-run: `9e88de1` (git commit at time of run)
Source surfaces checked: `INVARIANTS.md`, `ops/risk_register.md`, `fabro/programs/myosu-recurring.yaml`, `outputs/chain/runtime/spec.md`, `outputs/chain/pallet/spec.md`, `outputs/miner/service/spec.md`, `outputs/validator/oracle/spec.md`
No external network access was used during this audit run.

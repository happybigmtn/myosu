# Security Audit, Observability, and Release Governance

Status: Completed 2026-03-29.

This ExecPlan is a living document. Maintained per `genesis/PLANS.md`.

Provenance: Enhanced from `archive/genesis_1774729423/plans/011-security-observability-release.md`. Changes: added concrete audit checklist, observability metrics, release governance process, and no-ship gate enforcement.

## Purpose / Big Picture

Before claiming stage-0 completion, the codebase needs: a security audit of all boundary surfaces, observability for chain and service health, and a release governance process that enforces no-ship conditions from `INVARIANTS.md`.

After this plan, every security risk from the assessment has a mitigation or accepted-risk note, the chain exposes health metrics, and a release checklist gates any stage-0 completion claim.

## Progress

- [x] (2026-03-28) Identified 6 security risks in assessment.
- [x] (2026-03-29) Wrote a grounded stage-0 security audit in
  `ops/security-audit-stage0.md`. The live audit now records the stripped
  chain/runtime posture, artifact/wire boundary hardening from plan `008`, the
  CI blind-spot reduction from plan `010`, and the still-accepted key
  management risk.
- [x] (2026-03-29) Completed the artifact/wire boundary audit portion inside
  `ops/security-audit-stage0.md` by tying the stage-0 claims directly to the
  already-landed bounded-decode and artifact-validation work from plan `008`.
- [x] (2026-03-29) Added chain/node observability on the honest stage-0
  boundaries. The node now logs startup timing plus RPC-surface readiness in
  `crates/myosu-chain/node/src/service.rs` and
  `crates/myosu-chain/node/src/rpc.rs`; the pallet now emits concise
  `block_step_summary` and `epoch_mechanism_summary` lines from
  `crates/myosu-chain/pallets/game-solver/src/coinbase/block_step.rs`,
  `crates/myosu-chain/pallets/game-solver/src/coinbase/run_coinbase.rs`, and
  `crates/myosu-chain/pallets/game-solver/src/epoch/run_epoch.rs`.
- [x] (2026-03-29) Added service observability to miner and validator entry
  points. Miner training now logs batch duration, epochs, exploitability, and
  iterations/sec; miner bounded query serving logs response timing and action
  summary; validator scoring logs latency plus exact-match/L1/score output; and
  validator weight submission now logs success metadata and elapsed time.
- [x] (2026-03-29) Created `ops/release-gate-stage0.md` as the live release
  governance checklist, with one explicit section per invariant and direct
  proof commands for each.
- [x] (2026-03-29) Ran the no-ship gate check and documented the result in
  `ops/release-gate-stage0.md`, while also refreshing `ops/no-ship-ledger.md`
  so it no longer claims the repo has "no code yet".

## Surprises & Discoveries

- Observation: the weakest security surface is no longer the code itself; it
  is release-governance doctrine drift.
  Evidence: the code-side boundary hardening from plans `003`, `005`, `008`,
  `009`, and `010` is real, but `INVARIANTS.md` still points at legacy
  enforcement concepts and the old no-ship ledger still described an empty
  bootstrap repo until this slice refreshed it.
- Observation: service observability had a smaller gap than the plan wording
  implied.
  Evidence: miner and validator already initialized `tracing`; the missing step
  was emitting operator-meaningful duration/result fields around training,
  bounded query serving, scoring, and weight submission.
- Observation: chain observability needed a narrower interpretation than the
  original milestone language suggested.
  Evidence: the node can measure real startup and RPC-build wall-clock timing,
  but the runtime pallet cannot honestly promise Prometheus-style wall-clock
  metrics from inside `on_initialize`. The truthful stage-0 solution is node
  timing plus pallet health summaries at `block_step` and epoch boundaries.

## Decision Log

- Decision: Security audit is checklist-based, not formal pen-test.
  Rationale: Solo operator with limited bandwidth. Checklist catches structural issues. Formal pen-test is stage-1.
  Inversion: If we delay all security work for a formal audit, stage-0 ships with known unaddressed risks.
  Date/Author: 2026-03-28 / Genesis

- Decision: Observability uses structured logging (tracing crate), not external metrics infrastructure.
  Rationale: tracing is already a workspace dependency. Prometheus/Grafana are stage-1.
  Date/Author: 2026-03-28 / Genesis

## Failure Modes

| Codepath | Realistic failure | Handling |
|---|---|---|
| Security audit | Known risk is assessed as "accepted" but later exploited | Document acceptance rationale; re-evaluate if threat model changes |
| Observability | Logging overhead impacts block production timing | Use sampling and span filtering; verify block time is unaffected |
| Release gate | No-ship condition fails but operator ships anyway | Gate is advisory in stage-0; becomes hard gate in stage-1 |

## Outcomes & Retrospective

`011` is now complete at the honest stage-0 bar. The lane closed in three
pieces: a grounded security audit under `ops/`, a release gate that names all
six invariants and still refuses a premature stage-0 completion claim, and
operator-facing observability on every live boundary that matters in stage 0.
Miner and validator actions now log duration/result fields, the node logs
startup and RPC readiness timing, and the pallet emits block-step and epoch
health summaries instead of burying all signal inside trace-level math noise.

## Context and Orientation

Security risks from assessment:

| # | Risk | Severity | Mitigation Plan |
|---|------|----------|-----------------|
| SR-01 | Large runtime attack surface (20+ pallets) | High | Plan 003 feature-gates non-essential pallets |
| SR-02 | Untrusted artifact loading | Medium | Plan 008 adds hash verification |
| SR-03 | Unsafe mmap without bounds | Medium | Plan 008 adds bounds validation |
| SR-04 | bincode decode without size caps | Medium | Plan 008 adds size caps |
| SR-05 | CI blind spot for chain | High | Plan 010 adds chain CI |
| SR-06 | No key management infrastructure | High | Deferred to stage-1 (accepted risk) |

No-ship conditions from `INVARIANTS.md`:
1. INV-001: Structured closure honesty
2. INV-002: Proof honesty (no false-green)
3. INV-003: Game verification determinism
4. INV-004: Solver-gameplay separation
5. INV-005: Plan and land coherence
6. INV-006: Robopoker fork coherence

## Milestones

### Milestone 1: Chain security audit

Audit runtime for: unused but enabled pallets (attack surface), sudo access patterns, unsigned extrinsic acceptance, and weight manipulation vectors. Document findings in `ops/security-audit-stage0.md`.

Proof command:

    test -s ops/security-audit-stage0.md

### Milestone 2: Boundary security audit

Audit artifact loading, wire encoding, mmap usage, and bincode decode for: buffer overflow, DoS via large input, path traversal in artifact discovery. Verify mitigations from plan 008 are in place.

Proof command:

    grep -rn "with_limit\|MAX_DECODE\|file_size\|bounds" crates/myosu-games-poker/src/{wire,artifacts,codexpoker,solver}.rs | wc -l

### Milestone 3: Chain and node observability

Add operator-facing node startup/RPC readiness timing and concise block-step /
epoch health summaries on the chain side.

Proof commands:

    rg -n "node_service_start|node_rpc_ready|node_service_ready" \
      crates/myosu-chain/node/src/service.rs \
      crates/myosu-chain/node/src/rpc.rs

    rg -n "block_step_summary|epoch_mechanism_summary" \
      crates/myosu-chain/pallets/game-solver/src/coinbase/block_step.rs \
      crates/myosu-chain/pallets/game-solver/src/epoch/run_epoch.rs

### Milestone 4: Service observability

Add structured logging to miner (training iteration rate, checkpoint save timing) and validator (query latency, scoring duration, submission result).

Proof command:

    grep -rn "tracing::" crates/myosu-miner/src/ crates/myosu-validator/src/ 2>/dev/null | wc -l

### Milestone 5: Release governance checklist

Create `ops/release-gate-stage0.md` with a checklist that must be fully checked before claiming stage-0 completion. Each invariant maps to a specific proof command.

Proof command:

    test -s ops/release-gate-stage0.md
    grep -c "INV-00" ops/release-gate-stage0.md

Expected: 6 (one check per invariant).

## Plan of Work

1. Run chain security audit and document findings.
2. Verify boundary mitigations from plan 008.
3. Add observability logs to chain and services.
4. Create release governance checklist.
5. Run no-ship gate check.

## Concrete Steps

From `/home/r/coding/myosu`:

    # Check current chain observability summaries
    rg -n "node_service_start|node_rpc_ready|node_service_ready|block_step_summary|epoch_mechanism_summary" \
      crates/myosu-chain/node/src/service.rs \
      crates/myosu-chain/node/src/rpc.rs \
      crates/myosu-chain/pallets/game-solver/src/coinbase/block_step.rs \
      crates/myosu-chain/pallets/game-solver/src/epoch/run_epoch.rs

    # Verify invariant enforcement
    cat INVARIANTS.md

## Validation and Acceptance

Accepted when:
- Security audit document exists with findings for all 6 risks
- Chain and service observability emit operator-facing log output
- Release governance checklist covers all 6 invariants
- No-ship gate check results are documented

## Idempotence and Recovery

Audit is documentation-only. Observability changes are additive. Release checklist is a file.

## Interfaces and Dependencies

Depends on: 007 (miner/validator must exist), 008 (boundary hardening), 009 (TUI stable), 010 (CI gates), 013 (integration tests).
Blocks: none (final plan in sequence).

```text
All prior plans complete
         |
         v
Security audit (chain + boundaries)
         |
         v
Observability (chain + services)
         |
         v
Release governance checklist
         |
         v
Stage-0 exit gate
```

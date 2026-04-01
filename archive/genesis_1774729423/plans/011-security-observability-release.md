# Security, Observability, and Release Governance

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This plan follows `genesis/PLANS.md`.

## Purpose / Big Picture

Stage-0 cannot ship on code quality alone. This plan establishes threat boundaries, invariant-driven no-ship gates, incident readiness, and release controls so operators can trust runtime behavior and rollback decisions.

## Progress

- [x] (2026-03-28 21:50Z) Consolidated top risks from assessment: artifact trust boundary, chain CI blind spots, and emission-accounting correctness.
- [ ] Update risk register with current chain/gameplay/control-plane risks and owners.
- [ ] Define and script an invariant gate command covering INV-001 through INV-006 plus emission accounting.
- [ ] Add incident templates/runbooks for stage-0 failure classes.
- [ ] Add release checklist including rollback and hotfix protocol.
- [ ] Add scorecard metrics linking CI, invariants, and service uptime signals.

## Surprises & Discoveries

- Observation: ops surfaces exist but are not fully tied to enforceable proof commands.
  Evidence: `ops/scorecard.md`, `ops/no-ship-ledger.md`, `ops/risk_register.md`, `ops/decision_log.md`.
- Observation: no-ship criteria in doctrine are strong, but automated gate stitching is incomplete.
  Evidence: `AGENTS.md` bootstrap exit criteria and current CI scope.

## Decision Log

- Decision: release readiness requires passing invariant gate + CI gate, both mandatory.
  Rationale: either alone is insufficient.
  Inversion (failure mode): shipping with only compile/tests or only governance checklists allows severe latent failures.
  Date/Author: 2026-03-28 / Genesis

- Decision: security and observability updates are tracked in repo docs with dated entries, not ad-hoc chat logs.
  Rationale: operational memory must survive personnel changes.
  Inversion (failure mode): undocumented mitigations decay and incidents repeat.
  Date/Author: 2026-03-28 / Genesis

## Failure Modes

| Codepath | Realistic failure | Handling |
|---|---|---|
| Emission accounting | Distribution drift undetected for multiple epochs | Add deterministic gate comparing sum(distributions) vs expected emission |
| Artifact trust | Poisoned local artifact influences validator/miner output | Enforce integrity policy from `008` and document operator trust roots |
| Incident response | Operator lacks runbook during validator divergence | Add incident templates + first-response command checklist |

## Outcomes & Retrospective

- Pending implementation.

## Context and Orientation

Owned files in this plan:
- `ops/risk_register.md`
- `ops/decision_log.md`
- `ops/no-ship-ledger.md`
- `ops/scorecard.md`
- `ops/instrumentation_backlog.md`
- `ops/incidents/TEMPLATE_S0_Consensus_Failure.md`
- `ops/incidents/TEMPLATE_S1_Critical_Breach.md`
- `INVARIANTS.md`
- `fabro/checks/` gate scripts
- `genesis/GENESIS-REPORT.md`

Not owned here:
- Underlying runtime/node/pallet fixes (`003`-`005`)
- UX implementation details (`009`)

## Milestones

### Milestone 1: Risk and ownership refresh

Refresh risk severity, mitigation, and owner mapping.

Proof command:

    test -s ops/risk_register.md
    rg -n "Owner|Mitigation|Status|Next Review" ops/risk_register.md

### Milestone 2: Invariant gate script

Implement one command that fails when any invariant or emission accounting check fails.

Proof command:

    test -x fabro/checks/invariant-gate.sh
    fabro/checks/invariant-gate.sh

### Milestone 3: Incident readiness

Upgrade incident templates and add first-response command runbook.

Proof command:

    rg -n "Detection|Impact|Immediate Actions|Rollback|Postmortem" ops/incidents/TEMPLATE_S0_Consensus_Failure.md ops/incidents/TEMPLATE_S1_Critical_Breach.md

### Milestone 4: Release checklist and rollback protocol

Add explicit release/no-ship checklist with rollback triggers.

Proof command:

    rg -n "Release Checklist|No-Ship|Rollback|Hotfix" ops/no-ship-ledger.md ops/decision_log.md

### Milestone 5: Scorecard integration

Tie scorecard metrics to CI and invariant gate outputs.

Proof command:

    rg -n "CI|Invariant|Uptime|Determinism|Emission" ops/scorecard.md ops/kpi_registry.yaml

## Plan of Work

1. Refresh risk and incident docs.
2. Implement enforceable invariant gate command.
3. Integrate release/no-ship and scorecard surfaces.

## Concrete Steps

From `/home/r/coding/myosu`:

    sed -n '1,260p' INVARIANTS.md
    sed -n '1,260p' ops/risk_register.md
    rg -n "INV-00|emission|determinism|no-ship" ops/*.md

## Validation and Acceptance

Accepted when:
- invariant gate command is executable and documented
- risk/incidents/release docs include owners and response steps
- scorecard metrics reference CI + invariant sources

## Idempotence and Recovery

- Governance docs can be updated incrementally.
- If gate script is noisy, keep it non-ship-blocking for one cycle with a dated remediation ticket.

## Artifacts and Notes

- Final summary lands in `genesis/GENESIS-REPORT.md`.

## Interfaces and Dependencies

Depends on: `007-miner-validator-bootstrap.md`, `009-play-tui-productization.md`, `010-ci-proof-gates-expansion.md`

```text
runtime/node/pallet + services + UX proofs
                 |
                 v
           invariant gate script
                 |
                 v
risk + incident + release docs
                 |
                 v
go/no-go decision with scorecard evidence
```

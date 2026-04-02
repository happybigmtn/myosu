# Specification: Architecture Decision Records

Source: Genesis Plan 013 (Future Architecture and Governance), SPEC.md major decisions table
Status: Draft
Depends-on: 008-release-governance

## Purpose

The project has made at least ten significant architectural decisions (Substrate
fork, single-token model, enum dispatch for CFR, robopoker fork, commit-reveal
v2, checkpoint versioning, and others) that are currently documented only in
narrative form across THEORY.MD, SPEC.md, and scattered comments. As the project
approaches stage-1 with external operators and potential contributors, these
decisions need structured records that capture the context, alternatives
considered, and consequences — so that future participants can understand why the
system is shaped the way it is without re-deriving the reasoning. An ADR process
also provides a framework for recording future irreversible decisions before
they are made.

## Whole-System Goal

Current state: Architectural decisions are documented in narrative form in
THEORY.MD (~97K lines of stage-0 proof narrative), the major decisions table in
SPEC.md, and inline comments. No structured format, no alternatives analysis,
no reversibility assessment. Future architectural questions (dual-token
economics, EVM re-enablement, multi-subnet routing, on-chain governance) have
no established decision-making framework.

This spec adds: A structured ADR template and process, retroactive ADRs for
existing decisions, and a stage-2 architectural roadmap capturing the major
decisions the project will face as it grows.

If all ACs land: Every major existing decision has a structured record with
context and consequences, future decisions follow the same template, and the
stage-2 roadmap identifies upcoming decision points with prerequisites and
reversibility assessments.

Still not solved here: On-chain governance mechanisms, formal decision-making
authority for multi-stakeholder scenarios, and community participation in
architectural decisions.

## Scope

In scope:
- ADR template and process documentation
- Retroactive ADRs for existing major decisions (at least 7)
- Stage-2 architectural roadmap identifying future decision points

Out of scope:
- Implementing any of the future architectural changes described in the roadmap
- On-chain governance mechanisms
- Community governance or voting processes
- Formal specification of protocol upgrade procedures
- ADRs for operational or process decisions (only architectural/technical)

## Current State

THEORY.MD contains extensive narrative about stage-0 decisions but at ~97K lines
is not structured for reference lookup. SPEC.md contains a "Major Decisions
Already Made" table with 10 entries, each having a one-line rationale and
reference. INVARIANTS.md codifies 6 hard rules but does not document the
alternatives considered or the consequences of choosing them.

No `docs/adr/` directory exists. No ADR template exists. No process for
proposing or recording new architectural decisions is documented.

The following decisions from SPEC.md are candidates for retroactive ADRs:
Substrate chain fork from Bittensor, single-token model, ArcSwap double-buffer
for miner, enum dispatch for CFR traits, robopoker fork, shared chain-client
seam, SwapInterface no-op stub, commit-reveal v2 only, checkpoint versioning,
and pipe mode for agent integration.

## What Already Exists

| Sub-problem | Existing code / flow | Reuse / extend / replace | Why |
|---|---|---|---|
| Decision rationale | SPEC.md major decisions table | Extract | Structured ADRs from one-line rationales |
| Decision narrative | THEORY.MD | Extract | Context and consequences from narrative |
| Hard rules | INVARIANTS.md | Reference | Some invariants encode architectural decisions |
| Decision log | `ops/decision_log.md` | Reference | Operational decision history |
| Risk register | `ops/risk_register.md` | Reference | Risks inform consequence analysis |
| Spec types | SPEC.md spec type definitions | Extend | ADRs are a form of decision spec |

## Non-goals

- Creating a formal RFC or proposal process with review periods and approvals.
- Recording every minor technical choice as an ADR (only decisions that
  constrain the system shape or are difficult to reverse).
- Implementing any changes described in the stage-2 roadmap.
- Replacing THEORY.MD or SPEC.md with ADRs — the ADRs complement existing
  documentation.

## Behaviors

An ADR template defines the structure for recording architectural decisions:
title, status (proposed/accepted/deprecated/superseded), context (what forces
led to this decision), decision (what was decided), alternatives considered
(what else was evaluated and why it was rejected), consequences (what follows
from this decision), and reversibility (how difficult it would be to change
course).

Retroactive ADRs are created for each major existing decision identified in
SPEC.md, extracting context from THEORY.MD and consequences from observed
system behavior. Each retroactive ADR captures the decision as it stands today,
not as it was originally made — including any evolution since the initial choice.

The stage-2 architectural roadmap identifies the major decisions the project
will face as it transitions beyond stage-1: dual-token economics, EVM/smart
contract re-enablement, multi-subnet emission routing, chain upgrade governance,
and open validator registration. For each decision point, the roadmap describes
the prerequisites, options, risks, and reversibility.

New architectural decisions follow the ADR template before implementation. The
ADR is written when the decision is being considered, not after the fact.

## Acceptance Criteria

- An ADR template and process document exist in the repository.
- At least 7 retroactive ADRs are recorded for existing major architectural
  decisions, each with context, alternatives, consequences, and reversibility.
- A stage-2 architectural roadmap identifies at least 5 future decision points
  with prerequisites, options, and reversibility assessments.
- The ADR numbering and location conventions are documented so that new ADRs
  can be added without ambiguity.

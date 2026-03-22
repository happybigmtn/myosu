# Write failure recovery runbook Lane — Review

Review only the current slice for `operational-setup-failure-recovery-runbook`.

Current Slice Contract:
Plan file:
- `genesis/plans/018-operational-setup.md`

Child work item: `operational-setup-failure-recovery-runbook`

Full plan context (read this for domain knowledge, design decisions, and specifications):

# Operational Setup

**Plan ID:** 018
**Status:** New
**Priority:** MEDIUM — necessary for sustained operation

`genesis/PLANS.md` at `genesis/PLANS.md` governs this document.

---

## Purpose / Big Picture

After this plan lands, Myosu will have a runnable devnet, monitoring, and an operational runbook. An engineer can follow the runbook to start the devnet, run the demo, monitor the chain, and recover from common failures.

---

## Progress

- [ ] Write devnet launch runbook
- [ ] Set up basic chain monitoring (block time, emission, neuron count)
- [ ] Write failure recovery runbook (common failure modes)
- [ ] Document environment variables needed

---

## Surprises & Discoveries

*(To be written during implementation)*

---

## Decision Log

- Decision: No Kubernetes or Terraform in Phase 3.
  Rationale: This is a 180-day turnaround, not a production deployment. Docker for local dev only.
  Date/Author: 2026-03-21 / Interim CEO

---

## Outcomes & Retrospective

*(To be written at plan completion)*

---

## Milestones

### M1: Write devnet launch runbook
Step-by-step: start chain → register participants → run demo.

Proof: `test -f ops/runbooks/devnet-launch.md`; following it produces a working devnet.

### M2: Set up basic chain monitoring
Simple scripts that report block time, emission rate, neuron count.

Proof: `ops/scripts/monitor-chain.sh` runs and reports metrics.

### M3: Write failure recovery runbook
Document common failures and their recovery steps.

Proof: `test -f ops/runbooks/failure-recovery.md` covers at least 5 common failure modes.

### M4: Document environment variables
Create `.env.example` with all required variables.

Proof: `test -f .env.example`; all variables are documented with purpose.

---

## Validation

- `test -f ops/runbooks/devnet-launch.md`
- `test -f ops/runbooks/failure-recovery.md`
- `test -f .env.example`
- `ops/scripts/monitor-chain.sh` runs and reports block time, emission, neuron count


Workflow archetype: implement

Review profile: standard

Active plan:
- `genesis/plans/001-master-plan.md`

Active spec:
- `genesis/SPEC.md`

AC contract:
- Where: Runbook documenting common failure modes and recovery steps
- How: Document at least 5 common failure modes with step-by-step recovery procedures
- Required tests: test -f ops/runbooks/failure-recovery.md
- Verification plan: File exists and covers at least 5 distinct failure modes with recovery steps
- Rollback condition: Runbook is deleted or covers fewer than 5 failure modes

Proof commands:
- `test -f ops/runbooks/failure-recovery.md`

Artifacts to write:
- `spec.md`
- `review.md`


Verification artifact must cover
- summarize the automated proof commands that ran and their outcomes

Nemesis-style security review
- Pass 1 — first-principles challenge: question trust boundaries, authority assumptions, and who can trigger the slice's dangerous actions
- Pass 2 — coupled-state review: identify paired state or protocol surfaces and check that every mutation path keeps them consistent or explains the asymmetry
- check secret handling, capability scoping, pairing/idempotence behavior, and privilege escalation paths

Focus on:
- slice scope discipline
- proof-gate coverage for the active slice
- touched-surface containment
- implementation and verification artifact quality
- remaining blockers before the next slice

Deterministic evidence:
- treat `quality.md` as machine-generated truth about placeholder debt, warning debt, manual follow-up, and artifact mismatch risk
- if `quality.md` says `quality_ready: no`, do not bless the slice as merge-ready


Write `promotion.md` in this exact machine-readable form:

merge_ready: yes|no
manual_proof_pending: yes|no
reason: <one sentence>
next_action: <one sentence>

Only set `merge_ready: yes` when:
- `quality.md` says `quality_ready: yes`
- automated proof is sufficient for this slice
- any required manual proof has actually been performed
- no unresolved warnings or stale failures undermine confidence
- the implementation and verification artifacts match the real code.

Review stage ownership:
- you may write or replace `promotion.md` in this stage
- read `quality.md` before deciding `merge_ready`
- when the slice is security-sensitive, perform a Nemesis-style pass: first-principles assumption challenge plus coupled-state consistency review
- include security findings in the review verdict when the slice touches trust boundaries, keys, funds, auth, control-plane behavior, or external process control
- prefer not to modify source code here unless a tiny correction is required to make the review judgment truthful

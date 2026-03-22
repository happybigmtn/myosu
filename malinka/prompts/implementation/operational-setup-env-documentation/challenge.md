# Document environment variables Lane — Challenge

Perform a cheap adversarial review of the current slice for `operational-setup-env-documentation` before the expensive final review runs.

Your job is to challenge assumptions, find obvious scope drift, identify weak proof, and catch mismatches between code and artifacts. Do not bless the slice as merge-ready; that belongs to the final review gate.


Verification artifact must cover
- summarize the automated proof commands that ran and their outcomes

Current Slice Contract:
Plan file:
- `genesis/plans/018-operational-setup.md`

Child work item: `operational-setup-env-documentation`

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
- Where: Root .env.example listing all required environment variables with descriptions
- How: Create .env.example with every required variable annotated with its purpose
- Required tests: test -f .env.example
- Verification plan: File exists and all variables include a comment describing their purpose
- Rollback condition: .env.example is deleted or contains undocumented variables

Proof commands:
- `test -f .env.example`

Artifacts to write:
- `spec.md`
- `review.md`

Challenge checklist:
- Is the slice smaller than the plan says, or larger?
- Did the implementation actually satisfy the first proof gate?
- Are any touched surfaces outside the named slice?
- Are the artifacts overstating completion?
- Is there an obvious bug, trust-boundary issue, or missing test the final reviewer should not have to rediscover?

Write a short challenge note in `verification.md` or amend it if needed, focusing on concrete gaps and the next fixup target. Do not write `promotion.md` here.

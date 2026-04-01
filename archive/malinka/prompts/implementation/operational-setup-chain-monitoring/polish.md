# Set up basic chain monitoring Lane — Fixup

Fix only the current slice for `operational-setup-chain-monitoring`.

Current Slice Contract:
Plan file:
- `genesis/plans/018-operational-setup.md`

Child work item: `operational-setup-chain-monitoring`

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
- Where: Monitoring script reporting block time, emission rate, and neuron count
- How: Create a script that queries the chain RPC and reports block time, emission, and neuron count
- Required tests: bash -n ops/scripts/monitor-chain.sh && test -x ops/scripts/monitor-chain.sh
- Verification plan: Script is executable, passes syntax check, and reports block time, emission rate, and neuron count against a running devnet
- Rollback condition: Script is deleted, fails syntax check, or does not report all three metrics

Proof commands:
- `test -x ops/scripts/monitor-chain.sh`
- `bash -n ops/scripts/monitor-chain.sh`

Artifacts to write:
- `spec.md`
- `review.md`


Verification artifact must cover
- summarize the automated proof commands that ran and their outcomes

Priorities:
- unblock the active slice's first proof gate
- stay within the named slice and touched surfaces
- preserve setup constraints before expanding implementation scope
- keep implementation and verification artifacts durable and specific
- do not create or rewrite `promotion.md` during Fixup; that file is owned by the Review stage
- do not hand-author `quality.md`; the Quality Gate rewrites it after verification

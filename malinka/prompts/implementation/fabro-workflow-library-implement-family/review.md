# Complete implement/ workflow family Lane — Review

Review only the current slice for `fabro-workflow-library-implement-family`.

Current Slice Contract:
Plan file:
- `genesis/plans/003-fabro-workflow-library.md`

Child work item: `fabro-workflow-library-implement-family`

Full plan context (read this for domain knowledge, design decisions, and specifications):

# Complete Fabro Workflow Library

**Plan ID:** 003
**Carries forward:** `plans/031926-design-myosu-fabro-workflow-library.md` (STRONG)
**Status:** Partial — workflow library seeded but 2 of 6 families need completion

`genesis/PLANS.md` at `genesis/PLANS.md` governs this document.

---

## Purpose / Big Picture

After this plan lands, `fabro/workflows/` will contain a complete, documented library of workflow families mapped to Myosu lane types. Each workflow will have a concrete run-config, a documented phase contract, and at least one successful execution in `outputs/`. Contributors can look at the library and immediately know which workflow applies to their lane.

---

## Progress

- [x] (2026-03-19) Seeded workflow library layout under `fabro/workflows/`
- [x] (2026-03-19) Seeded `services/` family workflows for `miner:service` and `validator:oracle`
- [x] (2026-03-19) Seeded `maintenance/` family workflows for strategy, security, operations, learning
- [ ] Complete `implement/` family — real implementation workflows beyond bootstrap
- [ ] Complete `bootstrap/` family — real bootstrap workflows beyond the initial 4
- [ ] Document all workflow families with usage guide
- [ ] Verify each family has at least one successful execution

---

## Surprises & Discoveries

- Observation: Fabro's built-in implement workflow at `coding/fabro/fabro/workflows/implement/workflow.fabro` is already close to what Myosu needs; the gap is selection and adaptation, not invention.
  Evidence: Examples cover one-shot implementation, phased builds, acceptance/spec conformance, recurring maintenance loops.

---

## Decision Log

- Decision: Workflow families are identified as: bootstrap, implement, services, maintenance, planning, and review/promote.
  Rationale: This maps cleanly to the 6 Raspberry program frontiers (bootstrap, chain-core, services, product, platform, recurring).
  Date/Author: 2026-03-19 / Codex

- Decision: Workflow families should be reusable library shapes, not Myosu-specific wrappers.
  Rationale: The Fabro core provides the primitives; Myosu should consume them directly.
  Date/Author: 2026-03-19 / Codex

---

## Outcomes & Retrospective

*(To be written at plan completion)*

---

## Milestones

### M1: Complete `implement/` workflow family
The implement family should cover: spec → implement → verify → promote. Each phase must have a documented contract (what artifacts it produces, what gates it must pass).

Proof: `fabro/workflows/implement/workflow.fabro` exists; `fabro/run-configs/implement/game-traits.toml` exists and has been executed.

### M2: Complete `bootstrap/` workflow family
The bootstrap family should cover: spec → review → (restart if needed). Each phase must handle the "restart" path gracefully (bootstrap lanes may need multiple passes).

Proof: `fabro/workflows/bootstrap/` contains at least `bootstrap.fabro`; `fabro/run-configs/bootstrap/*.toml` covers all 4 bootstrap lanes.

### M3: Document workflow library with usage guide
Create `fabro/workflows/README.md` that maps each lane type to its recommended workflow family with rationale.

Proof: `test -f fabro/workflows/README.md`; `rg -c '## ' fabro/workflows/README.md` shows at least 6 workflow sections.

### M4: Verify each family has a successful execution
For each workflow family, confirm at least one lane has executed it successfully and produced reviewed artifacts.

Proof: For each family F, `find outputs -path '*/F/*' -name 'review.md'` returns at least one file.

---

## Context and Orientation

Current workflow library state:
```
fabro/workflows/
├── bootstrap/     # SPEC → REVIEW (+ RESTART path)
├── implement/     # SPEC → IMPLEMENT → VERIFY → PROMOTE
├── services/      # Service bringup workflows
├── maintenance/   # Recurring oversight workflows
└── README.md      # (MISSING — needs to be created)
```

---

## Plan of Work

1. Audit current `fabro/workflows/` to catalog what exists
2. Identify gaps in the 6 workflow families
3. Seed missing workflow graphs using Fabro's built-in families as reference
4. Create run-configs for each new workflow
5. Execute at least one lane per new workflow family
6. Write `fabro/workflows/README.md`

---

## Concrete Steps

```bash
# Audit current workflow library
find fabro/workflows -type f | sort

# Check which families have run-configs
find fabro/run-configs -type f -name '*.toml' | sort

# Check which have produced outputs
for family in bootstrap implement services maintenance; do
  count=$(find outputs -path "*/$family/*" -name 'review.md' 2>/dev/null | wc -l)
  echo "$family: $count reviewed executions"
done
```

---

## Validation

- `test -f fabro/workflows/README.md`
- `find fabro/workflows -type f -name '*.fabro' | wc -l` ≥ 6 (one per family)
- `find fabro/run-configs -type f -name '*.toml' | wc -l` ≥ 8 (at least bootstrap 4 + implement + services + maintenance)
- For each family: `find outputs -path "*/{family}/*" -name 'review.md'` returns ≥ 1


Workflow archetype: implement

Review profile: foundation

Active plan:
- `genesis/plans/001-master-plan.md`

Active spec:
- `genesis/SPEC.md`

AC contract:
- Where: Fabro implement workflow family and associated run-configs
- How: Add spec-implement-verify-promote workflow with documented phase contracts
- Required tests: test -f fabro/workflows/implement/workflow.fabro && test -f fabro/run-configs/implement/game-traits.toml
- Verification plan: workflow.fabro exists with phase contracts; at least one run-config references it
- Rollback condition: workflow file deleted or phase contracts found incomplete

Proof commands:
- `test -f fabro/workflows/implement/workflow.fabro`
- `test -f fabro/run-configs/implement/game-traits.toml`

Artifacts to write:
- `spec.md`
- `review.md`


Verification artifact must cover
- summarize the automated proof commands that ran and their outcomes

Nemesis-style security review
- Pass 1 — first-principles challenge: question trust boundaries, authority assumptions, and who can trigger the slice's dangerous actions
- Pass 2 — coupled-state review: identify paired state or protocol surfaces and check that every mutation path keeps them consistent or explains the asymmetry
- check state transitions that affect balances, commitments, randomness, payout safety, or replayability
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

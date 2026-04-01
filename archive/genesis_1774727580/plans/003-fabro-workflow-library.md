# Execution Playbook Library

**Plan ID:** 003
**Carries forward:** the earlier workflow-library design intent, translated to direct execution
**Status:** Complete — repo-native playbooks now exist for the main execution families

`genesis/PLANS.md` at `genesis/PLANS.md` governs this document.

---

## Purpose / Big Picture

After this plan lands, Myosu will have a complete, documented library of
direct-execution playbooks mapped to the main lane types. Each playbook will
name its inputs, concrete commands, proof expectations, and output artifacts.
Contributors can look at the library and immediately know how to execute a lane
without relying on historical orchestration tooling.

---

## Progress

- [x] Define the playbook families and their scope
- [x] Create a bootstrap playbook
- [x] Create an implementation playbook
- [x] Create a services playbook
- [x] Create maintenance and review playbooks
- [x] Document selection guidance and proof expectations
- [x] Verify each playbook against at least one real Myosu lane

---

## Surprises & Discoveries

- Observation: The useful part of the old workflow work is the phase structure,
  not the historical graph format.
  Evidence: The same lane families still exist even after removing the
  orchestration dependency.

---

## Decision Log

- Decision: The playbook families are bootstrap, implement, services,
  maintenance, planning, and review.
  Rationale: These match the major execution shapes in the repo without tying
  them to a specific runner.
  Date/Author: 2026-03-19 / Codex

- Decision: Playbooks should be plain repo documentation plus executable
  commands.
  Rationale: The execution model should survive even if external tooling
  changes.
  Date/Author: 2026-03-19 / Codex

---

## Outcomes & Retrospective

*(To be written at plan completion)*

---

## Milestones

### M1: Define the playbook families
Document the purpose of the bootstrap, implement, services, maintenance,
planning, and review families.

Proof: the playbook index contains all six families with a short contract for
each.

### M2: Create bootstrap and implementation playbooks
The bootstrap playbook should cover inspect → verify → rewrite artifacts. The
implementation playbook should cover orient → implement → verify → update docs.

Proof: both playbooks exist and name concrete commands plus expected outputs.

### M3: Create services, maintenance, planning, and review playbooks
Document the recurring execution shapes used for daemons, audits, planning
passes, and final review.

Proof: each family has a dedicated section or file with commands and proof
expectations.

### M4: Verify each playbook against a real lane
For each family, point to at least one real Myosu lane or artifact that uses
it.

Proof: the playbook library includes one concrete Myosu example per family.

---

## Context and Orientation

Target library shape:

```text
docs/execution-playbooks/
├── README.md
├── bootstrap.md
├── implementation.md
├── services.md
├── maintenance.md
├── planning.md
└── review.md
```

---

## Plan of Work

1. Define the six playbook families
2. Pick a documentation home in the repo
3. Write each playbook in terms of direct commands and outputs
4. Map real lanes to each playbook family
5. Add selection guidance and proof expectations

---

## Concrete Steps

```bash
# Draft the playbook library
mkdir -p docs/execution-playbooks

# Verify coverage of families
rg -n '^## ' docs/execution-playbooks/README.md
rg -n 'Example lane|Proof' docs/execution-playbooks/*.md
```

---

## Validation

- `test -f docs/execution-playbooks/README.md`
- The library covers all 6 families
- Each family names commands, proof expectations, and one Myosu example

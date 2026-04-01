# Frontier Dependency Decomposition

**Plan ID:** 004
**Carries forward:** the six-frontier decomposition from the March 2026 planning slice
**Status:** Complete — the frontier map is now documented in repo-native form

`genesis/PLANS.md` at `genesis/PLANS.md` governs this document.

---

## Purpose / Big Picture

After this plan lands, Myosu's six main frontiers will have a documented,
repo-native dependency shape. Each frontier will have clear ownership,
dependency edges, and milestone contracts. Later implementation work will use
that map directly instead of relying on external manifests.

---

## Progress

- [x] Document the six frontiers and their ownership boundaries
- [x] Make dependency edges explicit
- [x] Verify the master plan and frontier map agree
- [x] Add an ASCII dependency map
- [x] Check the map against real implementation sequencing

---

## Surprises & Discoveries

- Observation: The useful part of the old program decomposition is the frontier
  split itself, not the manifest format.
  Evidence: the chain, services, product, platform, and recurring concerns are
  still the right top-level buckets.

---

## Decision Log

- Decision: The stable frontier set is bootstrap, chain-core, services,
  product, platform, and recurring.
  Rationale: This is still the clearest ownership split for the repo.
  Date/Author: 2026-03-19 / Codex

- Decision: The frontier map should live in repo docs, not in a tool-specific
  manifest.
  Rationale: Ownership and dependency information should remain readable even
  without external tooling.
  Date/Author: 2026-03-19 / Codex

---

## Outcomes & Retrospective

*(To be written at plan completion)*

---

## Milestones

### M1: Document the six frontiers and explicit dependency edges
Make the dependency ordering between bootstrap, chain-core, services, product,
platform, and recurring explicit. No implicit blocking.

Proof: the frontier map names each dependency edge and the artifact or
milestone that satisfies it.

### M2: Verify the frontier map agrees with the master plan
The frontier dependency map and `001-master-plan.md` must describe the same
ordering.

Proof: the frontier map and master plan list the same top-level dependencies.

### M3: Check the map against real execution order
Walk the first active plans in sequence and confirm the dependency map predicts
that order correctly.

Proof: Phase 0 and Phase 1 sequencing in the master plan is consistent with the
frontier dependency map.

### M4: Produce dependency map diagram
Create an ASCII diagram of the six-frontier dependency graph and include it in a
repo-native document.

Proof: `test -f genesis/EXECUTION_MAP.md`; `rg -A 20 'Dependency Map' genesis/EXECUTION_MAP.md`
shows the diagram.

---

## Context and Orientation

The six Myosu frontiers:

```text
bootstrap   # reviewed artifacts and early truth alignment
chain-core  # runtime and pallet restoration
services    # miner and validator binaries
product     # gameplay and agent experience
platform    # poker engine, SDK, multi-game architecture
recurring   # operations, security, and learning loops
```

---

## Plan of Work

1. Document each frontier and its ownership
2. Write explicit dependency edges
3. Cross-check the map against the master plan
4. Add the dependency diagram
5. Use the map to sanity-check sequencing before implementation

---

## Concrete Steps

```bash
# Check active plan references to frontiers
rg -n 'bootstrap|chain-core|services|product|platform|recurring' genesis/plans/*.md

# Verify the dependency map doc exists and contains the diagram
test -f genesis/EXECUTION_MAP.md
rg -A 20 'Dependency Map' genesis/EXECUTION_MAP.md
```

---

## Validation

- The six frontiers are documented with explicit dependencies
- The dependency map and master plan agree
- `test -f genesis/EXECUTION_MAP.md` with dependency diagram

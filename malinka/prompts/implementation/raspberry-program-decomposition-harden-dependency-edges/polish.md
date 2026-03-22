# Harden dependency edges with explicit ordering Lane — Fixup

Fix only the current slice for `raspberry-program-decomposition-harden-dependency-edges`.

Current Slice Contract:
Plan file:
- `genesis/plans/004-raspberry-program-decomposition.md`

Child work item: `raspberry-program-decomposition-harden-dependency-edges`

Full plan context (read this for domain knowledge, design decisions, and specifications):

# Complete Raspberry Program Decomposition

**Plan ID:** 004
**Carries forward:** `plans/031926-decompose-myosu-into-raspberry-programs.md` (VERY STRONG)
**Status:** Partial — all 6 programs seeded; dependency ordering needs hardening

`genesis/PLANS.md` at `genesis/PLANS.md` governs this document.

---

## Purpose / Big Picture

After this plan lands, Myosu's Raspberry control plane will have a fully functional program-of-programs shape. Each of the 6 frontier programs (bootstrap, chain-core, services, product, platform, recurring) will have clear lane ownership, dependency edges, and milestone contracts. Fabro can dispatch any program and Raspberry can report truthful state.

---

## Progress

- [x] (2026-03-19) Seeded `myosu-bootstrap.yaml` (all 4 lanes complete as of 2026-03-21)
- [x] (2026-03-19) Seeded `myosu-chain-core.yaml`
- [x] (2026-03-19) Seeded `myosu-services.yaml`
- [x] (2026-03-19) Seeded `myosu-product.yaml`
- [x] (2026-03-19) Seeded `myosu-platform.yaml`
- [x] (2026-03-19) Seeded `myosu-recurring.yaml`
- [ ] Harden dependency edges — explicit ordering of which program must complete before which
- [ ] Verify `myosu.yaml` (parent portfolio) correctly dispatches all 6 child programs
- [ ] Stress-test autodev loop with all 6 programs active

---

## Surprises & Discoveries

- Observation: the autodev loop expanded the portfolio from 7 to 11 child programs autonomously, adding `play-tui-implementation`, `games-poker-engine-implementation`, `games-multi-game-implementation`, and `sdk-core-implementation`.
  Evidence: `myosu.yaml` now has 11 child programs.

- Observation: generated implementation programs produced compile-passing but non-functional code.
  Evidence: `myosu-play` binary exits immediately without entering terminal loop.

---

## Decision Log

- Decision: 6 frontier programs are the stable set: bootstrap, chain-core, services, product, platform, recurring.
  Rationale: This maps cleanly to the 6 spec ownership categories in `specsarchive/`.
  Date/Author: 2026-03-19 / Codex

- Decision: Implementation-family programs are children of platform/product, not top-level.
  Rationale: Implementation programs are ephemeral — they come and go as lanes progress. The stable frontier programs own the lane contracts.
  Date/Author: 2026-03-19 / Codex

---

## Outcomes & Retrospective

*(To be written at plan completion)*

---

## Milestones

### M1: Harden dependency edges with explicit ordering
Each program manifest should explicitly list which reviewed artifacts from other programs it depends on. No implicit blocking.

Proof: For each program, `rg 'depends_on|artifact:|precondition:' fabro/programs/myosu-*.yaml` returns explicit dependency declarations.

### M2: Verify `myosu.yaml` (parent) correctly dispatches all 6 child programs
Run `raspberry status --manifest fabro/programs/myosu.yaml` and confirm all 6 children appear.

Proof: Output shows `myosu-bootstrap`, `myosu-chain-core`, `myosu-services`, `myosu-product`, `myosu-platform`, `myosu-recurring` each with a reported status.

### M3: Stress-test autodev loop with all 6 programs
Trigger `raspberry autodev --manifest fabro/programs/myosu.yaml` with all 6 programs ready. Observe that:
- at most `max_parallel` programs run simultaneously
- completed programs are correctly marked
- blocked programs correctly report blocking dependencies

Proof: After autodev run, Raspberry reports truthful status for all 6 programs matching actual Fabro run state.

### M4: Produce dependency map diagram
Create an ASCII diagram of the 6-program dependency graph and include it in `fabro/programs/README.md`.

Proof: `test -f fabro/programs/README.md`; `rg -A 20 'Dependency Map' fabro/programs/README.md` shows the diagram.

---

## Context and Orientation

The 6 Myosu frontier programs:
```
fabro/programs/myosu-bootstrap.yaml    # bootstrap — COMPLETE
fabro/programs/myosu-chain-core.yaml   # chain restart — BLOCKED on bootstrap
fabro/programs/myosu-services.yaml      # miner + validator services — BLOCKED on chain-core
fabro/programs/myosu-product.yaml       # gameplay + agent experience — BLOCKED on bootstrap
fabro/programs/myosu-platform.yaml      # engine + SDK — BLOCKED on bootstrap
fabro/programs/myysu-recurring.yaml     # recurring oversight — BLOCKED on chain-core
```

---

## Plan of Work

1. Audit all 6 program manifests for dependency completeness
2. Add explicit `depends_on` declarations referencing `outputs/**/review.md` artifacts
3. Verify parent portfolio (`myosu.yaml`) includes all 6 children
4. Run autodev loop and verify correct dispatch behavior
5. Document the program map

---

## Concrete Steps

```bash
# Audit dependency declarations in each program
for prog in myosu-bootstrap myosu-chain-core myosu-services myosu-product myosu-platform myosu-recurring; do
  echo "=== $prog ==="
  rg -n 'depends_on|artifact:|precondition:|ready_when:' "fabro/programs/${prog}.yaml" || echo "(none found)"
done

# Verify parent portfolio
raspberry status --manifest fabro/programs/myosu.yaml

# Check for implicit vs explicit blocking
# A program is implicitly blocked if it references an artifact that doesn't exist yet
for artifact_ref in $(rg -oh 'outputs/[^"'\'']+\.(md|json)' fabro/programs/myosu-*.yaml | sort -u); do
  if [ ! -f "$artifact_ref" ]; then
    echo "MISSING: $artifact_ref"
  fi
done
```

---

## Validation

- `raspberry status --manifest fabro/programs/myosu.yaml` shows all 6 child programs
- No program manifest references a missing `outputs/` artifact without an explicit `blocked_until:` declaration
- Autodev loop correctly respects `max_parallel` constraint
- `test -f fabro/programs/README.md` with dependency diagram


Workflow archetype: implement

Review profile: foundation

Active plan:
- `genesis/plans/001-master-plan.md`

Active spec:
- `genesis/SPEC.md`

AC contract:
- Where: All 6 frontier program manifests under fabro/programs/
- How: Add explicit depends_on declarations referencing outputs/**/review.md artifacts to each program manifest
- Required tests: for prog in myosu-bootstrap myosu-chain-core myosu-services myosu-product myosu-platform myosu-recurring; do rg 'depends_on|artifact:|precondition:' fabro/programs/${prog}.yaml || exit 1; done
- Verification plan: Every program manifest has at least one explicit dependency declaration; no implicit blocking via missing artifact references
- Rollback condition: Any program manifest found without explicit dependency declarations or referencing a missing artifact without a blocked_until declaration

Proof commands:
- `rg 'depends_on|artifact:|precondition:' fabro/programs/myosu-bootstrap.yaml`
- `rg 'depends_on|artifact:|precondition:' fabro/programs/myosu-chain-core.yaml`
- `rg 'depends_on|artifact:|precondition:' fabro/programs/myosu-services.yaml`
- `rg 'depends_on|artifact:|precondition:' fabro/programs/myosu-product.yaml`
- `rg 'depends_on|artifact:|precondition:' fabro/programs/myosu-platform.yaml`
- `rg 'depends_on|artifact:|precondition:' fabro/programs/myosu-recurring.yaml`

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

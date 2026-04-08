# 001 — Master Plan

## Objective

Sequence the remaining stage-0 work into phases that reduce risk monotonically, deliver verifiable checkpoints, and avoid blocking on external dependencies.

## Phase Structure

### Phase 1: Reduce and Clean (Plans 002–005)

Remove dead code, deduplicate pallets, clean naming, and audit inherited complexity. This phase has zero external dependencies and the highest leverage per line changed. Every later phase benefits from a smaller, clearer codebase.

### Decision Gate 1 (Plan 006)

Verify trunk compiles, all CI jobs pass, and the reduced codebase matches all existing proof contracts. No new work proceeds until this gate is green.

### Phase 2: Harden and Measure (Plans 007–009)

Harden emission accounting, fill test gaps, and establish the quality benchmark surface needed for the miner convergence research gate.

### Decision Gate 2 (Plan 010)

Verify emission invariants, test gaps closed, and quality benchmark surface exists. Confirm stage-0 exit criteria readiness.

### Phase 3: Package and Document (Plans 011–013)

Container packaging, README overhaul, and documentation cleanup to make the operator experience viable.

### Phase 4: Research Gates (Plans 014–015, independent)

Token economics decision and polkadot-sdk migration feasibility. These run in parallel with any phase and do not block stage-0 exit.

## Sequencing Rationale

The previous planning snapshot used the same Phase 1 → Phase 2 → Phase 3 ordering and explicitly rejected a network-first alternative. That reasoning remains valid: debugging network or packaging issues against a codebase with 90K lines of dead pallet code is strictly harder than debugging against a clean codebase. The code-reduction-first ordering has the additional benefit that every subsequent diff is smaller, every code search is faster, and every new contributor's onboarding is simpler.

Phase 4 research gates are decoupled because they depend on external human review (F-003) and upstream analysis (CHAIN-SDK-001), neither of which should block mechanical cleanup work.

## Plan Index

| # | Title | Phase | Depends On | Estimated Scope |
|---|-------|-------|------------|-----------------|
| 002 | Dead Pallet Removal | 1 | none | L |
| 003 | Pallet Naming Normalization | 1 | 002 | M |
| 004 | Inherited Migration Cleanup | 1 | 002 | M |
| 005 | Stale Document Cleanup | 1 | none | S |
| 006 | Phase 1 Decision Gate | gate | 002–005 | S |
| 007 | Emission Dust Policy | 2 | 006 | S |
| 008 | Test Gap Closure | 2 | 006 | M |
| 009 | Miner Quality Benchmark | 2 | 008 | M |
| 010 | Phase 2 Decision Gate | gate | 007–009 | S |
| 011 | Container Packaging | 3 | 010 | L |
| 012 | README and Onboarding Overhaul | 3 | 006 | M |
| 013 | Fabro Ghost Cleanup | 3 | 006 | S |
| 014 | Token Economics Research Gate | 4 | none | L (external) |
| 015 | SDK Migration Research Gate | 4 | none | L (external) |

## Acceptance Criteria

- All 15 plans are written with concrete acceptance criteria and verification steps
- Sequencing dependencies are explicit and acyclic
- Each phase boundary has a decision gate plan
- No plan depends on an unresolved external input without declaring it as a blocker

## Verification

- Read plans 002–015 and confirm each has: title, description, acceptance criteria, verification, dependencies
- Confirm dependency graph is acyclic
- Confirm gate plans (006, 010) list all prerequisites

## Dependencies

- None (this is the root plan)

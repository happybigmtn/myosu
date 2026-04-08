# Plan Index

Generated: 2026-04-07
Source of truth: trunk @ 4e0b37f

---

## Overview

15 plans across 4 phases plus 2 decision gates. The sequence prioritizes codebase reduction first, hardening second, packaging third, and research gates independently.

## Sequencing Rationale

**Why reduce-first, not network-first or packaging-first?**

The previous planning snapshot (April 7, 2026) reached the same conclusion and explicitly rejected alternatives. The reasoning is:

1. **Code reduction (Phase 1) has zero external dependencies.** Deleting the 90K-line dead pallet, normalizing naming, and cleaning migrations are purely mechanical. Every later phase benefits from a smaller, clearer codebase.

2. **Hardening (Phase 2) requires a clean baseline.** Adding emission dust policy or quality benchmarks against a codebase with duplicate pallets and stale naming creates confusion about which code path is being measured.

3. **Packaging (Phase 3) should package the final form.** Docker images and README overhauls should reflect the post-cleanup codebase, not the current one.

4. **Research gates (Phase 4) are externally blocked.** Token economics needs human review. SDK migration needs patch classification. Neither blocks mechanical work.

**Alternative considered: Network-first (build multi-node devnet before cleanup).** Rejected because debugging network issues against 182K lines of pallet code (two copies) is strictly harder than debugging against 91K lines (one copy). The multi-node devnet already works (4-authority finality, cross-node emission proven) — the remaining work is packaging, not proving.

**Alternative considered: Packaging-first (Docker images now).** Rejected because the Docker images would need to be rebuilt after Phase 1 renames the pallet and Phase 2 changes emission behavior. Packaging too early creates maintenance burden.

## Plan Table

| # | Title | Phase | Depends On | Type | Est. |
|---|-------|-------|------------|------|------|
| 001 | Master Plan | — | — | index | — |
| 002 | Dead Pallet Removal | 1 | none | implementation | L |
| 003 | Pallet Naming Normalization | 1 | 002 | implementation | M |
| 004 | Inherited Migration Cleanup | 1 | 002 | implementation | M |
| 005 | Stale Document Cleanup | 1 | none | implementation | S |
| 006 | Phase 1 Decision Gate | gate | 002–005 | gate | S |
| 007 | Emission Dust Policy | 2 | 006 | design+impl | S |
| 008 | Test Gap Closure | 2 | 006 | implementation | M |
| 009 | Miner Quality Benchmark | 2 | 008 | design+impl | M |
| 010 | Phase 2 Decision Gate | gate | 007–009 | gate | S |
| 011 | Container Packaging | 3 | 010 | implementation | L |
| 012 | README and Onboarding Overhaul | 3 | 006 | implementation | M |
| 013 | Fabro Ghost Cleanup | 3 | 006 | decision+impl | S |
| 014 | Token Economics Research Gate | 4 | none | research | L (ext) |
| 015 | SDK Migration Research Gate | 4 | none | research | L (ext) |

## Dependency Graph

```
002 (dead pallet) ──┬──► 003 (naming) ──┐
                    ├──► 004 (migrations)├──► 006 (gate 1) ──┬──► 007 (dust) ──┐
005 (stale docs) ───┘                   │                   ├──► 008 (tests) ──┤
                                        │                   ├──► 012 (readme)  │
                                        │                   └──► 013 (fabro)   │
                                        │                                      │
                                        │      009 (quality) ◄─── 008 (tests) │
                                        │           │                          │
                                        │           ▼                          │
                                        └──► 010 (gate 2) ──► 011 (containers)│
                                                                               │
014 (token econ) ──── independent ─────────────────────────────────────────────┘
015 (sdk migration) ── independent
```

## Parallel Execution Opportunities

For multi-worker teams:

- **Worker 1:** 002 → 003 → 004 → (wait for 006) → 007 → 009
- **Worker 2:** 005 → (wait for 006) → 008 → (wait for 010) → 011
- **Worker 3:** 014 or 015 (research, any time)
- **Worker 4:** 012 → 013 (after 006)

Plans 002 and 005 can start immediately in parallel. Plans 014 and 015 can start at any time.

## Phase Summaries

### Phase 1: Reduce and Clean

**Goal:** Remove 90K+ lines of dead code, normalize naming, clean migrations, update stale docs.
**Exit:** Plan 006 gate green.
**Risk:** Low. All changes are deletion or renaming. No new behavior.

### Phase 2: Harden and Measure

**Goal:** Decide emission dust policy, close test gaps, create quality benchmark.
**Exit:** Plan 010 gate green.
**Risk:** Medium. Dust policy and quality benchmark involve design decisions.

### Phase 3: Package and Document

**Goal:** Docker images, README overhaul, fabro/ghost cleanup.
**Exit:** Operators can run the system without compiling from source.
**Risk:** Medium. Docker WASM build may require research (noted in Plan 011).

### Phase 4: Research Gates (Independent)

**Goal:** Token economics and SDK migration decisions.
**Exit:** ADRs updated, decisions recorded.
**Risk:** High (external). Both depend on human review or upstream analysis that cannot be automated.

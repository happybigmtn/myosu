# Myosu Plan Index

Date: 2026-04-05

---

## Plan Set

| # | Title | Phase | Status | Dependencies |
|---|-------|-------|--------|--------------|
| 001 | [Master Plan](plans/001-master-plan.md) | -- | Active | -- |
| 002 | [Dead Code Removal](plans/002-dead-code-removal.md) | 1: Reduce | Ready | None |
| 003 | [Emission Hardening](plans/003-emission-hardening.md) | 1: Reduce | Ready | 002 |
| 004 | [Test Deduplication](plans/004-test-deduplication.md) | 1: Reduce | Blocked | 002 |
| 005 | [Pallet Storage Audit](plans/005-pallet-storage-audit.md) | 1: Reduce | Blocked | 002, 003, 004 |
| 006 | [Multi-Node Devnet](plans/006-multi-node-devnet.md) | 2: Network | Ready | None (prefer 002-005 first) |
| 007 | [Consensus Finality Proof](plans/007-consensus-finality-proof.md) | 2: Network | Blocked | 006 |
| 008 | [Cross-Node Emission](plans/008-cross-node-emission.md) | 2: Network | Blocked | 003, 006 |
| 009 | [Phase 2 Decision Gate](plans/009-phase-2-gate.md) | Gate | Blocked | 002-008 |
| 010 | [Container Packaging](plans/010-container-packaging.md) | 3: Package | Blocked | 009 |
| 011 | [Operator Documentation](plans/011-operator-documentation.md) | 3: Package | Partially ready | 010 (multi-node part) |
| 012 | [Release Process](plans/012-release-process.md) | 3: Package | Blocked | 010, 011 |
| 013 | [Token Economics Research](plans/013-token-economics-research.md) | 4: Research | Ready | None |
| 014 | [SDK Migration Research](plans/014-sdk-migration-research.md) | 4: Research | Ready | Prefer 002 first |

---

## Sequencing Rationale

### Why Phase 1 before Phase 2

The chain carries ~150K lines of duplicated pallet source (game-solver +
subtensor), ~44 duplicated test files, and ~114 dormant storage items.
Attempting multi-node proofs against this surface means any failure requires
auditing twice the code. Reducing first makes network debugging tractable.

### Why Phase 2 before Phase 3

Packaging a system that has not been proven at network scale creates false
confidence for operators. The multi-node proofs (Plans 006-008) validate
that the system works beyond the single-node local loop before we invite
operators to run it.

### Why the Decision Gate (009)

Plans 006-008 test hypotheses about cross-node determinism and finality
that have not been validated. If they fail, the fix may require returning to
Phase 1 (e.g., fixing emission determinism) rather than proceeding to Phase 3.
The gate makes this branching explicit rather than burying it in a Phase 3 plan.

### Alternative: Network-First Sequence

An alternative sequence would be:
- 006 → 007 → 008 → 002 → 003 → 004 → 005 → 009

This prioritizes learning about network behavior sooner. It was rejected because
debugging network issues against a codebase with duplicated pallets and untested
emission accounting is strictly harder than debugging against a clean codebase.
The cost of Phase 1 is bounded (mostly deletion and gating), while the cost of
debugging network issues against inherited complexity is unbounded.

### Parallel Work

Plans 013 and 014 (research gates) can proceed in parallel with all other work.
They produce documents, not code changes, so they have no conflict with
implementation plans.

Plan 011 (operator documentation) can be partially started before Plan 010
(the local devnet quickstart path is independent of container packaging).

---

## Starting Points

A single focused worker should start with Plan 002 (dead code removal).
It is the highest-leverage independent action: it removes ~150K lines of
duplicated source and unblocks Plans 003, 004, and 005.

If two workers are available:
- Worker 1: Plan 002 → 003 → 004 → 005
- Worker 2: Plan 013 (token economics research) or Plan 006 (multi-node devnet)

If three workers are available, add Plan 014 (SDK migration research) or
begin Plan 011 (quickstart documentation for local devnet path).

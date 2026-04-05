# 001: Master Plan -- Stage-0 Exit and Devnet Readiness

## Objective

Close the remaining stage-0 gaps and prepare the system for multi-node devnet
operation. This plan sequences all work from current state (local loop proven,
multi-game proven, operator tooling partially hardened) to stage-0 exit and
initial devnet readiness.

## Phase Structure

### Phase 1: Reduce and Harden (Plans 002-005)

Clean up inherited chain complexity, harden emission accounting, and remove
the dual-pallet copy. This is prerequisite work that reduces the surface
area and increases confidence in the core chain behavior.

### Phase 2: Network Proof (Plans 006-008)

Extend from single-node to multi-node devnet. Prove consensus finality,
peer discovery, and cross-node emission agreement. This is the bridge
from "local proof" to "operator-ready network."

### Decision Gate: Plan 009

After Phase 2, evaluate whether the emission model and network behavior
justify proceeding to operator packaging. If multi-node emission accounting
does not converge, return to Phase 1 hardening.

### Phase 3: Operator Packaging (Plans 010-012)

Container images, stable chain spec distribution, first-run documentation,
and release process. This makes the system operator-ready rather than
developer-only.

### Phase 4: Research Gates (Plans 013-014)

Open questions that affect post-stage-0 direction but do not block stage-0
exit. Token economics, upstream SDK migration, and encoder optimization.

## Dependency Graph

```
002 (dead code removal) ─────┐
003 (emission hardening) ────┤
004 (test duplication) ──────┼─► 005 (pallet storage audit) ─► 009 (gate)
                             │
006 (multi-node devnet) ─────┤
007 (consensus proof) ───────┤
008 (cross-node emission) ───┘
                             
009 (decision gate) ─► 010 (container packaging)
                    ─► 011 (operator documentation)
                    ─► 012 (release process)

013 (token economics research) ─── independent
014 (SDK migration research) ──── independent
```

## Why This Sequence

The obvious alternative is to skip Phase 1 and go directly to multi-node
devnet (Phase 2). This was rejected because:

1. The dual-pallet copy (game-solver + subtensor) means any chain change
   must be audited against two 200K+ line codebases. Removing the dead copy
   first dramatically reduces the Phase 2 risk surface.
2. Emission accounting under identity swap is untested at network scale.
   Hardening it locally (Plan 003) before testing it across nodes (Plan 008)
   is cheaper than debugging emission divergence in a distributed system.
3. The test duplication (44 files × 2 pallets) makes CI slower and creates
   confusion about which test suite is authoritative.

Another alternative would be to start with operator packaging (Phase 3)
to get external operators testing sooner. This was rejected because packaging
a system with known inherited complexity and untested network behavior would
create false confidence. Operators should receive a system that works at
network scale, not just locally.

## Not Doing

- **Game portfolio expansion**: Three games are sufficient for stage-0.
- **Web/mobile gameplay**: TUI and pipe serve stage-0 users.
- **Full AMM token economics**: The identity swap is the correct stage-0 model.
- **Upstream polkadot-sdk migration**: Research gate only (Plan 014).
- **Production deployment**: Stage-0 is devnet-only.
- **Governance/upgrade mechanisms**: Post-stage-0 concern.
- **Benchmarking and weight calibration**: Not needed until production.

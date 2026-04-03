# ADR 007: Versioned Binary Checkpoints For Solver State

- Status: Accepted
- Date: 2026-04-02
- Deciders: Myosu maintainers
- Consulted: `AGENTS.md`, `THEORY.MD`, `specs/031626-02b-poker-engine.md`, `specs/031626-06-multi-game-architecture.md`
- Informed: poker, second-game, miner, validator, and gameplay contributors
- Related: `crates/myosu-games-poker/src/solver.rs`, `crates/myosu-games-liars-dice/src/solver.rs`, `docs/operator-guide/quickstart.md`, `docs/operator-guide/upgrading.md`

## Context

This is a retroactive record of the checkpoint framing decision that now sits at
the center of Myosu's local artifact workflow.

Miners, validators, and gameplay all need to load saved solver state from disk.
Plain raw bincode or upstream-internal persistence formats are too fragile for
that boundary: they can drift silently, accept truncated payloads too late, or
leave operators with unreadable failures when the format changes. Myosu needed a
small, explicit on-disk contract before checkpoint-backed workflows could be
treated as real stage-0 product surfaces.

The repo now uses the same framing idea in poker and liar's dice: a fixed magic
header, an explicit version word, bounded decode, and tests for malformed input.

## Decision

Myosu solver checkpoints are versioned binary artifacts with an explicit header.

The current contract is:

- four magic bytes (`MYOS`)
- a little-endian checkpoint format version
- the encoded solver/profile payload behind that header
- strict load-time validation before the solver state is accepted

Checkpoint framing is part of the stable local artifact boundary shared by
miner, validator, gameplay, and operator documentation.

## Alternatives Considered

### Option A: Headered binary checkpoints with magic and version

This won because it gives Myosu a cheap but explicit artifact contract that can
fail clearly instead of silently.

### Option B: Raw bincode with no framing

This was rejected because format drift or truncated files would be harder to
diagnose and easier to mis-handle.

### Option C: Database-only or service-internal persistence

This was rejected because stage-0 operators and proofs rely on file-based local
artifacts rather than external database infrastructure.

## Consequences

### Positive

- Miner, validator, and gameplay processes can exchange solver state through a
  shared, inspectable artifact contract.
- Format changes have an obvious migration boundary.
- Malformed-input tests can target a clear header and size contract.

### Negative

- Any future format bump requires version-management discipline and migration
  compatibility work.
- The repo now owns a long-lived local artifact format instead of treating
  checkpoints as an incidental internal detail.

### Follow-up

- Keep operator docs aligned with the actual checkpoint contract.
- Bump checkpoint versions deliberately and only with a matching migration or
  compatibility plan.

## Reversibility

Moderate.

The repo can change checkpoint versions or even replace the format, but only by
updating all artifact producers and consumers together and providing a safe
migration or clear invalidation story. The decision should be reopened if a new
artifact format materially improves portability or compatibility without
weakening corruption detection.

## Validation / Evidence

- `crates/myosu-games-poker/src/solver.rs` defines `CHECKPOINT_MAGIC`,
  `CHECKPOINT_VERSION`, and load/save validation errors.
- `crates/myosu-games-liars-dice/src/solver.rs` applies the same framing model
  to the second-game solver.
- Operator docs in `docs/operator-guide/quickstart.md` and
  `docs/operator-guide/upgrading.md` treat checkpoints as durable local
  artifacts that survive across service boundaries.

# Harden Artifact Loading, Wire Encoding, and Checkpoint Persistence

Status: Completed 2026-03-29.

This ExecPlan is a living document. Maintained per `genesis/PLANS.md`.

Provenance: Enhanced from `archive/genesis_1774729423/plans/008-artifact-wire-checkpoint-hardening.md`. Changes: added concrete size caps, signing scheme design, version format, and mmap safety audit.

## Purpose / Big Picture

The artifact pipeline (`myosu-games-poker/src/artifacts.rs`, `codexpoker.rs`, `wire.rs`, `solver.rs`) loads trained strategy data from disk, decodes binary formats, and serves responses over the wire. The original downstream risks were real, but most of them had already been partially addressed by the time execution resumed. The remaining job was to close the honest trust-boundary gaps rather than blindly implement the stale wording.

After this plan, all binary decode boundaries have size caps, artifact loading validates integrity hashes, mmap operations validate file size, and checkpoints carry a version header.

## Progress

- [x] (2026-03-28) Audited artifact, wire, solver, and codexpoker modules for security boundaries.
- [x] (2026-03-29) Added 256MB decode caps to the bincode decode paths in
  `wire.rs`, `artifacts.rs`, and `solver.rs`, with focused tests proving the
  limit machinery rejects over-budget payloads.
- [x] (2026-03-29) Audited artifact-loading callsites and tightened the public
  API around the already-verified manifest-backed path by removing the unused
  raw single-file encoder loader/export.
- [x] (2026-03-29) Added load-time codexpoker value-range validation so
  truncated or offset-invalid blueprint value files fail during blueprint load
  instead of degrading into silent lookup misses later.
- [x] (already landed before plan-008 execution) Checkpoints carry `MYOS` magic
  plus a version header in `solver.rs`.
- [x] (2026-03-29) Added proptest-based malformed-input coverage for the
  remaining decode boundaries in `wire.rs`, `artifacts.rs`, and `solver.rs`,
  proving they fail with `Err` instead of unwinding on truncated or tampered
  payloads.

## Surprises & Discoveries

- Observation: the active miner/validator/play callsites were already using
  `load_encoder_dir()`, so manifest-backed SHA-256 verification was not
  actually optional in the runtime-facing path. The real remaining gap was API
  shape: `myosu-games-poker` still exported an unused raw single-file loader
  that bypassed the manifest boundary entirely.
  Evidence: repo-wide callsite audit on 2026-03-29 plus
  `crates/myosu-games-poker/src/lib.rs`.
- Observation: `CodexpokerStore` had been trusting key-record offsets and
  lengths to point inside the mapped values file. A truncated values file could
  therefore survive load and only fail later as a lookup miss.
  Evidence: `crates/myosu-games-poker/src/codexpoker.rs` before the 2026-03-29
  load-time validation pass.
- Observation: checkpoint versioning was no longer missing by the time plan 008
  execution resumed. `solver.rs` already rejects wrong magic and wrong version,
  so the real first hardening gap was decode size caps, not checkpoint format.
  Evidence: `crates/myosu-games-poker/src/solver.rs` audit on 2026-03-29.

## Decision Log

- Decision: Use bincode's `DefaultOptions::new().with_limit(MAX_SIZE)` for all decode paths.
  Rationale: Prevents OOM from malformed or malicious input. MAX_SIZE = 256MB covers largest reasonable blueprint.
  Inversion: Without limits, a single crafted byte sequence can exhaust memory.
  Date/Author: 2026-03-28 / Genesis

- Decision: Checkpoint format: 4-byte magic (`MYOS`) + 2-byte version + payload.
  Rationale: Prevents silent corruption when format changes. Version enables forward-compatible migration.
  Date/Author: 2026-03-28 / Genesis

- Decision: No full PKI for artifact signing in stage-0. Use SHA-256 hash verification against known manifest.
  Rationale: PKI adds infrastructure complexity. Hash verification against a trusted manifest is sufficient for local and devnet use.
  Inversion: If artifacts are distributed over untrusted networks without PKI, hash verification alone is insufficient (MITM can replace both artifact and manifest).
  Date/Author: 2026-03-28 / Genesis

## Failure Modes

| Codepath | Realistic failure | Handling |
|---|---|---|
| bincode decode | Malformed checkpoint causes OOM | Size cap rejects before allocation |
| Artifact loading | Corrupted file passes hash check | SHA-256 catches single-bit errors; manifest hash covers all street files |
| mmap loading | Truncated file causes out-of-bounds read | Validate file size == expected_entries * entry_size before mapping |
| Checkpoint version | Old-format checkpoint loaded by new code | Magic byte check fails fast with clear error message |

## Outcomes & Retrospective

Plan 008 is complete at the intended stage-0 scope. The trust-boundary work
landed in four honest slices: size-capped decode guardrails, codexpoker
load-time mmap/value-range validation, artifact-surface tightening around the
already-verified manifest-backed loader, and property-style malformed-input
coverage for the remaining decode surfaces.

## Context and Orientation

Key files:
- `crates/myosu-games-poker/src/artifacts.rs` (400 lines) -- encoder loading, manifest verification, SHA-256
- `crates/myosu-games-poker/src/codexpoker.rs` (200 lines) -- mmap-based blueprint store
- `crates/myosu-games-poker/src/wire.rs` (300 lines) -- strategy query/response encoding
- `crates/myosu-games-poker/src/solver.rs` (330 lines) -- checkpoint save/load, MCCFR profile persistence

```text
ARTIFACT DATA FLOW

  Disk files                     In-memory
  +-----------------+           +-------------------+
  | encoder.bin     |--load---->| NlheEncoder       |
  | manifest.json   |--verify-->| NlheAbstractionM. |
  | *.abstraction   |--mmap---->| CodexpokerStore   |
  | checkpoint.bin  |--decode-->| PokerSolver state |
  +-----------------+           +-------------------+
       |                              |
       | SHA-256 verify               | bincode encode
       | size cap check               | version header
       | mmap bounds check            |
       v                              v
  Trusted in-memory state        Wire output
```

## Milestones

### Milestone 1: Size-capped bincode decode

Add `bincode::options().with_limit(MAX_DECODE_SIZE)` to all decode callsites in `wire.rs`, `solver.rs`, and `artifacts.rs`. Define `MAX_DECODE_SIZE = 256 * 1024 * 1024` (256MB).

Proof command:

    grep -rn "bincode::deserialize\|from_reader\|decode_from" crates/myosu-games-poker/src/{wire,solver,artifacts}.rs | wc -l
    # After: all callsites use options().with_limit()
    cargo test -p myosu-games-poker wire --quiet
    cargo test -p myosu-games-poker artifact --quiet

### Milestone 2: Artifact hash enforcement

Keep SHA-256 verification mandatory on the manifest-backed loader and remove or
avoid unverified artifact-loading entrypoints that would let application code
bypass the manifest boundary.

Proof command:

    cargo test -p myosu-games-poker artifact_hash --quiet

### Milestone 3: Mmap bounds validation

Validate codexpoker key-record offsets and lengths against the mapped values
file during blueprint load, and reject truncated value payloads with a clear
load-time error instead of silently deferring failure to lookup time.

Proof command:

    cargo test -p myosu-games-poker codexpoker_bounds --quiet

### Milestone 4: Checkpoint version header

Implement `MYOS` magic + version header for checkpoint save/load in `solver.rs`. Add migration path from unversioned checkpoints (detect by missing magic, assume version 0).

Proof command:

    cargo test -p myosu-games-poker checkpoint_version --quiet

### Milestone 5: Malformed input property tests

Add proptest-based malformed-input tests that mutate valid payloads into
truncated or tampered ones and verify the decode/load paths return `Err`
instead of unwinding.

Proof command:

    cargo test -p myosu-games-poker fuzz --quiet

## Plan of Work

1. Add size caps to bincode decode.
2. Tighten the artifact-loading API around manifest-backed verification.
3. Add mmap bounds validation.
4. Verify checkpoint version/header behavior stays intact.
5. Add malformed-input property tests.

## Concrete Steps

From `/home/r/coding/myosu`:

    grep -rn "bincode" crates/myosu-games-poker/src/
    cargo test -p myosu-games-poker --quiet

## Validation and Acceptance

Accepted when:
- All bincode decode paths have size caps
- Artifact loading rejects corrupted files
- Mmap validates file size before mapping
- Checkpoints carry version header
- Property-style malformed-input tests prove the remaining decode boundaries
  fail with `Err` rather than unwinding
- Fuzz tests pass (random bytes -> error, not panic)

## Idempotence and Recovery

All changes are additive. Existing checkpoints work via migration path (missing magic = version 0).

## Interfaces and Dependencies

Depends on: 006 (game trait boundaries locked).
Blocks: 009 (TUI productization needs trusted artifacts), 011 (security audit).

```text
wire.rs (size caps)
artifacts.rs (hash enforcement)
codexpoker.rs (mmap bounds)
solver.rs (version header)
         |
         v
proptest fuzz targets
```

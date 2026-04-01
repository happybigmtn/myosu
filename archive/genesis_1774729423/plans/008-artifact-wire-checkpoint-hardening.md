# Harden Artifacts, Wire Formats, and Checkpoints

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This plan follows `genesis/PLANS.md`.

## Purpose / Big Picture

Artifact loading and wire decoding are critical trust boundaries for advisor quality, miner training continuity, and validator determinism. This plan adds explicit format versioning, bounded decoding, integrity checks, and checkpoint validation for poker artifacts.

## Progress

- [x] (2026-03-28 21:38Z) Audited artifact and wire surfaces in poker/play crates and identified missing size/integrity boundaries.
- [ ] Add explicit 4-byte magic + version headers for wire/checkpoint payloads.
- [ ] Enforce decode size limits and reject untrusted oversized payloads.
- [ ] Add manifest hash verification for codexpoker blueprint assets before load.
- [ ] Add checkpoint save/load roundtrip tests with corruption cases.
- [ ] Expose clear operator diagnostics for artifact load failures in `myosu-play`.

## Surprises & Discoveries

- Observation: high-value artifact loading paths rely on local file trust without strong integrity policy.
  Evidence: `crates/myosu-games-poker/src/codexpoker.rs`, `artifacts.rs`, `crates/myosu-play/src/main.rs`.
- Observation: wire tests exist, but boundary tests for malformed/oversized payloads are sparse.
  Evidence: `crates/myosu-games-poker/src/wire.rs`, `robopoker.rs` tests.

## Decision Log

- Decision: keep wire/checkpoint format changes backward-compatible through explicit version dispatch.
  Rationale: avoids hard breaks for existing local artifacts.
  Inversion (failure mode): silent format mutation without versioning will corrupt old artifacts with no clear error.
  Date/Author: 2026-03-28 / Genesis

- Decision: reject by default when integrity metadata is missing.
  Rationale: this is a trust boundary, not a convenience layer.
  Inversion (failure mode): permissive fallback paths allow poisoned or partial artifacts into gameplay/validation.
  Date/Author: 2026-03-28 / Genesis

## Failure Modes

| Codepath | Realistic failure | Handling |
|---|---|---|
| Artifact decode | OOM or long stall on oversized payload | Add max byte thresholds before decode |
| Blueprint loading | Tampered file loaded as valid policy map | Verify manifest hashes before mmap/open |
| Checkpoint restore | Version mismatch loads wrong policy | Add magic/version dispatch and explicit error message |

## Outcomes & Retrospective

- Pending implementation.

## Context and Orientation

Owned files in this plan:
- `crates/myosu-games-poker/src/wire.rs`
- `crates/myosu-games-poker/src/artifacts.rs`
- `crates/myosu-games-poker/src/codexpoker.rs`
- `crates/myosu-games-poker/src/solver.rs`
- `crates/myosu-games-poker/src/robopoker.rs`
- `crates/myosu-games-poker/src/lib.rs`
- `crates/myosu-play/src/main.rs`

Not owned here:
- Generic gameplay trait boundaries (`006`)
- Miner/validator service scaffolding (`007`)

## Milestones

### Milestone 1: Versioned payload headers

Add and enforce `magic + version` for wire/checkpoint payloads.

Proof command:

    rg -n "MAGIC|VERSION|header" crates/myosu-games-poker/src/wire.rs crates/myosu-games-poker/src/artifacts.rs

### Milestone 2: Decode boundary checks

Enforce size limits before decode and add malformed payload tests.

Proof command:

    cargo test -p myosu-games-poker wire_types_serialize --quiet
    cargo test -p myosu-games-poker malformed --quiet

### Milestone 3: Blueprint integrity verification

Require manifest hash checks before loading keys/values/index payloads.

Proof command:

    rg -n "sha256|hash|manifest" crates/myosu-games-poker/src/codexpoker.rs
    cargo test -p myosu-games-poker codexpoker_blueprint_answers_request --quiet

### Milestone 4: Checkpoint corruption tests

Add save/load roundtrip with deliberate corruption and wrong-version cases.

Proof command:

    cargo test -p myosu-games-poker checkpoint --quiet

### Milestone 5: Operator diagnostics in play CLI

Surface actionable artifact-load errors in `myosu-play` startup and auto-discovery output.

Proof command:

    cargo test -p myosu-play auto_blueprint_assets_reports_incomplete_root_when_nothing_loads --quiet
    cargo test -p myosu-play auto_blueprint_assets_prefers_later_valid_root_over_earlier_incomplete_root --quiet

## Plan of Work

1. Add wire/checkpoint versioning.
2. Add decode and integrity boundaries.
3. Add corruption tests and CLI diagnostics.

## Concrete Steps

From `/home/r/coding/myosu`:

    sed -n '1,260p' crates/myosu-games-poker/src/wire.rs
    sed -n '1,320p' crates/myosu-games-poker/src/codexpoker.rs
    cargo test -p myosu-games-poker -p myosu-play --quiet

## Validation and Acceptance

Accepted when:
- versioned headers are mandatory for artifact payloads
- oversized/malformed payloads fail safely
- integrity checks block tampered assets
- play CLI reports artifact load issues clearly

## Idempotence and Recovery

- Roundtrip and corruption tests are repeatable.
- If migration compatibility fails, support both old and new format decoders behind explicit version dispatch until old artifacts are rotated.

## Artifacts and Notes

- Update `outputs/games/poker-engine/spec.md` and `outputs/play/tui/review.md` with the new artifact trust policy.

## Interfaces and Dependencies

Depends on: `006-game-traits-and-poker-boundaries.md`
Blocks: `007-miner-validator-bootstrap.md`, `009-play-tui-productization.md`, `011-security-observability-release.md`

```text
snapshot/request/query
        |
        v
wire + artifact codecs (versioned)
        |
        +--> checkpoint read/write
        +--> codexpoker blueprint load (hash verified)
        |
        v
play/miner/validator consume trusted strategy data
```

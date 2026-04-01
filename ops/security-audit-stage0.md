# Stage-0 Security Audit

Last updated: 2026-03-30
Status: Active stage-0 audit snapshot. Useful for release gating, not a formal
external pen-test.

## Scope

This audit covers the live stage-0 surfaces that now exist in the repo:

- stripped chain runtime, node, and `pallet-game-solver`
- artifact, wire, and checkpoint boundaries used by gameplay, miner, and validator
- miner and validator service entrypoints
- current release/truth surfaces in `ops/`, `genesis/plans/`, and `INVARIANTS.md`

## Findings

### SA-01: Chain attack surface is reduced, but owner-level bootstrap power is still intentionally strong

- Severity: Medium
- Surface: `crates/myosu-chain/runtime`, `crates/myosu-chain/pallets/game-solver`
- Evidence:
  - plan `003` stripped drand/crowdloan/frontier-era baggage from the live runtime path
  - plan `005` reduced the pallet to the stage-0 seam
  - local proof path is green under `SKIP_WASM_BUILD=1 cargo check -p myosu-chain-runtime --features fast-runtime`,
    `SKIP_WASM_BUILD=1 cargo check -p myosu-chain --features fast-runtime`, and
    `SKIP_WASM_BUILD=1 cargo check -p pallet-game-solver`
- Assessment:
  - the runtime no longer exposes the earlier inherited attack surface that made the restart unsafe to reason about
  - stage-0 still intentionally carries strong owner/sudo bootstrap paths for subnet normalization and validator bring-up
- Mitigation:
  - acceptable for stage-0 devnet and bootstrap use
  - keep these paths explicit in release docs and do not market them as production governance

### SA-02: Artifact and wire boundary hardening is present and directly evidenced

- Severity: Medium
- Surface: `crates/myosu-games-poker`
- Evidence:
  - plan `008` is complete
  - bounded decode and artifact validation surfaces are present in the live poker crate
  - gameplay, miner, and validator all use the same hardened artifact/wire seam
- Assessment:
  - the repo is no longer relying on unconstrained decode or trust-blind artifact loading as its primary stage-0 path
  - this closes the earlier untrusted-artifact and unsafe-wire risks enough for stage-0 use
- Mitigation:
  - keep artifact-loading proofs in CI and release gate
  - defer stronger signing/provenance guarantees to a later release phase

### SA-03: CI blind spot is materially improved

- Severity: Medium
- Surface: `.github/workflows/ci.yml`
- Evidence:
  - plan `010` added separate `chain-core`, `doctrine`, `plan-quality`, and `chain-clippy` jobs
  - the existing gameplay job now also runs under the honest `SKIP_WASM_BUILD=1` constraint instead of assuming a local wasm target
- Assessment:
  - the repo no longer has a gameplay-only CI story
  - compile/lint/doctrine regressions on the stripped chain and control plane are now gateable
- Mitigation:
  - hosted timing proof is now closed on the current repo surface
  - keep the workflow and repo-shape preflight current as the CI surface evolves

### SA-04: Release-governance doctrine is now grounded, but still the easiest drift vector

- Severity: Medium
- Surface: `INVARIANTS.md`, `ops/no-ship-ledger.md`, release-process docs
- Evidence:
  - `INVARIANTS.md` now anchors `INV-001`, `INV-005`, and `INV-006` to current
    repo surfaces instead of deleted Malinka files
  - `ops/stage0-completion-contract.md` now defines a concrete completion-claim
    and fail-closed rollback contract
- Assessment:
  - the operator doctrine is now much closer to the code and proof posture
  - it is still the easiest place for false claims or drift to re-enter if the
    completion surfaces stop being updated together
- Mitigation:
  - keep the release gate, no-ship ledger, and completion contract synced in
    the same slice as any future completion claim change
  - treat missing doctrine sync as a release blocker even when code is green

### SA-05: Key management remains an accepted stage-1 risk

- Severity: High
- Surface: operator keys, signing flow, secret handling
- Evidence:
  - plan `011` itself marks key-management infrastructure as deferred
  - no hardened operator-key lifecycle or secret rotation system exists in the repo
- Assessment:
  - acceptable only because the current stage is still local/devnet/bootstrap oriented
  - not acceptable as a production-claims surface
- Mitigation:
  - keep this risk explicitly accepted for stage-0
  - require dedicated key-management work before any stronger release claim

## Risk Mapping

| Risk | Current state | Notes |
|------|---------------|-------|
| SR-01 Large runtime attack surface | Mitigated for stage-0 | reduced by plans `003` and `005` |
| SR-02 Untrusted artifact loading | Mitigated for stage-0 | hardened in plan `008` |
| SR-03 Unsafe mmap without bounds | Mitigated for stage-0 | hardened in plan `008` |
| SR-04 Decode without size caps | Mitigated for stage-0 | hardened in plan `008` |
| SR-05 CI blind spot for chain | Mitigated for stage-0 | structurally fixed in plan `010` and proven on hosted GitHub Actions |
| SR-06 No key-management infrastructure | Accepted stage-0 risk | still a release-upgrade blocker |

## Conclusion

The stage-0 codebase is in a meaningfully safer posture than the original
bootstrap audit assumed. The highest-value hardening work already landed in the
runtime/pallet reduction, artifact boundary hardening, CI expansion, and the
release-governance sync that grounded the remaining invariants on live repo
surfaces. The largest remaining security gap is now stage-1 key-management
maturity, not missing stage-0 gate categories.

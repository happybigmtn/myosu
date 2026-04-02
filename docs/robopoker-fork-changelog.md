# Robopoker Fork Changelog

This repo currently consumes the `happybigmtn/robopoker` fork at:

- upstream baseline: `v1.0.0`
- pinned fork rev: `04716310143094ab41ec7172e6cea5a2a66744ef`
- local audit path: `/home/r/coding/robopoker`
- audit date: `2026-03-29`

The checked-out local fork HEAD matches the workspace pin and is clean at the
time of audit.

## Changes Since `v1.0.0`

Commits beyond `v1.0.0`:

- `c87a8e4` `feat: add serde feature for serializable NLHE types`
- `2323afc` `RF-01: Add serde feature for NLHE types serialization`
- `0471631` `fix: gate serde_test.rs on serde feature`

`git diff --stat v1.0.0..04716310143094ab41ec7172e6cea5a2a66744ef` reports 28
changed files with 190 insertions and 5 deletions.

## Functional Summary

The current fork divergence is still narrow. It is almost entirely the RF-01
surface needed by Myosu's game-wire and checkpoint work:

- add serde feature wiring across `rbp-core`, `rbp-mccfr`, `rbp-cards`,
  `rbp-gameplay`, `rbp-nlhe`, and the umbrella `rbp` crate
- derive or gate serde support for the NLHE/card/gameplay types Myosu needs to
  serialize
- add `crates/nlhe/tests/serde_test.rs`
- fix that serde test so it only compiles when the serde feature is enabled

There is no audited evidence yet in the pinned fork rev for RF-02, RF-03, or
RF-04. As of this audit, the fork divergence is still a serde-only extension.

## Files Touched

High-signal areas changed from `v1.0.0` to the pinned rev:

- `crates/cards/*`
- `crates/gameplay/*`
- `crates/mccfr/*`
- `crates/nlhe/*`
- `crates/rbp/Cargo.toml`

The largest single addition is:

- `crates/nlhe/tests/serde_test.rs`

## Why This Exists

INV-006 requires the fork to remain anchored to the proven `v1.0.0` MCCFR
baseline with explicit documentation of every downstream change. This file is
the repo-local record for that requirement until or unless the fork itself
carries an equivalent `CHANGELOG.md`.

## Remaining Obligation

If the fork grows beyond the current serde support, this file must be updated
in the same slice that changes the pinned robopoker rev in the workspace.
Security-driven pin changes follow
[`ops/cve-tracking-process.md`](../ops/cve-tracking-process.md) and should
update both this changelog and the stage-0 security audit snapshot together.

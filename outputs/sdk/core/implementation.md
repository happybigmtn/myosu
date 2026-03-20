# `sdk:core` Implementation

## Lane
`core-implement`

## Slice
Slice 3 — Scaffold Tool (`AC-SDK-02`)

## Goal
Replace the initial scaffold surface with a real generator that emits a compileable `myosu-games-<name>` crate aligned to the reviewed robopoker trait surface and the `CfrGame: Copy` constraint.

## Touched Surfaces
- `Cargo.lock`
- `crates/myosu-sdk/Cargo.toml`
- `crates/myosu-sdk/src/lib.rs`
- `crates/myosu-sdk/src/scaffold/mod.rs`
- `crates/myosu-sdk/src/scaffold/templates.rs`
- `crates/myosu-sdk/src/scaffold/tests.rs`

## What Changed
- Added the minimal SDK-facing dependencies and re-exports the scaffolded crate needs to depend on `myosu-sdk` alone:
  - `rbp-transport::Support`
  - `rbp-mccfr::{Branch, Tree, Node, CfrPublic, CfrSecret}`
  - `GameRenderer`, `Buffer`, and `Rect` behind the existing `tui` feature
- Reworked `ScaffoldGenerator` into a real Slice 3 API:
  - validates lowercase game names
  - preserves the expected package name format `myosu-games-<name>`
  - adds `crate_name()` and `generate()` helpers
  - computes the SDK dependency path relative to the generated crate instead of hardcoding `../../...`
- Replaced the old template bodies with compileable stubs that match the actual robopoker traits in this repo:
  - `GameAction`, `GameTurn`, `GamePublicInfo`, `GameSecretInfo`, and `GameInfo`
  - `Game` uses fixed-size history storage so the scaffold demonstrates the `Copy` constraint directly
  - `GameEncoder` implements the upstream `Encoder` trait signature
  - `renderer.rs` is gated by a generated `tui = ["myosu-sdk/tui"]` feature and compiles against the real `GameRenderer` API
- Tightened the scaffolded compliance test so it stays compileable today while still failing at the intended boundary:
  - the file contains the intended `assert_game_valid::<Game>()` follow-up as an inline replacement comment
  - the generated test fails via `todo!()` with an explicit compliance message until the developer wires their real game in
- Upgraded Slice 3 proof coverage:
  - the scaffold tests now shell out to nested Cargo checks on the generated crate
  - default and `--features tui` compilation are both verified
  - generated tests are verified to fail for the expected scaffold `todo!()` reason
  - overwrite refusal and default-directory generation are both covered

## Notes
- Slice 2 test-harness behavior was preserved and re-verified after the scaffold work.
- Registration and documentation surfaces were left untouched in this pass.

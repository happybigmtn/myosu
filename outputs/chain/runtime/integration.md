# `chain:runtime` Integration — Phase 0

## Workspace Integration

This slice integrates the chain restart surfaces into the root Cargo workspace
without claiming that runtime execution works yet.

- `crates/myosu-chain` is now the workspace anchor for the chain restart lane.
- `myosu-runtime`, `myosu-chain-common`, and `myosu-node` now exist as explicit
  Cargo packages under that anchor.
- Root `default-members` stay limited to the previously active crates plus the
  new anchor crate, so ordinary repo-root builds do not automatically pull the
  runtime implementation slice forward.

## Downstream Contract

After this slice:

- downstream commands can target `-p myosu-runtime` and `-p myosu-node`
- Phase 1 runtime work no longer needs to create package manifests or fix root
  workspace visibility first
- the existing `pallet-game-solver` crate remains intact on the same
  `polkadot-sdk` `stable2407` line

## Expected Next Integration Step

The next runtime slice should integrate a minimal runtime implementation into
the already-wired `myosu-runtime` package. No further root-workspace surgery
should be necessary unless the minimal runtime needs additional standard
Substrate crates.

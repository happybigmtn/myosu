# `chain:runtime` Implementation — Restart Slice 1 Fixup

## Slice Repaired

This fixup repairs the **proof contract** for Restart Slice 1 without widening
the runtime implementation scope.

The runtime code stays on the same approved Phase 1 surface:

- workspace member `crates/myosu-chain/runtime`
- minimal FRAME runtime in `crates/myosu-chain/runtime/src/lib.rs`
- runtime-side genesis preset plumbing in `crates/myosu-chain/runtime/src/chain_spec.rs`

The concrete failure was in verification, not runtime logic:

- the slice proof in `outputs/chain/runtime/spec.md` was written as prose, which
  produced a malformed shell script during verification
- the same proof shape also let a future `myosu-node` command leak into the
  active runtime-only slice
- in this sandbox, raw `cargo` proof commands also default to a read-only shared
  target directory unless `CARGO_TARGET_DIR` is set explicitly

## Files Changed

| File | Change |
|------|--------|
| `outputs/chain/runtime/spec.md` | Added an explicit "current approved slice proof" section, restricted the active gate to Phase 1, and rewrote the runtime proof as executable commands using `CARGO_TARGET_DIR=.raspberry/cargo-target` |
| `outputs/chain/runtime/implementation.md` | Replaced the prior implementation note with this fixup-specific slice description |
| `outputs/chain/runtime/verification.md` | Replaced the verification note with the actual commands run during fixup and their outcomes |
| `outputs/chain/runtime/integration.md` | Updated the integration contract to describe the Phase 1-only proof boundary and sandbox-local cargo target requirement |

No runtime source files changed in this fixup because the observed failure was a
deterministic proof-script problem, not a Rust compiler error in the runtime
crate.

## Proof Commands For The Active Slice

```bash
env CARGO_TARGET_DIR=.raspberry/cargo-target cargo check -p myosu-runtime
env CARGO_TARGET_DIR=.raspberry/cargo-target cargo build -p myosu-runtime --release
```

These are the commands the active Slice 1 proof gate should run. Phase 2 node
proof is deliberately out of scope for this fixup.

## What Remains Next

| Next step | Description |
|-----------|-------------|
| Finish Phase 1 proof | Let the env-prefixed runtime proof commands complete to a final exit status in a clean verification run |
| Only then start Phase 2 | Add `myosu-node` after the runtime-only proof is fully green; do not mix node proof into Slice 1 again |

## Stage-Owned Outputs

Per fixup instructions, `outputs/chain/runtime/quality.md` was not hand-authored
and `outputs/chain/runtime/promotion.md` was not created or rewritten here.

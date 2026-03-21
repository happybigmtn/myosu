# `chain:runtime` Integration — Slice 1

## Runtime Handoff

This slice leaves the runtime lane in a state that downstream work can actually consume:

- `myosu-runtime` is a real workspace package with a proven Wasm build
- `myosu-chain-common` is a real workspace package for shared types/utilities
- the root workspace now resolves the chain restart packages through the same pinned `stable2407` SDK line

## Downstream Impact

- **Next `chain:runtime` work** can focus on `myosu-node` instead of continuing to fight dead manifests.
- **`chain:pallet`** remains intentionally unintegrated here. The runtime still contains only the minimal core pallets.
- **Service lanes** are still blocked on node/devnet bring-up, but the upstream runtime build blocker has been reduced to node work instead of missing workspace plumbing.

## Integration Notes

- In sandboxed or offline environments, use:

```bash
WASM_BUILD_WORKSPACE_HINT="$PWD" CARGO_NET_OFFLINE=true
```

for runtime proof commands so the nested Wasm builder uses the workspace lockfile and local dependency cache.

- The runtime artifacts were verified under `/tmp/myosu-chain-target/...`; no repo-local target dir assumptions are required.

## Stage Ownership Note

`quality.md` and `promotion.md` were intentionally not written in this slice:

- `quality.md` is owned by the Quality Gate
- `promotion.md` is owned by the Review stage

That keeps this implementation pass inside the lane contract instead of hand-authoring later-stage artifacts early.

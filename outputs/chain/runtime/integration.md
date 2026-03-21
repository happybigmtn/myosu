# `chain:runtime` Integration — Slice 1

## Runtime Handoff

This slice leaves the runtime lane in a state that downstream work can consume:

- `myosu-runtime` is a real workspace package with a proven Wasm build
- `myosu-chain-common` is a real workspace package for shared types and utilities
- the root workspace resolves the chain restart packages through the same pinned `stable2407` SDK line

## Proof Boundary For Downstream Lanes

- `chain:runtime` is proven only through the minimal runtime crate in this slice
- node packaging, chain spec wiring, and block production are still deferred
- `chain:pallet` remains intentionally unintegrated here; the runtime still contains only the minimal core pallets

Downstream lanes should treat the current handoff as "runtime build path proven"
rather than "node/devnet ready."

## Integration Notes

- `./fabro/checks/chain-runtime-reset.sh` is now the canonical Slice 1 proof entrypoint for Fabro/Raspberry automation. Downstream lanes should invoke that script instead of inferring commands from prose.
- The proof script ignores the shell's ambient `CARGO_TARGET_DIR` and uses `/tmp/myosu-chain-target` by default so automation stays inside a writable sandbox path. Set `MYOSU_CHAIN_TARGET_DIR` if a different writable target dir is required.
- In sandboxed or offline environments, run the runtime proof with `WASM_BUILD_WORKSPACE_HINT="$PWD"` and `CARGO_NET_OFFLINE=true` so the nested Wasm builder uses the workspace lockfile and local dependency cache.
- The verified Wasm outputs live under `/tmp/myosu-chain-target/release/wbuild/myosu-runtime/`; no repo-local target directory assumptions are required.
- The earlier implementation run also carried adjacent edits outside the runtime-owned proof boundary. This fixup does not broaden integration claims to cover those surfaces.

## Stage Ownership Note

`quality.md` and `promotion.md` remain intentionally unwritten here:

- `quality.md` is owned by the Quality Gate
- `promotion.md` is owned by the Review stage

That keeps this implementation fixup inside the lane contract instead of
hand-authoring later-stage artifacts early.

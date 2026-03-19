Read these inputs before you write anything:

- `crates/myosu-chain/runtime/`
- `crates/myosu-chain/node/`
- `crates/myosu-chain/common/`
- `Cargo.toml`
- `outputs/chain/runtime/spec.md` if it already exists
- `outputs/chain/runtime/review.md` if it already exists

Your task is to produce or replace `outputs/chain/runtime/spec.md`.

This is a restart lane, not a normal implementation lane. The `spec.md` must:

- inventory the current runtime/node/common surfaces
- state clearly why the current runtime path is not yet trustworthy
- define the smallest honest restart boundary
- define the next phased implementation slices
- define the proof shape for each phase

Do not pretend the runtime is already buildable if the repo does not prove it.

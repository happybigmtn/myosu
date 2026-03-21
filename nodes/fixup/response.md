Updated the current `chain:runtime` fixup artifacts in [implementation.md](/home/r/.fabro/runs/20260320-01KM6ZFDDEXKQB27C3Y3WSP01R/worktree/outputs/chain/runtime/implementation.md), [verification.md](/home/r/.fabro/runs/20260320-01KM6ZFDDEXKQB27C3Y3WSP01R/worktree/outputs/chain/runtime/verification.md), and [integration.md](/home/r/.fabro/runs/20260320-01KM6ZFDDEXKQB27C3Y3WSP01R/worktree/outputs/chain/runtime/integration.md). The fix keeps the lane on the current approved slice only: Phase 1 runtime bring-up, with a bounded proof contract that explicitly does not claim node bring-up yet. I left `quality.md` and `promotion.md` untouched per the lane rules.

I reran the approved proof commands and recorded the outcomes in the verification artifact:
- `CARGO_TARGET_DIR=/tmp/myosu-chain-target cargo check --offline -p myosu-chain-common` -> exit 0
- `CARGO_TARGET_DIR=/tmp/myosu-chain-target cargo check --offline -p myosu-chain` -> exit 0
- `WASM_BUILD_WORKSPACE_HINT="$PWD" CARGO_NET_OFFLINE=true CARGO_TARGET_DIR=/tmp/myosu-chain-target cargo check --offline -p myosu-runtime` -> exit 0
- `WASM_BUILD_WORKSPACE_HINT="$PWD" CARGO_NET_OFFLINE=true CARGO_TARGET_DIR=/tmp/myosu-chain-target cargo build --offline --release -p myosu-runtime` -> exit 0

The Wasm artifacts were confirmed under `/tmp/myosu-chain-target/release/wbuild/myosu-runtime/`. Residual risk remains the same: `myosu-node` is still not a workspace package, and the `trie-db v0.29.1` future-incompat warning is still present.
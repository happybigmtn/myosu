# LEARNINGS

## 2026-04-02

- `myosu-play --smoke-test` must bypass `~/.codexpoker` auto-discovery when it is being used as a smoke proof. Ambient local blueprint state can hide real regressions; the truthful default for smoke mode is a built-in demo surface unless the proof explicitly targets blueprint loading.
- `cargo audit` is not a truthful security gate in this repo unless warning-class advisories are handled explicitly. A zero exit status paired with `13 allowed warnings found` still leaves live security debt in the tree.
- Moving owned code from `bincode` 1.x to 2.x is not a fix for `RUSTSEC-2025-0141`; both major lines are currently flagged unmaintained, so the real decision is replacement, isolation, or explicit acceptance.

## 2026-04-03

- A successful `cargo check` is not enough to clear a runtime-reduction review item when the governing spec and the shipped runtime disagree on the accepted pallet count. Archive only after the contract text and the code describe the same stage-0 shape.
- Trimming pallet calls out of metadata is a different contract from returning an explicit unavailable dispatch error. Review the call surface that clients see, not just the surviving dispatch count.
- A devnet proof that injects `--bootnodes` at runtime does not prove the chain spec itself carries bootnode addresses. Build-spec output and node startup arguments are separate evidence surfaces.
- Zeroing root-weight terms is not the same as removing AMM or swap-shaped logic from the stage-0 coinbase path. Review single-token emission claims against the whole execution path, not just the final weighting variables.

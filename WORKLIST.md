# WORKLIST

## 2026-04-02 review follow-ups

- `RT-004` Repair the review provenance for the TODO/FIXME cleanup claim. The current `REVIEW.md` entry points at `0a5273c76d735fc85e50df80218e64492765626c`, which is `myosu: EM-001 remove root stake weighting`, not a backlog-reduction slice. Either find the real cleanup commit and re-review it, or land a dedicated replacement increment with proof.

## 2026-04-03 review follow-ups

- `SEC-001` Triage the warning-class advisories now that the CI gate is truthful. `cargo audit -D warnings` with the current ignore list still fails on `RUSTSEC-2025-0141` (`bincode`), `RUSTSEC-2024-0388` (`derivative`), `RUSTSEC-2025-0057` (`fxhash`), `RUSTSEC-2024-0384` (`instant`), `RUSTSEC-2020-0168` (`mach`), `RUSTSEC-2022-0061` (`parity-wasm`), `RUSTSEC-2024-0436` (`paste`), `RUSTSEC-2024-0370` (`proc-macro-error`), `RUSTSEC-2025-0010` (`ring`), `RUSTSEC-2021-0127` (`serde_cbor`), `RUSTSEC-2026-0002` (`lru`), and `RUSTSEC-2024-0442` (`wasmtime-jit-debug`). Owned crates still pull `bincode 1.3.3` directly (`myosu-games-kuhn`, `myosu-games-liars-dice`, `myosu-games-poker`, and downstream consumers), so this is not just inherited chain debt.
- `RT-001` Reconcile `specs/040226-01-chain-runtime-reduction.md` with the shipped stage-0 runtime. The current runtime composes nine pallets, not the spec's "seven or fewer" claim.
- `RT-003` Restore the intended unavailable-extrinsic contract or update the spec/API claim. The current implementation removes trimmed calls from metadata entirely instead of returning an explicit dispatch error.
- `EM-001` Remove or explicitly justify the remaining swap/price-shaped helpers in the stage-0 coinbase path (`inject_and_maybe_swap`, price-derived subnet terms) so the single-token emission claim is truthful.
- `EM-002` Add proof that actually covers cross-validator scoring determinism, or rename/rescope the current unit proof to Yuma-epoch determinism only.
- `DN-001` Embed truthful bootnodes in the devnet chain spec or narrow the claim to the operator-bundle rewrite path. The current proof depends on external `--bootnodes` injection.

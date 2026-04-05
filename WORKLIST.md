# WORKLIST

## 2026-04-02 review follow-ups

- `SEC-001` Triage the warning-class advisories now that the CI gate is truthful. `cargo audit -D warnings` with the current ignore list still fails on `RUSTSEC-2025-0141` (`bincode`), `RUSTSEC-2024-0388` (`derivative`), `RUSTSEC-2025-0057` (`fxhash`), `RUSTSEC-2024-0384` (`instant`), `RUSTSEC-2020-0168` (`mach`), `RUSTSEC-2022-0061` (`parity-wasm`), `RUSTSEC-2024-0436` (`paste`), `RUSTSEC-2024-0370` (`proc-macro-error`), `RUSTSEC-2025-0010` (`ring`), `RUSTSEC-2021-0127` (`serde_cbor`), `RUSTSEC-2026-0002` (`lru`), and `RUSTSEC-2024-0442` (`wasmtime-jit-debug`). Owned crates still pull `bincode 1.3.3` directly (`myosu-games-kuhn`, `myosu-games-liars-dice`, `myosu-games-poker`, and downstream consumers), so this is not just inherited chain debt.

## 2026-04-05 follow-ups

- `EM-DUST-001` Decide whether stage-0 coinbase should accumulate, recycle, or explicitly track truncation dust. The `cargo test -p pallet-game-solver -- truncation` sweep now measures a worst-case loss of 2 rao per accrued block, which is 6 rao over the default tempo-2 epoch and exceeds the `P-002` investigation threshold for a correction decision.
- `CI-SEC-001` Decide whether to replace `dtolnay/rust-toolchain` with `rustup` script steps or carry an explicit `zizmor` allowance. Raw `zizmor .github/workflows/ci.yml` is now clean on pinning/permissions/artifact risks but still emits six low-severity `superfluous-actions` advisories against the current helper action.
- `DOC-OPS-001` Resolve the dangling `@RTK.md` reference at the top of `AGENTS.md` or restore the missing file. The repo currently has no `RTK.md`, so future loops cannot faithfully consume that referenced contract.

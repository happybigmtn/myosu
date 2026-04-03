# WORKLIST

## 2026-04-02 review follow-ups

- `SEC-001` Fix the dependency-audit gate so it is truthful. `cargo audit` with the current ignore list still reports `13 allowed warnings found`: `RUSTSEC-2025-0141` (`bincode`), `RUSTSEC-2024-0388` (`derivative`), `RUSTSEC-2025-0057` (`fxhash`), `RUSTSEC-2024-0384` (`instant`), `RUSTSEC-2020-0168` (`mach`), `RUSTSEC-2022-0061` (`parity-wasm`), `RUSTSEC-2024-0436` (`paste`), `RUSTSEC-2024-0370` (`proc-macro-error`), `RUSTSEC-2025-0010` (`ring`), `RUSTSEC-2021-0127` (`serde_cbor`), `RUSTSEC-2026-0002` (`lru`), and `RUSTSEC-2024-0442` (`wasmtime-jit-debug`). Owned crates still pull `bincode 1.3.3` directly (`myosu-games-kuhn`, `myosu-games-liars-dice`, `myosu-games-poker`, and downstream consumers), so this is not just inherited chain debt.
- `RT-004` Repair the review provenance for the TODO/FIXME cleanup claim. The current `REVIEW.md` entry points at `0a5273c76d735fc85e50df80218e64492765626c`, which is `myosu: EM-001 remove root stake weighting`, not a backlog-reduction slice. Either find the real cleanup commit and re-review it, or land a dedicated replacement increment with proof.

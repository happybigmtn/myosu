# WORKLIST

## 2026-04-02 review follow-ups

- `SEC-001` Triage and remove the current `cargo audit` allowlist. CI now mirrors this deferred advisory set so unrelated work can land, but the remediation task is still open for `RUSTSEC-2025-0141` (`bincode`), `RUSTSEC-2024-0388` (`derivative`), `RUSTSEC-2025-0057` (`fxhash`), `RUSTSEC-2024-0384` (`instant`), `RUSTSEC-2020-0168` (`mach`), `RUSTSEC-2022-0061` (`parity-wasm`), `RUSTSEC-2024-0436` (`paste`), `RUSTSEC-2024-0370` (`proc-macro-error`), `RUSTSEC-2025-0010` (`ring`), `RUSTSEC-2021-0127` (`serde_cbor`), `RUSTSEC-2026-0002` (`lru`), and `RUSTSEC-2024-0442` (`wasmtime-jit-debug`). Owned crates still pull `bincode 1.3.3` directly (`myosu-games-kuhn`, `myosu-games-liars-dice`, `myosu-games-poker`, and downstream consumers), so this is not just inherited chain debt.

## 2026-04-05 follow-ups

- `EM-DUST-001` Resolved 2026-04-08 by [ADR 011](/home/r/coding/myosu/docs/adr/011-emission-dust-policy.md). Stage-0 no longer drops the owner/server/validator split remainder: `run_coinbase` now closes the integer budget in the validator bucket, `try_state` tightened from `1_000` rao to `1`, and the pallet/E2E emission proofs now enforce the exact-budget contract instead of a wide dust tolerance.
- `CI-SEC-001` Decide whether to replace `dtolnay/rust-toolchain` with `rustup` script steps or carry an explicit `zizmor` allowance. Raw `zizmor .github/workflows/ci.yml` is now clean on pinning/permissions/artifact risks but still emits six low-severity `superfluous-actions` advisories against the current helper action.
- `AXON-HTTP-001` Revisit Liar's Dice HTTP axon parity only if remote validator queries become necessary. Stage-0 now intentionally keeps `myosu-miner --serve-http` poker-only and uses the bounded file-based query/response path for Liar's Dice validation.
- `MINER-QUAL-001` Narrowed 2026-04-08 by the new Liar's Dice
  `quality_benchmark` proof in
  [crates/myosu-validator/src/validation.rs](/home/r/coding/myosu/crates/myosu-validator/src/validation.rs)
  and the operator guidance in
  [docs/operator-guide/quickstart.md](/home/r/coding/myosu/docs/operator-guide/quickstart.md):
  the repo now has a truthful, exploitability-based Liar's Dice benchmark and a
  current recommendation of `512` minimum training iterations. Poker is no
  longer missing a benchmark path: `docs/execution-playbooks/poker-quality-benchmark.md`
  and `bash ops/poker_quality_benchmark.sh --db-url ... --robopoker-dir ... --encoder-dir ...`
  now show how to generate a full encoder from robopoker's PostgreSQL
  `isomorphism` table and measure exploitability through
  `cargo run -p myosu-games-poker --example quality_benchmark -- <encoder-dir> ...`.
  The remaining blocker is recording a real full-encoder poker exploitability
  ladder and turning it into a minimum-iterations recommendation. The
  validator's same-checkpoint path still self-scores the miner response, and
  the checked-in poker bootstrap artifacts remain intentionally sparse enough
  that positive-iteration poker training fails upstream with
  `isomorphism not found`.

## 2026-04-08 follow-ups

- `CHAIN-SDK-002` Decide whether to keep the checked-in Aura->Babe transition surface and what
  non-local transaction-pool replacement policy Myosu actually wants before attempting any
  upstream `polkadot-sdk` re-pin. `RES-002` classified 10 fork-only commits as currently needed,
  8 as safe-drop, and 3 as uncertain; the biggest leverage before a migration spike is deleting
  dead Aura->Babe or txpool behavior instead of backporting it by inertia.
- `DOC-RUNTIME-001` Refresh `specs/070426-runtime-architecture.md` so it matches the live post-DEBT-003 runtime surface. `GATE-001` verified the code and plan acceptance criteria against `pallet_game_solver` / `GameSolver`, but the spec still describes the older `pallet_subtensor` alias and `SubtensorModule` naming.

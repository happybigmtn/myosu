# WORKLIST

## 2026-04-02 review follow-ups

- `SEC-001` Triage and remove the current `cargo audit` allowlist. CI now mirrors this deferred advisory set so unrelated work can land, but the remediation task is still open for `RUSTSEC-2025-0141` (`bincode`), `RUSTSEC-2024-0388` (`derivative`), `RUSTSEC-2025-0057` (`fxhash`), `RUSTSEC-2024-0384` (`instant`), `RUSTSEC-2020-0168` (`mach`), `RUSTSEC-2022-0061` (`parity-wasm`), `RUSTSEC-2024-0436` (`paste`), `RUSTSEC-2024-0370` (`proc-macro-error`), `RUSTSEC-2025-0010` (`ring`), `RUSTSEC-2021-0127` (`serde_cbor`), `RUSTSEC-2026-0002` (`lru`), and `RUSTSEC-2024-0442` (`wasmtime-jit-debug`). Owned crates still pull `bincode 1.3.3` directly (`myosu-games-kuhn`, `myosu-games-liars-dice`, `myosu-games-poker`, and downstream consumers), so this is not just inherited chain debt.

## 2026-04-05 follow-ups

- `EM-DUST-001` Resolved 2026-04-08 by [ADR 011](/home/r/coding/myosu/docs/adr/011-emission-dust-policy.md). Stage-0 no longer drops the owner/server/validator split remainder: `run_coinbase` now closes the integer budget in the validator bucket, `try_state` tightened from `1_000` rao to `1`, and the pallet/E2E emission proofs now enforce the exact-budget contract instead of a wide dust tolerance.
- `CI-SEC-001` ~~Decide whether to replace `dtolnay/rust-toolchain` with `rustup` script steps or carry an explicit `zizmor` allowance.~~ **Resolved 2026-06-04** by `.github/zizmor.yml` + `tests/e2e/zizmor_policy.sh` + ADR-013 (`docs/adr/013-zizmor-policy-decision.md`): carry the per-line zizmor allowance for the 8 `dtolnay/rust-toolchain` `superfluous-actions` lines (one per affected job, including the new `zizmor-policy` job), each with a `# reason:` justification directly above the rule. The same commit also fixed a phantom-SHA bug on `actions/checkout` line 458 (the developer-quickstart job's reference used a single-nibble-typo'd SHA that returned HTTP 404 on github.com; the other 12 `actions/checkout` references already used the real v6 release SHA). Rejected alternative: switch to raw `rustup` script steps — would break `Swatinem/rust-cache` cache key coherence (the cache action keys off the toolchain action's toolchain selection, not the default `~/.cargo` location rustup uses) and would expand the attack surface for the chain-build path with a multi-line `curl ... | sh` chain that zizmor's `artipacked` and `template-injection` audits routinely flag. New `zizmor-policy` CI job installs zizmor via `cargo install --locked zizmor --version '^1.25' --root /usr/local` and runs the proof harness on every PR / push to trunk.
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
  `isomorphism not found`. **NEM-001B follow-up (2026-06-04)** ships the
  truthful poker substitute: `crates/myosu-validator/src/validation.rs` gains
  the `PokerQualityBenchmarkPoint` + `POKER_REFERENCE_LADDER` +
  `POKER_USEFUL_REFERENCE_MATCH_RATIO` + `poker_quality_benchmark_points` +
  `PokerQualityBenchmarkReport` surface, the `poker_quality_benchmark` example
  binary, and `tests/e2e/poker_quality_benchmark.sh` (the 6-sub-check CI
  proof). The mix-ladder (`mix=0.0..=1.0`, default ladder `[0.0, 0.25, 0.5,
  0.75, 1.0]`) is monotonically non-increasing in mean L1 distance and
  monotonically non-decreasing in exact-action-match ratio against the
  80-scenario `bootstrap_scenarios()` reference pack on a fresh checkout;
  `mix=1.0` is the bit-exact self-match anchor (`mean_l1_distance=0.0`,
  `exact_action_matches=80`). The remaining unblock for a real positive-iteration
  poker exploitability ladder remains PROMOTE-001 (external artifact supply),
  which is outside NEM-001B's scope by design.

## 2026-04-08 follow-ups

- `CHAIN-SDK-002` Decide whether to keep the checked-in Aura->Babe transition surface and what
  non-local transaction-pool replacement policy Myosu actually wants before attempting any
  upstream `polkadot-sdk` re-pin. `RES-002` classified 10 fork-only commits as currently needed,
  8 as safe-drop, and 3 as uncertain; the biggest leverage before a migration spike is deleting
  dead Aura->Babe or txpool behavior instead of backporting it by inertia.
- `DOC-RUNTIME-001` Refresh `specs/070426-runtime-architecture.md` so it matches the live post-DEBT-003 runtime surface. `GATE-001` verified the code and plan acceptance criteria against `pallet_game_solver` / `GameSolver`, but the spec still describes the older `pallet_subtensor` alias and `SubtensorModule` naming.

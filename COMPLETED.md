# COMPLETED

- `TEST-002` Added corrupted-keyfile recovery coverage in
  [crates/myosu-keys/src/storage.rs](/home/r/coding/myosu/crates/myosu-keys/src/storage.rs)
  by exercising the real `load_active_pair()` path against a deliberately
  truncated active key JSON file. The new regression proves `myosu-keys`
  returns `KeyError::DeserializeKeyfile` with a clear parse-failure message
  instead of panicking or silently producing bad key material, and the task
  plan’s claim about missing corruption coverage is now closed. Removed
  `TEST-002` from
  [IMPLEMENTATION_PLAN.md](/home/r/coding/myosu/IMPLEMENTATION_PLAN.md).
  Validation: `SKIP_WASM_BUILD=1 cargo test -p myosu-keys --quiet -- corrupt`; `SKIP_WASM_BUILD=1 cargo test -p myosu-keys --quiet`.
  Commit: `PENDING`

- `TEST-001` Added miner HTTP axon security coverage in
  [crates/myosu-miner/src/axon.rs](/home/r/coding/myosu/crates/myosu-miner/src/axon.rs)
  for the live TCP server path: malformed `/strategy` payloads now have an
  explicit regression test proving they return `400` without panicking,
  oversized requests with a `>1 MiB` declared body are verified to hit the
  current 64 KiB server limit and return a clean rejection, and 16 concurrent
  strategy requests are exercised against one server instance to prove the
  sequential accept loop completes without deadlock or response corruption.
  The task plan's claim that no axon tests existed was stale; the live code
  already covered health/strategy happy paths, and this increment closes the
  missing security-focused cases. Removed `TEST-001` from
  [IMPLEMENTATION_PLAN.md](/home/r/coding/myosu/IMPLEMENTATION_PLAN.md).
  Validation: `cargo test -p myosu-miner --quiet -- axon`; `SKIP_WASM_BUILD=1 cargo test -p myosu-miner --quiet -- axon`; `SKIP_WASM_BUILD=1 cargo test -p myosu-miner --quiet`; `cargo fmt --check`.
  Commit: `a728d82`

- `EMIT-001` Closed the stage-0 emission dust policy with
  [ADR 011](/home/r/coding/myosu/docs/adr/011-emission-dust-policy.md): the
  coinbase split now closes its integer remainder in the validator bucket
  instead of dropping up to 2 rao per accrued block, the `try_state`
  `TotalIssuance` alert delta tightened from `1_000` rao to `1`, and the
  pallet/E2E emission proofs now enforce the exact-budget contract rather than
  a wide rounding tolerance. Updated
  [WORKLIST.md](/home/r/coding/myosu/WORKLIST.md) to resolve `EM-DUST-001` and
  removed `EMIT-001` from
  [IMPLEMENTATION_PLAN.md](/home/r/coding/myosu/IMPLEMENTATION_PLAN.md).
  Validation: `test -f docs/adr/011-emission-dust-policy.md`; `cargo test -p pallet-game-solver --quiet -- truncation`; `cargo test -p pallet-game-solver --quiet -- stage_0_coinbase_emission_accounting_matches_accrued_epoch_budget`; `bash tests/e2e/emission_flow.sh`; `bash .github/scripts/check_doctrine_integrity.sh`.
  Commit: `2c5eb39`

- `GATE-001` Verified the Phase 1 cleanup gate and recorded the decision in
  [ops/evidence/gate-001-phase-1-checkpoint-2026-04-08.md](/home/r/coding/myosu/ops/evidence/gate-001-phase-1-checkpoint-2026-04-08.md):
  Phase 1 is complete and Phase 2 can begin. The gate surfaced two truthful
  fixes needed to finish verification on this machine: the repo-owned E2E shell
  proofs now resolve binaries from `${CARGO_TARGET_DIR:-$repo_root/target}`
  instead of assuming a repo-local `target/`, and an unnecessary dead-code
  warning in [crates/myosu-chain/pallets/utility/src/tests.rs](/home/r/coding/myosu/crates/myosu-chain/pallets/utility/src/tests.rs)
  was removed so the workspace suite stays warning-clean. The decision artifact
  also records the current spec/code conflict: the task's authoritative spec
  still talks about the old `pallet_subtensor` alias, while the live runtime and
  queue acceptance criteria correctly use `pallet_game_solver` / `GameSolver`.
  Validation: `test ! -d crates/myosu-chain/pallets/subtensor`; `ls crates/myosu-chain/pallets/game-solver/src/migrations/migrate_*.rs | wc -l`; `SKIP_WASM_BUILD=1 cargo check --workspace`; `SKIP_WASM_BUILD=1 cargo test --workspace --quiet`; `cargo test -p pallet-game-solver --quiet -- stage_0`; `SKIP_WASM_BUILD=1 cargo test -p myosu-chain --test stage0_local_loop --quiet`; `bash tests/e2e/local_loop.sh`; `bash tests/e2e/emission_flow.sh`; `bash .github/scripts/check_doctrine_integrity.sh`.
  Commit: `6e3b88f`

- `DEBT-005` Cleaned the stale root documentation surfaces so they no longer
  present the nonexistent Fabro/Raspberry control plane as current repo truth:
  [AGENTS.md](/home/r/coding/myosu/AGENTS.md), [OS.md](/home/r/coding/myosu/OS.md),
  and [README.md](/home/r/coding/myosu/README.md) now describe Fabro/Raspberry
  as planned/not implemented and point readers at direct proof commands
  instead. Moved `THEORY.MD` from repo root to `archive/THEORY.MD`. Also
  removed the completed task from `IMPLEMENTATION_PLAN.md`; the task's own
  claim that the plan still referenced `050426-*` was stale, because the live
  plan already pointed at `gen-20260408-013810/specs/080426-*.md`. A broader
  regression check exposed stale `genesis/plans/*.md` filenames in
  `.github/scripts/check_stage0_repo_shape.sh`, so this increment updated that
  script to the live plan stack and documented the coupling in `AGENTS.md`.
  Validation: `rg -n "planned, not implemented|planned, not yet implemented" AGENTS.md OS.md README.md`; `! grep -q "fabro run" README.md`; `test ! -f THEORY.MD`; `test -f archive/THEORY.MD`; `bash .github/scripts/check_doctrine_integrity.sh`; `bash .github/scripts/check_stage0_repo_shape.sh`.
  Commit: `PENDING`

- `DEBT-004` Deleted 55 unreferenced subtensor-era migration files from
  `crates/myosu-chain/pallets/game-solver/src/migrations/`, leaving only the
  live runtime migration `migrate_init_total_issuance.rs` and the still-used
  `migrate_create_root_network.rs` helper that remains referenced from
  default-build code/tests. Added one-line retention comments to both kept
  files.
  Validation: `ls crates/myosu-chain/pallets/game-solver/src/migrations/migrate_*.rs | wc -l`; `SKIP_WASM_BUILD=1 cargo check --workspace`; `cargo test -p pallet-game-solver --quiet -- stage_0`; `SKIP_WASM_BUILD=1 cargo test -p myosu-chain --features fast-runtime,try-runtime --quiet devnet_runtime_upgrade_smoke_test_passes_on_fresh_genesis`.
  Commit: `db7b5c6`

- `DEBT-001` Relocated the live RPC and runtime-api crates from
  `pallets/subtensor/` to `pallets/game-solver/`, rewired workspace/runtime
  path dependencies, and updated the chain support tool's runtime-api path.
  Validation: `SKIP_WASM_BUILD=1 cargo check --workspace`; `cargo test -p pallet-game-solver --quiet -- stage_0`.
  Commit: `2066e0e`

- `DEBT-002` Deleted the dead `crates/myosu-chain/pallets/subtensor/` tree,
  removed the root workspace dependency on `pallet-subtensor`, and dropped the
  old pallet path from the chain support version-bump tool.
  Validation: `test ! -d crates/myosu-chain/pallets/subtensor`; `SKIP_WASM_BUILD=1 cargo check --workspace`; `cargo test -p pallet-game-solver --quiet -- stage_0`; `SKIP_WASM_BUILD=1 cargo test -p myosu-chain --test stage0_local_loop --quiet`; `if grep -rq 'pallets/subtensor' crates/; then exit 1; fi`.
  Commit: `75414e5`

- `DEBT-003` Renamed the live runtime pallet surface from
  `pallet_subtensor` / `SubtensorModule` to `pallet_game_solver` / `GameSolver`,
  updated the chain client's hard-coded storage prefix and call/event variants,
  renamed the relocated RPC crates to `game-solver-rpc*`, and fixed local/devnet
  genesis patch builders to emit `gameSolver` instead of the stale
  `subtensorModule` key. The truthful alias proof is an exact-word search because
  the repo still intentionally contains distinct pallets such as
  `pallet_subtensor_proxy`, `pallet_subtensor_swap`, and `pallet_subtensor_utility`.
  Validation: `rg -n '\\bpallet_subtensor\\b|\\bSubtensorModule\\b|subtensorModule' crates/myosu-chain crates/myosu-chain-client Cargo.toml`; `SKIP_WASM_BUILD=1 cargo check -p myosu-chain-runtime -p myosu-chain`; `cargo test -p pallet-game-solver --quiet -- stage_0`; `cargo clean -p myosu-chain-runtime && SKIP_WASM_BUILD=1 cargo build -p myosu-chain-runtime --quiet`; `SKIP_WASM_BUILD=1 cargo run -p myosu-chain --quiet -- build-spec --chain devnet --raw > /dev/null`.
  Commit: `b59351c`

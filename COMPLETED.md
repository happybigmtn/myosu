# COMPLETED

- `PROMO-002` Added
  [promotion_manifest.sh](/home/r/Coding/myosu/tests/e2e/promotion_manifest.sh)
  as the repo-owned E2E gate for the solver promotion ledger. The harness runs
  the canonical promotion manifest example against
  [ops/solver_promotion.yaml](/home/r/Coding/myosu/ops/solver_promotion.yaml),
  asserts exactly 22 promotion rows, rejects any emitted tier above live code
  support, and requires `nlhe-heads-up` plus `liars-dice` to stay at
  `benchmarked` or above. The harness also honors
  `MYOSU_SOLVER_PROMOTION_LEDGER`, so negative ledger fixtures can prove that
  manual over-promotion fails closed without editing the checked-in YAML.
  While running the broad local gates, the repo-shape check exposed stale
  hard-coded `genesis/plans/` filenames in
  [.github/scripts/check_stage0_repo_shape.sh](/home/r/Coding/myosu/.github/scripts/check_stage0_repo_shape.sh);
  that script now checks the current 001-011 promotion plan stack.
  Removed `PROMO-002` and its promotion-infrastructure checkpoint from
  [IMPLEMENTATION_PLAN.md](/home/r/Coding/myosu/IMPLEMENTATION_PLAN.md).
  Validation: `bash tests/e2e/promotion_manifest.sh`; manual negative test with
  a temp ledger changing `cribbage` to `tier: promotable_local`; `bash -n tests/e2e/promotion_manifest.sh`; `SKIP_WASM_BUILD=1 cargo test -p myosu-games-canonical --quiet`; `bash tests/e2e/canonical_ten_play_harness.sh`; `bash .github/scripts/check_plan_quality.sh`; `bash .github/scripts/check_doctrine_integrity.sh`; `bash .github/scripts/check_stage0_repo_shape.sh`; `SKIP_WASM_BUILD=1 cargo test --workspace --quiet`.
  Commit: `PENDING`

- `PROMO-001` Added the solver promotion ledger in
  [ops/solver_promotion.yaml](/home/r/Coding/myosu/ops/solver_promotion.yaml)
  with exactly 22 `ResearchGame` entries. NLHE heads-up and Liar's Dice start
  at `benchmarked`; every portfolio-routed research game starts at `routed`.
  The canonical policy module now parses and validates that ledger against the
  live `ALL_RESEARCH_GAMES` inventory, rejects unknown slugs and unsupported
  tier claims, and emits joined manifest rows with code-reported bundle support.
  Added
  [promotion_manifest.rs](/home/r/Coding/myosu/crates/myosu-games-canonical/examples/promotion_manifest.rs)
  with table and JSON output modes, plus unit and integration coverage in
  [policy.rs](/home/r/Coding/myosu/crates/myosu-games-canonical/src/policy.rs)
  and
  [promotion_manifest.rs](/home/r/Coding/myosu/crates/myosu-games-canonical/tests/promotion_manifest.rs).
  Removed `PROMO-001` from
  [IMPLEMENTATION_PLAN.md](/home/r/Coding/myosu/IMPLEMENTATION_PLAN.md).
  Validation: `SKIP_WASM_BUILD=1 cargo test -p myosu-games-canonical --quiet`; `SKIP_WASM_BUILD=1 cargo run -p myosu-games-canonical --example promotion_manifest --quiet`; `test -f ops/solver_promotion.yaml && echo EXISTS`; `grep -c 'tier:' ops/solver_promotion.yaml`; `SKIP_WASM_BUILD=1 cargo clippy -p myosu-games-canonical -- -D warnings`; `SKIP_WASM_BUILD=1 cargo check -p myosu-games -p myosu-games-kuhn -p myosu-games-poker -p myosu-games-liars-dice -p myosu-games-portfolio -p myosu-games-canonical -p myosu-tui -p myosu-play -p myosu-chain-client -p myosu-miner -p myosu-validator`; `cargo fmt --check`; `bash .github/scripts/check_doctrine_integrity.sh`; `bash .github/scripts/check_plan_quality.sh`; `SKIP_WASM_BUILD=1 cargo test --workspace --quiet`.
  Commit: `0c41057e1e`

- `POLICY-001` Added the canonical policy bundle surface in
  [policy.rs](/home/r/Coding/myosu/crates/myosu-games-canonical/src/policy.rs):
  promotion tiers, policy distribution/provenance/benchmark/bundle/proof types,
  deterministic sorted-JSON SHA-256 bundle hashing, bundle verification, and
  deterministic PPM-weighted sampling. The module is re-exported from
  [lib.rs](/home/r/Coding/myosu/crates/myosu-games-canonical/src/lib.rs), and
  the canonical crate now declares the direct `sha2`/`hex` hashing dependencies.
  During the broad workspace gate, fixed the unrelated devnet chain-spec
  regression where genesis patches used `gameSolver` even though the current
  default-build runtime still deserializes that inherited genesis field as
  `subtensorModule`.
  Removed `POLICY-001` from
  [IMPLEMENTATION_PLAN.md](/home/r/Coding/myosu/IMPLEMENTATION_PLAN.md).
  Validation: `test -f crates/myosu-games-canonical/src/policy.rs && echo EXISTS`; `SKIP_WASM_BUILD=1 cargo test -p myosu-games-canonical --quiet`; `SKIP_WASM_BUILD=1 cargo clippy -p myosu-games-canonical -- -D warnings`.
  Commit: `PENDING`

- `F-007` Re-verified the remaining miner-quality work as blocked instead of
  leaving it falsely runnable in
  [IMPLEMENTATION_PLAN.md](/home/r/coding/myosu/IMPLEMENTATION_PLAN.md). The
  live proof surfaces still split by game: Liar's Dice has a truthful
  exploitability ladder and `512`-iteration recommendation, while poker still
  depends on an operator-supplied full encoder from robopoker/PostgreSQL and
  cannot honestly publish a minimum from the checked-in sparse bootstrap
  artifacts. This increment converts `F-007` to `- [!]` so the active queue no
  longer implies the poker side can be closed from repo-local evidence alone.
  Validation: `SKIP_WASM_BUILD=1 cargo test -p myosu-validator --quiet -- quality_benchmark`; `cargo test -p myosu-games-poker --quiet benchmark_reports_sparse_encoder_failure_cleanly`; `bash .github/scripts/check_plan_quality.sh`.
  Commit: `PENDING`

- `RES-003` Closed the poker-only quality-benchmark follow-up by adding a
  repo-owned full-encoder path instead of pretending the sparse bootstrap
  artifacts are sufficient. The poker crate now has
  [import_robopoker_lookup.rs](/home/r/coding/myosu/crates/myosu-games-poker/examples/import_robopoker_lookup.rs),
  [quality_benchmark.rs](/home/r/coding/myosu/crates/myosu-games-poker/examples/quality_benchmark.rs),
  and supporting library surfaces in
  [benchmark.rs](/home/r/coding/myosu/crates/myosu-games-poker/src/benchmark.rs)
  and
  [lookup_dump.rs](/home/r/coding/myosu/crates/myosu-games-poker/src/lookup_dump.rs).
  The local artifact loader in
  [artifacts.rs](/home/r/coding/myosu/crates/myosu-games-poker/src/artifacts.rs)
  now leaves room for multi-gigabyte full encoder files instead of capping them
  at `256 MiB`, and the operator flow is documented in
  [poker-quality-benchmark.md](/home/r/coding/myosu/docs/execution-playbooks/poker-quality-benchmark.md)
  plus
  [ops/poker_quality_benchmark.sh](/home/r/coding/myosu/ops/poker_quality_benchmark.sh).
  Removed `RES-003` from
  [IMPLEMENTATION_PLAN.md](/home/r/coding/myosu/IMPLEMENTATION_PLAN.md) and
  narrowed `WORKLIST.md` `MINER-QUAL-001` to the remaining task of recording a
  real full-encoder exploitability ladder and choosing a poker minimum
  iteration floor.
  Validation: `cargo test -p myosu-games-poker --quiet`; `bash -n ops/poker_quality_benchmark.sh`; `bash .github/scripts/check_doctrine_integrity.sh`.
  Commit: `25ffc41` (implementation commit)

- `RES-002` Classified the 21 fork-only `opentensor/polkadot-sdk` commits in
  [ADR 009](/home/r/coding/myosu/docs/adr/009-polkadot-sdk-migration-feasibility.md)
  against `upstream/stable2506`, refreshed the audit counts from the live fork
  checkout (`21` fork-only / `49` upstream-only as of 2026-04-08), and turned
  the old feasibility note into a commit-level migration map: `10` patches are
  currently needed by myosu, `8` are safe-drop candidates, and `3` remain
  uncertain. The increment also resolved
  `WORKLIST.md` `CHAIN-SDK-001`, recorded the classification outcome in
  [ops/decision_log.md](/home/r/coding/myosu/ops/decision_log.md), and made
  the active queue truthful by marking the externally blocked token-economics
  review tasks `RES-001` and `F-003` as blocked in
  [IMPLEMENTATION_PLAN.md](/home/r/coding/myosu/IMPLEMENTATION_PLAN.md) instead
  of leaving them as runnable.
  Validation: `git -C /tmp/polkadot-sdk-audit rev-list --left-right --count 71629fd93b6c12a362a5cfb6331accef9b2b2b61...upstream/stable2506`; `base=$(git -C /tmp/polkadot-sdk-audit merge-base 71629fd93b6c12a362a5cfb6331accef9b2b2b61 upstream/stable2506) && git -C /tmp/polkadot-sdk-audit log --reverse --format='%H%x09%s' ${base}..71629fd93b6c12a362a5cfb6331accef9b2b2b61`; `rg -n "DispatchGuard =|copy_keys\\(|raw_public_keys|key_phrase_by_type|initial_consensus|HardForks::new_initial_set_id|skip_block_justifications|SingleState" crates/myosu-chain -g '!target'`; `bash .github/scripts/check_plan_quality.sh`; `bash .github/scripts/check_doctrine_integrity.sh`.
  Commit: `5587a9a`

- `OPS-003` Added a repo-owned container packaging path for the stage-0
  operator workflow with a multi-stage
  [Dockerfile](/home/r/coding/myosu/Dockerfile), a single-host
  [docker-compose.yml](/home/r/coding/myosu/docker-compose.yml), and runtime
  entrypoint helpers under
  [ops/docker/](/home/r/coding/myosu/ops/docker/). The compose proof now
  boots the built-in `devnet` chain spec inside Docker, starts an authority
  node with the named `//myosu//devnet//authority-1` key, runs a bounded miner
  bootstrap that writes the shared checkpoint and stays up as the live HTTP
  axon, and exits from the validator container only after enabling subnet
  staking, scoring the miner response, and submitting weights on subnet `7`.
  The operator docs in
  [docs/operator-guide/quickstart.md](/home/r/coding/myosu/docs/operator-guide/quickstart.md)
  and the operational notes in
  [AGENTS.md](/home/r/coding/myosu/AGENTS.md) now point at that compose-based
  proof surface instead of source-only setup.
  Validation: `docker build --target chain-runtime -t myosu-chain .`; `docker build --target miner-runtime -t myosu-miner .`; `docker build --target validator-runtime -t myosu-validator .`; `docker compose up --build --abort-on-container-exit --exit-code-from validator`; `docker image inspect myosu-chain myosu-miner myosu-validator --format '{{.RepoTags}} {{.Size}}'`; `docker compose down -v`; `bash .github/scripts/check_doctrine_integrity.sh`.
  Implementation commit: `4fff63d`

- `OPS-001` Reworked
  [README.md](/home/r/coding/myosu/README.md) into a truthful onboarding
  surface for the current stage-0 repo: it now lists the missing prerequisites
  called out by the plan (stable Rust with edition 2024 support, the required
  WASM targets, and `protoc`), documents
  `cargo test -p myosu-games-kuhn --quiet` as the fastest meaningful quick
  verify, separates the developer proof path from the operator path, and
  points operator onboarding at the maintained
  [docs/operator-guide/quickstart.md](/home/r/coding/myosu/docs/operator-guide/quickstart.md)
  instead of duplicating a long command wall in the landing page. Removed the
  task from
  [IMPLEMENTATION_PLAN.md](/home/r/coding/myosu/IMPLEMENTATION_PLAN.md).
  Validation: `! grep -q "fabro run" README.md && grep -q "Prerequisites" README.md && grep -q "myosu-games-kuhn" README.md`; `cargo test -p myosu-games-kuhn --quiet`; `SKIP_WASM_BUILD=1 cargo run -p myosu-play --quiet -- --smoke-test`; `printf 'quit\n' | SKIP_WASM_BUILD=1 cargo run -p myosu-play --quiet -- pipe`.
  Commit: `28f4041`

- `GATE-003` Verified the Phase 2 hardening gate against the live repo state,
  recorded the decision in
  [ops/evidence/gate-003-phase-2-checkpoint-2026-04-08.md](/home/r/coding/myosu/ops/evidence/gate-003-phase-2-checkpoint-2026-04-08.md),
  and removed the task from
  [IMPLEMENTATION_PLAN.md](/home/r/coding/myosu/IMPLEMENTATION_PLAN.md).
  The gate surfaced one real blocker instead of closing cleanly on the first
  pass: the repo-owned
  [tests/e2e/cross_node_emission.sh](/home/r/coding/myosu/tests/e2e/cross_node_emission.sh)
  proof was still reading runtime storage through the stale `SubtensorModule`
  prefix after the live pallet rename to `GameSolver`, which made the
  comparison snapshot read zero emissions and fail its own non-zero assertion.
  This increment updated that E2E driver to the live storage prefix, reran the
  proof green, and then closed the gate with the full workspace suite plus all
  seven repo-owned E2E scripts passing. `WORKLIST.md` already resolves
  `EM-DUST-001` and truthfully narrows `MINER-QUAL-001` to the remaining
  poker-only follow-on, so Phase 3 packaging can proceed without bluffing that
  poker quality measurement is solved.
  Validation: `test -f docs/adr/011-emission-dust-policy.md`; `SKIP_WASM_BUILD=1 cargo test --workspace --quiet`; `bash tests/e2e/local_loop.sh`; `bash tests/e2e/two_node_sync.sh`; `bash tests/e2e/four_node_finality.sh`; `bash tests/e2e/consensus_resilience.sh`; `bash tests/e2e/cross_node_emission.sh`; `bash tests/e2e/validator_determinism.sh`; `bash tests/e2e/emission_flow.sh`; `bash .github/scripts/check_stage0_repo_shape.sh`; `bash .github/scripts/check_doctrine_integrity.sh`; `bash .github/scripts/check_plan_quality.sh`.
  Commit: `24e7247`

- `BENCH-001` Added a truthful Liar's Dice quality benchmark in
  [crates/myosu-validator/src/validation.rs](/home/r/coding/myosu/crates/myosu-validator/src/validation.rs)
  that bypasses the validator same-checkpoint path and measures the solver's
  exact exploitability directly at 0, 128, 256, and 512 iterations. The
  benchmark proves exploitability decreases materially across that ladder and
  supports the current operator recommendation in
  [docs/operator-guide/quickstart.md](/home/r/coding/myosu/docs/operator-guide/quickstart.md)
  to treat `512` iterations as the minimum meaningful Liar's Dice training
  floor. Removed `BENCH-001` from
  [IMPLEMENTATION_PLAN.md](/home/r/coding/myosu/IMPLEMENTATION_PLAN.md) and
  narrowed `WORKLIST.md` `MINER-QUAL-001` to the remaining poker-only blocker
  around sparse encoder artifacts.
  Validation: `SKIP_WASM_BUILD=1 cargo test -p myosu-validator --quiet -- quality_benchmark`; `SKIP_WASM_BUILD=1 cargo test -p myosu-validator --quiet`; `SKIP_WASM_BUILD=1 cargo test -p myosu-games-liars-dice --quiet`; `bash .github/scripts/check_doctrine_integrity.sh`.
  Commit: `c8be68f`

- `GATE-002` Verified the Phase 2 hardening checkpoint against the live repo
  state and removed the task from
  [IMPLEMENTATION_PLAN.md](/home/r/coding/myosu/IMPLEMENTATION_PLAN.md).
  The current codebase already satisfied the gate without further code changes:
  `WORKLIST.md` resolves `EM-DUST-001` via
  [ADR 011](/home/r/coding/myosu/docs/adr/011-emission-dust-policy.md), the
  miner HTTP axon security tests remain present in
  [crates/myosu-miner/src/axon.rs](/home/r/coding/myosu/crates/myosu-miner/src/axon.rs),
  and the broad workspace plus stage-0 E2E proofs are green on this checkout.
  Validation: `SKIP_WASM_BUILD=1 cargo test -p myosu-miner --quiet axon`;
  `SKIP_WASM_BUILD=1 cargo test --workspace --quiet`;
  `bash tests/e2e/emission_flow.sh`; `bash tests/e2e/local_loop.sh`.
  Commit: `PENDING`

- `TEST-003` Added cross-game scoring documentation coverage in
  [crates/myosu-validator/src/validation.rs](/home/r/coding/myosu/crates/myosu-validator/src/validation.rs)
  by exercising the live validator scoring path against the same response
  degradation pattern in both poker and Liar's Dice: each sampled strategy is
  collapsed to one weak legal action, then scored through the existing
  `score_from_l1_distance()` flow. The new `cross_game_*` test records the
  current stage-0 conclusion in code comments: for this sampled one-hot
  degradation pattern, both games stay in the same rough score band, which is
  encouraging for stage-0 fairness but not a proof of universal cross-subnet
  fairness across all game configs or exploitability units. Removed
  `TEST-003` from
  [IMPLEMENTATION_PLAN.md](/home/r/coding/myosu/IMPLEMENTATION_PLAN.md).
  Validation: `SKIP_WASM_BUILD=1 cargo test -p myosu-validator --quiet -- cross_game`; `SKIP_WASM_BUILD=1 cargo test -p myosu-validator --quiet`.
  Commit: `8e47ebe`

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
  Commit: `7026ad1`

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

- `OPS-002` Removed the tracked ghost bootstrap control-plane surface instead
  of inventing a fake execution entrypoint. Deleted
  [fabro.toml](/home/r/coding/myosu/fabro.toml), rewrote
  [AGENTS.md](/home/r/coding/myosu/AGENTS.md) and
  [OS.md](/home/r/coding/myosu/OS.md) so the repo-owned control plane is now
  only checked-in doctrine plus executable proof commands, and cleaned the last
  active operator-facing references in
  [docs/execution-playbooks/README.md](/home/r/coding/myosu/docs/execution-playbooks/README.md)
  and [specs/README.md](/home/r/coding/myosu/specs/README.md). The live repo
  may still contain ignored local `.raspberry/` state on a developer machine,
  but it is not tracked and no longer appears in active doctrine.
  Validation: `! rg -n "Fabro|fabro|Raspberry|raspberry" AGENTS.md OS.md README.md docs/execution-playbooks/README.md specs/README.md`; `test ! -f fabro.toml`; `bash .github/scripts/check_doctrine_integrity.sh`; `bash .github/scripts/check_stage0_repo_shape.sh`.
  Commit: `PENDING`

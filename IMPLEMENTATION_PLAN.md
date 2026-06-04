# IMPLEMENTATION_PLAN

## CEO Testnet Roadmap — Immediate P0 (signed 2026-06-03)

North star: A named multi-node Myosu testnet where miners submit strategy, two validators score it identically, emissions flow by measured quality, and a human/agent reads a solved hand through the same gameplay surface against that live chain.
Full roadmap: genesis/plans/000-ceo-testnet-roadmap.md

- [x] [P0] Replace the testnet chain-spec stub: make `finney_testnet_config` in `crates/myosu-chain/node/src/chain_spec/testnet.rs` call the full game-solver genesis builder (factor the devnet provisioning in `devnet.rs:95-175` into a shared `genesis_with_game_solver(...)` and invoke it for testnet with testnet authority/owner/operator URIs) so the testnet spec bootstraps subnet 7, a subnet owner, and staking pools.

  Resolution: factored the devnet provisioning into a shared `genesis_with_game_solver` builder in the new `crates/myosu-chain/node/src/chain_spec/game_solver_spec.rs` (`GameSolverGenesisSpec` carries the chain-specific authority/owner/operator URIs, the historic devnet economic constants, and a `with_default_economics(...)` constructor that lets a caller override only the chain-specific fields). `devnet::devnet_config` and `testnet::finney_testnet_config` are now both one-liner delegates to the shared builder. The testnet path passes testnet URIs (`//myosu//testnet//authority-{1,2,3}`, `//myosu//testnet//{miner-1,validator-1,validator-2,orchestrator}`, `//myosu//testnet//subnet-owner[//hotkey]`) and stays `ChainType::Local` until a persistent testnet entrypoint exists. Proof: `cargo test -p myosu-chain --lib chain_spec` (6/6 pass, including the new `finney_testnet_config_bootstraps_subnet_seven_in_storage` and `build_spec_testnet_raw_contains_subnet_seven_keys_and_owner`); `cargo test -p myosu-chain --test testnet_build_spec` (2/2 pass, exercises the literal `build-spec --chain test_finney --raw` CLI and confirms the raw storage map is non-empty with well-formed hex keys); `cargo test -p myosu-chain --tests` (30/30 pass total); `SKIP_WASM_BUILD=1 cargo build --workspace` exits 0. The two remaining P0 rows (testnet entrypoint, live-read proof) are now unblocked and remain open.
- [x] [P0] Add a `build-spec --chain testnet --raw` proof test under `crates/myosu-chain/node/tests/` asserting the raw genesis JSON contains subnet 7, a non-empty subnet owner, and `SubnetworkN(7)`-backing storage keys (fail-closed if the game-solver patch is absent).

  Resolution: `crates/myosu-chain/node/tests/testnet_build_spec.rs` exercises the literal `build-spec` CLI the operator would run. Two tests: `build_spec_testnet_raw_carries_game_solver_bootstrap_storage` (invokes the compiled `myosu-chain` binary with `--chain test_finney --raw`, asserts the raw storage map is non-empty, well-formed hex, and ≥100 keys — the game-solver bootstrap writes a few dozen storage keys, so a regression that dropped the bootstrap step would shrink the map below the threshold) and `build_spec_testnet_round_trip_via_file_reader_is_consistent` (round-trips the spec through the file reader path and asserts name=`Myosu Testnet`, id=`myosu-testnet`, chainType=`Local`, protocolId=`myosu-testnet` — the regression guard for spec symmetry across `build-spec` invocations). The in-process proof in `chain_spec::testnet::tests::build_spec_testnet_raw_contains_subnet_seven_keys_and_owner` continues to cover the patch+raw shape assertions in a single test process. Both pass: `cargo test -p myosu-chain --test testnet_build_spec` → 2/2.
- [x] [P0] Extend the stage0 multi-node compose path to a second validator: add a `validator-2` service using `//myosu//devnet//validator-2` (already endowed in devnet genesis) and assert both validators' submitted weights for `miner-1` agree within INV-003 epsilon in the compose proof's exit check.

  Resolution: docker-compose.yml gains a `validator-2` service keyed to `//myosu//devnet//validator-2` (mirrors `validator`; shares the same `validator-runtime` build target and the same chain/miner endpoints; keyed to the second endowed operator URI from `devnet.rs`'s `DEVNET_OPERATOR_URIS`). A new `devnet_operator_set_endows_both_validators_with_distinct_accounts` unit test in `crates/myosu-chain/node/src/chain_spec/devnet.rs` guards the operator-set shape (both validator URIs resolve to distinct sr25519 accounts, both are distinct from the miner-1 hotkey, and the System account store has a non-zero free balance for validator-1/validator-2/miner-1/subnet-owner — catching any refactor that drops validator-2 from the endowed operator set, which would leave the second compose service with nothing to register against). The new `myosu_chain_client::evaluate_validator_agreement` helper plus `target_weight_in_row` lookup (in `crates/myosu-chain-client/src/lib.rs`) implement the fail-closed agreement check: it takes two on-chain `Weights` rows + a target miner UID + an INV-003 epsilon, returns a `ValidatorAgreementReport` on pass and a typed `ValidatorAgreementError` (`InvalidEpsilon`, `MissingTargetWeight`, `WeightsDiverge`) on fail. The on-chain row is a `Vec<(u16, u16)>`, so agreement collapses to integer equality (u16 weights cannot be non-finite; the score-domain epsilon is floored to 0 weight units). The new `compose_proof_driver` example (in `crates/myosu-chain-client/examples/`) connects to a running node, resolves the miner + validator UIDs, reads the two `Weights` rows via `ChainClient::get_weights_for_uid`, runs the shared `evaluate_validator_agreement` helper, and emits key/value lines (`miner_uid`, `validator_a_uid`, `validator_b_uid`, `validator_a_target_weight`, `validator_b_target_weight`, `agreement_within_epsilon`, …) that the bash proof consumes. Two new bash proofs: `tests/e2e/stage0_multi_validator_compose.sh` runs the same topology as a process tree (no docker required) and asserts both validators print `WEIGHTS myosu-validator submission ok`, then `compose_proof_driver` confirms `validator_a_target_weight == validator_b_target_weight` for `miner-1` on subnet 7; `tests/e2e/compose_proof.sh` boots the actual `docker-compose.yml` stack (`chain + miner + validator + validator-2`) and runs the same fail-closed exit check against the live on-chain rows. Both proofs exit non-zero on `WeightsDiverge`, `MissingTargetWeight`, or a missing `WEIGHTS … submission ok` line. Proof: `cargo test -p myosu-chain-client --lib` (24/24 pass, including 7 new `evaluate_validator_agreement_*` and `target_weight_in_row` cases); `cargo test -p myosu-chain --lib chain_spec` (7/7 pass, including the new `devnet_operator_set_endows_both_validators_with_distinct_accounts`); `SKIP_WASM_BUILD=1 cargo build --workspace --tests` exits 0.
- [x] [P0] Add a testnet operator entrypoint + healthcheck: a `chain-testnet-entrypoint.sh` and a compose profile (or `docker-compose.testnet.yml`) that boots the node from the testnet spec, registers the subnet owner, and exposes a persistent CORS-enabled WS/HTTP RPC endpoint that survives the proof run (not exit-on-validator).

  Resolution: shipped the persistent testnet entrypoint + healthcheck as four committed artifacts. `ops/docker/chain-testnet-entrypoint.sh` is the long-running `myosu-chain` entrypoint for the `test_finney` spec (validator + force-authoring + `--rpc-external --rpc-cors all --rpc-methods unsafe` + prometheus + p2p; the `enable-subtoken` extrinsic is intentionally NOT invoked from the chain's loop because it is a one-shot transaction and would either block forever waiting for its own RPC or risk a self-referential race). `ops/docker/chain-testnet-subnet-owner-init.sh` is the one-shot subnet-owner registration entrypoint that waits for the chain RPC via the shared `wait-for-rpc.sh` helper, then runs `myosu-validator --chain ws://chain:9944 --subnet 7 --key //myosu//testnet//subnet-owner --enable-subtoken`, and exits. `docker-compose.testnet.yml` wires the two together (chain service with `system_health` healthcheck + `testnet-chain-data` named volume; `subnet-owner-init` sidecar that `depends_on: chain: service_healthy` and runs the registration once and exits) and exposes the CORS-enabled RPC on `localhost:9944`. `Dockerfile` copies the two new entrypoints into both the `chain-runtime` and `validator-runtime` stages and `chmod +x`s them. `tests/e2e/testnet_entrypoint_compose.sh` is the CI-grade proof: it mirrors the compose topology as a process tree (no docker required), resolves the on-chain `SubnetOwner(7)` storage key from the raw testnet spec (filters the 24 other `0700`-suffixed keys whose values are u16/u64, not 32-byte AccountId), boots the chain from `test_finney` with the same flag set the compose file uses, waits for `chain_getHeader` to return parseable blocks (the reliable healthcheck probe for a single-authority fresh chain where `isSyncing` can briefly stay true during genesis), waits for block 3, runs the validator subnet-owner init, asserts both `VALIDATOR myosu-validator bootstrap ok` and `SUBTOKEN myosu-validator subnet ok` log lines, asserts the chain process is STILL running (the "not exit-on-validator" P0 invariant), re-probes RPC health, asserts the on-chain `SubnetOwner(7)` storage value is a non-zero 32-byte AccountId, and asserts the chain has continued authoring past the target block. Proof: `bash tests/e2e/testnet_entrypoint_compose.sh` → exit 0; `cargo test -p myosu-chain --lib chain_spec` → 7/7; `cargo test -p myosu-chain --test testnet_build_spec` → 3/3. The chain's persistent volume (`testnet-chain-data` in compose; the worktree under `target/e2e/testnet-entrypoint-compose.*/chain` in the bash proof) survives restarts so subsequent `docker compose up` runs continue the same chain.


- [x] [P0] Add a live-read proof: a `myosu-play --chain-endpoint ws://... --read-solved` mode (or a `myosu-chain-client` example) that connects to a running node, discovers the miner axon, plays one poker hand, and prints `bundle_hash`, `miner_uid`, and `emission` — the externally-verifiable "read a solved result through the same surface" milestone.

  Resolution: shipped the live-read proof surface end-to-end as a single coherent change. The new `crates/myosu-play/src/live_read.rs` module (486 lines, 6 unit tests) implements the proof step-by-step: `ChainClient::connect(...)` → `discover_any_chain_visible_miner(endpoint, subnet)` (a permissive variant of the existing `discover_best_chain_visible_miner` that accepts zero-incentive candidates as long as they have a published axon, since the `Incentive` vector only updates at the end of an epoch) → render the demo NLHE renderer, POST a `/strategy` request to the discovered miner's HTTP endpoint, decode the wire response, SHA-256-hash the canonical `(edge, probability)` pairs into a deterministic `bundle_hash` (sort by `format!("{edge:?}")` + `f32::to_bits`, render as `"{edge:?}|{bits:08x}\n"`), then read the on-chain `miner_uid = Uids(7, hotkey)` + `emission = Emission(7)[uid]` and return a structured `LiveReadReport` rendered as plain key/value lines (`status=solved`, `chain_endpoint=ws://...`, `subnet=7`, `discovered_miner_uid=...`, `discovered_miner_incentive=...`, `discovered_miner_hotkey=...`, `discovered_miner_endpoint=...`, `live_miner_action_count=...`, `live_miner_recommended_edge=...`, `live_miner_recommended_action=...`, `miner_uid=...`, `emission=...`, `bundle_hash=<64-hex>`). Failure paths (`NoMiner`, `QueryFailed`, `ChainReadFailed`) print `status=<reason>` + `failure_detail=...` and exit non-zero so a wrapper script can `grep ^status=`. CLI: `myosu-play --read-solved` (the literal wording from the P0 row) plus the ergonomic `myosu-play live-read --chain-endpoint ws://... --subnet 7` subcommand; the subcommand's `--chain-endpoint` flag is the literal alias the P0 row calls out. `crates/myosu-play/src/cli.rs` gains the new `LiveReadArgs` and `Mode::LiveRead` variants (with two CLI tests for both invocation shapes). `crates/myosu-play/src/main.rs` dispatches `--read-solved` to `run_read_solved` (which builds a `LiveReadArgs` from the top-level `--chain` + `--subnet` flags) and `live-read` to `run_live_read` (the subcommand path). `crates/myosu-play/src/discovery.rs` gains `discover_any_chain_visible_miner` + `select_any_candidate_with_endpoint` (3 new unit tests: zero-incentive-miner-with-axon is accepted, nonzero-incentive still ranks first when available, miners-without-axon are still skipped). `crates/myosu-chain-client/src/lib.rs` gains `ChainClient::get_chain_visible_miner_axons` — the same shape as `get_chain_visible_miners` but without the strict non-zero-incentive filter, since the live-read milestone is about reading a solved result from the miner axon, not about the chain having scored it (incentive 0 is a normal fresh-chain state). `myosu-play` Cargo.toml gains `sha2` (for the bundle hash) and the two `rbp-gameplay` / `rbp-nlhe` dev-deps (only used in the `live_read` test module to construct sample `NlheEdge::from(Edge::Fold/Call)` values). `crates/myosu-play/src/live.rs` re-exports `connect_endpoint` as `connect_endpoint_for_chain` so the live-read module can reuse the live-query endpoint normalizer without a private re-export. The new `tests/e2e/live_read.sh` (494 lines) is the CI-grade end-to-end proof: it mirrors the docker-compose devnet topology as a process tree (chain + bootstrap_artifacts + miner `--register --serve-axon` + miner `--serve-http` + `myosu-play --read-solved`), waits for block 2, asserts the miner HTTP `/health` is `{"status":"ok",...}`, runs `myosu-play --read-solved --chain ws://... --subnet 7`, and fail-closed asserts every required key (status=solved, subnet=7, chain_endpoint echo, miner_uid is a u16, bundle_hash is 64 lowercase hex, emission is a non-negative u64, discovered_miner_uid agrees with miner_uid via the cross-storage-key read, discovered_miner_endpoint ends with the bound port, discovered_miner_hotkey is a plausible SS58 string ≥40 chars, discovered_miner_incentive is a u16, the chain has continued authoring past the target block, and the miner HTTP `/health` is STILL ok after the proof). Proof: `bash tests/e2e/live_read.sh` → exit 0 (live chain at block 4, miner_uid=1, discovered_miner_uid=1, discovered_miner_endpoint=0.0.0.0:8091, bundle_hash=ff94fe93ac0777f3fb7a89b1c754b81205eab7682608304dad3f972d9fa7a1c9, emission=0); `cargo test -p myosu-play --bin myosu-play` → 62/62 (6 new live_read tests + 3 new any-discovery tests + 2 new CLI tests for the two flag shapes); `cargo test -p myosu-chain-client --lib` → 24/24; `cargo test -p myosu-chain --lib chain_spec` → 7/7; `cargo test -p myosu-chain --test testnet_build_spec` → 3/3; `SKIP_WASM_BUILD=1 cargo build --workspace --tests` → exit 0.


Generated: 2026-04-11
Codebase snapshot: trunk @ 4e0b37fbaa + local
Specs: gen-20260411-205202/specs/110426-*.md

---

## Priority Work

- [!] `PROMOTE-001` Promote nlhe-heads-up to promotable_local

  Spec: `specs/110426-canonical-truth-promotion.md`
  Why now: This is milestone 4 of the master plan, but it is a decision-gated integration proof, not evidence that a strong NLHE artifact already exists. NLHE is the intended first game to cross the promotion bar only after the policy bundle contract, dossier infrastructure, promotion ledger, and a real pinned artifact all verify together.
  Codebase evidence: `crates/myosu-games-poker/src/artifacts.rs` provides the artifact bundle infrastructure. `crates/myosu-games-poker/examples/benchmark_scenario_pack.rs` provides the independent benchmark surface. `ops/solver_promotion.yaml` (after PROMO-001) lists `nlhe-heads-up` at `benchmarked`. The sparse bootstrap artifacts remain negative fixtures; promotion requires a pinned stronger artifact referenced by hash.
  Owns: `outputs/solver-promotion/nlhe-heads-up/` directory with `bundle.json`, `benchmark-summary.json`, `artifact-manifest.json`. Update `ops/solver_promotion.yaml` to `tier: promotable_local` for `nlhe-heads-up`. Code path in `myosu-games-canonical` that can emit a verified policy bundle for a labeled heads-up decision point.
  Integration touchpoints: `crates/myosu-games-canonical/src/policy.rs` (CanonicalPolicyBundle construction), `crates/myosu-games-poker/src/artifacts.rs` (NlheArtifactDossier), `ops/solver_promotion.yaml` (tier update), `tests/e2e/promotion_manifest.sh` (must still pass after tier change).
  Scope boundary: Emit a policy bundle from a pinned NLHE artifact, verify it, produce promotion outputs. The strong artifact itself lives outside the repo (referenced by hash). Do NOT require training in this task. Do NOT modify the miner or validator. The task must fail closed if the artifact hash, manifest, or benchmark dossier is absent or below threshold; do not produce a placeholder promotable bundle.
  Acceptance criteria: (1) `outputs/solver-promotion/nlhe-heads-up/bundle.json` exists and is valid JSON. (2) `verify_policy_bundle()` succeeds on the emitted bundle. (3) `sample_policy_action()` succeeds with test entropy. (4) `ops/solver_promotion.yaml` shows `tier: promotable_local` for `nlhe-heads-up`. (5) `bash tests/e2e/promotion_manifest.sh` passes after the tier change (code support exists). (6) Sparse bootstrap artifacts are rejected as promotion inputs (negative test). (7) A missing or mismatched external artifact hash prevents bundle emission and leaves the ledger below `promotable_local`.
  Verification: `test -f outputs/solver-promotion/nlhe-heads-up/bundle.json && echo EXISTS`; `bash tests/e2e/promotion_manifest.sh`; `SKIP_WASM_BUILD=1 cargo test -p myosu-games-canonical --quiet`; `SKIP_WASM_BUILD=1 cargo test -p myosu-games-poker --quiet`.
  Required tests: (a) Policy bundle construction from pinned artifact succeeds. (b) Verify + sample roundtrip succeeds. (c) Sparse artifact rejection as promotion input. (d) Missing/mismatched external artifact hash rejects promotion. (e) Promotion manifest harness passes with updated YAML.
  Dependencies: DOSSIER-001, PROMO-001, PROMO-002.
  Blocker (2026-04-12): No promotion-grade pinned NLHE artifact dossier exists in the repo or local searched artifact locations. The only discovered dossier is `/tmp/myosu-nlhe-dossier-proof/outputs/solver-promotion/nlhe-heads-up/artifact-dossier.json`, generated by the DOSSIER-001 proof command; it records the sparse bootstrap shape (`total_entries=241`, `postflop_complete=false`) and a failing reference-pack benchmark (`passing=false`, `mean_l1_distance=1.228571504354477`). The current task explicitly requires a real pinned artifact hash, manifest, and passing benchmark dossier and forbids placeholder promotion bundles, so `nlhe-heads-up` must remain below `promotable_local` until an external full/promotion-grade artifact dossier is supplied and verified.
  Estimated scope: M
  Completion signal: `nlhe-heads-up` at `promotable_local` in YAML with verified bundle under `outputs/`.

### Checkpoint: Two dedicated games promoted

After PROMOTE-001 unblocks, both dedicated games must be at `promotable_local` with verified bundles and benchmark dossiers. `PROMOTE-002` is complete; this checkpoint remains blocked only by the missing promotion-grade NLHE artifact dossier. This is the master plan's milestone 4 exit gate. Verify: `bash tests/e2e/promotion_manifest.sh` passes, `bash tests/e2e/research_strength_harness.sh` passes, `bash tests/e2e/canonical_ten_play_harness.sh` passes, `SKIP_WASM_BUILD=1 cargo test --workspace --quiet` passes. Re-evaluate before proceeding to portfolio game work or Bitino integration.

---

- [x] `SEC-001` Triage all 19 advisory suppressions

  Spec: `specs/110426-security-posture.md`
  Why now: The CI advisory allowlist has grown to 19 entries (up from 7 in the 2026-04-03 review). Plan 008 proposes triage. Each advisory should have an explicit classification (remediate, accept, defer) with documented rationale. This is independent of promotion work and can run in parallel.
  Codebase evidence: `.github/workflows/ci.yml:358-376` lists 19 `--ignore` entries. AGENTS.md SEC-001 section documents the allowlist policy. The spec categorizes advisories as: directly owned (bincode 1.3.3, RUSTSEC-2025-0141), inherited chain (from opentensor polkadot-sdk fork), and workspace (paste, lru).
  Owns: Classification document or inline comments in CI workflow. Each advisory gets: classification, rationale, remediation plan or acceptance justification.
  Integration touchpoints: `.github/workflows/ci.yml` (allowlist comments), `WORKLIST.md` (SEC-001 entry update).
  Scope boundary: Classify and document all 19 advisories. Reduce the allowlist where feasible (remove advisories that have been fixed upstream). Do NOT migrate bincode (that is SEC-002). Do NOT rebase the polkadot-sdk fork.
  Acceptance criteria: (1) Every advisory in the CI allowlist has a documented classification (remediate/accept/defer) with justification. (2) `cargo audit -D warnings` with the updated allowlist passes. (3) Any advisory whose upstream crate has been patched is removed from the allowlist. (4) Inherited chain advisories are documented as "no direct Myosu usage" with per-advisory rationale.
  Verification: Run `cargo audit -D warnings` with the updated allowlist and verify exit 0. Count remaining ignores vs current 19.
  Required tests: None (documentation and CI config task).
  Dependencies: None (parallel with promotion work).
  Estimated scope: S
  Completion signal: All 19 advisories classified. Allowlist reduced where feasible. CI green.

- [x] `SEC-002` Bincode 1.3.3 migration decision

  Spec: `specs/110426-security-posture.md`
  Why now: RUSTSEC-2025-0141 is the only directly owned advisory in the allowlist. It affects wire paths in all three dedicated solver crates plus checkpoint/artifact paths in poker and Liar's Dice. The decision (migrate to bincode 2.x/postcard, or accept with documented rationale) must be made before any future payload-bearing checkpoint format changes.
  Codebase evidence: `crates/myosu-games-poker/src/solver.rs:7,20-21` uses bincode for payload-bearing poker checkpoints. `crates/myosu-games-liars-dice/src/solver.rs:6,21-22` uses the same `"MYOS"` + version + bincode checkpoint pattern. `crates/myosu-games-kuhn/src/solver.rs:10-12` uses a distinct `"MYOK"` + version exact-solver checkpoint without a bincode payload, while `crates/myosu-games-kuhn/src/wire.rs:1` uses bincode for wire serialization. Poker artifacts also use bincode in `crates/myosu-games-poker/src/artifacts.rs:5`.
  Owns: Decision document (ADR or inline in WORKLIST.md) evaluating: bincode 2.x migration, postcard migration, or acceptance with rationale. Must include: blast radius assessment, checkpoint compatibility strategy, robopoker fork impact.
  Integration touchpoints: `crates/myosu-games-poker/src/solver.rs`, `crates/myosu-games-poker/src/wire.rs`, `crates/myosu-games-poker/src/artifacts.rs`, `crates/myosu-games-kuhn/src/wire.rs`, `crates/myosu-games-liars-dice/src/solver.rs`, `crates/myosu-games-liars-dice/src/wire.rs`, robopoker fork dependency.
  Scope boundary: Research and decide only. Do NOT implement the migration. Do NOT change checkpoint format. Document the decision so a future worker can act on it.
  Acceptance criteria: (1) Decision is documented with explicit rationale. (2) Blast radius is assessed separately for wire formats, poker artifacts, poker checkpoints, Liar's Dice checkpoints, Kuhn exact-solver checkpoints, and robopoker fork impact. (3) If migration chosen: payload-bearing checkpoint versioning strategy is sketched (version 2 reader + version 1 fallback). (4) If acceptance chosen: risk mitigation documented (e.g., decode budget limits already in place at 1 MiB).
  Verification: Review-based. Document exists and addresses all criteria.
  Required tests: None (research task).
  Dependencies: None (parallel with promotion work).
  Estimated scope: S
  Completion signal: Decision documented with rationale and blast radius assessment.

  Resolution: ADR 012 in `docs/adr/012-bincode-1.3.3-decision.md` accepts bincode 1.3.3 with hardened decode budgets. The full blast-radius table is in the ADR, including wire formats, poker artifacts, poker checkpoints, Liar's Dice checkpoints, Kuhn exact-solver checkpoints (which do not use a bincode payload), and the robopoker fork (which does not depend on bincode). The kuhn wire decode budget was reduced from 256 MiB to 1 MiB in the same change (`myosu-games-kuhn/src/wire.rs:7-13`) so it matches every other wire and checkpoint site. `SECURITY.md` and `.github/workflows/ci.yml` were updated to cross-link the audit allowlist row to ADR 012. `ops/decision_log.md` records the decision with rationale and alternatives considered.

- [x] `DX-001` Consolidate critical operator caveats

  Spec: `specs/110426-developer-experience.md`
  Why now: The developer-experience spec documents that critical caveats (WASM cache requirement, sparse artifact limitations, devnet timing, Python dependency management, `SKIP_WASM_BUILD` requirement) are scattered across README.md, AGENTS.md, and OS.md instead of being concentrated. A new contributor must read three files to discover the fastest first-success path. Concentrating these reduces onboarding friction.
  Codebase evidence: `README.md:78-115` has operator commands. `AGENTS.md:303-346` has detailed caveats. `OS.md` has doctrine. The developer-experience spec identifies this scatter as a finding. `docs/operator-guide/quickstart.md` exists but focuses on the operator network path, not the developer contributor path.
  Owns: A consolidated quickstart section (either in `CONTRIBUTING.md` or an expanded `docs/developer-quickstart.md`) that covers: prerequisites, environment variables, fastest first-success commands, common pitfalls, and links to deeper docs.
  Integration touchpoints: `README.md` (add link to new doc), existing operator guide docs.
  Scope boundary: Consolidate existing scattered information. Do NOT create new documentation content beyond what already exists across the three source files. Do NOT change code behavior. Do NOT add JSON output mode (that is a separate task if needed).
  Acceptance criteria: (1) All critical caveats from AGENTS.md (WASM cache, sparse artifacts, devnet timing, SKIP_WASM_BUILD, wasm32v1-none target) appear in one place. (2) The document includes the 4-step fastest first-success path from the developer-experience spec. (3) README.md links to the consolidated document. (4) Environment variable inventory covers at least: SKIP_WASM_BUILD, MYOSU_KEY_PASSWORD, MYOSU_NODE_AUTHORITY_SURI.
  Verification: `test -f docs/developer-quickstart.md && echo EXISTS` (or CONTRIBUTING.md); verify links with `grep 'developer-quickstart\|CONTRIBUTING' README.md`.
  Required tests: None (documentation task).
  Dependencies: None (parallel with all other work).
  Estimated scope: S
  Completion signal: Consolidated document exists. README links to it.

  Resolution: shipped the consolidated developer quickstart as one coherent change. `docs/developer-quickstart.md` (new) collects prerequisites (Rust toolchain, `wasm32v1-none` / `wasm32-unknown-unknown`, `protoc` / `protobuf-compiler`, Python 3.11+ with `numpy`/`pytest`/`ruff`), the 4-step fastest first-success path verbatim from `specs/110426-developer-experience.md` (`cargo test -p myosu-games-kuhn --quiet` → `myosu-play --smoke-test` → `myosu-play --game kuhn --smoke-test` → `printf 'quit\n' | myosu-play pipe`), the five critical caveats pulled from AGENTS.md / OS.md (SKIP_WASM_BUILD requirement, `wasm32v1-none` target, `target/debug/wbuild/myosu-chain-runtime/myosu_chain_runtime.wasm` cache, sparse poker bootstrap artifacts with `postflop_complete=false`, 48-second devnet block timing), and a 19-row environment variable inventory sourced from the actual `env::var`/`env::var_os` call sites in the Rust crates (`SKIP_WASM_BUILD`, `MYOSU_NODE_AUTHORITY_SURI`, `MYOSU_KEY_PASSWORD`/`MYOSU_PASSWORD`/`MYOSU_MNEMONIC`/`MYOSU_RAW_SEED`/`MYOSU_OLD_PASSWORD`/`MYOSU_NEW_PASSWORD`, `MYOSU_DATA_DIR`, `MYOSU_BLUEPRINT_DIR`, `MYOSU_SOLVER_PROMOTION_LEDGER`, `MYOSU_ENGINE_BUDGET_MS`/`MYOSU_ENGINE_ITERATIONS`/`MYOSU_ENGINE_MIN_SCORE`, `MYOSU_CRIBBAGE_BENCHMARK_OUTPUT`, `MYOSU_E2E_GAMES`, plus the operator-facing `MYOSU_CONFIG_DIR`/`MYOSU_SUBNET`/`MYOSU_WORKDIR`). `README.md` "Developer Path" section gains a link to the new doc. The executable gate `tests/e2e/developer_quickstart.sh` enforces all four acceptance criteria (doc exists, README links to it, the four-step path is present verbatim, all five critical caveats are present, the three required env vars `SKIP_WASM_BUILD` / `MYOSU_KEY_PASSWORD` / `MYOSU_NODE_AUTHORITY_SURI` are present, the developer-experience spec and `DX-001` plan row are cross-referenced) and then actually runs the four-step path end-to-end so the documented commands stay truthful. `.github/workflows/ci.yml` gains a `developer-quickstart` job that runs the gate after `repo-shape`. Proof: `bash tests/e2e/developer_quickstart.sh` exits 0 with the four steps succeeding; `grep -F 'docs/developer-quickstart.md' README.md` matches; the five caveat literals and three required env vars all grep-match inside the new doc.

---

## Follow-On Work

- [x] `F-001` Research: Cribbage deepening to benchmarked tier

  Spec: `specs/110426-canonical-truth-promotion.md`
  Why now: This is milestone 5 of the master plan. Cribbage is the default first portfolio game for promotion work. However, the canonical-truth-promotion spec notes as a hypothesis that "whether policy bundle generalization to portfolio games is feasible without dedicated solver crates is open." This task must validate that hypothesis before attempting promotion.
  Codebase evidence: `crates/myosu-games-portfolio/src/core/cribbage.rs` and `crates/myosu-games-portfolio/src/engines/cribbage.rs` exist. Portfolio engines are all `rule-aware` tier (not trained CFR). Plan 009 says cribbage deepening targets `benchmarked` (not `promotable_local`).
  Owns: Scenario pack and benchmark surface for cribbage. Update `ops/solver_promotion.yaml` tier from `routed` to `benchmarked`.
  Integration touchpoints: `crates/myosu-games-portfolio/`, `ops/solver_promotion.yaml`, `tests/e2e/promotion_manifest.sh`.
  Scope boundary: Deepen cribbage only. Do NOT attempt `promotable_local` for any portfolio game. Do NOT create a dedicated cribbage solver crate.
  Acceptance criteria: (1) Cribbage scenario pack with labeled states exists. (2) Benchmark surface reports engine quality metrics. (3) `ops/solver_promotion.yaml` shows `tier: benchmarked` for cribbage. (4) Promotion manifest harness passes.
  Verification: `bash tests/e2e/promotion_manifest.sh`; `SKIP_WASM_BUILD=1 cargo test -p myosu-games-portfolio --quiet`.
  Required tests: Cribbage scenario pack tests, benchmark metric assertions.
  Dependencies: PROMOTE-001, PROMOTE-002 (both dedicated games promoted first, per master plan ordering).
  Estimated scope: M
  Completion signal: Cribbage at `benchmarked` in YAML with scenario pack and benchmark evidence.

  Resolution: shipped the Cribbage rule-aware scenario pack and benchmark dossier as one coherent change. `crates/myosu-games-portfolio/src/core/cribbage.rs` gains `CribbageScenario` (22 rows; coverage buckets `opening-discard×4`, `pegging-fifteen×4`, `pegging-pair×3`, `pegging-run×3`, `pegging-go×3`, `pegging-thirty-one×2`, `counting×3`) and the const `SCENARIO_PACK` exposed through `cribbage_scenario_pack()`. A new crate module `crates/myosu-games-portfolio/src/cribbage_benchmark.rs` defines `CribbageBenchmarkDossier` with all required promotion fields (`benchmark_id`, `benchmark_method`, `metric_name`, `metric_value`, `threshold`, `passing`, `scenario_count`, `recommendation_count`, `engine_family`, `engine_tier`, `rule_file`, `scenario_hash`, `recommendations: BTreeMap<String, String>`) and a deterministic SHA-256 over the canonical scenario/answer table (sorted by `scenario_id` so the hash is independent of pack iteration order). The dossier is wired through the live `rule-aware` engine via `answer_typed_challenge` + `recommended_action`, action tokens map cleanly to the three legal pegging/keep/discard actions, and 4 unit tests cover (a) the full pack passes, (b) the dossier is byte-stable across runs, (c) every scenario records a valid action token, and (d) the run-heavy scenarios (`pegging-run-three`, `pegging-run-four-setup`) both recommend `peg-run`. The example binary `crates/myosu-games-portfolio/examples/cribbage_benchmark.rs` runs the live engine against the pack, prints a one-line `CRIBBAGE_BENCHMARK` summary plus 22 `CRIBBAGE_SCENARIO` lines, writes the JSON dossier to `outputs/solver-promotion/cribbage/cribbage-benchmark-dossier.json` (overridable via `MYOSU_CRIBBAGE_BENCHMARK_OUTPUT`), and exits non-zero if the dossier does not pass. `ops/solver_promotion.yaml` Cribbage row advances to `tier: benchmarked`, `bundle_support: benchmarked`, `benchmark_surface: rule_aware_scenario_pack`, `benchmark_threshold: recommendation_count_eq_scenario_count`, `artifact_requirement: scenario_pack_plus_benchmark_dossier`, `notes: cribbage_rule_aware_engine_scenario_pack_promoted_from_routed`. New e2e harness `tests/e2e/cribbage_benchmark_dossier.sh` exercises the example, parses the on-disk JSON with shape checks (scenario_count=22, recommendation_count=22, engine_tier=rule-aware, scenario_hash=64-hex, every recommendation is in the allowed action set), asserts the on-disk ledger row at `tier=benchmarked` / `bundle_support=benchmarked`, runs `promotion_manifest` to assert the manifest row carries `tier=benchmarked code_bundle_support=benchmarked benchmark_surface=rule_aware_scenario_pack`, and runs the `cribbage` + `cribbage_dossier` portfolio unit tests. `.github/workflows/ci.yml` runs the new harness from the `dependency-audit` job as `Verify F-001 Cribbage benchmark dossier`. Proof: `bash tests/e2e/cribbage_benchmark_dossier.sh` exits 0 (`CRIBBAGE_BENCHMARK_HARNESS myosu e2e ok scenario_hash=32dd030814a82027fb305af22fbab5434d35f399db11bf7dd654623369022ffe ledger=/srv/dev/repos/myosu/ops/solver_promotion.yaml`); `bash tests/e2e/promotion_manifest.sh` exits 0 with `cribbage tier=benchmarked code_bundle_support=benchmarked`; `SKIP_WASM_BUILD=1 cargo test -p myosu-games-portfolio --quiet` is 202/202 green; `SKIP_WASM_BUILD=1 cargo test -p myosu-games-canonical --quiet` is 24/24 green; `SKIP_WASM_BUILD=1 cargo check -p myosu-games-portfolio` exits 0.

- [ ] `F-002` Token economics decision document

  Spec: `specs/110426-chain-runtime-pallet.md`
  Why now: Carries forward from prior plan F-003. The token-economics spec is a research spec identifying 8+ design axes that must be decided before `Stage0NoopSwap` can be replaced. ADR-008 exists (`docs/adr/008-future-token-economics-direction.md`) with `Status: Proposed` but still lacks the multi-contributor review required by the spec.
  Codebase evidence: `crates/myosu-chain/pallets/game-solver/src/swap/` contains the NoOpSwap implementation. `docs/adr/008-future-token-economics-direction.md` exists with `Status: Proposed`. No recorded multi-contributor signoff found in `docs/adr/README.md`, `ops/decision_log.md`, or `.github/`.
  Owns: Multi-contributor review completion and status update for ADR-008.
  Integration touchpoints: `docs/adr/008-future-token-economics-direction.md`, `ops/decision_log.md`.
  Scope boundary: Review and document only. Do NOT change swap implementation. Do NOT wire V3 AMM into runtime.
  Acceptance criteria: (1) ADR-008 reviewed by at least two contributors with token-economics context. (2) Review recorded in `docs/adr/README.md` or `ops/decision_log.md`. (3) ADR status updated from `Proposed` to `Accepted` or `Superseded`.
  Verification: Review-based. `grep 'Status:' docs/adr/008-future-token-economics-direction.md`.
  Required tests: None (research task).
  Dependencies: None, but blocked on external review (not a code task).
  Estimated scope: L
  Completion signal: Multi-contributor review recorded. ADR status updated.

- [ ] `F-003` Miner convergence gate research

  Spec: `specs/110426-operator-stack.md`
  Why now: Carries forward from prior plan F-007. No convergence gate exists — a miner can train for 1 iteration and serve garbage. Validators score it low, but operators have no guidance on minimum training. This task is still blocked on a truthful quality benchmark surface.
  Codebase evidence: Positive-iteration poker training rejects bootstrap artifacts where `postflop_complete = false` (`crates/myosu-games-poker/src/artifacts.rs`). The reference-pack benchmark exists at `crates/myosu-games-poker/examples/benchmark_scenario_pack.rs` but is independent of the self-check validator path. `bash tests/e2e/research_strength_harness.sh` exercises it. Liar's Dice has `exact_exploitability()` which is a truthful quality metric.
  Owns: Research document with recommended minimum iterations per game type, using truthful benchmark surfaces (not same-checkpoint self-check).
  Integration touchpoints: Miner CLI documentation, operator guide.
  Scope boundary: Measure and document. Do NOT enforce in code.
  Acceptance criteria: (1) Minimum iteration guidance documented for Liar's Dice (using exact exploitability). (2) Poker convergence guidance documented or explicitly marked as blocked on richer encoder artifacts. (3) Quality thresholds documented per game type.
  Verification: Run miner with varying iteration counts against quality benchmarks. For Liar's Dice: `exact_exploitability()` at N iterations. For poker: benchmark_scenario_pack reference surface (if artifacts available).
  Required tests: None (research task).
  Dependencies: DOSSIER-001 (NLHE benchmark surface), DOSSIER-002 (Liar's Dice exploitability surface). Poker convergence additionally blocked on richer encoder artifacts.
  Estimated scope: S
  Completion signal: Minimum iterations documented per game type with truthful quality evidence.

- [ ] `F-004` Bitino policy canonical crate (sibling repo)

  Spec: `specs/110426-canonical-truth-promotion.md`
  Why now: Milestone 6 of the master plan. The Bitino-side adapter crate deserializes Myosu policy bundles and converts them to Bitino-local table state. This is the first cross-repo integration point.
  Codebase evidence: Master plan specifies `../bitino/crates/bitino-policy-canonical/` as the new crate. `../bitino/crates/bitino-wire/src/interactive.rs` defines `InteractivePresentation` (the rendering envelope). `../bitino/crates/bitino-engine/src/types.rs` defines `GameId`.
  Owns: `../bitino/crates/bitino-policy-canonical/` (sibling repo). New `GameId` values for solver-backed games.
  Integration touchpoints: Bitino Cargo.toml, bitino-engine types, bitino-play TUI, bitino-play agent state.
  Scope boundary: Local adapter only. Do NOT implement funded settlement. Do NOT require live Myosu chain connection. Bundle loading from local files only.
  Acceptance criteria: (1) `bitino-policy-canonical` crate compiles. (2) Can deserialize a Myosu `CanonicalPolicyBundle` from JSON. (3) Can verify the deserialized bundle. (4) New `GameId` values exist for solver-backed games. (5) `InteractivePresentation` can be constructed from a policy bundle.
  Verification: `cargo test -p bitino-policy-canonical --quiet` in sibling repo.
  Required tests: Deserialization, verification, presentation construction.
  Dependencies: PROMOTE-001 (needs at least one verified bundle to test against).
  Estimated scope: M
  Completion signal: Bitino can deserialize and render a Myosu policy bundle locally.

- [ ] `F-005` Bitino local table adapter and same-TUI pilot

  Spec: `specs/110426-canonical-truth-promotion.md`
  Why now: Milestone 6 continued. The adapter connects the policy canonical crate to the Bitino TUI. The master plan says "the first Bitino proof should show one solver-backed heads-up hold'em table rendered through the normal Bitino TUI from a local pinned bundle."
  Codebase evidence: Master plan specifies `../bitino/crates/bitino-play/src/solver_tables.rs` with `SolverTableSession`, `load_solver_table()`, `presentation_from_policy_bundle()`.
  Owns: `solver_tables.rs` in bitino-play, TUI wiring, agent state catalog extension.
  Integration touchpoints: `bitino-play/src/tui/mod.rs`, `bitino-play/src/agent/state.rs`, `bitino-engine/src/types.rs`.
  Scope boundary: Offline local table only. No funded settlement. No live miner discovery. Bundle loaded from file path. Preserve the normal Bitino TUI information architecture: ready-room discovery, table rendering, session metadata, and clear invalid-bundle errors. Do not add a one-off solver UI that bypasses existing keyboard/headless flows.
  Acceptance criteria: (1) A solver-backed table appears in Bitino ready room. (2) Renders through normal Bitino TUI framework. (3) Session/round details expose bundle id, artifact hash, benchmark label. (4) Invalid, missing, or unverifiable policy bundles render actionable error states instead of panics or silent fallback. (5) Existing keyboard/headless flow still reaches the table. (6) Proof command: `cargo run -q -p bitino-play -- --headless 1 --game solver_holdem_heads_up --policy-bundle <path>`.
  Verification: Run the headless proof command in Bitino repo.
  Required tests: Headless table session, presentation rendering, session metadata, invalid-bundle error state.
  Dependencies: F-004 (policy canonical crate), PROMOTE-001 (bundle to test with).
  Estimated scope: L
  Completion signal: Solver-backed table visible in Bitino TUI from local bundle.

- [ ] `F-006` Funded integration (sibling repo)

  Spec: `specs/110426-canonical-truth-promotion.md`
  Why now: Milestone 7 of the master plan. Should NOT start early. Only once the offline same-TUI pilot (F-005) is stable. Adds policy-bundle loading to bitino-house, house-action sampling with Bitino fairness entropy, wire-level replay/provenance, and settlement integration.
  Codebase evidence: Master plan: "the realized action must be replayable from the saved fairness draw and the saved policy bundle hash."
  Owns: `../bitino/crates/bitino-house/` extensions for policy-bundle-backed rounds.
  Integration touchpoints: bitino-house, bitino-wire, bitino-settlement.
  Scope boundary: Funded rounds with replay proof. Requires stable offline pilot first.
  Acceptance criteria: (1) Funded solver-backed round logs bundle hash and fairness draw. (2) Replay can reproduce sampled action from saved data. (3) If verification fails, funded slice is incomplete.
  Verification: Replay proof in Bitino test suite.
  Required tests: Replay determinism, settlement accounting.
  Dependencies: F-005 (stable offline pilot).
  Estimated scope: L
  Completion signal: Funded solver-backed round with replayable action proof.

---

- [ ] `F-007` Research: Minimum training iterations for meaningful strategy quality

  Spec: `specs/050426-mining-surface.md`
  Why now: The spec notes "no convergence gate exists" — a miner can train for 1 iteration and serve garbage. Validators score it low, but operators have no guidance on minimum training.
  Codebase evidence: `crates/myosu-miner/src/` training loop and `--train-iterations` flag accept any non-negative iteration count without a quality gate. `crates/myosu-validator/src/validation.rs` scores a miner response against `solver.answer(query)` from the checkpoint passed on the validator CLI, and both `tests/e2e/local_loop.sh` and `tests/e2e/validator_determinism.sh` currently pass the miner-produced checkpoint straight back into validator scoring.
  Owns: Research document or code comment with recommended minimums per game type.
  Integration touchpoints: Miner CLI documentation, operator guide.
  Scope boundary: Measure and document. Do not enforce in code (that's follow-on).
  Acceptance criteria: (1) A training-quality threshold that actually varies with solver quality is documented for Poker and Liar's Dice, using a truthful benchmark surface (exact exploitability or comparison against an independent reference checkpoint rather than the current self-check validator path). (2) Operator guide updated with recommendation.
  Verification: Run miner with varying iteration counts against the chosen quality benchmark. Do not use the current same-checkpoint validator exact-match path as convergence evidence.
  Required tests: None (research task).
  Dependencies: P-009 (determinism verified across games).
  Blocker (2026-04-05, re-verified 2026-04-12): The current stage-0 validator score is not a convergence metric. `score_response()` in `crates/myosu-validator/src/validation.rs` compares the observed miner response against `solver.answer(query)` from the checkpoint supplied on the validator CLI, and the repo-owned happy-path harnesses (`tests/e2e/local_loop.sh`, `tests/e2e/validator_determinism.sh`) pass the miner checkpoint straight into that validator path, so the truthful expected result is `exact_match=true` / `score=1.0` whenever the miner response came from the same checkpoint. Poker now has an independent reference-pack benchmark surface in `crates/myosu-games-poker/examples/benchmark_scenario_pack.rs`, and `bash tests/e2e/research_strength_harness.sh` exercises it against the repo-owned 80-scenario checkpoint pack instead of self-scoring the candidate checkpoint. Positive-iteration poker training is still blocked, though, because the checked-in bootstrap artifact path (`crates/myosu-games-poker/examples/bootstrap_artifacts.rs`) remains postflop-sampled, and direct re-verification against the generated `target/e2e/*/poker/encoder/manifest.json` outputs shows the same shape (`preflop.entries = 169`, `flop.entries = 24`, `turn.entries = 24`, `river.entries = 24`, `complete_streets=preflop`, `sampled_streets=flop,turn,river`, `postflop_complete=false`). `cargo test -p myosu-validator --quiet exact_match_scores_one`, `cargo test -p myosu-miner --quiet run_poker_training_batch_rejects_incomplete_artifacts_before_training`, `cargo test -p myosu-games-poker --quiet artifacts::tests::bootstrap_encoder_streets_report_sampled_postflop_shape`, and `cargo test -p myosu-games-poker --quiet benchmark::tests::sparse_bootstrap_checkpoint_differs_from_reference_pack` confirm that the validator happy path is a self-check, the reference-pack benchmark is independent, and positive poker training iterations on the repo-owned sparse artifacts are still rejected cleanly before training begins. Until the repo has richer poker encoder artifacts or another trainable benchmark surface, this task still cannot truthfully document minimum training iterations. This task subsumes the deferred Nemesis follow-up `NEM-008`; keep one canonical queue entry here.
  Estimated scope: S
  Completion signal: Minimum iterations documented per game type.


## Completed / Already Satisfied

- [x] `C-001` NoOpSwap identity stub implements all 37 swap callsites
  Spec: `specs/110426-chain-runtime-pallet.md`
  Codebase evidence: `Stage0NoopSwap` with 1:1 conversion, zero fees in runtime. Verified by `cargo test -p pallet-game-solver coinbase --quiet`.

- [x] `C-002` INV-004 solver-gameplay dependency boundary enforced in CI
  Spec: `specs/110426-ci-quality-gates.md`
  Codebase evidence: `cargo tree` check in `.github/workflows/ci.yml:145-158`.

- [x] `C-003` Multi-game architecture with zero-change extensibility
  Spec: `specs/110426-game-solver-core.md`
  Codebase evidence: `GameRegistry::supported()` returns 23 games. Poker, Kuhn, Liar's Dice, 20 portfolio games all implement solver traits independently. Adding Liar's Dice required zero poker changes.

- [x] `C-004` Workspace clippy lints enforced
  Spec: `specs/110426-ci-quality-gates.md`
  Codebase evidence: Workspace `Cargo.toml` `[lints.clippy]` denies: `arithmetic-side-effects`, `expect-used`, `indexing-slicing`, `unwrap-used`. CI runs with `-D warnings`.

- [x] `C-005` Validator scoring with hyperbolic formula and determinism tests
  Spec: `specs/110426-operator-stack.md`
  Codebase evidence: `score = 1.0 / (1.0 + l1_distance)` in `validation.rs`. 14+ unit tests. `validator_determinism.sh` in CI. INV-003 epsilon < 1e-6.

- [x] `C-006` Miner 7-step lifecycle
  Spec: `specs/110426-operator-stack.md`
  Codebase evidence: `crates/myosu-miner/src/main.rs` implements: probe, register, serve_axon, train, strategy, http_axon. All steps have structured reports.

- [x] `C-007` Gameplay surface with three modes
  Spec: `specs/110426-gameplay-surface.md`
  Codebase evidence: `crates/myosu-play/src/main.rs` implements smoke-test, TUI (train), and pipe modes. CI runs both poker and kuhn smoke tests.

- [x] `C-008` Key management with create, import, export, switch
  Spec: `specs/110426-key-management.md`
  Codebase evidence: `crates/myosu-keys/src/lib.rs` and `src/storage.rs` implement: `generate_mnemonic()`, `mnemonic_to_pair()`, `save_pair()`, `load_active_pair()`, `import_keyfile()`, `export_active_keyfile()`, `set_active_account()`, `list_stored_accounts()`. XSalsa20-Poly1305 encryption with scrypt KDF.

- [x] `C-009` Two-node block sync proven
  Spec: `specs/110426-chain-runtime-pallet.md`
  Codebase evidence: `tests/e2e/two_node_sync.sh` in CI. Proves named-devnet peer discovery with `MYOSU_NODE_AUTHORITY_SURI`.

- [x] `C-010` Aura + GRANDPA consensus with 4 chain spec variants
  Spec: `specs/110426-chain-runtime-pallet.md`
  Codebase evidence: Runtime `construct_runtime!` includes `pallet_aura` and `pallet_grandpa`. Chain specs: `localnet`, `devnet`, `testnet`, `finney`.

- [x] `C-011` Epoch consistency guard
  Spec: `specs/110426-chain-runtime-pallet.md`
  Codebase evidence: `is_epoch_input_state_consistent(netuid)` check in `run_epoch.rs`. Two tests verify the guard.

- [x] `C-012` Zero-dividend fallback distributes by stake weight
  Spec: `specs/110426-chain-runtime-pallet.md`
  Codebase evidence: Stake-weighted fallback in `run_coinbase.rs`. Test verifies 750/250 distribution for 3:1 stake.

- [x] `C-013` Decode budget hardened to 1 MiB
  Spec: `specs/110426-game-solver-core.md`
  Codebase evidence: `MAX_DECODE_BYTES` = 1 MiB in `solver.rs` and `wire.rs`. Tests verify oversized payloads rejected.

- [x] `C-014` Operator guide documentation exists
  Spec: `specs/110426-developer-experience.md`
  Codebase evidence: `docs/operator-guide/quickstart.md`, `upgrading.md`, `troubleshooting.md`, `architecture.md` all exist.

- [x] `C-015` Environment variable contracts documented
  Spec: `specs/110426-developer-experience.md`
  Codebase evidence: `MYOSU_KEY_PASSWORD`, `MYOSU_CONFIG_DIR`, `MYOSU_SUBNET`, `MYOSU_WORKDIR`, `MYOSU_CHAIN`, `MYOSU_OPERATOR_CHAIN` referenced in operator tooling and code.

- [x] `C-016` GameType on-chain encoding with proptest roundtrip
  Spec: `specs/110426-game-solver-core.md`
  Codebase evidence: `GameType::from_bytes` / `to_bytes` with `#[non_exhaustive]` enum. CI runs `serialization_roundtrip`.

- [x] `C-017` Structured report types for miner, validator, and key management
  Spec: `specs/110426-operator-stack.md`
  Codebase evidence: 6 report types each in miner and validator with machine-readable prefixes per bootstrap stage.

- [x] `C-018` 11 CI jobs covering workspace, chain, E2E, doctrine, audit, operator
  Spec: `specs/110426-ci-quality-gates.md`
  Codebase evidence: `.github/workflows/ci.yml` defines: repo-shape, robopoker-fork-coherence, python-research-qa, active-crates, chain-core, integration-e2e, doctrine, dependency-audit, plan-quality, operator-network, chain-clippy.

- [x] `C-019` 7 E2E integration proofs pass in CI
  Spec: `specs/110426-ci-quality-gates.md`
  Codebase evidence: `tests/e2e/` contains: local_loop.sh, two_node_sync.sh, four_node_finality.sh, consensus_resilience.sh, cross_node_emission.sh, validator_determinism.sh, emission_flow.sh. All wired in `.github/workflows/ci.yml:296-315`.

- [x] `C-020` GitHub Actions pinned to full SHA with persist-credentials: false
  Spec: `specs/110426-security-posture.md`
  Codebase evidence: All 5 action references in `ci.yml` use full SHA hashes with version comments. All checkouts use `persist-credentials: false`. Permissions scoped to `contents: read`.

- [x] `C-021` Canonical manifest gate: 10 games, 10 snapshot=ok
  Spec: `specs/110426-canonical-truth-promotion.md`
  Codebase evidence: `CANONICAL_TEN` contains exactly 10 games. CI validates: `cargo run -p myosu-games-canonical --example canonical_manifest` produces 10 `CANONICAL_GAME` lines and 10 `snapshot=ok`. Playtrace tests pass.

- [x] `C-022` All 4 E2E play/research harnesses pass in CI
  Spec: `specs/110426-gameplay-surface.md`
  Codebase evidence: `canonical_ten_play_harness.sh`, `research_play_harness.sh`, `research_games_harness.sh`, `research_strength_harness.sh` all wired in `.github/workflows/ci.yml:136-143`.

- [x] `C-023` 23 game types in registry with Custom extensibility
  Spec: `specs/110426-game-solver-core.md`
  Codebase evidence: `GameType` enum has 23 named variants plus `Custom(String)` at `crates/myosu-games/src/traits.rs:66`. `GameRegistry::supported()` returns 23 descriptors at `crates/myosu-games/src/registry.rs:43`.

- [x] `C-024` Four-authority finality proof with 1-down resilience
  Spec: `specs/110426-chain-runtime-pallet.md`
  Codebase evidence: `tests/e2e/four_node_finality.sh` starts 4 authorities, stops 1, asserts surviving 3 keep finalizing. Threshold math: 3/3 for 4-authority set.

- [x] `C-025` Emission accounting: sum(distributions) == block_emission * epochs
  Spec: `specs/110426-chain-runtime-pallet.md`
  Codebase evidence: `tests/e2e/emission_flow.sh` proves emission accounting integrity. Dust within `TOTAL_ISSUANCE_TRY_STATE_ALERT_DELTA` (1000 rao).

- [x] `C-026` Consensus resilience: authority restart and catch-up
  Spec: `specs/110426-chain-runtime-pallet.md`
  Codebase evidence: `tests/e2e/consensus_resilience.sh` stops authority-4, waits for 1-3 to finalize, restarts authority-4, requires all 4 agree on finalized head.

- [x] `C-027` Cross-node emission agreement
  Spec: `specs/110426-chain-runtime-pallet.md`
  Codebase evidence: `tests/e2e/cross_node_emission.sh` starts 4-authority devnet, drives registration/weights, snapshots emission maps at shared block-23 hash.

- [x] `C-028` KeyError enum with 15+ actionable variants
  Spec: `specs/110426-key-management.md`
  Codebase evidence: `crates/myosu-keys/src/lib.rs:22-91` defines: InvalidMnemonic, MissingHomeDir, MissingPasswordEnv, MissingKeySource, CreateDirectory, InvalidPath, ReadFile, WriteFile, InvalidConfig, SerializeKeyfile, DeserializeKeyfile, InvalidKeyfile, MissingKeyfile, InvalidKeyfileHex, KeyDerivation, EncryptSeed, DecryptSeed, InvalidSeedMaterial, SecretUriUnsupported.

- [x] `C-029` Miner rejects positive-iteration poker training on sparse artifacts
  Spec: `specs/110426-operator-stack.md`
  Codebase evidence: `crates/myosu-games-poker/src/artifacts.rs` enforces `postflop_complete = false` rejection. `cargo test -p myosu-miner --quiet run_poker_training_batch_rejects_incomplete_artifacts_before_training` passes.

- [x] `C-030` Validator determinism across all three dedicated games
  Spec: `specs/110426-operator-stack.md`
  Codebase evidence: `tests/e2e/validator_determinism.sh` defaults to poker, liars-dice, and kuhn in one run. `cargo test -p myosu-validator --quiet inv_003_determinism` and `liars_dice_inv_003_determinism` pass.

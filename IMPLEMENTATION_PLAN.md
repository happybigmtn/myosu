# IMPLEMENTATION_PLAN

Generated: 2026-04-05
Codebase snapshot: trunk @ ba63a7d + local
Specs: gen-20260405-145446/specs/050426-*.md

---

## Priority Work

### Completed prerequisites

- `P-001` is already satisfied by commit `ba63a7d` (`myosu: auto loop checkpoint`), which landed the zero-dividend fallback, epoch consistency guard, validator scoring change, decode budget tightening, INV-004 CI gate, and Cargo.toml de-workspacing that the older queue still described as uncommitted.
- `P-002` is satisfied in the current local slice: `cargo test -p pallet-game-solver -- truncation` now sweeps 1 / 100 / 1_000 / 10_000 accrued blocks across representative emission rates and measures a worst-case stage-0 drift of 2 rao per accrued block (6 rao over the default tempo-2 epoch). The correction decision is intentionally deferred to `WORKLIST.md`.

### Checkpoint: Chain core confidence

After P-001 through P-007: emission is measured, E2E tests are wired, CI is hardened, dead code is audited. Pause and verify trunk CI is green, all E2E scripts pass, and no new regressions. Re-evaluate scope before proceeding to multi-node and operator work.

---

### Cluster 5: Validation and Scoring Hardening (dependency: P-001)

---

### Cluster 6: Miner HTTP Axon Gaps (dependency: P-001)

---

### Checkpoint: Scoring and serving confidence

After P-008 through P-010: wire codecs are fuzz-tested, determinism is verified across all games, and the Liar's Dice HTTP gap is resolved. Verify CI green before proceeding to multi-node work.

---

### Cluster 7: Multi-Node Devnet Foundation (dependency: P-003, P-005)

- [ ] `P-011` Three-node GRANDPA finality proof

  Spec: `specs/050426-network-consensus.md`
  Why now: Two-node sync is proven (and wired into CI via P-005). Three-node GRANDPA finality is the next consensus milestone. The spec explicitly lists this as "not proven (design-phase)" and gates Phase 1 completion on it.
  Codebase evidence: `tests/e2e/two_node_sync.sh` (2-node). Chain specs in `crates/myosu-chain/node/` define `devnet` with 3 authorities. No 3-node E2E test exists.
  Owns: New E2E script `tests/e2e/three_node_finality.sh` and CI wiring.
  Integration touchpoints: Node binary, devnet chain spec (3 authorities), GRANDPA finality gadget, CI `integration-e2e` job.
  Scope boundary: Prove finality with 3 nodes. Test one-node-down tolerance (2/3 still finalizes). Do not test network partitions or restart recovery (those are follow-on).
  Acceptance criteria: (1) 3 nodes start, produce blocks, and reach GRANDPA finality. (2) Stopping 1 node does not halt finality (2/3 quorum). (3) Script runs in CI.
  Verification: `bash tests/e2e/three_node_finality.sh`
  Required tests: The E2E script is the test. Should assert finalized block height increases after epoch transitions.
  Dependencies: P-005 (two_node_sync in CI — proves the infrastructure works).
  Blocker (2026-04-05): Live repros confirm this is a chain/runtime issue, not just a missing harness. Two ad hoc full-mesh 3-authority devnets with unique RPC/P2P/Prometheus ports and distinct `--node-key-file` identities for every authority reproduced the same stall on both the default `litep2p` backend and explicit `--network-backend libp2p`: all three authorities reached finalized block `#2`, then after terminating `authority-3` the surviving authorities kept importing blocks through `#9` while finalized height stayed frozen at `#2` on both nodes. The explicit `libp2p` repro also logged a GRANDPA prevote equivocation for `authority-2`. There is still no tracked `tests/e2e/three_node_finality.sh`; do not treat that missing script as absence of a repro.
  Estimated scope: M
  Completion signal: 3-node finality script passes in CI.

- [ ] `P-012` Cross-node emission agreement test

  Spec: `specs/050426-emission-epoch-mechanism.md`
  Why now: The spec explicitly states cross-node emission agreement is "tested single-node only." Fixed-point determinism is assumed but not proven across nodes. This is the highest-risk multi-node property — if nodes disagree on emission, the chain forks.
  Codebase evidence: All epoch/coinbase tests run in single-node mock runtime. `substrate_fixed` types (I32F32, I64F64, U96F32) are deterministic per the spec but unverified across separate process instances.
  Owns: New E2E test or extension of `three_node_finality.sh` that compares emission storage across nodes after epoch transitions.
  Integration touchpoints: RPC endpoints for reading storage, epoch mechanism, coinbase pipeline, node binary.
  Scope boundary: Compare emission-related storage values across 3 nodes after N epochs. Assert bit-identical. Do not test under adversarial conditions.
  Acceptance criteria: (1) After 3+ epoch transitions on a 3-node devnet, emission storage values (total issuance, per-subnet pending, stake maps) are identical across all nodes. (2) Test runs in CI.
  Verification: E2E script that queries storage via RPC on all 3 nodes and diffs.
  Required tests: The E2E script.
  Dependencies: P-011 (3-node devnet running), P-002 (truncation drift quantified).
  Estimated scope: M
  Completion signal: Cross-node emission agreement test passes in CI.

---

### Checkpoint: Multi-node confidence

After P-011 and P-012: 3-node finality is proven, cross-node emission agreement is verified. This satisfies the Phase 1 gate from the network-consensus spec. Re-evaluate whether Phase 2 (operator packaging) work should begin or whether restart resilience testing is needed first.

---

- [ ] `NEM-001` Fix GRANDPA finality stalling on 3-node devnet after authority stop

  Status (2026-04-05): Blocked. Live repro plus `finality-grandpa` threshold
  math show a 3-authority equal-weight voter set needs 3 votes to finalize, so
  two surviving authorities can keep importing best blocks but cannot satisfy
  the requested 2-of-3 finality tolerance without changing authority count or
  voting weights.

  Spec: `WORKLIST.md:NET-FINALITY-001`  
  Why now: 3-node GRANDPA finality is a stage-0 exit gate. The stalling behavior blocks multi-node production readiness. Operators cannot run production networks.  
  Codebase evidence: `WORKLIST.md:NET-FINALITY-001` documents stall with repro steps. Node logs show `Backing off claiming new slot for block authorship: finality is lagging`.  
  Owns: Chain spec, node service GRANDPA configuration, GRANDPA voter setup.  
  Integration touchpoints: `myosu-chain` node binary, devnet chain spec, GRANDPA finality gadget.  
  Scope boundary: Diagnose root cause (authority scheduling? voter set configuration?). Implement fix. Verify 3-node finality + 1-node-down tolerance.  
  Required tests: E2E test proving 2/3 quorum continues finalizing after one authority stops.  
  Dependencies: None (independent of other NEMs).  
  Completion signal: `tests/e2e/three_node_finality.sh` passes in CI with finalized height increasing after authority stop and restart.

- [ ] `NEM-002` Verify cross-node emission agreement across 3-node devnet

  Status (2026-04-05): Blocked on `NEM-001`. This pass did not add a new
  cross-node emission proof while the stage-0 multi-node finality contract is
  still scoped around an unattainable 2-of-3 expectation.

  Spec: `pallet-game-solver` emission mechanism  
  Why now: INV-003 and INV-005 require that emission accounting is deterministic across nodes. Currently only proven single-node. If nodes disagree on emission, Yuma Consensus breaks.  
  Codebase evidence: All epoch/coinbase tests run in single-node mock runtime. `pallet-game-solver/src/coinbase/run_coinbase.rs` uses U96F32 throughout.  
  Owns: E2E test extension comparing TotalIssuance and emission storage values across 3 nodes.  
  Integration touchpoints: RPC endpoints for reading storage, epoch mechanism, coinbase pipeline.  
  Scope boundary: Query storage via RPC on all 3 nodes after N epoch transitions. Assert bit-identical values for TotalIssuance, PendingServerEmission, PendingValidatorEmission.  
  Required tests: E2E script that queries storage on all 3 nodes and diffs.  
  Dependencies: NEM-001 (3-node devnet must work reliably).  
  Completion signal: Cross-node emission agreement test passes in CI with identical storage values across all nodes.


## Follow-On Work

### Operator Tooling and Onboarding

- [ ] `F-001` Fresh-machine operator bundle test

  Spec: `specs/050426-operator-tooling.md`
  Why now: The spec explicitly states "bundle has not been tested on a fresh machine outside CI." Operator onboarding cannot be trusted until this is verified.
  Codebase evidence: `.github/scripts/prepare_operator_network_bundle.sh`, `docs/operator-guide/quickstart.md`, CI `operator-network` job.
  Owns: Test procedure (Docker or VM) that runs the bundle from scratch and verifies miner+validator pair starts.
  Integration touchpoints: Bundle scripts, node binary, miner binary, validator binary, key management.
  Scope boundary: Test the existing bundle. Document failures. Do not rewrite the bundle.
  Acceptance criteria: (1) Bundle produces a running miner+validator pair on a clean Ubuntu 22.04 (or equivalent) with no pre-existing Rust toolchain (or documents exact prerequisites). (2) Failures are filed as concrete fix tasks.
  Verification: Run bundle on fresh Docker image, verify miner and validator produce expected report output.
  Required tests: The bundle test procedure itself.
  Dependencies: P-011 (multi-node devnet for realistic test).
  Estimated scope: M
  Completion signal: Bundle test passes or failures are documented as fix tasks.

- [ ] `F-002` Node restart resilience test

  Spec: `specs/050426-network-consensus.md`
  Why now: The spec lists "node restart resilience (catch-up without fork)" as unproven. Operators will restart nodes; this must work.
  Codebase evidence: No restart test exists in `tests/e2e/`.
  Owns: New E2E script testing node restart and catch-up.
  Integration touchpoints: Node binary, GRANDPA, block import.
  Scope boundary: Single node restart in a 3-node network. Verify it catches up to finalized head. Do not test simultaneous restart of all nodes.
  Acceptance criteria: (1) A restarted node catches up to the finalized head within a bounded time. (2) No fork occurs.
  Verification: E2E script.
  Required tests: The E2E script.
  Dependencies: P-011 (3-node devnet).
  Estimated scope: M
  Completion signal: Restart test passes in CI.

### Token Economics Research Gate

- [ ] `F-003` Token economics decision document

  Spec: `specs/050426-token-economics.md`
  Why now: The token-economics spec is explicitly a research spec, not an implementation spec. It identifies 8+ design axes (single vs dual token, AMM type, fee model, registration cost, emission schedule) that must be decided before `NoOpSwap` can be replaced. No implementation work should begin until this decision document exists.
  Codebase evidence: `crates/myosu-chain/pallets/swap-interface/src/lib.rs` (SwapEngine trait), `crates/myosu-chain/pallets/swap/` (V3 AMM implementation exists but is not wired into stage-0 runtime), `runtime/src/lib.rs` (Stage0NoopSwap).
  Owns: Decision document evaluating all design axes with recommendations.
  Integration touchpoints: Swap interface trait, swap pallet, runtime swap config, emission pipeline.
  Scope boundary: Research and document only. Do NOT change swap implementation. Do NOT wire V3 AMM into runtime.
  Acceptance criteria: (1) Each design axis from the spec has a concrete recommendation with rationale. (2) Migration path from NoOpSwap to chosen model is sketched. (3) The review requirement matches `specs/050426-token-economics.md`: the document must be reviewed by at least two contributors with token-economics context before this task closes.
  Verification: Review-based.
  Required tests: None (research task).
  Dependencies: P-002 (emission understanding informs economic model).
  Blocker (2026-04-05): `docs/adr/008-future-token-economics-direction.md` now records the repo-local recommendation, but the spec requires review by at least two contributors with context before `F-003` can be removed from the queue. That review is external to the current coding loop and is not yet recorded in-repo.
  Estimated scope: L
  Completion signal: Decision document exists, the multi-contributor review is recorded, and only then is `NoOpSwap` replacement work allowed to start.

### Miner Convergence Gate

- [ ] `F-007` Research: Minimum training iterations for meaningful strategy quality

  Spec: `specs/050426-mining-surface.md`
  Why now: The spec notes "no convergence gate exists" — a miner can train for 1 iteration and serve garbage. Validators score it low, but operators have no guidance on minimum training.
  Codebase evidence: `crates/myosu-miner/src/` training loop, `--train-iterations` flag. No convergence check or quality gate.
  Owns: Research document or code comment with recommended minimums per game type.
  Integration touchpoints: Miner CLI documentation, operator guide.
  Scope boundary: Measure and document. Do not enforce in code (that's follow-on).
  Acceptance criteria: (1) Minimum iterations for score > 0.5 documented for Poker and Liar's Dice. (2) Operator guide updated with recommendation.
  Verification: Run miner with varying iteration counts, measure validator score.
  Required tests: None (research task).
  Dependencies: P-009 (determinism verified across games).
  Blocker (2026-04-05): The poker half is not currently measurable with repo-owned stage-0 artifacts. The only checked-in bootstrap artifact path (`crates/myosu-games-poker/examples/bootstrap_artifacts.rs`) emits the same sparse single-lookup encoder that `crates/myosu-miner/src/training.rs` already tests as non-trainable: `run_training_batch_reports_sparse_encoder_failure_cleanly` proves `--train-iterations 1` fails with upstream `isomorphism not found`. Until richer poker encoder artifacts exist (or the abstraction pipeline work is pulled forward), this task cannot truthfully document a poker score-vs-iterations threshold.
  Estimated scope: S
  Completion signal: Minimum iterations documented per game type.

---

- [ ] `NEM-008` Document minimum training iteration recommendations

  Status (2026-04-05): Deferred. This pass prioritized executable correctness
  fixes and did not run the iteration-to-score measurements needed to publish a
  truthful operator recommendation.

  Spec: `crates/myosu-miner/src/training.rs`  
  Why now: A miner can train for 0 iterations and serve garbage. No guidance exists for minimum viable training.  
  Codebase evidence: `crates/myosu-miner/src/cli.rs` — `train_iterations` defaults to 0 with no minimum enforced.  
  Owns: Research document, CLI help text.  
  Integration touchpoints: Miner CLI documentation, operator guide.  
  Scope boundary: Measure validator scores at varying iteration counts. Document recommended minimums.  
  Required tests: None (research/documentation task).  
  Dependencies: None.  
  Completion signal: `docs/operator-guide/` documents recommended minimum iterations per game type.


## Completed / Already Satisfied

- [x] `C-001` NoOpSwap identity stub implements all 37 swap callsites
  Spec: `specs/050426-token-economics.md`
  Codebase evidence: `runtime/src/lib.rs` lines 89-150 define `Stage0NoopSwap` with 1:1 conversion, zero fees. `swap_stub.rs` documents `max_price()` returning `Balance::max_value()` (with new documentation from working tree).

- [x] `C-002` INV-004 solver-gameplay dependency boundary enforced in CI
  Spec: `specs/050426-game-trait-interface.md`
  Codebase evidence: Working tree adds `cargo tree` check to ci.yml verifying `myosu-play` does not depend on `myosu-miner` and vice versa. Confirmed via `cargo tree -p myosu-play` (no myosu-miner in tree).

- [x] `C-003` Multi-game architecture with zero-change extensibility
  Spec: `specs/050426-game-trait-interface.md`
  Codebase evidence: `GameRegistry::supported()` returns 4 games. Poker, Kuhn, Liar's Dice all implement `CfrGame`/`GameRenderer` traits. Adding Liar's Dice required zero changes to poker code.

- [x] `C-004` Workspace clippy lints enforced (arithmetic_side_effects, expect_used, indexing_slicing, unwrap_used)
  Spec: `specs/050426-ci-invariant-enforcement.md`
  Codebase evidence: Workspace `Cargo.toml` `[lints.clippy]` section. CI `chain-clippy` job runs with `-D warnings`.

- [x] `C-005` Validator scoring with hyperbolic formula and determinism test
  Spec: `specs/050426-validation-surface.md`
  Codebase evidence: Working tree changes `validation.rs` to `score = 1.0 / (1.0 + l1_distance)` with `score_from_l1_distance()` function. 14 unit tests. `validator_determinism.sh` in CI.

- [x] `C-006` Miner 7-step lifecycle (probe, register, serve, train, checkpoint, file-serve, HTTP-serve)
  Spec: `specs/050426-mining-surface.md`
  Codebase evidence: `crates/myosu-miner/src/lib.rs` (~2.2K lines) implements full lifecycle. `axon.rs` handles HTTP serving for poker. Training, checkpointing, and file-based serving work for all games.

- [x] `C-007` Gameplay surface with three modes (smoke-test, TUI, pipe)
  Spec: `specs/050426-gameplay-surface.md`
  Codebase evidence: `crates/myosu-play/src/entrypoint.rs` (~3.1K lines). `--smoke-test` flag, `train` subcommand (TUI), pipe protocol. CI runs smoke tests.

- [x] `C-008` Key management (`myosu-keys` crate with create, import, export, switch)
  Spec: `specs/050426-operator-tooling.md`
  Codebase evidence: `crates/myosu-keys/src/lib.rs` implements all documented commands. Network-namespaced storage.

- [x] `C-009` Two-node block sync proven
  Spec: `specs/050426-network-consensus.md`
  Codebase evidence: `tests/e2e/two_node_sync.sh` (8.7K) exists and is referenced in spec as proven.

- [x] `C-010` Aura + GRANDPA consensus configured with 4 chain spec variants
  Spec: `specs/050426-chain-runtime-pallet.md`
  Codebase evidence: Runtime `construct_runtime!` includes `pallet_aura` (3) and `pallet_grandpa` (4). Chain specs: `localnet`, `devnet`, `testnet`, `finney`.

- [x] `C-011` Epoch consistency guard (skip epoch on inconsistent state)
  Spec: `specs/050426-emission-epoch-mechanism.md`
  Codebase evidence: Working tree adds `is_epoch_input_state_consistent(netuid)` check at entry of both `epoch()` and `epoch_dense()` in `run_epoch.rs`. Two tests in `epoch.rs` verify the guard.

- [x] `C-012` Zero-dividend fallback distributes by stake weight
  Spec: `specs/050426-emission-epoch-mechanism.md`
  Codebase evidence: Working tree adds stake-weighted fallback in `run_coinbase.rs` `calculate_dividend_distribution` when `total_dividends == 0`. Test verifies 750/250 distribution for 3:1 stake ratio.

- [x] `C-013` Decode budget hardened to 1 MiB for poker wire codec
  Spec: `specs/050426-game-trait-interface.md`
  Codebase evidence: Working tree changes `MAX_DECODE_BYTES` from 256 MiB to 1 MiB in both `solver.rs` and `wire.rs`. Tests verify oversized payloads are rejected.

- [x] `C-014` Operator guide documentation exists
  Spec: `specs/050426-operator-tooling.md`
  Codebase evidence: `docs/operator-guide/quickstart.md`, `docs/operator-guide/upgrading.md`, `docs/operator-guide/troubleshooting.md`, `docs/operator-guide/architecture.md` all exist.

- [x] `C-015` Six environment variable contracts documented
  Spec: `specs/050426-operator-tooling.md`
  Codebase evidence: `MYOSU_KEY_PASSWORD`, `MYOSU_CONFIG_DIR`, `MYOSU_SUBNET`, `MYOSU_WORKDIR`, `MYOSU_CHAIN`, `MYOSU_OPERATOR_CHAIN` referenced in operator tooling spec and code.

- [x] `C-016` GameType on-chain encoding with proptest roundtrip
  Spec: `specs/050426-game-trait-interface.md`
  Codebase evidence: `GameType::from_bytes` / `to_bytes` in `myosu-games` crate with `#[non_exhaustive]` enum.

- [x] `C-017` Structured report types for miner, validator, and key management
  Spec: `specs/050426-validation-surface.md`
  Codebase evidence: 6 report types each in miner and validator (MINER, REGISTRATION, AXON, HTTP, TRAINING, STRATEGY / VALIDATOR, REGISTRATION, SUBTOKEN, PERMIT, VALIDATION, WEIGHTS).

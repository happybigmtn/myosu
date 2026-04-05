# IMPLEMENTATION_PLAN

**Generated:** 2026-04-05  
**Audit Basis:** Nemesis Pass 1 audit findings (nemesis/draft-nemesis-audit.md)  
**Task ID Prefix:** NEM-

---

## Priority Work

### Cluster: Consensus and Multi-Node Foundation

- [ ] `NEM-001` Fix GRANDPA finality stalling on 3-node devnet after authority stop

  Spec: `specs/050426-network-consensus.md`  
  Why now: 3-node GRANDPA finality is a Phase 1 gate. The stalling behavior blocks P-011 which blocks the entire multi-node production readiness milestone. Operators cannot run production networks.  
  Codebase evidence: `tests/e2e/three_node_finality.sh` reproduces finality freeze after authority stop with node logs showing `Backing off claiming new slot for block authorship: finality is lagging`. Chain specs in `crates/myosu-chain/node/src/devnet.rs` define 3-authority config. `WORKLIST.md:NET-FINALITY-001` documents this.  
  Owns: Chain spec, node service GRANDPA configuration, E2E test harness.  
  Integration touchpoints: `myosu-chain` node binary, devnet chain spec, GRANDPA finality gadget, CI `integration-e2e` job.  
  Scope boundary: Diagnose root cause (authority scheduling? voter configuration?). Implement fix. Verify 3-node finality + 1-node-down tolerance. Do not test network partitions beyond this scope.  
  Required tests: `tests/e2e/three_node_finality.sh` passes (2/3 quorum finalizes, catch-up works).  
  Dependencies: None (independent of other NEMs).  
  Completion signal: `bash tests/e2e/three_node_finality.sh` exits 0 in CI with finalized height increasing after authority stop and restart.

- [ ] `NEM-002` Verify cross-node emission agreement across 3-node devnet

  Spec: `specs/050426-emission-epoch-mechanism.md`  
  Why now: INV-003 and INV-005 require that emission accounting is deterministic across nodes. Currently only proven single-node. If nodes disagree on emission, Yuma Consensus breaks and validators disagree on weights.  
  Codebase evidence: All epoch/coinbase tests in `src/tests/epoch.rs` and `src/tests/stage_0_flow.rs` run in single-node mock runtime. `crates/myosu-chain/pallets/game-solver/src/coinbase/run_coinbase.rs` uses U96F32 throughout. `IMPLEMENTATION_PLAN.md:P-012` explicitly states "tested single-node only."  
  Owns: E2E test extension comparing TotalIssuance and emission storage values across 3 nodes.  
  Integration touchpoints: RPC endpoints for reading storage, epoch mechanism, coinbase pipeline, node binary.  
  Scope boundary: Query storage via RPC on all 3 nodes after N epoch transitions. Assert bit-identical. Do not test under adversarial conditions.  
  Required tests: E2E script that queries `TotalIssuance`, `PendingServerEmission`, `PendingValidatorEmission` on all 3 nodes and diffs.  
  Dependencies: NEM-001 (3-node devnet must work).  
  Completion signal: Cross-node emission agreement test passes in CI with identical storage values across all nodes.

---

### Cluster: Emission and Accounting Hardening

- [ ] `NEM-003` Tighten TotalIssuance accounting delta or document why 1000 RAO is acceptable

  Spec: `crates/myosu-chain/pallets/game-solver/src/utils/try_state.rs`  
  Why now: The 1000 RAO tolerance hides whether systematic drift is bounded. `EM-DUST-001` documents 2 rao/block truncation loss. This tolerance makes it impossible to distinguish bounded drift from unbounded accumulation.  
  Codebase evidence: `src/utils/try_state.rs:21` defines `let delta = 1000;`. `run_coinbase.rs` uses `tou64!` macro truncating U96F32 → u64, losing fractional RAO per block. `cargo test -p pallet-game-solver -- truncation` measures worst-case 2 rao/block.  
  Owns: `try_state.rs` delta configuration, potential increase of try-runtime checks frequency.  
  Integration touchpoints: `pallet-game-solver`, runtime migration, try-runtime feature.  
  Scope boundary: Either (a) tighten delta to match measured truncation bounds, or (b) document why 1000 RAO is an acceptable long-term bound for stage-0. Do not silently continue with the current tolerance without rationale.  
  Required tests: Extend truncation sweep test to verify delta is at least 2x worst-case measured drift.  
  Dependencies: None.  
  Completion signal: Delta is either (a) tightened to ≤ 10 RAO or (b) a documented rationale exists in `EM-DUST-001` for why 1000 RAO is stage-0 acceptable.

- [ ] `NEM-004` Emit event when epoch is skipped due to input state inconsistency

  Spec: `crates/myosu-chain/pallets/game-solver/src/epoch/run_epoch.rs`  
  Why now: Silent epoch skipping (returning `Vec::new()` on inconsistency) means operators and monitoring systems cannot detect when a subnet stops receiving emission. This was added as a safety guard (C-011) but lacks observability.  
  Codebase evidence: `src/epoch/run_epoch.rs:66-68` — `log::error!` exists but no `deposit_event`. `src/coinbase/run_coinbase.rs:306-310` — pending emissions accumulate but no alarm fires.  
  Owns: New `Event` variant in `game-solver`, `deposit_event` call in `run_epoch` and `run_coinbase`.  
  Integration touchpoints: `pallet-game-solver` events, chain RPC event subscription.  
  Scope boundary: Add event emission only. Do not change skip behavior.  
  Required tests: Unit test verifying event is deposited when `is_epoch_input_state_consistent` returns false.  
  Dependencies: None.  
  Completion signal: Epoch skip produces a chain event that monitoring systems can observe.

---

### Cluster: Validator Scoring Correctness

- [ ] `NEM-005` Verify L1 distance asymmetry in validator scoring is intentional and documented

  Spec: `crates/myosu-validator/src/validation.rs`, `specs/050426-validation-surface.md`  
  Why now: The current L1 distance implementation treats "missing from observed" and "unexpected in observed" asymmetrically. This could allow miners to game scoring by including zero-probability actions.  
  Codebase evidence: `src/validation.rs:345-367` — first pass penalizes missing from observed (up to 1.0 per action), second pass only counts unexpected actions if expected == 0.0 (effectively ignoring zero-probability game-playing actions).  
  Owns: Scoring implementation, spec documentation.  
  Integration touchpoints: `myosu-validator` scoring, spec validation surface.  
  Scope boundary: Either (a) document the asymmetry as intentional design, or (b) fix to symmetric L1 where all action set differences contribute to distance.  
  Required tests: Unit test for edge case where miner includes zero-probability unexpected actions.  
  Dependencies: None.  
  Completion signal: Either spec documents asymmetry as intentional or scoring is fixed to symmetric L1.

---

### Cluster: Security Hardening

- [ ] `NEM-007` Remove key passwords from CLI process arguments

  Spec: `crates/myosu-keys/src/lib.rs`, `crates/myosu-miner/src/cli.rs`, `crates/myosu-validator/src/cli.rs`  
  Why now: Key passwords in process arguments are visible to all users via `ps aux` and `/proc/$pid/cmdline`. On shared hosting, this is a credential exposure.  
  Codebase evidence: CLI uses `--key <uri>` pattern where URI may contain passwords. `load_active_secret_uri_from_env` is the correct pattern but not consistently enforced.  
  Owns: CLI argument parsing, key loading logic.  
  Integration touchpoints: `myosu-keys`, `myosu-miner`, `myosu-validator`.  
  Scope boundary: Require passwords via environment variable only. Reject URIs containing passwords. Add warning/error when password appears in arguments.  
  Required tests: Unit test verifying passwords in CLI args produce error.  
  Dependencies: None.  
  Completion signal: CLI rejects key URIs with embedded passwords, requires `MYOSU_KEY_PASSWORD` env var.

- [ ] `NEM-014` Add runtime INV-004 enforcement beyond cargo-dependency convention

  Spec: `crates/myosu-miner/`, `crates/myosu-play/`, INV-004  
  Why now: CI `cargo tree` check catches dependency violations at PR time, but runtime behavior is unguarded. A future refactor could accidentally bypass the convention without compile failure if both crates share a transitive dependency.  
  Codebase evidence: CI workflow runs `cargo tree` check. Crates have separate Cargo.toml but INV-004 says "runtime state or trust boundaries" must not cross.  
  Owns: Potential runtime assertion or feature flag in both binaries.  
  Integration touchpoints: `myosu-miner`, `myosu-play`, CI.  
  Scope boundary: Add a runtime check (e.g., asserting no shared state or logging boundary crossings). Do not over-engineer — the CI check may be sufficient.  
  Required tests: Demonstrate CI check still catches violations.  
  Dependencies: None.  
  Completion signal: Either runtime enforcement added or explicit decision documented that CI-only enforcement is stage-0 sufficient.

---

### Cluster: Documentation and Spec Consistency

- [ ] `NEM-010` Clarify INV-006 robopoker fork tracking — tag vs branch

  Spec: `INVARIANTS.md`, `docs/robopoker-fork-changelog.md`, `AGENTS.md`  
  Why now: INV-006 says "must track v1.0.0 as baseline" but AGENTS.md notes "fork uses branch, not upstream tag." This inconsistency confuses contributors auditing fork coherence.  
  Codebase evidence: `INVARIANTS.md:INV-006` — "The robopoker fork must track v1.0.0 as its baseline." `AGENTS.md` — "INV-006 says 'git tag v1.0.0' — fork uses branch, not upstream tag."  
  Owns: `INVARIANTS.md` INV-006 text, `docs/robopoker-fork-changelog.md` baseline documentation.  
  Integration touchpoints: Fork coherence documentation, INV-003 validator determinism.  
  Scope boundary: Update INV-006 to accurately describe current practice (branch pin + changelog) OR revert to tag tracking.  
  Required tests: None (documentation task).  
  Dependencies: None.  
  Completion signal: INV-006 text matches actual fork tracking practice.

- [ ] `NEM-011` Add 4-byte magic + version header to poker checkpoint format

  Spec: `crates/myosu-games-poker/src/solver.rs`, AGENTS.md key decision  
  Why now: AGENTS.md specifies "checkpoint versioning: 4-byte magic + version" as a key engineering decision to prevent silent corruption on format changes. The current implementation does not include this.  
  Codebase evidence: AGENTS.md — "checkpoint versioning: 4-byte magic + version." `crates/myosu-games-poker/src/solver.rs` — `save()` / `load()` use bincode without magic header.  
  Owns: `solver.rs` save/load implementation, version constants.  
  Integration touchpoints: `myosu-miner` checkpointing, `myosu-games-poker`.  
  Scope boundary: Add magic bytes + version to checkpoint header. Implement version migration path for existing checkpoints (graceful fail or auto-upgrade).  
  Required tests: Unit test verifying old-format checkpoints are rejected with clear error.  
  Dependencies: None.  
  Completion signal: Checkpoints include magic bytes. Loading old-format produces error, not silent corruption.

- [ ] `NEM-012` Restore or remove dangling @RTK.md reference in AGENTS.md

  Spec: `AGENTS.md`, `WORKLIST.md:DOC-OPS-001`  
  Why now: `WORKLIST.md:DOC-OPS-001` explicitly tracks this dangling reference. Future operator loops cannot consume the contract.  
  Codebase evidence: `WORKLIST.md:DOC-OPS-001` — "Resolve the dangling @RTK.md reference at the top of AGENTS.md." No `RTK.md` in repository.  
  Owns: `AGENTS.md`.  
  Integration touchpoints: Operator loop surfaces.  
  Scope boundary: Either restore the file or remove the reference from AGENTS.md.  
  Required tests: None (documentation task).  
  Dependencies: None.  
  Completion signal: AGENTS.md contains no dangling references.

---

## Follow-On Work

- [ ] `NEM-006` Audit SwapInterface NoOp stub max_price bound for future compatibility

  Spec: `crates/myosu-chain/runtime/src/lib.rs`, `crates/myosu-chain/pallets/swap-interface/src/lib.rs`  
  Why now: Stage-0 uses `Stage0NoopSwap` with `Balance::max_value()` for max_price. If future code assumes a finite bound for safety checks, this will fail open.  
  Codebase evidence: `runtime/src/lib.rs:89-150` defines `Stage0NoopSwap`. AGENTS.md notes "SwapInterface no-op stub (1:1 identity)" as intentional. `pallet-game-solver/src/lib.rs` calls `T::SwapInterface::stage0_max_price()`.  
  Owns: Stub implementation, any code paths depending on max_price.  
  Integration touchpoints: Emission calculation, swap pallet.  
  Scope boundary: Audit all callers of `stage0_max_price()`. Document whether they handle max_value correctly. Add runtime assertion if they don't.  
  Required tests: Unit tests for all max_price callers with max_value input.  
  Dependencies: None (can be done in parallel with other NEMs).  
  Completion signal: All `max_price()` callers handle unbounded return correctly or are documented as unsafe for NoOpSwap.

- [ ] `NEM-008` Document and test GRANDPA recovery after network partition

  Spec: `specs/050426-network-consensus.md`  
  Why now: NEM-001 fixes stalling but does not prove recovery after partition. Operators need to understand expected behavior when 2/3 quorum is temporarily lost.  
  Codebase evidence: `WORKLIST.md:NET-FINALITY-001` — "Investigate why GRANDPA finality stalls after one authority is stopped."  
  Owns: Recovery test, operator documentation.  
  Integration touchpoints: Node service, GRANDPA gadget, operator guide.  
  Scope boundary: Document expected behavior. Add E2E test for recovery path if feasible.  
  Required tests: Documentation update. Optional: E2E recovery test.  
  Dependencies: NEM-001 (fix must land first).  
  Completion signal: `docs/operator-guide/` documents expected behavior for 1-of-3, 2-of-3, 3-of-3 scenarios.

- [ ] `NEM-009` Apply decode budget hardening to Liar's Dice wire format

  Spec: `crates/myosu-games-liars-dice/src/`, `specs/050426-game-trait-interface.md`  
  Why now: Poker wire codec was hardened to 1 MiB (`C-013`). Liar's Dice wire format may not have equivalent bounds, creating an inconsistent attack surface.  
  Codebase evidence: `crates/myosu-games-poker/src/wire.rs:8` — `MAX_DECODE_BYTES: u64 = 1_048_576`. `crates/myosu-games-liars-dice/` — `decode_strategy_query` / `decode_strategy_response` not verified for bounds.  
  Owns: Liar's Dice wire implementation.  
  Integration touchpoints: `myosu-games-liars-dice`, validator query path.  
  Scope boundary: Add consistent decode budget to Liar's Dice. Verify Kuhn poker also has bounds.  
  Required tests: Unit test verifying oversized Liar's Dice payloads are rejected.  
  Dependencies: None.  
  Completion signal: All game wire formats have consistent decode budgets.

- [ ] `NEM-013` Research minimum training iterations for meaningful strategy quality

  Spec: `specs/050426-mining-surface.md`, `crates/myosu-miner/src/training.rs`  
  Why now: A miner can train for 1 MCCFR iteration and serve garbage. Validators score it near zero, but the system has no guidance on minimum training.  
  Codebase evidence: `crates/myosu-miner/src/training.rs` accepts `--train-iterations` without minimum. `F-007` in `IMPLEMENTATION_PLAN.md` is follow-on but not started.  
  Owns: Research document, optional CLI guidance.  
  Integration touchpoints: Miner CLI documentation, operator guide.  
  Scope boundary: Measure validator scores at varying iteration counts. Document minimums per game type.  
  Required tests: None (research task).  
  Dependencies: None.  
  Completion signal: `docs/operator-guide/quickstart.md` or miner `--help` documents recommended minimum iterations per game.

---

## Completed / Already Satisfied

- [x] `NEM-C01` Decode budget hardened to 1 MiB for poker wire codec (C-013)
  Spec: `crates/myosu-games-poker/src/wire.rs`
  Evidence: `MAX_DECODE_BYTES: u64 = 1_048_576`. Tests verify oversized payloads are rejected.

- [x] `NEM-C02` Epoch consistency guard prevents corruption on inconsistent state (C-011)
  Spec: `crates/myosu-chain/pallets/game-solver/src/epoch/run_epoch.rs`
  Evidence: `is_epoch_input_state_consistent()` added. Tests verify duplicate hotkeys trigger skip.

- [x] `NEM-C03` INV-004 solver-gameplay separation enforced in CI (C-002)
  Spec: `ci.yml` cargo tree check
  Evidence: `cargo tree` check verifies no path from myosu-play → myosu-miner or vice versa.

- [x] `NEM-C04` Validator scoring hyperbolic formula implemented and tested (C-005)
  Spec: `crates/myosu-validator/src/validation.rs`
  Evidence: `score = 1.0 / (1.0 + l1_distance)`. 14 unit tests including `inv_003_determinism`.

- [x] `NEM-C05` Two-node block sync proven (C-009)
  Spec: `tests/e2e/two_node_sync.sh`
  Evidence: Script exists and is referenced in network-consensus spec as proven.

- [x] `NEM-C06` GRANDPA + Aura consensus configured for 4 chain spec variants (C-010)
  Spec: `crates/myosu-chain/runtime/src/lib.rs`
  Evidence: `pallet_aura` (3) and `pallet_grandpa` (4) in `construct_runtime!`. Chain specs: localnet, devnet, testnet, finney.

- [x] `NEM-C07` Zero-dividend fallback distributes by stake weight (C-012)
  Spec: `crates/myosu-chain/pallets/game-solver/src/coinbase/run_coinbase.rs`
  Evidence: `calculate_dividend_distribution` adds stake-weighted fallback when `total_dividends == 0`.

- [x] `NEM-C08` SwapInterface NoOp stub implements all 37 swap callsites (C-001)
  Spec: `crates/myosu-chain/runtime/src/lib.rs`
  Evidence: `Stage0NoopSwap` with 1:1 conversion, zero fees. `max_price()` documented as returning `Balance::max_value()`.

---

*This document is a draft artifact for Nemesis Pass 1 review. Tasks are evidence-backed and execution-ready. Open questions require Pass 2 investigation.*

# IMPLEMENTATION_PLAN

**Generated:** 2026-04-05  
**Audit Basis:** Nemesis synthesis pass (nemesis/nemesis-audit.md)  
**Task ID Prefix:** NEM-

---

## Priority Work

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

- [x] `NEM-003` Tighten TotalIssuance accounting delta or document rationale

  Satisfied (2026-04-05): The stage-0 `try_state` delta is now explicitly
  documented as an alert threshold, not a correctness bound, and default-build
  tests pin the measured drift against that threshold.

  Spec: `pallet-game-solver/src/utils/try_state.rs`  
  Why now: The 1000 RAO tolerance hides whether systematic drift is bounded. `EM-DUST-001` documents 2 rao/block truncation loss.  
  Codebase evidence: `src/utils/try_state.rs:21` defines `let delta = 1000;`. `run_coinbase.rs` uses `tou64!` macro truncating U96F32 → u64.  
  Owns: `try_state.rs` delta configuration, documentation.  
  Integration touchpoints: `pallet-game-solver`, runtime migration, try-runtime feature.  
  Scope boundary: Either (a) tighten delta to ≤ 10 RAO based on measured truncation bounds, or (b) document why 1000 RAO is stage-0 acceptable in `EM-DUST-001`.  
  Required tests: Extend truncation sweep test to verify delta is at least 5x worst-case measured drift.  
  Dependencies: None.  
  Completion signal: Delta tightened with tests passing, or documented rationale committed to WORKLIST.md `EM-DUST-001`.

- [x] `NEM-004` Emit event when epoch is skipped due to input state inconsistency

  Satisfied (2026-04-05): `EpochSkipped { netuid, reason }` now lands on-chain
  when the epoch guard rejects inconsistent input state, and a regression test
  proves the event surface.

  Spec: `pallet-game-solver/src/epoch/run_epoch.rs`  
  Why now: Silent epoch skipping means operators cannot detect when a subnet stops receiving emission.  
  Codebase evidence: `src/epoch/run_epoch.rs:66-68` — `log::error!` exists but no `deposit_event`. Grep confirms no `EpochSkipped` event variant.  
  Owns: New `Event` variant in `game-solver`, `deposit_event` call in `run_epoch`.  
  Integration touchpoints: `pallet-game-solver` events, chain RPC event subscription.  
  Scope boundary: Add `EpochSkipped { netuid, reason }` event. Deposit event when `is_epoch_input_state_consistent` returns false.  
  Required tests: Unit test verifying event is deposited when consistency check fails.  
  Dependencies: None.  
  Completion signal: Epoch skip produces a chain event observable via RPC.

- [x] `NEM-005` Document L1 distance asymmetry or fix to symmetric

  Satisfied (2026-04-05): Validator scoring now computes symmetric L1 over the
  union of expected and observed actions, with regressions covering explicit
  zero-weight action entries.

  Spec: `crates/myosu-validator/src/validation.rs`  
  Why now: The current L1 distance implementation treats "missing from observed" and "unexpected in observed" asymmetrically via the `expected == 0.0` check.  
  Codebase evidence: `src/validation.rs` — second pass only counts unexpected actions if `expected.probability_for(action) == 0.0`.  
  Owns: Scoring implementation documentation or fix.  
  Integration touchpoints: `myosu-validator` scoring, miner scoring fairness.  
  Scope boundary: Either (a) document the asymmetry as intentional design in code comments, or (b) fix to symmetric L1 where all action set differences contribute equally.  
  Required tests: Unit test for edge case where miner includes unexpected actions.  
  Dependencies: None.  
  Completion signal: Code comment documents asymmetry rationale, or scoring fixed to symmetric L1.

- [x] `NEM-006` Document SwapInterface NoOp stub max_price behavior

  Satisfied (2026-04-05): `Stage0NoopSwap` rustdoc now states that
  `max_price()` intentionally returns an unbounded value in the stage-0
  identity-swap runtime.

  Spec: `crates/myosu-chain/runtime/src/lib.rs`  
  Why now: Stage-0 uses `Stage0NoopSwap` with `C::MAX` for max_price. Future code may assume finite bounds.  
  Codebase evidence: `runtime/src/lib.rs:89-150` defines `Stage0NoopSwap` with `fn max_price<C: Currency>() -> C { C::MAX }`.  
  Owns: Runtime documentation, swap interface comments.  
  Integration touchpoints: Emission calculation, swap pallet interface.  
  Scope boundary: Add rustdoc comments to `Stage0NoopSwap` documenting that `max_price()` returns unbounded value intentionally.  
  Required tests: None (documentation task).  
  Dependencies: None.  
  Completion signal: `Stage0NoopSwap` rustdoc explains max_price behavior and stage-0 context.

- [x] `NEM-007` Align Liar's Dice decode budget with poker

  Satisfied (2026-04-05): Liar's Dice now uses the same 1 MiB decode budget as
  poker, with oversized-payload regressions for wire responses and checkpoints.

  Spec: `crates/myosu-games-liars-dice/src/wire.rs`  
  Why now: Poker wire codec uses 1 MiB budget. Liar's Dice uses 256 MiB — inconsistent attack surface.  
  Codebase evidence: `liars-dice/src/wire.rs` — `MAX_DECODE_BYTES: u64 = 256 * 1024 * 1024`. `poker/src/wire.rs` — `MAX_DECODE_BYTES: u64 = 1_048_576`.  
  Owns: Liar's Dice wire implementation.  
  Integration touchpoints: `myosu-games-liars-dice`, validator query path.  
  Scope boundary: Reduce Liar's Dice `MAX_DECODE_BYTES` to 1 MiB to match poker. Update tests.  
  Required tests: Unit test verifying oversized Liar's Dice payloads are rejected.  
  Dependencies: None.  
  Completion signal: Both poker and Liar's Dice use identical 1 MiB decode budgets.

---

## Follow-On Work

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

- [x] `NEM-009` Evaluate INV-004 runtime enforcement necessity

  Satisfied (2026-04-05): ADR 010 records the explicit stage-0 decision that
  compile-time CI and invariant-test enforcement are the truthful INV-004
  boundary today.

  Spec: `crates/myosu-miner/`, `crates/myosu-play/`  
  Why now: CI `cargo tree` check catches dependency violations at PR level, but runtime enforcement is unverified.  
  Codebase evidence: CI workflow runs `cargo tree` check. No runtime assertion exists.  
  Owns: Potential runtime assertion or documented decision.  
  Integration touchpoints: CI, runtime architecture decision records.  
  Scope boundary: Either (a) add runtime check logging boundary crossings, or (b) document explicit decision that CI-only enforcement is stage-0 sufficient.  
  Required tests: Demonstrate CI check still catches violations.  
  Dependencies: None.  
  Completion signal: Runtime enforcement added, or explicit decision documented.

- [x] `NEM-010` Close WORKLIST.md DOC-OPS-001 if stale

  Satisfied (2026-04-05): `WORKLIST.md` no longer tracks `DOC-OPS-001` after
  confirming the live repo `AGENTS.md` file has no `@RTK.md` reference.

  Spec: `WORKLIST.md:DOC-OPS-001`  
  Why now: WORKLIST tracks a dangling `@RTK.md` reference but AGENTS.md no longer contains it.  
  Codebase evidence: `WORKLIST.md:DOC-OPS-001` tracks reference. AGENTS.md verified to not contain `@RTK.md`.  
  Owns: WORKLIST.md maintenance.  
  Integration touchpoints: Documentation tracking.  
  Scope boundary: Remove `DOC-OPS-001` entry from WORKLIST.md if confirmed resolved, or locate and fix the reference if it moved.  
  Required tests: None.  
  Dependencies: None.  
  Completion signal: `DOC-OPS-001` resolved or removed from WORKLIST.md.

---

## Completed / Already Satisfied

- [x] `NEM-C01` Checkpoint magic header implemented  
  Spec: `crates/myosu-games-poker/src/solver.rs`  
  Evidence: `CHECKPOINT_MAGIC: [u8; 4] = *b"MYOS"` and `CHECKPOINT_VERSION: u32 = 1` defined. `save()`/`load()` implement versioned header. Draft NEM-011 was incorrect.

- [x] `NEM-C02` Epoch consistency guard prevents corruption  
  Spec: `pallet-game-solver/src/epoch/run_epoch.rs`  
  Evidence: `is_epoch_input_state_consistent()` returns false on duplicate hotkeys. Epoch is skipped (though silently — see NEM-004).

- [x] `NEM-C03` INV-004 separation enforced in CI  
  Spec: `.github/workflows/ci.yml`  
  Evidence: `cargo tree` check verifies no path from myosu-play → myosu-miner or vice versa.

- [x] `NEM-C04` Validator scoring hyperbolic formula implemented  
  Spec: `crates/myosu-validator/src/validation.rs`  
  Evidence: `score = 1.0 / (1.0 + l1_distance)`. Unit tests include `inv_003_determinism`.

- [x] `NEM-C05` Key password uses environment variable pattern  
  Spec: `crates/myosu-keys/src/storage.rs`  
  Evidence: `load_active_secret_uri_from_env()` requires password from env var. Draft NEM-007 (password in CLI args) was incorrect for current implementation.

---

*This plan represents the final Nemesis synthesis pass. Tasks are execution-ready, bounded, and grounded in verified codebase evidence.*

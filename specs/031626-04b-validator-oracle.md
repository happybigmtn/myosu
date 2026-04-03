# Specification: Validator Oracle — Exploitability-Based Miner Scoring

Source: Master spec AC-VL-01, chain pallet + poker engine analysis
Status: Draft
Date: 2026-03-30
Depends-on: GS-01..09 (pallet accepts weights), PE-01..04 (exploitability computable), MN-01..05 (miners serve axons)

## Purpose

Build the `myosu-validator` binary — a long-running process that stakes on a
subnet, periodically queries each miner's strategy via their axon endpoints,
computes exploitability scores, and submits weight vectors to the chain for
Yuma Consensus processing.

The validator is the quality control layer. It converts miner strategies into
objective scores that Yuma uses to allocate emissions. The exploitability oracle
is the core innovation: a deterministic, reproducible measurement of how far
a strategy is from Nash equilibrium.

**Key design constraint**: validator scoring must be deterministic (INV-003).
Two validators evaluating the same miner strategy on the same test positions
must produce identical scores within floating-point tolerance.

## Whole-System Goal

Current state:
- Chain pallet and shared client already support the stage-0 weight path
- Poker scoring and checkpoint-backed query surfaces are already live
- Miners already expose the stage-0 HTTP axon surface
- `myosu-validator` exists today, but it is still a bootstrap-oriented
  validator rather than the full periodic multi-miner oracle loop described by
  this spec

This spec defines the remaining work to turn the existing validator bootstrap
crate into the fuller long-running oracle described here:
- periodic miner discovery and evaluation
- more explicit deterministic position generation
- cleaner scoring and submission decomposition
- a stronger end-to-end validator loop beyond the current stage-0 bootstrap

If all ACs land:
- `myosu-validator --chain ws://localhost:9944 --subnet 1 --key //Bob` starts evaluating miners
- Each miner receives an exploitability-based score
- Weight vector submitted to chain every tempo
- Commit-reveal hides test positions from miners
- Yuma Consensus produces correct emissions based on validator weights

Still not solved here:
- Multi-game validation (non-poker)
- Distributed validation across multiple machines
- Validator reputation or slashing for dishonest scoring
- Advanced anti-gaming (adversarial test position selection)

12-month direction:
- Game-specific validation plugins per subnet
- Reputation system penalizing inconsistent validators
- Adversarial test position generation via reinforcement learning

## Why This Spec Exists As One Unit

- Evaluation, scoring, weight submission, and commit-reveal form a single
  operational loop — a validator that evaluates but doesn't submit weights
  earns nothing
- Determinism (INV-003) requires the evaluation and scoring to be tested together
- The commit-reveal flow is specifically designed for game solving anti-gaming

## Scope

In scope:
- CLI binary with chain connection
- Miner discovery via on-chain Axon registry
- HTTP client for querying miner axons
- Exploitability computation per miner
- Score normalization to weight vector (u16 values)
- Weight submission (set_weights and commit-reveal)
- Deterministic test position generation
- Staking via add_stake extrinsic

Out of scope:
- Multi-game support — poker only for bootstrap
- Slashing for dishonest validation
- Distributed evaluation across machines
- Advanced anti-gaming strategies
- Monitoring dashboard

## Current State

- `crates/myosu-validator/` already exists with `cli.rs`, `chain.rs`, and
  `validation.rs`
- The current binary can probe the chain, register, ensure validator permit
  readiness, score checkpoint-backed strategy responses, and submit stage-0
  weights
- The missing gap is not crate creation; it is the fuller always-on
  multi-miner evaluation loop and the more decomposed module structure
  described by this spec

## What Already Exists

| Sub-problem | Existing code / flow | Reuse / extend / replace | Why |
|-------------|----------------------|--------------------------|-----|
| CLI | `crates/myosu-validator/src/cli.rs` | extend | Bootstrap flags already exist |
| Chain client | `crates/myosu-validator/src/chain.rs` + `myosu-chain-client` | extend | Registration, permit, and weight submission already exist |
| Scoring | `crates/myosu-validator/src/validation.rs` | extend | Checkpoint-backed validation and reporting already exist |
| Weight submission | `crates/myosu-validator/src/chain.rs` | extend | Stage-0 weight path already exists |
| Miner discovery / live querying | `crates/myosu-play/src/discovery.rs` and miner HTTP surface as reference | extend | The repo already has a proven live-query seam to build on |

## Ownership Map

| Component | Status | Location |
|-----------|--------|----------|
| CLI + main | Live | crates/myosu-validator/src/main.rs, crates/myosu-validator/src/cli.rs |
| Chain client | Live | crates/myosu-validator/src/chain.rs + crates/myosu-chain-client/ |
| Validation and scoring | Partially live | crates/myosu-validator/src/validation.rs |
| Position generation | Draft future split | currently folded into validation design, not a dedicated module yet |
| Weight submitter | Partially live | crates/myosu-validator/src/chain.rs |

---

## Acceptance Criteria

### AC-VO-01: CLI and Chain Registration

- Where: `crates/myosu-validator/src/main.rs`, `crates/myosu-validator/src/chain.rs`
- How: Binary with clap CLI:
  ```
  myosu-validator --chain ws://localhost:9944 --subnet 1 --key //Bob --stake 10000
  ```
  On startup: connect to chain, register neuron if not already registered,
  stake tokens via `add_stake`, verify validator permit received.
- Required tests:
  - `cargo test -p myosu-validator chain::tests::register_and_stake`
  - `cargo test -p myosu-validator chain::tests::verify_validator_permit`
- Pass/fail:
  - Registers on subnet and receives UID; stakes tokens; has ValidatorPermit after epoch
  - Insufficient stake → no ValidatorPermit → clear warning log
- Blocking note: validators must be registered and staked to submit weights.
- Rollback condition: ValidatorPermit requires more stake than available.

### AC-VO-02: Miner Discovery and Query

- Where: future split from `crates/myosu-validator/src/validation.rs`
- How: Query the chain for all registered miners (neurons with axon endpoints)
  on the subnet. For each miner:
  1. Read `Axons[subnet][hotkey]` to get IP:port
  2. `GET /health` to verify miner is alive
  3. Generate test positions (see VO-03)
  4. `POST /strategy` with each test position
  5. Collect action distributions
  Timeout: 5 seconds per query. Skip unresponsive miners (score = 0).
- Required tests:
  - `cargo test -p myosu-validator evaluator::tests::discover_miners`
  - `cargo test -p myosu-validator evaluator::tests::query_miner_strategy`
  - `cargo test -p myosu-validator evaluator::tests::timeout_unresponsive_miner`
- Pass/fail:
  - Discovers all miners with axon endpoints on the subnet
  - Valid strategy response collected for responsive miners
  - Unresponsive miner (timeout) gets score 0, doesn't block others
  - Invalid response (malformed JSON) → score 0
- Blocking note: without miner queries, the validator has nothing to score.
- Rollback condition: miner axon protocol incompatible with validator client.

### AC-VO-03: Deterministic Test Position Generation

- Where: future split from `crates/myosu-validator/src/validation.rs`
- How: Generate a set of N test positions (game states) for miner evaluation.
  Positions must be:
  1. **Deterministic** given a seed — same seed → same positions (INV-003)
  2. **Varied** — cover preflop, flop, turn, river decisions
  3. **Representative** — weighted toward strategically important spots

  ```rust
  pub fn generate_test_positions(seed: u64, count: usize) -> Vec<NlheInfo> {
      let mut rng = SmallRng::seed_from_u64(seed);
      (0..count)
          .map(|_| random_game_state(&mut rng))
          .map(|game| encoder.seed(&game))  // convert to info set
          .collect()
  }
  ```

  The seed is derived from: `hash(subnet_id, epoch_number, validator_hotkey)`.

  **Encoder pinning for INV-003**: All validators must use identical
  abstraction tables (NlheEncoder state). Pin the encoder to a versioned,
  hash-checked artifact. On startup, log the encoder hash. The INV-003
  integration test (milestone 7) must use independently-initialized
  validators to catch encoder divergence.
  This ensures: same validator produces same positions each epoch (deterministic),
  different validators produce different positions (diverse coverage),
  positions change each epoch (no memorization).
- Required tests:
  - `cargo test -p myosu-validator positions::tests::deterministic_same_seed`
  - `cargo test -p myosu-validator positions::tests::different_seeds_different_positions`
  - `cargo test -p myosu-validator positions::tests::covers_all_streets`
  - `cargo test -p myosu-validator positions::tests::positions_are_valid_info_sets`
- Pass/fail:
  - Same seed produces identical position set (byte-level equality)
  - Different seeds produce different position sets
  - At least 20% of positions are on each street (preflop, flop, turn, river)
  - All generated positions are valid NlheInfo with non-empty choices
- Blocking note: deterministic positions are required for INV-003.
- Rollback condition: encoder requires pre-loaded abstraction tables that aren't available.

### AC-VO-04: Exploitability Scoring

- Where: future split from `crates/myosu-validator/src/validation.rs`
- How: For each miner, compute a score from their strategy responses:

  ```rust
  pub fn score_miner(
      responses: &[(NlheInfo, Vec<(NlheEdge, Probability)>)],
      encoder: &NlheEncoder,
  ) -> u16 {
      // Build synthetic profile from miner responses
      let exploit = remote_poker_exploitability(
          |info| lookup_response(responses, info),
          encoder,
      );
      // Convert exploitability to weight (lower exploit = higher weight)
      // Baseline: 1000 mbb/h (random strategy)
      let normalized = 1.0 - (exploit / BASELINE_EXPLOITABILITY).min(1.0);
      (normalized * u16::MAX as f64) as u16
  }
  ```

  Scoring formula: `weight = max(0, 1 - exploit/baseline) * 65535`. A perfect
  Nash strategy gets weight 65535, a random strategy gets ~0.
- Required tests:
  - `cargo test -p myosu-validator scoring::tests::nash_strategy_max_weight`
  - `cargo test -p myosu-validator scoring::tests::random_strategy_low_weight`
  - `cargo test -p myosu-validator scoring::tests::unresponsive_miner_zero`
  - `cargo test -p myosu-validator scoring::tests::scoring_is_deterministic`
- Pass/fail:
  - Nash-like strategy → weight near 65535
  - Random strategy → weight near 0
  - Unresponsive miner → weight exactly 0
  - Same responses + same positions → identical score (INV-003)
- Blocking note: the scoring function is Yuma's input. Bad scores → bad incentives.
- Rollback condition: exploitability computation is too slow for per-epoch evaluation.

### AC-VO-05: Weight Submission (Direct and Commit-Reveal)

- Where: future split from `crates/myosu-validator/src/chain.rs`
- How: After scoring all miners, construct and submit the weight vector:

  **Direct mode** (simpler, for devnet):
  ```rust
  let weights: Vec<(u16, u16)> = miners.iter()
      .map(|m| (m.uid, score_miner(&m.responses, &encoder)))
      .collect();
  chain.set_weights(subnet_id, weights).await?;
  ```

  **Commit-reveal mode** (for production, anti-gaming):
  1. Generate test positions (VO-03) — don't send them yet
  2. Hash the position set + random salt
  3. Submit `commit_weights(subnet_id, hash)`
  4. Wait for reveal window (next tempo)
  5. Query miners with the committed positions
  6. Score miners (VO-04)
  7. Submit `reveal_weights(subnet_id, uids, values, salt)`

  **Clarification on commit-reveal**: The on-chain commit-reveal mechanism
  hides the **weight vector** from other validators (preventing weight
  copying — the standard Bittensor anti-gaming mechanism). It does NOT hide
  test positions from miners — miners see queries in real-time at step 5.
  Miner gaming of specific positions is mitigated by: (a) large, varied test
  position sets that change each epoch, and (b) the validator choosing
  positions deterministically from a seed miners can't predict.

  Evaluation cadence: once per tempo (every `Tempo[subnet]` blocks).
  **Miners should be queried concurrently** (`futures::join_all` or bounded
  `FuturesUnordered`) to avoid sequential round-trip overhead. Set a hard
  deadline at `tempo * 0.8` blocks — if exceeded, submit partial weights
  (scored miners get computed weight, unscored get 0).
- Required tests:
  - `cargo test -p myosu-validator submitter::tests::submit_direct_weights`
  - `cargo test -p myosu-validator submitter::tests::commit_reveal_flow`
  - `cargo test -p myosu-validator submitter::tests::weights_sum_valid`
- Pass/fail:
  - Direct weight submission succeeds and is visible on-chain
  - Commit-reveal flow: commit → wait → query → score → reveal succeeds
  - All weights are u16 values
  - Weight vector contains entry for every active miner
  - Empty subnet (no miners) → no submission
- Blocking note: without weight submission, Yuma has no input.
- Rollback condition: commit-reveal timing window misaligned with evaluation loop.

### AC-VO-06: Evaluation Loop Orchestration

- Where: `crates/myosu-validator/src/main.rs (extend)`
- How: Main loop that runs the full evaluation cycle:
  ```rust
  loop {
      let current_block = chain.current_block().await;
      let tempo = chain.tempo(subnet_id).await;
      if current_block % tempo == 0 {
          let miners = discover_miners(&chain, subnet_id).await;
          let positions = generate_test_positions(seed, N_POSITIONS);
          let mut weights = Vec::new();
          for miner in &miners {
              let responses = query_miner(miner, &positions).await;
              let score = score_miner(&responses, &encoder);
              weights.push((miner.uid, score));
          }
          submit_weights(&chain, subnet_id, &weights).await?;
          log::info!("Submitted weights for {} miners", weights.len());
      }
      sleep(Duration::from_secs(6)).await;  // one block
  }
  ```
  Includes: retry logic for failed queries, timeout handling, graceful shutdown.
- Required tests:
  - `cargo test -p myosu-validator main::tests::evaluation_loop_completes`
  - `cargo test -p myosu-validator main::tests::handles_miner_failure`
  - `cargo test -p myosu-validator main::tests::graceful_shutdown`
- Pass/fail:
  - Full evaluation cycle completes within tempo period
  - Failed miner query doesn't block other evaluations
  - SIGINT causes clean shutdown with log message
  - Log shows weight submission confirmation
- Blocking note: the orchestration loop ties all pieces together.
- Rollback condition: evaluation takes longer than tempo period.

### AC-VO-07: Two-Validator INV-003 Agreement Test

- Where: `crates/myosu-validator/tests/determinism.rs (new)`
- How: Integration test that validates INV-003 (game verification determinism)
  end-to-end by running two independently initialized validator instances
  against the same miner:

  1. Start devnet with one miner (trained for 100 iterations)
  2. Initialize validator A with fresh state, seed from epoch
  3. Initialize validator B with fresh state, same seed derivation
  4. Both query the same miner with positions generated from same seed
  5. Both compute exploitability scores
  6. Assert: `|score_a - score_b| < 1e-6`
  7. Additionally verify: both used identical encoder hash (AP-02 pinning)

  This test catches:
  - Non-deterministic RNG in position generation
  - Floating-point ordering differences
  - Encoder hash mismatches between validators
  - Race conditions in miner query responses

- Whole-system effect: proves the most important property of the system —
  that independent validators converge on the same truth about miner quality.
  Without this, Yuma Consensus is meaningless.
- State: two validator instances + one miner + devnet.
- Wiring contract:
  - Trigger: `cargo test -p myosu-validator determinism`
  - Callsite: tests/determinism.rs
  - State effect: two complete evaluation cycles
  - Persistence effect: N/A
  - Observable signal: score equality assertion passes
- Required tests:
  - `cargo test -p myosu-validator determinism::tests::two_validators_agree`
  - `cargo test -p myosu-validator determinism::tests::different_encoder_hash_detected`
- Pass/fail:
  - Two validators produce identical scores within epsilon (1e-6) for same miner
  - Two validators generate identical test positions from same seed
  - Validator with wrong encoder hash produces different scores (test validates
    that the pinning mechanism works by showing what happens when it breaks)
  - Test completes within 60 seconds
- Blocking note: this is the single most important correctness property. OS.md
  lists validator determinism as a no-ship condition. The pieces exist (VO-03
  deterministic positions, VO-04 deterministic scoring) but no test proves two
  independent instances agree.
- Rollback condition: floating-point non-determinism in the exploitability
  computation that cannot be resolved with canonical ordering.

---

## Decision Log

- 2026-03-16: Exploit-to-weight formula `1 - exploit/baseline` — simple,
  monotonic, maps [0, baseline] to [65535, 0]. Can tune baseline per game.
- 2026-03-16: Seed from `hash(subnet, epoch, hotkey)` — ensures determinism
  per validator while varying across validators for coverage.
- 2026-03-16: 6-second poll interval — matches block time, minimal overhead.
- 2026-03-16: Both direct and commit-reveal modes — direct for development
  simplicity, commit-reveal for production anti-gaming.
- 2026-03-17: VO-07 added — two-validator agreement test for INV-003. Discovered
  during pre-implementation audit that no existing AC validates end-to-end
  determinism across independent validator instances.

## Milestone Verification

| # | Scenario | Validates | ACs |
|---|----------|-----------|-----|
| 1 | Validator registers and stakes on devnet | Registration | VO-01 |
| 2 | Discovers miners and queries their axons | Discovery | VO-02 |
| 3 | Same seed → same test positions | Determinism | VO-03 |
| 4 | Trained miner scores higher than random | Scoring | VO-04 |
| 5 | Weights submitted and visible on-chain | Submission | VO-05 |
| 6 | Full loop: discover → query → score → submit in one tempo | Integration | VO-06 |
| 7 | Two validators produce same score for same miner (INV-003) | Determinism | VO-07 |
| 8 | Wrong encoder hash → different scores (negative control) | Pinning | VO-07 |

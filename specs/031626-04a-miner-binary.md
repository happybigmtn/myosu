# Specification: Miner Binary — MCCFR Solver Node

Source: Master spec AC-MN-01, robopoker solver + chain pallet analysis
Status: Draft
Date: 2026-03-30
Depends-on: GS-01..09 (pallet accepts registrations + weights), PE-01..04 (poker solver works)

## Purpose

Build the `myosu-miner` binary — a long-running process that registers as a
neuron on a game-solving subnet, trains a strategy via MCCFR, serves strategy
queries to validators via an HTTP axon endpoint, and advertises its endpoint
on-chain.

The miner is the supply side of the solver market. It converts compute into
strategy quality, which Yuma Consensus converts into token emissions.

**Key design constraint**: the miner must be able to start with zero training
and progressively improve. Validators score the current best strategy, not
the training state.

## Whole-System Goal

Current state:
- Chain pallet and shared client already support stage-0 miner bootstrap
  actions such as registration and axon advertisement
- Poker advice artifacts, checkpoint-backed strategy queries, and bounded local
  training are already live in the repo
- `myosu-miner` exists today, but it is still a bootstrap-style service rather
  than the full always-training daemon described by this spec

This spec defines the remaining work to turn the existing bootstrap-style
`myosu-miner` crate into the fuller long-running miner described here:
- stronger continuous training orchestration
- durable checkpoint lifecycle
- stable HTTP axon serving
- cleaner on-chain advertisement and restart behavior

If all ACs land:
- `myosu-miner --chain ws://localhost:9944 --subnet 1 --key //Alice` registers and starts training
- Validators can query the miner's axon for strategy distributions
- Training checkpoints survive restarts
- Miner advertises its endpoint on-chain

Still not solved here:
- GPU-accelerated training
- Distributed training across multiple workers
- Pre-computed abstraction sharing
- Multiple simultaneous subnet participation

12-month direction:
- Multi-GPU training with rayon worker pools
- Abstraction download from shared store
- Automatic subnet selection based on profitability

## Why This Spec Exists As One Unit

- Registration, training, serving, and advertising form a single operational
  lifecycle — a miner that registers but doesn't train is useless, and one
  that trains but doesn't serve is invisible
- Testing requires the full loop: register → train → serve → query

## Scope

In scope:
- CLI binary with clap argument parsing
- Chain client (subxt or raw RPC) for extrinsics
- Background training loop
- HTTP axon server for strategy queries
- On-chain axon advertisement
- File-based checkpointing
- Graceful shutdown

Out of scope:
- Database-backed training (PostgreSQL)
- Multi-subnet participation
- Distributed training workers
- Web dashboard or monitoring UI

## Current State

- `crates/myosu-miner/` already exists with `cli.rs`, `chain.rs`,
  `training.rs`, `strategy.rs`, and `axon.rs`
- The current binary can probe the chain, optionally register, optionally
  advertise an axon, run a bounded training batch, serve a single strategy
  query from a checkpoint, or start an HTTP axon server
- The missing gap is not crate creation; it is evolving this bootstrap surface
  into the fuller continuously improving miner described by this spec

## What Already Exists

| Sub-problem | Existing code / flow | Reuse / extend / replace | Why |
|-------------|----------------------|--------------------------|-----|
| CLI | `crates/myosu-miner/src/cli.rs` | extend | Argument parsing and stage-0 bootstrap flags already exist |
| Chain interaction | `crates/myosu-miner/src/chain.rs` + `myosu-chain-client` | extend | Probe/register/serve paths already exist |
| Training | `crates/myosu-miner/src/training.rs` | extend | Bounded local training batch and checkpoint writing already exist |
| Strategy queries | `crates/myosu-miner/src/strategy.rs` | extend | Checkpoint-backed single-query serving already exists |
| HTTP server | `crates/myosu-miner/src/axon.rs` | extend | Stage-0 `/health` and `/strategy` surface already exists |

## Non-goals

- Custom networking protocol — HTTP is sufficient for bootstrap
- Authentication on axon endpoint — open queries for bootstrap
- Training optimization — default Pluribus config is sufficient
- Metrics export — log-based monitoring for bootstrap

## Ownership Map

| Component | Status | Location |
|-----------|--------|----------|
| CLI + main | Live | crates/myosu-miner/src/main.rs, crates/myosu-miner/src/cli.rs |
| Chain client | Live | crates/myosu-miner/src/chain.rs + crates/myosu-chain-client/ |
| Training loop | Partially live | crates/myosu-miner/src/training.rs |
| Strategy serving | Live | crates/myosu-miner/src/strategy.rs |
| Axon server | Live | crates/myosu-miner/src/axon.rs |
| Checkpoint mgmt | Partially live | crates/myosu-miner/src/training.rs, crates/myosu-miner/src/strategy.rs |

---

## Acceptance Criteria

### AC-MN-01: CLI and Chain Registration

- Where: `crates/myosu-miner/src/main.rs`, `crates/myosu-miner/src/chain.rs`
- How: Binary with clap CLI:
  ```
  myosu-miner --chain ws://localhost:9944 --subnet 1 --key //Alice --port 8080 --data-dir ./miner-data
  ```
  On startup: connect to chain via WebSocket RPC, submit `register_neuron(subnet_id)`
  extrinsic, wait for confirmation, log assigned UID.
- Whole-system effect: miner becomes a participant in the subnet.
- State: chain connection, registered UID, key pair.
- Wiring contract:
  - Trigger: `myosu-miner` CLI launch
  - Callsite: `crates/myosu-miner/src/main.rs`
  - State effect: neuron registered on chain
  - Persistence effect: UID stored in local state
  - Observable signal: log "Registered as UID {n} on subnet {s}"
- Required tests:
  - `cargo test -p myosu-miner chain::tests::connect_to_devnet`
  - `cargo test -p myosu-miner chain::tests::register_neuron_success`
  - `cargo test -p myosu-miner chain::tests::already_registered_skips`
- Pass/fail:
  - Connects to devnet RPC within 5 seconds
  - Registration extrinsic succeeds and returns UID
  - If already registered, skips registration and uses existing UID
  - Invalid key → clear error message
  - Chain unreachable → clear error with retry suggestion
- Blocking note: registration is the first thing a miner does.
- Rollback condition: subxt/RPC client incompatible with chain runtime.

### AC-MN-02: Background Training Loop

- Where: `crates/myosu-miner/src/training.rs`
- How: Spawn a background task (tokio::spawn) that runs MCCFR training
  continuously:
  ```rust
  async fn training_loop(
      published: Arc<ArcSwap<PokerSolver>>,
      checkpoint_dir: PathBuf,
  ) {
      let mut hot = PokerSolver::new(encoder.clone());
      loop {
          hot.train(BATCH_SIZE);  // e.g., 1000 iterations per batch
          // Publish snapshot for zero-contention reads
          published.store(Arc::new(hot.snapshot_profile()));
          // Checkpoint every CHECKPOINT_INTERVAL batches
          if hot.epochs() % CHECKPOINT_INTERVAL == 0 {
              hot.save_checkpoint(&checkpoint_dir.join("latest.bin"))?;
          }
          tokio::task::yield_now().await;
      }
  }
  ```
  The solver uses a **double-buffer pattern** via `arc_swap::ArcSwap<PokerSolver>`.
  The training loop writes to a private "hot" profile, and every N iterations
  publishes a snapshot to the shared `ArcSwap`. Reads always hit the published
  snapshot with **zero contention** — no locks, no blocking. This avoids the
  RwLock problem where training batches (seconds of CPU work) would starve
  all strategy queries.

  The training task's `JoinHandle` is monitored. On panic (OOM, arithmetic
  overflow), the miner logs the error, saves a checkpoint, and either restarts
  training from the last checkpoint or shuts down. The `/health` endpoint
  includes `training_active: bool` and `last_training_epoch: usize` so
  validators and operators can detect stalled miners.
- Whole-system effect: produces strategy quality over time. The longer a
  miner runs, the better its strategy becomes.
- State: PokerSolver training state (regrets, weights).
- Wiring contract:
  - Trigger: after registration completes
  - Callsite: `main.rs` spawns training task
  - State effect: solver improves over time
  - Persistence effect: checkpoints written to disk
  - Observable signal: log "Epoch {n}, exploitability: {x} mbb/h"
- Required tests:
  - `cargo test -p myosu-miner training::tests::training_loop_runs`
  - `cargo test -p myosu-miner training::tests::checkpoint_written`
  - `cargo test -p myosu-miner training::tests::solver_accessible_during_training`
- Pass/fail:
  - Training loop runs for 100 iterations without panic
  - Checkpoint file created at expected interval
  - Strategy query succeeds while training is running (no deadlock)
  - Exploitability logged and decreasing over time
- Blocking note: without training, the miner serves random strategies.
- Rollback condition: ArcSwap snapshot clone too expensive or publish contention
  makes query latency > 2 seconds.

### AC-MN-03: HTTP Axon Server

- Where: `crates/myosu-miner/src/axon.rs`
- How: HTTP server (axum or actix-web) on the configured port:

  `POST /strategy` — accepts `WireStrategy` query, returns `WireStrategy` response.
  `POST /strategy/batch` — accepts `Vec<WireStrategy>`, returns `Vec<WireStrategy>`.
  `GET /health` — returns `{ "status": "ok", "epochs": N, "exploitability": X, "training_active": true }`.

  Content type: `application/octet-stream` (bincode) for strategy endpoints,
  `application/json` for health. Include `X-Myosu-Version: 1` header.

  The axon reads from the shared `ArcSwap<PokerSolver>` via `published.load()`
  with zero contention — no locks, no blocking during training batches.
- Whole-system effect: this is how validators discover and score the miner.
- State: HTTP server bound to port.
- Wiring contract:
  - Trigger: after registration completes
  - Callsite: `main.rs` spawns axon server
  - State effect: HTTP server listening
  - Persistence effect: N/A
  - Observable signal: `curl http://localhost:8080/health` responds
- Required tests:
  - `cargo test -p myosu-miner axon::tests::health_endpoint`
  - `cargo test -p myosu-miner axon::tests::strategy_endpoint`
  - `cargo test -p myosu-miner axon::tests::invalid_query_returns_error`
- Pass/fail:
  - `GET /health` returns 200 with JSON containing epochs and exploitability
  - `POST /strategy` with valid WireStrategy returns valid response
  - `POST /strategy` with invalid bytes returns 400
  - Server handles 100 concurrent requests without error
- Blocking note: validators query this endpoint. No axon = invisible miner.
- Rollback condition: HTTP framework conflicts with tokio runtime used by training.

### AC-MN-04: On-Chain Axon Advertisement

- Where: `crates/myosu-miner/src/chain.rs (extend)`
- How: After the axon server starts, submit `serve_axon(subnet_id, ip, port, ...)`
  extrinsic so validators can discover the miner's endpoint from chain state.
  Re-advertise on IP/port change or periodically (every 1000 blocks).
- Whole-system effect: validators discover miners by reading Axons storage.
- State: chain connection.
- Wiring contract:
  - Trigger: axon server starts listening
  - Callsite: `main.rs` after axon bind
  - State effect: AxonInfo stored on chain for this neuron
  - Persistence effect: on-chain storage updated
  - Observable signal: `AxonServed` event on chain
- Required tests:
  - `cargo test -p myosu-miner chain::tests::serve_axon_success`
  - `cargo test -p myosu-miner chain::tests::axon_discoverable_via_rpc`
- Pass/fail:
  - `serve_axon` extrinsic succeeds
  - Querying `Axons[subnet][hotkey]` via RPC returns correct IP and port
  - Re-advertisement succeeds after initial advertisement
- Blocking note: without advertisement, validators don't know where to query.
- Rollback condition: IP detection fails in container/cloud environments.

### AC-MN-05: Graceful Shutdown and Resume

- Where: `crates/myosu-miner/src/main.rs (extend)`
- How: On SIGINT/SIGTERM:
  1. Stop accepting new axon queries
  2. Finish current training batch
  3. Save final checkpoint
  4. Log final epoch count and exploitability
  5. Exit cleanly

  On restart with existing checkpoint:
  1. Load checkpoint from data-dir
  2. Resume training from saved epoch
  3. Re-register axon (UID persists on chain if not pruned)
- Whole-system effect: miners can be stopped and restarted without losing progress.
- State: signal handler, checkpoint state.
- Wiring contract:
  - Trigger: SIGINT or SIGTERM
  - Callsite: tokio signal handler in main.rs
  - State effect: training stops, checkpoint saved
  - Persistence effect: final checkpoint on disk
  - Observable signal: log "Shutdown complete. Saved checkpoint at epoch {n}"
- Required tests:
  - `cargo test -p myosu-miner main::tests::graceful_shutdown`
  - `cargo test -p myosu-miner main::tests::resume_from_checkpoint`
- Pass/fail:
  - Sending SIGINT causes clean exit within 5 seconds
  - Checkpoint file exists after shutdown
  - Restarting loads checkpoint and resumes at correct epoch
  - No data corruption in checkpoint after forced shutdown
- Blocking note: production miners need restart resilience.
- Rollback condition: checkpoint format changes between versions.

---

## Decision Log

- 2026-03-16: HTTP axon (not gRPC) — simpler, widely supported, sufficient
  bandwidth for strategy queries (~1KB per request/response).
- 2026-03-16: Arc<RwLock> for solver sharing — training writes rarely conflict
  with strategy reads. RwLock allows concurrent reads.
- 2026-03-16: File checkpoints (not database) — simpler, portable, sufficient
  for single-miner operation.

## Milestone Verification

| # | Scenario | Validates | ACs |
|---|----------|-----------|-----|
| 1 | `myosu-miner --chain ws://localhost:9944 --subnet 1` registers on devnet | Registration | MN-01 |
| 2 | Training runs and exploitability decreases over 1000 iterations | Training | MN-02 |
| 3 | `curl http://localhost:8080/health` returns miner status | Axon server | MN-03 |
| 4 | Validator queries strategy via axon and gets valid response | Query handling | MN-03 |
| 5 | Axon discoverable via chain RPC query | Advertisement | MN-04 |
| 6 | Stop miner, restart, resume from checkpoint | Resilience | MN-05 |

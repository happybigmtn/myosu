# Specification: Miner Binary — MCCFR Solver Node

Source: Master spec AC-MN-01, robopoker solver + chain pallet analysis
Status: Draft
Date: 2026-03-16
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
- Chain pallet (GS-*) accepts neuron registration, weight submission, axon serving
- Poker engine (PE-*) provides PokerSolver with training and query interfaces
- No miner binary exists

This spec adds:
- `myosu-miner` binary crate
- Chain client for registration and axon advertisement
- Training loop running continuously in the background
- HTTP axon endpoint serving strategy queries
- Checkpoint management for training persistence

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

- No miner code exists
- `myosu-games-poker` provides `PokerSolver` (PE-01)
- Chain pallet provides `register_neuron`, `serve_axon` extrinsics (GS-03, GS-07)

## What Already Exists

| Sub-problem | Existing code / flow | Reuse / extend / replace | Why |
|-------------|----------------------|--------------------------|-----|
| Solver training | `PokerSolver::train()` (PE-01) | reuse | MCCFR training loop |
| Strategy queries | `PokerSolver::handle_query()` (PE-02) | reuse | Query handler |
| Chain interaction | subxt or jsonrpsee | new | Need RPC client for extrinsics |
| HTTP server | actix-web or axum | new | Axon endpoint |
| CLI | clap | new | Argument parsing |

## Non-goals

- Custom networking protocol — HTTP is sufficient for bootstrap
- Authentication on axon endpoint — open queries for bootstrap
- Training optimization — default Pluribus config is sufficient
- Metrics export — log-based monitoring for bootstrap

## Ownership Map

| Component | Status | Location |
|-----------|--------|----------|
| CLI + main | New | crates/myosu-miner/src/main.rs |
| Chain client | New | crates/myosu-miner/src/chain.rs |
| Training loop | New | crates/myosu-miner/src/training.rs |
| Axon server | New | crates/myosu-miner/src/axon.rs |
| Checkpoint mgmt | New | crates/myosu-miner/src/checkpoint.rs |

---

### AC-MN-01: CLI and Chain Registration

- Where: `crates/myosu-miner/src/main.rs (new)`, `src/chain.rs (new)`
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

- Where: `crates/myosu-miner/src/training.rs (new)`
- How: Spawn a background task (tokio::spawn) that runs MCCFR training
  continuously:
  ```rust
  async fn training_loop(solver: Arc<RwLock<PokerSolver>>, checkpoint_dir: PathBuf) {
      loop {
          {
              let mut s = solver.write().await;
              s.train(BATCH_SIZE);  // e.g., 1000 iterations per batch
          }
          // Checkpoint every CHECKPOINT_INTERVAL batches
          if solver.read().await.epochs() % CHECKPOINT_INTERVAL == 0 {
              solver.read().await.save(&checkpoint_dir.join("latest.bin"))?;
          }
          tokio::task::yield_now().await;
      }
  }
  ```
  The solver is behind `Arc<RwLock>` so the axon server can read the current
  strategy while training writes to it. RwLock allows concurrent reads
  (strategy queries) with exclusive writes (training updates).
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
- Rollback condition: RwLock contention makes query latency > 2 seconds.

### AC-MN-03: HTTP Axon Server

- Where: `crates/myosu-miner/src/axon.rs (new)`
- How: HTTP server (axum or actix-web) on the configured port:

  `POST /strategy` — accepts `WireStrategy` query, returns `WireStrategy` response.
  `GET /health` — returns `{ "status": "ok", "epochs": N, "exploitability": X }`.

  The axon reads from the shared `Arc<RwLock<PokerSolver>>` without blocking
  training (uses RwLock read lock).
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

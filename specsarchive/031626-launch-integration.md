# Specification: Launch Integration — End-to-End NLHE HU Product

Source: Stage 0 + Stage 1 gap analysis for NLHE HU launch
Status: Draft
Date: 2026-03-16
Depends-on: ALL prior specs (this is the capstone)

## Purpose

Wire all components into a single launchable product: a human plays
No-Limit Hold'em heads-up against the strongest available solver strategy
in a polished TUI, backed by a chain with real incentive mechanics and
live miners competing on strategy quality.

This spec covers the integration seams that no individual spec owns:
the devnet orchestration, the miner bootstrap sequence, the TUI-gameplay
wiring, and the end-to-end acceptance test.

## Whole-System Goal

Current state:
- 12 specs cover individual components (chain, pallet, traits, poker,
  miner, validator, gameplay, TUI, abstractions)
- No spec covers running them all together
- No spec defines the miner bootstrap sequence
- GP-01..04 was written before TUI spec — doesn't use myosu-tui

This spec adds:
- Devnet orchestration (docker-compose or process manager)
- Miner bootstrap sequence (abstractions → register → train → serve)
- GP ↔ TUI wiring (gameplay uses myosu-tui shell)
- End-to-end acceptance test
- Launch checklist

If all ACs land:
- `docker compose up` starts chain + miner + validator
- `myosu-play --chain ws://localhost:9944 --subnet 1` launches TUI
- Human plays a complete NLHE HU session against trained bot
- Miner earns emissions proportional to strategy quality

## Scope

In scope:
- Docker compose for devnet (chain node + miner + validator)
- Miner bootstrap script (download abstractions → register → train)
- myosu-play refactored to use myosu-tui as rendering backend
- End-to-end integration test (all components, 1 hand to completion)
- Launch readiness checklist

Out of scope:
- Production/mainnet deployment (needs token economics spec)
- Multi-subnet orchestration (only NLHE HU for launch)
- CI/CD pipeline
- Monitoring/alerting infrastructure

---

### AC-LI-01: Devnet Orchestration

- Where: `ops/devnet/docker-compose.yml (new)`, `ops/devnet/README.md (new)`
- How: Docker compose with 3 services:

  ```yaml
  services:
    chain:
      build: crates/myosu-chain/
      command: myosu-node --dev --tmp --rpc-external
      ports: ["9944:9944"]

    miner:
      build: crates/myosu-miner/
      command: >
        myosu-miner
          --chain ws://chain:9944
          --subnet 1
          --key //Alice//miner
          --port 8080
          --data-dir /data
      volumes: ["miner-data:/data"]
      depends_on: [chain]

    validator:
      build: crates/myosu-validator/
      command: >
        myosu-validator
          --chain ws://chain:9944
          --subnet 1
          --key //Bob
          --stake 10000
      depends_on: [chain, miner]
  ```

  The miner service includes abstraction download in its entrypoint
  (check for files, download if missing, then start).

  A genesis subnet (nlhe_hu, id=1) is pre-configured in the dev chain spec
  (already specified in CF-04/GS-09).

- Whole-system effect: one command starts the entire stack.
- Wiring contract:
  - Trigger: `docker compose up` in `ops/devnet/`
  - Callsite: docker compose orchestration
  - State effect: chain producing blocks, miner training, validator scoring
  - Persistence effect: chain data + miner checkpoints in volumes
  - Observable signal: `docker compose logs` shows block production + training epochs
- Required tests:
  - `docker compose up -d && sleep 30 && curl -sf http://localhost:8080/health`
  - `docker compose down` cleans up
- Pass/fail:
  - Chain produces blocks within 10 seconds
  - Miner registers and starts training within 30 seconds
  - Validator registers and submits weights within first tempo
  - `curl http://localhost:8080/health` returns valid JSON
  - `docker compose down` exits cleanly
- Blocking note: developers and testers need one command to run the full stack.
- Rollback condition: Docker build times exceed 30 minutes.

### AC-LI-02: Miner Bootstrap Sequence

- Where: `crates/myosu-miner/src/bootstrap.rs (new)`, `scripts/miner-bootstrap.sh (new)`
- How: Automated startup sequence:

  ```
  1. CHECK abstractions exist at --data-dir/abstractions/
     └─ if missing: download via scripts/download-abstractions.sh
     └─ verify SHA-256 hash

  2. LOAD encoder from abstractions
     └─ log encoder hash for INV-003 audit

  3. CHECK chain connectivity
     └─ retry with exponential backoff (max 60 seconds)

  4. CHECK subnet exists
     └─ if not: error "subnet {id} not found, create with create_subnet extrinsic"

  5. REGISTER neuron (if not already registered)
     └─ burn cost: log balance before/after
     └─ if already registered: log "reusing existing UID {n}"

  6. LOAD checkpoint (if exists at --data-dir/checkpoint.bin)
     └─ verify version header (magic MYOS + version)
     └─ log "resuming from epoch {n}"

  7. START training loop (ArcSwap double-buffer)
     └─ log "training started, epoch {n}"

  8. START axon server (HTTP on --port)
     └─ log "axon serving on :{port}"

  9. ADVERTISE axon on chain (serve_axon extrinsic)
     └─ log "axon advertised: {ip}:{port}"

  10. ENTER main loop (train + serve + checkpoint)
  ```

  Each step logs clearly. Any failure at steps 1-5 halts with actionable
  error message. Steps 6-9 are recoverable (retry or skip).

- Whole-system effect: miner goes from zero to fully operational automatically.
- Required tests:
  - `cargo test -p myosu-miner bootstrap::tests::full_sequence_on_devnet`
  - `cargo test -p myosu-miner bootstrap::tests::missing_abstractions_downloads`
  - `cargo test -p myosu-miner bootstrap::tests::resume_from_checkpoint`
- Pass/fail:
  - Fresh miner with no data: downloads abstractions, registers, starts training
  - Miner with existing checkpoint: loads checkpoint, resumes training
  - Chain unreachable: retries with backoff, gives up after 60s with clear error
  - Subnet not found: clear error with instructions

### AC-LI-03: Gameplay ↔ TUI Wiring

- Where: `crates/myosu-play/src/main.rs (extend)`, `crates/myosu-games-poker/src/renderer.rs (new)`
- How: Refactor GP-01..04 to use myosu-tui as the rendering backend:

  1. `myosu-play` depends on `myosu-tui` and `myosu-games-poker`
  2. `NlheRenderer` implements `GameRenderer` trait (TU-01):
     - `render_state()` draws board, hands, stacks, pot
     - `declaration()` returns appropriate hero text
     - `completions()` returns legal actions (fold, call, raise, shove)
     - `parse_input()` parses poker commands
     - `pipe_output()` renders design.md pipe format
  3. `myosu-play` creates the TUI shell with `NlheRenderer`
  4. Bot strategy queries happen via the event loop (TU-03) async channel
  5. Hand history recording (GP-04) hooks into the log panel

  This replaces the ad-hoc CLI prompts in GP-02 with the proper TUI shell.

- Whole-system effect: the gameplay experience matches design.md exactly.
- Required tests:
  - `cargo test -p myosu-games-poker renderer::tests::nlhe_renderer_implements_trait`
  - `cargo test -p myosu-play integration::tests::one_hand_in_tui`
  - `cargo test -p myosu-play integration::tests::pipe_mode_one_hand`
- Pass/fail:
  - NlheRenderer implements all GameRenderer methods
  - TUI displays design.md 8.1 (NLHE HU) screen format
  - Bot actions appear in log panel
  - `/stats` shows session summary
  - `--pipe` mode plays a complete hand via stdin/stdout

### AC-LI-04: End-to-End Acceptance Test

- Where: `tests/e2e/nlhe_launch.rs (new)`
- How: Integration test that exercises the full product:

  ```rust
  #[tokio::test]
  async fn nlhe_hu_launch_product() {
      // 1. Start devnet node
      let node = start_devnet_node().await;

      // 2. Create NLHE HU subnet (or verify genesis subnet exists)
      let subnet = ensure_subnet(&node, "nlhe_hu").await;

      // 3. Start miner with pre-computed abstractions (preflop only for speed)
      let miner = start_miner(&node, subnet, preflop_abstractions()).await;

      // 4. Train for 100 iterations
      miner.train(100).await;

      // 5. Start validator, evaluate miner, submit weights
      let validator = start_validator(&node, subnet).await;
      validator.evaluate_and_submit().await;

      // 6. Verify Yuma ran and distributed emissions
      let emissions = query_emissions(&node, subnet).await;
      assert!(emissions.total > 0);

      // 7. Play one hand via pipe mode
      let result = play_one_hand_pipe(&miner).await;
      assert!(result.hand_completed);
      assert!(result.bot_actions_valid);

      // 8. Verify hand history recorded
      assert!(result.history_file_exists);

      // Cleanup
      node.stop().await;
  }
  ```

  This test uses preflop-only abstractions (169 entries, <1 second to load)
  for speed. Full abstractions are too large for CI.

- Whole-system effect: proves the entire product works end-to-end. If this
  test passes, we can launch.
- Required tests: this IS the test.
- Pass/fail:
  - All 8 steps complete without error
  - Total test time < 120 seconds
  - Miner strategy is non-random after 100 iterations
  - Emissions distributed to miner after tempo
  - Hand completes with valid pot accounting

### AC-LI-05: Launch Readiness Checklist

- Where: `docs/launch-checklist.md (new)`
- How: Document that must be completed before calling Stage 0 "done":

  ```markdown
  # NLHE HU Launch Readiness

  ## Chain
  - [ ] myosu-node --dev produces blocks
  - [ ] Game-solving pallet at index 7
  - [ ] Genesis subnet (nlhe_hu, id=1)
  - [ ] Yuma Consensus matches subtensor test vectors
  - [ ] Emission accounting: sum == block_emission

  ## Solver
  - [ ] Abstraction artifact published with SHA-256
  - [ ] Miner bootstraps from zero (download + register + train)
  - [ ] Miner checkpoint save/restore works
  - [ ] Miner axon serves valid strategy queries
  - [ ] Exploitability decreases over training

  ## Scoring
  - [ ] Validator registers and stakes
  - [ ] Validator queries miner, computes exploitability
  - [ ] Validator submits weights, visible on chain
  - [ ] Two validators produce identical scores (INV-003)
  - [ ] Yuma distributes emissions after tempo

  ## Gameplay
  - [ ] TUI renders design.md 8.1 (NLHE HU) format
  - [ ] Bot plays from miner strategy (not random)
  - [ ] Fallback to random on miner timeout
  - [ ] Hand completes to showdown correctly
  - [ ] /stats shows session statistics
  - [ ] /analyze shows coaching output
  - [ ] --pipe mode works for agents
  - [ ] Hand history JSON recorded

  ## Integration
  - [ ] docker compose up starts full stack
  - [ ] E2E test (LI-04) passes
  - [ ] All 6 invariants verified

  ## Not Required for Launch
  - [ ] Multi-subnet (only nlhe_hu)
  - [ ] Mainnet (devnet only)
  - [ ] Token economics (flat emission)
  - [ ] Liar's Dice (defer to post-launch)
  - [ ] Web UI (TUI only)
  ```

- Blocking note: prevents declaring launch before all critical paths work.

---

## Decision Log

- 2026-03-16: Docker compose for devnet — simplest orchestration, no k8s.
- 2026-03-16: Preflop-only abstractions for E2E test — 169 entries loads
  instantly, full abstractions are 3GB and impractical for CI.
- 2026-03-16: Liar's Dice NOT required for NLHE HU launch — multi-game
  validation is a separate milestone, not a launch blocker.

## Milestone Verification

| # | Scenario | Validates | ACs |
|---|----------|-----------|-----|
| 1 | `docker compose up` starts chain + miner + validator | Devnet orchestration | LI-01 |
| 2 | Fresh miner downloads abstractions and starts training | Bootstrap | LI-02 |
| 3 | Human plays one hand in TUI matching design.md 8.1 | TUI wiring | LI-03 |
| 4 | E2E test: chain → miner → validator → Yuma → gameplay | Full product | LI-04 |
| 5 | All launch checklist items checked | Readiness | LI-05 |

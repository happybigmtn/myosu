# Miner Binary Implementation

**Plan ID:** 011
**Status:** New
**Priority:** HIGH — core chain participant

`genesis/PLANS.md` at `genesis/PLANS.md` governs this document.

---

## Purpose / Big Picture

After this plan lands, the miner binary (`myosu-miner`) will connect to the local devnet, register as a neuron, run MCCFR game-solving, and submit strategy weights to the chain. A miner operator can start the binary, stake funds, and watch it compete on strategy quality.

---

## Progress

- [ ] Create `crates/myosu-miner/` crate scaffold
- [ ] Implement neuron registration with chain
- [ ] Implement MCCFR game-solver using `myosu-games`
- [ ] Implement weight submission to chain
- [ ] Implement hotkey management
- [ ] Test end-to-end: miner registers → solves → submits weights

---

## Surprises & Discoveries

*(To be written during implementation)*

---

## Decision Log

- Decision: Miner submits weights to the chain, not directly to validators.
  Rationale: The game-solver pallet receives weight submissions and distributes emission based on Yuma Consensus. Direct validator submission would bypass consensus.
  Date/Author: 2026-03-21 / Interim CEO

---

## Milestones

### M1: Create `crates/myosu-miner/` scaffold
Set up the crate with CLI interface via clap. Binary accepts `--subnet`, `--hotkey`, `--wallet`.

Proof: `cargo build -p myosu-miner` produces a binary at `target/debug/myosu-miner` that responds to `--help`.

Key files:
- `crates/myosu-miner/src/main.rs`
- `crates/myosu-miner/src/cli.rs`

### M2: Implement neuron registration
Register the miner as a neuron on the target subnet.

Proof: `cargo run -p myosu-miner -- --subnet 1 --register` succeeds and the chain shows the new neuron registered.

### M3: Implement MCCFR game-solver loop
The miner runs MCCFR solving for the configured game (NLHE) and produces strategy checkpoints.

Proof: `cargo run -p myosu-miner -- --subnet 1 --solve --iterations 10000` produces a checkpoint file.

### M4: Implement weight submission
Submit strategy weights to the game-solver pallet.

Proof: After solving, `cargo run -p myosu-miner -- --subnet 1 --submit` succeeds and the chain shows weight submission.

### M5: End-to-end miner workflow
Register → solve → submit → receive emission.

Proof: A miner that registers, solves for 10,000 iterations, and submits weights receives a positive emission within one epoch.

---

## Context and Orientation

The miner binary follows the subtensor miner pattern:
```
myosu-miner
├── CLI: --subnet, --hotkey, --wallet, --register, --solve, --submit
├── Neuron registration via subtensor RPC
├── MCCFR solving via myosu-games
└── Weight submission via game-solver pallet tx
```

Dependencies:
- `myosu-games` — game-solving traits
- `myosu-games-poker` — NLHE solver
- `sp-core` + `sp-runtime` — chain types
- `substrate-api-sidecar` or direct RPC — for chain interaction

---

## Plan of Work

1. Create `crates/myosu-miner/` scaffold with CLI
2. Add to workspace `Cargo.toml`
3. Implement chain connection (substrate RPC)
4. Implement neuron registration
5. Implement MCCFR solving loop
6. Implement weight submission
7. Test end-to-end on local devnet

---

## Concrete Steps

```bash
# Create scaffold
cargo new --lib crates/myosu-miner
# Add to workspace
echo '  "crates/myosu-miner",' >> Cargo.toml

# Implement CLI with clap
cargo add clap --features derive

# Implement registration
cargo run -p myosu-miner -- --help
# Expected: shows --subnet, --register, --solve, --submit

# End-to-end test (after chain is running)
# Terminal 1: start devnet
./target/debug/myosu-chain --dev --rpc-port 9933

# Terminal 2: register miner
cargo run -p myosu-miner -- --subnet 1 --register --rpc ws://localhost:9944

# Terminal 2: solve and submit
cargo run -p myosu-miner -- --subnet 1 --solve --iterations 10000 --submit --rpc ws://localhost:9944
```

---

## Validation

- `cargo build -p myosu-miner` passes
- `cargo run -p myosu-miner -- --help` shows all CLI options
- `cargo run -p myosu-miner -- --subnet 1 --register` succeeds
- `cargo run -p myosu-miner -- --subnet 1 --solve --submit` submits weights
- Chain shows miner registered and weights submitted

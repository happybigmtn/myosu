# Implement weight submission to chain Lane — Review

Review only the current slice for `miner-binary-weight-submission`.

Current Slice Contract:
Plan file:
- `genesis/plans/011-miner-binary.md`

Child work item: `miner-binary-weight-submission`

Full plan context (read this for domain knowledge, design decisions, and specifications):

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


Workflow archetype: implement

Review profile: hardened

Active plan:
- `genesis/plans/001-master-plan.md`

Active spec:
- `genesis/SPEC.md`

AC contract:
- Where: Weight submission logic sending strategy weights to game-solver pallet
- How: Submit solved strategy weights to the game-solver pallet via chain transaction
- Required tests: cargo run -p myosu-miner -- --subnet 1 --solve --submit --rpc ws://localhost:9944
- Verification plan: Weights submitted and chain shows weight submission event
- Rollback condition: Submission fails or chain does not record weights

Proof commands:
- `cargo build -p myosu-miner`
- `cargo run -p myosu-miner -- --subnet 1 --submit --rpc ws://localhost:9944`

Artifacts to write:
- `spec.md`
- `review.md`


Verification artifact must cover
- summarize the automated proof commands that ran and their outcomes

Nemesis-style security review
- Pass 1 — first-principles challenge: question trust boundaries, authority assumptions, and who can trigger the slice's dangerous actions
- Pass 2 — coupled-state review: identify paired state or protocol surfaces and check that every mutation path keeps them consistent or explains the asymmetry
- check state transitions that affect balances, commitments, randomness, payout safety, or replayability
- check secret handling, capability scoping, pairing/idempotence behavior, and privilege escalation paths

Focus on:
- slice scope discipline
- proof-gate coverage for the active slice
- touched-surface containment
- implementation and verification artifact quality
- remaining blockers before the next slice

Deterministic evidence:
- treat `quality.md` as machine-generated truth about placeholder debt, warning debt, manual follow-up, and artifact mismatch risk
- if `quality.md` says `quality_ready: no`, do not bless the slice as merge-ready


Write `promotion.md` in this exact machine-readable form:

merge_ready: yes|no
manual_proof_pending: yes|no
reason: <one sentence>
next_action: <one sentence>

Only set `merge_ready: yes` when:
- `quality.md` says `quality_ready: yes`
- automated proof is sufficient for this slice
- any required manual proof has actually been performed
- no unresolved warnings or stale failures undermine confidence
- the implementation and verification artifacts match the real code.

Review stage ownership:
- you may write or replace `promotion.md` in this stage
- read `quality.md` before deciding `merge_ready`
- when the slice is security-sensitive, perform a Nemesis-style pass: first-principles assumption challenge plus coupled-state consistency review
- include security findings in the review verdict when the slice touches trust boundaries, keys, funds, auth, control-plane behavior, or external process control
- prefer not to modify source code here unless a tiny correction is required to make the review judgment truthful

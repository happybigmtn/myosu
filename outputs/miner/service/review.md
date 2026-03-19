# Miner Service Review

**Lane**: `miner:service`
**Date**: 2026-03-19
**Judgment**: **KEEP lane open; RESET implementation scope**

---

## 1. Keep / Reopen / Reset Decision

### Decision: **RESET implementation scope** for the lane contract

The `spec.md` is accepted as written. The lane boundary, ownership map, and slice definitions are correct.

The lane is **not** a continuation of any previous effort — no `myosu-miner` or `myosu-cluster` code exists anywhere in `crates/`. The spec archive files (`031626-04a-miner-binary.md`, `031626-08-abstraction-pipeline.md`) were design documents, not implementations.

**Therefore**: the lane starts fresh. There is nothing to reopen. There is nothing to keep from a prior implementation attempt.

---

## 2. Current State Assessment

### What exists

Nothing. There are zero lines of miner service code in the repository.

The `specsarchive/` files are the only source of truth for the intended design.

### What the chain lanes expose

Both `chain:pallet` and `chain:runtime` are in `restart required` state per their respective `spec.md` files:

- `chain:pallet`: cannot compile due to 50+ missing workspace keys and import errors
- `chain:runtime`: cannot compile; all subtensor-derived pallets are blocked on the same missing keys

The miner service lane **cannot exercise honest on-chain registration** (AC-MN-01, AC-MN-04) until `chain:pallet` restart completes.

### What can be built immediately

- Slice 1 (CLI skeleton) and Slice 2 (training loop with mock encoder) are buildable without any upstream dependencies
- Slice 3 (`myosu-cluster`) is buildable using the existing `rbp-clustering` code from the robopoker fork
- Slice 4 requires `chain:pallet` to compile
- Slice 5 requires RF-02 (`NlheEncoder::from_dir()`) to be implemented in the robopoker fork

---

## 3. First Proof and Health Checks to Add

### 3.1 First Proof Check (Slice 1 → Slice 2 transition)

**Proof check**: `cargo test -p myosu-miner -- --test-threads=1` runs the training loop for 1000 iterations and confirms `/health` reflects increasing epoch count.

This is the **minimal proof** that the lane produces real training progress, not just a running process.

### 3.2 First Health Check to Add (Slice 2 → Slice 4 transition)

**Health check**: `GET /health` must include `training_active: bool` and `exploitability: f64`.

The `/health` endpoint is the lane's primary health surface. It must be the **first thing** exercised by any operator or automated bringup script.

### 3.3 Health Checks to Add (Post-Slice 4)

| Health check | Method | Expected signal |
|---|---|---|
| Chain registration | Query `Axons[subnet][hotkey]` via RPC | `Some(AxonInfo { ip, port, ... })` |
| Axon reachability | `curl http://{ip}:{port}/health` | HTTP 200, JSON body |
| Checkpoint persistence | After SIGTERM, restart and compare epoch count | Epoch count continues from checkpoint |
| Encoder hash stability | Restart and compare `encoder.hash()` log output | SHA-256 matches previous run |
| Abstraction artifact integrity | Run `sha256sum` on all `.lookup.bin` files | Matches `manifest.json` hashes |

### 3.4 Proof Checks to Add (Post-Slice 5)

| Proof check | Method | Expected outcome |
|---|---|---|
| Strategy quality improvement | Validator queries miner at epoch 0 and epoch 10,000 | Score at epoch 10,000 ≥ epoch 0 |
| INV-003 alignment | Two miners load same abstraction artifact | Both log identical `encoder.hash()` |
| Coldkey swap resilience | Change operator coldkey, re-register | UID preserved, axon re-advertised |

---

## 4. Risk Factors

| Risk | Severity | Mitigation |
|------|----------|------------|
| `chain:pallet` restart takes extended time | **High** — blocks Slice 4 | Begin slices 1-3 immediately; they are chain-independent |
| `NlheEncoder::from_dir()` not implemented in RF-02 | **High** — blocks Slice 5 | Slice 5 is optional for proof-of-training; mock encoder suffices for Slice 2 |
| Abstraction artifact too large (>3GB) for download bootstrap | **Medium** | myosu-cluster local compute is the fallback; operator can self-host artifact |
| ArcSwap double-buffer pattern causes memory pressure | **Medium** | Monitor `Arc::strong_count`; fallback to `RwLock` if profiling shows issue |

---

## 5. Recommended Next Actions

1. **Start slices 1 and 3 immediately** — both are buildable today without any upstream lane completing
2. **Parallelize**: begin `myosu-miner` CLI skeleton (Slice 1) and `myosu-cluster` binary (Slice 3) in parallel
3. **Track `chain:pallet` restart progress** — once it completes, immediately exercise Slice 4 integration
4. **Do not wait for `chain:runtime`** — Slice 4 can be validated against a mock/stub chain client if the pallet itself compiles; real devnet validation comes later
5. **Wire `myosu-chain-client`** as a shared crate early — `validator-oracle` and `play-tui` lanes will need the same `register_neuron` and `serve_axon` wrappers

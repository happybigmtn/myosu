# Miner Service Spec

**Lane**: `miner:service`
**Date**: 2026-03-19
**Status**: Bootstrap — lane boundary defined; blocked on `chain:pallet` + `chain:runtime` restart completion for on-chain registration and serving primitives

---

## 1. Service Purpose and Operator-Visible Outcome

### Purpose

The `miner:service` lane produces the `myosu-miner` binary — a long-running process that:

1. **Registers** as a neuron on a game-solving subnet via the chain pallet
2. **Trains** a poker strategy continuously via MCCFR using a `PokerSolver`
3. **Serves** strategy queries over an HTTP axon endpoint to validators
4. **Advertises** its axon on-chain so validators can discover it
5. **Checkpoints** training state to survive restarts

### Operator-Visible Outcome

```
$ myosu-miner --chain ws://localhost:9944 --subnet 1 --key //Alice --port 8080 --data-dir ./miner-data
[00:00:00] Registered as UID 7 on subnet 1
[00:00:01] Axon serving on http://10.0.0.1:8080
[00:00:02] Epoch 0, exploitability: 500.0 mbb/h
[00:00:10] Epoch 100, exploitability: 340.0 mbb/h
[00:00:30] Epoch 1000, exploitability: 87.0 mbb/h
```

An operator launching the binary sees registration confirmation, axon advertisement confirmation, and a decreasing exploitability number as the only meaningful signal of progress.

---

## 2. Readiness Conditions Before Honest Bringup

All conditions must be satisfied before the lane is considered bringup-ready:

| # | Condition | Owner | Status |
|---|-----------|-------|--------|
| R1 | `crates/myosu-miner/` crate exists and `cargo build -p myosu-miner` exits 0 | `miner:service` | **Not started** |
| R2 | `crates/myosu-cluster/` crate exists and `cargo build -p myosu-cluster` exits 0 | `miner:service` | **Not started** |
| R3 | Chain pallet (`chain:pallet`) restart completes — `cargo check -p pallet-game-solver` passes | `chain:pallet` | **Restart required** |
| R4 | Chain runtime (`chain:runtime`) restart completes — `cargo build -p myosu-runtime` passes | `chain:runtime` | **Restart required** |
| R5 | Abstraction artifact exists at a versioned URL or local path | `miner:service` | **Not started** |
| R6 | `NlheEncoder::from_dir()` is implemented in the robopoker fork (RF-02) | External / RF-02 | **Not confirmed** |
| R7 | A live development chain (devnet) is running and reachable at the target RPC URL | Operator | **Not available** |

### Dependency Graph

```
miner:service
    │
    ├── R1: myosu-miner crate (BLOCKED BY: nothing — can start immediately)
    │       └── AC-MN-01 through AC-MN-05
    │
    ├── R2: myosu-cluster crate (BLOCKED BY: nothing — can start immediately)
    │       └── AC-AP-01, AC-AP-03
    │
    ├── R3: chain:pallet restart (BLOCKED BY: R3/R4)
    │       └── pallet-game-solver must compile
    │
    ├── R4: chain:runtime restart (BLOCKED BY: R4)
    │       └── myosu-runtime must compile
    │
    └── R5: abstraction artifact
            └── Either: run myosu-cluster locally OR download pre-computed artifact

R3 and R4 are the critical blockers for on-chain registration (AC-MN-01, AC-MN-04).
R5 is the critical blocker for training (AC-MN-02) — without abstractions, the solver
produces random strategies.
```

**Lane decision**: R1 and R2 can begin immediately and are independent of the chain lanes. R3/R4 must complete before AC-MN-01 (registration) and AC-MN-04 (axon advertisement) can be exercised honestly. The first honest bringup requires all seven conditions.

---

## 3. Expected Health Surfaces Once Running

### 3.1 HTTP Axon Endpoint (port 8080)

| Endpoint | Method | Response | Health signal |
|----------|--------|----------|---------------|
| `/health` | GET | `{"status": "ok", "epoch": N, "exploitability": X.X, "training_active": true}` | Process alive, training loop cycling |
| `/strategy` | POST | Bincode-encoded `WireStrategy` | Validator can score miner |
| `/strategy/batch` | POST | Bincode-encoded `Vec<WireStrategy>` | Batch query support |

`/health` is the primary surface for external monitoring and lane health checks. It must return within 50ms under load.

### 3.2 On-Chain State

| Surface | Query | Healthy when |
|---------|-------|--------------|
| `Axons[subnet_id][hotkey]` | Chain RPC | IP and port match configured values |
| `Neuron[subnet_id][hotkey].uid` | Chain RPC | UID is registered and non-zero |

### 3.3 Local State

| Surface | Path | Healthy when |
|---------|------|--------------|
| Checkpoint file | `{data-dir}/checkpoint/latest.bin` | File exists and epoch number is incrementing |
| Abstraction directory | `{data-dir}/abstractions/` | `manifest.json` present with valid SHA-256 |
| Log output | stderr/stdout | No ERROR-level messages for > 5 minutes |

### 3.4 Proof Posture

The lane exposes a **proof-of-training** posture:

- Validator queries `/strategy` and computes a game-theoretic score
- Score is monotonically non-worsening over time
- `exploitability` in `/health` decreases over epochs
- Encoder hash is logged at startup and verifiable against the on-chain `total_sha256`

---

## 4. Owned Files, Binaries, and Endpoints

### 4.1 Binaries

| Binary | Source path | Description |
|--------|-------------|-------------|
| `myosu-miner` | `crates/myosu-miner/src/main.rs` | CLI binary: registration + training + serving |
| `myosu-cluster` | `crates/myosu-cluster/src/main.rs` | Clustering binary: abstraction table generation |

### 4.2 Crate Structure

```
crates/
├── myosu-miner/           # AC-MN-01..05
│   ├── src/
│   │   ├── main.rs        # CLI (clap), signal handler, shutdown
│   │   ├── chain.rs       # Chain client: registration + axon advertisement
│   │   ├── training.rs    # Background MCCFR training loop + ArcSwap pattern
│   │   ├── axon.rs        # HTTP server: /health, /strategy, /strategy/batch
│   │   └── checkpoint.rs  # File-based checkpoint save/load
│   └── Cargo.toml
│
├── myosu-cluster/         # AC-AP-01, AC-AP-03
│   ├── src/
│   │   ├── main.rs        # CLI (clap), pipeline orchestration
│   │   └── writer.rs      # Binary abstraction table writer + manifest
│   └── Cargo.toml
│
├── myosu-chain-client/    # Shared: used by miner, validator, play lanes
│   ├── src/
│   │   ├── lib.rs
│   │   ├── registration.rs  # register_neuron extrinsic wrapper
│   │   └── axon.rs          # serve_axon extrinsic wrapper
│   └── Cargo.toml
```

### 4.3 Chain Surfaces

| Surface | Chain location | Purpose |
|---------|---------------|---------|
| `register_neuron(subnet_id)` | Pallet: `game-solver` | Register miner UID on subnet |
| `serve_axon(subnet_id, ip, port, ...)` | Pallet: `game-solver` | Advertise axon endpoint on-chain |
| `Axons[subnet_id][hotkey]` | Storage: `game-solver` | Validators discover miner |
| `NeuronInfo.net_uid` | Storage: `game-solver` | Miner identity |

### 4.4 Local Files

| Path | Format | Purpose |
|------|--------|---------|
| `{data-dir}/abstractions/manifest.json` | JSON | Abstraction artifact manifest with SHA-256 |
| `{data-dir}/abstractions/{street}.lookup.bin` | Bincode | Isomorphism → Abstraction mapping per street |
| `{data-dir}/checkpoint/latest.bin` | Bincode | PokerSolver checkpoint |

---

## 5. Implementation Slices

Implement in order. Each slice is independently testable.

### Slice 1: `myosu-miner` CLI Skeleton + Chain Client Shell

**Goal**: `myosu-miner --help` works; `myosu-miner --chain ws://localhost:9944 --subnet 1 --key //Alice` prints "connecting..." without panicking.

**Owned files**:
- `crates/myosu-miner/Cargo.toml` — depends on `clap`, `tokio`, `subxt` (or `jsonrpsee`), `tracing`
- `crates/myosu-miner/src/main.rs` — clap CLI, early return if RPC unreachable
- `crates/myosu-chain-client/Cargo.toml` — shared chain client crate
- `crates/myosu-chain-client/src/lib.rs` — re-exports registration + axon wrappers
- `crates/myosu-chain-client/src/registration.rs` — `register_neuron()` extrinsic stub
- `crates/myosu-chain-client/src/axon.rs` — `serve_axon()` extrinsic stub

**Proof**: `cargo build -p myosu-miner` exits 0; `--help` shows expected flags.

**Blockers**: none within this slice.

---

### Slice 2: Training Loop (No Chain)

**Goal**: `myosu-miner` runs a background MCCFR training loop and serves `/health` via HTTP without touching the chain.

**Owned files**:
- `crates/myosu-miner/src/training.rs` — tokio task running `PokerSolver::train()` in a loop
- `crates/myosu-miner/src/axon.rs` — axum HTTP server, `/health` endpoint only
- `crates/myosu-miner/src/checkpoint.rs` — `PokerSolver::save_checkpoint()` / `load_checkpoint()`

**Observable**: `GET /health` returns `{"status": "ok", "epoch": N, "training_active": true}`.

**Proof**: Training loop runs for 1000 iterations; `/health` reflects increasing epoch count.

**Blockers**:
- R5 (abstraction artifact) — without abstractions, solver produces random strategies. Mock with default encoder for this slice.
- R6 (`NlheEncoder::from_dir()`) — without it, the training loop cannot load real abstractions. Use mock encoder for this slice.

---

### Slice 3: `myosu-cluster` Binary

**Goal**: `myosu-cluster --street preflop --output ./abstractions/` produces valid `manifest.json` and lookup files.

**Owned files**:
- `crates/myosu-cluster/Cargo.toml`
- `crates/myosu-cluster/src/main.rs` — CLI, street selection, pipeline orchestration
- `crates/myosu-cluster/src/writer.rs` — binary table writer + manifest JSON with SHA-256

**Observable**: `./abstractions/manifest.json` exists with correct SHA-256 hashes after running.

**Proof**: `myosu-cluster --street preflop` produces exactly 169-entry lookup file with 1:1 isomorphism mapping; `sha256sum` of output files matches manifest.

**Blockers**: none within this slice. Uses robopoker's existing clustering code (`rbp-clustering`).

---

### Slice 4: Full Registration + Serving Integration

**Goal**: `myosu-miner` registers on-chain and advertises axon; validators can query `/strategy`.

**Owned files**: All `crates/myosu-miner/src/*.rs` files from slices 1-2.

**Observable**:
- On-chain `Axons[subnet_id][hotkey]` storage entry exists after startup
- `GET /health` includes `"axon_registered": true`
- `POST /strategy` accepts and returns valid `WireStrategy` responses

**Proof**:
- `cargo build -p myosu-miner && cargo test -p myosu-miner` passes
- Integration test against a running devnet chain: registration succeeds, axon is queryable

**Blockers**:
- R3 (`chain:pallet` restart) — `register_neuron` and `serve_axon` extrinsics must exist and compile
- R4 (`chain:runtime` restart) — runtime must be buildable to run a devnet

---

### Slice 5: Abstraction Bootstrap Integration

**Goal**: On startup, `myosu-miner` detects missing abstractions and either downloads or computes them before training begins.

**Owned files**:
- `crates/myosu-miner/src/main.rs` — bootstrap sequence: check abstractions → download or compute → load encoder → start training
- `scripts/download-abstractions.sh` — fetch + verify artifact
- `artifacts/abstractions/manifest.json` — published artifact (or local dev copy)

**Observable**:
- Startup log: "Loading abstractions from {path}, encoder hash: {sha256}"
- If no abstractions found: "No abstractions found. Downloading..." or "Computing locally..."
- Encoder hash matches `total_sha256` in manifest

**Proof**: Clean start from empty `{data-dir}/` directory completes without error; encoder hash is stable across restarts.

**Blockers**:
- R5 (abstraction artifact) — artifact must exist at a URL or be computable via `myosu-cluster`
- R6 (`NlheEncoder::from_dir()` in robopoker fork) — must be merged / available

---

## 6. Lane Boundary

### What this lane OWNS

- `crates/myosu-miner/` — the miner binary
- `crates/myosu-cluster/` — the abstraction pipeline binary
- `crates/myosu-chain-client/` — shared chain interaction primitives (registration, axon serving)
- `scripts/download-abstractions.sh` — bootstrap download script
- `outputs/miner/service/spec.md` — this document
- `outputs/miner/service/review.md` — lane review artifact

### What this lane DOES NOT OWN

- `crates/myosu-chain/pallets/game-solver/` — chain pallet (owned by `chain:pallet` lane)
- `crates/myosu-chain/runtime/` — chain runtime (owned by `chain:runtime` lane)
- `robopoker` fork (RF-02) — `NlheEncoder::from_dir()` implementation
- Abstraction artifact hosting / publication — publication step is manual for now

### Interface Contracts

| Interface | Type | Defined by | Consumed by |
|-----------|------|-----------|-------------|
| `register_neuron(subnet_id)` | Extrinsic | `chain:pallet` | `myosu-chain-client` |
| `serve_axon(subnet, ip, port, ...)` | Extrinsic | `chain:pallet` | `myosu-chain-client` |
| `Axons[subnet][hotkey]` | Storage read | `chain:pallet` | Validator / operator |
| `NlheEncoder::from_dir(path)` | API | RF-02 (robopoker fork) | `myosu-miner` training |
| `PokerSolver::train()` | API | `myosu-games-poker` (PE-01) | `myosu-miner` training |
| `PokerSolver::handle_query()` | API | `myosu-games-poker` (PE-02) | `myosu-miner` axon |

---

## 7. Relationship to Other Lanes

| Lane | Dependency direction | Nature |
|------|---------------------|--------|
| `chain:pallet` | **Blocking upstream** | Provides `register_neuron` + `serve_axon` extrinsics; cannot compile |
| `chain:runtime` | **Blocking upstream** | Provides devnet chain to run integration tests against; cannot compile |
| `poker-engine` | **Upstream API** | Provides `PokerSolver` training and query interfaces |
| `sdk-core` | **Upstream API** | May provide shared types (`WireStrategy`, `AxonInfo`) |
| `validator-oracle` | **Downstream consumer** | Queries `/strategy` endpoint; will need matching health checks |

The lane can implement slices 1–3 without any upstream lane completing. Slice 4 requires `chain:pallet` restart to complete. Slice 5 requires the RF-02 `NlheEncoder::from_dir()` work.

---

## 8. Milestone Verification

| # | Scenario | Validates | Slice |
|---|----------|-----------|-------|
| 1 | `cargo build -p myosu-miner` exits 0 with CLI skeleton | Binary builds | Slice 1 |
| 2 | `GET /health` returns epoch + training_active after 1000 iterations | Training loop runs | Slice 2 |
| 3 | `myosu-cluster --street preflop` produces 169-entry file + valid manifest | Clustering binary | Slice 3 |
| 4 | Miner registers on devnet, axon discoverable via chain RPC | Full registration | Slice 4 |
| 5 | Miner starts from empty data-dir, loads abstractions, logs encoder hash | Abstraction bootstrap | Slice 5 |
| 6 | Two miners with same abstractions produce identical encoder hash | INV-003 alignment | Slice 5 |

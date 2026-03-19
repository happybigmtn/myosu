# `validator:oracle` Lane Spec

**Lane**: `validator:oracle`
**Date**: 2026-03-19
**Status**: Bootstrap — blocked upstream
**Depends-on**: `games:poker-engine` (exploitability functions), `chain:pallet` (game-solver pallet with Axons + set_weights), `miner:service` (axon HTTP protocol stable)

---

## 1. Lane Boundary and Operator-Visible Purpose

`validator:oracle` owns the **`myosu-validator`** binary — a long-running process that is the quality-control layer of the Myosu game-solving system.

**Operator-visible outcome**: Running `myosu-validator --chain ws://localhost:9944 --subnet 1 --key //Alice --stake 10000` against a live devnet with registered miners produces objective, deterministic exploitability scores that flow into Yuma Consensus via weight vector submission.

The validator converts miner strategies into objective scores. The exploitability oracle is the core innovation: a deterministic, reproducible measurement of how far a strategy is from Nash equilibrium, expressed in milli-big-blinds per hand (mbb/h).

**What this lane does NOT own**:
- The chain pallet or runtime (owned by `chain:pallet`, `chain:runtime`)
- The poker engine exploitability computation (owned by `games:poker-engine`)
- The miner binary and axon protocol (owned by `miner:service`)
- Any multi-game support beyond NLHE heads-up
- Slashing, reputation, or advanced anti-gaming logic

---

## 2. Service Ownership Surface

### 2.1 Binary and Crate

```
crates/myosu-validator/
├── Cargo.toml              # Workspace member; depends on myosu-games, myosu-games-poker, subxt
└── src/
    ├── main.rs             # CLI entry, evaluation loop orchestration
    ├── chain.rs            # Chain RPC client (subxt): stake, set_weights, commit/reveal
    ├── evaluator.rs        # Miner discovery and HTTP query orchestration
    ├── scoring.rs          # Exploitability → weight normalization
    ├── positions.rs        # Deterministic test position generation
    ├── submitter.rs        # Weight vector submission (direct + commit-reveal)
    └── lib.rs              # Re-exports for integration tests
```

### 2.2 Endpoints and Chain Surfaces Consumed

| Surface | Type | Source lane | Notes |
|---------|------|------------|-------|
| `ws://localhost:9944` | WebSocket RPC | `chain:runtime` | Chain node RPC endpoint |
| `Axons[subnet][hotkey]` | On-chain storage | `chain:pallet` | IP:port registry for miners |
| `set_weights(subnet, Vec<(uid, u16)>)` | Extrinsic | `chain:pallet` | Direct weight submission |
| `commit_weights(subnet, hash)` / `reveal_weights(...)` | Extrinsic pair | `chain:pallet` | Commit-reveal anti-gaming |
| `add_stake(subnet, amount)` | Extrinsic | `chain:pallet` | Validator staking |
| `Tempo[subnet]`, `Keys[subnet]` | On-chain constants | `chain:pallet` | Tempo period, validator UID |
| `POST /strategy` / `GET /health` | HTTP | `miner:service` | Miner axon protocol |
| `poker_exploitability()`, `remote_poker_exploitability()` | Rust fn | `games:poker-engine` | Core scoring |
| `NlheInfo`, `NlheEdge`, `NlheEncoder` | Rust types | `games:poker-engine` | Wire format |

### 2.3 Health and Determinism Surfaces

**Health while running**:
- Block subscription active (no missed blocks for >3 consecutive slots)
- All discovered miners queried within tempo window
- At least one weight submission transaction confirmed on-chain per tempo
- Zero panic/unwrap in main loop
- Log output every tempo showing `evaluated N miners, submitted weights`

**Determinism surface (INV-003)**:
- Test position generation: `hash(subnet_id, epoch_number, validator_hotkey)` → identical positions
- Encoder pinning: `NlheEncoder` hash logged on startup; two validators with same seed → identical scores within `1e-6` epsilon
- Scoring: `weight = max(0, 1 - exploit/baseline) * 65535` — pure function, no global state

---

## 3. Readiness Conditions Before Honest Bringup

The lane is **not ready to run** until all of the following are true:

### 3.1 Upstream Lanes Unblocked

| Blocker | Source lane | Criteria |
|---------|-------------|----------|
| `games:poker-engine` not created | `games:poker-engine` | `cargo build -p myosu-games-poker` exits 0 |
| `poker_exploitability()` / `remote_poker_exploitability()` missing | `games:poker-engine` | Both functions exist as `pub fn` in `myosu_games_poker` |
| `chain:pallet` restart in progress | `chain:pallet` | `cargo check -p pallet-game-solver` exits 0 |
| `Axons` storage, `set_weights`, `add_stake` not available | `chain:pallet` | Requires pallet restart to land |
| Axon HTTP protocol not stable | `miner:service` | `POST /strategy` and `GET /health` respond correctly per MN-03 |

### 3.2 Local Crate Available

| Criteria | Proof |
|----------|-------|
| `crates/myosu-validator/` exists as workspace member | `cargo metadata --format-version 1` lists `myosu-validator` |
| `Cargo.toml` has all required dependencies | `cargo fetch` succeeds |
| `src/lib.rs` exists with stub re-exports | Compiles without error |

### 3.3 Integration Points Verified

| Integration | Verification |
|-------------|--------------|
| `subxt` can connect to a running chain node | `cargo test -p myosu-validator chain::tests::connect_to_devnet` |
| `NlheEncoder` exposes `hash()` or equivalent fingerprint | Checked in `myosu-games-poker` exploit module |
| Deterministic position generation (same seed → same output) | Unit test: `positions::tests::deterministic_same_seed` |
| Exploitability function is pure (no interior mutability) | Code review + INV-003 integration test |

---

## 4. Health Surfaces Once Running

### 4.1 Operator Log Signals (required to emit every tempo)

```
# Template (repeated every tempo period):
[INFO] myosu_validator: tempo=142 subnet=1 uid=3 evaluated=8 miners skipped=1 epoch=3042
[INFO] myosu_validator: weights submitted (direct) n=8 hash=0xabc123...
[WARN] myosu_validator: miner 5 timeout after 5s (score=0)
[ERROR] myosu_validator: chain connection lost — reconnecting in 1s
```

### 4.2 Required Log Events

| Event | Level | Trigger |
|-------|-------|---------|
| Validator registered with UID | Info | Startup, after `add_stake` + `Keys` query |
| Epoch evaluated | Info | Every tempo |
| Miner scored | Debug | Per miner, per tempo |
| Miner skipped (timeout/invalid) | Warn | Per unresponsive miner |
| Weights submitted | Info | After `set_weights` or `reveal_weights` confirmed |
| Chain unreachable | Error | WebSocket connection lost |
| Determinism mismatch detected | Error | VO-03 test fails in production |

### 4.3 Determinism Checkpoints

- **Startup**: Log `encoder_hash=0x...` (SHA256 of encoder state) — must match across validators
- **Per-epoch**: Log `position_seed=...` and `n_positions=N` — reproducible from seed alone
- **Per-scorer**: Log `score=... exploitability=... mbb/h` for each miner

---

## 5. Proof Posture

The lane exposes **no external proof endpoint** in bootstrap. After bootstrap, the expected proof surfaces are:

| Proof surface | Type | Protected? |
|--------------|------|------------|
| `cargo test -p myosu-validator` | Unit tests | Public |
| `cargo test -p myosu-validator determinism::tests::two_validators_agree` | Integration test (INV-003) | Public — must pass to ship |
| `myosu-validator --dry-run --subnet 1` | Dry-run mode (no chain writes) | Public — for operator verification |
| `myosu-validator --help` | CLI surface | Public |

---

## 6. Next Implementation Slices (Smallest Honest First)

### Phase Gate 0: Upstream Dependency Check (no-op if upstream is ready)

Before writing any validator code, verify:
```bash
cargo build -p myosu-games-poker
cargo check -p pallet-game-solver
```
If either fails, this lane cannot proceed. Do not begin.

---

### Slice 1 — Create `myosu-validator` Crate Skeleton

**Files**: `crates/myosu-validator/Cargo.toml`, `crates/myosu-validator/src/lib.rs`

**Cargo.toml dependencies**:
- `subxt` (for chain RPC)
- `reqwest` (for HTTP miner queries)
- `myosu-games` (for `StrategyQuery`, `StrategyResponse`, `GameType`)
- `myosu-games-poker` (for `poker_exploitability`, `remote_poker_exploitability`, `NlheEncoder`)
- `tokio`, `futures` (async runtime)
- `clap` (CLI)
- `serde`, `serde_json`
- `tracing`, `tracing-subscriber`

**lib.rs**: Re-export stub modules initially.

**Proof**: `cargo build -p myosu-validator` exits 0.

---

### Slice 2 — Chain Client (`chain.rs`)

**File**: `crates/myosu-validator/src/chain.rs`

Implement:
- `ChainClient::new(rpc_url)` — subxt WebSocket client
- `ChainClient::get_tempo(subnet_id) -> u64` — query `Tempo[subnet]` storage
- `ChainClient::get_miner_axons(subnet_id) -> Vec<(Hotkey, AxonInfo)>` — query `Axons[subnet]`
- `ChainClient::get_validator_uid(hotkey, subnet_id) -> u16` — query `Keys[subnet][hotkey]`
- `ChainClient::add_stake(subnet_id, amount) -> TxHash` — submit `add_stake`
- `ChainClient::set_weights(subnet_id, weights: Vec<(u16, u16)>) -> TxHash`
- `ChainClient::commit_weights(subnet_id, hash) -> TxHash`
- `ChainClient::reveal_weights(subnet_id, uids, values, salt) -> TxHash`

**Proof**: `cargo test -p myosu-validator chain::tests::connect_and_query` (requires running devnet; mock in unit test).

---

### Slice 3 — Deterministic Test Positions (`positions.rs`)

**File**: `crates/myosu-validator/src/positions.rs`

Implement:
- `generate_test_positions(seed: u64, count: usize) -> Vec<NlheInfo>`
- Seed = `blake2b_32(&(subnet_id, epoch_number, hotkey))`
- Use `rand::rngs::SmallRng::seed_from_u64(seed)` for deterministic RNG
- Cover preflop, flop, turn, river (≥20% per street)
- All positions valid: non-empty choices, reachable game states

**Proof**: `cargo test -p myosu-validator positions::tests::deterministic_same_seed` and `positions::tests::covers_all_streets`.

---

### Slice 4 — Exploitability Scoring (`scoring.rs`)

**File**: `crates/myosu-validator/src/scoring.rs`

Implement:
```rust
pub fn score_miner(
    responses: &[(NlheInfo, Vec<(NlheEdge, Probability)>)],
    encoder: &NlheEncoder,
) -> u16 {
    let exploit = remote_poker_exploitability(
        |info| lookup_response(responses, info),
        encoder,
    );
    let normalized = 1.0 - (exploit / BASELINE_EXPLOITABILITY).min(1.0);
    (normalized * u16::MAX as f64) as u16
}
const BASELINE_EXPLOITABILITY: f64 = 1000.0; // mbb/h for random strategy
```

- Baseline = 1000 mbb/h (random strategy reference)
- Lower exploitability → higher weight
- Perfect Nash → 65535, random → 0
- Unresponsive miner → score 0

**Proof**: `cargo test -p myosu-validator scoring::tests::nash_strategy_max_weight` (mocked).

---

### Slice 5 — Miner Evaluator (`evaluator.rs`)

**File**: `crates/myosu-validator/src/evaluator.rs`

Implement:
- `Evaluator::discover_miners(chain: &ChainClient, subnet_id) -> Vec<DiscoveredMiner>`
- `Evaluator::query_miner(miner: &DiscoveredMiner, positions: &[NlheInfo]) -> QueryResult`
- `Evaluator::query_all_miners(miners, positions, timeout: Duration) -> Vec<QueryResult>`
- HTTP: `GET {axon.ip}/health` → verify alive; `POST {axon.ip}/strategy` with `NlheInfo` bytes
- Concurrent queries via `futures::stream::FuturesUnordered` with bounded concurrency
- 5s timeout per miner; skip on timeout or invalid response (score = 0)

**Proof**: `cargo test -p myosu-validator evaluator::tests::discover_miners` (mocked).

---

### Slice 6 — Weight Submitter (`submitter.rs`)

**File**: `crates/myosu-validator/src/submitter.rs`

Implement:
- `Submitter::submit_direct(chain: &ChainClient, subnet_id, weights) -> TxHash`
- `Submitter::submit_commit_reveal(chain: &ChainClient, subnet_id, weights, positions, salt) -> TxHash`
- Commit-reveal timing: commit at tempo T, reveal at tempo T+1
- Deadline: `tempo * 0.8` — if exceeded, submit partial weights (scored miners get weight, unscored get 0)

**Proof**: `cargo test -p myosu-validator submitter::tests::submit_direct_weights`.

---

### Slice 7 — Main Loop + CLI (`main.rs`)

**File**: `crates/myosu-validator/src/main.rs`

Implement:
- `clap` CLI: `--chain`, `--subnet`, `--key`, `--stake`, `--mode (direct|commit-reveal)`, `--positions-count`
- Main loop: poll chain every 6s; if `current_block % tempo == 0`, run evaluation cycle
- Evaluation cycle: discover → query → score → submit
- Graceful shutdown on SIGINT (flush logs, finish in-progress submission)
- Dry-run mode: `--dry-run` — queries miners and logs scores without chain writes

**Proof**: `cargo test -p myosu-validator main::tests::evaluation_loop_completes`.

---

### Slice 8 — INV-003 Two-Validator Determinism Test

**File**: `crates/myosu-validator/tests/determinism.rs`

**This is the no-ship condition.** The system cannot be considered correct without this test passing.

Implement integration test:
1. Start devnet with one trained miner
2. Spawn two independently-initialized validators with same seed derivation
3. Both query the same miner with positions from same seed
4. Both compute exploitability scores
5. Assert: `|score_a - score_b| < 1e-6`
6. Verify both logged identical `encoder_hash`

**Proof**: `cargo test -p myosu-validator determinism::tests::two_validators_agree` exits 0 within 60s.

---

## 7. Dependency Order (Phase Gate)

```
games:poker-engine (slices 1-6 complete)
         │
         ▼
chain:pallet (restart complete, pallet-game-solver builds)
         │
         ▼
miner:service (axon protocol stable)
         │
         ▼
myosu-validator (slices 1-8, in order)
```

Slices 1-2 (skeleton + chain client) can begin once `chain:pallet` restart is complete.
Slices 3-6 (positions, scoring, evaluator, submitter) require `games:poker-engine` complete.
Slice 7 (main loop) requires all of the above.
Slice 8 (INV-003 test) requires a running devnet with at least one trained miner.

---

## 8. Files This Lane Must Own

| File | Status |
|------|--------|
| `crates/myosu-validator/Cargo.toml` | Create |
| `crates/myosu-validator/src/lib.rs` | Create |
| `crates/myosu-validator/src/main.rs` | Create |
| `crates/myosu-validator/src/chain.rs` | Create |
| `crates/myosu-validator/src/evaluator.rs` | Create |
| `crates/myosu-validator/src/scoring.rs` | Create |
| `crates/myosu-validator/src/positions.rs` | Create |
| `crates/myosu-validator/src/submitter.rs` | Create |
| `crates/myosu-validator/tests/determinism.rs` | Create |
| `outputs/validator/oracle/spec.md` | This file |
| `outputs/validator/oracle/review.md` | Review artifact |

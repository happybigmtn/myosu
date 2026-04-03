# Specification: Operational RPCs — Chain Queries for System Screens

Source: DESIGN.md 10.1-10.4 operational screens
Status: Draft
Date: 2026-03-30
Depends-on: GS-01..10 (game-solving pallet), CF-01..11 (chain scaffold)
Blocks: DESIGN.md 10.1-10.4 implementation

## Purpose

DESIGN.md defines four operational screens (network console, miner
inspection, validator divergence, session summary) that display chain
state. These screens need custom RPC methods on the game-solving pallet
to query subnet health, miner scores, validator agreement, and emission
history.

Standard Substrate RPCs (`system_health`, `state_getStorage`) exist but
require callers to decode raw storage. Custom RPCs provide typed, ergonomic
access to game-solving-specific data.

## Current State

- `crates/myosu-chain/node/src/rpc.rs` already assembles the live node RPC
  module set and merges the current custom runtime-facing methods
- `crates/myosu-chain/pallets/game-solver/src/rpc_info/` already exposes typed
  helper structs and query builders for subnet, metagraph, stake, delegate, and
  neuron views
- `crates/myosu-chain/node/src/service.rs` already logs startup and RPC
  readiness timing as part of the current operator surface
- `crates/myosu-chain/node/tests/stage0_local_loop.rs` already proves the
  chain/gameplay/operator loop without the richer operational screens in
  `DESIGN.md`

## What Already Exists

| Sub-problem | Existing code / flow | Reuse / extend / replace | Why |
|-------------|----------------------|--------------------------|-----|
| Node RPC assembly | `crates/myosu-chain/node/src/rpc.rs` | extend | This is the real merge point for any new operator RPCs |
| Typed subnet and neuron views | `crates/myosu-chain/pallets/game-solver/src/rpc_info/` | reuse and extend | The pallet already owns typed query helpers here |
| Operator startup health | `crates/myosu-chain/node/src/service.rs` | reuse | Current observability surface should stay aligned with richer RPCs |
| End-to-end operator proof | `crates/myosu-chain/node/tests/stage0_local_loop.rs` | reuse | New RPCs should remain consistent with the owned stage-0 loop |

The honest gap is no longer "there is no custom chain query surface." The gap
is that the current query helpers and RPC assembly are narrower than the richer
operational screens described in `DESIGN.md`.

## Scope

## Acceptance Criteria

### AC-OR-01: Subnet Field RPC

- Where: `crates/myosu-chain/node/src/rpc.rs` plus
  `crates/myosu-chain/pallets/game-solver/src/rpc_info/`
- How: Custom RPC method returning the full subnet field for the network
  console (DESIGN.md 10.1):

  ```rust
  #[rpc(server)]
  pub trait GameSolverApi<BlockHash> {
      #[method(name = "gameSolver_subnetField")]
      fn subnet_field(&self, at: Option<BlockHash>) -> RpcResult<Vec<SubnetInfo>>;
  }

  #[derive(Serialize, Deserialize)]
  pub struct SubnetInfo {
      pub id: u16,
      pub game_type: String,
      pub miners: u16,
      pub best_exploit: Option<f64>,
      pub agreement: Option<f64>,      // % validator agreement
      pub status: SubnetStatus,
  }

  pub enum SubnetStatus { Active, Bootstrap, Frozen }
  ```

  The RPC reads from on-chain storage: `SubnetRegistry`, `NeuronCount`,
  `Weights`, and `Emissions`. It computes `agreement` as the standard
  deviation of validators' weight vectors — low stddev = high agreement.

  `best_exploit` is derived from the highest-weighted miner's most recent
  score. This requires storing the raw exploitability alongside weights
  (or recomputing from weight + baseline).

- Required tests:
  - `rpc::tests::subnet_field_returns_all_subnets`
  - `rpc::tests::subnet_field_includes_miner_count`
  - `rpc::tests::empty_subnet_shows_bootstrap`
- Pass/fail:
  - Returns one entry per registered subnet
  - Miner count matches `NeuronCount` storage
  - Subnet with 0 miners has status `Bootstrap`

### AC-OR-02: Miner Inspection RPC

- Where: `crates/myosu-chain/node/src/rpc.rs` plus
  `crates/myosu-chain/pallets/game-solver/src/rpc_info/`
- How: Custom RPC for miner detail view (DESIGN.md 10.2):

  ```rust
  #[method(name = "gameSolver_minerInfo")]
  fn miner_info(
      &self,
      subnet_id: u16,
      uid: u16,
      at: Option<BlockHash>,
  ) -> RpcResult<MinerInfo>;

  #[derive(Serialize, Deserialize)]
  pub struct MinerInfo {
      pub uid: u16,
      pub subnet_id: u16,
      pub hotkey: AccountId,
      pub stake: u128,
      pub exploitability: Option<f64>,
      pub trend: Vec<EpochScore>,       // last N epochs
      pub latency_ms: Option<u32>,      // from last validator query
      pub last_checkpoint: Option<u64>, // block number
  }

  pub struct EpochScore {
      pub epoch: u32,
      pub exploitability: f64,
      pub delta: f64,
  }
  ```

  `trend` requires storing per-epoch scores. Options:
  1. Store last N scores on-chain (increases storage, simplest)
  2. Index from events (requires off-chain indexer)
  3. Validator stores locally and exposes via its own RPC

  **Decision**: store last 10 epoch scores on-chain per neuron. Storage
  cost: 10 × 16 bytes × max_neurons_per_subnet. For 256 neurons, that's
  40KB per subnet — acceptable. At 20 subnets, total epoch storage is
  ~800KB.

  **Pruning**: when a neuron is deregistered (pruned or voluntarily exits),
  its epoch history is deleted from storage in the same extrinsic. This
  prevents ghost data from accumulating for neurons that no longer exist.

- Required tests:
  - `rpc::tests::miner_info_returns_valid_data`
  - `rpc::tests::miner_info_includes_epoch_history`
  - `rpc::tests::nonexistent_miner_returns_error`
- Pass/fail:
  - Returns miner info for valid (subnet, uid)
  - Epoch history has up to 10 entries, sorted by epoch descending
  - Invalid uid returns clear error

### AC-OR-03: Validator Divergence RPC

- Where: `crates/myosu-chain/node/src/rpc.rs` plus
  `crates/myosu-chain/pallets/game-solver/src/rpc_info/`
- How: Custom RPC for consensus monitoring (DESIGN.md 10.3):

  ```rust
  #[method(name = "gameSolver_validatorDivergence")]
  fn validator_divergence(
      &self,
      subnet_id: u16,
      at: Option<BlockHash>,
  ) -> RpcResult<DivergenceReport>;

  #[derive(Serialize, Deserialize)]
  pub struct DivergenceReport {
      pub subnet_id: u16,
      pub threshold: f64,
      pub observed: f64,
      pub severity: Severity,
      pub validators: Vec<ValidatorScore>,
  }

  pub struct ValidatorScore {
      pub uid: u16,
      pub score_current: f64,
      pub score_previous: f64,
      pub divergence: f64,
      pub status: ValidatorStatus,
  }

  pub enum ValidatorStatus { Ok, Outlier }
  pub enum Severity { S0, S1, S2, S3 }
  ```

  Divergence = maximum pairwise distance between validators' weight
  vectors for the same subnet. If divergence exceeds threshold, the
  report includes `severity` and flags outlier validators.

  ```rust
  fn compute_divergence(weights: &[Vec<(u16, u16)>]) -> f64 {
      // For each pair of validators, compute L2 distance of weight vectors
      // Return the maximum pairwise distance
      // Complexity: O(V^2 * M) where V = validators, M = miners
      // At V=100, M=256: ~1.3M ops, <1ms. Acceptable for on-demand RPC.
  }
  ```

- Required tests:
  - `rpc::tests::divergence_zero_for_identical_weights`
  - `rpc::tests::divergence_flags_outlier`
  - `rpc::tests::severity_scales_with_divergence`
- Pass/fail:
  - All validators submit identical weights → divergence = 0, severity S0
  - One validator submits very different weights → flagged as Outlier
  - Severity increases with divergence magnitude

### AC-OR-04: Signal Stream RPC

- Where: `crates/myosu-chain/node/src/rpc.rs` plus existing node event/query
  surfaces
- How: Subscription-based RPC for the signal log in operational screens:

  ```rust
  #[subscription(name = "gameSolver_subscribeSignals", item = Signal)]
  fn subscribe_signals(&self, subnet_id: Option<u16>);

  #[derive(Serialize, Deserialize)]
  pub struct Signal {
      pub timestamp: u64,
      pub subnet_id: u16,
      pub message: String,
      pub level: SignalLevel,
  }

  pub enum SignalLevel { Info, Warning, Error }
  ```

  Signals are derived from on-chain events:
  - `WeightsFinalized` → "subnet 1 weights finalized"
  - `NeuronRegistered` → "subnet 3 miner 7 registered"
  - `DivergenceDetected` → "subnet 2 validator divergence 3.2e-4"
  - `EmissionsDistributed` → "epoch 1482 emissions distributed"

  If `subnet_id` is None, subscribe to all subnets. The TUI network
  console (DESIGN.md 10.1) uses this for the SIGNAL panel.

- Required tests:
  - `rpc::tests::signal_emitted_on_weight_finalization`
  - `rpc::tests::signal_filtered_by_subnet`
  - `rpc::tests::subscription_receives_new_signals`
- Pass/fail:
  - Weight finalization produces an Info signal
  - Divergence detection produces a Warning signal
  - Subscriber with subnet_id=1 only receives subnet 1 signals

---

## TUI integration

Each operational screen (DESIGN.md 10.1-10.4) maps to one or more RPCs:

| Screen | RPCs used | Refresh |
|--------|-----------|---------|
| 10.1 Network Console | `subnet_field` + `subscribe_signals` | Live (subscription) |
| 10.2 Miner Inspection | `miner_info` | On demand ([r] refresh) |
| 10.3 Validator Divergence | `validator_divergence` | On demand |
| 10.4 Session Summary | (local data, no RPC needed) | Static |

The TUI connects to the chain RPC via `ws://` and maintains the
subscription. If the chain connection drops, operational screens show
`CHAIN UNREACHABLE` declaration per the existing fallback pattern.

**Reconnection**: on connection drop, the TUI attempts reconnection with
exponential backoff (1s, 2s, 4s, 8s, max 30s). On reconnect, all active
subscriptions are re-established. The declaration changes back from
`CHAIN UNREACHABLE` to the normal state once the connection is restored.

## Decision log

- 2026-03-17: Custom RPCs over raw storage reads. Rationale: operational
  screens need computed values (agreement %, divergence, epoch trends) that
  would require client-side computation from raw storage. Better to compute
  on the node.
- 2026-03-17: Store last 10 epoch scores on-chain. Rationale: simplest path
  for trend display. Off-chain indexing adds infrastructure complexity that
  isn't justified for 10 data points per neuron.
- 2026-03-17: Subscription-based signal stream. Rationale: the network
  console needs live updates. Polling would add unnecessary latency and load.

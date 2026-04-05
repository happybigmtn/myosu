# Myosu System Specification

Date: 2026-04-05

---

## System Identity

Myosu is a decentralized game-solving protocol for imperfect-information games.
The system produces, scores, and distributes Nash-approximate strategy through
an incentive-aligned network of miners, validators, and a coordination chain.

묘수 means "brilliant move" or "masterstroke."

---

## System Shape

```
┌──────────────────────────────────────────────────┐
│ CHAIN (Substrate, opentensor polkadot-sdk fork)  │
│                                                  │
│  pallet-game-solver                              │
│    subnet registry → neuron registry → weights   │
│    emission distribution ← Yuma Consensus        │
│    commit-reveal v2 (hash-based)                 │
│    staking (share-pool, single token model)      │
│    registration (burned, no PoW)                 │
│    no-op swap stub (identity, 1:1)               │
└──────────────────┬───────────────────────────────┘
                   │ RPC (WebSocket, JSON-RPC)
        ┌──────────┼──────────┐
        ▼          ▼          ▼
   ┌─────────┐ ┌──────────┐ ┌──────────────┐
   │ MINER   │ │VALIDATOR │ │ GAMEPLAY      │
   │         │ │          │ │              │
   │ MCCFR   │ │ L1 dist  │ │ TUI / Pipe   │
   │ trainer │◄┤ scoring  │ │ agent=human  │
   │ HTTP    │ │ weights  │ │ discovery    │
   │ axon    │ │ submit   │ │ live advice  │
   └─────────┘ └──────────┘ └──────────────┘
```

### Crate Map

| Crate | Role | Binary? |
|-------|------|---------|
| `myosu-chain` | Substrate node binary | Yes |
| `myosu-chain-runtime` | WASM runtime | No (wasm) |
| `pallet-game-solver` | On-chain game-solving coordination | No (pallet) |
| `myosu-chain-client` | Shared RPC/storage/extrinsic client | No (library) |
| `myosu-games` | Game trait re-exports, wire types | No (library) |
| `myosu-games-poker` | Poker solver, wire codec, renderer | No (library) |
| `myosu-games-liars-dice` | Liar's Dice solver, wire codec, renderer | No (library) |
| `myosu-games-kuhn` | Kuhn Poker solver, wire codec, renderer | No (library) |
| `myosu-miner` | Off-chain strategy producer | Yes |
| `myosu-validator` | Off-chain quality scorer | Yes |
| `myosu-play` | Gameplay surface (TUI, pipe, smoke) | Yes |
| `myosu-keys` | Operator key management | Yes |
| `myosu-tui` | Terminal UI shell and renderer framework | No (library) |

---

## Core Behaviors (Verified Against Code)

### 1. Chain Coordination

The chain runs a Substrate node with Aura/GRANDPA consensus. The stage-0
default runtime includes ~9 pallets. `pallet-game-solver` provides:

- **Subnet registration**: `register_network` creates a subnet. Subnet owners
  can configure tempo, immunity period, and other hyperparameters.
- **Neuron registration**: `burned_register` registers a neuron on a subnet
  by burning tokens. No proof-of-work.
- **Staking**: `add_stake` stakes tokens to a hotkey. Share-pool model with
  single-token economics (no Alpha/TAO dual-token).
- **Weight submission**: `commit_weights` / `reveal_weights` implements
  commit-reveal v2 (hash-based). Validators commit weight hashes, then reveal
  actual weights after the reveal window.
- **Axon serving**: `serve_axon` publishes a miner's HTTP endpoint on-chain.
- **Epoch processing**: Yuma Consensus runs per-subnet at each tempo interval.
  Weights are clipped, normalized, and used to compute incentive/dividend
  distributions. Emissions are distributed to miners and validators proportional
  to measured quality.
- **Coinbase**: Per-block emission distributed across subnets based on
  accumulated pending emission, then distributed within subnets per epoch output.

**No-op swap stub**: All swap operations (TAO↔Alpha) are identity conversions
with zero fees. This is an intentional stage-0 simplification. The full AMM
path is inherited but unused.

### 2. Mining

The miner binary (`myosu-miner`):

1. Probes chain for subnet state
2. Optionally registers and publishes axon endpoint
3. Runs bounded MCCFR training batch (configurable iterations)
4. Saves checkpoint to disk
5. Serves strategy via one-shot file or persistent HTTP axon

The MCCFR engine comes from the robopoker fork (`happybigmtn/robopoker`).
Training produces a `Profile` mapping information sets to action distributions.
The solver supports Liar's Dice through the same trait interface.

### 3. Validation

The validator binary (`myosu-validator`):

1. Loads its own solver from checkpoint + encoder
2. Reads a wire-encoded strategy query and miner response
3. Computes the expected response locally
4. Measures L1 distance between expected and observed distributions
5. Converts to score: `1.0 / (1.0 + l1_distance)`
6. Exact match (L1 < epsilon) scores 1.0
7. Optionally submits weights on-chain via commit-reveal

Determinism (INV-003): Given identical checkpoint, encoder, and query, any
two validators produce identical scores within floating-point epsilon (< 1e-6).

### 4. Gameplay

The play surface (`myosu-play`) has three modes:

- **Smoke test**: Scripted hand progression proving the full surface works.
  Runs poker through PREFLOP→FLOP→TURN→RIVER→complete. Also proves Kuhn
  and Liar's Dice surfaces.
- **Train mode**: Interactive TUI with ratatui. Shows game state, legal actions,
  solver advisor overlay showing action distributions. Supports `/deal`, `/quit`.
- **Pipe mode**: Line-oriented text protocol for agent consumption. Outputs
  structured `STATE`, `ACTION`, `CLARIFY`, `ERROR`, `QUIT` messages.

Discovery: When `--chain` and `--subnet` are provided, the play surface discovers
the highest-incentive miner on-chain, queries its axon for live strategy advice,
and overlays it on the game display with staleness tracking.

### 5. Game Trait Interface

Games implement the robopoker CFR traits:
- `CfrGame`: Root state, apply edge, utility computation
- `CfrTurn`: Player turn with available edges
- `CfrInfo`: Information set (player-visible state)
- `CfrEdge`: Action type
- `Profile`: Strategy mapping (info → edge → probability)
- `Encoder`: Information set abstraction

Myosu wraps these with:
- `GameType`: Enum identifying the game variant
- `GameConfig`: Typed configuration (stack size, dice count, etc.)
- `StrategyQuery<I>` / `StrategyResponse<E>`: Wire types for miner-validator communication
- `GameRenderer` trait: TUI/pipe rendering interface

Adding a new game requires:
1. Implement the CFR traits for the game
2. Add a `GameType` variant
3. Implement `GameRenderer` for TUI/pipe output
4. Register wire codec encode/decode functions
5. No changes to existing game code (proven with Liar's Dice and Kuhn)

---

## Token Model

Stage-0 uses a single token (MYOSU, inheriting the TAO balance type). All
swap operations are 1:1 identity. Registration burns tokens directly.
Staking is direct token locking.

The inherited dual-token (TAO + subnet Alpha) model exists in code but is
dormant behind the no-op swap stub. Future token economics are an open
design question, not a stage-0 deliverable.

---

## Network Model

Stage-0 is a single-node local devnet. The chain supports:
- `devnet`: Single-authority local chain (Alice)
- `test_finney`: Multi-authority testnet chain spec

Multi-node support is partially proven (`two_node_sync.sh` demonstrates
peer discovery and block sync between two nodes). GRANDPA finality under
network partition is not yet tested.

---

## Invariant Summary

| ID | Statement | Enforcement |
|----|-----------|-------------|
| INV-001 | Structured closure honesty | Plan/gate proof commands |
| INV-002 | Proof honesty (no false greens) | Adjudicator proof gating |
| INV-003 | Validator determinism (epsilon < 1e-6) | Deterministic PRNG, canonical serialization |
| INV-004 | Solver-gameplay separation | `cargo tree` dependency check in CI |
| INV-005 | Plan/land coherence | Release gate surfaces |
| INV-006 | Robopoker fork coherence | Documented fork changelog |

---

## External Dependencies

| Dependency | Source | Risk |
|------------|--------|------|
| `polkadot-sdk` | opentensor fork, rev `71629fd` | Fork may diverge from upstream |
| `substrate-fixed` | encointer fork, tag `v0.6.0` | Required for bit-identical Yuma output |
| `robopoker` (rbp_*) | happybigmtn fork | MCCFR engine; must track INV-006 |
| `tle` (timelock) | ideal-lab5, rev `5416406` | Used by inherited drand pallet (feature-gated) |

---

## What This Spec Does Not Cover

- Token economics beyond single-token identity model
- Multi-node production deployment
- Web, mobile, or hosted gameplay surfaces
- Public testnet or mainnet operations
- Hardware requirements for full NLHE encoder (7-11 GB)
- Governance, upgrades, or runtime migration

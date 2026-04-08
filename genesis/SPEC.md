# Myosu System Specification

Generated: 2026-04-07
Source of truth: trunk @ 4e0b37f

---

## What Myosu Is

Myosu is a Substrate-based blockchain that coordinates decentralized computation of Nash-approximate strategies for imperfect-information games. The system incentivizes quality through a modified Yuma Consensus mechanism: miners compete on strategy quality, validators measure it deterministically, and the chain distributes token emissions proportional to measured quality.

The name 묘수 means "brilliant move" or "masterstroke."

## System Layers

### Chain Layer

A Substrate blockchain using Aura block authoring and GRANDPA finality. The runtime is a stripped fork of Bittensor's subtensor, reduced to the pallets needed for stage-0 game-solving coordination.

**Active runtime pallets (stage-0 default build):**
- System (0), Timestamp (2), Aura (3), Grandpa (4), Balances (5), TransactionPayment (6), SubtensorModule/pallet-game-solver (7), Utility (11), AdminUtils (19)

**Consensus:** 4-authority devnet with Aura slots and GRANDPA finality. Finality threshold requires all but floor((n-1)/3) authorities.

**Economics (stage-0):** Single-token model. NoOpSwap identity stub at all 37 swap callsites. Block emission distributed to subnets, then within subnets to server (miner) and validator dividends via Yuma Consensus output. Owner cut taken before distribution.

**Chain specs:** localnet (single authority), devnet (4 named authorities), test_finney, finney.

### Game Engine Layer

A trait-based game abstraction rooted in robopoker's CFR framework.

**Core traits (from `myosu-games`):**
- `CfrGame` — defines game tree traversal
- `CfrEdge` — action at a decision node
- `CfrInfo` — information set key
- `CfrTurn` — player turn or chance node
- `Profile` — strategy profile (probability distribution over actions per info set)
- `Encoder` — information set abstraction mapping

**Additional myosu traits:**
- `GameConfig` — typed game parameters
- `GameType` — `#[non_exhaustive]` enum with on-chain byte encoding
- `StrategyQuery` / `StrategyResponse` — miner-validator wire types
- `GameRenderer` — TUI rendering interface
- `GameRegistry` — runtime game discovery (currently 4 types)

**Implemented games:**

| Game | Crate | Solver | Renderer | Wire Codec | Status |
|------|-------|--------|----------|------------|--------|
| NLHE Heads-Up | `myosu-games-poker` | robopoker `NlheSolver` | Full (cards, actions, pot) | bincode | Complete |
| Kuhn Poker | `myosu-games-kuhn` | Native MCCFR | Full | bincode | Complete |
| Liar's Dice | `myosu-games-liars-dice` | Native MCCFR | Full (dice, claims) | bincode | Complete |
| NLHE 6-Max | registered in GameType | None | None | None | Placeholder |

### Miner Layer

Binary: `myosu-miner`. Lifecycle:

1. **Probe** — connect to chain, verify subnet exists, fetch neuron info
2. **Register** — burned registration extrinsic if not already registered
3. **Publish axon** — announce IP:port on-chain for validator discovery
4. **Train** — bounded MCCFR training batch (`--train-iterations`), writes checkpoint
5. **Checkpoint** — bincode-serialized solver state, versioned with 4-byte magic
6. **File-serve** — write strategy response to disk for validator consumption
7. **HTTP-serve** — poker-only HTTP axon serving strategy responses

Training works for all three games. HTTP serving works for poker only. Liar's Dice and Kuhn use file-based query/response.

### Validator Layer

Binary: `myosu-validator`. Scoring algorithm:

1. Load miner checkpoint and encoder artifacts
2. Decode strategy query from saved file
3. Compute expected response from local solver
4. Compare observed vs expected action distributions
5. Score: `score = 1.0 / (1.0 + l1_distance)` (hyperbolic formula)
6. Submit weight on-chain reflecting score

Determinism is proven for both poker and Liar's Dice via E2E test.

### Gameplay Layer

Binary: `myosu-play`. Three modes:

1. **Smoke test** (`--smoke-test`) — automated proof that gameplay surface works
2. **TUI** (`train` subcommand) — ratatui-based interactive poker, Kuhn, or Liar's Dice
3. **Pipe** (`pipe` subcommand) — structured stdin/stdout for agent consumption

The TUI shell (`myosu-tui`) provides screen management, input handling, theming, and event processing. Blueprint loading from artifact directories powers the bot opponent.

### Key Management Layer

Binary: `myosu-keys`. Operations: create, import-keyfile, import-mnemonic, import-raw-seed, export-active-keyfile, show-active, switch-active, change-password, list, print-bootstrap. All storage is network-namespaced under `--config-dir`.

### Chain Client Layer

Library: `myosu-chain-client`. Wraps Substrate JSON-RPC for:
- System health, RPC methods, neuron info
- Registration, axon serving, staking, weight submission
- Block subscription, account queries

## Key Behavioral Contracts

### Emission Flow (per block)

1. Block authored by Aura authority
2. `on_finalize` triggers `run_coinbase(block_emission)`
3. Block emission split across subnets by weight
4. Per-subnet emission accumulated in `PendingEmission`
5. When tempo fires, `drain_pending` distributes accumulated emission
6. Yuma Consensus (`epoch()` or `epoch_dense()`) computes incentives/dividends
7. Dividends split: owner cut, server (miner) share, validator share
8. Token balances updated; `TotalIssuance` increased

**Truncation behavior:** U96F32 to u64 floor conversion loses at most 2 rao per accrued block. No dust accumulation policy exists yet (WORKLIST EM-DUST-001).

### Strategy Query-Response Protocol

1. Validator constructs `StrategyQuery` with a game state
2. Query serialized via game-specific wire codec (bincode, 1 MiB decode budget)
3. Miner deserializes, evaluates solver against encoded game state
4. Miner constructs `StrategyResponse` with action probability distribution
5. Response serialized, returned to validator
6. Validator compares against local solver evaluation

### Invariant Surface

Six invariants defined in INVARIANTS.md. All are actively enforced:

| ID | Name | Enforcement |
|----|------|-------------|
| INV-001 | Structured Closure Honesty | Plan/review process |
| INV-002 | Proof Honesty | CI gates |
| INV-003 | Game Verification Determinism | `validator_determinism.sh`, unit tests |
| INV-004 | Solver-Gameplay Separation | CI `cargo tree` check |
| INV-005 | Plan And Land Coherence | Review process |
| INV-006 | Robopoker Fork Coherence | Advisory CI check (`continue-on-error`) |

## Dependency Pinning

| Dependency | Source | Pin | Notes |
|-----------|--------|-----|-------|
| polkadot-sdk | opentensor fork | rev `71629fd` | Contains subtensor-specific patches. 21 fork-only commits per ADR 009. |
| robopoker | happybigmtn fork | workspace rev (tracked by INV-006) | Needs serde, encoder constructors, clustering API, file checkpoints. |
| substrate-fixed | encointer fork | tag v0.6.0 | Required for bit-identical Yuma fixed-point math. |
| tle (timelock) | ideal-lab5 | rev `5416406` | For commit-reveal. Minimal usage in stage-0. |
| w3f-bls | opentensor fork | branch `fix-no-std` | BLS primitives for consensus. |

## Not Doing (Explicit)

These are intentionally out of scope for the current planning horizon:

- Production deployment or mainnet operations
- Web or mobile gameplay frontend
- Full AMM token economics (NoOpSwap is the stage-0 contract)
- Game portfolio expansion beyond the three proven games
- Governance mechanisms (DAO, on-chain voting, council)
- Runtime upgrade migration paths (no deployed state to migrate)
- Benchmarking and weight calibration (not needed until production)
- Public testnet or hosted infrastructure
- Multi-language SDK or client library

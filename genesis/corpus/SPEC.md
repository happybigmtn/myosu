# Myosu Genesis Specification

Status: Active
Date: 2026-04-02
Supersedes: `genesisarchive/SPEC.md` (2026-03-28)

## What Myosu Is

Myosu (묘수 -- "brilliant move") is a decentralized game-solving chain for
imperfect-information games. It connects four layers:

1. **Chain** -- on-chain coordination (subnets, neurons, weights, emissions) via a
   Substrate-based blockchain forked from Bittensor's subtensor.
2. **Miners** -- off-chain strategy computation via MCCFR (Monte Carlo
   Counterfactual Regret Minimization), served over HTTP.
3. **Validators** -- off-chain quality scoring via exploitability measurement,
   submitting weights to the chain for Yuma Consensus.
4. **Gameplay** -- human/agent consumption through a terminal UI or pipe protocol.

The core proposition: anyone can train a game solver, anyone can verify its
quality, the chain distributes economic rewards proportional to measured quality,
and humans can play against the best available strategy.

## Who It Is For

**Today (stage-0):** A solo protocol engineer / founder-operator building a
provable demo of the full train -> score -> emit -> play loop.

**Tomorrow (stage-1+):** Operators running miners and validators on a public
devnet, competing for emissions by producing higher-quality game strategies.

**Eventually:** Players, poker training platforms, and game-solving researchers
consuming verified strategies via API or embedded gameplay.

## Current Architecture

```
                          +-----------------+
                          |  myosu-chain    |  Substrate node
                          |  (242K lines)   |  pallet-game-solver at index 7
                          +--------+--------+
                                   |
                    +--------------+--------------+
                    |                             |
            +-------+-------+            +-------+-------+
            |  myosu-miner  |            | myosu-validator|
            |  (1.4K lines) |            |  (1.6K lines)  |
            +-------+-------+            +-------+-------+
                    |                             |
                    |   myosu-chain-client        |
                    |   (2.6K lines, shared)      |
                    +--------------+--------------+
                                   |
                          +--------+--------+
                          |  myosu-play     |  TUI / pipe mode
                          |  (2.7K lines)   |
                          +--------+--------+
                                   |
                    +--------------+--------------+
                    |                             |
            +-------+-------+            +-------+-------+
            | myosu-games-  |            | myosu-games-  |
            |    poker      |            |  liars-dice   |
            | (9.4K lines)  |            | (2.1K lines)  |
            +-------+-------+            +-------+-------+
                    |                             |
                    +--------------+--------------+
                                   |
                          +--------+--------+
                          |  myosu-games    |  Game traits
                          |  (641 lines)    |
                          +--------+--------+
                                   |
                          +--------+--------+
                          |  myosu-tui      |  Terminal UI shell
                          |  (3.2K lines)   |
                          +--------+--------+
                                   |
                          +--------+--------+
                          |  myosu-keys     |  Key management
                          |  (2.0K lines)   |
                          +--------+--------+
```

### Workspace Crates

| Crate | Type | Purpose |
|-------|------|---------|
| `myosu-games` | lib | Game-agnostic CFR trait re-exports and `GameType`/`GameConfig` |
| `myosu-games-poker` | lib | NLHE state machine, renderer, solver, wire protocol, artifacts |
| `myosu-games-liars-dice` | lib | Liar's Dice state, renderer, solver, wire protocol |
| `myosu-tui` | lib | Game-agnostic terminal UI shell (ratatui + crossterm) |
| `myosu-play` | bin | Interactive gameplay: TUI, pipe mode, smoke tests |
| `myosu-chain-client` | lib | JSON-RPC client shared by miner, validator, and play |
| `myosu-miner` | bin | MCCFR training, checkpoint, HTTP axon serving |
| `myosu-validator` | bin | Exploitability scoring, weight submission |
| `myosu-keys` | bin | Key creation, encryption, import/export, operator bootstrap |
| `myosu-chain` | bin | Substrate node binary |
| `myosu-chain-runtime` | lib | Runtime definition with pallet-game-solver |
| `pallet-game-solver` | lib | Stage-0 core pallet (renamed pallet-subtensor fork) |

### Chain Pallets (11 total)

| Pallet | Status | Notes |
|--------|--------|-------|
| `pallet-game-solver` | Active | Stage-0 core, game solving coordination |
| `pallet-admin-utils` | Inherited | Admin utilities |
| `pallet-crowdloan` | Inherited | Not used in stage-0 |
| `pallet-drand` | Inherited | DRAND oracle, BLS signatures -- CRV3 stripped |
| `pallet-proxy` | Inherited | Proxy mechanism |
| `pallet-registry` | Inherited | Entity registry |
| `pallet-subtensor` | Inherited | Main subtensor pallet (game-solver superset) |
| `pallet-swap` | Inherited | Token swap -- no-op stub |
| `pallet-swap-interface` | Inherited | Swap trait definitions |
| `pallet-transaction-fee` | Inherited | Custom transaction fees |
| `pallet-utility` | Inherited | Utility functions |

## Tech Stack

| Layer | Technology | Version/Pin |
|-------|-----------|-------------|
| Language | Rust | Stable (via rust-toolchain.toml) |
| Format | Edition 2024 | rustfmt.toml |
| Chain | Polkadot SDK | Opentensor fork (git pin) |
| EVM | Frontier | Opentensor fork (git pin) |
| Game solver | robopoker | happybigmtn fork (git rev pin) |
| Crypto | libsecp256k1, sha2, XSalsa20Poly1305 | Latest compatible |
| Async | tokio | Workspace version |
| TUI | ratatui + crossterm | Workspace version |
| Serialization | serde, parity-scale-codec | Workspace version |
| CI | GitHub Actions | 7 jobs on trunk/main |
| Automation | Fabro | fabro.toml, programs, workflows |
| Research | Python 3 + numpy | Root-level experiment files |

## Major Decisions Already Made

| Decision | Rationale | Reference |
|----------|-----------|-----------|
| Substrate chain fork from Bittensor | Yuma Consensus for quality-weighted emissions | `THEORY.MD` |
| Single-token model (no dual Alpha/TAO) | AMM complexity for zero stage-0 value | `THEORY.MD` |
| ArcSwap double-buffer for miner | Zero read contention during training | `THEORY.MD` |
| Enum dispatch (not trait objects) for CFR | CFR traits require Copy+Sized | `crates/myosu-games/src/traits.rs` |
| robopoker fork (not upstream dep) | Need serde, encoder constructors, clustering API | INV-006, `docs/robopoker-fork-changelog.md` |
| `myosu-chain-client` shared seam | Prevents DRY across miner/validator/play | `THEORY.MD` |
| SwapInterface no-op stub | Registration/staking/emission call it | `crates/myosu-chain/pallets/swap/` |
| Commit-reveal v2 only (hash-based) | CRV3 timelock depends on stripped pallet_drand | `THEORY.MD` |
| Checkpoint versioning (4-byte magic) | Prevent silent corruption on format changes | `crates/myosu-games-poker/src/artifacts.rs` |
| Pipe mode for agent integration | Structured I/O for non-human consumers | `crates/myosu-play/src/main.rs` |

## Stage-0 Exit Criteria

From `OS.md` -- myosu stays in stage 0 until ALL of these are true:

1. Chain compiles and produces blocks on local devnet
2. pallet-game-solver integrated at runtime index 7
3. At least one poker subnet registers and runs solver evaluation
4. One miner produces strategy from robopoker MCCFR
5. One validator computes deterministic quality and submits weights
6. Two validators score the same miner identically (INV-003)
7. Yuma-style economics distribute emissions by quality
8. Human can play poker hand against trained strategy
9. Local training surface works with blueprint bot + solver advisor
10. Second game (Liar's Dice) works without poker code changes
11. Gameplay and miner cleanly separated (INV-004)
12. Emission accounting stays green + all invariants pass

**Current status:** Criteria 1-5, 8-11 are verified by code and CI. Criteria 6,
7, 12 require runtime verification on a live chain.

## Invariants

Six hard rules govern the project (see `INVARIANTS.md` for full definitions):

- **INV-001:** Structured Closure Honesty -- no task complete without trusted outcome
- **INV-002:** Proof Honesty -- named proofs must execute honestly
- **INV-003:** Game Verification Determinism -- validators agree within epsilon
- **INV-004:** Solver-Gameplay Separation -- no direct miner<->play coupling
- **INV-005:** Plan And Land Coherence -- plan/git/runtime must not drift
- **INV-006:** Robopoker Fork Coherence -- track v1.0.0 baseline, document changes

## Governance

The repository uses a doctrine hierarchy (from `OS.md`):

```
specs/ > INVARIANTS.md > OS.md > ops/ > historical docs
```

Specifications come in three types (from `SPEC.md`):
- **Decision specs:** Architectural choices
- **Migration/Port specs:** Moving capabilities between systems
- **Capability specs:** Adding/changing durable capabilities

Plans follow the ExecPlan format defined in `PLANS.md` and instantiated in
`genesis/plans/`.

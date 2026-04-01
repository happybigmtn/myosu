# Myosu Genesis Specification

Status: Active
Date: 2026-03-28
Supersedes: `archive/genesis_1774729423/SPEC.md`
Owners: Interim CEO/CTO office (turnaround)

## Purpose

Myosu is a game-solving protocol with two realities in one repository:
- A functioning local poker training/advisor product (`myosu-play` + `myosu-tui` + `myosu-games-poker`).
- An in-progress chain fork intended to score solver quality and distribute incentives.

This genesis spec defines the durable target for the next 180 days: align product, protocol, and doctrine so every claim in docs is provable in code and CI.

## One-Sentence Product Definition

Myosu is a verifiable strategy-quality pipeline that starts as a local NLHE advisor and evolves into a chain-scored solver market.

## Who This Is For

Primary: Solo protocol engineer / founder-operator who must ship a verifiable stage-0 product with limited engineering bandwidth.

Secondary: Future contributors who need an unambiguous control plane. Validators, miners, and operators once network surfaces are live.

## Architecture

```text
                    CURRENT STATE                              180-DAY TARGET STATE

  +-----------------+                            +-----------------+
  | myosu-play      |  <-- works today           | myosu-play      |  <-- polished, artifact-verified
  | myosu-tui       |                            | myosu-tui       |
  | (local advisor) |                            | (local + chain) |
  +--------+--------+                            +--------+--------+
           |                                              |
  +--------v--------+                            +--------v--------+
  | myosu-games     |  <-- works today           | myosu-games     |  <-- multi-game (+ Liar's Dice)
  | myosu-games-    |                            | myosu-games-    |
  |   poker         |                            |   poker         |
  +-----------------+                            | myosu-games-    |
                                                 |   liars-dice    |
           |                                     +--------+--------+
           |                                              |
           X  (not connected)              +------+-------+-------+------+
           |                               |      |               |      |
  +--------v--------+            +---------v--+ +-v-----------+ +-v------v----+
  | myosu-chain     |            | myosu-chain| | myosu-miner | | myosu-      |
  | runtime + node  |            | (reduced,  | | (train +    | |  validator  |
  | (216K lines,    |            |  6 pallets,| |  serve)     | | (score +    |
  |  20+ pallets,   |            |  devnet    | |             | |  submit)    |
  |  EVM/Frontier)  |            |  working)  | +-------------+ +-------------+
  +-----------------+            +------------+
```

## Module Boundaries

### Gameplay/Product (working)

| Crate | Lines | Role |
|-------|-------|------|
| `myosu-games` | 683 | Shared game types, trait exports, GameRegistry |
| `myosu-games-poker` | 3,738 | NLHE state, solver, renderer, wire, artifacts |
| `myosu-tui` | 3,975 | Shell, event loop, screens, input, pipe, theme |
| `myosu-play` | 31,337 | Train/pipe CLI, artifact auto-discovery |

### Chain (in progress)

| Crate | Lines | Role |
|-------|-------|------|
| `myosu-chain-runtime` | ~3K | Runtime composition (20+ pallets, needs reduction) |
| `myosu-chain` (node) | ~5K | Service, RPC, chain spec, consensus |
| `pallet-game-solver` | ~2.7K + modules | Core game-solving pallet (subtensor fork) |
| 12 supporting pallets | ~160K | admin-utils, commitments, crowdloan, drand, proxy, registry, shield, subtensor, swap, utility, tx-fee |

### Not Yet Created

| Crate | Role | Stage-0 Critical? |
|-------|------|-------------------|
| `myosu-chain-client` | Shared RPC client for miner/validator | Yes |
| `myosu-miner` | MCCFR training loop + strategy serving | Yes |
| `myosu-validator` | Exploitability scoring + weight submission | Yes |
| `myosu-games-liars-dice` | Architecture proof game | Yes |
| `myosu-keys` | Key management | No (stage-1) |
| `myosu-sdk` | Third-party game engine scaffold | No (stage-2) |

## Tech Stack

- Language: Rust (workspace-first), edition 2024
- Product UI: `ratatui` 0.29, `crossterm` 0.28, async `tokio`
- Solver core: robopoker fork (`happybigmtn/robopoker`, pinned rev)
- Chain base: Substrate via `opentensor/polkadot-sdk` fork
- EVM/networking: `opentensor/frontier` fork (to be reduced/removed in stage-0)
- Fixed-point math: `encointer/substrate-fixed` (pinned for Yuma bit-identity)
- Orchestration: Fabro + Raspberry (`fabro/`, `outputs/`)

## Durable Decisions

1. Gameplay and chain are separate executable concerns; shared logic in game crates.
2. Local advisor product is the immediate value wedge while chain complexity reduces.
3. Numbered `specs/031626-*` are canonical after cleanup; mirror duplicates archived.
4. `PLANS.md` ExecPlan contract unchanged; all turnaround work uses that format.
5. CI must gate chain/runtime/node before claiming stage-0 exit.
6. Prefer reversible reductions (stub/isolate) over broad rewrites.
7. Single-token model (MYOSU) for stage-0; dual-token AMM deferred.
8. Commit-reveal v2 only (hash-based); CRV3 timelock stripped.
9. Emission split: 61% miners / 21% validators / 18% owner.

## Non-Goals (180-Day Window)

- No full multi-game production network.
- No protocol tokenomics redesign beyond stage-0 proof.
- No migration away from Fabro/Raspberry orchestration.
- No web UI (TUI is the product surface).
- No mainnet launch.

## Canonical Locations

- Doctrine: `SPEC.md`, `PLANS.md`, `INVARIANTS.md`, `OS.md`
- Specifications: `specs/031626-*.md`
- Turnaround plans: `genesis/plans/*.md`
- Turnaround assessment: `genesis/ASSESSMENT.md`
- Turnaround report: `genesis/GENESIS-REPORT.md`
- Output artifacts: `outputs/`
- Fabro control plane: `fabro/`
- Archive: `archive/`, `specsarchive/`

## Relationship to genesis/PLANS.md

This spec is the durable architecture statement. Execution details, milestones, and progress tracking live in numbered ExecPlans under `genesis/plans/`.

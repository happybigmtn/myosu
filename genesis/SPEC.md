# Myosu Genesis Specification

**Status:** Draft — for 180-day turnaround planning
**Replaces:** `specs/031626-00-master-index.md` (the previous master index in the old spec corpus)

---

## Purpose / One-Line Definition

**Myosu is a decentralized game-solving subnet chain** where miners compete to produce optimal game strategies via MCCFR and validators verify quality through exploitability scoring — built on a Substrate fork with a Rust/Fabro agent orchestration layer and a terminal gameplay UI.

묘수 (myosu) — Korean for "brilliant move" or "masterstroke."

---

## Who It's For

**Primary:** Quantitative poker researchers and competitive poker players who currently pay $500-5,000/year for proprietary solvers (PioSolver, MonkerSolver) with no verification and no cross-game platform.

**Secondary:** AI/crypto researchers interested in decentralized compute markets for game-solving. Bridge, mahjong, and backgammon players who have no professional-grade solver tools.

**Operator:** A single developer (r@regenesis.dev) using Fabro agent orchestration to build and maintain the system.

---

## Architecture

```
                    FABRO / RASPBERRY (control plane)
                    runs at: /home/r/coding/myosu/
                    autonomous loop: raspberry autodev

         ┌──────────┴──────────┐
         │  fabro/programs/     │  fabro/workflows/
         │  outputs/           │  .raspberry/
         └──────────┬──────────┘
                    │
    ┌───────────────┼────────────────────────┐
    │               │                        │
    ▼               ▼                        ▼
┌─────────┐   ┌─────────┐          ┌─────────────────┐
│myosu-games│  │myosu-tui │          │  game-solver     │
│(traits)  │   │(shell)   │          │  pallet          │
│          │   │          │          │  [DISABLED]      │
│ thin-wrap│   │ five-panel│          │                  │
│ robopoker│   │ TUI shell │          │ Yuma Consensus   │
│ MCCFR    │   │          │          │ stake/emission   │
└────┬─────┘   └────┬─────┘          └────────┬────────┘
     │              │                         │
     └──────────────┴─────────────────────────┘
                    │
                    ▼
              ┌─────────────────────┐
              │  robopoker MCCFR    │
              │  (rbp-core,        │
              │   rbp-mccfr)       │
              │  private fork:     │
              │  happybigmtn/robopoker│
              └─────────────────────┘
                    │
                    ▼ (future)
              ┌──────────────────────────────────────┐
              │           Substrate Chain             │
              │  [NOT BUILT — staged for Phase 2]    │
              │                                       │
              │  ┌──────────┐    ┌───────────────┐  │
              │  │  miners   │◄───│ myosu-games   │  │
              │  │(solvers)  │    │ (MCCFR impl)  │  │
              │  └────┬─────┘    └───────────────┘  │
              │       │ submit weights                │
              │       ▼                               │
              │  ┌──────────────┐                   │
              │  │  validators   │                   │
              │  │(oracle/proof) │                   │
              │  └───────┬──────┘                   │
              │          │                          │
              │          ▼                          │
              │  ┌──────────────┐                  │
              │  │game-solver   │                  │
              │  │  pallet      │                  │
              │  │(consensus +  │                  │
              │  │ emissions)   │                  │
              │  └──────────────┘                  │
              │          │                         │
              │          ▼                         │
              │  ┌──────────────────┐              │
              │  │ human gameplay   │              │
              │  │ (myosu-tui or   │              │
              │  │  myosu-play CLI)│              │
              │  └──────────────────┘              │
              └──────────────────────────────────────┘
```

---

## Tech Stack

| Layer | Technology | Version/Source |
|-------|-----------|----------------|
| **Chain** | Substrate (polkadot-sdk) | `stable2407` branch |
| **Game Solver** | robopoker MCCFR | Private fork: `happybigmtn/robopoker` rev `0471631` |
| **Async Runtime** | Tokio | v1 |
| **CLI Parsing** | Clap | v4 |
| **Serialization** | Serde + parity-scale-codec | serde v1, scale-codec via polkadot-sdk |
| **TUI** | Ratatui + Crossterm | ratatui v0.29, crossterm v0.28 |
| **Error Handling** | thiserror + anyhow | |
| **Orchestration** | Fabro + Raspberry | `/home/r/coding/fabro/` sibling repo |
| **Language** | Rust 2024 | Primary |
| **Research** | Python + numpy | Experiment pipeline only |

---

## What Already Exists

### Active Workspace Members (build and test)
- `crates/myosu-games` — Game trait abstraction layer (thin-wrap of robopoker)
- `crates/myosu-tui` — Terminal UI shell (compiles, no tests)
- `crates/myosu-chain/pallets/game-solver` — Core pallet (well-tested, not wired into runtime)

### Commented-Out Workspace Members (Stage 1+)
- `crates/myosu-chain` — Full chain runtime (not wired)
- `crates/myosu-miner` — Miner binary (does not exist)
- `crates/myosu-validator` — Validator binary (does not exist)
- `crates/myosu-play` — Gameplay CLI (scaffold only, in worktree)

### Fabro/Raspberry Control Plane
- `fabro/programs/` — 7 program manifests (bootstrap, chain-core, services, product, platform, recurring + autodev)
- `fabro/workflows/` — Workflow graphs for implement, services, maintenance, bootstrap
- `fabro/run-configs/` — TOML run configurations
- `outputs/` — Curated lane deliverables (games:traits is fully complete; others have reviewed specs)
- `.raspberry/` — Raspberry supervisory state

### Spec Corpus
- 22 specs in `specs/` covering: game engine traits, poker engine, multi-game architecture, chain fork, miner/validator binaries, TUI, gameplay CLI, launch integration, agent experience, SDK, and more
- 5 ExecPlans in `plans/` covering Fabro migration, bootstrap, workflow library, program decomposition, and iterative execution
- Full legacy corpus preserved in `specsarchive/`

---

## Key Decisions Already Made

1. **Thin-wrap, not reimplement MCCFR.** Use `rbp-core` and `rbp-mccfr` from robopoker directly. Myosu provides the chain + incentive layer, not the CFR algorithm.

2. **Substrate fork base.** Fork subtensor's Substrate implementation for consensus, staking, and emissions. Avoid rebuilding Yuma Consensus from scratch.

3. **Fabro as primary execution substrate.** All work is dispatched via Fabro/Raspberry. The autodev loop runs autonomously.

4. **Five-panel TUI shell.** The terminal UI uses a fixed five-panel layout (header, transcript, state, declaration, input) with a `GameRenderer` trait for game-specific rendering.

5. **Multi-game via trait abstraction.** The `GameConfig` / `CfrGame` / `StrategyQuery` trait hierarchy allows any game implementing MCCFR to plug in. NLHE is first; Liar's Dice is the architecture proof.

6. **No dedicated blockchain RPC client.** The project uses jsonrpsee for on-chain RPC. No outbound HTTP/WebSocket clients needed.

7. **No containerization in Phase 0-1.** Local development uses `cargo build`. Containerization (Docker) is deferred to launch integration.

---

## What Does NOT Exist Yet

| Component | Status |
|-----------|--------|
| Runnable chain | Disabled — workspace member commented out |
| Miner binary | Not implemented |
| Validator binary | Not implemented |
| NLHE game engine | Thin-wrap scaffold in worktree only |
| SDK crate | Scaffold in worktree only |
| CI/CD pipeline | None |
| Containerization | None |
| Production deployment | None |
| Real users | None |
| Real compute market | None |

---

## Dependencies on External Systems

| System | Nature of Dependency | Risk |
|--------|---------------------|------|
| `github.com/happybigmtn/robopoker` | MCCFR solver (private fork) | **HIGH** — fork could go private or diverge |
| `github.com/paritytech/polkadot-sdk` | Substrate framework (stable2407) | MEDIUM — fork divergence risk |
| `github.com/opentensor/subtensor` | Referenced for chain design | LOW — used as reference, not imported |
| MiniMax API | Fabro execution backend | MEDIUM — auth propagation issues |
| Anthropic API | Fabro/Codex execution backend | LOW — well-integrated |

---

## Spec Relationships

This spec is the anchor for all genesis plans. It supersedes the old master index (`specs/031626-00-master-index.md`) for planning purposes, but the full spec corpus under `specs/` remains authoritative for architectural decisions. The genesis plans carry forward the 5 existing ExecPlans and fill the gaps identified in the assessment.

---

*This spec governs the 180-day turnaround. It is authoritative for the duration of the turnaround program and should be updated if the architecture changes materially.*

# Myosu Genesis Specification

Status: Active
Date: 2026-03-28
Owners: Interim CEO/CTO office (turnaround)

## Purpose

Myosu is a game-solving protocol effort with two realities in one repository:
- A functioning local poker training/advisor product surface (`myosu-play` + `myosu-tui` + `myosu-games-poker`).
- An in-progress chain fork intended to score solver quality and distribute incentives.

This genesis spec defines the durable target for the next 180 days: align product, protocol, and doctrine so each claim in docs is provable in code and CI.

## Who This Is For

Primary user:
- Solo protocol engineer / founder-operator who must ship a verifiable stage-0 product with limited engineering bandwidth.

Secondary users:
- Future contributors who need an unambiguous control plane for specs/plans.
- Validators/miners/operators once network surfaces are live.

## One-Sentence Product Definition

Myosu is a verifiable strategy-quality pipeline that starts as a local NLHE advisor product and evolves into a chain-scored solver market.

## Architecture (Current-to-Target)

```text
                          +------------------------------+
                          |        Documentation         |
                          | SPEC/PLANS + genesis corpus  |
                          +---------------+--------------+
                                          |
                                          v
+----------------+      +-----------------+-----------------+      +----------------+
|  Gameplay UX   |----->|  Shared Game Layer (Rust crates)  |<-----| Future Games   |
| myosu-play     |      | myosu-games / myosu-games-poker   |      | (Liar's Dice+) |
| myosu-tui      |      +-----------------+------------------+      +----------------+
+-------+--------+                        |
        |                                 v
        |                  +--------------+--------------+
        |                  | Chain Runtime + Pallet      |
        +----------------->| myosu-chain-runtime         |
                           | pallet-game-solver          |
                           +--------------+--------------+
                                          |
                                          v
                           +--------------+--------------+
                           | Node / RPC / Devnet         |
                           | myosu-chain (node crate)    |
                           +-----------------------------+
```

## Module Boundaries

Gameplay/Product boundary (working):
- `crates/myosu-games`: shared game types + trait exports.
- `crates/myosu-games-poker`: NLHE state, artifacts, request/wire, solver wrapper, renderer.
- `crates/myosu-tui`: shell/event/input/schema/pipe primitives.
- `crates/myosu-play`: executable train/pipe interface.

Chain boundary (in progress):
- `crates/myosu-chain/pallets/game-solver`: solver-market pallet surface.
- `crates/myosu-chain/runtime`: runtime composition.
- `crates/myosu-chain/node`: service/rpc/chainspec/cli.
- Supporting pallets and primitives remain heavily subtensor-derived.

## Tech Stack

- Language: Rust (workspace-first), edition 2024.
- Product UI: `ratatui`, `crossterm`, async `tokio`.
- Solver core: robopoker fork (`happybigmtn/robopoker`, pinned rev).
- Chain base: Substrate via `opentensor/polkadot-sdk` fork.
- EVM/networking in chain stack: `opentensor/frontier` fork.
- Fixed-point math: `encointer/substrate-fixed`.
- Orchestration/control plane: Fabro + Raspberry artifacts (`fabro/`, `.raspberry/`, `outputs/`).

## Durable Decisions Already Made

1. Keep gameplay and chain as separate executable concerns; shared logic belongs in game crates.
2. Keep the local advisor product as the immediate value wedge while chain complexity is reduced.
3. Treat numbered `specs/031626-*` as canonical after cleanup; remove mirror duplicates from active control plane.
4. Preserve `PLANS.md` ExecPlan contract unchanged; all turnaround work uses that format.
5. Expand CI from gameplay-only to include chain/runtime/node proofs before claiming stage-0 exit.
6. Prefer reversible reductions of chain scope (remove/stub/isolate) over broad rewrites.

## Non-Goals (180-Day Turnaround)

- No attempt to ship a full multi-game production network in one step.
- No protocol tokenomics redesign beyond what is needed for executable stage-0 proof.
- No migration to new orchestration systems outside Fabro/Raspberry in this window.

## Relationship to `genesis/PLANS.md`

This spec is the durable architecture and control-plane statement.
Execution details, milestones, progress tracking, and proofs live in numbered ExecPlans under `genesis/plans/`.

## Canonical Locations

- Genesis spec: `genesis/SPEC.md`
- Genesis planning contract: `genesis/PLANS.md`
- Genesis assessment: `genesis/ASSESSMENT.md`
- Genesis design system: `genesis/DESIGN.md`
- Turnaround execution plans: `genesis/plans/*.md`
- Turnaround report: `genesis/GENESIS-REPORT.md`


---
os_kind: "autonomous_company_os"
os_version: "0.1"
last_updated: "2026-03-16"
company_name: "Myosu"
company_stage: "stage_0_bootstrap"
domain_overlay: "game_solving_chain"
primary_mission_doctrine: "specs/031626-myosu-game-solving-chain.md"
hard_invariants_doctrine:
  - "INVARIANTS.md"
kpi_registry_path: "ops/kpi_registry.yaml"
scorecard_path: "ops/scorecard.md"
instrumentation_backlog_path: "ops/instrumentation_backlog.md"
risk_register_path: "ops/risk_register.md"
incident_ledger_path: "ops/incidents/"
decision_log_path: "ops/decision_log.md"
evidence_root: "ops/evidence/"
---

# Autonomous Company OS

This file is the live operating system for the myosu repo.

Myosu (묘수, "brilliant move") is a **game-solving subnet chain** — a fork of
Bittensor's subtensor where miners compete to produce optimal game strategies
and validators verify quality. The first subnets target poker (NLHE variants),
with expansion into backgammon, mahjong, bridge, and other imperfect-information
games.

## Intent

The product we are building is:
- a Substrate-based blockchain forked from subtensor
- with game-specific subnets where miners run MCCFR/CFR solvers
- validators measure strategy exploitability
- Yuma Consensus distributes emissions to the best solvers
- a gameplay layer where humans play against the trained strategies
- robopoker v1.0.0 as the core solver engine for poker subnets

Current company objective:
- scaffold the chain fork, solver incentive layer, and gameplay runtime
  as a coherent system that malinka can autonomously develop

## Rule 0

If the repo lacks trustworthy operating truth for a critical decision, the
first job is to install that truth before adding scope.

1. challenge the requirement
2. delete unnecessary work before automating it
3. simplify the process before scaling it
4. shorten the path from signal -> decision -> action

## Doctrine Hierarchy

1. Mission doctrine — `specs/`, `ralph/SPEC.md`
2. Hard invariants — `INVARIANTS.md`
3. Company operating system — `OS.md`
4. KPI registry and scorecard — `ops/kpi_registry.yaml`, `ops/scorecard.md`
5. Reference context — `ops/risk_register.md`, `ops/decision_log.md`
6. Runtime truth — `state/`

## Current Stage

Myosu is in `stage_0_bootstrap`.

Why:
- the repo is greenfield — no runtime exists yet
- the architectural plan is defined but no code has been written
- the chain fork, solver integration, and gameplay layer are all ahead
- dependency on robopoker v1.0.0 and subtensor fork are the critical paths

Stage-0 default priority order:
1. chain fork scaffold (Substrate runtime with game-solving pallets)
2. solver integration (robopoker MCCFR as miner workload)
3. validator oracle (exploitability scoring)
4. gameplay layer (human vs bot play)
5. additional game subnets beyond poker

## North Star

| field | value |
|---|---|
| metric | `solver_exploitability_convergence` |
| definition | the best miner's strategy exploitability in milli-big-blinds/hand, approaching zero as the network matures |
| formula | `min(exploitability_scores) across active miners` |
| source | validator consensus on exploitability measurement |
| cadence | per-tempo evaluation |
| why it matters | a game-solving chain is only valuable if it produces strategies that approach Nash equilibrium |

## Active Functions

| Function | Mandate | Primary outputs |
|---|---|---|
| Strategy | keep the repo aimed at chain launch and solver quality | priorities, stage call, bottleneck decisions |
| Security | guard chain consensus, solver verification, gameplay fairness | audit roadmap, risk register, no-ship escalation |
| Execution / Dev | convert priorities into landed, verified code | delivery proof, closure truth |

## Reference Downstream Lanes

- `codexpoker` — existing poker platform that will consume myosu solver output
- game communities — poker, backgammon, mahjong, bridge players

## Bootstrap Exit Criteria

Myosu remains in stage 0 until:
- Substrate chain compiles and produces blocks on local devnet
- at least one poker subnet registers and runs solver evaluation
- one miner produces a strategy profile from robopoker MCCFR
- one validator computes exploitability and submits weights
- one human can play a hand of poker against the trained bot

## Current Priority Order

1. Substrate chain fork scaffold from subtensor
2. game-solving pallet replacing Yuma incentive mechanics
3. robopoker v1.0.0 integration as miner solver engine
4. validator exploitability oracle
5. gameplay layer for human vs bot
6. additional game subnets (backgammon, mahjong, bridge)

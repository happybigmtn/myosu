# 180-Day Myosu Turnaround Master Plan

**Plan ID:** 001
**Status:** Draft — awaiting execution
**Horizon:** 180 days from execution start

This ExecPlan governs the 180-day turnaround for Myosu from bootstrapped scaffolding to a demonstrable, testable game-solving chain. It is the parent plan; all numbered plans are children.

`PLANS.md` at `genesis/PLANS.md` governs this document. This plan must be updated as child plans complete or change scope.

---

## Purpose / Big Picture

After 180 days, Myosu will have:
- A **runnable local chain** with working game-solver pallet
- A **working NLHE game engine** that can solve a poker situation and return a strategy
- A **playable TUI** that lets a human play against the best bot strategy
- **Comprehensive test coverage** on all user-facing code
- A **CI/CD pipeline** with automated quality gates
- At least **two complete Fabro lanes** through bootstrap + implementation

The product will be demonstrable to a quantitative poker researcher. It will not be production-deployed.

---

## Phase 0: Stabilization (Days 1-30)

**Goal:** Fix the broken foundation so everything else can be built on solid ground.

| Plan | Name | Milestones | Proof |
|------|------|------------|-------|
| 002 | Fabro control-plane completion | 4 | Fabro lanes executable |
| 005 | Test coverage sprint | 4 | `cargo test -p myosu-tui` passes with real tests |
| 006 | CI/CD pipeline setup | 3 | GitHub Actions runs on PR |
| 010 | Fabro quality hardening | 3 | Autodev no longer produces fake completions |

---

## Phase 1: Foundation (Days 31-90)

**Goal:** Build the core value chain — chain + game engine + TUI.

| Plan | Name | Milestones | Proof |
|------|------|------------|-------|
| 007 | Chain restart | 6 | `cargo build -p myosu-chain` succeeds; devnet produces blocks |
| 008 | NLHE game engine | 5 | `cargo test -p myosu-games-poker` passes; strategy query returns action |
| 009 | TUI full implementation | 5 | Human can play NLHE vs. best bot via TUI |
| 004 | Program decomposition completion | 3 | All 6 programs report meaningful milestone state |

---

## Phase 2: Growth (Days 91-150)

**Goal:** Expand the surface — miner, validator, SDK, multi-game.

| Plan | Name | Milestones | Proof |
|------|------|------------|-------|
| 011 | Miner binary | 4 | Miner binary builds, connects to chain, submits weights |
| 012 | Validator binary | 4 | Validator binary builds, connects to chain, submits exploitability scores |
| 013 | Game engine SDK | 4 | Third party can implement a new game via SDK in < 30 min |
| 014 | Liar's Dice proof | 3 | Liar's Dice engine works; 147,420 terminal states verified |
| 015 | Agent experience | 3 | Agent memory/journal/promotion artifacts functional |

---

## Phase 3: Polish (Days 151-180)

**Goal:** Clean up, integrate, prepare for demo.

| Plan | Name | Milestones | Proof |
|------|------|------------|-------|
| 016 | End-to-end demo | 5 | Full pipeline: miner → validator → chain → TUI gameplay |
| 017 | Documentation | 3 | README, API docs, developer guide |
| 018 | Operational setup | 3 | Devnet launch, monitoring, runbook |
| 003 | Fabro workflow library | 2 | All workflow families complete and documented |

---

## ASCII Roadmap

```
Day  1 ─────────────────────────────────────────────────────────────────── Day 180
  │                                                                             │
  ├─ Phase 0 ──────────────────┬─ Phase 1 ────────────────────┬─ Phase 2 ──────┤
  │  │                         │  │                           │  │             │
  │  P002 Fabro completion     │  P007 Chain restart         │  P011 Miner    │
  │  P005 Test sprint         │  P008 NLHE engine           │  P012 Validator│
  │  P006 CI/CD               │  P009 TUI full impl         │  P013 SDK      │
  │  P010 Fabro quality       │  P004 Program decomp        │  P014 Liar's   │
  │                           │                              │  P015 Agent    │
  │                           │                              │                │
  │                           │                              │                │
  ├─ Phase 3 ──────────────────────────────────────────────────────────────┤
  │  P016 E2E demo   P017 docs   P018 ops setup   P003 workflow library    │
  │                                                                          │
  ▼                                                                          ▼
Stabilize                        Core value chain                           Polish
```

---

## Dependency Graph

```
P002 ──┬── P010 ── P004
       │             │
P005 ──┘             │
       │             │
       ▼             ▼
P006            P007 ──┬── P008 ──┬── P009
                      │          │
                      │          ▼
                      │      P016 (E2E demo)
                      │
P011 ──┬── P012 ──┬── P016
       │          │
P013 ──┘          │
       │          │
       ▼          ▼
P014 ───────── P015
```

---

## Decision Log

- Decision: Run Phase 0 (stabilization) before Phase 1 (foundation).
  Rationale: The autodev loop generates fake completions (compile-passing but non-functional code). Building on this foundation without fixing quality first means wasted work.
  Date/Author: 2026-03-21 / Interim CEO

- Decision: Chain restart is Phase 1, not Phase 0.
  Rationale: Chain restart requires deep Substrate expertise and ~6 milestones. It should not block test sprint or CI/CD. The game engine can be developed against the robopoker thin-wrap without a live chain.
  Date/Author: 2026-03-21 / Interim CEO

- Decision: Miner and validator are Phase 2, not Phase 1.
  Rationale: The chain must be buildable and the game engine must be working before miners and validators can be meaningfully implemented. They depend on both P007 and P008.
  Date/Author: 2026-03-21 / Interim CEO

- Decision: NLHE is the only game in Phase 1.
  Rationale: The narrowest wedge. One working game beats four partial games. Liar's Dice (architecture proof) moves to Phase 2.
  Date/Author: 2026-03-21 / Interim CEO

- Decision: Demo is the only Phase 3 deliverable that matters.
  Rationale: If the full pipeline (miner → validator → chain → TUI) doesn't work end-to-end by day 180, the turnaround has failed. Everything else serves this goal.
  Date/Author: 2026-03-21 / Interim CEO

---

## What Does NOT Belong in This Plan

- Production deployment (no Kubernetes, no Terraform, no cloud infrastructure)
- Token economics beyond basic emission accounting
- Mobile or web UI (TUI only)
- Multiple consensus mechanisms beyond Yuma
- Formal security audit
- External API integrations beyond chain RPC

---

## Proof of Master Plan Completeness

The master plan is complete when:
1. `cargo test -p myosu-games-poker` passes with real NLHE tests
2. The chain builds and produces blocks on a local devnet
3. A human can play NLHE against the best bot strategy via the TUI
4. `cargo test -p myosu-tui` passes with integration tests
5. CI passes on a PR
6. Autodev produces no fake completions
7. End-to-end demo runs: strategy query → miner submission → validator scoring → chain state → TUI display

---

## Cross-Plan Coordination

| Plan | Depends On | Enables |
|------|-----------|---------|
| P002 (Fabro completion) | P006 (CI) for quality gates | P004, P010 |
| P004 (Program decomp) | P002 | P007, P011, P012, P013 |
| P005 (Test sprint) | P002 | P008, P009, P011, P012 |
| P006 (CI/CD) | P002 | All plans benefit |
| P007 (Chain restart) | P004 | P008, P011, P012, P016 |
| P008 (NLHE engine) | P005, P007 | P009, P011, P012, P014, P016 |
| P009 (TUI full) | P005, P007, P008 | P016 |
| P010 (Fabro quality) | P002 | P004 |
| P011 (Miner) | P007, P008 | P016 |
| P012 (Validator) | P007, P008 | P016 |
| P013 (SDK) | P007, P008 | P014 |
| P014 (Liar's Dice) | P008, P013 | P016 |
| P015 (Agent experience) | P002 | — |
| P016 (E2E demo) | P007, P008, P009, P011, P012 | — |
| P017 (Docs) | All prior | — |
| P018 (Ops setup) | P016 | — |
| P003 (Workflow library) | All prior | — |

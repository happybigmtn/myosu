# Genesis Assessment: Myosu 180-Day Turnaround

**Date:** 2026-03-21
**Assessor:** Interim CEO/CTO
**Scope:** Full codebase inventory, all plans, specs, git history, test coverage, dependencies

---

## The One Sentence

Myosu is a **Rust/Substrate blockchain fork** that incentivizes a decentralized market of MCCFR game-solving miners and exploitability-validating validators, wrapped in a Fabro-agent orchestration layer, with a terminal gameplay UI as its primary human-facing surface — currently in a bootstrapped-but-fragmented Stage 0 where the execution substrate is working but the product surface is mostly scaffolding.

---

## Demand Reality

### Who Uses This?

**No one.** There are zero production users. There is no deployed chain, no live miners or validators, no gameplay traffic, no revenue.

The only active consumer of this codebase is its sole author (r@regenesis.dev, 57 commits). The Fabro/Raspberry agent orchestration layer runs autonomously against the codebase, but this is agent-driven scaffolding, not human usage.

### What Specific Behavior Proves Real Demand?

Nothing. The strongest signal is the ongoing investment in execution infrastructure (Fabro, Raspberry, autodev loops), which suggests the author believes in the product thesis strongly enough to automate their own work on it. The game-solving subnet market thesis (replacing $2.5B proprietary solver market with open competition) is plausible, but there is no evidence anyone outside this repository wants it.

### Classification

**Side project / bootstrapped prototype.** The doctrine (`OS.md`) frames this as an autonomous company, but the operational reality is a single-developer codebase with agent-orchestrated execution. The ambition is large; the execution team is one person plus their AI agents.

---

## Status Quo

### What's the Pain Without This Project?

For the sole author: losing Myosu means losing the Fabro execution framework that has been built specifically for this repo, plus the accumulated plans and specs. The operational loop (`raspberry autodev`) would have to be rebuilt from scratch.

For a hypothetical future user: without Myosu, poker/baduk/bridge players pay $500-5,000 for proprietary solvers (PioSolver, MonkerSolver) with no way to verify correctness and no cross-game platform.

### Duct-Tape Alternatives

- **For game solving:** Use existing proprietary solvers (PioSolver, MonkerSolver) or open-source CFR implementations (OpenSpiel, DREAM) — all centralized, single-game, no token incentives.
- **For exploitability verification:** Trust the solver vendor or run your own CFR implementation.
- **For gameplay:** Human vs. human, or human vs. rule-based bots, or human vs. proprietary solver with no competition layer.

There is no existing open, decentralized, multi-game alternative.

---

## Desperate Specificity

### The ONE Person This Is For

**Kihong (Kevin) Park, 34, quantitative poker researcher at a prop shop** — tired of paying $2,500/year for PioSolver licenses, frustrated that he can't verify whether the solver's output is actually Nash-equilibrium-optimal, and wanting to contribute compute to a network that rewards him for finding better solutions while also letting him play against the best strategies for games PioSolver doesn't cover (Liar's Dice, backgammon, mahjong).

What keeps Kevin up at night: the proprietary solver lock-in and the inability to verify exploitability claims.
What gets Kevin promoted: publishing better preflop ranges derived from real MCCFR computation, demonstrating that open competition produces superior strategies.

---

## Narrowest Wedge

### The Smallest Thing Worth Doing This Week

**`crates/myosu-games-poker/` — a working NLHE strategy query CLI.**

A binary that takes a game state (hole cards + board), calls the robopoker MCCFR solver via `myosu-games`'s trait abstraction, and returns a recommended action. The chain, the validators, the miners, the token economics, the TUI — all of it is downstream of this one kernel.

Without a working game engine that can solve a single poker situation and return a strategy, nothing else matters. The entire product collapses to this primitive.

---

## Observation & Surprise

### Surprises During Exploration

1. **The execution substrate (Fabro/Raspberry) is more mature than the product itself.** The agent orchestration layer has been hardened through real live runs, has working autodev loops, and has discovered/hardened 10+ defects in the sibling Fabro repo. But all the programs it manages (`games:t`, `tui:shell`, `chain:runtime`, etc.) are mostly scaffolding — generated crates that compile but don't yet do anything user-facing.

2. **Chain code has excellent tests; application code has none.** The `pallet-game-solver` and `pallet-subtensor` have ~4,000+ tests in `game-solver` alone. But `myosu-games` has only 9 doc tests, and `myosu-tui` has zero tests. The quality is inverted: the part furthest from users (the chain pallet) is the most tested; the part closest to users (the TUI) is entirely untested.

3. **The full Substrate chain fork is disabled.** `crates/myosu-chain` is commented out of the workspace. The pallets exist, the tests exist, but they're not being built or run. This means the chain is simultaneously "comprehensively tested" and "not actually buildable."

4. **The autodev loop generates code that passes proof commands but doesn't work.** `myosu-play` compiles and its proof command (`cargo build -p myosu-play`) passes, but the binary exits immediately without entering the terminal loop. The synthesis produces compileable scaffolding, not working behavior.

5. **The game solver design is genuinely sophisticated.** The Yuma Consensus + exploitability scoring mechanism for validator rewards is a non-trivial economic design. The thin-wrap of robopoker's CFR traits into `myosu-games` avoids reimplementing MCCFR. The checkpoint format (magic bytes + version + bincode) is thoughtful.

6. **Single contributor, single operator.** r@regenesis.dev is the only human who has ever committed to this repo. All 57 commits are from one person (plus some Fabro automation artifacts).

---

## Future-Fit

### How Does This Compound?

The game-solving subnet thesis compounds if:
- **Compute gets cheaper** → more people can run miners profitably
- **Open solver benchmarks emerge** → Myosu's strategies can be compared to proprietary ones
- **Multi-game platform proves out** → adding new games is additive, not multiplicative
- **AI coding agents improve** → the Fabro autodev loop gets better at synthesizing real crates

### How Does It Decay?

The thesis decays if:
- **Proprietary solvers add verification features** → removes Myosu's trust-free advantage
- **OpenSpiel or similar generalizes to exploitability scoring** → removes the need for a chain
- **The sole operator burns out** → the codebase has no bus factor
- **robopoker fork diverges or goes private** → `rbp-core` and `rbp-mccfr` are pinned to a private GitHub fork

### The Bet

**This project bets that open, decentralized game-solving with exploitability verification will displace proprietary solvers, and that the 180-day window is sufficient to move from scaffolding to a demo-able product.**

---

## What Works, What's Broken, What's Half-Built

### What Works

| Component | Status | Evidence |
|-----------|--------|---------|
| Fabro/Raspberry execution substrate | **Working** | Live autodev loops, real Fabro runs, reviewed artifacts under `outputs/` |
| `myosu-games` trait abstraction | **Working** | `cargo test -p myosu-games` passes; robopoker integration functional |
| `myosu-tui` shell infrastructure | **Working (compile)** | `cargo build -p myosu-tui` passes; shell layout, events, theme, pipe mode exist |
| Chain pallet tests | **Excellent** | ~4,000+ tests in `pallet-game-solver` alone |
| Planning/SPEC infrastructure | **Excellent** | `PLANS.md`, `SPEC.md`, 5 ExecPlans, 22 specs — all well-structured |
| Spec authoring framework (`ralph/SPEC.md`) | **Exceptional** | The hidden backbone of spec quality — defines decision/migration/capability spec types rigorously |

### What's Broken

| Component | Status | Impact |
|-----------|--------|--------|
| Full chain runtime | **Not buildable** | `crates/myosu-chain` commented out; no parent workspace; runtime not wired |
| `myosu-play` binary | **Compile-skeleton** | Builds but exits immediately without terminal loop |
| `myosu-games-poker` crate | **Not built** | Spec exists in worktree, not in main; no NLHE implementation |
| `myosu-sdk` crate | **Not built** | Spec exists in worktree, not in main; no SDK implementation |
| CI/CD pipeline | **None** | No GitHub Actions, no automated test runs on PRs |
| Containerization | **None** | No Docker, no docker-compose for local dev |
| Test coverage for user-facing code | **Critical gap** | `myosu-tui` (0 tests), `myosu-games` (9 doc tests only) |

### What's Half-Built

| Component | Status | Gaps |
|-----------|--------|------|
| Game trait implementations | **Scaffolding** | `myosu-games-poker`, `myosu-games-liars-dice` exist in worktree; compile and pass `cargo test` but do no real MCCFR computation |
| Fabro bootstrap lanes | **4 of ~19 done** | `games:traits` is fully implemented; `tui:shell`, `chain:runtime`, `chain:pallet` have reviewed artifacts but no implementation |
| Product surfaces | **Scaffolding** | `play:tui`, `agent:experience` have spec/review but no real implementation |
| Platform surfaces | **Scaffolding** | `games:poker-engine`, `games:multi-game`, `sdk:core` have spec/review but no real implementation |
| Service surfaces | **Scaffolding** | `miner:service`, `validator:oracle` have spec/review but no real implementation |
| Operational RPCs | **Spec only** | `specs/031626-18-operational-rpcs.md` exists but no implementation |

---

## Tech Debt Inventory

### Critical (blocks user-visible progress)

| ID | File/Module | Description | Risk |
|----|-------------|-------------|------|
| TD-01 | `crates/myosu-chain/` | Full chain workspace commented out; runtime not wired; cannot build a runnable chain | Chain work is untestable in isolation |
| TD-02 | `crates/myosu-tui/` | Zero test coverage; any refactor risks silent breakage | High churn risk as TUI evolves |
| TD-03 | `crates/myosu-games-poker/` | Not in main workspace; only in worktree; NLHE thin-wrap not implemented | Game engine is the core value prop |
| TD-04 | `fabro/` autodev quality | Synth generates compile-passing but non-functional code; `play-tui` binary exits without loop | Wastes compute on fake completions |

### High (significant drag)

| ID | File/Module | Description | Risk |
|----|-------------|-------------|------|
| TD-05 | `crates/myosu-miner/` | Does not exist; spec exists; no miner binary | Core chain participant missing |
| TD-06 | `crates/myosu-validator/` | Does not exist; spec exists; no validator binary | Core chain participant missing |
| TD-07 | `crates/myosu-sdk/` | Worktree only; no real SDK | Third-party game registration impossible |
| TD-08 | Python research pipeline | `numpy` not declared in `requirements.txt`; research results not wired to chain | Research-to-production gap |
| TD-09 | CI/CD | No automated test runs; no lint; no type checks | Quality gate entirely manual |
| TD-10 | `fabro/checks/` | Some proof commands are `cargo test` which only runs doc tests for myosu-games | False confidence in "passing" lanes |

### Medium (manageable)

| ID | File/Module | Description | Risk |
|----|-------------|-------------|------|
| TD-11 | Duplicate spec filenames | 13 pairs of zero-padded vs non-zero-padded filenames in `specs/` | Confusion about which is canonical |
| TD-12 | `specs/031626-03-game-solving-pallet.md` | Names 11 ACs (CH-01 through CH-11) but only details 2 | Incomplete spec = incomplete implementation |
| TD-13 | `genesis/` directory | Empty — no genesis chain initialization state | Cannot bootstrap a real chain |
| TD-14 | Fabro orchestrator auth | MiniMax bridge requires interactive bash shell; non-interactive runs fail auth | Blocks unattended autodev in production |
| TD-15 | Dead-code warnings | `myosu-games-liars-dice` emits dead-code warnings during test | Violates zero-warnings bar |
| TD-16 | `ops/` KPI/evidence tracking | `ops/kpi_registry.yaml`, `ops/scorecard.md`, etc. exist but appear stale | Operational tracking not integrated into autodev loop |

### Low (cosmetic)

| ID | File/Module | Description |
|----|-------------|-------------|
| TD-17 | `.worktrees/` directory | Contains `autodev-live/` worktree with committed artifacts; `.worktrees/` in `.gitignore` |
| TD-18 | `requirements.txt` | Missing for Python research pipeline |
| TD-19 | `.env` template | Missing for environment variable documentation |

---

## Security Risks

| ID | Risk | Severity | Location |
|----|------|----------|----------|
| SEC-01 | robopoker pinned to private GitHub fork (`happybigmtn/robopoker`) | **HIGH** | `crates/myosu-games/Cargo.toml` — if fork goes private or is deleted, build breaks permanently |
| SEC-02 | No secrets management — `.env` template missing | **MEDIUM** | No `FABRIC_*` or `MINIMAX_API_KEY` template |
| SEC-03 | `arkworks` + BLS curve libraries for DRAND randomness — hazmat crypto | **MEDIUM** | `pallet-drand/`, ARKworks dependencies in `pallet-game-solver` |
| SEC-04 | `tx.origin` not used (confirmed clean) | **LOW** | N/A — Web3 rule satisfied |
| SEC-05 | No `unsafe` blocks audited in `pallet-game-solver` | **LOW** | Unaudited; `procedural-fork` introduces risk of subtle Substrate fork divergence |

---

## Test Coverage Gaps

### Untested Modules (Critical)

| Module | File | Lines | Test Coverage | Risk if Broken |
|--------|------|-------|---------------|----------------|
| `myosu-tui` entire crate | `crates/myosu-tui/src/` | 3,574 | **0 tests** | Any TUI change silently breaks user experience |
| `myosu-games` game trait impls | `crates/myosu-games/src/traits.rs` | 371 | **9 doc tests only** | Game type serialization could silently break |
| `myosu-play` CLI entrypoint | `crates/myosu-play/src/main.rs` | ~50 (scaffold) | **0 tests** | Binary could silently fail |
| `myosu-sdk` | `crates/myosu-sdk/` (worktree only) | N/A | **0 tests** | SDK API contract untested |
| `pallet-game-solver` RPC layer | `pallets/game-solver/src/rpc_info/` | ~500 | **0 tests** | Chain RPC could silently fail |

### Undertested Modules (High)

| Module | File | Test Count | Gap |
|--------|------|------------|-----|
| `myosu-games` property tests | `crates/myosu-games/src/` | 9 doc tests | No property-based tests for GameConfig, GameType serialization round-trip |
| `myosu-tui` integration | `crates/myosu-tui/src/` | 0 | No screen transition tests, no event loop tests, no shell state tests |
| Chain pallet migrations | `pallets/game-solver/src/migrations/` | 47 migrations | 47 storage migrations with minimal test coverage per migration |

---

## Existing Plan Assessment

### Plan 1: `031826-clean-up-myosu-for-fabro-primary-executor.md`
**Rating: STRONG** — Well-scoped, specific milestones, has Decision Log, concrete proof commands, references specific file paths.
**Genesis action:** Carry forward as-is into `genesis/plans/001-fabro-cleanup-completion.md`. Mark complete — this was executed on 2026-03-19.

### Plan 2: `031826-bootstrap-fabro-primary-executor-surface.md`
**Rating: STRONG** — Comprehensive bootstrap surface seeding with 14 Decision Log entries, specific file references, and thorough Surprises section.
**Genesis action:** Carry forward as-is into `genesis/plans/002-fabro-bootstrap-completion.md`. Mark partially complete — bootstrap surface is seeded, but 3 of 4 lanes still need re-verification.

### Plan 3: `031926-design-myosu-fabro-workflow-library.md`
**Rating: STRONG** — Workflow family mapping is excellent, library layout is specific, references actual Fabro source files.
**Genesis action:** Carry forward as-is into `genesis/plans/003-fabro-workflow-library.md`. The 6 workflow families (implement/verify, services, maintenance, etc.) are a good taxonomy.

### Plan 4: `031926-decompose-myosu-into-raspberry-programs.md`
**Rating: VERY STRONG** — Source map from archived specs to frontier programs is the most practically useful artifact in the plan set. The 6-program decomposition (bootstrap, chain-core, services, product, platform, recurring) is sound.
**Genesis action:** Carry forward as-is into `genesis/plans/004-raspberry-program-decomposition.md`. This plan is partially executed — all 6 program manifests exist and have been seeded.

### Plan 5: `031926-iterative-execution-and-raspberry-hardening.md`
**Rating: STRONG (but drifted)** — This plan has become an execution journal rather than a plan. The original 3 milestones are buried at line 520+. The 60+ timestamped progress entries are valuable historical record but make the plan hard to follow as actionable guidance.
**Genesis action:** REPLACE with `genesis/plans/005-iterative-execution-hardening.md`. The content is gold but needs restructuring: milestones front-and-center, execution log moved to a separate section.

### Missing Plans (Gaps in Current Portfolio)

| Missing Plan | What It Should Cover | Why It's Critical |
|-------------|---------------------|-------------------|
| Chain restart | Re-enable `crates/myosu-chain` workspace, wire runtime, get chain building | Everything downstream of the chain is blocked |
| NLHE game implementation | Implement `myosu-games-poker` thin-wrap of robopoker | The core product feature — without it, there's no game |
| TUI full implementation | Wire game rendering into `myosu-tui` shell, add screens, pipe mode end-to-end | Primary human-facing surface |
| Test coverage sprint | Add tests to `myosu-tui` and `myosu-games` | The quality inversion is a ticking time bomb |
| CI/CD setup | GitHub Actions, cargo test on PR, lint gates | No automated quality gates exist |
| Miner binary | `crates/myosu-miner` implementation | Core chain participant |
| Validator binary | `crates/myosu-validator` implementation | Core chain participant |
| SDK development | `crates/myosu-sdk` implementation | Third-party game registration |
| Fabro quality hardening | Strengthen proof commands so compile != working | Autodev currently produces fake completions |

---

## Architectural Assessment

### What the System Actually Is

```
┌─────────────────────────────────────────────────────────────────────┐
│                     FABRO / RASPBERRY (control plane)                │
│  fabro/programs/*.yaml  │  fabro/workflows/*.fabro  │  autodev loop │
└─────────────────────────────────────────────────────────────────────┘
                                    │
                    ┌───────────────┼────────────────┐
                    ▼               ▼                ▼
            ┌─────────────┐ ┌─────────────┐ ┌─────────────┐
            │ myosu-games  │ │  myosu-tui  │ │  game-solver │
            │ (traits)     │ │ (terminal)  │ │  (pallet)    │
            │              │ │             │ │  [DISABLED]  │
            └─────────────┘ └─────────────┘ └─────────────┘
                    │               │                │
                    └───────┬───────┘                │
                            ▼                        │
                     robopoker MCCFR                  │
                     (rbp-core, rbp-mccfr)           │
                     (private fork)                  │
                                                        │
                        ┌─────────────────────────────┘
                        ▼
              ┌──────────────────────┐
              │   Substrate Chain    │
              │ (NOT BUILT - staged) │
              │  miners + validators  │
              │  (NOT IMPLEMENTED)   │
              └──────────────────────┘
```

### Key Architectural Decision Points

1. **Fabro as executor, Raspberry as supervisor** — This is the right choice for a one-person project. The autodev loop generates work and monitors itself. But the quality of generated output needs hardening before it can be trusted.

2. **Thin-wrap robopoker** — The decision to wrap (not reimplement) MCCFR via `myosu-games` traits is correct. Robopoker's `rbp-core` and `rbp-mccfr` are the actual solver; Myosu provides the chain + incentive layer.

3. **Substrate fork strategy** — Forking Subtensor's Substrate implementation gives a working consensus + staking + emissions base. The risk is fork divergence from upstream (polkadot-sdk `stable2407`).

4. **Chain-as-staged** — Keeping the chain disabled while building application layers first is a reasonable staging decision. But 6+ months of disabled chain increases the risk of upstream divergence.

---

## No-Ship Conditions

Per `INVARIANTS.md` and `OS.md`, this project should not ship if:

1. **INV-003 violation:** Validator exploitability scores disagree above epsilon (1e-6) on identical inputs. Currently **not testable** — no validator implementation exists.

2. **INV-004 violation:** Any direct dependency between `myosu-play` and `myosu-miner`. Currently **satisfied by absence** — neither crate is implemented yet.

3. **False-green proof:** Any Fabro lane completes with a passing proof command that doesn't actually verify the intended behavior. Currently **known violation** — autodev produces compile-passing but non-functional code.

4. **Structured closure failure:** Any Fabro turn lands on trunk without a trusted `RESULT:` or `BLOCKED:` outcome. Currently **partially addressed** — Fabro inspect-by-run-id is now stable, but detached-submit path is still being hardened.

---

## Recommended Focus for 180 Days

**The project has 4 things in reasonable shape and ~15 things in broken or missing shape. The turnaround requires ruthless prioritization.**

### Top 4 Things That Work (Protect and Build On)
1. Fabro/Raspberry execution substrate — the most mature part of the stack
2. `myosu-games` trait abstraction — the right architectural bet
3. Chain pallet test infrastructure — solid foundation for when chain restarts
4. SPEC/PLANS infrastructure — excellent governance framework

### Top 4 Things to Fix First (Unblock Everything Else)
1. **Restart the chain** — re-enable `crates/myosu-chain`, wire runtime, get `cargo check` passing
2. **Add tests to user-facing code** — `myosu-tui` (0 tests) and `myosu-games` (9 doc tests) are the most likely to silently break
3. **Implement NLHE game engine** — `myosu-games-poker` thin-wrap is the core value proposition
4. **Fix autodev quality** — strength proof commands so compile != working

Everything else (miner binary, validator binary, SDK, multi-game, agent experience) is downstream of these four.

---

*Assessment compiled 2026-03-21. Sources: 5 ExecPlans, 22 specs, 57 git commits, full source tree, dependency graph, test coverage inventory.*

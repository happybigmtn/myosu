---
os_kind: autonomous_kernel
os_version: "3.0"
last_updated: "2026-03-17"
system: myosu
state: stage_0
domain: game_solving_chain
mission_doctrine: specs/031626-00-master-index.md
invariants: INVARIANTS.md
kpi: ops/kpi_registry.yaml
scorecard: ops/scorecard.md
risk: ops/risk_register.md
decisions: ops/decision_log.md
evidence: ops/evidence/
---

# myosu kernel

묘수. Decentralized game-solving protocol. Permissionless strategy computation
for imperfect-information games. Miners produce Nash-approximate strategies via
MCCFR. Validators measure exploitability. Yuma Consensus distributes emissions
to the strongest solvers. Agents and humans play through the same text interface.

## system architecture

```
┌──────────────────────────────────────────────────────────┐
│ CHAIN (Substrate)                                        │
│                                                          │
│  subnet_registry ──► neuron_registry ──► weight_storage  │
│       │                    │                   │         │
│       ▼                    ▼                   ▼         │
│  emission_dist  ◄─── yuma_consensus  ◄─── staking       │
│  (per tempo)         (median clip)        (voting power) │
└────────────────────┬─────────────────┬───────────────────┘
                     │                 │
          ┌──────────▼─────┐  ┌────────▼────────┐
          │ MINERS          │  │ VALIDATORS       │
          │                 │  │                  │
          │ MCCFR trainer   │  │ exploit oracle   │
          │ HTTP /strategy  │◄─┤ submit_weights   │
          └────────┬────────┘  └──────────────────┘
                   │
          ┌────────▼──────────┐
          │ GAMEPLAY           │
          │                    │
          │ TUI / HTTP / WS    │
          │ agent = human      │
          └────────────────────┘
```

| layer | function | actors |
|-------|----------|--------|
| chain | on-chain coordination: subnets, neurons, weights, emissions | pallet_game_solver |
| solvers | off-chain compute: MCCFR training, strategy serving | miners |
| validation | off-chain quality: exploitability scoring, weight submission | validators |
| gameplay | output surface: text interface, agent-native, HTTP/WS/pipe | humans, agents |

## source repo knowledge

### robopoker (happybigmtn/robopoker fork)

Audited 2026-03-17. Key findings:

| area | status | notes |
|------|--------|-------|
| CFR traits (CfrGame, CfrEdge, CfrTurn, CfrInfo, Profile, Encoder) | WORKS AS-IS | All game-agnostic. None are object-safe (require Copy+Sized). Use enum dispatch. |
| RPS reference impl | WORKS AS-IS | Full CfrGame+Profile+Encoder+Solver. 60+ convergence tests. |
| Flagship solver alias | WORKS AS-IS | `NlheSolver<PluribusRegret, LinearWeight, PluribusSampling>` at nlhe/src/lib.rs:80-84 |
| Programmatic MCCFR | WORKS AS-IS | `Solver::step()` runs one iteration. `Solver::solve()` runs full loop. |
| Exploitability | WORKS AS-IS | `(BR(P1) + BR(P2)) / 2` in profile.rs:424-429. |
| Hand evaluation | WORKS AS-IS | Bitwise evaluator (no lookup tables). Nanosecond speed. |
| Game state generation | WORKS AS-IS | `NlheInfo::random()`, `Observation::from(Street)`, `Partial::initial()` |
| Serde on NLHE types (RF-01) | NEEDS FORK CHANGE | ~12 types across 4 crates. `Metrics` field needs `#[serde(skip)]`. serde 1.0 already in workspace. |
| Non-DB encoder constructor (RF-02) | NEEDS FORK CHANGE | Internal repr IS `BTreeMap<Isomorphism, Abstraction>`. `from_map()` is a one-liner. |
| Clustering pipeline | **DB-ONLY** | `Layer::cluster()` takes `&Client`. No standalone file path. Need RF-03. |
| File-based checkpoints | **MISSING** | Only PostgreSQL persistence. Need RF-04. |
| Wire serialization | MISSING | No binary wire format. DTOs use string-encoded JSON. RF-01 is prerequisite. |
| Game parameters | HARDCODED | `N=2`, `STACK=100`, `B_BLIND=2`, `S_BLIND=1` are compile-time constants. |
| Full encoder memory | 7-11 GB RAM | 138M entries. Hardware reqs must be documented. |
| Training loop control | process-global atomic | `rbp_core::interrupted()`. Use `Solver::step()` for external orchestration. |

### subtensor (opentensor/subtensor)

Audited 2026-03-17. Key findings:

| area | status | notes |
|------|--------|-------|
| Pallet inventory | 14 keep, 12 strip, 1 replace | runtime/src/lib.rs:1591-1629 |
| **Config supertraits** | **BLOCKED** | `pallet_subtensor::Config` requires `pallet_drand::Config + pallet_crowdloan::Config`. Must strip FIRST. |
| **SwapInterface** | **BLOCKED** | Called in registration, staking, AND emission. Need no-op stub. |
| **fp_self_contained** | **BLOCKED** | Frontier's extrinsic types. Must replace with standard Substrate types. |
| **CRV3 timelock** | **BLOCKED** | Depends on `pallet_drand::Pulses`. Must strip, keep commit-reveal v2 only. |
| Yuma Consensus | NEEDS ADAPTATION | epoch/run_epoch.rs + math.rs = ~3200 lines. Port core, strip Alpha/AMM. |
| Emission/coinbase | COMPLEX | 957 lines, deeply entangled with AMM. **Recommend rewrite, not port.** |
| Weight submission | STRAIGHTFORWARD | subnets/weights.rs, 1343 lines. commit-reveal v2 is clean. |
| Axon serving | STRAIGHTFORWARD | subnets/serving.rs, 372 lines. Self-contained. |
| Neuron registration | STRAIGHTFORWARD | But calls SwapInterface for burn. Need stub. |
| Pruning | STRAIGHTFORWARD | Lowest-emission, immunity-period aware. |
| Staking | COMPLEX | Share-pool model + SwapInterface. Simplify to direct token staking. |
| Runtime APIs | NEEDS ADAPTATION | 5 API traits, ~20 methods. Strip AMM/Alpha-specific methods. |
| Node service | NEEDS ADAPTATION | 29 Frontier/EVM references in service.rs. 12 in rpc.rs. |
| Chain spec (devnet) | STRAIGHTFORWARD | devnet.rs already clean. Rebrand tokens/names. |
| Storage items | ~194 total, ~80 needed | Must inventory carefully to avoid importing unused state. |
| substrate_fixed | encointer/substrate-fixed fork | v0.6.0 with `transcendental::{exp, ln}`. Git dependency required. |
| polkadot-sdk | opentensor fork | May contain subtensor-specific patches. |
| safe-math + share-pool primitives | REQUIRED | Small local crates. Must carry into myosu. |
| freeze_struct macro | CAUTION | Generates compile-time hash checks on storage struct layouts. |

## current priority

| # | work | AC prefix | count | blocking |
|---|------|-----------|-------|----------|
| 1 | fork robopoker: serde + encoder + clustering + checkpoints | RF-01..04 | 4 | GT-02, PE-01, AP-01, MN-05 |
| 2 | fork subtensor: strip + stubs + primitives | CF-01..11 | 11 | everything |
| 3 | game engine traits: re-export + wire | GT-01..05 | 5 | PE, MN, VO, GP |
| 4 | poker engine: solver, query, exploit | PE-01..04 | 4 | MN, VO, GP |
| 5 | game-solving pallet: Yuma, subnets, staking | GS-01..10 | 10 | MN, VO |
| 6 | shared chain client | CC-01 | 1 | MN, VO, GP |
| 7 | miner: train, serve, checkpoint | MN-01..05 | 5 | VO, GP |
| 8 | validator: score, submit weights, INV-003 test | VO-01..07 | 7 | GP |
| 9 | gameplay: human vs solver | GP-01..04 | 4 | stage_0 exit |
| 10 | multi-game proof: Liar's Dice | MG-01..04 | 4 | stage_0 exit |
| 11 | TUI + NLHE game: shell, renderer, training, solver advisor | TU-01..12 | 12 | LI-03, stage_0 exit |
| 12 | abstraction pipeline | AP-01..03 | 3 | MN-02 (full training) |
| 13 | launch integration + invariant gate | LI-01..06 | 6 | stage_0 exit |
| 14 | agent experience | AX-01..06 | 6 | stage_1 |

**Total: 82 ACs across 14 stages.**

## chain fork critical path

```
CF-07 (strip drand/crowdloan supertraits)
  │
  ├──► CF-08 (replace fp_self_contained)
  │      │
  │      ▼
  │    CF-01 (strip 12 pallets from construct_runtime!)
  │      │
  │      ▼
  │    CF-02 (prune workspace dependencies)
  │
  ├──► CF-06 (SwapInterface no-op stub)
  │
  ├──► CF-09 (strip CRV3 timelock)
  │
  ├──► CF-10 (port safe-math + share-pool + runtime_common)
  │
  ├──► CF-11 (stub ProxyInterface + CommitmentsInterface + AuthorshipProvider)
  │
  ▼
CF-03 (minimal node service)
  │
  ▼
CF-04 (local devnet chain spec)
  │
  ▼
CF-05 (E2E devnet smoke test)
```

CF-07 is the first commit. Without it, nothing compiles.

## robopoker fork critical path

```
RF-01 (serde feature on NLHE types)
  │
  ├──► RF-02 (non-DB encoder constructor: from_map, from_file, from_dir)
  │
  ├──► RF-04 (file-based checkpoint save/load for NlheProfile)
  │
  ▼
RF-03 (expose clustering APIs for standalone use)
```

RF-01 enables everything else. RF-03 is less urgent (miners can use
pre-computed artifacts initially).

## key engineering decisions (updated with audit findings)

| decision | rationale | spec |
|----------|-----------|------|
| ArcSwap double-buffer for miner | zero read contention during training batches | MN-02 |
| RemoteProfile adapter for validator | `Profile::exploitability()` needs a Profile impl from query responses | GT-04 |
| checkpoint versioning: 4-byte magic + version | prevent silent corruption on format changes | PE-01 |
| encoder pinning: hash-checked artifact | INV-003 requires identical encoder across validators | VO-03 |
| commit-reveal v2 only (hash-based) | CRV3 timelock depends on pallet_drand which is stripped | GS-04, CF-09 |
| 13 pallets after CF-01 (14th added by GS-09) | SafeMode at index 20 included; index 7 reserved | CF-01, GS-09 |
| `substrate_fixed` pinned to encointer fork v0.6.0 | bit-identical Yuma output requires identical fixed-point lib | GS-05 |
| SwapInterface no-op stub (1:1 identity) | registration/staking/emission all call it; can't strip without stub | CF-06 |
| emission rewrite (not port) | coinbase assumes root network + AMM + multi-subnet; 80% unused | GS-06 |
| single-token model (not dual Alpha/TAO) | AMM pools add massive complexity for zero Stage 0 value | CF-06 |
| enum dispatch (not trait objects) | all CFR traits require Copy+Sized; no dyn dispatch possible | GT-03 |
| `Solver::step()` for training control | process-global `interrupted()` flag unsuitable for external orchestration | MN-02 |
| polkadot-sdk from opentensor fork | upstream may diverge; accept as known risk | CF-02 |
| robopoker fork (not upstream dep) | need serde, encoder constructors, clustering API exposure, file checkpoints | RF-01..04 |
| shared `myosu-chain-client` crate | prevents DRY violation across miner/validator/play | CC-01 |
| PokerSolver must support snapshot_profile() | ArcSwap publishing requires cheap profile cloning | PE-01, MN-02 |
| port codexpoker TUI patterns (not rewrite) | 33K lines of production code; training, blueprint, truth stream | TU-08..12 |
| BotBackend::action_distribution() shared API | same method serves bot decisions (sample) and solver advisor (display) | TU-09, TU-11 |
| solver advisor ON by default in training mode | learning GTO is the value prop; OFF by default in chain mode (miner privacy) | TU-11 |
| mmap blueprint files (not RAM load) | 50MB+ profiles stay on disk; < 1μs lookup via page faults | TU-10 |
| training commands (/deal, /board, /stack) | enables scenario drilling with solver advisor | TU-09 |

## spec inconsistencies to fix

| issue | fix |
|-------|-----|
| INVARIANTS.md references nonexistent `myosu-solver` crate | change to `myosu-miner` / `myosu-games-poker` |
| MN-02/MN-03 code snippets use RwLock, text says ArcSwap | fix code snippets to ArcSwap |
| CF-01 says "14 pallets" | change to "13 pallets (index 7 reserved)" |
| master index missing 031626-10-agent-experience.md | add to spec index |
| master index crate path `myosu-games/poker/` | change to `myosu-games-poker/` |
| master index out-of-scope says "no TUI" | update: TUI addressed by 031626-07 |
| GS-01 says "~25 storage items" | actual count ~31 |
| INV-006 says "git tag v1.0.0" | fork uses branch, not upstream tag |
| design.md pipe enrichment modes contradict agent spec scope | resolve: mark enriched modes as future |
| GP spec says TUI out of scope but LI-03 wires to TUI | add note: GP-02 rendering superseded by LI-03 |
| AX-* dependency on TU-06 overly broad | AX-03/04 are independent of pipe mode |

## bootstrap exit criteria

Myosu remains in stage 0 until ALL of the following are true:
- Substrate chain compiles and produces blocks on local devnet
- Game-solving pallet integrated at index 7 with Yuma Consensus
- At least one poker subnet registers and runs solver evaluation
- One miner produces a strategy profile from robopoker MCCFR
- One validator computes exploitability and submits weights
- Two validators produce identical scores for same miner (INV-003)
- Yuma Consensus distributes emissions proportional to quality
- One human can play a hand of poker against the trained bot
- Training mode works offline with blueprint bot and solver advisor
- Solver advisor shows action distribution during hero decisions
- Liar's Dice validates multi-game architecture (zero existing code changes)
- No dependency path between myosu-play and myosu-miner (INV-004)
- Emission accounting: sum(distributions) == block_emission * epochs (no-ship gate)
- All 6 invariants pass (INV-001 through INV-006) via consolidated gate test

## doctrine hierarchy

| priority | source | controls |
|----------|--------|----------|
| 1 | `specs/`, `ralph/SPEC.md` | what system must become |
| 2 | `INVARIANTS.md` | what must never be violated |
| 3 | `OS.md` | how the system decides |
| 4 | `ops/kpi_registry.yaml`, `ops/scorecard.md` | what is green/yellow/red |
| 5 | `ops/risk_register.md`, `ops/decision_log.md` | context for decisions |
| 6 | `state/` | whether the kernel is behaving |

## malinka platform capabilities

Malinka is the autonomous development kernel executing this plan. Binary at
`~/.local/bin/malinka`, source at `~/coding/malinka`.

### task dispatch

- Parses `ralph/IMPLEMENT.md` into a dependency DAG
- `Depends on:` field → tasks only dispatch when all deps are `[x]`
- `Conflicts with:` field → mutually exclusive tasks never run simultaneously
- `Affinity:` field → workspace reuse for related tasks
- `Priority:` field → explicit priority override (lower = higher priority)
- `Tags:` field → workflow routing and filtering
- Repair-pending tasks get priority over new dispatch
- Section order is tiebreaker when priorities are equal

### IMPLEMENT.md fields (all optional except Where + Tests)

```
- [ ] **TASK-ID** — Title
  - Where: `path/to/files`
  - Depends on: `DEP-01`, `DEP-02`
  - Conflicts with: `OTHER-01`
  - Tests: `cargo check -p crate-name`
  - Tags: `chain`, `critical`
  - Priority: `0`
  - Affinity: `chain-scaffold`
  - Blocking: why this matters
  - Verify: acceptance criteria
  - Integration: `Trigger=X; Callsite=Y; State=Z; Persistence=W; Signal=V`
  - Rollback: what could go wrong
  - Spec path: `specs/031626-01.md`
  - Blocked by: `lineage-ref`
  - Quality gate: `approved`
```

### proof system

- `Tests:` field in IMPLEMENT.md = per-task proof commands
- Proof gate runs AFTER worker produces `RESULT:` line
- Pass → trunk integrator lands the work
- `ZeroTestsMatched` → `proof_contract_repair` (re-dispatch)
- `CommandFailed` → `rejected` (task marked failed)
- For greenfield crates: use `cargo check -p <crate>` not specific test names
- `Requires services:` field can start devnet/DB before proof runs

### trunk integrator

- On proof pass: commits workspace diff to trunk automatically
- Commit message: `AC-{TASK_ID}: {title}`
- Worker does NOT create commits — malinka handles it
- Workspace changes are uncommitted diffs that malinka applies

### engine: kimi adapter

```yaml
engine:
  adapter: kimi
  model: kimi-code/kimi-for-coding
  reasoning: high
  command: kimi --yolo --thinking
  timeout_secs: 3600
```

Ensure kimi is authenticated before running: `kimi login` (one-time setup).

Previous claude configuration hit rate limits; all tasks now use kimi.

### recurring lanes

| lane | role | cadence | what it does |
|------|------|---------|-------------|
| strategy | steering | 24h | reads doctrine, produces generated_work items for the plan |
| security | auditing | 24h | scans audit_scopes, produces findings + remediation_packets |
| operations | monitoring | 15m | checks runtime health, diagnoses failures |
| learning | improving | 6h | identifies patterns, proposes process improvements |

Each lane writes `state/tasks/<lane>/source-output.json`.
Strategy can generate new plan entries. Security can create remediation tasks.
Operations has `pre_authorized_action_classes: [inspect, status_check, log_read]`.

### workspace model

- Each task gets a git worktree clone at `~/coding/myosu-workspaces/<task-id>/`
- Worker operates in isolation — no interference with other workers
- On success: trunk integrator applies diff to trunk
- On failure: workspace preserved for debugging (configurable)
- `workspace.max_drift_commits: 20` — max divergence from trunk before rebase

### company OS doctrine

```
OS.md frontmatter:
  domain_overlay: platform | world_simulation | decentralized_casino | other
  company_stage: stage_0_bootstrap | stage_1_pmf | stage_2_early_scale | ...
```

Controls deployment freezes, no-ship gates, error budgets, allowed exceptions.

### health monitoring

| Command | Shows |
|---------|-------|
| `malinka health` | Workers, recurring status, execution metrics |
| `malinka queue` | Admitted queue, selected tasks, priorities |
| `malinka list` | All tasks grouped by section |
| `malinka status` | Counts + next ready task |
| `malinka monitor` | Current task runtime state |

**Key state files:**
| File | Purpose |
|------|---------|
| `state/health.json` | Machine-readable health snapshot |
| `state/session.json` | Failed tasks, retry counts, provider cooldowns |
| `state/queue.json` | Admitted queue state |
| `state/workspaces/*.json` | Per-task workspace status |

**Troubleshooting:**

| Issue | Cause | Fix |
|-------|-------|-----|
| `git exited with code -1` | Git credential prompt without stdin | `export GIT_TERMINAL_PROMPT=0` |
| `patch does not apply` | Stale workspace patches | `rm state/workspaces/*.patch` and restart |
| Task stuck retrying | High retry count blocking dispatch | Clear from `state/session.json` failed_tasks/retrying_tasks |
| No tasks selected | All workers busy or tasks blocked | Check `malinka queue` for blocked status |

### session state + retry

- `state/session.json` — running tasks, failed tasks, retry counts
- Exponential backoff on failure: `max_retry_backoff_ms: 300000`
- `proof_contract_repair` tasks get re-dispatched with priority
- Hot-reload: config changes in `project.yaml` picked up within one poll cycle (30s)

**Clear a stuck task:**
```python
import json
with open('state/session.json', 'r') as f: d = json.load(f)
d['failed_tasks'] = [t for t in d['failed_tasks'] if t['task_id'] != 'TASK-ID']
d['retrying_tasks'] = [t for t in d['retrying_tasks'] if t['task_id'] != 'TASK-ID']
with open('state/session.json', 'w') as f: json.dump(d, f)
```

### semantic adapters (recurring source validation)

Bounded vocabulary for recurring lane outputs:
- `response_class`: page | ticket | log | freeze | other (PR1 fix)
- `evidence_freshness`: fresh | stale | missing
- `severity`: s0 | s1 | s2 | s3
- `lane`: platform | world_simulation | decentralized_casino | other (PR1 fix)
- `arbitration_status`: approved_with_conditions | challenged | blocked

### commands

```bash
# Start supervisor
export GIT_TERMINAL_PROMPT=0   # Required: prevents git credential hangs
malinka run --native           # Foreground mode (good for debugging)
malinka run --native --task X  # Run single task

# Monitoring
malinka status                 # Task counts, next ready, completion stats
malinka queue                  # Admitted queue, selected tasks, priorities
malinka health                 # Live workers, recurring status, metrics
malinka monitor                # Current task runtime state
malinka list                   # All tasks grouped by section

# Maintenance
malinka soak --turns N         # Bounded N-turn run
malinka daemon                 # Background daemon mode
malinka set-status TASK done   # Manually mark task complete
```

### key config (project.yaml)

```yaml
agent:
  max_concurrent_agents: 8     # total worker slots
polling:
  interval_ms: 30000           # config hot-reload interval
health:
  heartbeat_stale_after: 90s
  result_stale_after: 30m
workspace:
  root: ~/coding/myosu-workspaces
  max_drift_commits: 20
```

## manual prerequisites (before malinka can execute)

| # | work | who | est. |
|---|------|-----|------|
| 1 | Fork robopoker v1.0.0 to happybigmtn/robopoker | human | 1h |
| 2 | RF-01: add serde feature (~12 types, 4 crates) | human/malinka | 1-2d |
| 3 | RF-02: add from_map/from_file/from_dir to NlheEncoder | human/malinka | 1d |
| 4 | RF-03: expose clustering APIs from rbp-clustering | human/malinka | 1d |
| 5 | RF-04: file-based checkpoint save/load | human/malinka | 1d |
| 6 | Copy subtensor into myosu workspace | human | 1h |
| 7 | CF-07: strip drand/crowdloan supertraits (first commit) | human | 2h |
| 8 | CF-06: SwapInterface no-op stub | human/malinka | 1d |
| 9 | CF-08..11: remaining stubs and primitive ports | malinka | 2-3d |

After these, malinka executes CF-01..05 → GT-01..05 → PE-01..04 → GS-01..10
→ CC-01 → MN-01..05 → VO-01..07 → GP-01..04 → MG-01..04 → TU-01..07
→ AP-01..03 → LI-01..06 → AX-01..06 autonomously.

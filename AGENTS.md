---
os_kind: autonomous_kernel
os_version: "3.0"
last_updated: "2026-04-02"
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
| 1 | `SPEC.md`, `specs/`, `plans/`, `fabro/programs/` | what system must become and how current lanes are supervised |
| 2 | `INVARIANTS.md` | what must never be violated |
| 3 | `OS.md` | how the system decides |
| 4 | `outputs/`, `ops/` | durable lane artifacts plus risk and operating context |
| 5 | `specsarchive/`, `ralph/IMPLEMENT.md` | historical context only; not the active control plane |
| 6 | `.raspberry/`, Fabro run state | runtime truth and local supervisory state |

## Fabro/Raspberry Execution Model

Fabro is the execution substrate. Raspberry is the control plane.

Execution plane:
- checked-in workflow graphs live under `fabro/workflows/`
- checked-in run configs live under `fabro/run-configs/`
- checked-in prompts live under `fabro/prompts/`
- proof and readiness helpers live under `fabro/checks/`

Control plane:
- program manifests live under `fabro/programs/`
- the current bootstrap entrypoint is `fabro/programs/myosu-bootstrap.yaml`
- curated lane deliverables live under `outputs/`
- lane readiness, blockage, proof posture, and operational state should be
  derived from Raspberry manifests plus Fabro run truth

Historical-only surfaces:
- `ralph/IMPLEMENT.md`
- `specsarchive/`

Deleted Malinka-only surfaces:
- `project.yaml`
- `WORKFLOW.md`

Do not recreate deleted Malinka control files. New execution work should land
as Fabro assets plus Raspberry program updates.

## Current Operator Loop

Primary local commands:

```bash
fabro run fabro/run-configs/bootstrap/game-traits.toml
fabro run fabro/run-configs/bootstrap/tui-shell.toml
fabro run fabro/run-configs/bootstrap/chain-runtime-restart.toml
fabro run fabro/run-configs/bootstrap/chain-pallet-restart.toml

raspberry plan --manifest fabro/programs/myosu-bootstrap.yaml
raspberry status --manifest fabro/programs/myosu-bootstrap.yaml
raspberry execute --manifest fabro/programs/myosu-bootstrap.yaml
```

Preferred runtime truth sources:
- Fabro inspect surfaces and stable run metadata
- Raspberry program state under `.raspberry/`

Avoid building control-plane logic around raw Fabro run-directory layout unless
there is no stable inspection surface available yet.

Runtime wasm cache for node smoke proofs:
- `SKIP_WASM_BUILD=1 cargo test -p myosu-chain --test stage0_local_loop --quiet` still requires a cached runtime wasm at `target/debug/wbuild/myosu-chain-runtime/myosu_chain_runtime.wasm`
- On Rust 1.94 in this repo, install `wasm32v1-none` once via `rustup target add wasm32v1-none`, then refresh the cache with `cargo build -p myosu-chain-runtime` before trusting node smoke results after runtime edits
- `tests/e2e/helpers/start_devnet.sh` uses the same cached-runtime requirement, launches the single-authority local chain through `--dev`, and steers the node’s `--tmp` data under `target/e2e/devnet/tmp` via `TMPDIR` so `stop_devnet.sh` can clean the helper-owned temp tree
- Fresh local devnet start-up is not instant: JSON-RPC on `127.0.0.1:9955` can take well over a minute on cold boots while genesis + service initialization complete, so `tests/e2e/helpers/wait_for_block.sh` should keep a generous startup timeout; `tests/e2e/emission_flow.sh` now uses `MYOSU_E2E_WAIT_TIMEOUT=120` for its initial block wait
- `tests/e2e/two_node_sync.sh` proves named-`devnet` peer discovery by starting the bootnode with `MYOSU_NODE_AUTHORITY_SURI=//myosu//devnet//authority-1`; the node service now honors that env var to seed Aura/GRANDPA keys for authority nodes on non-`Local` chains
- `tests/e2e/local_loop.sh` overlaps the `myosu-miner` / `myosu-validator` / `myosu-play` binary build with that RPC warm-up window; the poker zero-iteration bootstrap can still print an upstream robopoker panic about a missing abstraction lookup to stderr while exiting 0, so treat the script’s stdout contract plus produced checkpoint/response files as the truthful proof surface for that step
- Any ad hoc `cargo run -p myosu-chain-client --example ...` proof that sets a custom `CARGO_TARGET_DIR` should also set `SKIP_WASM_BUILD=1`, otherwise the runtime build script will try to rebuild the wasm in that fresh target tree instead of reusing the cached artifact from `cargo build -p myosu-chain-runtime`
- `SKIP_WASM_BUILD=1 cargo run -p myosu-chain -- build-spec --chain devnet --raw` can still cold-build `frame-storage-access-test-runtime` from the inherited polkadot-sdk toolchain before it emits the JSON; let that compile finish once instead of treating the quiet build phase as a hang
- `bash ops/deploy-bootnode.sh --dry-run` now prepares `target/bootnode/devnet/` with a stable node key, launcher script, systemd unit, and metadata file, then prints the truthful bootnode multiaddr without starting the process
- `SKIP_WASM_BUILD=1 cargo test -p myosu-miner -p myosu-validator --quiet` can cold-build `wasm-opt-sys` and stay quiet for several minutes; do not treat the silence as a hang while `cargo` still has active compiler children
- `cargo test -p pallet-game-solver coinbase --quiet` exercises the default-build stage-0 coinbase assertions from `src/tests/stage_0_flow.rs`; the inherited `src/tests/coinbase.rs` suite still sits behind `legacy-subtensor-tests`
- Local security-audit proof currently needs the CI ignore set: run `cargo audit` with ignores for `RUSTSEC-2025-0009`, `RUSTSEC-2025-0055`, `RUSTSEC-2023-0091`, `RUSTSEC-2024-0438`, `RUSTSEC-2025-0118`, `RUSTSEC-2026-0020`, and `RUSTSEC-2026-0021` until the inherited chain stack is rebased
- Root-level `PY-*` proof commands currently assume the default `python` interpreter already has `numpy`; until `PY-003` adds a managed Python environment, install it once with `python -m pip install numpy` before running the Python quality-gate checks

## Bootstrap Lanes

The current bootstrap program intentionally stays narrow:

- `games:traits` — trusted leaf crate, continue
- `tui:shell` — trusted leaf crate, continue
- `chain:runtime` — restart lane
- `chain:pallet` — restart lane, blocked on runtime review

Do not widen the bootstrap manifest until:
- doctrine cutover is complete
- the trusted lanes have produced curated `spec.md` and `review.md` artifacts
- the Fabro-to-Raspberry run-truth bridge is more stable

## Current Expectations

When adding or changing active supervisory work:

- add or update a workflow graph in `fabro/workflows/`
- add or update a run config in `fabro/run-configs/`
- add or update prompt files in `fabro/prompts/` if the workflow needs them
- add or update the Raspberry program manifest in `fabro/programs/`
- add or update curated artifact roots in `outputs/`

When evaluating whether something is "done":

- code changes alone are not enough
- the lane should produce curated artifacts under `outputs/`
- proof should be executable from Fabro/Raspberry entrypoints
- stale references to Malinka-era infrastructure should be removed rather than
  normalized

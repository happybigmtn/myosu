---
os_kind: "autonomous_company_os"
os_version: "0.4"
last_updated: "2026-04-08"
company_name: "Myosu"
company_stage: "stage_0_bootstrap"
domain_overlay: "platform"
primary_mission_doctrine: "specs/031626-00-master-index.md"
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

This file is the live operating system for the myosu repo. It is not the full
product pitch, not a historical archive, and not a substitute for plan files.
Its job is narrower: define the mission, state the current doctrine hierarchy,
name the currently truthful control plane, and make the stage-0 operator loop
easy to follow.

## Mission

Myosu is a decentralized game-solving protocol for imperfect-information games.
Miners produce Nash-approximate strategies with MCCFR. Validators measure
quality through deterministic scoring. The chain coordinates subnets, neurons,
weights, and emissions. Humans and agents consume the resulting strategy
through the same gameplay surface.

The stage-0 requirement is not "have a compelling architecture." It is "prove
the loop works." Myosu remains in stage 0 until a stripped chain, a miner, a
validator, and gameplay connect as one honest local proof.

## System Shape

```text
chain        -> on-chain coordination: subnets, neurons, weights, emissions
miners       -> off-chain compute: training and strategy serving
validators   -> off-chain scoring: quality measurement and weight submission
gameplay     -> human/agent consumption: TUI, pipe, and related surfaces
```

Current first-class game truths:
- Poker is the full stage-0 vertical slice.
- Liar's Dice is the second-game proof that the same shared seams can host a
  distinct subnet and local gameplay surface.

## Current Truth

These statements are currently proven in-repo:

- `myosu-chain` builds, authors blocks on a local devnet, and exposes the
  stripped game-solver RPC surface.
- `myosu-miner` can register, publish an axon, train bounded local artifacts,
  and answer strategy queries.
- `myosu-validator` can register, stake, acquire validator permit, score saved
  responses deterministically, and submit weights.
- `myosu-play` can consume artifact-backed strategy, run a human-facing poker
  hand, expose agent-facing pipe output, and prove local gameplay smoke.
- The node-owned local proof now covers two live subnets on one chain:
  poker and Liar's Dice both reach miner/validator/gameplay proof on the same
  local node-owned loop.

What is still not a first-class operator product:

- production deployment
- public multi-node network operations
- polished miner/validator operations beyond the proven local loop
- broad web or hosted gameplay surfaces
- stage-1-safe key handling and named-network packaging

## Stage-0 Exit Criteria

Myosu remains in stage 0 until all of the following are true:

- the chain compiles and produces blocks on a local devnet
- `pallet-game-solver` is integrated at runtime index `7`
- at least one poker subnet registers and runs solver evaluation
- at least one miner produces strategy from the robopoker MCCFR engine
- at least one validator computes deterministic quality and submits weights
- two validators can score the same miner identically enough to satisfy
  `INV-003`
- Yuma-style economics distribute emission according to measured quality
- a human can play a poker hand against trained strategy
- the local training surface works with blueprint bot plus solver advisor
- the shared game seam admits a second game without poker rewrites
- gameplay and miner remain cleanly separated (`INV-004`)
- emission accounting and the invariant gate stay green

The canonical detailed invariant surface remains [INVARIANTS.md](INVARIANTS.md).

## Doctrine Hierarchy

Use this order when sources disagree:

1. `SPEC.md`, `specs/`, `genesis/plans/`
   These define what the system must become and how current supervised work is
   shaped.
2. `INVARIANTS.md`
   These define what must not be violated even when plans change.
3. `OS.md`
   This file explains how the repo currently decides and what the current
   operator loop actually is.
4. `ops/` and `outputs/`
   These hold durable operating context, evidence, and curated lane artifacts.
5. `specsarchive/` and `ralph/IMPLEMENT.md`
   Historical reference only.
6. local generated state such as `target/` and helper temp dirs
   Runtime truth for the current machine, not the top-level control doctrine.

## Current Control Plane

Historical planning material still mentions a bootstrap supervision layer that
is not part of the checked-in repo. The current control plane is:

Today the truthful control plane is simpler:
- specs, plans, and invariants define intent
- executable cargo and shell proofs define runnable truth
- `outputs/` and `ops/` hold durable evidence and operating context

Historical-only surfaces:
- `specsarchive/`
- `ralph/IMPLEMENT.md`
- deleted Malinka control files such as `project.yaml` and `WORKFLOW.md`

## Current Operator Loop

The truthful current operator surfaces are:

1. Doctrine integrity and repo-shape checks for document truth.
2. The node-owned stage-0 local loop for end-to-end chain truth.
3. The local gameplay/advisor surface for human and agent consumption.

Doctrine and operator proof:

```bash
bash .github/scripts/check_doctrine_integrity.sh
```

Node-owned stage-0 proof:

```bash
SKIP_WASM_BUILD=1 cargo test -p myosu-chain --test stage0_local_loop --quiet
env SKIP_WASM_BUILD=1 target/debug/myosu-chain --stage0-local-loop-smoke
```

Gameplay/advisor proof:

```bash
SKIP_WASM_BUILD=1 cargo run -p myosu-play --quiet -- --smoke-test
printf 'quit\n' | SKIP_WASM_BUILD=1 cargo run -p myosu-play --quiet -- pipe
```

The preferred proof is the node-owned chain smoke or its cargo-managed test
wrapper. Manual miner/validator bring-up still exists as a diagnostic surface,
but it is not the primary first-class operator story anymore.

## Current Priorities

Completed local execution stack:
- `002` canonical spec/doctrine freshness sync
- `003` runtime reduction
- `004` minimal devnet node
- `005` stage-0 pallet reduction
- `006` game boundary hardening
- `007` miner/validator/bootstrap bring-up
- `008` artifact/wire/checkpoint hardening
- `009` poker play/TUI productization
- `011` security, observability, and release governance
- `012` additive multi-game proof
- `013` integration harness
- `014` through `019` doctrine/governance cleanup and Genesis governance
- `020` second-game subnet execution proof

Hosted proof already closed:
- `010` GitHub Actions proof and timing gate

Promoted next-step lane:
- `021` operator hardening and network packaging

## Evidence and Proof Surfaces

Prefer executable proof over prose claims.

Current high-signal proof commands:

```bash
cargo test -p pallet-game-solver stage_0_flow --quiet
SKIP_WASM_BUILD=1 cargo test -p myosu-chain --test stage0_local_loop --quiet
SKIP_WASM_BUILD=1 cargo run -p myosu-play --quiet -- --smoke-test
cargo test -p myosu-games-liars-dice --quiet
SKIP_WASM_BUILD=1 cargo test -p myosu-miner -p myosu-validator --quiet
```

If a doc claims a surface is first-class, there should be a command close to
this file or a nearby playbook that proves it.

## Documentation Boundaries

`OS.md` should describe:
- mission and stage-0 meaning
- doctrine hierarchy
- current control plane
- current operator loop
- what is current versus planned

`OS.md` should not become:
- the full market thesis
- the complete product roadmap
- a dump of every historical technical discovery
- a replacement for plan files or invariant docs

## References

- [SPEC.md](SPEC.md)
- [INVARIANTS.md](INVARIANTS.md)
- [plans/001-master-plan.md](plans/001-master-plan.md)
- [fabro/programs/myosu-bootstrap.yaml](fabro/programs/myosu-bootstrap.yaml)
- [docs/execution-playbooks/bootstrap.md](docs/execution-playbooks/bootstrap.md)
- [docs/execution-playbooks/stage0-local-loop.md](docs/execution-playbooks/stage0-local-loop.md)
- [docs/execution-playbooks/local-advisor.md](docs/execution-playbooks/local-advisor.md)

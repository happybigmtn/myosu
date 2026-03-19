# Decompose Myosu into Raspberry Programs

This ExecPlan is a living document. The sections `Progress`,
`Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must
be kept up to date as work proceeds.

`PLANS.md` is checked into the repository root and this document must be
maintained in accordance with it. This plan depends on
`specs/031826-fabro-primary-executor-decision.md`,
`specs/031826-myosu-fabro-primary-executor-migration.md`,
`plans/031826-bootstrap-fabro-primary-executor-surface.md`, and
`plans/031926-design-myosu-fabro-workflow-library.md`.

## Purpose / Big Picture

After this slice lands, Myosu will have a clear Raspberry program-of-programs
shape instead of one giant manifest or one giant historical task list. A
contributor will be able to see which Raspberry program owns bootstrap work,
which owns chain-core restart work, which owns services, which owns product
surfaces, and which owns recurring oversight. The result should make the full
scope of `specsarchive/` and legacy `ralph/IMPLEMENT.md` supervisable through
multiple coherent control-plane frontiers.

## Progress

- [x] (2026-03-19 04:30Z) Concluded from the workflow-library review that the
  full Myosu scope is too broad for one workflow or one Raspberry manifest.
- [x] (2026-03-19 04:30Z) Identified the candidate top-level program frontiers:
  bootstrap, chain-core, services, product, platform, and recurring oversight.
- [x] (2026-03-19 04:30Z) Inventoried `specsarchive/` and
  `ralph/archive/IMPLEMENT-malinka-legacy.md` into a source map of candidate
  units, lanes, and dependency edges.
- [x] (2026-03-19 04:30Z) Defined the first stable Raspberry program set and
  assigned each near-term lane to an initial owning program.
- [x] (2026-03-19 04:30Z) Defined the output-root and milestone strategy across
  programs so shared units do not produce conflicting artifact contracts.
- [x] (2026-03-19 04:30Z) Seeded the next program manifest after
  `myosu-bootstrap.yaml`.
- [x] (2026-03-19 05:19Z) Seeded `myosu-services.yaml` as the next frontier
  manifest after `myosu-chain-core.yaml`, with practical cross-program
  readiness expressed through shared-output checks.
- [x] (2026-03-19 05:19Z) Seeded `myosu-product.yaml` as the next frontier
  manifest after services, keeping the initial product frontier narrow to
  `play:tui` and `agent:experience`.
- [x] (2026-03-19 05:19Z) Seeded `myosu-platform.yaml` as the reusable engine
  and SDK frontier, keeping the first platform surface narrow to
  `games:poker-engine`, `games:multi-game`, and `sdk:core`.
- [x] (2026-03-19 05:19Z) Seeded `myosu-recurring.yaml` as the recurring
  oversight frontier, using contract-first recurring lanes for strategy,
  security, operations, and learning.
- [x] (2026-03-19 15:40Z) Added a top-level `myosu.yaml` portfolio manifest
  that supervises the seeded frontier manifests as child Raspberry programs.

## Surprises & Discoveries

- Observation: the historical Myosu scope is wider than a normal product repo's
  execution plane.
  Evidence: `specsarchive/` covers chain/runtime, solvers, validation,
  gameplay, TUI, abstractions, launch integration, agent experience, SDK work,
  incentive design, operational RPCs, and key management.

- Observation: the old `IMPLEMENT` surface encoded dependency clues that are
  still useful even though it is no longer the active control plane.
  Evidence: the archived task ledger groups work into stages with blocking
  relationships that still help identify which units naturally cluster into the
  same Raspberry program frontier.

- Observation: the current bootstrap manifest is the right seed, but it is not
  the right total shape.
  Evidence: `fabro/programs/myosu-bootstrap.yaml` correctly supervises the first
  four lanes, but it intentionally avoids services, launch orchestration, and
  recurring oversight because those would overload the first control-plane
  frontier.

- Observation: the archived specs and legacy task ledger converge on the same
  supervisory frontiers even though they were written in different styles.
  Evidence: the specs cluster naturally into chain core, services, product/TUI,
  platform/SDK/expansion, and recurring/operational oversight, while the legacy
  task ledger stages group into matching blocks such as chain fork, miners and
  validators, gameplay/TUI, abstraction/platform work, launch, and agent
  integration.

## Decision Log

- Decision: use multiple Raspberry program manifests to cover the full Myosu
  surface rather than extending `myosu-bootstrap.yaml` until it becomes huge.
  Rationale: program boundaries should stay legible to operators, and different
  frontiers have different lane kinds, proof shapes, and execution cadence.
  Date/Author: 2026-03-19 / Codex

- Decision: decompose by supervisory frontier, not by the old stage numbering
  in `IMPLEMENT`.
  Rationale: the old stage order is useful historical sequencing data, but
  Raspberry programs should group work by coherent control-plane ownership:
  restart work together, services together, product surfaces together, and
  recurring oversight together.
  Date/Author: 2026-03-19 / Codex

- Decision: keep one shared `outputs/` tree across programs rather than a
  separate artifact root per program.
  Rationale: output ownership should remain lane-centric (`outputs/games/traits`,
  `outputs/chain/runtime`, etc.). Multiple programs can supervise different
  frontiers without forcing a second artifact taxonomy.
  Date/Author: 2026-03-19 / Codex

- Decision: seed `myosu-chain-core.yaml` as the first non-bootstrap frontier
  manifest.
  Rationale: chain restart work is the clearest next coherent frontier, and it
  already has two existing restart lanes with checked-in run configs and output
  roots.
  Date/Author: 2026-03-19 / Codex

## Outcomes & Retrospective

This plan now has concrete outputs:

- a source-map from archived material to supervisory frontiers
- an explicit first stable set of Raspberry programs
- cross-program output and milestone rules in `fabro/programs/README.md`
- `fabro/programs/myosu-chain-core.yaml` as the first non-bootstrap frontier

The next step after this slice is no longer frontier seeding. It is choosing
which seeded frontier should be executed next and how the first real
cross-program dependencies should be exercised.

That higher-level control plane now exists too:

- `fabro/programs/myosu.yaml`

The new top-level manifest does not replace the frontier manifests. It wraps
them so Raspberry can supervise the full repo as a program-of-programs while
preserving each frontier's own lane-level execution semantics.

## Context and Orientation

Myosu now has:

- a Fabro execution surface under `fabro/`
- a first Raspberry program manifest at `fabro/programs/myosu-bootstrap.yaml`
- curated artifact roots under `outputs/`
- historical product and task context under `specsarchive/` and `ralph/archive/`

What it does not yet have is a full control-plane decomposition for the whole
repo. The archived material is too wide for one manifest because it spans
multiple supervisory shapes:

- trusted bounded leaf work
- phased restart work
- service bringup and health tracking
- interface and product-surface work
- launch orchestration
- recurring oversight and upstream maintenance

This plan exists to define how that breadth should become multiple Raspberry
program manifests.

### Candidate program frontiers

These are the current candidate programs, not yet final:

1. `myosu-bootstrap`
   Owns the current narrow bootstrap foothold:
   - `games:traits`
   - `tui:shell`
   - `chain:runtime`
   - `chain:pallet`

2. `myosu-chain-core`
   Owns chain restart and core chain surfaces:
   - `chain:runtime`
   - `chain:pallet`
   - `chain:node`
   - `chain:spec`
   - `chain:client`
   - `launch:devnet` once chain bootstrap matures

3. `myosu-services`
   Owns long-lived off-chain service lanes:
   - `miner:service`
   - `validator:oracle`
   - `validator:determinism`
   - `abstractions:artifacts`

4. `myosu-product`
   Owns user- and agent-facing surfaces:
   - `play:tui`
   - `play:api`
   - `spectator:relay`
   - `agent:experience`

5. `myosu-platform`
   Owns reusable platform and expansion work:
   - `games:poker-engine`
   - `games:multi-game`
   - `sdk:core`
   - `platform:upstream-sync`

6. `myosu-recurring`
   Owns recurring oversight and backlog-processing work:
   - `security:*`
   - `operations:*`
   - `strategy:*`
   - `learning:*`

The final decomposition may collapse or split some of these, but this is the
starting frontier map.

## Source Map from Archived Material

This is the current reduction from archived specs and the legacy task ledger to
supervisory frontiers.

### Bootstrap foothold

Sources:
- `specsarchive/031626-02a-game-engine-traits.md`
- `specsarchive/031626-07-tui-implementation.md`
- `specsarchive/031626-01-chain-fork-scaffold.md`
- legacy tasks `GT-*`, `TU-01..07`, `CF-*`

Current lanes:
- `games:traits`
- `tui:shell`
- `chain:runtime`
- `chain:pallet`

### Chain core

Sources:
- `specsarchive/031626-01-chain-fork-scaffold.md`
- `specsarchive/031626-03-game-solving-pallet.md`
- `specsarchive/031626-18-operational-rpcs.md`
- legacy tasks `CF-*`, `GS-*`, `CC-01`

Candidate lanes:
- `chain:runtime`
- `chain:pallet`
- `chain:node`
- `chain:spec`
- `chain:client`
- `launch:devnet`

### Services

Sources:
- `specsarchive/031626-04a-miner-binary.md`
- `specsarchive/031626-04b-validator-oracle.md`
- `specsarchive/031626-08-abstraction-pipeline.md`
- legacy tasks `MN-*`, `VO-*`, `AP-*`

Candidate lanes:
- `miner:service`
- `validator:oracle`
- `validator:determinism`
- `abstractions:artifacts`

### Product surfaces

Sources:
- `specsarchive/031626-05-gameplay-cli.md`
- `specsarchive/031626-07-tui-implementation.md`
- `specsarchive/031626-10-agent-experience.md`
- `specsarchive/031626-17-spectator-protocol.md`
- legacy tasks `GP-*`, `TU-08..12`, `AX-*`

Candidate lanes:
- `play:tui`
- `play:api`
- `spectator:relay`
- `agent:experience`

### Platform and expansion

Sources:
- `specsarchive/031626-02b-poker-engine.md`
- `specsarchive/031626-06-multi-game-architecture.md`
- `specsarchive/031626-14-poker-variant-family.md`
- `specsarchive/031626-19-game-engine-sdk.md`
- `specsarchive/031626-16-cross-game-scoring.md`
- legacy tasks `PE-*`, `MG-*`, future SDK/platform work

Candidate lanes:
- `games:poker-engine`
- `games:multi-game`
- `games:variants`
- `sdk:core`
- `platform:metrics`
- `platform:upstream-sync`

### Recurring and oversight

Sources:
- `specsarchive/031626-09-launch-integration.md`
- `specsarchive/031626-18-operational-rpcs.md`
- `specsarchive/031626-99-malinka-enhancements.md`
- `ops/`
- legacy tasks `LI-*`, `MNT-*`

Candidate lanes:
- `launch:devnet`
- `operations:scorecard`
- `security:audit`
- `strategy:planning`
- `learning:improvement`

## Stable Program Frontier Set

This is the first stable decomposition to use going forward.

### 1. `myosu-bootstrap`

Owns:
- `games:traits`
- `tui:shell`
- `chain:runtime`
- `chain:pallet`

Why:
- narrow trusted foothold
- establishes first curated artifacts
- proves Fabro/Raspberry split

### 2. `myosu-chain-core`

Owns:
- `chain:runtime`
- `chain:pallet`
- future `chain:node`
- future `chain:spec`
- future `chain:client`
- future `launch:devnet`

Why:
- coherent restart frontier
- chain-specific proof posture
- should eventually own the chain restart path independently of bootstrap

### 3. `myosu-services`

Owns:
- `miner:service`
- `validator:oracle`
- `validator:determinism`
- `abstractions:artifacts`

Why:
- service health and readiness dominate
- different execution cadence from chain restart or product work

### 4. `myosu-product`

Owns:
- `play:tui`
- `play:api`
- `spectator:relay`
- `agent:experience`

Why:
- user- and agent-facing surfaces
- shared interface and interaction concerns

Seeded now:
- `play:tui`
- `agent:experience`

### 5. `myosu-platform`

Owns:
- `games:poker-engine`
- `games:multi-game`
- `games:variants`
- `sdk:core`
- `platform:upstream-sync`

Why:
- reusable engine and expansion work
- not purely service or launch oriented

Seeded now:
- `games:poker-engine`
- `games:multi-game`
- `sdk:core`

### 6. `myosu-recurring`

Owns:
- `security:*`
- `operations:*`
- `strategy:*`
- `learning:*`

Why:
- recurring oversight and backlog processing are fundamentally different from
  bounded delivery lanes

Seeded now:
- `strategy:planning`
- `security:audit`
- `operations:scorecard`
- `learning:improvement`

## Cross-Program Output and Milestone Rules

These rules now define the shared contract across programs.

### Shared `outputs/`

- All programs share one `outputs/` tree.
- Output ownership is lane-centric, not program-centric.
- A lane's output root should stay stable across programs.

Examples:
- `outputs/games/traits/`
- `outputs/tui/shell/`
- `outputs/chain/runtime/`
- `outputs/chain/pallet/`

### Milestone naming

- use shared milestone names when the meaning is portable across programs:
  `specified`, `reviewed`, `implemented`, `verified`
- use lane-specific milestone names inside multi-lane units when ambiguity would
  otherwise appear:
  `runtime_reviewed`, `pallet_reviewed`
- use service/orchestration-specific milestone names only when the lane really
  differs in semantics:
  `service_ready`, `launch_ready`

### Cross-program dependencies

- Dependencies should be expressed with the same unit/lane/milestone vocabulary
  across programs.
- Do not duplicate artifacts into a second output root just to satisfy a second
  manifest.
- It is acceptable for an early bootstrap manifest and a later dedicated
  frontier manifest to supervise the same lane temporarily, as long as the lane
  keeps one output root and one milestone contract.

## Milestones

### Milestone 1: Source-map the archived scope

At the end of this milestone, the archived specs and historical task ledger are
reduced to one source map of candidate units, lanes, proofs, and dependencies.
The proof is a checked-in inventory section in this plan.

### Milestone 2: Freeze the first program frontier set

At the end of this milestone, the first stable set of Raspberry programs is
defined, with every near-term lane assigned to an owning program. The proof is
an explicit frontier table in this plan.

### Milestone 3: Define artifact and milestone boundaries across programs

At the end of this milestone, the plan states where outputs live, how
milestones are named, and how cross-program dependencies are expressed without
duplicating artifact contracts. The proof is a checked-in contract section in
this plan.

### Milestone 4: Seed the next manifests

At the end of this milestone, at least one new Raspberry program manifest
exists beyond `myosu-bootstrap.yaml`, or the plan names the exact manifest(s)
and order to create next. The proof is a checked-in manifest or a concrete
manifest creation sequence.

## Plan of Work

Start by mining `specsarchive/` and the archived `IMPLEMENT` ledger for
candidate units, lanes, and dependencies. Do not carry over the old files'
formatting or stage numbering mechanically; extract the supervisory meaning.

Then define the stable frontiers. The question is not "what comes next in the
old task list?" The question is "which work should one Raspberry program be
able to supervise coherently without becoming a grab bag?"

Once the frontier set is stable, define how those programs share `outputs/`,
how they express cross-program dependencies, and which program should be seeded
next after bootstrap.

## Concrete Steps

Work from the repository root.

1. Inventory the archived source material.

       rg -n '^#|^##|^### ' specsarchive/*.md
       sed -n '1,260p' ralph/archive/IMPLEMENT-malinka-legacy.md

2. Build a source map of candidate units, lanes, proofs, and dependencies.

3. Freeze the first stable Raspberry program frontier set.

4. Define cross-program output, milestone, and dependency rules.

5. Seed the next manifest(s) after `fabro/programs/myosu-bootstrap.yaml`.

## Validation and Acceptance

Acceptance is complete when:

- this plan contains a source map from archived materials to candidate
  unit/lane frontiers
- the first stable set of Raspberry programs is explicit
- the plan defines how `outputs/` and milestones work across programs
- at least one next manifest beyond bootstrap is checked in or named with
  enough detail that a stateless contributor could create it next

## Idempotence and Recovery

This planning slice is additive. It should not delete the bootstrap manifest or
reassign its existing lanes unless the replacement frontier is checked in and
documented in the same slice.

## Artifacts and Notes

Primary source material for this slice:

    specsarchive/
    ralph/archive/IMPLEMENT-malinka-legacy.md
    fabro/programs/myosu-bootstrap.yaml
    outputs/

## Interfaces and Dependencies

The outputs of this slice should influence:

- `fabro/programs/`
- `outputs/`
- `fabro/workflows/`
- `fabro/run-configs/`
- future Raspberry multi-program supervision and dispatch policy

Revision Note: Initial draft created to separate program decomposition from
workflow-family design. Updated after execution to add the archived source map,
freeze the first stable program frontier set, define shared output/milestone
rules, and seed `fabro/programs/myosu-chain-core.yaml` plus
`fabro/programs/README.md`. Updated again after seeding
`fabro/programs/myosu-services.yaml`, `fabro/programs/myosu-product.yaml`,
`fabro/programs/myosu-platform.yaml`, and `fabro/programs/myosu-recurring.yaml`.

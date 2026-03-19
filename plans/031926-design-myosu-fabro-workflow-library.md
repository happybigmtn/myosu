# Design the Myosu Fabro Workflow Library

This ExecPlan is a living document. The sections `Progress`,
`Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must
be kept up to date as work proceeds.

`PLANS.md` is checked into the repository root and this document must be
maintained in accordance with it. This plan depends on
`specs/031826-fabro-primary-executor-decision.md`,
`specs/031826-myosu-fabro-primary-executor-migration.md`, and
`plans/031826-bootstrap-fabro-primary-executor-surface.md`.

## Purpose / Big Picture

After this slice lands, Myosu will not just have a single bootstrap Fabro
surface. It will have a small, intentional workflow library: the set of
workflow patterns that Raspberry can dispatch for trusted leaf crates, chain
restart work, service bringup, launch orchestration, spec conformance, and
recurring oversight. A contributor will be able to look at `fabro/workflows/`
and see which workflow family applies to which kind of Myosu lane.

## Progress

- [x] (2026-03-19 04:30Z) Reviewed Fabro's built-in implement workflow at
  `coding/fabro/fabro/workflows/implement/workflow.fabro`.
- [x] (2026-03-19 04:30Z) Reviewed the Fabro example workflows under
  `coding/fabro/docs/examples/`.
- [x] (2026-03-19 04:30Z) Identified the first workflow pattern families
  relevant to Myosu: implement/verify, phased build, definition-of-done
  conformance, semantic-port backlog loops, and large ensemble planning.
- [x] (2026-03-19 04:30Z) Mapped each current and near-term Myosu lane type to
  a recommended Fabro workflow family.
- [x] (2026-03-19 04:30Z) Decided which workflow families should be reusable
  library shapes and which graphs should remain Myosu-specific wrappers.
- [x] (2026-03-19 04:30Z) Seeded the first real workflow-library layout under
  `fabro/workflows/` beyond the bootstrap stubs.
- [x] (2026-03-19 05:19Z) Seeded the first `services/` family workflow and
  paired run-configs for `miner:service` and `validator:oracle`.
- [x] (2026-03-19 05:19Z) Seeded the first `maintenance/` family workflows for
  recurring strategy, security, operations, and learning lanes.

## Surprises & Discoveries

- Observation: Fabro's examples are already close to the kinds of supervision
  Myosu will need; the gap is mostly selection and adaptation, not invention.
  Evidence: the examples cover one-shot implementation, phased builds,
  acceptance/spec conformance, recurring maintenance loops, and large
  verification-heavy application builds.

- Observation: Myosu likely needs multiple workflow families running at once,
  not one "canonical" workflow reused everywhere.
  Evidence: trusted leaf crates, chain restarts, miner/validator services,
  launch/devnet orchestration, and recurring security/operations review have
  different risk profiles and proof shapes.

- Observation: a first workflow promotion had already started before this plan
  was fully executed.
  Evidence: `fabro/workflows/implement/game-traits.fabro` and its paired run
  config/prompts already existed and provide a stronger `implement` family for
  the `games:traits` lane than the original bootstrap stub alone.

- Observation: the best first restart-family promotion is `chain:runtime`, not
  `chain:pallet`.
  Evidence: `chain:pallet` is explicitly blocked on `chain@runtime_reviewed` in
  the Raspberry bootstrap manifest, so the runtime lane is the first restart
  lane that should gain a richer workflow family.

- Observation: the full `specsarchive/` and legacy `IMPLEMENT` surface is too
  broad to model as one workflow or even as one Raspberry program manifest.
  Evidence: the archived material spans trusted leaf crates, chain restart
  work, services, interfaces, orchestration, SDK/platform work, and recurring
  oversight. Those are multiple lane families with different proof and control
  semantics.

## Decision Log

- Decision: use Fabro's existing example workflow families as the vocabulary
  for Myosu workflow design.
  Rationale: this keeps Myosu close to proven Fabro patterns and reduces the
  risk of inventing a bespoke orchestration language too early.
  Date/Author: 2026-03-19 / Codex

- Decision: treat workflow-library design as a dedicated planning problem.
  Rationale: choosing workflow families for each lane type affects long-lived
  execution architecture, review burden, and operator trust.
  Date/Author: 2026-03-19 / Codex

- Decision: use a reusable family directory shape (`bootstrap/`, `implement/`,
  `restart/`, and later `conformance/`, `services/`, `orchestration/`,
  `maintenance/`) instead of one flat workflow directory.
  Rationale: this keeps the workflow library legible as it grows and makes it
  obvious which graphs are generic pattern families versus one-off bootstrap
  stubs.
  Date/Author: 2026-03-19 / Codex

- Decision: promote `chain:runtime` into the `restart/` family first.
  Rationale: runtime restart work is the clearest case where a phased,
  verification-gated workflow family is strictly more appropriate than the
  minimal bootstrap stub.
  Date/Author: 2026-03-19 / Codex

- Decision: cover the full Myosu surface with multiple Raspberry programs that
  each dispatch family-appropriate workflows, rather than with one giant
  workflow or one giant manifest.
  Rationale: Raspberry should supervise a program-of-programs: bootstrap,
  chain-core, services, product surfaces, launch/orchestration, and recurring
  oversight. That decomposition is the only way to keep the control plane
  legible while still covering the full archived scope.
  Date/Author: 2026-03-19 / Codex

## Outcomes & Retrospective

This plan now has a concrete first outcome:

- the workflow-family map is explicit
- `fabro/workflows/README.md` defines the library shape
- `games:traits` is recognized as an `implement/` family lane
- `chain:runtime` has been promoted into a `restart/` family workflow

The next likely promotion is `chain:pallet`, followed by the first
service-oriented workflow family for `miner:service` or `validator:oracle`.

That next service-oriented workflow family now exists as explicit lane wrappers:

- `fabro/workflows/services/miner-service.fabro`
- `fabro/workflows/services/validator-oracle.fabro`

with paired Myosu-specific run configs for miner and validator bootstrap lanes.

The first maintenance-family workflows now also exist as explicit lane wrappers:

- `fabro/workflows/maintenance/strategy-planning.fabro`
- `fabro/workflows/maintenance/security-audit.fabro`
- `fabro/workflows/maintenance/operations-scorecard.fabro`
- `fabro/workflows/maintenance/learning-improvement.fabro`

The next larger planning question after workflow-family design is program
decomposition: how the full archived Myosu scope should be split into multiple
Raspberry program manifests and dependency frontiers. That is a separate
planning problem from workflow-family selection. That follow-on work now lives
in `plans/031926-decompose-myosu-into-raspberry-programs.md`.

## Context and Orientation

Fabro is the execution substrate for Myosu. Checked-in workflows live under
`fabro/workflows/`. Raspberry supervises units and lanes through
`fabro/programs/myosu-bootstrap.yaml`.

The current bootstrap surface uses very small lane-specific workflows only to
establish the execution/control-plane split. That is enough for cutover, but it
is not yet a workflow library.

Fabro's own examples already show several workflow families:

- **Implement & simplify loop**
  Source: `coding/fabro/fabro/workflows/implement/workflow.fabro`
  Best for: bounded code changes with preflight, simplify passes, and final
  verification.

- **Phased build with verification gates**
  Source: `coding/fabro/docs/examples/solitaire.mdx`
  Best for: greenfield or restart work that must build layer by layer.

- **Definition-of-done conformance**
  Source: `coding/fabro/docs/examples/definition-of-done.mdx`
  Best for: auditing implementation against explicit specs or checklists.

- **Semantic port loop**
  Source: `coding/fabro/docs/examples/semantic-port.mdx`
  Best for: backlog-style recurring maintenance or upstream/downstream
  synchronization.

- **Large ensemble planning plus verification chain**
  Source: `coding/fabro/docs/examples/clone-substack.mdx`
  Best for: large, high-risk feature builds where one-shot planning is likely
  too weak.

- **NL spec conformance loop**
  Source: `coding/fabro/docs/examples/nlspec-conformance.mdx`
  Best for: specification-to-implementation work where conformance, not just
  "shipping code", is the primary bar.

Myosu's lanes are broader than a normal app repo's:

- trusted leaf crates (`games:traits`, `tui:shell`)
- chain restart work (`chain:runtime`, `chain:pallet`)
- future service lanes (`miner:service`, `validator:oracle`)
- future interface lanes (`play:tui`, spectators, APIs)
- future orchestration lanes (`launch:devnet`)
- future recurring lanes (strategy, security, operations, learning)

This plan exists to decide which workflow family fits each of those lane types.

## Workflow Family Map

| Lane type | Recommended family | Why |
|---|---|---|
| Trusted leaf crate continuation (`games:traits`, `tui:shell`) | `implement/` | The code base is trusted enough to iterate with bounded implement/fix/verify loops. |
| Chain restart (`chain:runtime`, `chain:pallet`) | `restart/` | These lanes need phased rebuilds and explicit restart boundaries before implementation can be trusted. |
| Service bringup (`miner:service`, `validator:oracle`) | `services/` | Service lanes need health checks, readiness gates, and bounded repair loops, not just compile/test loops. |
| Interface bringup (`play:tui`, future HTTP/WS surfaces) | `implement/` or `services/` depending on runtime surface | Pure code/UI work fits `implement/`; persistent API bringup may need service-style health gates. |
| Launch/devnet orchestration | `orchestration/` | Environment sequencing and cross-lane readiness dominate this work. |
| Spec conformance or readiness audit | `conformance/` | Best fit when the main question is “does code satisfy the checklist/spec?” |
| Upstream sync (`robopoker`, `subtensor`) | `maintenance/` | Semantic-port style recurring backlog loops fit this work naturally. |
| Recurring strategy/security/operations review | `maintenance/` or `conformance/` | Depends on whether the lane processes a queue or audits against a standing checklist. |

## Proposed Library Layout

Execution workflows should live under:

    fabro/workflows/
      README.md
      bootstrap/
      implement/
      restart/
      conformance/
      services/
      orchestration/
      maintenance/

Rules:

- `bootstrap/` is for narrow lane-entry workflows
- `implement/` is for trusted bounded code changes
- `restart/` is for phased rebuild/reduction work
- `conformance/` is for spec or checklist closure
- `services/` is for service bringup and stabilization
- `orchestration/` is for multi-lane environment control
- `maintenance/` is for recurring backlog-style flows

Myosu-specific prompts and run configs can stay in corresponding
`fabro/prompts/<family>/` and `fabro/run-configs/<family>/` trees while
reusing family-level graph structure.

## Milestones

### Milestone 1: Workflow family map

At the end of this milestone, every important Myosu lane type is mapped to a
preferred Fabro workflow family with rationale. The proof is a checked-in
decision matrix in this plan.

### Milestone 2: Library layout

At the end of this milestone, the repository has a proposed directory layout
for reusable workflow templates versus Myosu-specific workflows. The proof is a
concrete filesystem plan under `fabro/workflows/`.

### Milestone 3: First non-bootstrap workflow promotions

At the end of this milestone, at least one current bootstrap workflow is
replaced or promoted into a more appropriate workflow family based on the map.
The proof is a checked-in workflow change with validation.

## Plan of Work

First, classify the existing Fabro workflow families by what they optimize for:
speed, trust, decomposition, recurrence, or conformance. Then map Myosu lane
types to those families. Avoid designing bespoke workflows unless none of the
existing patterns fit cleanly.

Next, separate reusable workflow templates from Myosu-specific wrappers. For
example, a generic `phased-restart` workflow could be reused by both
`chain:runtime` and `chain:pallet`, while a Myosu-specific run config provides
the exact prompts, proof scripts, and artifact paths.

Finally, use the map to decide which bootstrap workflows should remain narrow
and which should be replaced by richer workflow families.

Current result:

- `games:traits` is already promoted into `implement/`
- `chain:runtime` is now promoted into `restart/`
- `tui:shell` remains bootstrap-width for now
- `chain:pallet` remains bootstrap-width until the runtime restart lane
  produces reviewed artifacts

## Concrete Steps

Work from the repository root.

1. Re-read Fabro's built-in and example workflows.

       sed -n '1,260p' /home/r/coding/fabro/fabro/workflows/implement/workflow.fabro
       find /home/r/coding/fabro/docs/examples -maxdepth 2 -type f | sort
       for f in /home/r/coding/fabro/docs/examples/*.mdx; do sed -n '1,260p' "$f"; done

2. Build a lane-type to workflow-family matrix.

3. Propose the `fabro/workflows/` library layout for Myosu.

4. Promote or replace the first bootstrap workflow where the chosen family is
   clearly better than the current stub.

## Validation and Acceptance

Acceptance is complete when:

- the plan contains a clear workflow-family map for Myosu lane types
- the repo has a proposed workflow-library layout under `fabro/workflows/`
- at least one bootstrap workflow has actually been promoted into a richer
  workflow family

## Idempotence and Recovery

This planning slice is safe and additive. It should not delete bootstrap
workflows unless a replacement is checked in and validated in the same slice.

## Artifacts and Notes

Primary sources for this planning slice:

    coding/fabro/fabro/workflows/implement/workflow.fabro
    coding/fabro/docs/examples/clone-substack.mdx
    coding/fabro/docs/examples/definition-of-done.mdx
    coding/fabro/docs/examples/nlspec-conformance.mdx
    coding/fabro/docs/examples/semantic-port.mdx
    coding/fabro/docs/examples/solitaire.mdx

## Interfaces and Dependencies

The outputs of this slice should influence:

- `fabro/workflows/`
- `fabro/run-configs/`
- `fabro/programs/`
- future Raspberry lane dispatch policy

Revision Note: Initial draft created to separate workflow-library design from
bootstrap cleanup. Updated after execution to add the explicit lane-type map,
seed `fabro/workflows/README.md`, and promote `chain:runtime` into the
`restart/` family while recognizing the existing `games:traits` promotion into
`implement/`.

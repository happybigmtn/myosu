# Bootstrap the Remaining Fabro Primary-Executor Surface

This ExecPlan is a living document. The sections `Progress`,
`Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must
be kept up to date as work proceeds.

`PLANS.md` is checked into the repository root and this document must be
maintained in accordance with it. This plan depends on
`specs/031826-fabro-primary-executor-decision.md`,
`specs/031826-myosu-fabro-primary-executor-migration.md`, and
`plans/031826-clean-up-myosu-for-fabro-primary-executor.md`.

## Purpose / Big Picture

After this slice lands, the new Fabro-aligned doctrine surface will stop being
just a set of files on disk and become the repo's real default entrypoint.
`OS.md` and `AGENTS.md` will point at the new canonical `specs/` and `plans/`
surface first, stale references to deleted Malinka files will be removed, and
Myosu will have a first Fabro-native operational skeleton for supervising real
workstreams.

## Progress

- [x] (2026-03-19 03:17Z) Drafted this follow-on ExecPlan during the cleanup
  slice so the new doctrine surface has an immediate next step.
- [x] (2026-03-19 03:34Z) Audited `ralph/IMPLEMENT.md`, the active workspace,
  and the current codebase to decide which executed work is trustworthy enough
  to carry into the Fabro-first reset.
- [x] (2026-03-19 03:34Z) Verified that `myosu-games` and `myosu-tui` are real
  leaf crates with passing tests, while the workspace still fails at the
  chain/pallet layer.
- [x] (2026-03-19 03:34Z) Confirmed that `WORKFLOW.md`, `project.yaml`, and
  `ops/` should be treated as Malinka-era remnants that may be replaced or
  deleted rather than automatically preserved.
- [x] (2026-03-19 03:34Z) Reviewed Fabro's workflow, run-config, context,
  checkpoint, output, and architecture docs plus the Raspberry supervisor
  fixtures and code to derive the actual execution-plane and control-plane
  deliverable model Myosu should target.
- [x] (2026-03-19 03:34Z) Seeded the first checked-in Fabro execution-plane
  assets under `fabro/`, including bootstrap workflows, run configs, shared
  prompts, and proof helper scripts.
- [x] (2026-03-19 03:34Z) Seeded the first Raspberry control-plane manifest at
  `fabro/programs/myosu-bootstrap.yaml` with units, lanes, artifacts,
  milestones, proof profiles, and checks.
- [x] (2026-03-19 03:34Z) Created curated `outputs/` roots for the bootstrap
  lanes without falsely marking any lane complete.
- [x] (2026-03-19 03:34Z) Verified TOML parsing for all new run configs and ran
  the bootstrap proof scripts: trusted-lane checks pass, restart-lane presence
  checks pass.
- [x] (2026-03-19 03:34Z) Reviewed Fabro's latest
  `031826-port-and-generalize-fabro-dispatch-for-myosu.md` plan and ranked the
  highest-value next Raspberry-building steps for Myosu's migration.
- [x] (2026-03-19 04:03Z) Deleted `project.yaml` and `WORKFLOW.md` entirely
  because they were confirmed to be Malinka-only remnants rather than bridge
  surfaces worth preserving.
- [x] (2026-03-19 04:09Z) Inventoried the remaining active-doctrine references
  to `specsarchive/`, `ralph/`, deleted Malinka files, and stale Malinka-first
  wording.
- [x] (2026-03-19 04:09Z) Rewrote `OS.md` and `AGENTS.md` so Fabro/Raspberry is
  the active execution/control story and deleted Malinka files are explicitly
  treated as gone.
- [x] (2026-03-19 04:09Z) Rewrote the active Fabro migration specs and index so
  they no longer treat `project.yaml` and `WORKFLOW.md` as live bridge files.
- [x] (2026-03-19 04:09Z) Defined the first Fabro-native operational skeleton
  for Myosu bootstrap workstreams.
- [x] (2026-03-19 04:10Z) Archived the detailed Malinka-era task ledger and
  capability review, then rewrote `ralph/IMPLEMENT.md` and
  `ops/malinka-capabilities.md` into explicit historical stubs.
- [x] (2026-03-19 04:19Z) Rewrote `README.md` so a newcomer can find the
  active doctrine, bootstrap plan, Raspberry manifest, and Fabro operator loop
  without reading `specsarchive/`.
- [x] (2026-03-19 04:30Z) Reviewed Fabro's built-in implement workflow and the
  example workflow corpus, extracted the workflow pattern families relevant to
  Myosu, and decided that Myosu workflow-library design deserves its own
  dedicated plan.
- [x] (2026-03-19 14:05Z) Seeded the first non-bootstrap implementation lane
  for `games:traits`, including a real implement/fixup/verify workflow, a
  dedicated run config, a proof script, and a Raspberry manifest for the lane.
- [x] (2026-03-19 05:19Z) Ran the real `games:traits` bootstrap lane through
  Fabro, producing `outputs/games/traits/spec.md` and
  `outputs/games/traits/review.md`.
- [x] (2026-03-19 05:19Z) Ran the first real `games:traits` implementation
  lane through Fabro, producing `outputs/games/traits/implementation.md` and
  `outputs/games/traits/verification.md`, with Slice 1 completed.
- [x] (2026-03-19 05:34Z) Seeded the first non-bootstrap frontier manifests:
  `myosu-chain-core.yaml`, `myosu-services.yaml`, `myosu-product.yaml`,
  `myosu-platform.yaml`, and `myosu-recurring.yaml`.
- [x] (2026-03-19 05:34Z) Seeded the matching workflow-family surfaces needed
  for those frontiers: `restart/`, `services/`, and `maintenance/`, plus the
  next bootstrap contract lanes for product and platform work.
- [x] (2026-03-19 05:34Z) Completed the two follow-on planning tracks:
  workflow-library design and full Raspberry program decomposition.
- [x] (2026-03-19 05:34Z) Rehomed the Raspberry observer TUI capability spec
  and implementation plan into the companion Fabro repository, so Myosu no
  longer carries Fabro-owned implementation docs locally.

## Surprises & Discoveries

- Observation: several doctrine and operational files still depend on the old
  `specs/031626-00-master-index.md` path and on `ralph/IMPLEMENT.md`.
  Evidence: `rg -n "specs/031626|ralph/IMPLEMENT.md|malinka"` across the repo
  during the cleanup slice showed references in `OS.md`, `AGENTS.md`,
  `WORKFLOW.md`, and `project.yaml`.

- Observation: deleting `project.yaml` and `WORKFLOW.md` makes doctrine
  cleanup more urgent, not less.
  Evidence: live plans and specs still contain references to those files, so
  the remaining cutover work is now clearly about removing stale doctrine and
  replacing it with the Fabro/Raspberry entrypoints rather than "migrating"
  the deleted files.

- Observation: `ralph/IMPLEMENT.md` and `ops/malinka-capabilities.md` were
  still visually authoritative even after the active control plane changed.
  Evidence: both files contained dense operational detail in active-looking
  formats, so leaving them untouched would continue to mislead contributors
  into treating them as live control surfaces.

- Observation: the current workspace is split between trustworthy leaf crates
  and an untrustworthy chain fork.
  Evidence: `cargo test -p myosu-games` passes, `cargo test -p myosu-tui`
  passes, but `cargo check -p pallet-game-solver` and `cargo test --workspace`
  fail with thousands of unresolved imports and missing dependencies in the
  chain layer.

- Observation: the chain/runtime path is not crate-shaped yet.
  Evidence: `crates/myosu-chain/node/`, `crates/myosu-chain/runtime/`, and
  `crates/myosu-chain/common/` currently have source trees but no `Cargo.toml`
  manifests, and the workspace root only includes `myosu-games`, `myosu-tui`,
  and `pallet-game-solver`.

- Observation: some copied primitives may be salvageable, but they are not
  wired into the current workspace.
  Evidence: `safe-math` and `share-pool` source files contain coherent logic,
  but `cargo check --manifest-path .../safe-math/Cargo.toml` and
  `cargo check --manifest-path .../share-pool/Cargo.toml` both fail because the
  root workspace is missing inherited dependency entries.

- Observation: the TUI and schema work is more trustworthy as reusable code
  than as proven end-to-end feature completion.
  Evidence: the crates pass their local tests, but some proof claims in
  `IMPLEMENT.md` exceed what the tests actually show; for example, the only
  terminal-loop tests in `crates/myosu-tui/src/events.rs` are ignored because
  they require a real TTY.

- Observation: Fabro's stable author-time contract is not "a task list", but a
  trio of checked-in workflow assets: Graphviz workflow graphs, TOML run
  configs, and prompt/script files.
  Evidence: the docs in `docs/core-concepts/how-fabro-works.mdx`,
  `docs/core-concepts/workflows.mdx`, `docs/reference/dot-language.mdx`, and
  `docs/execution/run-configuration.mdx` consistently describe `.fabro` graphs
  plus optional `.toml` run configs as the execution entrypoints.

- Observation: Fabro's examples already span several distinct workflow
  families, not one universal "implement" loop.
  Evidence: `fabro/workflows/implement/workflow.fabro` is a code-first
  implement/simplify/verify loop; `docs/examples/clone-substack.mdx` shows a
  large ensemble-planning and verify-chain workflow; `docs/examples/
  definition-of-done.mdx` shows an audit/triage/fix loop; `docs/examples/
  semantic-port.mdx` shows a recurring backlog-processing loop; and
  `docs/examples/solitaire.mdx` shows phased implementation with verification
  gates after each layer.

- Observation: Myosu's current bootstrap lanes are defined enough to generate
  curated lane doctrine, but not yet to perform real product implementation.
  Evidence: the bootstrap workflows only produce `spec.md` and `review.md`
  artifacts, so the first true code-writing lane needed its own workflow and
  artifact contract (`implementation.md`, `verification.md`) rather than more
  bootstrap prompt edits.

- Observation: once the bootstrap artifacts existed, the `games:traits`
  implementation lane was strong enough to complete a real code slice without
  further prompt tuning.
  Evidence: the direct Fabro implementation run replaced the absolute
  robopoker path dependencies with pinned git dependencies and wrote both
  `implementation.md` and `verification.md` under `outputs/games/traits/`.

- Observation: Fabro treats raw run directories as internal implementation
  detail, not as a durable external contract.
  Evidence: `docs/reference/run-directory.mdx` explicitly warns that the run
  directory structure is internal and recommends stable machine-consumption via
  `fabro inspect`.

- Observation: Raspberry's control-plane model is artifact- and milestone-
  oriented rather than task-list-oriented.
  Evidence: `test/fixtures/raspberry-supervisor/myosu-program.yaml` models
  units, lanes, artifacts, milestones, proof profiles, and checks; lane success
  is expressed as durable artifact production plus health/proof/precondition
  signals rather than as a mutable task checklist.

- Observation: Fabro's execution plane already cleanly separates work product
  from control-plane truth.
  Evidence: the docs in `docs/agents/outputs.mdx`,
  `docs/execution/context.mdx`, and `docs/execution/checkpoints.mdx` show that
  code changes and stage outputs live in Fabro run branches and metadata,
  while the Raspberry layer only needs stable lane-level truth and curated
  deliverables.

- Observation: a small, lane-specific bootstrap surface is enough to make
  Myosu legible to both Fabro and Raspberry without pretending the whole repo
  is migrated already.
  Evidence: the new `fabro/` tree contains four lane workflows, four run
  configs, one program manifest, shared prompts, and minimal proof scripts,
  while `outputs/` only seeds the trusted and restart lanes we actually have
  judgments for.

- Observation: the full Raspberry frontier stack was easier to seed by using
  contract-first artifact lanes than by waiting for implementation-ready code
  in every area.
  Evidence: `myosu-services.yaml`, `myosu-product.yaml`,
  `myosu-platform.yaml`, and `myosu-recurring.yaml` could all be checked in
  honestly once their lanes were framed as `spec.md`/`review.md` contract work
  over stable output roots, even though most of those areas are not yet ready
  for implementation-family workflows.

- Observation: the biggest remaining Raspberry gap inside Fabro is not manifest
  shape anymore; it is the stability of the lane-to-run-truth bridge.
  Evidence: the latest Fabro plan has already landed generalized manifests,
  richer lane state, scoped checks, explicit state paths, and Myosu fixtures.
  The most brittle remaining behavior is still in runtime refresh and command
  rendering, where Raspberry currently infers live Fabro runs by scanning
  run directories and matching `run.toml` contents.

- Observation: Fabro's own docs explicitly say raw run-directory layout is an
  internal detail, which makes Raspberry's current direct filesystem coupling a
  temporary bridge, not a durable foundation.
  Evidence: `docs/reference/run-directory.mdx` says consumers should prefer
  stable inspection surfaces, while `raspberry-supervisor` currently uses
  `latest_fabro_run_for_lane()` to scan `~/.fabro/runs/`.

## Decision Log

- Decision: keep the next slice focused on doctrine and execution-entrypoint
  retargeting rather than on rewriting the whole product corpus.
  Rationale: the cleanup slice already established the new source of truth, and
  the next blocker is repo-level adoption, not a full product-doc rewrite.
  Date/Author: 2026-03-19 / Codex

- Decision: do not assume `WORKFLOW.md`, `project.yaml`, or `ops/` should be
  preserved just because they currently exist.
  Rationale: the user explicitly clarified that these are Malinka remnants, so
  the Fabro-first migration should evaluate replacement and deletion as first-
  class options rather than treating them as durable doctrine by default.
  Date/Author: 2026-03-19 / User + Codex

- Decision: delete `project.yaml` and `WORKFLOW.md` now instead of carrying
  them forward as compatibility bridges.
  Rationale: the user explicitly confirmed that Myosu is not using Malinka
  anymore, so keeping those files would prolong a false control-plane entry
  path.
  Date/Author: 2026-03-19 / User + Codex

- Decision: preserve the tested leaf crates (`myosu-games`, `myosu-tui`, and
  the agent-facing schema work) as salvageable assets.
  Rationale: these crates compile, have passing tests, and are architecturally
  decoupled from the broken chain fork. Rewriting them from zero would discard
  useful verified work.
  Date/Author: 2026-03-19 / Codex

- Decision: make `games:traits` the first non-bootstrap implementation lane.
  Rationale: `myosu-games` is a trusted leaf crate with passing tests, which
  makes it the safest place to prove a real Fabro implementation loop before
  attempting the same on the untrusted chain/runtime restart lanes.
  Date/Author: 2026-03-19 / Codex

- Decision: treat the direct Fabro implementation success as sufficient proof
  of the lane shape even though Raspberry's detached-submit handoff still needs
  one more hardening pass.
  Rationale: the lane contract, workflow, proof commands, and output artifacts
  are now proven end to end. The remaining bug is in the supervisory submit
  path, not in the Myosu lane definition itself.
  Date/Author: 2026-03-19 / Codex

- Decision: treat the current chain fork effort as a restart candidate rather
  than a trustworthy base.
  Rationale: the current pallet is a non-building transplant, the chain/runtime
  directories are not yet complete crates, and the workspace's main test path
  fails before any real product loop can be exercised.
  Date/Author: 2026-03-19 / Codex

- Decision: shape Myosu's execution plane around checked-in Fabro assets under
  `fabro/`: workflow graphs, run configs, prompt files, and helper scripts.
  Rationale: this is the stable author-time contract Fabro itself expects, and
  it is much closer to the execution substrate than `IMPLEMENT.md`.
  Date/Author: 2026-03-19 / Codex

- Decision: shape Myosu's control plane around Raspberry program manifests plus
  curated output roots, not around raw Fabro run directories.
  Rationale: Fabro's run directories are intentionally internal. Raspberry
  should consume stable lane truth and curated artifact roots, using Fabro's
  library/API/inspect surfaces as adapters where needed.
  Date/Author: 2026-03-19 / Codex

- Decision: use `outputs/` as the first durable artifact surface for Raspberry
  milestones.
  Rationale: Raspberry fixtures already model `output_root`, and `outputs/`
  expresses "curated lane deliverables" better than inheriting the Malinka-era
  `ops/` directory as a control-plane default.
  Date/Author: 2026-03-19 / Codex

- Decision: the first Fabro-native operational skeleton should start in
  standalone `fabro run` + Raspberry CLI mode, while keeping server mode as a
  later scaling target.
  Rationale: Fabro's docs show that both modes share the same engine. Starting
  with local checked-in run configs is the smallest truthful surface for
  Myosu's bootstrap phase.
  Date/Author: 2026-03-19 / Codex

- Decision: the first bootstrap program manifest should cover only four lanes:
  `games:traits`, `tui:shell`, `chain:runtime`, and `chain:pallet`.
  Rationale: these are the lanes we have enough evidence to classify today.
  Pulling miner, validator, play, or launch into the first manifest would add
  orchestration noise before the base execution/control shape is proven.
  Date/Author: 2026-03-19 / Codex

- Decision: Myosu's workflow-library design should be handled in a dedicated
  plan rather than folded into the bootstrap cutover plan.
  Rationale: selecting workflow families for chain restart, trusted-leaf
  continuation, service bringup, launch orchestration, recurring audits, and
  spec conformance is now a durable execution-architecture question, not a
  one-line bootstrap cleanup task.
  Date/Author: 2026-03-19 / Codex

- Decision: use lane-specific workflow graphs for the first bootstrap slice,
  even though their structure is repetitive.
  Rationale: this keeps each workflow's verify step and artifact paths explicit
  without depending on unproven variable interpolation behavior in the graph
  language.
  Date/Author: 2026-03-19 / Codex

- Decision: do not pre-create `spec.md` or `review.md` artifact files.
  Rationale: Raspberry milestones are artifact-backed. Pre-seeding those files
  would incorrectly mark lanes as already specified or reviewed.
  Date/Author: 2026-03-19 / Codex

- Decision: the highest-value next Raspberry step in Fabro is a stable
  Fabro-to-Raspberry run-truth adapter, not more manifest surface area.
  Rationale: manifest and lane-state semantics have already moved far enough to
  support Myosu-like programs. The remaining bottleneck is that Raspberry still
  depends on an internal run-directory heuristic where it should depend on a
  stable Fabro inspection surface.
  Date/Author: 2026-03-19 / Codex

- Decision: recurring compiler and trust/landing work should remain behind the
  run-truth and bootstrap-render replacement work.
  Rationale: the latest Fabro plan and code show that Raspberry's core program
  model is still settling. Layering recurring/trust semantics on top before the
  watch/status/execute core is stabilized would harden the wrong abstractions.
  Date/Author: 2026-03-19 / Codex

- Decision: seed all major frontiers as contract-first Raspberry programs
  before deepening execution semantics inside each one.
  Rationale: this gives Myosu a complete, legible program-of-programs control
  plane now, while still letting execution maturity grow frontier by frontier.
  Date/Author: 2026-03-19 / Codex

- Decision: keep `ralph/IMPLEMENT.md` and `ops/malinka-capabilities.md` only as
  archival redirect files, with detailed legacy content moved alongside them.
  Rationale: this preserves historical context without allowing the old files to
  masquerade as the active control plane.
  Date/Author: 2026-03-19 / Codex

## Outcomes & Retrospective

This plan is now partially executed. The cleanup slice already created the new
canonical planning surface, and the first review inside this follow-on slice
produced a concrete keep-vs-reset result:

- keep the tested game-trait, TUI, and schema leaf crates
- reset the current chain fork effort to a smaller, Fabro-first restart plan
- do not preserve Malinka-era operational files by default

That translation is now largely done. The repo now has:

- retargeted doctrine entrypoints
- archived Malinka-only control files
- a seeded Fabro execution tree
- a seeded Raspberry program-of-programs stack
- one real implementation lane already proven (`games:traits`)

The docs review also made the target shape much clearer. Myosu does not need to
invent a new execution contract to satisfy Fabro. It needs to check in the
right kinds of author-time assets and expose the right kinds of control-plane
artifacts for Raspberry:

- execution plane: workflows, run configs, prompt files, helper scripts
- control plane: program manifest, curated outputs, checks, proof profiles, and
  lane state

The initial checked-in skeleton for that shape now exists:

- `fabro/README.md`
- `fabro/programs/myosu-bootstrap.yaml`
- `fabro/workflows/bootstrap/*.fabro`
- `fabro/run-configs/bootstrap/*.toml`
- `fabro/prompts/bootstrap/*.md`
- `fabro/checks/*.sh`
- `outputs/README.md`
- `outputs/games/traits/`
- `outputs/tui/shell/`
- `outputs/chain/runtime/`
- `outputs/chain/pallet/`

That skeleton has now expanded into a fuller Raspberry stack:

- `fabro/programs/myosu-bootstrap.yaml`
- `fabro/programs/myosu-chain-core.yaml`
- `fabro/programs/myosu-services.yaml`
- `fabro/programs/myosu-product.yaml`
- `fabro/programs/myosu-platform.yaml`
- `fabro/programs/myosu-recurring.yaml`
- `fabro/programs/myosu-games-traits-implementation.yaml`

And the root newcomer path is now explicit in `README.md`, which points first
to:

- `SPEC.md`
- `PLANS.md`
- `specs/031626-00-master-index.md`
- `plans/031826-bootstrap-fabro-primary-executor-surface.md`
- `fabro/programs/myosu-bootstrap.yaml`

The highest-value next steps are now execution-oriented rather than structure-
oriented:

1. Execute the ready bootstrap contract lanes so more frontiers gain reviewed
   artifacts under `outputs/`.
2. Promote the next honest implementation lane after `games:traits`; the most
   likely candidates are `tui:shell` or a runtime restart phase.
3. Replace the remaining bootstrap `status` / `watch` behavior in Raspberry
   with the ported semantics so there is one control-plane behavior to extend.
4. Introduce a stable Fabro↔Raspberry run-truth adapter and bind `execute` to
   authoritative Fabro run ids.
5. Only after the run-truth bridge is stable, deepen recurring/trust-plane
   execution beyond contract-first recurring lanes.

## Context and Orientation

The earlier cutover slices created the new `specs/`, `plans/`, and
`specsarchive/` directories plus root `SPEC.md` and `PLANS.md`, rewrote the
active doctrine files, and deleted the old Malinka-only control files.

This plan is no longer about cutover uncertainty. It is now the top-level
status document for the seeded Fabro execution tree and Raspberry program
stack.

This plan also records the bootstrap trust review of the code that already
exists. That review established three categories:

- trusted enough to keep: `crates/myosu-games/`, `crates/myosu-tui/`, and the
  machine-readable schema work
- selectively salvageable: copied primitives like `safe-math` and `share-pool`
- restart from scratch: the current chain fork / pallet transplant

Fabro's docs add a second important distinction:

- the **execution plane** is the checked-in author-time material that Fabro can
  run directly
- the **control plane** is the Raspberry program that decides which lanes are
  ready, blocked, healthy, and complete based on artifacts, checks, and Fabro
  run truth

## Milestones

### Milestone 1: Retarget doctrine references

At the end of this milestone, repo-level doctrine files point first to the new
canonical `specs/` and `plans/` surface. The proof is that a contributor can
find current doctrine without being sent first into the archive or `ralph/`.

### Milestone 2: Define the first Fabro-native operational skeleton

At the end of this milestone, Myosu has a small but explicit Fabro-native
operational surface for its real workstreams. The proof is a checked-in
program, manifest, or run-config skeleton that names Myosu units such as chain,
miner, validator, gameplay, and operations, but it should be built around the
trusted surfaces from the code review rather than around the broken chain fork.

### Milestone 3: Prove newcomer orientation

At the end of this milestone, validation commands and path checks show that a
new contributor can locate current doctrine and active execution slices without
relying on the archive.

### Milestone 4: Convert the trust review into migration scope

At the end of this milestone, the repo has an explicit keep/reset decision for
each major work area: trusted leaf crates retained, copied primitives either
rewired or dropped, and the chain fork moved to a restart plan. The proof is
that future Fabro work no longer assumes the current chain transplant is a safe
base.

## Plan of Work

Start by auditing remaining repo references to legacy planning surfaces and by
classifying existing code into keep, salvage, or reset buckets. Then update the
top-level doctrine files so they point first to `SPEC.md`, `PLANS.md`,
`specs/`, and `plans/`, while removing stale references to deleted Malinka
files. Once the references are clean and the trusted code surfaces are known,
define the first Fabro-native operational skeleton for Myosu's actual
workstreams. Keep the first skeleton small and truthful; it needs to prove the
new direction, not replace every old workflow in one pass.

The execution-plane deliverables should be checked into a dedicated `fabro/`
tree. The control-plane deliverables should be checked into a Raspberry-facing
program manifest plus curated `outputs/` roots. Raw Fabro run directories may
exist for runtime state, but they are not themselves the durable deliverables.

### Proposed deliverable shape

Execution plane:

    fabro/
      workflows/
        bootstrap/
          game-traits.fabro
          tui-shell.fabro
          chain-runtime-restart.fabro
          chain-pallet-restart.fabro
      run-configs/
        bootstrap/
          game-traits.toml
          tui-shell.toml
          chain-runtime-restart.toml
          chain-pallet-restart.toml
      prompts/
        bootstrap/
          plan.md
          implement.md
          fix.md
          review.md
      checks/
        cargo-workspace.sh
        tui-snapshots.sh
        chain-ready.sh

Control plane:

    fabro/programs/
      myosu-bootstrap.yaml
    outputs/
      games/
        traits/
          spec.md
          review.md
      tui/
        shell/
          spec.md
          review.md
      chain/
        runtime/
          spec.md
          review.md
        pallet/
          spec.md
          review.md
      miner/
      validator/
      play/

The program manifest should use Raspberry's existing unit/lane model:

- **units** are durable Myosu surfaces such as `games`, `tui`, `chain`,
  `miner`, `validator`, and `play`
- **lanes** are independently schedulable streams such as `traits`, `shell`,
  `runtime`, `pallet`, `service`, `oracle`, or `tui`
- **artifacts** are curated checked-in outputs under `outputs/`
- **milestones** are artifact-backed lifecycle checkpoints such as `specified`,
  `reviewed`, `service_ready`, or `launch_ready`
- **checks** are preconditions, proofs, and health probes that determine lane
  readiness and service health

Code changes live on Fabro run branches and in the working tree as normal repo
changes. Curated deliverables live under `outputs/` and are what Raspberry uses
to decide whether a lane's milestone is durably satisfied.

## Concrete Steps

Work from the repository root.

1. Audit remaining legacy references.

       rg -n "specsarchive|specs/031626|ralph/IMPLEMENT.md|ralph/SPEC.md|Malinka|malinka" \
         OS.md AGENTS.md README.md plans specs

2. Audit the current codebase by trust level.

       cargo test -p myosu-games
       cargo test -p myosu-tui
       cargo check -p pallet-game-solver
       cargo test --workspace
       cargo check --manifest-path crates/myosu-chain/primitives/safe-math/Cargo.toml
       cargo check --manifest-path crates/myosu-chain/primitives/share-pool/Cargo.toml

3. Update the doctrine entrypoints.

       sed -n '1,220p' OS.md
       sed -n '1,260p' AGENTS.md

4. Add the first Fabro-native operational skeleton and explain it in the new
   canonical specs/plans surface.

       mkdir -p fabro/workflows/bootstrap fabro/run-configs/bootstrap fabro/prompts/bootstrap fabro/checks fabro/programs outputs

       # Seed a Raspberry program manifest with units, lanes, artifacts,
       # milestones, checks, and run-config paths.

       # Seed one workflow + one run config per trusted bootstrap lane.

5. Validate the resulting repo orientation.

       test -f SPEC.md
       test -f PLANS.md
       test ! -f project.yaml
       test ! -f WORKFLOW.md
       rg -n "SPEC.md|PLANS.md|plans/" OS.md AGENTS.md
       find fabro -maxdepth 3 -type f | sort
       find outputs -maxdepth 3 -type f | sort
       python - <<'PY'
       import tomllib
       from pathlib import Path
       for path in sorted(Path('fabro/run-configs/bootstrap').glob('*.toml')):
           tomllib.load(path.open('rb'))
           print(path.name)
       PY
       ./fabro/checks/games-traits.sh
       ./fabro/checks/tui-shell.sh
       ./fabro/checks/chain-runtime-reset.sh
       ./fabro/checks/chain-pallet-reset.sh

## Validation and Acceptance

Acceptance is complete when all of the following are true:

- repo-level doctrine files point at the new canonical planning surface first
- a first Fabro-native operational skeleton exists for real Myosu workstreams
- a newcomer can locate active doctrine and active plans without opening
  `specsarchive/`
- the repo has an explicit keep/reset decision for current code execution work
- the repo no longer depends on `project.yaml` or `WORKFLOW.md`
- the new execution surface does not assume `ops/` survives unchanged
- Fabro can find checked-in workflow graphs and run configs for the bootstrap
  lanes
- Raspberry can find a checked-in program manifest plus curated artifact roots
  for those same lanes
- a newcomer can open `README.md` and find the active doctrine and bootstrap
  operator loop without reading `specsarchive/`

## Idempotence and Recovery

This slice should be carried out additively. Prefer updating references and
adding new Fabro-native surfaces before deleting or downgrading legacy ones. If
uncertainty appears, keep the compatibility bridge and mark it explicitly
instead of removing it blindly.

## Artifacts and Notes

Expected primary files for this slice:

    OS.md
    AGENTS.md
    ops/
    specs/
    plans/
    fabro/
    outputs/
    crates/myosu-games/
    crates/myosu-tui/
    crates/myosu-chain/

## Interfaces and Dependencies

The key interfaces for this slice are repo-entrypoint interfaces:

- doctrine entrypoints in `OS.md` and `AGENTS.md`
- legacy operational surfaces under `ops/`
- the first Fabro-native operational surface for Myosu workstreams
- trust classification for existing code: keep, salvage, or reset
- execution-plane assets under `fabro/`
- control-plane assets under `fabro/programs/` and `outputs/`

Revision Note: Initial draft created during the cleanup slice so the new
canonical doctrine surface immediately points to the next bounded migration
step. Updated during the bootstrap review to capture the user's guidance that
`WORKFLOW.md`, `project.yaml`, and `ops/` are Malinka remnants, and to record
the current keep-vs-reset decision for existing code. Updated again after a
deeper Fabro docs review to define the execution-plane and control-plane
deliverable shapes Myosu should target. Updated again after seeding the first
checked-in `fabro/` execution assets, Raspberry program manifest, and curated
`outputs/` roots. Updated again after deleting `project.yaml` and
`WORKFLOW.md` so the plan now treats stale references to them as cleanup work
rather than migration work.

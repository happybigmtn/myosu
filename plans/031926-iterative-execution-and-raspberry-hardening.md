# Execute Myosu Frontiers and Harden Raspberry Iteratively

This ExecPlan is a living document. The sections `Progress`,
`Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must
be kept up to date as work proceeds.

`PLANS.md` is checked into the repository root and this document must be
maintained in accordance with it. This plan depends on
`plans/031826-clean-up-myosu-for-fabro-primary-executor.md`,
`plans/031826-bootstrap-fabro-primary-executor-surface.md`,
`plans/031926-design-myosu-fabro-workflow-library.md`,
`plans/031926-decompose-myosu-into-raspberry-programs.md`,
`specs/031826-fabro-primary-executor-decision.md`, and
`specs/031826-myosu-fabro-primary-executor-migration.md`. It also depends on
the active Fabro/Raspberry execution substrate in the sibling repository at
`/home/r/coding/fabro`.

## Purpose / Big Picture

After this slice lands, Myosu will stop being primarily a repository of seeded
Fabro assets and start behaving like a real multi-frontier Fabro/Raspberry
program. A contributor will be able to run the next ready bootstrap lanes,
watch those reviewed artifacts unblock chain-core, services, product,
platform, and recurring frontiers, and trust `raspberry execute/status/watch`
while that expansion happens.

The user-visible outcome is not another batch of manifests. The user-visible
outcome is that Myosu becomes iteratively executable: one run produces durable
artifacts, those artifacts unlock the next frontiers, and any control-plane
defects discovered during that buildout are fixed in Raspberry/Fabro instead of
worked around locally.

## Progress

- [x] (2026-03-19 05:40Z) Read the current Myosu ExecPlan framework in
  `PLANS.md` and re-read every live plan under `plans/`.
- [x] (2026-03-19 05:40Z) Audited the current Myosu `fabro/programs/`,
  `fabro/run-configs/`, `fabro/workflows/`, and `outputs/` surfaces after the
  latest implementation updates.
- [x] (2026-03-19 05:40Z) Confirmed that `games:traits` is no longer just
  bootstrapped; it has completed both a bootstrap run and the first real
  implementation run, producing `spec.md`, `review.md`, `implementation.md`,
  and `verification.md`.
- [x] (2026-03-19 05:40Z) Identified the next real execution sequence:
  remaining ready bootstrap lanes first, frontier unblocking second,
  Raspberry/Fabro run-truth hardening continuously.
- [x] (2026-03-19 06:35Z) Executed the remaining bootstrap-ready lanes and
  captured reviewed artifacts under `outputs/` for `tui:shell`,
  `chain:runtime`, and `chain:pallet`.
- [x] (2026-03-19 06:35Z) Used those artifacts to unblock the next frontier
  programs: services are now ready, product has advanced through `play:tui`,
  platform has advanced through `games:poker-engine`, and recurring remains
  ready across all four lanes.
- [ ] Fix Raspberry/Fabro defects only when they are discovered by real Myosu
  execution, then rerun the affected frontier until `execute/status/watch`
  truth is trustworthy again.
- [x] (2026-03-19 06:16Z) Proved at least one non-`games:traits` frontier
  progresses via Raspberry dispatch by submitting `games:multi-game` through
  `raspberry execute`.
- [x] (2026-03-19 06:44Z) Completed the next live frontier round directly
  through Fabro: `games:multi-game`, `validator:oracle`, `miner:service`, and
  `sdk:core` all now have reviewed artifacts under `outputs/`.
- [ ] Convert the current Raspberry-dispatched `games:multi-game` false-submit
  into a truthful failure or successful live run, then rerun the lane with the
  repaired Fabro detach path.
- [x] (2026-03-19 06:44Z) Finished the direct Fabro foreground runs for
  `validator:oracle` and `games:multi-game`, then rerendered
  services/platform against the new reviewed artifacts.
- [x] (2026-03-21 04:15Z) Executed the `agent:experience` bootstrap lane in
  reviewed-artifact terms, then wrote `outputs/agent-integration/` to decide
  the next honest product move: product needs an implementation family next,
  with `play:tui` as the first implementation lane and `agent:experience`
  following after `myosu-play` exists.

## Surprises & Discoveries

- Observation: the planning surface has already moved from “define the
  control-plane shape” to “run the shape and fix the friction.”
  Evidence: the existing plans cover cleanup, bootstrap doctrine, workflow
  families, and multi-program decomposition. The newly completed
  `games:traits` bootstrap and implementation outputs show that another
  planning-only slice would mostly restate work that now needs to be executed.

- Observation: `games:traits` is now the strongest proof that the current
  Fabro/Raspberry split can drive a real Myosu lane.
  Evidence: `outputs/games/traits/spec.md`,
  `outputs/games/traits/review.md`,
  `outputs/games/traits/implementation.md`, and
  `outputs/games/traits/verification.md` all exist and were produced by real
  Fabro runs, not handwritten stubs.

- Observation: the remaining frontier manifests are mostly blocked by missing
  reviewed artifacts, not by missing YAML structure.
  Evidence: `myosu-services.yaml` gates on
  `outputs/chain/runtime/review.md` and `outputs/chain/pallet/review.md`;
  `myosu-product.yaml` gates on `outputs/games/traits/review.md` and
  `outputs/tui/shell/review.md`; `myosu-platform.yaml` gates on
  `outputs/games/traits/review.md` and then on downstream reviewed outputs.

- Observation: the most valuable Fabro/Raspberry fixes are now discovered by
  running Myosu lanes for real.
  Evidence: the recent fixes in the sibling Fabro repo came directly from
  attempting real Myosu runs: relative `directory = ...` resolution,
  detached-run argv rebuilding, and inspect-by-run-id support.

- Observation: the bootstrap frontier is now fully completed in reviewed-artifact
  terms, and the next pressure comes from services/platform execution rather
  than from more bootstrap scaffolding.
  Evidence: `outputs/tui/shell/review.md`,
  `outputs/chain/runtime/review.md`, and
  `outputs/chain/pallet/review.md` now exist alongside the already-complete
  `games:traits` outputs, and `myosu-services.yaml` now reports both lanes
  ready.

- Observation: the current Fabro detach path is still not trustworthy even
  when `raspberry execute` returns a run id.
  Evidence: `games:multi-game` was submitted through Raspberry as
  `01KM2BS4ASVRXVT2ND1GVVMKJ0`, but the run directory never gained a `run.pid`
  or manifest; only `status.json=submitted` and a `detach.log` parse failure
  were present.

- Observation: direct foreground Fabro execution remains the strongest honest
  fallback while detach is being hardened.
  Evidence: fresh foreground runs for `games:multi-game`
  (`01KM2CGPHAJ95J38TQ7SPN46NZ`) and `validator:oracle`
  (`01KM2CGPCCC86SHEEQ6QFTRFEM`) are live with real manifests, states, and
  stage labels.

- Observation: the MiniMax bridge currently depends on launching Fabro from an
  interactive shell, because `.bashrc` returns early for non-interactive
  shells before exporting `MINIMAX_API_KEY` / `ANTHROPIC_*`.
  Evidence: the failed foreground runs
  (`01KM2CGPHAJ95J38TQ7SPN46NZ`, `01KM2CGPCCC86SHEEQ6QFTRFEM`) showed
  `apiKeySource: "none"` and repeated 401 `authentication_failed` retries,
  while relaunching the same lanes through `bash -ic` succeeded.

- Observation: services and platform are now bootstrap-complete in reviewed
  artifact terms.
  Evidence: `outputs/validator/oracle/spec.md`,
  `outputs/validator/oracle/review.md`,
  `outputs/miner/service/spec.md`,
  `outputs/miner/service/review.md`,
  `outputs/games/multi-game/spec.md`,
  `outputs/games/multi-game/review.md`,
  `outputs/sdk/core/spec.md`, and
  `outputs/sdk/core/review.md` all exist, and Raspberry now reports
  `myosu-services` and `myosu-platform` fully complete.

- Observation: the product frontier no longer needs another upstream-unblock
  pass before implementation-family work starts.
  Evidence: `outputs/play/tui/review.md` and
  `outputs/agent/experience/review.md` both recommend implementation-family
  work, and `outputs/games/traits/verification.md` already reduced the older
  robopoker portability blocker those product reviews still mention.

- Observation: the current `games:traits` implementation success does not mean
  the supervisory path is “done”; it means direct Fabro execution is strong
  enough that remaining Raspberry defects can now be isolated against a known
  good execution substrate.
  Evidence: direct Fabro implementation runs succeeded for
  `games:traits`, while some Raspberry detached-submit paths still required
  hardening in the sibling Fabro repo.

## Decision Log

- Decision: make the next plan execution-first rather than manifest-first.
  Rationale: the repository now has enough program manifests, workflow
  families, and run configs to learn more by running them than by describing
  them again.
  Date/Author: 2026-03-19 / Codex

- Decision: treat the latest `fabro/` updates as the new baseline, not as
  speculative future work.
  Rationale: the current Myosu tree already includes seeded frontiers for
  bootstrap, chain-core, services, product, platform, and recurring oversight,
  plus a completed `games:traits` implementation example.
  Date/Author: 2026-03-19 / Codex

- Decision: sequence execution by artifact-unblock leverage rather than by
  historical task numbering.
  Rationale: the next best Myosu lanes are whichever produce reviewed artifacts
  that unlock the most downstream frontiers. Right now that means
  `tui:shell`, `chain:runtime`, then `chain:pallet`.
  Date/Author: 2026-03-19 / Codex

- Decision: treat Raspberry/Fabro defects as first-class scope within this
  execution slice instead of treating them as a separate repo concern.
  Rationale: Myosu is now the proving ground for Raspberry. When a run-truth,
  detached-submit, or status/watch defect blocks Myosu progress, the correct
  move is to fix it in `/home/r/coding/fabro` and resume the lane.
  Date/Author: 2026-03-19 / Codex

- Decision: do not seed more frontier manifests until at least one additional
  frontier beyond `games:traits` has actually progressed through execution.
  Rationale: the main risk is no longer “missing decomposition.” The main risk
  is believing the decomposition works without enough executed evidence.
  Date/Author: 2026-03-19 / Codex

- Decision: stop relying on `--detach` for new execution progress until the
  false-submit path is repaired, and use direct foreground Fabro runs when the
  next artifact matters more than the control-plane experiment.
  Rationale: the current detach path can emit a run id and still fail to start
  a worker, which is worse than a slower but truthful foreground run while the
  bug is being isolated.
  Date/Author: 2026-03-19 / Codex

- Decision: launch Fabro through an interactive shell for MiniMax-backed
  execution until the MiniMax bridge env is made available to non-interactive
  shells in a less fragile way.
  Rationale: the current `.bashrc` layout guards the export lines behind an
  interactive-shell early return, so non-interactive Fabro launches do not
  reach the Anthropic-compatible bridge credentials.
  Date/Author: 2026-03-19 / Codex

- Decision: treat `play:tui` as the first product implementation lane and
  `agent:experience` as the immediate follow-on extension lane.
  Rationale: `play:tui` creates the `myosu-play` binary that
  `agent:experience` extends, while `agent:experience` Slices 3-9 depend on
  that binary existing.
  Date/Author: 2026-03-21 / Codex

## Outcomes & Retrospective

This plan starts after the biggest conceptual migration work is already done.
Myosu now has:

- Fabro-native execution assets under `fabro/`
- multiple Raspberry frontier manifests under `fabro/programs/`
- durable artifact roots under `outputs/`
- one fully proven bootstrap-to-implementation example in `games:traits`

So the next outcome worth chasing is not another design artifact. It is a
repeatable execution loop:

1. run the next ready lanes
2. collect their reviewed outputs
3. let those outputs unlock the next frontiers
4. improve Raspberry/Fabro only where real runs expose defects

If this plan succeeds, the repository will gain real forward motion across the
frontier map instead of only better documentation about the frontier map.

## Context and Orientation

This repository now has two active execution/control-plane layers.

The first layer is the Myosu author-time execution surface under `fabro/`:

- `fabro/workflows/` contains Graphviz workflow graphs
- `fabro/run-configs/` contains TOML run configs
- `fabro/prompts/` contains lane-family prompts
- `fabro/checks/` contains proof and readiness scripts
- `fabro/programs/` contains Raspberry program manifests

The second layer is the durable artifact surface under `outputs/`, which
Raspberry uses for milestone truth:

- `outputs/games/traits/` is complete through implementation and verification
- `outputs/tui/shell/` still has only `.gitkeep`
- `outputs/chain/runtime/` still has only `.gitkeep`
- `outputs/chain/pallet/` still has only `.gitkeep`
- the other frontier output roots are still empty stubs

The current live plans already established the broad map:

- `031826-clean-up...` created the Fabro-aligned planning surface
- `031826-bootstrap...` created and then exercised the first bootstrap lanes
- `031926-design...` mapped workflow families to lane types
- `031926-decompose...` split Myosu into multiple Raspberry frontier programs

The important current program manifests are:

- `fabro/programs/myosu-bootstrap.yaml`
- `fabro/programs/myosu-chain-core.yaml`
- `fabro/programs/myosu-services.yaml`
- `fabro/programs/myosu-product.yaml`
- `fabro/programs/myosu-platform.yaml`
- `fabro/programs/myosu-recurring.yaml`
- `fabro/programs/myosu-games-traits-implementation.yaml`

The key execution fact right now is:

- `games:traits` has already produced reviewed and implemented artifacts
- `tui:shell` and `chain:runtime` are still bootstrap-ready and should be run
  next
- `chain:pallet` depends on `chain@runtime_reviewed`
- services depend on both chain reviewed artifacts
- product depends on `games:traits/review.md` and `tui:shell/review.md`
- platform depends on `games:traits/review.md`, then on its own reviewed
  outputs
- recurring lanes have mixed dependencies, with security explicitly depending
  on `outputs/chain/runtime/review.md`

The sibling Fabro repo at `/home/r/coding/fabro` must be treated as active
work scope for this plan. The latest Myosu execution attempts already depended
on fixes there:

- stable inspect-by-run-id support
- detached-run argv rebuilding
- relative `directory = ...` resolution in run configs

This plan assumes that additional Raspberry/Fabro fixes will continue to be
required as more frontiers start running.

## Milestones

### Milestone 1: Finish the Remaining Ready Bootstrap Lanes

At the end of this milestone, `tui:shell`, `chain:runtime`, and then
`chain:pallet` have all been run through their bootstrap workflows, producing
real `spec.md` and `review.md` artifacts under `outputs/`. The proof is that
those files exist, `fabro inspect <run_id>` shows successful runs, and
Raspberry marks the corresponding lanes complete or milestone-satisfied.

### Milestone 2: Unlock the Next Frontier Programs with Real Artifacts

At the end of this milestone, the newly created reviewed artifacts are being
consumed by at least two downstream frontier manifests. The proof is that
Raspberry no longer reports those downstream lanes as blocked on missing
preconditions and at least one of the following frontiers has started moving:
`myosu-services`, `myosu-product`, `myosu-platform`, or `myosu-recurring`.

### Milestone 3: Tighten Raspberry/Fabro from Real Execution Failures

At the end of this milestone, execution-induced defects in
`/home/r/coding/fabro` have been fixed iteratively and re-verified against the
same Myosu lane that exposed them. The proof is not just green unit tests in
Fabro; it is that `raspberry execute/status/watch` become trustworthy enough to
resume the blocked Myosu frontier without ad hoc cleanup.

## Plan of Work

Start from the current strongest foothold: `games:traits` is already proven, so
do not revisit it unless a downstream frontier exposes a concrete dependency or
schema gap. Instead, run the remaining ready bootstrap lanes in the order that
maximizes downstream unblocking:

1. `tui:shell`
2. `chain:runtime`
3. `chain:pallet`

After each lane finishes, immediately inspect the resulting `spec.md` and
`review.md`, update the relevant program manifest state if needed, and rerender
Raspberry status for the affected frontier manifests. Do not batch all three
lanes and only inspect at the end; the whole point is to let each completed
artifact unlock the next frontier as soon as it exists.

Once `tui:shell/review.md` exists, rerender `myosu-product.yaml` and confirm
whether `play:tui` becomes ready. Once both `chain/runtime/review.md` and
`chain/pallet/review.md` exist, rerender `myosu-services.yaml` and
`myosu-recurring.yaml` to confirm the service and security frontiers can move.
Once `games:traits/review.md` and a `poker-engine/review.md` exist, rerender
`myosu-platform.yaml` to check whether `multi-game` and `sdk:core` are ready.

Whenever a run fails for reasons that belong to Raspberry or Fabro rather than
to Myosu lane design, stop and fix the sibling Fabro repo before continuing.
Examples include:

- detached child submission bugs
- stale `submitted` runs poisoning Raspberry state
- status/watch refresh reading the wrong run truth
- relative path handling for run configs and output roots

After each such fix, rerun the exact Myosu lane that exposed it rather than
assuming the fix is complete because the Fabro unit tests pass.

Do not add new program manifests during this slice. If a frontier cannot yet be
executed because its workflow family is too weak, create or refine the
necessary workflow and run config in place, but keep the frontier count stable.

## Concrete Steps

Work from the repository root unless otherwise stated.

1. Re-read the current execution frontiers and outputs.

       sed -n '1,260p' fabro/programs/myosu-bootstrap.yaml
       sed -n '1,260p' fabro/programs/myosu-chain-core.yaml
       sed -n '1,260p' fabro/programs/myosu-services.yaml
       sed -n '1,260p' fabro/programs/myosu-product.yaml
       sed -n '1,260p' fabro/programs/myosu-platform.yaml
       sed -n '1,260p' fabro/programs/myosu-recurring.yaml
       find outputs -maxdepth 3 -type f | sort

2. Run the next ready bootstrap lanes.

       bash -ic 'cd /home/r/coding/myosu && /home/r/.cache/cargo-target/debug/fabro run --detach fabro/run-configs/bootstrap/tui-shell.toml'
       bash -ic 'cd /home/r/coding/myosu && /home/r/.cache/cargo-target/debug/fabro run --detach fabro/run-configs/bootstrap/chain-runtime-restart.toml'
       bash -ic 'cd /home/r/coding/myosu && /home/r/.cache/cargo-target/debug/fabro run --detach fabro/run-configs/bootstrap/chain-pallet-restart.toml'

3. Inspect each run and confirm its artifacts landed.

       /home/r/.cache/cargo-target/debug/fabro inspect <run_id>
       find outputs/tui/shell -maxdepth 1 -type f | sort
       find outputs/chain/runtime -maxdepth 1 -type f | sort
       find outputs/chain/pallet -maxdepth 1 -type f | sort

4. Rerender the affected Raspberry frontier programs after each completed
   bootstrap lane.

       /home/r/.cache/cargo-target/debug/raspberry status --manifest fabro/programs/myosu-product.yaml
       /home/r/.cache/cargo-target/debug/raspberry status --manifest fabro/programs/myosu-chain-core.yaml
       /home/r/.cache/cargo-target/debug/raspberry status --manifest fabro/programs/myosu-services.yaml
       /home/r/.cache/cargo-target/debug/raspberry status --manifest fabro/programs/myosu-platform.yaml
       /home/r/.cache/cargo-target/debug/raspberry status --manifest fabro/programs/myosu-recurring.yaml

5. When a Raspberry/Fabro defect is discovered, switch to the sibling Fabro
   repo, fix it, verify it there, then immediately rerun the blocked Myosu
   lane that exposed it.

       cd /home/r/coding/fabro
       cargo test -p fabro-cli -- --nocapture
       cargo test -p raspberry-cli -- --nocapture
       git diff --check

6. After at least one downstream frontier becomes ready, execute it through
   Raspberry first, not via ad hoc direct Fabro commands, so the control-plane
   loop stays honest.

       bash -ic 'cd /home/r/coding/myosu && cargo run --manifest-path /home/r/coding/fabro/lib/crates/raspberry-cli/Cargo.toml -- execute --manifest fabro/programs/myosu-services.yaml --fabro-bin /home/r/.cache/cargo-target/debug/fabro'

## Validation and Acceptance

Acceptance is complete when all of the following are true:

- `outputs/tui/shell/spec.md` and `outputs/tui/shell/review.md` exist
- `outputs/chain/runtime/spec.md` and `outputs/chain/runtime/review.md` exist
- `outputs/chain/pallet/spec.md` and `outputs/chain/pallet/review.md` exist
- at least one downstream frontier beyond `games:traits` moves from blocked to
  ready or running because of those reviewed artifacts
- any Fabro/Raspberry defects found during that execution have been fixed in
  `/home/r/coding/fabro` and verified against the same Myosu path
- `raspberry status` for the relevant Myosu frontier manifests matches the
  actual completed or running Fabro runs

The strongest proof is a small observable narrative:

1. run a bootstrap lane
2. inspect its successful Fabro run
3. see the new artifacts in `outputs/`
4. rerender a downstream Raspberry program
5. observe that a previously blocked lane is now ready or running

## Idempotence and Recovery

This plan should be executed in small loops rather than in one giant batch.
That makes recovery simple:

- if a Fabro run fails, inspect it and either rerun the lane or fix Fabro
- if Raspberry state becomes stale, correct the supervisory bug rather than
  hand-editing output artifacts
- if a bootstrap run writes weak artifacts, rerun the same lane with the same
  run config after updating the relevant prompt or Fabro control-plane fix

Do not delete successful output artifacts just to force a rerun. If a lane
needs another pass, overwrite the artifacts through another Fabro run so the
execution history remains truthful.

If a detached Fabro run is left in `submitted` because the child exited before
the engine truly started, treat that as a Fabro defect and fix it in
`/home/r/coding/fabro`. Do not paper over it by manually marking the Myosu lane
complete.

## Artifacts and Notes

Current anchor artifacts proving the execution model:

    outputs/games/traits/spec.md
    outputs/games/traits/review.md
    outputs/games/traits/implementation.md
    outputs/games/traits/verification.md

Current execution/control roots:

    fabro/programs/
    fabro/run-configs/
    fabro/workflows/
    outputs/
    /home/r/coding/fabro/lib/crates/raspberry-supervisor/
    /home/r/coding/fabro/lib/crates/raspberry-cli/
    /home/r/coding/fabro/lib/crates/fabro-cli/

## Interfaces and Dependencies

This slice depends on several stable interfaces.

The Myosu-side control-plane interfaces are:

- `fabro/programs/*.yaml` for Raspberry frontier truth
- `outputs/**/spec.md` and `outputs/**/review.md` for reviewed bootstrap
  milestones
- `outputs/**/implementation.md` and `outputs/**/verification.md` for real
  implementation-family lanes

The Fabro-side execution interfaces are:

- `fabro run <run-config.toml>` for direct lane execution
- `fabro inspect <run_id>` for stable machine-readable run truth
- `raspberry status --manifest <program.yaml>` for frontier state
- `raspberry execute --manifest <program.yaml>` for control-plane dispatch

The most important technical dependency is the Fabro-to-Raspberry run-truth
bridge in `/home/r/coding/fabro`. This plan assumes that when real Myosu
execution reveals defects there, those defects will be fixed as part of the
same iterative execution loop.

Revision Note: Created after reading all live Myosu plans and the expanded
`fabro/` execution surface, to move the repo from frontier seeding into
frontier execution and Raspberry hardening.

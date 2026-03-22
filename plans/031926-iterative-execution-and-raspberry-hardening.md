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
- [ ] Execute `agent:experience`, the last remaining ready product lane, then
  use its reviewed artifacts to decide whether product needs an implementation
  family next or another upstream unblock.
- [x] (2026-03-19 16:31Z) Proved a real repo-wide Myosu autodev loop in a git
  worktree (`autodev-live`) can synth-generate the next implementation-family
  programs and reinsert them into the top-level portfolio automatically.
- [x] (2026-03-19 16:31Z) Verified that the worktree-backed `myosu.yaml`
  portfolio now expands from the original 7 child frontiers to 11 by adding:
  `play-tui-implementation`, `games-poker-engine-implementation`,
  `games-multi-game-implementation`, and `sdk-core-implementation`.
- [x] (2026-03-19 16:31Z) Verified that those four newly synthesized
  implementation-family child programs become ready at the top level and are
  dispatched by the repo-wide autodev loop.
- [x] (2026-03-19 16:34Z) Confirmed from the live worktree autodev report that
  those four generated implementation-family programs stay in flight across
  later top-level cycles instead of disappearing after the first dispatch.
- [x] (2026-03-19 16:37Z) Captured the first live implementation-child failure
  signatures from the worktree run:
  - `play:tui-implement` is running in `Implement` but its proof command fails
    because package `myosu-play` does not exist yet
  - `games:poker-engine-implement` is running in `Implement` but its proof
    command fails because package `myosu-games-poker` does not exist yet
- [ ] Follow the four dispatched implementation child programs through actual
  code/artifact progress, fix any new Fabro/Raspberry defects they expose, and
  only then promote the worktree branch back toward trunk.
- [x] (2026-03-19 21:49Z) Raised the implementation-family quality bar so
  child runs now require a deterministic `quality.md` artifact and a
  `Quality Gate` before review/promotion can pass.
- [ ] Re-run the live worktree implementation children under that hardened
  contract and use the resulting failures to drive the next Raspberry/Fabro
  engineering loop.
- [x] (2026-03-19 22:20Z) Landed native Codex slot selection in Fabro so
  future OpenAI/Codex review/promote stages can use the saved local slot pool
  instead of always relying on the default `.codex` home.

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

- Observation: the current `games:traits` implementation success does not mean
  the supervisory path is “done”; it means direct Fabro execution is strong
  enough that remaining Raspberry defects can now be isolated against a known
  good execution substrate.
  Evidence: direct Fabro implementation runs succeeded for
  `games:traits`, while some Raspberry detached-submit paths still required
  hardening in the sibling Fabro repo.

- Observation: the current top-level Myosu portfolio is finally behaving like a
  real autonomous control plane rather than a static frontier index.
  Evidence: the worktree-backed `autodev-live` run expanded `myosu.yaml` with
  new implementation-family child programs, and a later cycle reported all four
  as ready and dispatched them automatically.

- Observation: the remaining execution risk has moved from “can synthesis
  generate the next frontier?” to “can the generated implementation frontiers
  make durable progress without bogging down evaluation surfaces?”
  Evidence: the implementation child programs are no longer structurally
  blocked, but top-level `status` and TUI refreshes now pay the cost of their
  proof commands unless we tighten the observer/runtime boundary.

- Observation: the current live worktree run is no longer a one-shot proving
  event; it is sustaining multiple implementation-frontier children in flight.
  Evidence: the top-level worktree report
  `/home/r/coding/myosu/.worktrees/autodev-live/.raspberry/myosu-autodev.json`
  shows cycles 2-6 with `running_after=4`.

- Observation: the next binding constraint is now inside the generated
  implementation contracts themselves.
  Evidence: the first implementation-child status samples show running proof
  checks failing on nonexistent package IDs (`myosu-play`,
  `myosu-games-poker`) rather than on portfolio wiring or readiness gating.

- Observation: those earliest missing-package failures are likely stale by now,
  because the generated crates now exist in the worktree and the corresponding
  manual `cargo build -p ...` probes block on the artifact lock instead of
  failing immediately.
  Evidence: `crates/myosu-play/`, `crates/myosu-games-poker/`, and
  `crates/myosu-sdk/` now exist under
  `/home/r/coding/myosu/.worktrees/autodev-live/`, and manual cargo probes are
  waiting on the artifact directory rather than reporting unknown package IDs.

- Observation: at least one generated implementation child has already
  completed successfully enough to satisfy its managed milestone.
  Evidence: `myosu-play-tui-implementation` now reports complete in the
  worktree status view, with its `implementation.md` / `verification.md`
  artifacts present, even though stale failure text from the first preflight
  still appears in lane detail.

- Observation: the first manual smoke test of the generated `myosu-play`
  binary shows that the slice is still a compile-only skeleton, not a usable
  gameplay TUI.
  Evidence: invoking the built binary as `myosu-play train` in the worktree
  exits immediately with code 0 and does not enter a terminal loop, which is
  consistent with the generated implementation artifact's note that the shell
  is created but not run.

- Observation: the next confidence improvement is now encoded in the generated
  contract itself.
  Evidence: refreshing `myosu-play-tui-implementation.yaml` in the worktree now
  adds a `promotion.md` artifact and a `merge_ready` milestone, so a future
  implementation completion can be gated on an explicit merge verdict instead
  of only on `implementation.md` and `verification.md`.

- Observation: compile-and-test proof is still outrunning product quality.
  Evidence: `play` and `sdk` can satisfy the current slice proof commands while
  still containing placeholder-heavy code paths and incomplete user-facing
  behavior, so the next iteration needs stronger deterministic quality checks
  inside the generated workflow itself.

- Observation: the first live refreshed implementation package now encodes that
  stronger quality contract directly in the worktree.
  Evidence: `myosu-play-tui-implementation.yaml` now carries a `quality`
  artifact and the regenerated `play-tui.fabro` now runs a `Quality Gate`
  before review/promotion, with paths rooted under `outputs/play/tui/`.

- Observation: the OpenAI/Codex auth problem was partly a launch-identity
  problem, not only a credentials problem.
  Evidence: the machine already had multiple saved Codex slot homes and slot
  rotator config, but Fabro was calling raw `codex exec` without selecting a
  `CODEX_HOME`, so autonomous runs were pinned to the default auth home.

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

- Decision: move live proving from disposable `/tmp` copies to a real Myosu git
  worktree.
  Rationale: the user wants real production-shaped repo data, and a worktree
  preserves isolation while making the autonomous loop's outputs commitable.
  Date/Author: 2026-03-19 / User + Codex

- Decision: treat “great code” as a workflow-contract problem before treating
  it as a raw-model problem.
  Rationale: the system currently accepts compile-only slices because the
  synthesized implementation contract does not yet demand deterministic
  evidence about warnings, placeholders, smoke behavior, and artifact/code
  consistency.
  Date/Author: 2026-03-19 / User + Codex

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

The next execution outcome is sharper now: the worktree should stop promoting
implementation slices that are merely plausible. A child lane should only cross
`merge_ready` after the synthesized workflow has produced a deterministic
quality artifact and a promotion verdict that matches the actual code and proof
surfaces.

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

The current live proving target is no longer a temp copy. It is the real Myosu
git worktree:

- `/home/r/coding/myosu/.worktrees/autodev-live`

and the current repo-wide autodev loop has already produced the next generated
implementation family there.

Current live monitor snapshot:

- top-level portfolio expanded from 7 child programs to 11
- four generated implementation programs are in flight
- the worktree branch remains commit-capable while those runs proceed

Immediate next steps:

1. monitor the four generated implementation programs for first durable code or
   artifact changes
2. repair the generated implementation proof commands and/or first-slice
   synthesis so they target packages/modules that can actually be created from
   the current repo shape
3. inspect any failed or stalled child run in the worktree, not in `/tmp`
4. keep fixing Fabro/Raspberry only where the live worktree run exposes real
   blocking defects

Latest live note:

- `play-tui-implementation` has now been refreshed to require a new
  `promotion.md` artifact and a `merge_ready` milestone instead of treating
  `verified` as the final promotion bar
- the same promotion-hardening refresh is now being applied to the generated
  platform implementation-family programs

That shifts the next confidence goal from “did the lane write artifacts?” to
“did the generated workflow itself justify a merge-ready verdict?”

Current promotion-review policy:

- keep generated `promote` steps on `gpt-5.4`
- keep implementation workers on MiniMax
- only consider downgrading the promotion reviewer once the generated
  `promotion.md` contract and deterministic gates have proven reliable across
  several live implementation slices

Current active monitoring note:

- the implementation children appear to be past the “missing package” phase
- the immediate next question is whether they complete a first compile or stall
  indefinitely behind shared Cargo workspace contention
- the first answer has started to emerge: `play-tui-implementation` has
  completed, so the remaining question is how many of the other generated
  implementation children make the same transition without new synth/runtime
  fixes
- the current control-plane follow-on is to let finished implementation
  children become ready for rerun under the new `merge_ready` contract instead
  of pinning them as forever-`running` just because some artifacts exist

Latest live note:

- the lane-truth refresh has now converted all four generated implementation
  children from stale `running` to fresh `ready`
- the next live autodev cycle should therefore be able to re-dispatch them
  under the stronger promotion-aware workflow contract

Newest live note:

- the latest one-cycle top-level autodev pass saw all four implementation
  children as ready
- it dispatched two of them in that cycle, which matches the portfolio's
  `max_parallel = 2`

So the next monitoring question is straightforward: once those two finish, does
the next autodev pass dispatch the remaining two under the same refreshed
contract?

Latest live note:

- `games:poker-engine-implement` fresh rerun:
  - status `running`
  - stage `Implement`
  - run id `01KM3HX7QS8017YT3WPZECGCG9`
  - running proof check currently passing
- `games:multi-game-implement` fresh rerun:
  - status `running`
  - stage `Implement`
  - run id `01KM3HXYKR9YRZVZHE5F34FP2T`
  - running proof check currently passing

This is better than the first generation wave: the refreshed implementation
contracts are now getting through `Preflight` and into real `Implement` work
under live autodev control.

Current live monitor snapshot:

- still running:
  - `games:poker-engine-implement` (`01KM3HX7QS8017YT3WPZECGCG9`)
  - `games:multi-game-implement` (`01KM3HXYKR9YRZVZHE5F34FP2T`)
- ready and waiting for a free slot:
  - `play:tui-implement`
  - `sdk:core-implement`

The top-level worktree report now matches the intended scheduler behavior:
ready children are queued behind the portfolio's `max_parallel = 2` while the
two active implementation children remain in flight.
- the stronger answer is now: `play-tui-implementation` is good enough as a
  worktree checkpoint, but not yet good enough for automatic merge promotion
- the next target is to make that promotion rule universal across generated
  implementation programs, not just for `play:tui`

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

Latest live worktree review (2026-03-19):

- the generated `myosu-games-poker` and `myosu-games-liars-dice` crates are not
  just present on disk; direct proof reruns in the live worktree passed:
  - `cargo test -p myosu-games-poker`
  - `cargo test -p myosu-games-liars-dice`
- `outputs/games/poker-engine/promotion.md` and
  `outputs/games/multi-game/promotion.md` now exist with `merge_ready: yes`,
  but that is still slice-scoped rather than whole-lane completion
- `play:tui` remains a skeleton slice:
  - verification still centers on `cargo build -p myosu-play`
  - `crates/myosu-play/src/main.rs` still initializes `_shell` without running
    the shell loop
- `sdk:core` remains a skeleton slice:
  - verification is compile-only plus ignored placeholder tests
  - no `promotion.md` has landed yet
- the live worktree still emits at least one warning during direct package
  proof (`myosu-games-liars-dice` dead-code warning), so the current output is
  not yet at the repo's zero-warnings bar

This is meaningful progress: synth is producing real executable code in the
games frontiers. It is not yet end-to-end effective across the whole repo,
because play/sdk are still scaffold-first and the top-level orchestrator is not
yet continuously advancing the remaining ready implementation programs without
another explicit autodev pass.

Latest execution-loop note (2026-03-19, later):

- forcing a fresh `raspberry status` over the live worktree now rewrites
  `/home/r/coding/myosu/.worktrees/autodev-live/.raspberry/myosu-state.json`
  into truthful parent statuses:
  - `games-multi-game-implementation:program` -> `complete`
  - `games-poker-engine-implementation:program` -> `complete`
  - `play-tui-implementation:program` -> `ready`
  - `sdk-core-implementation:program` -> `ready`
- this came from a Raspberry harness fix, not from manual state editing
- another harness fix now makes top-level autodev defer `synth evolve` until
  the current ready frontier is consumed, which should improve actual
  development velocity for the remaining ready play/sdk work on the next live
  relaunch
- another harness fix now directs autonomous cargo work into
  `.raspberry/cargo-target` under the live target repo, to reduce contention
  with unrelated/background cargo activity in the same workspace
- live `play` / `sdk` promotion attempts also exposed two more harness bugs:
  - agent review/promote stages were ignoring node-level `provider: openai` and
    falling back to Anthropic
  - `promotion_check` could still pass on an old `promotion.md` after review /
    promote failed
- both of those are now being fixed in Fabro/Raspberry itself rather than
  treated as one-off repo anomalies
- stale child runtime records for `play` / `sdk` were reset in the proving
  worktree and the child implementation packages were re-evolved under the
  refreshed template
- fresh child runs are now in flight under the new workflow semantics:
  - `play:tui-implement` -> `01KM3SPS33KNYAFMK2RFY76HC1`
  - `sdk:core-implement` -> `01KM3SQ8SP199TQD61QSS353AB`
- this is the first live run wave in the worktree using:
  - regenerated implementation workflows
  - dedicated `.raspberry/cargo-target`
  - CLI-based `gpt-5.4` review/promote path
- the fresh child workflows now explicitly preserve the real proof commands for
  play/sdk instead of collapsing `verify` to a simple artifact-exists check
- current state:
  - both fresh child runs completed preflight successfully
  - both are currently in `Implement`
  - dedicated autonomous cargo target is actively being populated under
    `.raspberry/cargo-target`
- a direct proving-ground comparison now exists:
  - non-interactive child dispatches of the refreshed workflows hit 401-style
    auth failures in `Implement`
  - interactive-shell re-dispatches then recovered:
    - `play:tui-implement` -> `01KM3T2ADPSXBFN17FGPD0XW2K`
    - `sdk:core-implement` -> `01KM3T30RXG4K1GV0Q8KT4R7SP`
  - both interactive reruns cleared preflight and are back in `Implement`

Latest proving-ground update:

- Fabro/Raspberry auth propagation was tightened so project `fabro.toml`
  sandbox env is forwarded into backend execution, and Raspberry spawns Fabro
  through an interactive bash
- after those harness fixes, fresh non-interactive child dispatches succeeded:
  - `play:tui-implement` -> `01KM3T2ADPSXBFN17FGPD0XW2K`
  - `sdk:core-implement` -> `01KM3TE3PT8B2QAPQG2PSTHXB8`
- `play:tui-implement` is now visibly doing real agent work inside `Implement`
  instead of failing immediately on provider auth; its implement-stage log shows
  file reads, code edits, build/test proof reruns, and promotion artifact
  rewrites under the live worktree
- because that fresh `play` run also showed `Implement` writing
  `outputs/play/tui/promotion.md`, the implementation/fixup prompt contract in
  Fabro was tightened so promotion artifacts stay owned by the Promote stage

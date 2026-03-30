# Expand CI Proof Gates to Chain and Doctrine Surfaces

Status: Active next-step plan. Resume now that the stage-0 implementation
stack is closed and CI hardening is part of the chosen next phase.

This ExecPlan is a living document. Maintained per `genesis/PLANS.md`.

Provenance: Enhanced from `archive/genesis_1774729423/plans/010-ci-proof-gates-expansion.md`. Changes: added exact CI job definitions, chain compile commands, and doctrine check scripts.

## Purpose / Big Picture

Current CI only gates gameplay crates. The chain (216K lines, 13 pallets) has zero automated checks. Doc integrity (empty specs, duplicate mirrors) is not enforced. This plan extends CI to catch chain regressions, doctrine drift, and plan quality issues before merge.

After this plan, PR merges to trunk/main require: gameplay tests pass, chain compiles, canonical specs are non-empty, and genesis plans have milestones with proof commands.

## Progress

- [x] (2026-03-28) Confirmed CI gates only 4 gameplay crates.
- [x] (2026-03-29) Added a `chain-core` CI job that runs the honest stripped
  chain compile path: runtime, pallet, and node all check under
  `SKIP_WASM_BUILD=1`, with `fast-runtime` enabled for runtime and node.
- [x] (2026-03-29) Added a doctrine integrity CI job backed by a reusable
  repository script. The gate now enforces the stronger stage-0 rule from plan
  `002`: every active `031626-*` file in `specs/` must be indexed, every
  indexed target must exist, and no active canonical spec may be empty.
- [x] (2026-03-29) Added a genesis plan quality CI job backed by a reusable
  repository script. The gate now requires every numbered plan in the active
  range to expose at least one milestone heading and at least one `Proof
  command` / `Proof commands` label.
- [x] (2026-03-29) Added chain clippy to CI at deny-warnings level for the
  stripped runtime, live pallet, and node, because those commands are already
  green locally under the same `SKIP_WASM_BUILD=1` constraint.
- [x] (2026-03-29) Measured a warm-cache local timing estimate for the new job
  shape. The longest lane here is still the gameplay test bundle at ~48s,
  while doctrine, plan-quality, chain-core, and chain-clippy all stayed under
  ~2s after the build cache was hot.
- [x] (2026-03-30) Checked the hosted GitHub Actions surface directly and
  confirmed the remaining proof is not merely unobserved: the remote repo
  currently exposes no published workflows or runs, so the timing check is
  blocked on publishing the local `.github/` workflow set first.
- [x] (2026-03-30) Published the local workflow surface to GitHub on draft PR
  `#1` (`Publish hosted CI proof gates`) and captured three real hosted runs:
  `23729642411`, `23729721326`, and `23729737957`.
- [x] (2026-03-30) Fixed a hosted-run portability bug in the doctrine and
  plan-quality scripts by making them fall back to `find`/`grep` when `rg` is
  absent on the runner.
- [x] (2026-03-30) Added a fast `repo-shape` CI preflight so hosted runs fail
  on the real blocker first when GitHub is still behind the current local
  stage-0 workspace. The cargo, doctrine, and plan jobs now wait on that
  preflight instead of each rediscovering remote drift noisily.
- [x] (2026-03-30) Published that preflight to draft PR `#1` and observed the
  cleaner hosted result on run `23729944496`: `Stage-0 Repo Shape` failed in
  `4s`, and all downstream jobs skipped instead of emitting misleading cargo
  and doctrine noise.
- [x] (2026-03-30) Published the current stage-0 CI surface to the same draft
  PR branch and observed the first real hosted partial proof on run
  `23730119476`: `Stage-0 Repo Shape`, `Plan Quality`, and `Doctrine
  Integrity` all passed on the current repo surface before the cargo lanes
  began their longer compile/test/clippy work.
- [x] (2026-03-30) Verified the hosted warning about `actions/checkout@v4`
  against GitHub's latest release metadata and updated the workflow to
  `actions/checkout@v6` so future `010` evidence is not mixed with a known
  Node 20 deprecation warning.
- [x] (2026-03-30) Added workflow-level concurrency cancellation after needing
  to cancel stale run `23730119476` by hand. Future pushes on the same PR now
  replace older in-progress CI runs automatically instead of burning hosted
  time on superseded evidence.
- [x] (2026-03-30) Observed the first clean rerun under `actions/checkout@v6`
  and the new concurrency-aware workflow on run `23730306070`: `Stage-0 Repo
  Shape`, `Doctrine Integrity`, and `Plan Quality` all passed again with no
  Node 20 deprecation annotation visible in the early hosted output.
- [x] (2026-03-30) Let run `23730306070` settle and captured the first full
  current-surface hosted failures instead of treating the long lanes as merely
  pending: `Active Crates` failed in the full-package test step because two
  `myosu-play` startup-state tests depended on ambient local artifact
  discovery, while `Chain Core` and `Chain Clippy` both failed building
  `litep2p` because the runner lacked `protoc`.
- [x] (2026-03-30) Fixed those concrete blockers locally: the startup-state
  tests in `myosu-play` now use an explicit non-empty poker advice fixture
  instead of environment-dependent auto-discovery, the chain jobs install
  `protobuf-compiler`, and the node-owned smoke summary no longer trips
  deny-level Clippy on `expect()` once the chain lane gets far enough to lint
  the node binary.
- [ ] Verify GitHub-hosted CI runs complete in <15 minutes.

## Surprises & Discoveries

- Observation: Chain crates have strong existing tests (56K+ LOC in subtensor tests alone) but none run in CI.
  Evidence: `crates/myosu-chain/pallets/subtensor/src/tests/` has 35 test modules.
- Observation: The build environment truth matters more than aspirational CI
  YAML. `cargo check -p myosu-chain-runtime` still fails on a machine without
  `wasm32-unknown-unknown`, so the honest repo gate today is
  `SKIP_WASM_BUILD=1`, not an unstated target-install assumption.
  Evidence: local verification on 2026-03-29 failed without the wasm target
  and passed immediately with `SKIP_WASM_BUILD=1`.
- Observation: Running full pallet tests may take 30+ minutes. Need selective
  approach.
  Evidence: Pallet test suites are comprehensive but slow.
- Observation: The current plan corpus already converged on milestone headings,
  but not on a single exact proof-label spelling.
  Evidence: the new plan-quality check only needed to accept `Proof commands:`
  as well as `Proof command:` to cover the live numbered plans honestly.
- Observation: the existing gameplay CI lane had the same wasm-target
  assumption leak as the new chain lane.
  Evidence: `cargo clippy -p myosu-games -p myosu-tui -p myosu-games-poker -p myosu-play -- -D warnings`
  failed locally until the active-crates job also adopted `SKIP_WASM_BUILD=1`.
- Observation: the expanded workflow shape looks comfortably viable under a
  warm local cache.
  Evidence: measured local wall times on 2026-03-29 were approximately:
  doctrine `0.03s`, plan-quality `0.03s`, active-crates tests `47.97s`,
  active-crates clippy `0.61s`, chain-core `1.91s`, chain-clippy `1.88s`.
- Observation: the last `010` proof is blocked one step earlier than the plan
  originally implied.
  Evidence: on 2026-03-30, `gh workflow list --repo happybigmtn/myosu` returned
  no workflows and `gh run list --repo happybigmtn/myosu --limit 20 --json ...`
  returned `[]`, while local `git ls-files .github/workflows` returned no
  tracked workflow files.
- Observation: publishing the workflow did produce real hosted runs, but they
  validate the current remote repository state rather than the current local
  stage-0 workspace.
  Evidence: run `23729737957` completed on GitHub Actions and failed with
  remote-repo mismatches: doctrine saw unexpected legacy `031626-*` specs,
  plan-quality hit `genesis/plans/002-fabro-cleanup-completion.md`, the
  gameplay lane reported `myosu-games-poker` missing, and the chain lanes
  failed because `myosu-chain-runtime` is outside the remote workspace.
- Observation: the right immediate CI fix is not to weaken the jobs; it is to
  make the repo-shape blocker explicit and early.
  Evidence: the new `check_stage0_repo_shape.sh` preflight can prove the
  current stage-0 workspace contract using a handful of required files and
  workspace members before any cargo or doctrine job starts.
- Observation: the hosted workflow now fails in the right place.
  Evidence: run `23729944496` reported `stage-0 repo shape mismatch` and listed
  the missing current-stage files (`crates/myosu-games-poker/Cargo.toml`,
  `crates/myosu-play/Cargo.toml`, `crates/myosu-miner/Cargo.toml`,
  `crates/myosu-validator/Cargo.toml`, `crates/myosu-chain-client/Cargo.toml`,
  `crates/myosu-chain/runtime/Cargo.toml`, `crates/myosu-chain/node/Cargo.toml`,
  plus the current `002`, `010`, and `020` Genesis plans), while every
  dependent job skipped.
- Observation: once that current repo surface was published, the hosted
  workflow immediately moved past the remote-drift blocker.
  Evidence: on run `23730119476`, `Stage-0 Repo Shape` passed in `5s`,
  `Plan Quality` passed in `3s`, and `Doctrine Integrity` passed in `4s`,
  leaving only the longer `Active Crates`, `Chain Core`, and `Chain Clippy`
  lanes in progress.
- Observation: the current hosted proof still carries avoidable workflow-noise
  unrelated to Stage 0 correctness.
  Evidence: GitHub annotated run `23730119476` with Node 20 deprecation
  warnings for `actions/checkout@v4`, and `gh api
  repos/actions/checkout/releases/latest --jq '.tag_name'` returned `v6.0.2`.
- Observation: once the branch started carrying full current-stage CI surfaces,
  stale overlapping runs became a practical source of noise and wasted hosted
  minutes.
  Evidence: run `23730119476` had to be cancelled manually after the
  `checkout@v6` update created successor run `23730274703`.
- Observation: the cleaned-up workflow is preserving the same fast early proof
  while reducing hosted-noise.
  Evidence: run `23730306070` passed `Stage-0 Repo Shape` in `4s`,
  `Doctrine Integrity` in `3s`, and `Plan Quality` in `3s`, and the earlier
  Node 20 checkout deprecation warning did not appear in the observed watch
  output.
- Observation: the first full hosted run on the current repo surface exposed
  one real CI-environment gap and one real test-isolation gap.
  Evidence: on run `23730306070`, both chain jobs failed in `litep2p`'s build
  script with `Could not find protoc`, and `Active Crates` failed because
  `tests::startup_state_becomes_partial_when_discovery_returns_zero_results`
  plus `tests::startup_state_becomes_partial_when_live_query_fails` saw clean
  runner behavior (`AdviceSelection::startup_state == Empty`) instead of the
  ambient local-artifact behavior that had let them pass before.
- Observation: fixing the runner dependency surfaced one more honest local
  chain-CI issue before the hosted rerun.
  Evidence: once `protoc` was available locally, `cargo clippy -p myosu-chain
  --features fast-runtime -- -D warnings` reached `crates/myosu-chain/node`
  and failed on three `expect()` calls in the stage-0 smoke summary printer.
  Replacing those with explicit `sc_cli::Error::Input` returns kept the smoke
  contract strict while restoring deny-level Clippy green.

## Decision Log

- Decision: Chain CI starts as compile-only; full test suite is a follow-on after compile is stable.
  Rationale: Fast, reliable compile gate catches the most common regressions (broken imports, missing features). Full tests can be added incrementally.
  Inversion: Starting with full pallet tests creates 30-minute CI runs that developers bypass.
  Date/Author: 2026-03-28 / Genesis

- Decision: Include doctrine checks in CI.
  Rationale: Spec/doc drift is currently a top project risk. Automating catches it.
  Inversion: Code-only CI allows doc contradictions to reappear.
  Date/Author: 2026-03-28 / Genesis

## Failure Modes

| Codepath | Realistic failure | Handling |
|---|---|---|
| Chain compile job | Long compile causes timeout | Aggressive caching + split by crate family |
| Doctrine check | False positive from glob pattern drift | Maintain explicit required-file list, not glob |
| Plan quality check | New plan format doesn't match regex | Keep regex simple; check for `### Milestone` and `Proof command:` |

## Outcomes & Retrospective

Implementation is underway and the first real CI-hardening slice is now in.
The repo has moved from a single gameplay-only workflow to a broader gate set:
gameplay, chain compile, doctrine integrity, plan quality, and chain clippy.
The remaining open question is still operational rather than architectural:
confirm that the expanded workflow completes fast enough in GitHub Actions to
be worth keeping as a default merge gate. The difference now is that the repo
is past the vague "publish enough surface" phase. The current hosted run has
already proved the fast gates on the right repo shape and has already shown the
first real blocker set. The next `010` closure step is therefore: publish the
local fixes for hosted test isolation, runner `protoc`, and node deny-Clippy
cleanliness, then capture one full hosted green run and its timing envelope.

## Context and Orientation

Current CI at `.github/workflows/ci.yml`:
- Single job: "Active Crates" on ubuntu-latest
- Steps: checkout, rust toolchain (stable + clippy + rustfmt), cache, cargo check (4 crates), focused tests, full test suite, clippy, rustfmt
- No chain coverage

```text
CURRENT CI                              TARGET CI

+-------------------+                   +-------------------+
| gameplay-crates   |                   | gameplay-crates   |  (existing)
| - check           |                   | - check           |
| - test            |                   | - test            |
| - clippy          |                   | - clippy          |
| - rustfmt         |                   | - rustfmt         |
+-------------------+                   +-------------------+
                                        | chain-core        |  (NEW)
                                        | - runtime check   |
                                        | - pallet check    |
                                        | - node check      |
                                        +-------------------+
                                        | doctrine          |  (NEW)
                                        | - spec integrity  |
                                        | - plan quality    |
                                        +-------------------+
```

Owned files:
- `.github/workflows/ci.yml` (modify)
- `.github/workflows/chain.yml` (new, or merged into ci.yml)

## Milestones

### Milestone 1: Chain compile gate

Add a CI job that runs the honest stripped chain compile path for runtime, node,
and pallet-game-solver.

CI job definition:

    chain-core:
      name: Chain Core
      runs-on: ubuntu-latest
      steps:
        - uses: actions/checkout@v4
        - uses: dtolnay/rust-toolchain@stable
        - uses: Swatinem/rust-cache@v2
        - name: Runtime check
          run: SKIP_WASM_BUILD=1 cargo check -p myosu-chain-runtime --features fast-runtime
        - name: Pallet check
          run: SKIP_WASM_BUILD=1 cargo check -p pallet-game-solver
        - name: Node check
          run: SKIP_WASM_BUILD=1 cargo check -p myosu-chain --features fast-runtime

Proof command:

    grep -q "chain" .github/workflows/ci.yml || grep -q "chain" .github/workflows/chain.yml

### Milestone 2: Doctrine integrity check

Add a CI job that verifies the active canonical spec surface matches the master
index and that none of those canonicals are empty.

CI job definition:

    doctrine:
      name: Doctrine Integrity
      runs-on: ubuntu-latest
      steps:
        - uses: actions/checkout@v4
        - name: Canonical specs match master index and are non-empty
          run: bash .github/scripts/check_doctrine_integrity.sh

Proof command:

    grep -q "doctrine\|Doctrine" .github/workflows/ci.yml || grep -q "doctrine" .github/workflows/chain.yml

### Milestone 3: Genesis plan quality check

Verify each genesis plan has at least one milestone heading and at least one
proof-command label.

CI job definition:

    plan-quality:
      name: Plan Quality
      runs-on: ubuntu-latest
      steps:
        - uses: actions/checkout@v4
        - name: Plans have milestones and proofs
          run: bash .github/scripts/check_plan_quality.sh

Proof command:

    grep -q "plan-quality\|Plan Quality" .github/workflows/ci.yml || grep -q "plan" .github/workflows/chain.yml

### Milestone 4: Chain clippy (warning level)

Add clippy for the stripped runtime, live pallet, and node.

Proof command:

    grep -q "chain-clippy\\|Chain Clippy" .github/workflows/ci.yml

### Milestone 5: CI timing verification

Publish the local workflow to GitHub, run the hosted pipeline, and verify total
time < 15 minutes. If chain compile is slow, investigate incremental
compilation and caching.

Proof command:

    gh workflow list --repo happybigmtn/myosu
    gh run list --limit 1 --json durationMs

## Plan of Work

1. Add chain-core compile job.
2. Add doctrine integrity check.
3. Add plan quality check.
4. Add chain clippy (warning level).
5. Verify timing.

## Concrete Steps

From `/home/r/coding/myosu`:

    cat .github/workflows/ci.yml
    # Local validation before pushing:
    SKIP_WASM_BUILD=1 cargo check -p myosu-chain-runtime --features fast-runtime
    SKIP_WASM_BUILD=1 cargo check -p pallet-game-solver
    SKIP_WASM_BUILD=1 cargo check -p myosu-chain --features fast-runtime

## Validation and Acceptance

Accepted when:
- CI has separate jobs for: gameplay, chain-core, doctrine, plan-quality
- CI also runs chain clippy against the stripped runtime, live pallet, and node
- Chain compile failures block merge
- Empty canonical specs block merge
- Plans without milestones/proofs block merge
- The workflow is actually published on GitHub
- At least one hosted run is against the current stage-0 repo surface rather
  than the older remote workspace shape
- Total CI time < 15 minutes

## Idempotence and Recovery

CI config changes are deterministic. If a new gate is too flaky, add `continue-on-error: true` with a dated TODO.

## Interfaces and Dependencies

Depends on: 002 (spec normalization -- doctrine checks need clean specs), 003 (runtime reduction -- chain compile needs reduced runtime).
Blocks: 011 (security audit needs CI coverage).

```text
002 (clean specs)     003 (reduced runtime)
        \                    /
         v                  v
    .github/workflows/ci.yml
    +---gameplay (existing)---+
    +---chain-core (new)------+
    +---doctrine (new)--------+
    +---plan-quality (new)----+
```

# Specification: Malinka Platform Enhancements — Autonomous Build of Myosu Chain

Source: Myosu project requirements vs malinka current capabilities
Status: Draft
Date: 2026-03-16
Target: malinka engineering team (this spec lives in myosu but describes malinka work)

## Purpose

Enumerate the specific enhancements malinka needs to autonomously build, test,
supervise, and deploy the myosu game-solving chain. Myosu is a multi-crate
Rust/Substrate project with long-running processes, WASM compilation, integration
tests that spawn child processes, and cross-crate dependency ordering. These
requirements stress malinka's loop runner, proof gates, workspace management,
and engine configuration beyond what a single-crate Rust project demands.

The goal is: after these enhancements, malinka can pick up `ralph/IMPLEMENT.md`
and execute Stage 0 through Stage 6 (48 ACs across 10 specs) with minimal
human intervention.

## What Malinka Can Handle Today

These capabilities are confirmed working for myosu's use case:

1. **Plan parsing**: IMPLEMENT.md 6-field format is consumed by `loopcodex.sh`
   task extraction. The CF-/GS-/GT-/PE-/MN-/VO-/GP-/MG- prefixes are
   compatible with malinka's regex-based parser.

2. **Build prompt compilation**: `PROMPT_build.md` template substitution
   (SPEC_EXCERPT, TASK_BLOCK, COMPLETED_CONTEXT) works for myosu's specs.

3. **RESULT/BLOCKED structured output**: Adjudicator can parse the required
   terminal lines.

4. **Basic proof commands**: `cargo test`, `cargo clippy -- -D warnings` work
   as proof gates.

5. **Git operations**: Commit, push, branch management work.

6. **Recurring strategy/security brains**: Configuration in project.yaml is
   valid and will produce useful reports.

## What Needs Configuration (No Code Changes)

### NC-01: Engine timeout must be extended

**Problem**: myosu's `project.yaml` has `stall_timeout_ms: 300000` (5 min).
Substrate WASM compilation takes 5-20 minutes on first build. The stall
timeout will kill the engine mid-compilation.

**Fix**: Set `stall_timeout_ms: 1200000` (20 min) in myosu's `project.yaml`.
Also set `engine.timeout_secs: 7200` (2 hours) for tasks that involve full
workspace builds.

### NC-02: Fix `grep -c` exit code inversion in CF-02

**Problem**: CF-02's test command is `cargo tree -p myosu-runtime 2>&1 | grep -c "pallet.subtensor"` expecting count 0. But `grep -c` returns exit code 1 when count is 0. The loop's `verify_required_tests()` treats non-zero exit as FAIL. The desired success (no subtensor deps) is reported as failure.

**Fix**: Rewrite CF-02's test in IMPLEMENT.md to:
`! cargo tree -p myosu-runtime 2>&1 | grep -q 'pallet.subtensor'`

### NC-03: Proof command timeout configuration

**Problem**: `cargo build --release` for the Substrate runtime can take 10+
minutes. Proof commands may have default timeouts that are too short.

**Fix**: Add per-command timeout configuration to myosu's `project.yaml`:
```yaml
proof_commands:
  - command: cargo test
    timeout_ms: 300000
  - command: "cargo clippy -- -D warnings"
    timeout_ms: 300000
  - command: cargo build --release
    timeout_ms: 1200000
```

### NC-03: Workspace root vs crate-level scoping

**Problem**: myosu is a Cargo workspace with 8+ crates. `cargo test` at the
workspace root runs ALL tests. Tasks should scope to the relevant crate:
`cargo test -p pallet-game-solver`. The plan entries already specify scoped
commands, but malinka needs to respect them rather than running workspace-level
commands.

**Fix**: Ensure the `Tests:` field from IMPLEMENT.md entries is used as-is
for proof commands, not overridden by project.yaml's global proof_commands.

---

## What Needs Enhancement (Code Changes Required)

### EN-01: Per-command timeout in proof gates

- **What**: Allow proof_commands in project.yaml to specify individual timeouts
- **Why**: `cargo test` takes 30s but `cargo build --release` takes 20min.
  A single timeout for all proof commands either kills slow commands or waits
  too long for fast ones.
- **Current**: proof_commands is `Vec<String>`. No per-command timeout.
- **Proposed**: proof_commands becomes `Vec<ProofCommand>` where
  ```yaml
  proof_commands:
    - command: cargo test
      timeout_ms: 300000
    - command: cargo build --release
      timeout_ms: 1200000
  ```
  Backward compatible: bare strings use the default timeout.
- **Effort**: 2-4 hours (profile.rs + adjudicator.rs)
- **Priority**: P1 (blocks tasks that require WASM build as proof)

### EN-02: Task-level proof command override

- **What**: Allow individual IMPLEMENT.md entries to specify their own proof
  commands instead of always running the global list
- **Why**: GS-05 (Yuma port) needs `cargo test -p pallet-game-solver
  epoch::tests::yuma_matches_subtensor_output` as its primary proof. Running
  `cargo build --release` (20 min) after every pallet task is wasteful.
  The `Tests:` field already specifies the relevant command — it should be
  used as the proof gate, with global commands as a supplementary check.
- **Current**: All tasks run the same global proof_commands from project.yaml
- **Proposed**: Parse the `Tests:` field from the plan entry and use it as
  the primary proof command. Run global proof_commands only on landing (not
  per-iteration).
- **Effort**: 4-8 hours (plan.rs parser + adjudicator.rs)
- **Priority**: P1 (prevents 20-min proof gate on every 5-min task)

### EN-03: Long-running process management for integration tests

- **What**: Support starting/stopping child processes as part of proof execution
- **Why**: CF-05 and GS-09 require starting `myosu-node --dev` as a child
  process, waiting for it to be ready, running tests against it, then
  stopping it. This is a common pattern for blockchain integration tests.
- **Current**: Proof commands are simple shell commands. No lifecycle management.
- **Proposed**: Add a `services` block to project.yaml:
  ```yaml
  services:
    devnet:
      command: "cargo run -p myosu-node -- --dev --tmp"
      ready_check: "curl -sf http://localhost:9944/health"
      ready_timeout_ms: 30000
      stop_signal: SIGINT
  ```
  Tasks can declare `requires_services: [devnet]` to auto-start/stop.
  Alternatively, support `pre_command` / `post_command` hooks per task.
- **Effort**: 1-2 days (new service manager module)
- **Priority**: P1 (blocks CF-05, GS-09, all integration tests)

### EN-04: Multi-crate dependency ordering

- **What**: Understand Cargo workspace member ordering for build/test
- **Why**: myosu has cross-crate dependencies: `myosu-games` → `myosu-games-poker`
  → `myosu-miner`. Building in the wrong order wastes time. More importantly,
  if a task modifies `myosu-games` traits, downstream crates break — malinka
  should run downstream tests as a regression check.
- **Current**: Tasks are independent. No concept of crate dependency graph.
- **Proposed**: Add `affected_crates` to plan entry parsing. When a task
  modifies a crate, also run proof commands for dependent crates:
  ```yaml
  workspace:
    dependency_check: true  # run cargo test on dependents after changes
  ```
  Or simpler: when a task touches `crates/myosu-games/`, also run
  `cargo test -p myosu-games-poker` and `cargo test -p myosu-games-liars-dice`.
- **Effort**: 4-8 hours (workspace.rs + adjudicator.rs)
- **Priority**: P2 (nice to have — Cargo handles rebuild, but tests may miss regressions)

### EN-05: External repo reference during fork tasks

- **What**: Allow tasks to reference files from other repos (e.g., subtensor)
- **Why**: CF-01 (strip runtime) and GS-05 (port Yuma) need to read files from
  `/home/r/coding/subtensor/` as reference material. The build agent needs to
  know these repos exist and where they are.
- **Current**: Build agents only see the myosu repo.
- **Proposed**: Add a `references` block to project.yaml:
  ```yaml
  references:
    subtensor:
      path: ~/coding/subtensor
      purpose: "Substrate runtime fork source"
    robopoker:
      path: ~/coding/robopoker
      purpose: "MCCFR engine reference"
  ```
  Include these paths in the build prompt so agents can `Read` from them.
- **Effort**: 2-4 hours (profile.rs + prompt compilation)
- **Priority**: P1 (blocks CF-01, CF-02, GS-05 — the fork tasks that need
  to copy/adapt code from subtensor)

### EN-06: Substrate-specific proof patterns

- **What**: Understand Substrate WASM build output and chain spec validation
- **Why**: Several proof commands are Substrate-specific:
  - `cargo build --release` must produce `target/release/wbuild/myosu-runtime/`
  - `myosu-node build-spec --chain local --raw` must output valid JSON
  - `cargo tree -p myosu-runtime | grep -c pallet.subtensor` must return 0
  These are not standard `cargo test` commands. Malinka's adjudicator needs
  to handle non-zero-exit-code proof results (e.g., `grep -c` returns 1 when
  count is 0, which is a success for "no matches found").
- **Current**: Proof success = exit code 0.
- **Proposed**: Allow proof commands to specify expected exit code or output
  match:
  ```yaml
  proof_commands:
    - command: "cargo tree -p myosu-runtime | grep -c pallet_subtensor"
      expect_output: "0"
    - command: "myosu-node build-spec --chain local --raw | python3 -c 'import json,sys; json.load(sys.stdin)'"
      timeout_ms: 60000
  ```
- **Effort**: 4-8 hours (adjudicator.rs proof parsing)
- **Priority**: P2 (can work around with wrapper scripts, but native support is cleaner)

### EN-07: Parallel stage execution

- **What**: Run independent stages concurrently using git worktrees
- **Why**: Stages 2a (game engine traits) and 3 (game-solving pallet) are
  independent — they can be built in parallel. Currently malinka processes
  tasks sequentially from IMPLEMENT.md. With worktrees, it could run GT-01..05
  and GS-01..08 simultaneously.
- **Current**: Sequential task execution. Worktree support exists in loopcodex.sh
  but only for parallelizing within a stage, not across stages.
- **Proposed**: Parse the `Depends-on` field in specs to identify independent
  stages. Auto-parallelize stages with no dependency relationship using
  separate worktrees.
- **Effort**: 2-3 days (scheduler enhancement + worktree management)
- **Priority**: P2 (optimization — sequential works, just slower)

### EN-08: Substrate-specific build prompt enrichment

- **What**: The generic `PROMPT_build.md` has no Substrate domain context
- **Why**: Agents writing pallet code (GS-01..09) need FRAME macro patterns,
  `construct_runtime!` conventions, `TestExternalities` mock setup, and
  storage attribute syntax. Without this, agents make structural errors.
- **Current**: `PROMPT_build.md` is 37 lines of generic "implement + test + RESULT"
- **Proposed**: Support `RALPH_PROMPT_FILE` override per-repo. Myosu creates its
  own `loop/PROMPT_build.md` extending the generic template with a Substrate
  context block covering `#[pallet::*]` macros, storage types, mock runtime
  setup, and WASM build conventions.
- **Effort**: 4-8 hours (myosu creates custom prompt, malinka needs no change
  since `RALPH_PROMPT_FILE` already exists)
- **Priority**: P1 (blocks quality of pallet development tasks)

### EN-09: Branch naming convention (trunk, not main)

- **What**: Support configurable default branch name
- **Why**: myosu uses `trunk` as its default branch, not `main`. If malinka
  hardcodes `main` anywhere (push targets, merge base detection, PR creation),
  it will fail.
- **Current**: Unknown — needs audit of all branch name references.
- **Proposed**: Read default branch from `git remote show origin` or from
  project.yaml:
  ```yaml
  git:
    default_branch: trunk
  ```
- **Effort**: 1-2 hours (find/replace hardcoded "main" references)
- **Priority**: P0 (breaks everything if hardcoded)

---

## Manual Prerequisites (Before Malinka Starts)

These tasks must be completed by a human before malinka can begin autonomous
execution:

### MP-01: Fork robopoker and add serde + encoder changes (RF-01, RF-02)
- Clone `krukah/robopoker` to `happybigmtn/robopoker`
- Add `serde` feature flag to `rbp-nlhe`, `rbp-mccfr`, `rbp-gameplay`
- Add `NlheEncoder::from_map()` constructor
- Verify all existing tests pass
- **Why manual**: requires understanding robopoker's type system deeply;
  changes affect 5+ crates; must preserve existing API

### MP-02: Copy subtensor runtime into myosu workspace (CF-01, CF-02)
- Copy `/home/r/coding/subtensor/runtime/` → `crates/myosu-chain/runtime/`
- Copy `/home/r/coding/subtensor/node/` → `crates/myosu-chain/node/`
- Copy `/home/r/coding/subtensor/common/` → `crates/myosu-chain/common/`
- Strip AI/EVM pallets from construct_runtime!
- Get the chain to compile and produce blocks
- **Why manual**: 100+ files to copy, complex Cargo.toml surgery, deep
  Substrate knowledge required. This is the riskiest task in the entire project.
  Once it compiles, malinka can take over.

### MP-03: Generate Yuma Consensus test vectors
- Run subtensor's `epoch_mechanism()` on 5 synthetic inputs
- Capture all intermediate and final values as JSON fixtures
- Place in `crates/myosu-chain/pallets/game-solver/tests/fixtures/`
- **Why manual**: requires running subtensor's test infrastructure, which
  has its own complex setup

## Recommended Bootstrap Sequence

```
Phase 1: Human work (MP-01..03)
  ├─ Fork robopoker, add serde + encoder (1-2 days)
  ├─ Copy subtensor into myosu, strip to minimal chain (2-3 days)
  └─ Generate Yuma test vectors (0.5 day)

Phase 2: Malinka configuration (NC-01..03)
  ├─ Update project.yaml timeouts
  ├─ Configure proof command scoping
  └─ Verify loop runner works with myosu's plan format

Phase 3: Malinka enhancement (EN-01..08, parallel with Phase 1)
  ├─ P0: Branch naming (EN-08) — 1-2 hours
  ├─ P1: Per-command timeout (EN-01) — 2-4 hours
  ├─ P1: Task-level proof override (EN-02) — 4-8 hours
  ├─ P1: Service management (EN-03) — 1-2 days
  ├─ P1: External repo references (EN-05) — 2-4 hours
  ├─ P2: Multi-crate deps (EN-04) — 4-8 hours
  ├─ P2: Substrate proof patterns (EN-06) — 4-8 hours
  └─ P2: Parallel stages (EN-07) — 2-3 days

Phase 4: Malinka autonomous execution
  ├─ Stage 2a: Game engine traits (GT-01..05) — malinka drives
  ├─ Stage 2b: Poker engine (PE-01..04) — malinka drives
  ├─ Stage 3: Game-solving pallet (GS-01..10) — malinka drives
  ├─ Stage 4: Miner + validator (MN + VO) — malinka drives
  ├─ Stage 5: Gameplay CLI (GP-01..04) — malinka drives
  └─ Stage 6: Multi-game validation (MG-01..04) — malinka drives
```

At the Phase 4 boundary, malinka should be effective for all remaining work.
The human work in Phase 1 removes the highest-risk tasks (fork surgery) that
require deep Substrate/robopoker knowledge and multi-file operations that
are hard to spec as single-task ACs.

## Decision Log

- 2026-03-16: P0 for branch naming — trivial fix but blocks everything.
- 2026-03-16: P1 for service management — integration tests are the quality
  gate for the most critical specs (CF-05, GS-09).
- 2026-03-16: P2 for parallel stages — nice 2x speedup but sequential works.
- 2026-03-16: Manual subtensor fork (MP-02) — too risky for autonomous execution
  due to 100+ file copy and complex Cargo.toml surgery. Once chain compiles,
  malinka can handle individual pallet development.

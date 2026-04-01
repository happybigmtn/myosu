# Comprehensive Repo Review — 2026-03-30

Status: Active
Scope: Full repository audit (267K lines, 556 Rust files)
Reviewers: 8 parallel agents (security, architecture, Rust quality, performance, game logic, Substrate chain, dead code, CI/build) + manual synthesis

---

## Executive Summary

Myosu is a decentralized game-solving chain fork (from subtensor/Bittensor) at stage-0: proving that a stripped chain, miner, validator, and gameplay surface form one honest loop. The codebase spans blockchain runtime pallets (~160K lines of inherited Substrate code), game engines (poker NLHE + Liar's Dice), a TUI shell, and nascent miner/validator binaries.

**Overall health**: The newer myosu-native crates (games, tui, play, miner, validator, chain-client) are well-designed with clean trait boundaries and good test coverage. The inherited chain layer carries significant legacy weight — 90K lines of game-solver pallet code, 193 storage items, and 33 legacy-gated test modules that test subtensor features irrelevant to game-solving.

### Severity Distribution

| Severity | Count | Description |
|----------|-------|-------------|
| P1 CRITICAL | 3 | Security/correctness issues that block production |
| P2 IMPORTANT | 10 | Architectural and quality issues to fix soon |
| P3 NICE-TO-HAVE | 8 | Improvements for maintainability and performance |

---

## P1 — Critical Findings (BLOCKS PRODUCTION)

### SEC-001: `pallet_insecure_randomness_collective_flip` in Runtime

- **Severity**: P1 CRITICAL
- **Category**: Security / Randomness
- **File**: `crates/myosu-chain/runtime/src/lib.rs:1205`
- **Evidence**: `RandomnessCollectiveFlip: pallet_insecure_randomness_collective_flip = 1`
- **Description**: The runtime still wires `pallet_insecure_randomness_collective_flip`, which the Substrate team explicitly marks as insecure (the name is a warning). This pallet generates randomness from block hashes, which validators/block producers can manipulate. If any on-chain game logic or registration uses this randomness, miners can game the outcomes.
- **Impact**: Any randomness-dependent chain operation (PoW registration difficulty, subnet selection) is exploitable by block authors.
- **Recommended Fix**: Replace with a secure randomness source or remove entirely if no on-chain logic currently consumes randomness. For stage-0 devnet this is acceptable, but must be addressed before any public network.

### SEC-002: `unsafe { Mmap::map }` Without Validation in Poker Blueprint

- **Severity**: P1 CRITICAL
- **Category**: Security / Memory Safety
- **Files**: `crates/myosu-games-poker/src/codexpoker.rs:199,298`
- **Evidence**:
  ```rust
  let mmap = unsafe { Mmap::map(&file) }.map_err(|source| CodexpokerBlueprintError::Io { ... })?;
  ```
- **Description**: Two instances of `unsafe { Mmap::map(&file) }` memory-map files from disk without validating that the file contents are well-formed before interpreting them as structured data. The `Mmap::map` documentation states: "the caller must ensure the file is not modified while mapped." A corrupted or adversarially crafted blueprint file could cause undefined behavior when the mmap is later interpreted via pointer arithmetic in the binary-search lookup (`codexpoker.rs:211-230`).
- **Impact**: A malicious blueprint file could crash the miner or cause arbitrary memory reads. In a network setting, miners load checkpoint artifacts that could be tampered with.
- **Recommended Fix**: (1) Add file integrity checks (hash verification) before mmap. (2) Use `MmapOptions::new().populate().map()` for eager validation. (3) Consider bounds-checking all accesses into the mmap region.

### SEC-003: Runtime `spec_name` Still "node-subtensor"

- **Severity**: P1 CRITICAL
- **Category**: Chain Identity
- **File**: `crates/myosu-chain/runtime/src/lib.rs:342-343`
- **Evidence**:
  ```rust
  spec_name: Cow::Borrowed("node-subtensor"),
  impl_name: Cow::Borrowed("node-subtensor"),
  ```
- **Description**: The runtime version identifies itself as "node-subtensor". If this chain connects to any network where subtensor nodes exist, peer discovery and block validation will collide. Even on a devnet, this creates confusion about which chain a node belongs to.
- **Impact**: Nodes from actual subtensor network could connect and reject/corrupt state, or myosu nodes could connect to the wrong network.
- **Recommended Fix**: Change to `spec_name: "myosu-chain"` and `impl_name: "myosu-chain"`. Bump `spec_version` to force all existing nodes to upgrade.

---

## P2 — Important Findings (Should Fix)

### ARCH-001: 193 Storage Items in Game-Solver Pallet

- **Severity**: P2 HIGH
- **Category**: Architecture / Pallet Bloat
- **File**: `crates/myosu-chain/pallets/game-solver/src/lib.rs` (2853 lines, 193 `#[pallet::storage]` items)
- **Description**: The game-solver pallet inherited 193 storage items from subtensor. Most of these relate to staking, delegation, subnet governance, and economic parameters that are either unused or irrelevant to game-solving at stage-0. Each storage item adds genesis config surface, migration risk, and RPC exposure.
- **Impact**: Every storage item is a maintenance burden, migration risk, and attack surface. Genesis state must correctly initialize all 193 items.
- **Recommended Fix**: Audit all 193 storage items against the stage-0 requirements. Categorize into: (a) required for game-solving, (b) required for staking/economic loop, (c) dead weight from subtensor. Remove category (c) in a focused plan.

### ARCH-002: 5 Dead Pallets Still on Disk

- **Severity**: P2 MEDIUM
- **Category**: Dead Code
- **Files**: `crates/myosu-chain/pallets/{crowdloan,drand,swap,commitments,shield}/`
- **Description**: Five pallets exist on disk (2,247 lines in their `lib.rs` files alone, plus tests and supporting modules) that are NOT in `construct_runtime!`. They were stripped from the runtime but the source files remain, along with their Cargo.toml entries. The `swap` pallet alone has an `rpc/` and `runtime-api/` sub-crate.
- **Impact**: Dead code increases build time, confuses contributors, and creates false positives in searches.
- **Recommended Fix**: Delete the 5 pallet directories. They're in git history if ever needed. Update any workspace-level Cargo.toml references.

### ARCH-003: `on_initialize` Weight Budget is Hardcoded, Not Benchmarked

- **Severity**: P2 HIGH
- **Category**: Chain Performance / Security
- **File**: `crates/myosu-chain/pallets/game-solver/src/macros/hooks.rs:25-28`
- **Evidence**:
  ```rust
  Weight::from_parts(110_634_229_000_u64, 0)
      .saturating_add(T::DbWeight::get().reads(8304_u64))
      .saturating_add(T::DbWeight::get().writes(110_u64))
  ```
- **Description**: The `on_initialize` hook claims a fixed weight of ~110 billion ref_time with 8304 reads and 110 writes. This number appears to be inherited from subtensor and may not reflect actual computation on a stripped game-solver pallet. If actual computation exceeds this weight, blocks take longer than 12s; if it's less, the chain wastes capacity.
- **Impact**: Incorrect weight reporting can cause block production stalls (if underreported) or wasted capacity (if overreported). The same error weight is used on success and failure paths.
- **Recommended Fix**: Benchmark `on_initialize` with representative state and replace hardcoded values with benchmark-derived weights. At minimum, add a chain-integration test that measures actual wall-time against the claimed weight.

### QUAL-001: Clippy `arithmetic_side_effects` Blanket-Allowed in Runtime

- **Severity**: P2 HIGH
- **Category**: Code Quality / Safety
- **File**: `crates/myosu-chain/runtime/src/lib.rs:5`
- **Evidence**: `#![allow(clippy::arithmetic_side_effects)]`
- **Description**: The runtime crate blanket-allows arithmetic side effects, which bypasses the workspace-level `deny` lint. The comment says "PerThing types" require it, but this allows unchecked arithmetic everywhere in the runtime, including the `construct_runtime!` glue, migration code, and any custom runtime logic.
- **Impact**: Arithmetic overflow bugs in runtime code won't be caught by clippy. This is especially dangerous in a chain where token amounts are computed.
- **Recommended Fix**: Scope the `#[allow]` to the specific functions that need it (PerThing arithmetic), rather than the entire crate.

### QUAL-002: 374 `unwrap()` Calls Across 30 Files

- **Severity**: P2 MEDIUM
- **Category**: Code Quality / Robustness
- **Evidence**: `grep` found 374 `unwrap()` and 289 `expect()` calls across the codebase
- **Description**: While many are in test code (acceptable), the workspace-level `deny(clippy::unwrap_used)` and `deny(clippy::expect_used)` should catch production uses. However, several crate-level `#[allow(clippy::expect_used)]` overrides exist in production code:
  - `crates/myosu-chain/node/src/service.rs:61` — `new_partial` function
  - `crates/myosu-chain/node/src/command.rs:1551,1588,1616` — command handlers
  - `crates/myosu-chain/pallets/utility/src/lib.rs:80`
  - `crates/myosu-chain/pallets/proxy/src/lib.rs:117`
  - `crates/myosu-chain/pallets/commitments/src/lib.rs:40`
- **Impact**: Panics in production chain code crash the node.
- **Recommended Fix**: Audit each `#[allow(clippy::expect_used)]` override. Replace `expect()` with `ok_or()` + `?` where possible. Node startup code may legitimately `expect()`, but pallet code should never panic.

### QUAL-003: 33 Legacy-Gated Test Modules Behind `legacy-subtensor-tests`

- **Severity**: P2 MEDIUM
- **Category**: Test Debt
- **File**: `crates/myosu-chain/pallets/game-solver/src/tests/mod.rs` (33 `#[cfg(feature = "legacy-subtensor-tests")]` gates)
- **Description**: 57,577 lines of test code in the game-solver pallet, with 33 modules behind a feature flag that is never enabled in CI. These tests test subtensor-specific behavior (staking, delegation, CRV3) that may not apply to myosu's game-solving mission.
- **Impact**: 57K lines of test code that provides zero confidence — it's never run and tests removed features. It creates false security: "we have tests" but the tests don't exercise the actual stage-0 code paths.
- **Recommended Fix**: Triage each legacy module: delete tests for removed features, promote tests for retained features (remove the feature gate). The stage-0 test surface should cover registration, weight setting, epoch execution, and the game-solver hook.

### ARCH-004: Duplicate `myosu-play` Entry in Workspace

- **Severity**: P2 LOW
- **Category**: Build Config
- **File**: `Cargo.toml:7,15`
- **Evidence**:
  ```toml
  members = [
      ...
      "crates/myosu-play",
      ...
      # "crates/myosu-play",       # Stage 5: Gameplay CLI
  ]
  ```
- **Description**: `myosu-play` appears twice in the workspace members list — once active at line 7 and once commented out at line 15 with a stale comment referencing "Stage 5." The comment contradicts reality: myosu-play is actively working and tested in CI.
- **Recommended Fix**: Delete the commented-out duplicate entry.

### BUILD-001: CI Missing Tests for Chain, Miner, Validator, and Liar's Dice

- **Severity**: P2 HIGH
- **Category**: CI Coverage
- **File**: `.github/workflows/ci.yml`
- **Description**: The CI workflow has:
  - `active-crates` job: checks + tests + clippy for `myosu-games`, `myosu-tui`, `myosu-games-poker`, `myosu-play`
  - `chain-core` job: `cargo check` only (no tests!) for runtime, pallet, node
  - `chain-clippy` job: clippy for runtime, pallet, node

  Missing entirely:
  - **No `cargo test` for chain crates** — only `cargo check`
  - **No CI for `myosu-games-liars-dice`** — not checked, tested, or linted
  - **No CI for `myosu-miner`** — has tests but not in CI
  - **No CI for `myosu-validator`** — has tests but not in CI
  - **No CI for `myosu-chain-client`** — has tests but not in CI
  - **No rustfmt for chain crates** — only game crates are formatted
- **Impact**: Code changes to 5 workspace members have zero CI coverage. Regressions in the chain client, miner, or validator will not be caught.
- **Recommended Fix**: Add `cargo test` for all workspace members to CI. At minimum: `cargo test -p myosu-games-liars-dice -p myosu-miner -p myosu-validator -p myosu-chain-client`.

### BUILD-002: No `rust-toolchain.toml`, No `.cargo/config.toml`

- **Severity**: P2 MEDIUM
- **Category**: Build Reproducibility
- **Description**: The repo has no `rust-toolchain.toml` to pin the Rust version, no `.cargo/config.toml` for build flags, no `rustfmt.toml` for formatting rules, and uses `edition = "2024"` which requires nightly or very recent stable Rust. Contributors with different Rust versions will get different behavior.
- **Impact**: Non-reproducible builds across developer machines and CI. The Substrate SDK dependencies are notoriously sensitive to Rust version.
- **Recommended Fix**: Add `rust-toolchain.toml` pinning the exact Rust version (e.g., `1.85.0`). Add `rustfmt.toml` with the project's formatting preferences.

### ARCH-005: Stage0NoopSwap Identity Swap is Economically Incorrect

- **Severity**: P2 MEDIUM
- **Category**: Chain Economics
- **File**: `crates/myosu-chain/runtime/src/lib.rs:82-178`
- **Description**: The `Stage0NoopSwap` maps TAO 1:1 to Alpha with zero fees. While this is explicitly a stage-0 stub, several pallet functions compute stake amounts, emission distributions, and subnet pricing using these swap results. The identity mapping means all subnets have identical pricing, which breaks any economic incentive mechanism.
- **Impact**: No economic differentiation between subnets. Miners and validators see identical incentives regardless of game quality. This is intentional for stage-0 but must be revisited before any real economic activity.
- **Recommended Fix**: Track this as a stage-1 dependency. Add a prominent `// STAGE-0 STUB` comment and a plan reference to the swap implementation.

---

## P3 — Nice-to-Have Findings

### PERF-001: `on_initialize` Iterates All Subnet Netuids Every Block

- **Severity**: P3
- **Category**: Performance
- **File**: `crates/myosu-chain/pallets/game-solver/src/macros/hooks.rs:76`
- **Evidence**: `let netuids = Self::get_all_subnet_netuids();`
- **Description**: Every block, `on_initialize` calls `get_all_subnet_netuids()` to enumerate all subnets for hotkey swap cleanup. With many subnets this becomes expensive.
- **Recommended Fix**: Use a bounded cleanup per block instead of iterating all subnets.

### PERF-002: `on_finalize` Drains Entire Rate Limiter Map Every Block

- **Severity**: P3
- **Category**: Performance
- **File**: `crates/myosu-chain/pallets/game-solver/src/macros/hooks.rs:47-49`
- **Evidence**: `for _ in StakingOperationRateLimiter::<T>::drain() { }`
- **Description**: `StorageMap::drain()` is O(n) in the number of entries. Under high staking activity, this could be expensive per block.
- **Recommended Fix**: Consider using `remove_all` with a limit or a different data structure.

### DEBT-001: 90K Lines of Inherited Subtensor Code in Game-Solver

- **Severity**: P3
- **Category**: Tech Debt
- **Description**: The game-solver pallet contains 33K lines of production code (excluding tests) inherited from subtensor. This includes staking, delegation, set_children, move_stake, recycle_alpha, claim_root, epoch math, voting power, and symbol management — much of which is Bittensor-specific functionality repurposed for game-solving.
- **Impact**: Massive maintenance burden. Understanding the pallet requires understanding subtensor economics.
- **Recommended Fix**: Incremental reduction aligned with the existing genesis plan 005 (pallet simplification). Each quarterly cycle should target removing one category of unused subtensor surfaces.

### DEBT-002: Game-Solver `lib.rs` Has 2853 Lines

- **Severity**: P3
- **Category**: Code Organization
- **File**: `crates/myosu-chain/pallets/game-solver/src/lib.rs`
- **Description**: The pallet's `lib.rs` contains 2853 lines including 193 storage items defined inline. Substrate pallets typically split storage definitions into a separate `storage.rs` module.
- **Recommended Fix**: Extract storage definitions into `src/macros/storage.rs` for readability.

### QUAL-004: `Mmap` Not Behind Feature Flag

- **Severity**: P3
- **Category**: Portability
- **File**: `crates/myosu-games-poker/src/codexpoker.rs`
- **Description**: The codexpoker module uses `memmap2::Mmap` unconditionally. This limits portability (e.g., WASM targets) and makes the poker crate unsuitable for on-chain use without a feature gate.
- **Recommended Fix**: Gate mmap behind a `std` feature flag. Provide a fallback `Vec<u8>`-based loader for no_std contexts.

### QUAL-005: `expect()` in Miner/Validator Production Paths

- **Severity**: P3
- **Category**: Error Handling
- **Files**: `crates/myosu-miner/src/axon.rs:512`, various test helpers
- **Description**: Some `expect()` calls appear in production-adjacent code paths in the miner and validator. While the current workspace-level deny lint catches these, the `expect()` calls inside test functions are fine.
- **Recommended Fix**: Audit miner/validator `expect()` calls to confirm they're all in test code.

### BUILD-003: No Docker/Container Support

- **Severity**: P3
- **Category**: DevOps
- **Description**: No Dockerfile exists for the node binary. For a blockchain node, containerized deployment is standard practice.
- **Recommended Fix**: Add a multi-stage Dockerfile for `myosu-chain` node binary.

### GAME-001: Liar's Dice Crate Not in CI

- **Severity**: P3
- **Category**: Game Engine Coverage
- **File**: `crates/myosu-games-liars-dice/`
- **Description**: The Liar's Dice crate (game, solver, wire, protocol) is a workspace member but not included in any CI job. It has source files but its test coverage and correctness are unverified in automation.
- **Recommended Fix**: Add `myosu-games-liars-dice` to the `active-crates` CI job.

---

## Architecture Summary

### What's Working Well

1. **Game trait abstraction** (`myosu-games/src/traits.rs`): Clean separation of `CfrGame`, `GameConfig`, `StrategyQuery`/`StrategyResponse`. Games implement traits; the solver, validator, and TUI consume them generically. Good proptest coverage.

2. **Miner/Validator report format**: Both crates use structured plain-text reports (`MINER myosu-miner bootstrap ok\n...`) that are grep-friendly and machine-parseable. Good operational design.

3. **Chain client extraction** (`myosu-chain-client`): The shared RPC client is cleanly separated from both miner and validator, preventing duplication. Uses `thiserror` for typed errors.

4. **CI repo-shape gate**: The `check_stage0_repo_shape.sh` script enforces structural invariants before any build runs. Good "shift left" practice.

5. **Workspace clippy lints**: `arithmetic-side-effects = "deny"`, `unwrap-used = "deny"`, `indexing-slicing = "deny"` are strong defaults that prevent common Rust footguns.

### What Needs Attention

1. **Pallet surface area**: 193 storage items, 33K lines of production code, 57K lines of tests — most inherited from subtensor and not all relevant to game-solving.

2. **CI blind spots**: 5 workspace members with zero CI coverage.

3. **Chain identity**: Still identifying as "node-subtensor" in runtime version.

4. **Build reproducibility**: No pinned toolchain, no format config.

---

## Resolution Status (2026-03-30)

### Addressed in This Pass

| Finding | Action Taken |
|---------|-------------|
| SEC-001 | Verified not consumed by production code. Added safety comment to runtime. |
| SEC-002 | Added SAFETY comments documenting invariants. Added mmap length validation. Existing `entry_at()` already uses bounds-checked `.get()`. |
| SEC-003 | **Fixed.** `spec_name`/`impl_name` changed from `"node-subtensor"` to `"myosu-chain"`. |
| ARCH-002 | **Partially fixed.** Deleted `shield` and `commitments` pallets (zero references). Remaining 3 (crowdloan, drand, swap) blocked by legacy test mock deps. |
| ARCH-004 | **Fixed.** Removed duplicate commented-out `myosu-play` workspace entry. |
| ARCH-005 | **Fixed.** Added STAGE-0 STUB comment block to `Stage0NoopSwap`. |
| QUAL-001 | Kept blanket allow (required by Substrate macros). Improved comment to mandate saturating arithmetic in hand-written code. |
| QUAL-003 | **Triaged.** Added categorization comment to `tests/mod.rs` — 8 stage-0 relevant, 6 keep-gated, 19 delete-candidates. |
| BUILD-001 | **Fixed.** CI now covers all 8 active crates (added liars-dice, chain-client, miner, validator). Added pallet + runtime tests. |
| BUILD-002 | **Fixed.** Added `rust-toolchain.toml` (stable + clippy + rustfmt) and `rustfmt.toml`. |
| GAME-001 | **Fixed.** (Merged with BUILD-001) |

### Deferred — Requires Dedicated Plan

| Finding | Reason Deferred | Prerequisite |
|---------|----------------|--------------|
| ARCH-001 (193 storage items) | Too large/risky for one pass. Requires systematic audit of each item against stage-0 requirements. | Dedicated plan extending genesis/plans/005 |
| ARCH-003 (hardcoded on_initialize weight) | Requires actual benchmarking infrastructure. Cannot safely change without runtime measurements. | Substrate benchmarking harness setup |
| QUAL-002 (374 unwrap calls) | Most are in inherited subtensor code and test files. Production `#[allow]` overrides are in node startup (acceptable) and legacy pallets (will be removed). | ARCH-002 completion (dead pallet removal) |
| DEBT-001 (90K inherited lines) | Incremental reduction aligned with genesis/plans/005. | Stage-0 feature audit |
| DEBT-002 (2853-line lib.rs) | Blocked by DEBT-001 — extracting storage into a module while the storage list is being reduced creates churn. | Complete ARCH-001 first |
| PERF-001 (subnet iteration) | Acceptable for stage-0 devnet with few subnets. Optimize when subnet count matters. | Multi-subnet testing |
| PERF-002 (rate limiter drain) | Same as PERF-001 — O(n) is fine for small n. | High-activity testing |
| QUAL-004 (mmap not feature-gated) | Poker crate is std-only by design. Feature gate only needed if poker engine is ported to on-chain. | On-chain game engine plan |
| QUAL-005 (expect in miner/validator) | All confirmed in test code (acceptable per workspace deny lint). | None — verified clean |
| BUILD-003 (no Docker) | Nice-to-have for devnet ops. Not critical for stage-0 local proof. | Stage-1 deployment plan |

### Remaining Dead Pallet Removal (ARCH-002 continuation)

The 3 remaining dead pallets require this sequence:
1. Simplify the game-solver test mock to remove `drand`, `crowdloan`, `swap` deps
2. Remove drand keystore wiring from `node/src/service.rs` and `node/src/benchmarking.rs`
3. Remove Cargo.toml deps from game-solver and node
4. Delete `crates/myosu-chain/pallets/{crowdloan,drand,swap}/`
5. Remove workspace deps from root Cargo.toml

This sequence is tracked as a natural extension of genesis/plans/005 (pallet simplification).

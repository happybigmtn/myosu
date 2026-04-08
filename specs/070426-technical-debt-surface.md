# Specification: Technical Debt & Cleanup Surface

## Objective

Catalog the identified technical debt in the myosu codebase: the dead `pallet-subtensor` crate, the `pallet_subtensor` naming alias, inherited subtensor-era migrations in the active pallet, fabro/raspberry ghost infrastructure, and stale documentation references. This spec aggregates the targets of plans 002-005 and 013 into a single verification surface.

## Evidence Status

### Verified (code-grounded)

#### Dead pallet-subtensor crate (Plan 002 target)

- Location: `crates/myosu-chain/pallets/subtensor/`.
- Size: `lib.rs` is 2,750 lines / 92.4K.
- The dead pallet is declared in the workspace `Cargo.toml:107` as `pallet-subtensor = { path = "crates/myosu-chain/pallets/subtensor" }`.
- The runtime (`crates/myosu-chain/runtime/Cargo.toml:122`) does NOT use this crate. Instead it maps: `pallet_subtensor = { package = "pallet-game-solver", path = "../pallets/game-solver" }`.
- The runtime's `subtensor-custom-rpc-runtime-api` dependency (`runtime/Cargo.toml:154`) still points to `../pallets/subtensor/runtime-api`, creating a coupling to the dead crate's directory structure.
- The dead crate also has `benchmarks.rs` (60.9K), sub-modules for epoch, staking, subnets, swap, tests, etc.

#### Naming alias (Plan 003 target)

- The active pallet is `pallet-game-solver` at `crates/myosu-chain/pallets/game-solver/`.
- Its `lib.rs` is 2,868 lines / 96.7K.
- The runtime imports it as `pallet_subtensor` via Cargo alias (`runtime/Cargo.toml:122`).
- All runtime code (`lib.rs`) uses `pallet_subtensor::` and `SubtensorModule` as the storage prefix.
- Feature flags reference `pallet_subtensor/std`, `pallet_subtensor/runtime-benchmarks`, etc. throughout `runtime/Cargo.toml`.
- This naming trap means new contributors see `SubtensorModule` and `pallet_subtensor` everywhere but the actual code is in `pallets/game-solver/`.

#### Inherited migrations (Plan 004 target)

- Location: `crates/myosu-chain/pallets/game-solver/src/migrations/`.
- Count: 50+ migration files observed (partial listing includes files for features myosu never had: `migrate_delete_subnet_21.rs`, `migrate_fix_root_subnet_tao.rs`, `migrate_rao.rs`, `migrate_coldkey_swap_scheduled.rs`, `migrate_commit_reveal_v2.rs`, etc.).
- These migrations reference subtensor-era chain state (root network, dual-token Alpha/TAO, EVM features, commit-reveal V2/V3, CRV3 timelocking) that myosu has never had.
- Only `migrate_init_total_issuance` is wired into the runtime's `Migrations` type (`runtime/src/lib.rs:1303-1309`).
- The remaining migration files compile as part of the pallet but are not executed at runtime.

#### Fabro/Raspberry ghost infrastructure (Plan 013 target)

- README.md (lines 60-68) references:
  - `fabro run fabro/run-configs/bootstrap/game-traits.toml`
  - `fabro run fabro/run-configs/bootstrap/tui-shell.toml`
  - `fabro run fabro/run-configs/bootstrap/chain-runtime-restart.toml`
  - `fabro run fabro/run-configs/bootstrap/chain-pallet-restart.toml`
  - `raspberry plan/status/execute --manifest fabro/programs/myosu-bootstrap.yaml`
- README.md (line 33) links to `fabro/programs/myosu-bootstrap.yaml`.
- The `fabro/` directory exists at the repo root but the planning corpus (plan 013) identifies that `fabro/workflows/`, `fabro/run-configs/`, `fabro/programs/`, and `.raspberry/` either do not exist or do not contain the referenced files.
- Additional ghost references exist in `AGENTS.md` and `OS.md` per the planning corpus.

#### Stale documentation (Plan 005 target)

- Planning corpus identifies stale references including THEORY.MD, AGENTS.md fabro references, and broken `fabro run` commands.
- The README's "Current Operator Loop" section (lines 58-69) contains commands that reference ghost infrastructure.

### Recommendations (intended future direction)

- Plan 002: Delete the dead `pallet-subtensor` crate entirely. Move any still-needed RPC runtime-api code into the game-solver pallet's own directory.
- Plan 003: Rename the Cargo alias from `pallet_subtensor` to `pallet_game_solver` (or similar) in `runtime/Cargo.toml` and update all references in `lib.rs`.
- Plan 004: Delete all migration files in `pallets/game-solver/src/migrations/` except `migrate_init_total_issuance.rs` and the `mod.rs` that wires it.
- Plan 005: Remove stale documentation references, update or delete THEORY.MD and AGENTS.md fabro sections.
- Plan 013: Either build minimal fabro/raspberry directory structure with working entrypoint, or remove all references from README, AGENTS.md, and OS.md.
- All of these are Phase 1 (reduce and clean) work with zero external dependencies.

### Hypotheses / Unresolved

- Whether `subtensor-custom-rpc-runtime-api` at `pallets/subtensor/runtime-api/` can be moved to `pallets/game-solver/runtime-api/` without breaking RPC clients.
- Whether the dead pallet's test suite (`pallets/subtensor/src/tests/`) contains any tests that should be preserved (migrated to game-solver).
- Whether the `pallet_subtensor` storage prefix change (plan 003) requires a storage migration on any existing devnet state.
- Whether the unused migration files contribute meaningful compile time.

## Acceptance Criteria

- After plan 002: `crates/myosu-chain/pallets/subtensor/` directory no longer exists
- After plan 002: `cargo check -p myosu-chain-runtime` still succeeds
- After plan 003: No occurrence of the string `pallet_subtensor` or `pallet-subtensor` in `runtime/Cargo.toml` (except as a historical comment if needed)
- After plan 003: `construct_runtime!` uses the game-solver pallet's native name
- After plan 004: The `migrations/` directory in the game-solver pallet contains only the files actively wired into the runtime `Migrations` type
- After plan 005: No references to `THEORY.MD` remain in tracked files
- After plan 013: All `fabro run` and `raspberry` commands in README either work or are removed
- After all Phase 1 plans: All CI jobs pass, all proof commands pass, E2E smoke test passes
- Workspace `cargo check` succeeds after each individual plan completion (incremental safety)

## Verification

```bash
# Current state: dead pallet exists
test -d crates/myosu-chain/pallets/subtensor && echo "dead pallet exists"

# Current state: naming alias present
grep 'pallet_subtensor.*pallet-game-solver' crates/myosu-chain/runtime/Cargo.toml

# Current state: inherited migrations present
ls crates/myosu-chain/pallets/game-solver/src/migrations/ | wc -l

# Current state: only migrate_init_total_issuance is wired
grep 'migrate_init_total_issuance' crates/myosu-chain/runtime/src/lib.rs

# Current state: ghost infrastructure references in README
grep -c 'fabro run' README.md

# After cleanup: all proof commands still pass
cargo test -p pallet-game-solver stage_0_flow --quiet
SKIP_WASM_BUILD=1 cargo test -p myosu-chain --test stage0_local_loop --quiet
SKIP_WASM_BUILD=1 cargo run -p myosu-play --quiet -- --smoke-test
cargo test -p myosu-games-liars-dice --quiet
SKIP_WASM_BUILD=1 cargo test -p myosu-miner -p myosu-validator --quiet
bash .github/scripts/check_stage0_repo_shape.sh
```

## Open Questions

- What is the total compilation time impact of the dead pallet and unused migrations?
- Does renaming `SubtensorModule` to `GameSolverModule` in `construct_runtime!` change the storage prefix, and if so, does that break existing devnet state?
- Are there any external consumers (scripts, tools, monitoring) that depend on the `SubtensorModule` storage key prefix?
- Should the RPC runtime-api at `pallets/subtensor/runtime-api/` be moved, copied, or rewritten when the dead pallet is deleted?

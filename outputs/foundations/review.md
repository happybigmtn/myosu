# Foundations Lane Review

**Lane**: `foundations`
**Date**: 2026-03-20

---

## Keep / Reopen / Reset Judgment

**Judgment: KEEP (lane active, work in progress)**

The foundations lane is the honest bootstrap slice for the Fabro/Raspberry execution
frontier. It has correctly identified the two critical gaps that must be resolved before
the bootstrap program can be considered trustworthy:

1. **Raspberry state is uninitialized** — `.raspberry/` is absent; there is no
   resumption point and no observable execution truth
2. **`games:multi-game` false-submit** — a lane has claimed success without earning
   it; the nature and ownership of this defect must be classified

This review documents the honest state assessment and the go/no-go signal for the
rest of the bootstrap program.

---

## Bootstrap Lane Status Summary

### `games:traits` — KEEP

**Lane**: `games:traits`
**Output**: `outputs/games/traits/`
**Status**: Trusted kept leaf crate

All 10 unit tests and 4 doctests pass. The crate is buildable and the surface is
small. Two portability blockers exist (absolute robopoker paths, edition 2024) but
the implementation lane is unblocked for Slice 1.

**Bootstrap verdict**: TRU -- this lane earned its bootstrap reviewed milestone.

---

### `tui:shell` — KEEP (partial) / REOPEN (partial)

**Lane**: `tui:shell`
**Output**: `outputs/tui/shell/`
**Status**: Mixed — 4 modules KEEP, 3 modules REOPEN

| Module | Judgment | Rationale |
|--------|----------|-----------|
| screens.rs | **KEEP** | 18 tests, all transitions proven |
| input.rs | **KEEP** | 20+ tests, all handlers proven |
| renderer.rs | **KEEP** | Trait object-safe, mock coverage adequate |
| theme.rs | **KEEP** | 7 tests, 8 color tokens proven distinct |
| pipe.rs | **KEEP (with caveats)** | ANSI detection proven; no property test |
| schema.rs | **REOPEN** | 20-game claim, only 3 proven; high severity |
| events.rs | **REOPEN** | 2 tests ignored due to TTY; no headless alternative; high severity |
| shell.rs | **REOPEN** | No integration test for screen transitions; high severity |

**Bootstrap verdict**: PARTIAL — four modules earned KEEP; three modules need
implementation work before the lane can be declared fully bootstrapped.

---

### `chain:runtime` — RESET

**Lane**: `chain:runtime`
**Output**: `outputs/chain/runtime/`
**Status**: Restart from Phase 0 required

The runtime is not buildable. The `construct_runtime!` block imports 15+ crates
that don't exist in the workspace. The node directory is a scaffold with no
implementation files. The check script is a no-op that proves nothing.

**Salvageable**: `common/src/evm_context.rs`, `Currency`/`AlphaCurrency`/`TaoCurrency`
domain types, `pallets/game-solver/` as template for polkadot-sdk dependency line,
`NetUid` as a local domain type.

**Not salvageable**: Everything that imports a `subtensor_*` workspace key, the
entire `construct_runtime!` block, the node module declarations.

**Bootstrap verdict**: RESET — the chain restart lanes' reviews are authoritative.
The next implementation slice begins at Phase 0 (workspace wiring).

---

### `chain:pallet` — RESET

**Lane**: `chain:pallet`
**Output**: `outputs/chain/pallet/`
**Status**: Restart from Phase 1 required

`cargo check -p pallet-game-solver` fails with 50+ errors. The pallet is a
subtensor fork with subtensor workspace-key dependencies embedded at every level.
The `extensions/subtensor.rs` uses polkadot-sdk APIs removed in stable2407.

**Salvageable**: `stubs.rs`, `swap_stub.rs`, `benchmarks.rs`, pallet module
structure, `AxonInfo`/`PrometheusInfo`/`NeuronCertificate` data types,
`RateLimitKey` enum, `polkadot-sdk stable2407` dependency line.

**Not salvageable**: Everything that imports a `subtensor_*` workspace key, all 36
migration files, all RPC info files, all swap files, all coinbase files, the
`safe_math` dependency in `epoch/math.rs`, the `extensions/subtensor.rs` file.

**Bootstrap verdict**: RESET — restart from Phase 1 (fix deps + strip non-Myosu
modules). The review is authoritative and should not be re-litigated.

---

## Critical Findings

### Finding 1: `.raspberry/` Is Absent — No Execution Truth Exists

**Severity**: CRITICAL

The Raspberry runtime state directory `.raspberry/` does not exist in this worktree.
This means:

- `raspberry status --manifest fabro/programs/myosu-bootstrap.yaml` returns empty
- The bootstrap program has no resumption point — every run starts from scratch
- `execute/status/watch` truth is empty — there is nothing to trust
- The `managed_milestone: coordinated` tracking has never been exercised

**Required action**: Run `raspberry plan --manifest fabro/programs/myosu-bootstrap.yaml`
to initialize state, then capture the initial state snapshot as a baseline.

**Verification**:
```bash
raspberry plan --manifest fabro/programs/myosu-bootstrap.yaml
raspberry status --manifest fabro/programs/myosu-bootstrap.yaml
ls -la .raspberry/
```

---

### Finding 2: Chain Restart Check Scripts Are No-Ops

**Severity**: HIGH

`fabro/checks/chain-runtime-reset.sh` only checks file existence:
```bash
test -f crates/myosu-chain/pallets/game-solver/src/lib.rs
test -d crates/myosu-chain/node/src
```

It does NOT run `cargo check` or `cargo build`. This means the "proof" for
chain restart lanes is a file-existence check, not a build check.

Similarly, `fabro/checks/chain-pallet-reset.sh` only checks file existence.

**Required action**: The check scripts must be upgraded to run actual build commands
before the chain restart lanes can be treated as proven.

**Verification**:
```bash
cargo check -p pallet-game-solver  # Currently fails with 50+ errors
cargo check -p myosu-runtime      # Currently fails — workspace member commented out
```

---

### Finding 3: `games:multi-game` False-Submit Unassessed

**Severity**: HIGH (ownership TBD)

The `outputs/games/multi-game/` directory exists with `spec.md` and `review.md`.
The nature of the false-submit has not been classified in this lane. The two
possibilities are:

1. **Fabro/Raspberry defect**: The execution system submitted a false success
   (execution claimed success without any real work done)
2. **Lane-level defect**: The lane ran but the work was inadequate or incorrect

**Required action**: This lane must read `outputs/games/multi-game/review.md` and
determine which defect class applies. If it is a Fabro/Raspberry defect, it is
a blocker for the entire bootstrap program. If it is a lane-level defect, it is
specific to the `games:multi-game` lane.

---

### Finding 4: Workflow Graphs Not Audited

**Severity**: MEDIUM

The `fabro/workflows/bootstrap/*.fabro` workflow graphs were not read in this lane.
These graphs define the step ordering and execution flow for each bootstrap lane.
They represent an unaudited surface — if a workflow graph points to the wrong run
config or defines incorrect step dependencies, the bootstrap program will execute
incorrectly without any error.

**Required action**: Read all `fabro/workflows/bootstrap/*.fabro` files and confirm:
1. Each graph points to the correct run config
2. Each graph's step dependencies are correct
3. Each graph's check references are valid

---

## Go / No-Go Signal

### Go Conditions (all must be true to proceed)

| Condition | Status | Evidence |
|-----------|--------|----------|
| Raspberry state initialized | **NOT YET** | `.raspberry/` absent |
| Bootstrap lane states known | **YES** | 4 lanes assessed (2 KEEP, 2 RESET) |
| Check scripts reflect actual proof | **NO** | chain checks are no-ops |
| `games:multi-game` false-submit classified | **NOT YET** | Not yet assessed |
| Workflow graphs audited | **NOT YET** | Not yet read |
| Chain restart lane phases defined | **YES** | Phase 0 and Phase 1 defined in reviews |

**Current signal: NO-GO** — the `games:multi-game` false-submit must be classified
and the workflow graphs must be audited before the bootstrap program can be
declared trustworthy.

**Path to Go**:
1. Initialize `.raspberry/` state
2. Audit workflow graphs
3. Classify `games:multi-game` false-submit
4. Upgrade chain check scripts to real build commands
5. Re-run bootstrap with honest `execute/status/watch` truth

---

## Proof Commands

| Context | Command | Expected |
|---------|---------|----------|
| Initialize Raspberry state | `raspberry plan --manifest fabro/programs/myosu-bootstrap.yaml` | Exit 0; `.raspberry/` created |
| Check Raspberry status | `raspberry status --manifest fabro/programs/myosu-bootstrap.yaml` | Exit 0; milestone states returned |
| games:traits proof | `cargo test -p myosu-games` | Exit 0; 10 unit + 4 doctest pass |
| tui:shell proof | `cargo test -p myosu-tui` | Exit 0 (some #[ignore] acceptable) |
| chain:pallet proof | `cargo check -p pallet-game-solver` | Currently fails (50+ errors); target: exit 0 |
| chain:runtime proof | `cargo check -p myosu-runtime` | Currently fails (commented out); target: exit 0 |
| games:multi-game assessment | Read `outputs/games/multi-game/review.md` | Classification in review.md |

---

## File Reference Index

| File | Role |
|------|------|
| `fabro/programs/myosu-bootstrap.yaml` | Bootstrap program manifest; 4 units |
| `fabro/programs/myosu.yaml` | Repo-wide program manifest; 7 frontier units |
| `fabro/run-configs/bootstrap/game-traits.toml` | games:traits run config |
| `fabro/run-configs/bootstrap/tui-shell.toml` | tui:shell run config |
| `fabro/run-configs/bootstrap/chain-runtime-restart.toml` | chain:runtime run config |
| `fabro/run-configs/bootstrap/chain-pallet-restart.toml` | chain:pallet run config |
| `fabro/checks/games-traits.sh` | Bootstrap proof for games:traits |
| `fabro/checks/tui-shell.sh` | Bootstrap proof for tui:shell |
| `fabro/checks/chain-runtime-reset.sh` | Bootstrap proof for chain:runtime (NO-OP) |
| `fabro/checks/chain-pallet-reset.sh` | Bootstrap proof for chain:pallet (NO-OP) |
| `outputs/games/traits/spec.md` | games:traits lane spec |
| `outputs/games/traits/review.md` | games:traits lane review (KEEP) |
| `outputs/tui/shell/spec.md` | tui:shell lane spec |
| `outputs/tui/shell/review.md` | tui:shell lane review (PARTIAL KEEP / REOPEN) |
| `outputs/chain/runtime/spec.md` | chain:runtime lane spec |
| `outputs/chain/runtime/review.md` | chain:runtime lane review (RESET) |
| `outputs/chain/pallet/spec.md` | chain:pallet lane spec |
| `outputs/chain/pallet/review.md` | chain:pallet lane review (RESET) |
| `outputs/games/multi-game/review.md` | games:multi-game review (NOT YET ASSESSED) |
| `fabro/workflows/bootstrap/*.fabro` | Workflow graphs (NOT YET AUDITED) |

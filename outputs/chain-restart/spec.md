# Chain Restart — Spec

**Plan:** 007-chain-restart
**Date:** 2026-04-03
**Status:** Complete

---

## Goal

Bootstrap the first service slice: a deterministic `myosu-chain` binary with a known health surface.

---

## Build Verification

| Command | Result |
|---|---|
| `cargo check -p myosu-chain` | ✅ PASS — `Finished` in ~3 min, 10 warnings (runtime + pallet-game-solver) |
| `cargo build -p myosu-chain` | ✅ PASS — `Finished` in ~18 min, binary at `target/debug/myosu-chain` (1.1 GB) |

Binary run command:
```bash
./target/debug/myosu-chain --dev --rpc-port 9933
```

---

## Workspace Structure

Three packages are workspace members:

| Package | Path | Role |
|---|---|---|
| `myosu-chain` (node) | `crates/myosu-chain/node` | Binary entry point |
| `myosu-chain-runtime` | `crates/myosu-chain/runtime` | Runtime crate |
| `pallet-game-solver` | `crates/myosu-chain/pallets/game-solver` | Core game-solver pallet |

`pallet-admin-utils` is **not** a workspace member. It is declared as a workspace-level dependency and used by the runtime via `pallet-admin-utils.workspace = true`. This pattern is intentional — it allows the runtime to reference the pallet without compiling it as a top-level workspace target.

---

## Runtime Pallet Wiring

The runtime uses a feature-gated `construct_runtime!`:

**`#[cfg(not(feature = "full-runtime"))]`** — default build:
```
System              = 0
Timestamp           = 2
Aura                = 3
Grandpa             = 4
Balances            = 5
TransactionPayment  = 6
SubtensorModule     = 7   ← pallet-game-solver (aliased as pallet_subtensor)
Utility             = 11
AdminUtils          = 19
```

**`#[cfg(feature = "full-runtime")]`** — adds:
```
RandomnessCollectiveFlip = 1
Sudo                = 12
Multisig            = 13
Preimage            = 14
Scheduler           = 15
Proxy               = 16
SafeMode            = 20
```

The alias in `runtime/Cargo.toml`:
```toml
pallet_subtensor = { package = "pallet-game-solver", path = "../pallets/game-solver", default-features = false }
```

`SubtensorModule` in `construct_runtime!` refers to `pallet_game_solver`.

---

## Unwired Pallets

The following pallets exist in the workspace but are **not** in `construct_runtime!`:

| Pallet | Workspace dep | Status |
|---|---|---|
| `commitments` | `subtensor-commitments` | Not wired — exists, compiles |
| `registry` | `pallet-registry` | Not wired — exists, compiles |
| `swap` | `pallet-subtensor-swap` | Not wired — exists, compiles |
| `crowdloan` | `pallet-crowdloan` | DISABLED per plan decision |
| `drand` | `pallet-drand` | DISABLED per plan decision |
| `transaction-fee` | `subtensor-transaction-fee` | Runtime API only, not a frame pallet |

These are not blocking the stage-0 binary. Decision on whether to wire them belongs to the next planning cycle.

---

## Warnings

**`pallet-game-solver` — 8 warnings (pre-existing)**
- Unused imports: `StorePreimage`, `UnfilteredDispatchable`, `dispatch::GetDispatchInfo`, `ScheduleAnon`, `ecdsa::Signature`, `Hash`, `QueryPreimage`, `Saturating`
- Unused constants: `MAX_NUM_ROOT_CLAIMS`, `MAX_ROOT_CLAIM_THRESHOLD`, `MAX_SUBNET_CLAIMS`
- Dead functions: `do_recycle_alpha`, `do_burn_alpha`, `do_add_stake_burn`

**`myosu-chain-runtime` — 2 warnings (fixed in this session)**
- `fungible::HoldConsideration` — removed from `use frame_support::traits`
- `EnsureRoot`, `EnsureRootWithSuccess` — removed from `use frame_system`

---

## Health Surface (Stage 0)

| Component | Status |
|---|---|
| Node binary builds | ✅ |
| Runtime compiles | ✅ |
| `game-solver` pallet wired | ✅ index 7 as `SubtensorModule` |
| `admin-utils` wired | ✅ index 19 (runtime dep, not workspace member) |
| `pallet-subtensor` alias → game-solver | ✅ |
| Binary produces blocks | ⏳ Not verified in this session |
| RPC endpoint responds | ⏳ Not verified in this session |

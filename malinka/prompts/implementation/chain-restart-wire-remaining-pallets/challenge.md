# Wire remaining critical pallets Lane — Challenge

Perform a cheap adversarial review of the current slice for `chain-restart-wire-remaining-pallets` before the expensive final review runs.

Your job is to challenge assumptions, find obvious scope drift, identify weak proof, and catch mismatches between code and artifacts. Do not bless the slice as merge-ready; that belongs to the final review gate.


Verification artifact must cover
- summarize the automated proof commands that ran and their outcomes

Current Slice Contract:
Plan file:
- `genesis/plans/007-chain-restart.md`

Child work item: `chain-restart-wire-remaining-pallets`

Full plan context (read this for domain knowledge, design decisions, and specifications):

# Chain Restart

**Plan ID:** 007
**Status:** New — highest-risk technical plan
**Priority:** FOUNDATION — everything downstream depends on this

`genesis/PLANS.md` at `genesis/PLANS.md` governs this document.

---

## Purpose / Big Picture

After this plan lands, `crates/myosu-chain` will be re-enabled in the workspace, the Substrate runtime will be wired correctly, and `cargo build -p myosu-chain` will produce a runnable binary that can start a local devnet and produce blocks. The game-solver pallet will be integrated into the runtime.

This is the most technically demanding plan in the 180-day turnaround. It involves deep Substrate/FRAME knowledge and is the hardest to reverse.

---

## Progress

- [ ] Re-enable `crates/myosu-chain` in workspace `Cargo.toml`
- [ ] Create/verify `myosu-chain/Cargo.toml` with all pallet dependencies
- [ ] Wire `game-solver` pallet into runtime `lib.rs`
- [ ] Wire remaining pallets: `admin-utils`, `commitments`, `registry`, `swap`, `subtensor`
- [ ] Get `cargo check -p myosu-chain` to pass
- [ ] Get `cargo build -p myosu-chain` to produce a binary
- [ ] Start local devnet and verify block production

---

## Surprises & Discoveries

*(To be written during implementation)*

---

## Decision Log

- Decision: Restart the chain with `game-solver` as the primary pallet, not `subtensor`.
  Rationale: `subtensor` is the reference/fork-base. The unique value is `game-solver`. Other subtensor pallets (staking, emission) should be wired but are not the focus.
  Date/Author: 2026-03-21 / Interim CEO

- Decision: Disable `pallet-crowdloan` and `pallet-drand` for now.
  Rationale: These were already marked for stripping per plan `031926-iterative-execution-and-raspberry-hardening.md` (CF-09 strip CRV3 Timelock Commit-Reveal). Defer until Phase 3.
  Date/Author: 2026-03-21 / Interim CEO

---

## Outcomes & Retrospective

*(To be written at plan completion)*

---

## Milestones

### M1: Re-enable `crates/myosu-chain` in workspace
Uncomment `crates/myosu-chain` from root `Cargo.toml` members. Verify `cargo metadata` shows it.

Proof: `cargo metadata --format-version 1 | jq '.packages[] | select(.name == "myosu-chain")'` returns the package metadata.

Key files:
- `Cargo.toml` (root) — workspace members
- `crates/myosu-chain/Cargo.toml` — chain package

### M2: Create `myosu-chain/node` binary entry point
The chain needs a node binary (`main.rs`) that initializes the Substrate service.

Proof: `cargo check -p myosu-chain-node` passes.

Key files:
- `crates/myosu-chain/node/src/main.rs` — node entry point
- `crates/myosu-chain/node/src/service.rs` — service initialization

### M3: Wire `game-solver` pallet into runtime
Integrate `pallet_game_solver` into the runtime's `construct_runtime!` macro and configure the `GameSolverConfig`.

Proof: `cargo check -p myosu-chain-runtime` passes; no missing pallet errors.

Key files:
- `crates/myosu-chain/runtime/src/lib.rs` — runtime construction
- `crates/myosu-chain/pallets/game-solver/src/lib.rs` — pallet definition

### M4: Wire `subtensor` base pallet for consensus
Integrate `pallet_subtensor` for staking, emission, and registration infrastructure.

Proof: `cargo check -p myosu-chain-runtime` passes.

### M5: Wire remaining critical pallets
Wire `admin-utils`, `commitments`, `registry`, `swap`, `transaction-fee`.

Proof: `cargo check -p myosu-chain-runtime` passes with all pallets integrated.

### M6: Build binary and start devnet
Build the chain binary and start it against a local dev chain. Verify block production.

Proof: `cargo build -p myosu-chain && ./target/debug/myosu-chain --dev --rpc-port 9933` produces blocks; `curl -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","method":"chain_getBlock","id":1}' http://localhost:9933` returns a block.

---

## Context and Orientation

Current chain state:
```
crates/myosu-chain/
├── Cargo.toml          # COMMENTED OUT in workspace
├── node/               # EMPTY — needs main.rs
├── runtime/            # EMPTY — needs lib.rs
├── pallets/
│   ├── game-solver/    # ACTIVE — well-tested, needs wiring
│   ├── subtensor/      # BASE — needs wiring
│   ├── admin-utils/    # NEEDED
│   ├── commitments/   # NEEDED
│   ├── registry/       # NEEDED
│   ├── swap/           # NEEDED
│   ├── transaction-fee/ # NEEDED
│   ├── crowdloan/      # DISABLED
│   ├── drand/          # DISABLED
│   ├── proxy/          # DISABLED
│   ├── shield/         # DISABLED
│   └── utility/        # DISABLED
├── primitives/
│   ├── safe-math/      # EXISTS
│   └── share-pool/     # EXISTS
└── support/
    ├── macros/         # EXISTS
    ├── procedural-fork/ # EXISTS
    ├── linting/        # EXISTS
    └── tools/          # EXISTS
```

Architecture for the runtime wiring:

```
runtime/src/lib.rs:
  construct_runtime! {
    // ... system, aura, grandpa, balances, transaction-payment ...
    GameSolver: pallet_game_solver,
    Subtensor: pallet_subtensor,
    AdminUtils: pallet_admin_utils,
    Commitments: pallet_commitments,
    Registry: pallet_registry,
    Swap: pallet_swap,
    TransactionFee: pallet_transaction_fee,
  }
```

---

## Plan of Work

1. Uncomment `crates/myosu-chain` from root `Cargo.toml`
2. Create `crates/myosu-chain/node/src/main.rs` and `service.rs`
3. Create `crates/myosu-chain/runtime/src/lib.rs` with `construct_runtime!`
4. Wire `pallet_game_solver` first (the primary pallet)
5. Wire `pallet_subtensor` for base consensus
6. Wire remaining pallets in dependency order
7. Run `cargo check` iteratively to resolve missing types
8. Build the binary and test block production

---

## Concrete Steps

```bash
# Step 1: Uncomment myosu-chain in workspace
sed -i 's|# \(crates/myosu-chain"\)|  \1|' Cargo.toml
cargo metadata --format-version 1 | jq '.workspace_members | length'

# Step 2: Create node entry point
cat > crates/myosu-chain/node/src/main.rs << 'EOF'
fn main() {
    myosu_chain_node::service::run()
}
EOF

# Step 3: Run cargo check to see what's missing
cargo check -p myosu-chain 2>&1 | head -100

# Step 4: Iterate — fix missing modules, types, dependencies
# (This is the bulk of the work — expect 50-100 iterations)

# Step 5: Build the binary
cargo build -p myosu-chain

# Step 6: Start devnet
./target/debug/myosu-chain --dev --rpc-port 9933 &
sleep 10
curl -s -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"chain_getBlock","id":1}' \
  http://localhost:9933 | jq '.result.block.number'
```

---

## Validation

- `cargo check -p myosu-chain` passes with zero errors
- `cargo build -p myosu-chain` produces a binary at `target/debug/myosu-chain`
- `./target/debug/myosu-chain --dev --rpc-port 9933` starts and produces blocks
- `curl http://localhost:9933` returns a working RPC endpoint
- `pallet_game_solver` is callable via `api.tx.gameSolver.*`

---

## Failure Scenarios

| Scenario | Handling |
|----------|----------|
| `procedural-fork` diverges from upstream `frame-support-procedural` | Pin to specific upstream commit; do not auto-update |
| Runtime `construct_runtime!` macro errors | Work backwards from the error — usually missing `RuntimeEvent` or `RuntimeCall` variants |
| Substrate toolchain version mismatch | Pin `rust-toolchain.toml` to the version that works with `polkadot-sdk stable2407` |
| Missing `sp-api` runtime API implementations | Generate them using `sp-api::decl_runtime_apis!` macro |
| Chain produces blocks but RPC returns errors | Check that the RPC module is properly registered in the runtime |


Workflow archetype: implement

Review profile: standard

Active plan:
- `genesis/plans/001-master-plan.md`

Active spec:
- `genesis/SPEC.md`

AC contract:
- Where: Runtime construction macro — admin-utils, commitments, registry, swap, transaction-fee
- How: Wire admin-utils, commitments, registry, swap, and transaction-fee pallets into construct_runtime!
- Required tests: cargo check -p myosu-chain-runtime
- Verification plan: cargo check -p myosu-chain-runtime passes with all pallets integrated
- Rollback condition: cargo check -p myosu-chain-runtime fails with any missing pallet error

Proof commands:
- `cargo check -p myosu-chain-runtime`

Artifacts to write:
- `spec.md`
- `review.md`

Challenge checklist:
- Is the slice smaller than the plan says, or larger?
- Did the implementation actually satisfy the first proof gate?
- Are any touched surfaces outside the named slice?
- Are the artifacts overstating completion?
- Is there an obvious bug, trust-boundary issue, or missing test the final reviewer should not have to rediscover?

Write a short challenge note in `verification.md` or amend it if needed, focusing on concrete gaps and the next fixup target. Do not write `promotion.md` here.

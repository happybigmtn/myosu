# Stage-0 Release Gate

Last updated: 2026-03-30
Status: Advisory release gate for stage-0. A stage-0 completion claim should
not be made unless every item below is marked `PASS`.

## Invariant Checklist

### INV-001: Structured Closure Honesty

- Status: PASS
- Why:
  - the live invariant now points at the current control-plane surfaces instead
    of deleted Malinka-era files
  - completion claims are anchored to the release gate, no-ship ledger, and
    the stage-0 completion contract
- Proof commands:
  - `sed -n '1,80p' INVARIANTS.md`
  - `sed -n '1,120p' ops/no-ship-ledger.md`
  - `sed -n '1,200p' ops/stage0-completion-contract.md`

### INV-002: Proof Honesty

- Status: PASS
- Why:
  - active plans now carry real proof commands and recent completion claims are
    backed by executed commands, not placeholders
- Proof commands:
  - `bash .github/scripts/check_plan_quality.sh`
  - `bash .github/scripts/check_doctrine_integrity.sh`

### INV-003: Game Verification Determinism

- Status: PASS
- Why:
  - the validator scoring path and the node-owned local loop both have live,
    repeatable proof surfaces in the repo
- Proof commands:
  - `SKIP_WASM_BUILD=1 cargo test -p myosu-chain --test stage0_local_loop --quiet`
  - `cargo test -p myosu-validator --quiet`

### INV-004: Solver-Gameplay Separation

- Status: PASS
- Why:
  - gameplay and solver remain separate crates with no direct dependency path
- Proof commands:
  - `cargo tree -p myosu-play | rg 'myosu-miner'`
  - `cargo tree -p myosu-miner | rg 'myosu-play'`

Expected: both commands return no matches.

### INV-005: Plan And Land Coherence

- Status: PASS
- Why:
  - the repo now has an explicit completion-claim and fail-closed rollback
    contract
  - the master plan, Genesis report, release gate, and no-ship ledger now
    agree on the current promoted/completed truth
- Proof commands:
  - `sed -n '1,200p' ops/stage0-completion-contract.md`
  - `rg -n "No promoted|Completed|011|010|002" genesis/plans/001-master-plan.md genesis/GENESIS-REPORT.md ops/no-ship-ledger.md`

### INV-006: Robopoker Fork Coherence

- Status: PASS
- Why:
  - the workspace pin and the repo-local fork changelog now document the exact
    audited fork rev relative to the `v1.0.0` baseline
- Proof commands:
  - `sed -n '1,220p' docs/robopoker-fork-changelog.md`
  - `rg -n "happybigmtn/robopoker|04716310143094ab41ec7172e6cea5a2a66744ef" crates/myosu-games/Cargo.toml crates/myosu-games-poker/Cargo.toml Cargo.lock docs/robopoker-fork-changelog.md`

## No-Ship Result

- Current result: READY FOR STAGE-0 COMPLETION CLAIM ON CURRENT PROOF POSTURE
- Blocking items:
  - none

## Supporting Proofs

- Gameplay/TUI productization:
  - `SKIP_WASM_BUILD=1 cargo run -p myosu-play --quiet -- --smoke-test`
- Chain compile and lint:
  - `SKIP_WASM_BUILD=1 cargo check -p myosu-chain-runtime --features fast-runtime`
  - `SKIP_WASM_BUILD=1 cargo check -p myosu-chain --features fast-runtime`
  - `SKIP_WASM_BUILD=1 cargo clippy -p myosu-chain-runtime --features fast-runtime -- -D warnings`
  - `SKIP_WASM_BUILD=1 cargo clippy -p myosu-chain --features fast-runtime -- -D warnings`
- Pallet stage-0 surface:
  - `SKIP_WASM_BUILD=1 cargo check -p pallet-game-solver`
  - `cargo test -p pallet-game-solver stage_0_flow --quiet`

## Operator Note

This gate is intentionally strict. A green gate does not force a stage-0
completion claim, but it means the current local and hosted proof posture is no
longer blocked by stale governance doctrine.

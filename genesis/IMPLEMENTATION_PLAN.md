# IMPLEMENTATION_PLAN

This is the prioritized implementation queue derived from the 15 generated specs and verified against the current codebase at `/home/r/Coding/myosu`. All 15 specs document existing behavioral contracts ("This spec adds: Nothing new") and every acceptance criterion is satisfied by the current code. There is no implementation work to queue.

## Priority Work

No incomplete items. All specs are satisfied by the current codebase.

## Follow-On Work

No follow-on items. All specs are satisfied by the current codebase.

## Completed / Already Satisfied

- `specs/001-game-trait-framework.md` — GameType enum (4 variants) with byte-level round-trip serialization at `crates/myosu-games/src/traits.rs:57-113`, GameParams at `:141-162`, StrategyResponse validation (0.001 epsilon) at `:201-207`, GameRegistry with 3 built-in descriptors at `crates/myosu-games/src/registry.rs:43-49`. Property-based round-trip tests at `traits.rs:444-448`.

- `specs/002-poker-solver-engine.md` — PokerSolver wrapping NlheFlagshipSolver at `crates/myosu-games-poker/src/solver.rs:24-26`, MYOS checkpoint magic at `:19`, NlheBlueprint read-only inference at `crates/myosu-games-poker/src/robopoker.rs:79-149`, wire codecs at `crates/myosu-games-poker/src/wire.rs:26-85`, artifact bundles with abstraction manifests at `crates/myosu-games-poker/src/artifacts.rs:54-76`.

- `specs/003-liars-dice-engine.md` — Full game tree with 36 chance outcomes at `crates/myosu-games-liars-dice/src/game.rs:116-126`, public/secret info separation at `:91-110` and `:156-161`, MCCFR solver at `crates/myosu-games-liars-dice/src/solver.rs:121-193`, wire codec at `crates/myosu-games-liars-dice/src/wire.rs:27-85`, INV-004 independence test at `lib.rs:42-53`.

- `specs/004-terminal-ui-framework.md` — Five-panel layout at `crates/myosu-tui/src/shell.rs:75-79`, three responsive tiers (Narrow/Compact/Desktop) at `:82-88`, event loop with tokio select at `events.rs:110-159`, 8 screen types at `screens.rs:15-34`, 6 interaction states at `events.rs:40-48`, pipe mode at `pipe.rs:19-93`, GameRenderer trait at `renderer.rs:24-52`.

- `specs/005-gameplay-surface.md` — Three modes (train/pipe/smoke-test) at `crates/myosu-play/src/main.rs:66-94`, four-tier blueprint resolution at `blueprint.rs:420-503`, chain-based miner discovery at `discovery.rs:24-46`, live HTTP strategy queries at `live.rs:32-65`, 250ms background refresh at `main.rs:702`, Fresh/Stale/Offline connectivity at `main.rs:59-64`, smoke test with requirement flags at `main.rs:96-187`.

- `specs/006-chain-rpc-client.md` — 12+ RPC methods, 18+ storage query patterns, 13 transaction types, 13 polling operations, and sr25519 signing with 8 transaction extensions all implemented in `crates/myosu-chain-client/src/lib.rs` (27,000+ lines).

- `specs/007-miner-process.md` — 9 ordered startup phases at `crates/myosu-miner/src/main.rs:21-82`, key resolution at `cli.rs:89-99`, chain registration at `chain.rs:65-75`, MCCFR training with checkpoint resume at `training.rs:152-178`, single-shot strategy serving at `strategy.rs:154-197`, persistent HTTP axon server (/health, /strategy) at `axon.rs:252-277`, game selection (Poker/LiarsDice) at `cli.rs:6-10`.

- `specs/008-validator-process.md` — 12 ordered startup phases at `crates/myosu-validator/src/main.rs:27-117`, deterministic L1-distance scoring at `validation.rs:345-367`, subnet bootstrap (subtoken, tempo, rate limit, commit-reveal) at `main.rs:48-76`, stake bootstrap with validator permit at `main.rs:78-87`, direct/commit-reveal weight submission at `main.rs:95-114`, game selection at `cli.rs:6-10`.

- `specs/009-key-management.md` — 12-word sr25519 mnemonic at `crates/myosu-keys/src/lib.rs:90-94`, key import (mnemonic/seed/keyfile) at `main.rs:27-33`, XSalsa20Poly1305 + scrypt KDF at `storage.rs:347-375`, TOML config at `storage.rs:396-424`, JSON keyfile at `storage.rs:49-61`, 0o600 permissions at `storage.rs:453-467`, account management (list/display/switch/export/password rotation) at `main.rs:34-45`, bootstrap command generation at `main.rs:456-490`.

- `specs/010-game-solver-pallet.md` — Pallet-game-solver (aliased as pallet_subtensor in runtime) at `crates/myosu-chain/pallets/game-solver/`. Subnet storage for up to 128 subnets, hotkey/coldkey hierarchy, 1B RAO/block emission with Yuma Consensus, direct and commit-reveal weight submission, validator permits, stage-0 swap interface (1:1 identity) at `src/lib.rs:72-120`, per-subnet hyperparameters, axon info storage at `src/lib.rs:257-313`.

- `specs/011-chain-runtime.md` — Spec version 385 at `crates/myosu-chain/runtime/src/lib.rs:358`, pallet-game-solver at runtime index 7 via `SubtensorModule: pallet_subtensor = 7` (Cargo.toml aliases `pallet_subtensor = { package = "pallet-game-solver" }`), NoopSwap at `:85-258`, SafeMode at index 20, construct_runtime! at `:1214-1237` with all required pallets.

- `specs/012-chain-node.md` — Aura and BABE consensus at `crates/myosu-chain/node/src/consensus/`, 4 chain spec templates at `src/chain_spec/{devnet,localnet,testnet,finney}.rs`, WebSocket RPC (default 9944), build-spec command, stage-0 local loop smoke test at `tests/stage0_local_loop.rs`.

- `specs/013-operator-deployment-bundle.md` — 5 generated scripts at `.github/scripts/prepare_operator_network_bundle.sh:46-92`, TOML manifest at `:111-133`, auto-trigger key generation at `:29-32`, CI validation via `check_operator_network_bootstrap.sh`.

- `specs/014-ci-verification-pipeline.md` — 7 CI jobs at `.github/workflows/ci.yml`: repo-shape (foundation), active-crates, chain-core, doctrine, plan-quality, operator-network, chain-clippy. All jobs confirmed present with correct dependency ordering and concurrency control.

- `specs/015-python-research-framework.md` — 5 root-level Python files (data.py, main.py, methods.py, metrics.py, runner.py) totaling ~3,200 lines. 20-game survey corpus at `data.py:82-482`, bootstrap confidence intervals at `metrics.py:61-71`. Spec explicitly scopes out tests/linting/type checking and states no new work.

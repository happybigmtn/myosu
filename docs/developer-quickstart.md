# Developer Quickstart

This is the consolidated contributor quickstart for the myosu repo. It pulls
the critical caveats that were previously scattered across `README.md`,
`AGENTS.md`, and `OS.md` into one place so a fresh contributor can reach a
trustworthy first success in four commands, understand the most common failure
modes, and find the right environment variables without grepping the source
tree.

The plan entry is `IMPLEMENTATION_PLAN.md` `DX-001`
(spec [`specs/110426-developer-experience.md`](../specs/110426-developer-experience.md)).
The acceptance criteria for this quickstart are enforced by the executable
gate `tests/e2e/developer_quickstart.sh`, which is wired into the
`developer-quickstart` CI job in `.github/workflows/ci.yml`.

This page is contributor-facing. For operator-facing flows (key, bundle,
miner, validator, devnet) see
[`docs/operator-guide/quickstart.md`](operator-guide/quickstart.md).

## 1. Prerequisites

Install these on a Linux development host before running anything in the
repo:

- Stable Rust toolchain with edition 2024 support. `rust-toolchain.toml`
  pins `stable`; you need `cargo`, `rustfmt`, and `clippy`.
- The two WASM targets the current chain proof paths require:

  ```bash
  rustup target add wasm32v1-none
  rustup target add wasm32-unknown-unknown
  ```

- `protoc` / `protobuf-compiler` for the inherited Substrate and chain
  build steps. On Debian / Ubuntu:

  ```bash
  sudo apt-get update
  sudo apt-get install -y protobuf-compiler
  ```

- Python 3.11+ with `numpy`, `pytest`, and `ruff` available for the
  research-side quality gate. The repo does not yet pin Python tooling;
  install once with `python -m pip install numpy ruff pytest`.

## 2. Four-Step Fastest First-Success Path

This is the truthful local path that proves the gameplay surface without
needing the chain, keys, or operator bundle. It is the same path called out
by `specs/110426-developer-experience.md` as the "fastest first success":

```bash
# 1. Build the shared game-engine stack and confirm it is green.
SKIP_WASM_BUILD=1 cargo test -p myosu-games-kuhn --quiet

# 2. Smoke-test the gameplay surface for the stage-0 poker proof.
SKIP_WASM_BUILD=1 cargo run -p myosu-play --quiet -- --smoke-test

# 3. Smoke-test the gameplay surface for the second-game (Kuhn) proof.
SKIP_WASM_BUILD=1 cargo run -p myosu-play --quiet -- --game kuhn --smoke-test

# 4. Exercise the agent-facing pipe mode.
printf 'quit\n' | SKIP_WASM_BUILD=1 cargo run -p myosu-play --quiet -- pipe
```

All four commands run without chain, keys, or operator-bundle state. The
first cold build of the gameplay crates takes a few minutes; subsequent
runs are fast. If step 1 fails on a fresh clone, re-read the prerequisites
above; the most common cause is a missing `protoc` install or a missing
`wasm32v1-none` target.

## 3. Common Pitfalls

These are the caveats that the prior scattered docs (AGENTS.md, OS.md,
README.md) repeatedly called out. They are concentrated here so a
contributor can recognize them quickly.

### SKIP_WASM_BUILD is required for non-chain builds

`SKIP_WASM_BUILD=1` is required for every `cargo` invocation that does not
intend to rebuild the runtime WASM artifact. Without it, the
`myosu-chain-runtime` build script will try to rebuild the chain WASM in
the target tree, which takes 5+ minutes per command. See
`crates/myosu-chain/runtime/build.rs` for the gate.

### The WASM runtime cache at target/debug/wbuild must exist before node smoke

`SKIP_WASM_BUILD=1 cargo test -p myosu-chain --test stage0_local_loop --quiet`
still requires a pre-built runtime WASM at
`target/debug/wbuild/myosu-chain-runtime/myosu_chain_runtime.wasm`. On
Rust 1.94 in this repo, refresh that cache with
`cargo build -p myosu-chain-runtime` after installing
`wasm32v1-none` once via `rustup target add wasm32v1-none`, before
trusting node smoke results after runtime edits.

If `SKIP_WASM_BUILD=1 cargo run -p myosu-chain -- build-spec --chain devnet --raw`
still says `development wasm is not available` after the cache exists, force
just the runtime crate to refresh its generated `wasm_binary.rs` against
the cache with
`cargo clean -p myosu-chain-runtime && SKIP_WASM_BUILD=1 cargo build -p myosu-chain-runtime --quiet`,
then rerun the `build-spec` command.

### The repo-owned poker bootstrap artifacts are intentionally sparse

`crates/myosu-games-poker/examples/bootstrap_artifacts.rs` emits a
manifest-backed bundle with full preflop coverage plus a small
representative postflop catalog, so `crates/myosu-miner/src/training.rs`
rejects poker `--train-iterations > 0` on those bootstrap artifacts because
`complete_streets=preflop`, `sampled_streets=flop,turn,river`, and
`postflop_complete=false` rather than failing later inside robopoker.
Generated `target/e2e/*/poker/encoder/manifest.json` surfaces are just
copies of that bootstrap shape (`preflop.entries = 169`, `flop.entries = 24`,
`turn.entries = 24`, `river.entries = 24`); do not treat them as trainable
postflop poker encoders when evaluating `F-007` or any positive-iteration
poker benchmark.

### Devnet startup waits ~48 seconds per authored block on the local authority-backed devnet

The quickstart's local authority-backed `devnet` only launches
`authority-1` out of a four-authority chain spec, so local operator
transactions often wait roughly 48 seconds per authored block; miner
registration, axon publish, and validator stake flows should use
minute-scale timeouts, not the old 20-second budget.

### Long cargo quiet phases are not hangs

`SKIP_WASM_BUILD=1 cargo test -p myosu-miner -p myosu-validator --quiet`
can cold-build `wasm-opt-sys` and stay quiet for several minutes; do not
treat the silence as a hang while `cargo` still has active compiler
children. The same applies to
`SKIP_WASM_BUILD=1 cargo test -p myosu-keys --quiet`, which can spend about
a minute in `change_active_password_reencrypts_active_key`, and to
`SKIP_WASM_BUILD=1 cargo run -p myosu-chain -- build-spec --chain devnet --raw`
which can cold-build `frame-storage-access-test-runtime` from the inherited
polkadot-sdk toolchain before it emits the JSON.

## 4. Environment Variable Inventory

Every environment variable the myosu binaries actually read from the
shell, with the source of truth in the Rust source tree.

| Variable | Read by | Purpose |
| --- | --- | --- |
| `SKIP_WASM_BUILD` | `crates/myosu-chain/runtime/build.rs` | When set, skips the runtime WASM rebuild inside the build script. Must be exported for every non-chain `cargo` invocation. |
| `MYOSU_NODE_AUTHORITY_SURI` | `crates/myosu-chain/node/src/service.rs:757` | Override the authority secret URI used by the node service. Required when running a custom authority key on a devnet or testnet. |
| `MYOSU_KEY_PASSWORD` | `crates/myosu-miner/src/cli.rs`, `crates/myosu-validator/src/cli.rs`, `crates/myosu-keys/src/main.rs` | Password for the active key the miner, validator, and key-management binaries use to decrypt the keystore. The default env name in the CLI is `MYOSU_KEY_PASSWORD`; `MYOSU_PASSWORD` is the legacy name kept by the keys binary for backwards compatibility. |
| `MYOSU_PASSWORD` | `crates/myosu-keys/src/main.rs` | Legacy password env name accepted by `myosu-keys` for backwards compatibility. Prefer `MYOSU_KEY_PASSWORD` for new scripts. |
| `MYOSU_MNEMONIC` | `crates/myosu-keys/src/main.rs` | Mnemonic phrase used to import a BIP-39 key into the keystore. |
| `MYOSU_RAW_SEED` | `crates/myosu-keys/src/main.rs` | Raw 32-byte hex seed used to import a key into the keystore. |
| `MYOSU_OLD_PASSWORD` / `MYOSU_NEW_PASSWORD` | `crates/myosu-keys/src/main.rs` | Password rotation inputs for `myosu-keys change-password`. |
| `MYOSU_DATA_DIR` | `crates/myosu-play/src/blueprint.rs` | Override the runtime data directory the `myosu-play` TUI uses. |
| `MYOSU_BLUEPRINT_DIR` | `crates/myosu-play/src/blueprint.rs` | Override the blueprint directory the `myosu-play` engine consults. |
| `MYOSU_SOLVER_PROMOTION_LEDGER` | `crates/myosu-games-canonical/examples/promotion_manifest.rs` | Override the path of the solver promotion ledger YAML (`ops/solver_promotion.yaml` by default). |
| `MYOSU_ENGINE_BUDGET_MS` | `crates/myosu-games-portfolio/examples/strength_roundtrip.rs` and latency budget examples | Wall-clock budget for the rule-aware engine in milliseconds. |
| `MYOSU_ENGINE_ITERATIONS` | `crates/myosu-games-portfolio/examples/engine_quality_budget.rs` and latency budget examples | Iteration count for engine quality and latency budget examples. |
| `MYOSU_ENGINE_MIN_SCORE` | `crates/myosu-games-portfolio/examples/engine_quality_budget.rs` | Minimum passing score for the engine quality budget example. |
| `MYOSU_CRIBBAGE_BENCHMARK_OUTPUT` | `crates/myosu-games-portfolio/examples/cribbage_benchmark.rs` | Override the dossier output directory for the F-001 Cribbage benchmark. |
| `MYOSU_E2E_GAMES` | (consumed by `tests/e2e/validator_determinism.sh`) | Comma-separated list of game slugs that the portfolio validator determinism harness should cover. Defaults to the three dedicated games. |
| `MYOSU_CONFIG_DIR` | `docs/operator-guide/quickstart.md` | Operator-facing override for the config directory used by the operator bundle. |
| `MYOSU_SUBNET` | `docs/operator-guide/quickstart.md` | Operator-facing override for the subnet the operator bundle targets (stage-0 default is `7`). |
| `MYOSU_WORKDIR` | `docs/operator-guide/quickstart.md` | Operator-facing override for the per-run working directory. |

## 5. Where to Go Next

- Run the broader local green gate:
  `SKIP_WASM_BUILD=1 cargo test --workspace --quiet`
  (it can cold-build `wasm-opt-sys` and inherited Substrate crates for many
  minutes; the default workspace path intentionally skips the legacy
  `pallet-admin-utils` and `subtensor-transaction-fee` unit suites unless
  you opt into `--features full-runtime`).
- For a local chain, miner, and validator loop, switch to the operator
  guide: [`docs/operator-guide/quickstart.md`](operator-guide/quickstart.md).
- For the active plan and current priority queue, read
  [`IMPLEMENTATION_PLAN.md`](../IMPLEMENTATION_PLAN.md).
- For doctrine, read [`OS.md`](../OS.md) and
  [`AGENTS.md`](../AGENTS.md).
- For the hard non-negotiable constraints, read
  [`INVARIANTS.md`](../INVARIANTS.md).

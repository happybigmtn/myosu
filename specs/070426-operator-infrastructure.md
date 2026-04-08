# Specification: Operator Infrastructure

## Objective

Describe the operator-facing infrastructure: key management (`myosu-keys`), chain specifications, the chain client library, CI pipeline, devnet bootstrap, operator bundle generation, and the stage-0 proof commands.

## Evidence Status

### Verified (code-grounded)

- **Key management crate**: `crates/myosu-keys/` (`Cargo.toml:6`).
  - Operations documented in README.md (lines 88-103): `create`, `show-active`, `print-bootstrap`, `import-keyfile`, `import-mnemonic`, `import-raw-seed`, `list`, `export-active-keyfile`, `switch-active`, `change-password`.
  - All key operations use `--config-dir ~/.myosu` and `--network devnet|testnet|finney`.
  - Secrets handled via environment variables: `MYOSU_KEY_PASSWORD`, `MYOSU_IMPORT_MNEMONIC`, `MYOSU_IMPORT_RAW_SEED`, `MYOSU_OLD_PASSWORD`, `MYOSU_NEW_PASSWORD`.
  - Proof: `cargo check -p myosu-keys` and `cargo test -p myosu-keys --quiet`.

- **Chain client library**: `crates/myosu-chain-client/` (`Cargo.toml:9`).
  - Provides RPC client abstraction for extrinsic submission, storage queries, and game-solver pallet queries.

- **Chain specifications** (`crates/myosu-chain/node/src/chain_spec/`):
  - `devnet.rs`: Local devnet with Alice/Bob authorities.
  - `localnet.rs`: Minimal local network config.
  - `testnet.rs`: Test network placeholder.
  - `finney.rs`: Test finney placeholder.
  - Chain spec generation: `cargo run -p myosu-chain -- build-spec --chain devnet`.
  - Supported chain IDs: `devnet`, `test_finney`, `localnet`.

- **CI pipeline** (`.github/workflows/ci.yml`):
  - Trigger: PR to trunk/main, push to trunk/main, manual dispatch.
  - Jobs:
    1. `repo-shape`: Runs `check_stage0_repo_shape.sh`.
    2. `robopoker-fork-coherence`: Runs `check_robopoker_fork_status.sh` (continue-on-error).
    3. `python-research-qa`: Ruff check + pytest on `main.py`, `methods.py`, `runner.py`, `metrics.py`, `data.py`.
    4. `active-crates`: `cargo check` with `SKIP_WASM_BUILD=1`.
    5. (Additional jobs exist in the full CI file.)
  - CI scripts in `.github/scripts/`:
    - `check_doctrine_integrity.sh`: Plan/spec coherence.
    - `check_operator_network_bootstrap.sh`: Bootstrap readiness.
    - `check_stage0_repo_shape.sh`: Structure validation.
    - `check_robopoker_fork_status.sh`: Fork delta tracking.
    - `prepare_operator_network_bundle.sh`: Operator bundle generation.
    - `check_plan_quality.sh`: Plan quality gates.

- **Operator bundle** (`README.md:106-108`):
  - Generated via: `bash .github/scripts/prepare_operator_network_bundle.sh ./operator-bundle`.
  - Verification: `./operator-bundle/verify-bundle.sh`.
  - Expected contents: `devnet-spec.json`, `test-finney-spec.json`, `bundle-manifest.toml`.

- **Devnet bootstrap** (`README.md:60-68`):
  - References `fabro run` commands for bootstrap supervision.
  - References `raspberry plan/status/execute` commands.
  - **Conflict**: These commands reference `fabro/run-configs/` and `fabro/programs/myosu-bootstrap.yaml` which plan 013 identifies as ghost infrastructure (non-existent on disk).

- **Node binary**: `crates/myosu-chain/node/` (`Cargo.toml:13`).
  - Entry: `src/main.rs`.
  - Key flag: `--stage0-local-loop-smoke` runs the integrated smoke test.
  - Build: `SKIP_WASM_BUILD=1 cargo build -p myosu-chain --features fast-runtime`.
  - WASM target required: `rustup target add wasm32-unknown-unknown`.

- **Proof commands** (`README.md:118-125`):
  ```
  cargo test -p pallet-game-solver stage_0_flow --quiet
  SKIP_WASM_BUILD=1 cargo test -p myosu-chain --test stage0_local_loop --quiet
  SKIP_WASM_BUILD=1 cargo run -p myosu-play --quiet -- --smoke-test
  cargo test -p myosu-games-liars-dice --quiet
  SKIP_WASM_BUILD=1 cargo test -p myosu-miner -p myosu-validator --quiet
  ```

- **Documentation surface**:
  - `docs/operator-guide/quickstart.md`: Zero-to-running path.
  - `docs/operator-guide/architecture.md`: Mental model.
  - `docs/operator-guide/upgrading.md`: Release contract and rollback.
  - `docs/execution-playbooks/`: Operational runbooks.
  - `docs/adr/`: 9 Architecture Decision Records.
  - `CHANGELOG.md`: v0.1.0 baseline.
  - `SECURITY.md`: Vulnerability disclosure process.

### Recommendations (intended future direction)

- Plan 011 (container packaging) recommends Docker images for chain, miner, validator + docker-compose devnet.
- Plan 012 (README overhaul) recommends: listing prerequisites (Rust 2024 edition, WASM target, protoc), adding fastest test path (`cargo test -p myosu-games-kuhn`), removing broken `fabro run` commands.
- Plan 013 (fabro ghost cleanup) recommends resolving ghost infrastructure: either building the referenced fabro/raspberry directory structure or removing all references.

### Hypotheses / Unresolved

- Whether `protoc` (protobuf compiler) is actually required for building — the README plan references it but it's not verified from build scripts.
- Whether the operator bundle is tested in CI or only via manual script execution.

## Acceptance Criteria

- `cargo check -p myosu-keys` and `cargo test -p myosu-keys --quiet` pass
- `cargo check -p myosu-chain-client` passes
- `bash .github/scripts/check_stage0_repo_shape.sh` passes
- `bash .github/scripts/check_operator_network_bootstrap.sh` passes
- `bash .github/scripts/prepare_operator_network_bundle.sh ./operator-bundle` produces a bundle with `devnet-spec.json`, `test-finney-spec.json`, and `bundle-manifest.toml`
- `./operator-bundle/verify-bundle.sh` passes
- All five proof commands from README.md exit successfully
- `cargo run -p myosu-chain -- build-spec --chain devnet` produces valid JSON

## Verification

```bash
# Key management
cargo check -p myosu-keys
cargo test -p myosu-keys --quiet

# Chain client
SKIP_WASM_BUILD=1 cargo check -p myosu-chain-client

# CI structural checks
bash .github/scripts/check_stage0_repo_shape.sh
bash .github/scripts/check_operator_network_bootstrap.sh

# Operator bundle
bash .github/scripts/prepare_operator_network_bundle.sh ./operator-bundle
./operator-bundle/verify-bundle.sh
test -s ./operator-bundle/devnet-spec.json
test -s ./operator-bundle/test-finney-spec.json
test -s ./operator-bundle/bundle-manifest.toml

# Chain spec generation
SKIP_WASM_BUILD=1 cargo run -p myosu-chain --features fast-runtime -- build-spec --chain devnet >/dev/null

# All proof commands
cargo test -p pallet-game-solver stage_0_flow --quiet
SKIP_WASM_BUILD=1 cargo test -p myosu-chain --test stage0_local_loop --quiet
SKIP_WASM_BUILD=1 cargo run -p myosu-play --quiet -- --smoke-test
cargo test -p myosu-games-liars-dice --quiet
SKIP_WASM_BUILD=1 cargo test -p myosu-miner -p myosu-validator --quiet
```

## Open Questions

- Is `protoc` actually required for building, or is it a transitive dependency of an unused feature?
- Is the operator bundle generation tested in CI, or only available as a manual script?
- Do the `fabro run` and `raspberry` commands in the README actually work? Plan 013 suggests they do not.
- What is the upgrade path for key files when the key format changes across versions?

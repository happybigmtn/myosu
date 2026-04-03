# 묘수 myosu

Myosu is a decentralized game-solving chain for imperfect-information games.
Miners produce strategy, validators score it, the chain coordinates emissions,
and gameplay exposes the result to humans and agents through the same surface.

The fastest way to orient yourself is [OS.md](OS.md). It is the current
operating-system document for the repo.

## Start Here

Current top-level entrypoints:

- [OS.md](OS.md) for live doctrine, current operator loop, and stage-0 meaning
- [SPEC.md](SPEC.md) for durable repo decisions
- [INVARIANTS.md](INVARIANTS.md) for non-negotiable constraints
- [SECURITY.md](SECURITY.md) for vulnerability disclosure and response guidance
- [docs/operator-guide/quickstart.md](docs/operator-guide/quickstart.md) for
  the zero-to-running operator path using the current key, bundle, miner, and
  validator surfaces
- [docs/operator-guide/architecture.md](docs/operator-guide/architecture.md) for
  the operator-facing mental model of how chain, miner, validator, gameplay,
  and keys fit together
- [genesis/plans/001-master-plan.md](genesis/plans/001-master-plan.md) for the
  active plan stack
- [fabro/programs/myosu-bootstrap.yaml](fabro/programs/myosu-bootstrap.yaml)
  for the current Raspberry bootstrap program
- [docs/execution-playbooks/README.md](docs/execution-playbooks/README.md) for
  current execution playbooks
- [docs/execution-playbooks/operator-network.md](docs/execution-playbooks/operator-network.md)
  for the current operator-facing named-network and key-surface playbook

## Current Runnable Truth

These are the currently proven local surfaces:

- a stripped chain that authors blocks and serves the game-solver RPC surface
- miner and validator binaries that participate in the local stage-0 loop
- a poker gameplay surface in `myosu-play`
- a second-game proof with Liar's Dice
- a node-owned local loop proving poker and Liar's Dice coexist as distinct
  subnets on one local chain

What this repo does not yet claim as a first-class operator product:

- production deployment
- public multi-node network operations
- polished hosted miner or validator operations
- a broad web product surface

## Current Operator Loop

Bootstrap supervision:

```bash
fabro run fabro/run-configs/bootstrap/game-traits.toml
fabro run fabro/run-configs/bootstrap/tui-shell.toml
fabro run fabro/run-configs/bootstrap/chain-runtime-restart.toml
fabro run fabro/run-configs/bootstrap/chain-pallet-restart.toml

raspberry plan --manifest fabro/programs/myosu-bootstrap.yaml
raspberry status --manifest fabro/programs/myosu-bootstrap.yaml
raspberry execute --manifest fabro/programs/myosu-bootstrap.yaml
```

Node-owned stage-0 proof:

```bash
SKIP_WASM_BUILD=1 cargo test -p myosu-chain --test stage0_local_loop --quiet
env SKIP_WASM_BUILD=1 target/debug/myosu-chain --stage0-local-loop-smoke
```

Gameplay/advisor proof:

```bash
SKIP_WASM_BUILD=1 cargo run -p myosu-play --quiet -- --smoke-test
printf 'quit\n' | SKIP_WASM_BUILD=1 cargo run -p myosu-play --quiet -- pipe
```

Operator network prep:

```bash
export MYOSU_KEY_PASSWORD='replace-me'
cargo run -p myosu-keys --quiet -- create --config-dir ~/.myosu --network devnet
cargo run -p myosu-keys --quiet -- show-active --config-dir ~/.myosu
cargo run -p myosu-keys --quiet -- print-bootstrap --config-dir ~/.myosu --subnet 7
cargo run -p myosu-keys --quiet -- import-keyfile --config-dir ~/.myosu --source ./backup.json --network devnet
export MYOSU_IMPORT_MNEMONIC='word1 ... word12'
cargo run -p myosu-keys --quiet -- import-mnemonic --config-dir ~/.myosu --mnemonic-env MYOSU_IMPORT_MNEMONIC --password-env MYOSU_KEY_PASSWORD --network devnet
export MYOSU_IMPORT_RAW_SEED='0x...'
cargo run -p myosu-keys --quiet -- import-raw-seed --config-dir ~/.myosu --seed-env MYOSU_IMPORT_RAW_SEED --password-env MYOSU_KEY_PASSWORD --network devnet
cargo run -p myosu-keys --quiet -- list --config-dir ~/.myosu
cargo run -p myosu-keys --quiet -- export-active-keyfile --config-dir ~/.myosu --output ./active-backup.json
cargo run -p myosu-keys --quiet -- switch-active --config-dir ~/.myosu --address <ss58>
export MYOSU_OLD_PASSWORD='replace-me'
export MYOSU_NEW_PASSWORD='replace-me-too'
cargo run -p myosu-keys --quiet -- change-password --config-dir ~/.myosu --old-password-env MYOSU_OLD_PASSWORD --new-password-env MYOSU_NEW_PASSWORD
cargo check -p myosu-keys
cargo test -p myosu-keys --quiet
bash .github/scripts/check_operator_network_bootstrap.sh
bash .github/scripts/prepare_operator_network_bundle.sh ./operator-bundle
./operator-bundle/verify-bundle.sh
test -s ./operator-bundle/devnet-spec.json
test -s ./operator-bundle/test-finney-spec.json
test -s ./operator-bundle/bundle-manifest.toml
rustup target add wasm32-unknown-unknown
SKIP_WASM_BUILD=1 cargo build -p myosu-chain --features fast-runtime
env SKIP_WASM_BUILD=1 cargo run -p myosu-chain --features fast-runtime -- build-spec --chain devnet >/tmp/myosu-devnet-spec.json
env SKIP_WASM_BUILD=1 cargo run -p myosu-chain --features fast-runtime -- build-spec --chain test_finney >/tmp/myosu-testnet-spec.json
```

## Proof Commands

```bash
cargo test -p pallet-game-solver stage_0_flow --quiet
SKIP_WASM_BUILD=1 cargo test -p myosu-chain --test stage0_local_loop --quiet
SKIP_WASM_BUILD=1 cargo run -p myosu-play --quiet -- --smoke-test
cargo test -p myosu-games-liars-dice --quiet
SKIP_WASM_BUILD=1 cargo test -p myosu-miner -p myosu-validator --quiet
```

## Architecture

```text
chain        -> subnets, neurons, weights, emissions
miners       -> strategy computation and serving
validators   -> scoring and weight submission
gameplay     -> human and agent consumption
```

## Name

묘수 (myosu) means "brilliant move" or "masterstroke."

## License

MIT

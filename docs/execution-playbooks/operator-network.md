# Operator Network Playbook

## Goal

Bring up the current operator-facing chain package honestly, using the retained
named network surfaces and the new shared key helpers without pretending Myosu
already ships a polished wallet or public multi-node operations stack.

## When To Use It

Use this playbook when you need to:

- verify that named `devnet` or `test_finney` packaging still builds
- orient a second operator to the current chain-facing bring-up surface
- prepare for miner or validator work without relying only on the node-owned
  smoke wrappers

Use [stage0-local-loop.md](stage0-local-loop.md) when the goal is still the
smallest honest end-to-end proof. That remains the preferred full-loop surface.
Use [../operator-guide/troubleshooting.md](../operator-guide/troubleshooting.md)
when the happy-path commands below do not reach their expected success lines.

## Current Truth

- `cargo run -p myosu-chain --features fast-runtime -- build-spec --chain devnet`
  now succeeds
- `cargo run -p myosu-chain --features fast-runtime -- build-spec --chain test_finney`
  now succeeds
- `bash ops/deploy-bootnode.sh --dry-run` now prepares a stable devnet
  bootnode identity and metadata file under `target/bootnode/devnet/`
- `crates/myosu-keys/` now exists as a shared library for mnemonic derivation,
  SS58 address formatting, default Myosu key/config paths, encrypted seed-file
  persistence, and active-account config loading
- `myosu-keys` can now write `~/.myosu/config.toml` plus
  `~/.myosu/keys/<ss58>.json` with encrypted seed material and `0600`
  permissions
- `myosu-miner` and `myosu-validator` can now resolve their signing key either
  from a raw `--key` URI or from `--key-config-dir` plus a password env var
- `myosu-keys` now ships a minimal CLI for `create`, `import-keyfile`, `list`,
  `export-active-keyfile`, `import-mnemonic`, `import-raw-seed`,
  `show-active`, `switch-active`, `change-password`, and `print-bootstrap`
- there is **not** yet a finished wallet UI, import/export flow, or
  account-switching UX in the repo

## Operator Prep

Use this happy path first when the goal is to bootstrap one operator-owned
account and get truthful miner/validator start commands from the repo itself:

```bash
export MYOSU_KEY_PASSWORD='replace-me'
cargo run -p myosu-keys --quiet -- create --config-dir ~/.myosu --network devnet
cargo run -p myosu-keys --quiet -- show-active --config-dir ~/.myosu
cargo run -p myosu-keys --quiet -- print-bootstrap --config-dir ~/.myosu --subnet 7
```

That prints the active account plus the exact `myosu-miner` and
`myosu-validator` commands to run next for the current config dir.

If you want the repo-owned proof path instead of running the steps by hand,
use:

```bash
bash .github/scripts/check_operator_network_bootstrap.sh
```

If you want the repo to materialize reusable wrapper scripts for one operator
config dir, use:

```bash
export MYOSU_KEY_PASSWORD='replace-me'
bash .github/scripts/prepare_operator_network_bundle.sh
target/operator-network-bundle/verify-bundle.sh
```

That bundle now includes:
- runnable miner and validator wrapper scripts
- runnable named-network spec refresh scripts
- materialized `devnet-spec.json` and `test-finney-spec.json`
- the devnet bootnode multiaddr and RPC endpoint in `bundle-manifest.toml`
- machine-readable `bundle-manifest.toml`
- a bundle-local verifier

If the bootnode is not the default loopback host, set the advertised endpoint
before preparing the bundle so the generated bootnode metadata is truthful:

```bash
export MYOSU_KEY_PASSWORD='replace-me'
export MYOSU_OPERATOR_BOOTNODE_PUBLIC_HOST='devnet.myosu.example'
export MYOSU_OPERATOR_BOOTNODE_RPC_PORT='9944'
export MYOSU_OPERATOR_BOOTNODE_P2P_PORT='30333'
bash .github/scripts/prepare_operator_network_bundle.sh ./operator-bundle
```

## Devnet Join Surface

The bundle now carries the devnet bootnode address in two places:

- `bundle-manifest.toml` exposes `bootnode_multiaddr` and `bootnode_rpc_endpoint`
- `devnet-spec.json` is rewritten with that same `bootNodes` entry

That means an operator can join the devnet without editing the chain spec by
hand:

```bash
export MYOSU_KEY_PASSWORD='replace-me'
bash .github/scripts/prepare_operator_network_bundle.sh ./operator-bundle

cargo run -p myosu-chain -- \
  --chain ./operator-bundle/devnet-spec.json \
  --bootnodes "$(sed -n 's/^bootnode_multiaddr = \"\\(.*\\)\"$/\\1/p' ./operator-bundle/bundle-manifest.toml)" \
  --rpc-port 9955 \
  --prometheus-port 9616 \
  --base-path /tmp/myosu-devnet-node
```

The explicit `--bootnodes` flag is optional once you use the bundled
`devnet-spec.json`; it is shown here so operators can see the exact peer they
should expect to connect to.

If you are proving named-network packaging from a cold machine, install the
Rust wasm target first:

```bash
rustup target add wasm32-unknown-unknown
```

## Extended Account Ops

Use the wider keystore commands when you need recovery, backup, switching, or
password rotation:

```bash
export MYOSU_KEY_PASSWORD='replace-me'
cargo run -p myosu-keys --quiet -- create --config-dir ~/.myosu --network devnet
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
cargo run -p myosu-keys --quiet -- print-bootstrap --config-dir ~/.myosu --subnet 7
cargo check -p myosu-keys
cargo test -p myosu-keys --quiet
rustup target add wasm32-unknown-unknown
SKIP_WASM_BUILD=1 cargo build -p myosu-chain --features fast-runtime
env SKIP_WASM_BUILD=1 cargo run -p myosu-chain --features fast-runtime -- build-spec --chain devnet >/tmp/myosu-devnet-spec.json
env SKIP_WASM_BUILD=1 cargo run -p myosu-chain --features fast-runtime -- build-spec --chain test_finney >/tmp/myosu-testnet-spec.json
test -s /tmp/myosu-devnet-spec.json
test -s /tmp/myosu-testnet-spec.json
```

If you prefer to run the binaries directly instead of copying the printed
bootstrap output, the current operator-owned key path is:

```bash
export MYOSU_KEY_PASSWORD='replace-me'
cargo run -p myosu-miner -- --chain ws://127.0.0.1:9944 --subnet 7 --key-config-dir ~/.myosu
cargo run -p myosu-validator -- --chain ws://127.0.0.1:9944 --subnet 7 --key-config-dir ~/.myosu
```

## End-To-End Follow-Through

After the named network package is verified, fall back to the node-owned loop
for the actual current system proof:

```bash
SKIP_WASM_BUILD=1 cargo test -p myosu-chain --test stage0_local_loop --quiet
env SKIP_WASM_BUILD=1 "${CARGO_TARGET_DIR:-target}/debug/myosu-chain" --stage0-local-loop-smoke
```

## What The Key Surface Means Today

`myosu-keys` is currently a small operator CLI plus shared library seam, not a
user-facing wallet.
The current truthful uses are:

- derive a deterministic sr25519 pair from a mnemonic
- format the public key as an SS58 address
- resolve default `~/.myosu/` and `~/.myosu/keys/` paths for future operator
  tooling
- persist encrypted seed material plus active-account config under `~/.myosu/`
- resolve the active operator key back into a standard secret URI for
  `myosu-miner` and `myosu-validator`
- create a new operator account, list stored accounts, inspect the current
  active account, and switch which stored account is active through
  `myosu-keys`
- print the current miner and validator bootstrap commands for the active
  account and config dir
- import or export encrypted Myosu keyfiles without exposing decrypted seed
  material
- import mnemonic phrases or raw seeds through environment variables instead of
  shell argv
- rotate the active keystore password without changing the active account

It is not yet honest to claim:

- account switching UX
- mnemonic backup/verify flow
- key export / import parity with a broader Substrate wallet surface
- export formats beyond the current encrypted Myosu keyfile copy path
- richer account metadata or balance-aware switching flows
- password rotation for non-active accounts or bulk-keystore operations

## Proof

The current operator-network slice is healthy when:

- `cargo test -p myosu-keys --quiet` passes
- `cargo run -p myosu-keys --quiet -- create ...` creates a keystore and
  active config
- `cargo run -p myosu-keys --quiet -- list ...` reports all stored accounts for
  the config dir
- `cargo run -p myosu-keys --quiet -- export-active-keyfile ...` writes a copy
  of the active encrypted keyfile
- `cargo run -p myosu-keys --quiet -- import-keyfile ...` loads that encrypted
  keyfile into another config dir and makes it active
- `cargo run -p myosu-keys --quiet -- import-mnemonic ...` imports an account
  from a mnemonic env var and makes it active
- `cargo run -p myosu-keys --quiet -- import-raw-seed ...` imports an account
  from a raw-seed env var and makes it active
- `cargo run -p myosu-keys --quiet -- show-active ...` reports the same active
  account from disk
- `cargo run -p myosu-keys --quiet -- switch-active ...` changes the active
  account on disk and `show-active` reflects the new choice
- `cargo run -p myosu-keys --quiet -- change-password ...` re-encrypts the
  active account with a new password env var
- `cargo run -p myosu-keys --quiet -- print-bootstrap ...` prints the current
  operator bootstrap commands for miner and validator
- `bash .github/scripts/prepare_operator_network_bundle.sh ...` writes a small
  operator bundle with `start-miner.sh`, `start-validator.sh`,
  `build-devnet-spec.sh`, `build-test-finney-spec.sh`, `verify-bundle.sh`, and
  a local README derived from the active config and bootstrap output, and it
  now materializes `devnet-spec.json`, `test-finney-spec.json`, and
  `bundle-manifest.toml` into the bundle during preparation, with the devnet
  bootnode recorded both in the manifest and the bundled `devnet-spec.json`
- `bash .github/scripts/check_operator_network_bootstrap.sh` proves that the
  printed bootstrap commands reach the miner and validator `--help` surfaces
  and that the generated bundle plus named `build-spec` outputs still
  materialize afterward
- cold-machine named-network proof still requires the Rust
  `wasm32-unknown-unknown` target before the node-owned `build-spec` commands
  can compile the runtime honestly
- `SKIP_WASM_BUILD=1 cargo test -p myosu-keys -p myosu-miner -p myosu-validator --quiet`
  passes
- both named `build-spec` commands produce non-empty output files
- the node-owned local loop still passes afterward

## Known Constraints

- `devnet` and `test_finney` currently reuse the local genesis shape; this
  slice makes the named surfaces honest, not yet distinct production-like
  network definitions
- `myosu-keys` only exposes `create`, encrypted-keyfile `import` / `export`,
  env-var `import-mnemonic` / `import-raw-seed`, `list`, `show-active`, and
  `switch-active`, plus active-account password rotation and bootstrap-command
  printing; richer account lifecycle commands are still future work
- miner and validator now consume the active configured account, but the
  preferred full-loop story is still the node-owned local proof, not manual
  multi-operator bring-up

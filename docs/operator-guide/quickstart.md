# Operator Quickstart

## Goal

Bring up the current operator-facing Myosu surfaces from a fresh checkout:

- create an operator key
- materialize the repo-owned operator bundle
- point the miner and validator at a truthful chain endpoint
- run a miner bootstrap pass and then a live HTTP miner
- run the current validator bootstrap flow

This guide stays within the stage-0 truth. It does not pretend the repo already
ships a polished wallet, a public hosted devnet, or a long-running validator
daemon.

## Before You Start

Assumptions:

- you are running from the repository root
- you already have Rust and Cargo installed
- you are comfortable opening 2-4 terminals
- you either have a live devnet RPC + bootnode from your operator lead, or you
  are proving the surfaces locally on one machine
- the target chain already exposes subnet `7`; this guide does not create a
  subnet or perform owner-only subnet enablement

Set the shared environment first:

```bash
export MYOSU_KEY_PASSWORD='replace-me'
export MYOSU_CONFIG_DIR="$HOME/.myosu"
export MYOSU_SUBNET='7'
export MYOSU_WORKDIR="$PWD/target/operator-quickstart"
mkdir -p "$MYOSU_WORKDIR"
```

## 1. Build the Required Surfaces

Install the wasm targets used by the current chain proof paths, then build the
binaries this guide relies on:

```bash
rustup target add wasm32v1-none
rustup target add wasm32-unknown-unknown
cargo build -p myosu-chain-runtime
SKIP_WASM_BUILD=1 cargo build -p myosu-chain --features fast-runtime
SKIP_WASM_BUILD=1 cargo build -p myosu-keys -p myosu-games-poker -p myosu-miner -p myosu-validator
```

## 2. Create and Inspect an Operator Key

Create a keystore, print the active account, and print the repo-owned bootstrap
commands for that account:

```bash
cargo run -p myosu-keys --quiet -- create --config-dir "$MYOSU_CONFIG_DIR" --network devnet
cargo run -p myosu-keys --quiet -- show-active --config-dir "$MYOSU_CONFIG_DIR"
cargo run -p myosu-keys --quiet -- print-bootstrap --config-dir "$MYOSU_CONFIG_DIR" --subnet "$MYOSU_SUBNET"
```

The last command should print:

- `Active Address: ...`
- `Miner Command: cargo run -p myosu-miner -- ...`
- `Validator Command: cargo run -p myosu-validator -- ...`

## 3. Materialize the Operator Bundle

Decide which chain endpoint your miner and validator should use:

- If you already have a shared devnet RPC endpoint, use that.
- If you want a local follower node, use `ws://127.0.0.1:9955`.

Export that endpoint before generating the bundle so the wrapper scripts point
at the same place:

```bash
export MYOSU_CHAIN='ws://127.0.0.1:9955'
export MYOSU_OPERATOR_CHAIN="$MYOSU_CHAIN"
bash .github/scripts/prepare_operator_network_bundle.sh ./operator-bundle "$MYOSU_CONFIG_DIR"
./operator-bundle/verify-bundle.sh
```

Important:

- The second argument to `prepare_operator_network_bundle.sh` is the config
  dir. Passing `"$MYOSU_CONFIG_DIR"` makes the bundle reuse the key you just
  created.
- If you omit that second argument, the script uses a bundle-local config dir
  instead.
- If your bootnode is remote, export `MYOSU_OPERATOR_BOOTNODE_PUBLIC_HOST`,
  `MYOSU_OPERATOR_BOOTNODE_RPC_PORT`, and `MYOSU_OPERATOR_BOOTNODE_P2P_PORT`
  before generating the bundle so the bootnode metadata is truthful.

## 4. Bring Up a Chain Connection

Choose one of the following paths.

### Path A: Existing Shared Devnet

If your operator lead gave you a live RPC endpoint, keep `MYOSU_CHAIN` pointed
at that endpoint, regenerate the bundle if needed, and skip to step 5.

### Path B: Local Follower Node

Run this in a separate terminal:

```bash
env SKIP_WASM_BUILD=1 cargo run -p myosu-chain -- \
  --chain ./operator-bundle/devnet-spec.json \
  --base-path "$MYOSU_WORKDIR/devnet-node" \
  --rpc-port 9955 \
  --prometheus-port 9616
```

Then verify that the local RPC is answering:

```bash
curl -fsS \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"system_health","params":[]}' \
  http://127.0.0.1:9955
```

Continue once that request returns JSON.

## 5. Probe the Chain with the Printed Bootstrap Commands

Before starting long-running services, confirm that both binaries can resolve
the active key and talk to the chosen chain endpoint:

```bash
./operator-bundle/start-miner.sh
./operator-bundle/start-validator.sh
```

The expected first lines are:

- `MINER myosu-miner bootstrap ok`
- `VALIDATOR myosu-validator bootstrap ok`

## 6. Start a Miner

First write the bounded poker artifacts the current miner bootstrap flow needs:

```bash
mkdir -p "$MYOSU_WORKDIR/poker"
env SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-poker --example bootstrap_artifacts -- \
  "$MYOSU_WORKDIR/poker/encoder" \
  "$MYOSU_WORKDIR/poker/query.bin"
```

Now run a one-shot miner bootstrap pass. This registers the hotkey, publishes
the axon endpoint on-chain, writes a checkpoint, and answers one wire query:

```bash
./operator-bundle/start-miner.sh \
  --register \
  --serve-axon \
  --port 8080 \
  --encoder-dir "$MYOSU_WORKDIR/poker/encoder" \
  --query-file "$MYOSU_WORKDIR/poker/query.bin" \
  --response-file "$MYOSU_WORKDIR/poker/response.bin" \
  --data-dir "$MYOSU_WORKDIR/poker/miner-data"
```

That command should emit:

- `REGISTRATION myosu-miner subnet ok`
- `AXON myosu-miner publish ok`
- `TRAINING myosu-miner batch ok`
- `STRATEGY myosu-miner query ok`

The checkpoint should now exist at:

```bash
test -f "$MYOSU_WORKDIR/poker/miner-data/checkpoints/latest.bin"
```

Start the live HTTP miner in another terminal:

```bash
./operator-bundle/start-miner.sh \
  --port 8080 \
  --encoder-dir "$MYOSU_WORKDIR/poker/encoder" \
  --checkpoint "$MYOSU_WORKDIR/poker/miner-data/checkpoints/latest.bin" \
  --serve-http
```

Verify the live axon:

```bash
curl -fsS http://127.0.0.1:8080/health
```

The health endpoint should include `"status":"ok"`.

## 7. Start a Validator

The current validator surface is a bounded bootstrap/scoring command, not a
forever-running daemon. The honest operator-ready step is to register the
validator and wait until the permit/stake bootstrap path finishes:

```bash
./operator-bundle/start-validator.sh \
  --register \
  --stake-amount 100000000000000
```

That command should emit:

- `REGISTRATION myosu-validator subnet ok`
- `PERMIT myosu-validator ready ok`

If you also want the same-machine scoring and bootstrap weight path, reuse the
artifacts from the miner step:

```bash
./operator-bundle/start-validator.sh \
  --register \
  --stake-amount 100000000000000 \
  --submit-weights \
  --encoder-dir "$MYOSU_WORKDIR/poker/encoder" \
  --checkpoint "$MYOSU_WORKDIR/poker/miner-data/checkpoints/latest.bin" \
  --query-file "$MYOSU_WORKDIR/poker/query.bin" \
  --response-file "$MYOSU_WORKDIR/poker/response.bin"
```

That path should emit:

- `VALIDATION myosu-validator score ok`
- `WEIGHTS myosu-validator submission ok`

## 8. Where to Go Next

Use the deeper docs when you need more than the zero-to-running path:

- [`architecture.md`](./architecture.md) for the operator-facing explanation of
  how chain, miner, validator, gameplay, and keys fit together
- [`upgrading.md`](./upgrading.md) for release semantics, upgrade windows,
  bundle regeneration, and rollback procedure across operator-facing versions
- [`troubleshooting.md`](./troubleshooting.md) for the common operator failure
  modes across keys, bundle prep, chain connectivity, miner bootstrap, and
  validator scoring
- [`operator-network.md`](../execution-playbooks/operator-network.md) for named
  network packaging, bundle details, bootnode overrides, and extended key ops
- [`stage0-local-loop.md`](../execution-playbooks/stage0-local-loop.md) for the
  repo-owned full loop proof

## Current Constraints

- The bundle wrappers are real and recommended, but they only encode the chain
  endpoint, subnet, and key-config plumbing. You still add `--register`,
  `--serve-axon`, `--serve-http`, `--stake-amount`, and similar flags yourself.
- The validator binary is currently a bounded bootstrap/scoring tool. It is not
  yet a long-running validator service with an autonomous evaluation loop.
- The guide assumes subnet `7` already exists and is usable. If you need the
  owner-side local proof that creates the subnet, enables staking, and exercises
  the full loop on one machine, use
  [`stage0-local-loop.md`](../execution-playbooks/stage0-local-loop.md).
- The optional validator `--weight-hotkey` flag still expects a secret URI, so
  cross-operator weight targeting is not yet a polished operator surface.

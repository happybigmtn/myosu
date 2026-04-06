# Operator Troubleshooting

Use this guide when the happy-path commands in
[`quickstart.md`](./quickstart.md) or
[`operator-network.md`](../execution-playbooks/operator-network.md) do not
produce their expected success lines.

## Before You Debug

The current operator surfaces are still stage-0 Rust binaries. That means:

- many commands compile code before they print help or start work
- inherited chain/runtime warnings can appear before the real CLI output
- the validator is a bounded bootstrap/scoring command, not a long-running
  daemon

Treat a non-zero exit status, a missing success line, or a failed follow-up
probe as the truth. Do not treat compile warnings by themselves as proof that
the command failed.

Useful baseline checks:

```bash
cargo run -p myosu-keys --quiet -- show-active --config-dir "$MYOSU_CONFIG_DIR"
curl -fsS \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"system_health","params":[]}' \
  http://127.0.0.1:9955
./operator-bundle/start-miner.sh --help
./operator-bundle/start-validator.sh --help
```

## 1. `MYOSU_KEY_PASSWORD` is missing or wrong

Symptom: `myosu-keys`, `start-miner.sh`, or `start-validator.sh` says the
password environment variable is not set, or key loading fails with a decrypt
error.

Cause: the current keystore helpers expect the password env var recorded in the
bundle or bootstrap command, and the encrypted key file cannot be unlocked
without it.

Resolution:

```bash
export MYOSU_KEY_PASSWORD='replace-me'
cargo run -p myosu-keys --quiet -- show-active --config-dir "$MYOSU_CONFIG_DIR"
```

If you intentionally use a different env var name, regenerate the bootstrap
commands or bundle with the matching `--password-env` or `--key-password-env`
value.

Healthy result: `show-active` prints the active address and config path without
a decryption error.

## 2. `show-active` or `print-bootstrap` cannot find the active operator config

Symptom: `myosu-keys show-active` or `print-bootstrap` fails to load
`config.toml`, or the miner/validator resolves a different account than you
expected.

Cause: the command is pointed at the wrong config dir, or that config dir does
not have an active account yet.

Resolution:

```bash
cargo run -p myosu-keys --quiet -- create --config-dir "$MYOSU_CONFIG_DIR" --network devnet
cargo run -p myosu-keys --quiet -- show-active --config-dir "$MYOSU_CONFIG_DIR"
cargo run -p myosu-keys --quiet -- print-bootstrap --config-dir "$MYOSU_CONFIG_DIR" --subnet "$MYOSU_SUBNET"
```

If you imported a key into another directory, switch back to the intended
config dir before regenerating the bundle.

Healthy result: `show-active` and `print-bootstrap` report the same address you
intend to operate.

## 3. The operator bundle uses the wrong key or creates a fresh local key

Symptom: `prepare_operator_network_bundle.sh` succeeds, but the generated
`start-miner.sh` and `start-validator.sh` point at a different config dir or a
different active account than the one you just created.

Cause: the bundle script defaults to a bundle-local config dir unless you pass
the second positional argument or `MYOSU_OPERATOR_CONFIG_DIR`.

Resolution:

```bash
export MYOSU_OPERATOR_CHAIN="$MYOSU_CHAIN"
bash .github/scripts/prepare_operator_network_bundle.sh ./operator-bundle "$MYOSU_CONFIG_DIR"
./operator-bundle/verify-bundle.sh
grep '^config_dir = ' ./operator-bundle/bundle-manifest.toml
grep '^active_address = ' ./operator-bundle/bundle-manifest.toml
```

Healthy result: the manifest points at the config dir you intended, and the
active address matches `myosu-keys show-active`.

## 4. Chain build or `build-spec` fails on missing wasm targets

Symptom: chain build commands fail before the node starts, or helper scripts
tell you to install a missing Rust target.

Cause: the current stage-0 proof paths need both the modern runtime target
(`wasm32v1-none`) and the older build-spec target (`wasm32-unknown-unknown`)
depending on which command you are running.

Resolution:

```bash
rustup target add wasm32v1-none
rustup target add wasm32-unknown-unknown
cargo build -p myosu-chain-runtime
SKIP_WASM_BUILD=1 cargo build -p myosu-chain --features fast-runtime
```

Healthy result: the runtime cache builds once, and later chain build/spec
commands stop failing on missing-target errors.

## 5. `prepare_operator_network_bundle.sh` cannot read bootnode metadata

Symptom: the bundle script fails with `failed to read bootnode metadata` or the
generated bundle has no useful bootnode information.

Cause: the bootnode metadata file was never prepared, or the advertised host
and port overrides do not match the bootnode settings you meant to publish.

Resolution:

```bash
bash ops/deploy-bootnode.sh --dry-run
export MYOSU_OPERATOR_BOOTNODE_PUBLIC_HOST='127.0.0.1'
export MYOSU_OPERATOR_BOOTNODE_RPC_PORT='9944'
export MYOSU_OPERATOR_BOOTNODE_P2P_PORT='30333'
bash .github/scripts/prepare_operator_network_bundle.sh ./operator-bundle "$MYOSU_CONFIG_DIR"
```

For a remote bootnode, replace the loopback host and ports with the real public
values before regenerating the bundle.

Healthy result: `bundle-manifest.toml` contains `bootnode_multiaddr` and
`bootnode_rpc_endpoint`, and `devnet-spec.json` includes the same bootnode.

## 6. `curl system_health` or the miner/validator startup probe gets connection refused

Symptom: the quickstart health probe fails, or `myosu-miner` /
`myosu-validator` cannot talk to the chain endpoint.

Cause: the local node is not running, the bundle points at the wrong endpoint,
or the RPC port is different from the one you are probing.

Resolution:

```bash
export MYOSU_CHAIN='ws://127.0.0.1:9955'
export MYOSU_OPERATOR_CHAIN="$MYOSU_CHAIN"
env MYOSU_NODE_AUTHORITY_SURI='//myosu//devnet//authority-1' SKIP_WASM_BUILD=1 cargo run -p myosu-chain -- \
  --chain ./operator-bundle/devnet-spec.json \
  --base-path "$MYOSU_WORKDIR/devnet-node" \
  --validator \
  --rpc-port 9955 \
  --port 30333 \
  --allow-private-ip \
  --prometheus-port 9616
```

In another terminal:

```bash
curl -fsS \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"system_health","params":[]}' \
  http://127.0.0.1:9955
```

If the bundle was built against another endpoint, regenerate it after exporting
the correct `MYOSU_OPERATOR_CHAIN`.

Healthy result: the curl probe returns JSON, and the miner/validator startup
report prints `bootstrap ok`.

## 7. Registration or staking fails with `Inability to pay some fees`

Symptom: `start-miner.sh --register` or `start-validator.sh --stake-amount ...`
fails with `Invalid Transaction: Inability to pay some fees (e.g. account
balance too low)`.

Cause: on the checked-in local `devnet`, a freshly created operator key is not
endowed at genesis. Only the named bootstrap accounts start funded, so local
registration and staking require an explicit transfer into the new key first.

Resolution:

```bash
export MYOSU_OPERATOR_ADDRESS="$(
  cargo run -p myosu-keys --quiet -- show-active --config-dir "$MYOSU_CONFIG_DIR" \
    | sed -n 's/^Active Address: //p'
)"
cargo run --quiet -p myosu-chain-client --example fund_account -- \
  "$MYOSU_CHAIN" \
  "//myosu//devnet//subnet-owner" \
  "$MYOSU_OPERATOR_ADDRESS" \
  "120000000000000"
```

If you are joining a shared devnet instead of a local one, ask the operator
lead which funded account or faucet flow is expected there before retrying.

Healthy result: the funding command prints
`TRANSFER myosu-chain-client keep-alive ok`, and the later miner/validator
registration step stops failing on insufficient balance.

## 8. Registration or staking times out on the local authority-backed devnet

Symptom: `start-miner.sh --register`, `--serve-axon`, or
`start-validator.sh --stake-amount ...` appears to stall for a while, or older
binaries fail with a timeout even though the local node is still healthy.

Cause: the checked-in `devnet` chain spec reserves four Aura authorities, but
the quickstart local path only starts `authority-1`. On that single-authority
bring-up, the node only authors its own slots, so blocks land about once every
48 seconds. Bootstrap transactions can therefore need multiple tens of seconds
before inclusion.

Resolution:

- Wait for at least 1-3 minutes before treating the command as hung.
- Confirm the node is still producing blocks:

```bash
curl -fsS \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"chain_getHeader","params":[]}' \
  http://127.0.0.1:9955
```

- Rebuild to pick up the current operator timeout budget if your local binary
  still exits too early:

```bash
SKIP_WASM_BUILD=1 cargo build -p myosu-miner -p myosu-validator
```

Healthy result: the head number keeps advancing over time, and the miner or
validator eventually prints its `REGISTRATION ... ok`, `AXON ... ok`, or
`PERMIT ... ready ok` line.

## 9. The node looks stuck or `wait_for_block` times out

Symptom: the local node starts, but the RPC endpoint takes a long time to come
up, or block waiting/sync checks time out during a cold start.

Cause: fresh stage-0 node boots are not instant. The runtime cache, genesis
load, and service initialization can take well over a minute on cold machines.

Resolution:

```bash
export MYOSU_E2E_WAIT_TIMEOUT=120
bash tests/e2e/helpers/start_devnet.sh
bash tests/e2e/helpers/wait_for_block.sh 1
```

For manual bring-up, wait for the node log to show block production before
declaring failure.

Healthy result: the node reaches block `1` or higher, and `system_health`
starts reporting usable RPC state.

## 10. Miner or validator registration fails because subnet `7` is not ready

Symptom: `--register`, `--stake-amount`, or `--enable-subtoken` does not reach
its expected success line.

Cause: the current operator guide assumes subnet `7` already exists. On a local
chain, subnet creation and staking enablement are still owner-side actions.

Resolution:

- If you are joining a shared devnet, confirm with the operator lead that
  subnet `7` is live and staking is enabled there.
- If you are proving the loop locally, use the owner-side flow in
  [`stage0-local-loop.md`](../execution-playbooks/stage0-local-loop.md) or run
  the full proof script:

```bash
bash tests/e2e/local_loop.sh
```

Healthy result: the relevant command prints `REGISTRATION ... subnet ok`,
`SUBTOKEN ... subnet ok`, or `PERMIT ... ready ok`.

## 11. Miner bootstrap exits with `--encoder-dir` / `--checkpoint` / `--query-file` errors

Symptom: `myosu-miner` stops immediately with a flag-contract error such as
`--query-file requires --encoder-dir when --game poker` or
`--query-file requires --checkpoint or --train-iterations`.

Cause: the bounded poker bootstrap path needs a manifest-backed encoder and
either a checkpoint or a training batch that creates one in the same run.

Resolution:

```bash
mkdir -p "$MYOSU_WORKDIR/poker"
env SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-poker --example bootstrap_artifacts -- \
  "$MYOSU_WORKDIR/poker/encoder" \
  "$MYOSU_WORKDIR/poker/query.bin"

./operator-bundle/start-miner.sh \
  --register \
  --serve-axon \
  --port 8080 \
  --encoder-dir "$MYOSU_WORKDIR/poker/encoder" \
  --query-file "$MYOSU_WORKDIR/poker/query.bin" \
  --response-file "$MYOSU_WORKDIR/poker/response.bin" \
  --data-dir "$MYOSU_WORKDIR/poker/miner-data"
```

Healthy result: the miner prints `TRAINING myosu-miner batch ok` and
`STRATEGY myosu-miner query ok`, and the checkpoint exists under
`$MYOSU_WORKDIR/poker/miner-data/checkpoints/latest.bin`.

## 12. The live HTTP miner will not start or `/health` is not `{"status":"ok"}`

Symptom: `--serve-http` fails with a checkpoint or encoder error, the process
cannot bind the port, or `curl http://127.0.0.1:8080/health` does not return an
`ok` payload.

Cause: live HTTP serving only works for poker right now, and it still requires
both a checkpoint and an encoder dir. Liar's Dice remains on the bounded
file-based scoring path (`--query-file` + `--response-file`) instead of the
live axon. A port conflict can also prevent the listener from binding.

Resolution:

```bash
./operator-bundle/start-miner.sh \
  --port 8080 \
  --encoder-dir "$MYOSU_WORKDIR/poker/encoder" \
  --checkpoint "$MYOSU_WORKDIR/poker/miner-data/checkpoints/latest.bin" \
  --serve-http
```

If port `8080` is already in use, pick another port and update the health probe
to match.

If you are operating a Liar's Dice miner, do not pass `--serve-http`. Bootstrap
the checkpoint with the bounded query/response flow instead and let validators
score the emitted files directly.

Healthy result: the miner prints `HTTP myosu-miner axon ok`, and `curl /health`
returns JSON containing `"status":"ok"`.

## 13. Validator scoring or weight submission fails

Symptom: the validator does not print `VALIDATION myosu-validator score ok` or
`WEIGHTS myosu-validator submission ok`, or it reports decode/checkpoint
problems while scoring.

Cause: the validator expects a matching artifact set: encoder dir, checkpoint,
query file, and response file from the same miner proof slice. Weight
submission can also target the wrong hotkey if `--weight-hotkey` is set
incorrectly.

Resolution:

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

If you are weighting a different miner, make sure `--weight-hotkey` is the
intended secret URI. If you are only bootstrapping yourself, omit it so the
validator self-weights by default.

Healthy result: the validator prints both `VALIDATION myosu-validator score ok`
and `WEIGHTS myosu-validator submission ok`.

## 14. The validator exits after one run and looks like it crashed

Symptom: `start-validator.sh` prints its success lines and then exits back to
the shell instead of staying resident.

Cause: that is the current stage-0 design. The validator binary is still a
bounded bootstrap/scoring command, not a forever-running daemon.

Resolution:

- treat the printed success lines as the proof surface for now
- rerun the command when you need another bounded scoring or weight submission
- use the chain, miner, and gameplay surfaces for the long-lived parts of the
  local loop

Healthy result: the command exits `0` after printing the expected bounded
bootstrap or scoring reports.

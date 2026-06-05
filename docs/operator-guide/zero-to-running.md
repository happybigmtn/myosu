# Operator Bundle Zero-to-Running Proof (W-05)

This document is the operator-facing companion to
`tests/e2e/operator_bundle_live.sh`. It walks a new operator
through every step the harness runs, names the exact command at
each step, and points at the failure mode for that step so a
fresh operator running the same path manually knows where to
look if a step fails. The harness itself is the executable
proof-of-truth — it composes the existing P0 surfaces
(testnet entrypoint, multi-validator compose, live-read) into
one on-host zero-to-running check, and the `operator-bundle-live`
CI job runs it on every PR / push to trunk.

> The CEO + Design lens: the operator bundle is the documented
> first-class surface. Until W-05 shipped, the only end-to-end
> check was `ops/deploy-bootnode.sh --dry-run` — an operator
> reading the bundle had no executable proof that the bundle
> really composes into a live, miner-registering, weight-emitting
> chain. W-05 closes that gap with one harness, one CI job, and
> this doc.

## When to run this proof

Run `bash tests/e2e/operator_bundle_live.sh` whenever you want a
single command that answers the question "is the operator
bundle actually end-to-end live?". Concretely:

- after a chain-spec, pallet, runtime, or chain-binary change,
  to confirm the bundle still composes
- after a miner or validator CLI change, to confirm the
  operator-side flags still register + score + submit
- after a live-read change, to confirm the post-restart read
  still works
- as a fresh-machine smoke test for a new operator, with the
  harness as the executable checklist

The harness composes the existing P0 surfaces; it does NOT
add a new pallet, extrinsic, chain spec, consensus surface,
validator scoring formula, emission math, miner axon server, or
live-read proof. Every command the harness runs is one that
already passes in CI today.

## What the harness proves

The harness runs nine steps and fail-closes on any of them:

| # | Step                                                | What it proves                                                                                                |
|---|-----------------------------------------------------|---------------------------------------------------------------------------------------------------------------|
| 1 | build runtime + operator binaries                   | the harness can build the chain runtime with `fast-runtime`, the chain node, the miner / validator / play binaries, and the `compose_proof_driver` example before the proof starts |
| 2 | boot persistent testnet chain (test_finney spec)    | the persistent-entrypoint invariant from P0 #4 holds — the testnet spec bootstraps, the CORS+WS RPC is up, and `chain_getHeader` returns parseable blocks |
| 3 | wait for `target_block`                             | the chain is authoring blocks while the harness prepares the rest of the surface (sanity check on the slow path) |
| 4 | run one-shot subnet-owner init                      | the testnet-spec `//myosu//testnet//subnet-owner` account is registered and the subtoken is enabled (the P0 #4 surface) |
| 5 | bootstrap poker artifacts + run miner bootstrap     | the operator-side miner path works end-to-end on the testnet-spec authority / owner / operator accounts: bootstrap, register, publish axon, train one batch, query strategy |
| 6 | register both validators + submit weights           | the operator-side validator path works end-to-end: bootstrap, register, stake, acquire permit, score, submit weights. Both `//myosu//testnet//validator-1` and `//myosu//testnet//validator-2` reach `WEIGHTS ... submission ok` |
| 7 | assert both validators' weights agree               | the on-chain `Weights` rows for `//myosu//testnet//miner-1` are integer-equal (the INV-003 agreement check on the on-chain row, the same surface the P0 multi-validator compose proof enforces) |
| 8 | restart the chain (kill + re-launch, same base)     | the persistent-entrypoint invariant survives a restart: the new `chain_getHeader` returns a parseable block AND the post-restart tip is strictly greater than the pre-restart tip (proves the chain is still authoring) |
| 9 | post-restart live-read proof                        | the live-read half of the operator-bundle milestone is reachable after a restart: `myosu-miner --serve-http` reports `{"status":"ok",...}` on `/health`, `myosu-play --read-solved` reports `status=solved` with a 64-hex `bundle_hash` and a non-negative `emission` value |

A failure on any step prints a concrete `OPERATOR_BUNDLE_LIVE_FAIL
step=<N> <diagnostic>` line and exits non-zero. The harness's
final `OPERATOR_BUNDLE_LIVE myosu e2e ok` line is the proof.

## How to run it

The default invocation runs the harness in the foreground with
sensible defaults:

```bash
bash tests/e2e/operator_bundle_live.sh
```

The harness uses the following tunables (all optional, all
default to safe values):

| Env var                       | Default | Purpose                                                                  |
|-------------------------------|---------|--------------------------------------------------------------------------|
| `MYOSU_E2E_CHAIN_PORT`        | 9966    | chain HTTP+WS port (must be free)                                        |
| `MYOSU_E2E_MINER_PORT`        | 8089    | miner HTTP axon port (must be free)                                      |
| `MYOSU_E2E_TARGET_BLOCK`      | 6       | pre-restart target block (the chain waits for this many blocks before the two-validator compose) |
| `MYOSU_E2E_RESTART_BLOCK`     | 12      | post-restart target block (the chain waits for this many blocks after restart) |
| `MYOSU_E2E_READY_TIMEOUT`     | 240     | chain readiness timeout in seconds                                       |
| `MYOSU_E2E_EPOCH_TIMEOUT`     | 180     | validator permit + weight submit timeout in seconds                      |
| `MYOSU_E2E_MINER_TIMEOUT`     | 240     | miner HTTP `/health` timeout in seconds                                  |
| `MYOSU_E2E_VALIDATOR_STAKE`   | 100000000000000 | per-validator stake in rao (matches the testnet spec endowment)         |
| `MYOSU_E2E_COMPOSE_WEIGHT_EPSILON` | 0.000001 | epsilon passed to `compose_proof_driver` for the on-chain `Weights` agreement check |

For a faster local run (useful when iterating on the harness
itself), override the two block targets down to 4 and 6:

```bash
MYOSU_E2E_TARGET_BLOCK=4 MYOSU_E2E_RESTART_BLOCK=6 \
  MYOSU_E2E_CHAIN_PORT=9970 MYOSU_E2E_MINER_PORT=8093 \
  bash tests/e2e/operator_bundle_live.sh
```

For a debug run that keeps the work directory on exit
(`target/e2e/operator-bundle-live.<random>/` contains the
chain log, the bootstrap / miner / validator / live-read
stdout + stderr, and the persistent chain base-path), set:

```bash
MYOSU_KEEP_E2E_WORK=1 bash tests/e2e/operator_bundle_live.sh
```

The work directory is the on-host analogue of the docker
compose stack — every step's logs are in their own
`<step>.stdout` / `<step>.stderr` file inside the work dir.

## Step-by-step walkthrough (and failure modes)

This section is the operator-facing companion to the
harness's nine steps. Each step lists (a) the exact command the
harness runs, (b) the line protocol the harness grep-asserts,
and (c) the failure mode to look at if the step fails.

### Step 1: build runtime + operator binaries

The harness builds the chain runtime with `fast-runtime`, the
chain node, the miner / validator / play binaries, and the
`compose_proof_driver` example up front so the proof run is
not interrupted by build timeouts:

```bash
env -u SKIP_WASM_BUILD cargo build -p myosu-chain-runtime \
  --features fast-runtime --quiet
env SKIP_WASM_BUILD=1 cargo build -p myosu-chain \
  --features fast-runtime --quiet
env SKIP_WASM_BUILD=1 cargo build --quiet -p myosu-miner \
  -p myosu-validator -p myosu-play
env SKIP_WASM_BUILD=1 cargo build -p myosu-chain-client \
  --example compose_proof_driver --quiet
```

The `runtime_wasm` artifact at
`target/debug/wbuild/myosu-chain-runtime/myosu_chain_runtime.wasm`
is what `myosu-chain` loads at boot. A build failure here is a
real Rust change that broke the build, not a harness issue —
re-run the failing `cargo build` directly to see the compiler
error.

### Step 2: boot persistent testnet chain (test_finney spec)

The harness starts the chain with the same flag set the
`docker-compose.testnet.yml` `chain` service uses, so the
healthcheck + RPC + force-authoring behavior is identical to
the live operator-bundle surface:

```bash
MYOSU_NODE_AUTHORITY_SURI=//myosu//testnet//authority-1 \
  myosu-chain \
    --chain test_finney \
    --base-path <work_root>/chain \
    --node-key-file <work_root>/chain/node-key \
    --validator --force-authoring \
    --rpc-external --rpc-methods unsafe \
    --rpc-port <chain_port> --rpc-cors all \
    --prometheus-port 9622 --port 30449 \
    --allow-private-ip \
    --name "W-05 Operator Bundle Proof Authority"
```

The `--rpc-cors all --rpc-external --rpc-methods unsafe` triple
is the CORS-enabled WS/HTTP surface the W-01 manifest contract
guarantees. `--validator --force-authoring` is what keeps the
chain producing blocks while the external one-shot
subnet-owner-init and miner + validator bootstraps run against
it. The chain runs as a long-lived background process inside
the harness; the harness's `wait_for_block 1` probes
`chain_getHeader` and confirms the RPC is parseable.

**Failure mode:** if the chain process exits before reaching
block 1, the harness prints the last 120 lines of
`<work_root>/chain.log`. The most common cause is a port
collision (check `MYOSU_E2E_CHAIN_PORT` is free) or a
base-path / node-key collision (the harness generates a fresh
`node-key` per run inside `<work_root>/chain/`).

### Step 3: wait for `target_block`

The chain author at `//myosu//testnet//authority-1` produces a
block every `~48` seconds on a single-authority dev chain. The
harness waits up to `MYOSU_E2E_READY_TIMEOUT` seconds for the
chain to reach the pre-restart target block (default 6) before
proceeding to the subnet-owner init.

**Failure mode:** if the chain does not reach the target
within the timeout, the harness prints the last 120 lines of
`<work_root>/chain.log` and exits 1. The most common cause is
the chain stalled on `Backing off claiming new slot for block
authorship: finality is lagging.` — which on a single-authority
dev chain means the RPC and the consensus layer are out of
sync. Restart the harness; if it persists, the chain-side
invariant is broken and the failure is in the runtime, not the
harness.

### Step 4: run one-shot subnet-owner init

The harness runs the one-shot `myosu-validator --enable-subtoken`
extrinsic with the testnet-spec `//myosu//testnet//subnet-owner`
key:

```bash
env SKIP_WASM_BUILD=1 myosu-validator \
  --chain ws://127.0.0.1:<chain_port> \
  --subnet 7 \
  --key //myosu//testnet//subnet-owner \
  --enable-subtoken
```

The harness asserts the output carries both
`VALIDATOR myosu-validator bootstrap ok` and
`SUBTOKEN myosu-validator subnet ok`. The testnet spec already
enables subtoken for subnet 7, so the `--enable-subtoken` call
short-circuits with `already_enabled=true` and the
`SUBTOKEN ... ok` line is emitted from the `already_enabled`
branch.

**Failure mode:** a `VALIDATOR ... bootstrap ok` line missing
means the chain RPC rejected the validator's `chain_getHeader`
or `state_getMetadata` call — check that the chain is at
block ≥ 1 and that `--chain` URL is reachable. A
`SUBTOKEN ... subnet ok` line missing means the
`--enable-subtoken` extrinsic failed; the harness's
`run_logged` step prints the validator's stdout + stderr if
this happens.

### Step 5: bootstrap poker artifacts + run miner bootstrap

The harness runs the poker `bootstrap_artifacts` example to
write the deterministic encoder + query.bin + checkpoint
binaries the miner + validators need, then runs
`myosu-miner --register --serve-axon` with the testnet-spec
`//myosu//testnet//miner-1` key:

```bash
env SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-poker \
  --example bootstrap_artifacts -- <encoder_dir> <query_file>

env SKIP_WASM_BUILD=1 myosu-miner \
  --chain ws://127.0.0.1:<chain_port> \
  --subnet 7 \
  --key //myosu//testnet//miner-1 \
  --port <miner_port> \
  --register --serve-axon \
  --encoder-dir <encoder_dir> \
  --query-file <query_file> \
  --response-file <response_file> \
  --data-dir <miner_data_dir>
```

The harness asserts the miner output carries
`MINER ... bootstrap ok`, `REGISTRATION ... subnet ok`,
`AXON ... publish ok`, `TRAINING ... batch ok`, and
`STRATEGY ... query ok`, AND that the on-disk
`<miner_data_dir>/checkpoints/latest.bin` and
`<response_file>` are both non-empty.

**Failure mode:** a `MINER ... bootstrap ok` missing means the
chain RPC rejected the miner — most commonly a port collision
on `<miner_port>` or a `Transaction is temporarily banned` from
the previous `MyosuPermit` stake failure. A
`REGISTRATION ... subnet ok` missing means the
`register_network` extrinsic failed (the subnet is missing or
`NetworkRegistrationAllowed=false`); the harness's
`run_logged` step prints the miner's stdout + stderr so the
specific pallet error is in `<work_root>/miner_bootstrap.stderr`.

### Step 6: register both validators + submit weights

The harness runs the same operator-facing flow for
`//myosu//testnet//validator-1` and `//myosu//testnet//validator-2`
in a loop:

```bash
env SKIP_WASM_BUILD=1 myosu-validator \
  --chain ws://127.0.0.1:<chain_port> \
  --subnet 7 \
  --key <validator_key> \
  --register \
  --stake-amount <validator_stake> \
  --submit-weights \
  --weight-hotkey //myosu//testnet//miner-1 \
  --encoder-dir <encoder_dir> \
  --checkpoint <checkpoint_path> \
  --query-file <query_file> \
  --response-file <response_file>
```

The harness asserts the validator output carries
`VALIDATOR ... bootstrap ok`, `REGISTRATION ... subnet ok`,
`PERMIT ... ready ok`, `VALIDATION ... score ok`,
`exact_match=true`, and `WEIGHTS ... submission ok`.

**Failure mode:** the longest single failure mode is the
`PERMIT ... ready ok` line missing — the validator may need
to wait for the next epoch boundary (tempo is 2 blocks on the
testnet spec) before it can submit weights. The harness
extends the validator's permit-bound wait to
`MYOSU_E2E_EPOCH_TIMEOUT` seconds; on a slow host this may not
be enough, in which case bump it via the env var. The other
assertion is `WEIGHTS ... submission ok` — a missing line
means the `set_weights` extrinsic failed; the
`<work_root>/<validator>_bootstrap.stderr` carries the specific
pallet error.

### Step 7: assert both validators' weights agree

The harness runs the `compose_proof_driver` example to read
the on-chain `Weights` rows each validator submitted for
`//myosu//testnet//miner-1` and asserts they agree within
the INV-003 epsilon window:

```bash
env SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-chain-client \
  --example compose_proof_driver -- \
    ws://127.0.0.1:<chain_port> 7 \
    //myosu//testnet//miner-1 \
    //myosu//testnet//validator-1 \
    //myosu//testnet//validator-2 \
    <weight_epsilon>
```

The harness parses the example's key/value line protocol with
`grep ^key=`, requires `agreement_within_epsilon=true`,
`validator_a_target_weight == validator_b_target_weight`, and
both target weights are non-zero. The on-chain row is a
`Vec<(u16, u16)>` so agreement collapses to integer equality
(u16 weights cannot be non-finite; the score-domain epsilon is
floored to 0 weight units).

**Failure mode:** `agreement_within_epsilon=false` means the
two on-chain `Weights` rows disagree on the miner UID. The
`compose_proof_driver` stdout carries the literal
`validator_a_weights` and `validator_b_weights` lists, so the
operator can see which UIDs are in disagreement. The most
common cause is one validator scored against a different
checkpoint than the other — check that both validator bootstraps
used the same `<checkpoint_path>` from step 5.

### Step 8: restart the chain

The harness records the pre-restart tip, kills the running
chain, waits 3 seconds for the base-path lock + RPC port to
release, then re-launches the chain with the same flag set and
the same base-path / node-key. The post-restart chain has to
first re-bind + sync, then start authoring from a fresh slot,
so the harness waits for any block strictly greater than
`pre_restart_tip`:

```bash
# pre-restart:
pre_restart_tip=$(read_block_number)
kill $node_pid
sleep 3
# re-launch with the same flag set:
start_chain
wait_for_block $((pre_restart_tip + 1))
post_restart_tip=$(read_block_number)
wait_for_block <restart_block>
```

The harness asserts `post_restart_tip > pre_restart_tip` to
prove the chain survived the restart and is still authoring.
The persistent chain data dir is the same `<work_root>/chain/`
the pre-restart chain wrote to, so the chain does NOT start
from block 0 — it resumes from the pre-restart tip.

**Failure mode:** if the post-restart chain does not progress
past `pre_restart_tip` within `MYOSU_E2E_READY_TIMEOUT`
seconds, the harness exits 1 with the pre + post tips in the
error message. The most common cause is the chain author still
holding the base-path lock from the pre-restart process; the 3
second `sleep 3` settle is what releases it. If the failure
persists, the chain-side restart invariant is broken and the
failure is in the runtime, not the harness.

### Step 9: post-restart live-read proof

The harness starts the miner HTTP axon (long-running) so the
live-read proof can POST a real `/strategy` request against
the live miner artifact, then runs `myosu-play --read-solved`
to discover the miner axon, play one poker hand, and read
on-chain `miner_uid` + `emission`:

```bash
myosu-miner \
  --chain ws://127.0.0.1:<chain_port> \
  --subnet 7 \
  --key //myosu//testnet//miner-1 \
  --port <miner_port> \
  --encoder-dir <encoder_dir> \
  --checkpoint <checkpoint_path> \
  --serve-http &

# wait for /health 200
wait_for_miner_health <miner_timeout_secs>

env SKIP_WASM_BUILD=1 myosu-play \
  --chain ws://127.0.0.1:<chain_port> \
  --subnet 7 \
  --read-solved
```

The harness asserts the `myosu-play` output carries
`status=solved`, `chain_endpoint=...`, `subnet=7`, and that
`bundle_hash` is 64 lowercase hex, `miner_uid` is a positive
integer, `emission` is a non-negative integer. The harness
also cross-checks the live-read `miner_uid` against the
`compose_proof_driver` `miner_uid` from step 7 — a mismatch
means the chain moved UIDs between the two reads, which is a
real cross-check (the two reads use different on-chain
storage keys).

**Failure mode:** `status=solved` missing means the
live-read proof fell off one of its expected paths
(`NoMiner`, `QueryFailed`, `ChainReadFailed`) — the
`<work_root>/live_read_after_restart.stderr` carries the
specific failure reason. A `bundle_hash` not matching
`^[0-9a-f]{64}$` means the SHA-256 over the canonical
`(edge, probability)` pairs produced a malformed hash, which
is a real bug in the live-read module. An `emission` not
matching `^[0-9]+$` means the on-chain `Emission(7)[uid]`
storage read returned a non-integer value — check the
on-chain emission state directly via `myosu-chain-client`.

## Tunables that match the harness defaults

The harness is intentionally tolerant of local timing — the
default `MYOSU_E2E_READY_TIMEOUT=240` and
`MYOSU_E2E_EPOCH_TIMEOUT=180` cover the slow-path case. For a
local host with a warm `target/debug/` cache and a quiet
disk, the harness typically completes in 4-6 minutes. For a
cold build on a slow host, it can take 30+ minutes. The
companion drift-guard is wired into the `operator-bundle-live`
CI job, which uses the harness's default tunables.

## Relationship to other proof surfaces

| Proof surface                                                       | Layer                                  | Composed by W-05? |
|---------------------------------------------------------------------|----------------------------------------|--------------------|
| `tests/e2e/testnet_entrypoint_compose.sh`                           | persistent testnet RPC + subnet-owner init | yes (step 2-4)   |
| `tests/e2e/stage0_multi_validator_compose.sh`                       | two-validator on-chain `Weights` row agreement | yes (step 5-7) |
| `tests/e2e/live_read.sh`                                            | miner axon + read-solved               | yes (step 9)       |
| `ops/deploy-bootnode.sh --dry-run`                                  | operator-bundle manifest + config file | no (operator-side, runs in a docker compose stack) |
| `tests/e2e/operator_bundle_live.sh`                                 | all of the above, one harness          | this doc           |

The harness is a thin shell over the existing CI proofs. Every
command it runs is one that already passes in CI today. The
contribution of W-05 is the composition, the restart proof,
the cross-check between the compose-proof `miner_uid` and the
live-read `miner_uid`, and the single-line
`OPERATOR_BUNDLE_LIVE myosu e2e ok` proof marker that the
`operator-bundle-live` CI job greps for.

## Where to go next

- [`quickstart.md`](./quickstart.md) for the per-step operator
  path the harness composes into a single zero-to-running proof
- [`troubleshooting.md`](./troubleshooting.md) for the common
  operator failure modes across keys, bundle prep, chain
  connectivity, miner bootstrap, and validator scoring
- [`public-testnet.md`](./public-testnet.md) for the
  machine-readable public testnet contract (chain spec,
  promotable games, authority URIs, the 24h bootnode rotation
  policy) the W-01 surface pins
- [`observability.md`](./observability.md) for the
  `VALIDATOR_SCORING_METRIC` line the validator scoring loop
  emits once per scoring run, which is the operator-readable
  complement to the chain-side quality score
- [`INVARIANTS.md`](../../INVARIANTS.md) INV-003 for the
  on-chain `Weights` row determinism invariant the agreement
  check enforces
- `tests/e2e/operator_bundle_live.sh` for the executable
  source of truth this doc walks through

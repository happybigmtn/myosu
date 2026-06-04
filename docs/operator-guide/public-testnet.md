# Public Myosu Testnet — Reference Contract (W-01)

This document is the public operator-facing contract for the Myosu
testnet. It names the public WebSocket / HTTP endpoints, the chain spec
they serve, the four authority URIs the chain's Aura / Grandpa
consensus is anchored to, and the public read-only RPCs that are
guaranteed open. The companion machine-readable contract lives at
[`ops/testnet/manifest.yaml`](../../ops/testnet/manifest.yaml); the
companion example `system_health` response body lives at
[`ops/testnet/healthcheck.json`](../../ops/testnet/healthcheck.json);
the executable drift guard lives at
`tests/e2e/public_testnet_manifest.sh` and is wired into the
`public-testnet-manifest` CI job. The four sources of truth must stay
in sync — a drift in any one is a W-01 follow-up finding, not a
silent override of the public contract.

> The CEO lens: the testnet is the product. No public endpoint means
> no adoption. This document and the manifest it describes are the
> first operator-facing artifact an external agent or partner reads.
> If the contract drifts from what the public chain actually serves,
> the partner is right and the contract is wrong.

## What the public contract guarantees

### Chain spec

The public endpoints serve the `test_finney` chain spec
(`crates/myosu-chain/node/src/chain_spec/testnet.rs`). The spec is
`ChainType::Local` (a deliberate guard rail — see the long-form
comment in `chain_spec/testnet.rs`) so a deployment mistake cannot
accidentally promote the chain to a public mainnet without an
explicit spec change. The spec bootstraps subnet 7, the four testnet
authority URIs, the testnet subnet owner + hotkey, and the testnet
operator accounts (`miner-1`, `validator-1`, `validator-2`,
`orchestrator`).

### WebSocket / HTTP RPC endpoints

The public endpoints are reachable over `wss://` and `https://` on
the documented host / port pair. The flag set on the persistent
entrypoint is:

```text
--rpc-external
--rpc-cors all
--rpc-methods unsafe
--rpc-port 9944
```

so the same endpoint works from a browser (CORS is wide open) and
exposes the full Substrate RPC surface (`unsafe` methods is the
operator's documented choice; the public chain is read-only — there
is no key material on the public host that could submit
authored-by-someone-else extrinsics, and the validator hotkeys are
held by the operator off-host). The same flag set is what
`ops/docker/chain-testnet-entrypoint.sh` and `docker-compose.testnet.yml`
already wire, so the public contract is the literal contract the
repo-owned compose profile exposes; this document only makes it
explicit and points external consumers at it.

### Public read-only RPCs that are guaranteed open

The public contract guarantees the following JSON-RPC methods return
a parseable response from the public endpoints:

| Method           | Purpose                                             |
| ---------------- | --------------------------------------------------- |
| `system_health`  | Healthcheck (peers / isSyncing / shouldHavePeers).  |
| `system_chain`   | Returns the chain spec name (`Myosu Testnet`).      |
| `system_name`    | Returns the node name.                              |
| `system_version` | Returns the binary version string.                  |
| `chain_getHeader`| Returns the latest block header (proves authoring). |
| `state_getStorage`| Returns a raw SCALE-encoded storage value for a key.|

Anything that mutates state (transaction submission, key
generation, off-chain worker configuration, etc.) is intentionally
not part of the public contract — the public host is read-only by
operator policy, and consumers that need to author transactions
must do so against an operator-supplied RPC that is paired with a
funded SURI.

### Authority URIs

The public chain's Aura / Grandpa consensus is anchored to the four
testnet authority URIs. These are the same URIs the spec hard-codes,
so the public endpoints and the chain spec must agree:

- `//myosu//testnet//authority-1`
- `//myosu//testnet//authority-2`
- `//myosu//testnet//authority-3`
- `//myosu//testnet//subnet-owner`

The `subnet-owner` URI is also the coldkey for the
`--enable-subtoken` extrinsic that registers subnet 7 staking on
the running chain (the same one-shot extrinsic
`chain-testnet-subnet-owner-init.sh` runs as a compose sidecar).
External consumers never need any of these SURIs; they are listed
here so an operator auditing the public contract can confirm the
public chain's authority set matches the spec.

### Public subnet

The public endpoints serve subnet 7. The subnet owner is the
testnet coldkey above; subnet 7 is the only subnet with
`promotable_local` content in `ops/solver_promotion.yaml` (Liar's
Dice, per the F-001 / F-007 chain of work). External consumers
should connect to the public endpoints and ask for subnet 7
explicitly; the on-chain axon / incentive / weight data the public
chain serves is keyed off subnet 7 by default.

### Public promotable games

The public contract advertises one game at `tier: promotable_local`
or stricter: `liars-dice`. This matches the current
`ops/solver_promotion.yaml` ledger and the manifest's
`promotable_games` field. Games below that tier are not part of
the public contract — the public chain carries their subnet data,
but the engine is not yet strong enough to publish a verified
policy bundle. The contract advances as the ledger advances;
follow-on W-N+1 work promotes the next game once it lands in the
ledger.

### Bootnode rotation policy

The operator rotates the public bootnode list on a 24-hour schedule
and updates `last_rotated_at` in `ops/testnet/manifest.yaml` to the
ISO 8601 timestamp of the rotation. An external consumer that
caches the bootnode list should treat any list older than 24 hours
as advisory and refresh from the manifest before connecting. A
stale bootnode list is a follow-up W-N+1 finding, not a silent
override of this contract.

## How an external consumer connects

The minimum-viable connection is one curl / one `myosu-chain-client`
call. The example below uses the canonical `system_health` body the
public contract guarantees:

```bash
curl -fsS -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"system_health","params":[]}' \
  https://myosu-testnet.example.com:9944
```

A canonical response body looks like (and matches the
`ops/testnet/healthcheck.json` example file):

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "peers": 4,
    "isSyncing": false,
    "shouldHavePeers": true
  }
}
```

A consumer that wants to confirm the chain is authoring blocks can
follow the same pattern with `chain_getHeader` and inspect the
`number` field:

```bash
curl -fsS -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"chain_getHeader","params":[]}' \
  https://myosu-testnet.example.com:9944 \
  | python3 -c '
import json, sys
d = json.load(sys.stdin)
block_hex = d["result"]["number"]
print(int(block_hex, 16))
'
```

A consumer that wants to read a raw storage value (for example, the
`SubnetOwner(7)` AccountId) needs the SCALE-encoded storage key.
The `tests/e2e/testnet_entrypoint_compose.sh` proof harness already
resolves the `SubnetOwner(7)` key from the raw testnet spec; the
same key shape holds on the public chain. Pass the key as a string
to `state_getStorage`:

```bash
curl -fsS -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"state_getStorage","params":["<hex key>"]}' \
  https://myosu-testnet.example.com:9944
```

The response's `result` field is a hex-encoded SCALE value
(`0x`-prefixed, 32 bytes for `AccountId32`). Decode with
`scale_decode::Decode` or with `python3 -c "import json,sys;
d=json.load(sys.stdin); print(bytes.fromhex(d['result'].lstrip('0x')))"`.

## Failure modes the consumer should expect

The public contract is honest about what is and is not yet
deployed. An external consumer that hits the public endpoints may
encounter any of the following; none is a contract violation.

- **HTTP 5xx.** The public host is restarting (chain upgrade,
  bootnode rotation, operator maintenance). Retry with exponential
  backoff; the `public-testnet-manifest` CI job does not currently
  assert live uptime, only the contract shape, so a temporary
  outage is an operator-supplied signal, not a CI finding.
- **HTTP 404 / DNS NXDOMAIN.** The public hostname is not yet
  pointed at a live host. The contract advertises the shape; the
  operator's job is to do the DNS + cert work. The
  `last_rotated_at` timestamp in `ops/testnet/manifest.yaml` is
  the operator's "as of" record.
- **`isSyncing: true`.** The chain just started, or a multi-block
  upgrade is in progress. The compose profile intentionally
  re-probes `chain_getHeader` until the chain is authoring (the
  same `isSyncing` briefly stays `true` during genesis in the
  `testnet_entrypoint_compose.sh` proof harness). Wait and retry.
- **`peers: 0`.** The chain has the public endpoints open but no
  bootnodes have connected. This is a follow-up W-N+1 finding
  against the operator, not a public contract violation.
- **Unknown method.** The public contract lists six methods that
  are guaranteed open. Anything else (e.g. `author_submitExtrinsic`)
  returns `Method not found` from the Substrate RPC layer; the
  public contract is read-only by design.

## What this contract does NOT cover

- **Public deployment.** The hostname `myosu-testnet.example.com` is
  a placeholder; the contract advertises the shape, the operator
  ships the live host. The CI job does not currently test live
  uptime; it tests the contract shape, which is the part the
  repo can own.
- **Multi-host determinism.** Two public hosts producing
  weight-submission agreement within INV-003 epsilon is a separate
  W-04 row.
- **Operator-bundle end-to-end proof.** A fresh operator who runs
  `bash tests/e2e/operator_bundle_live.sh` and sees a clean
  devnet, two validators agree, and emission flow is a separate
  W-05 row.
- **First-class observability.** Structured
  `VALIDATOR_SCORING_METRIC` lines a wrapper script can scrape is
  a separate W-06 row.
- **Public policy bundles.** The Liar's Dice `promotable_local`
  policy bundle as a downloadable sample is a separate W-02 row.

## Drift guard

The contract drift guard is the executable end-to-end gate. It
fail-closes on any of the following:

- `docs/operator-guide/public-testnet.md` is missing or unlabeled.
- `ops/testnet/manifest.yaml` is missing or fails to parse as
  YAML.
- `ops/testnet/healthcheck.json` is missing or does not contain
  the `peers` / `isSyncing` / `shouldHavePeers` fields.
- The manifest's `chain_spec` field is not `test_finney`.
- The doc does not name the `wss://` / `--rpc-cors all` /
  `--rpc-methods unsafe` contract.
- The doc does not embed the example `system_health` body.
- The doc does not name the four authority URIs.
- `README.md` "Operator Path" loses the pointer to this doc.

A drift in any of the above forces the contract to be updated in
the same commit. Run the guard locally with
`bash tests/e2e/public_testnet_manifest.sh`; the
`public-testnet-manifest` CI job runs it on every PR and push to
trunk.

## Source of truth

The sources the guard pins against:

- `docs/operator-guide/public-testnet.md` (this file)
- `ops/testnet/manifest.yaml` (the machine-readable contract)
- `ops/testnet/healthcheck.json` (the canonical `system_health` body)
- `ops/docker/chain-testnet-entrypoint.sh` (the persistent
  entrypoint the public endpoints run on)
- `docker-compose.testnet.yml` (the compose profile that wires the
  entrypoint + the one-shot subnet-owner init)
- `crates/myosu-chain/node/src/chain_spec/testnet.rs` (the
  `test_finney` spec the public endpoints serve)
- `ops/solver_promotion.yaml` (the `promotable_games` ledger)
- `tests/e2e/public_testnet_manifest.sh` (the drift guard)
- `.github/workflows/ci.yml` (the `public-testnet-manifest` CI job)

A drift between any two of these is a W-01 follow-up finding.

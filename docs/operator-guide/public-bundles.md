# Public policy bundles (W-02)

The W-02 "Public policy bundles" surface lets an external operator or
agent discover, fetch, and verify a published Myosu solver policy
bundle without going through the chain RPC, a wallet, or a token. The
surface is byte-stable: the canonical on-disk triple is reproducible
from a clean checkout, and the verifier rejects any drift in the
canonical triple (the `tests/e2e/public_bundles_manifest.sh` harness is
the executable drift guard, the `public-bundles-manifest` CI job is the
gate that runs the harness on every push / PR to trunk).

## What ships today

| Slug         | Tier                | Surface path                                       |
|--------------|---------------------|----------------------------------------------------|
| `liars-dice` | `promotable_local`  | `ops/bundles/liars-dice/liars-dice/` (this repo)   |

The `liars-dice` row is the only current `promotable_local` game in
`ops/solver_promotion.yaml`. The `nlhe-heads-up` row is still
`benchmarked` (blocked on external artifact supply per the PROMOTE-001
blocker note); when an external full/promotion-grade NLHE artifact
dossier is supplied, the NLHE bundle can land in
`ops/bundles/nlhe-heads-up/nlhe-heads-up/` and gain its own row in
this table. Until then, NLHE is intentionally absent from the public
bundles surface.

## URL contract (for an external operator / agent)

When a public Myosu host is in production, the canonical URL for the
Liar's Dice public bundle is

```
https://myosu.example.com/bundles/liars-dice/bundle.json
```

with the two siblings served at the same prefix:

```
https://myosu.example.com/bundles/liars-dice/benchmark-summary.json
https://myosu.example.com/bundles/liars-dice/artifact-manifest.json
```

The on-disk shape (`ops/bundles/liars-dice/liars-dice/<file>.json`) is
the `verify_promotion_outputs --slug liars-dice --outputs-dir
ops/bundles/liars-dice` convention this repo uses, so an operator can
mirror the public URL prefix to a local checkout and run the verifier
without any HTTP server.

This contract is operator-side, not a code change: this doc ships the
URL shape and the verifier roundtrip; a future row will ship the public
host.

## `provenance` field contract

An external verifier MUST see the following fields on every published
bundle (the field names are stable across the `CanonicalPolicyBundle`
serde schema in `crates/myosu-games-canonical/src/policy.rs`):

| Field                          | Required value (today)                                              |
|--------------------------------|---------------------------------------------------------------------|
| `provenance.game_slug`         | `liars-dice`                                                        |
| `provenance.engine_tier`       | `promotable_local`                                                  |
| `provenance.solver_family`     | `liars-dice-cfr`                                                    |
| `provenance.artifact_hash`     | the 64-lowercase-hex hash recorded in `artifact-manifest.json`      |
| `provenance.benchmark.passing` | `true`                                                              |
| `provenance.benchmark.metric_name` | `exact_exploitability`                                          |
| `provenance.benchmark.metric_value` | the float recorded in `benchmark-summary.json`                 |
| `provenance.benchmark.threshold` | the float recorded in `benchmark-summary.json`                    |
| `bundle_hash`                  | the 64-lowercase-hex SHA-256-derived hash of the canonical bundle bytes |

A bundle missing any of these fields, or carrying an `engine_tier`
below `promotable_local`, or carrying a non-passing benchmark, fails
the `verify_policy_bundle` roundtrip below with a `PROMOTION_GATE_FAIL`
exit and a sanitized reason line.

## `verify_policy_bundle` roundtrip

An operator who has downloaded the canonical triple (either from the
public host or by mirroring `ops/bundles/liars-dice/liars-dice/` into
their checkout) can run the canonical verifier with the same command
the `promotion_manifest_quality_gate` CI job uses:

```bash
# 1. Drop the three files into <some-dir>/liars-dice/ and point the
#    verifier at <some-dir>.
mkdir -p /tmp/myosu-public-bundles/liars-dice
curl -fsSL https://myosu.example.com/bundles/liars-dice/bundle.json \
    -o /tmp/myosu-public-bundles/liars-dice/bundle.json
curl -fsSL https://myosu.example.com/bundles/liars-dice/benchmark-summary.json \
    -o /tmp/myosu-public-bundles/liars-dice/benchmark-summary.json
curl -fsSL https://myosu.example.com/bundles/liars-dice/artifact-manifest.json \
    -o /tmp/myosu-public-bundles/liars-dice/artifact-manifest.json

# 2. Run the canonical verifier against the mirrored dir.
SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-canonical \
    --example verify_promotion_outputs -- \
    --slug liars-dice --outputs-dir /tmp/myosu-public-bundles
```

Expected stdout on a clean roundtrip:

```
PROMOTION_GATE_PASS slug=liars-dice tier=promotable_local benchmark_id=liars-dice-exact-n1024-epochs512 metric_name=exact_exploitability metric_value=0.6738117933273315 threshold=0.7 bundle_hash=b5e0af16218860d698958a292c6a05201082082d18c250158ba4a8a775fb540c
```

A non-zero exit (with one `PROMOTION_GATE_FAIL reason=<sanitized>`
line) means the canonical triple is no longer the bundle the verifier
expects — the bundle's `artifact_hash` does not match the dossier, the
benchmark is non-passing, or one of the three files is missing. Do
NOT load the bundle into production; the fail-closed behavior is the
proof that a future drift cannot silently regress the surface.

## Public-bundle read-only contract

The public-bundle surface is read-only by construction: it ships JSON
files and a verifier, no chain RPC, no wallet, no token. The operator
or agent who fetches the canonical triple and runs the verifier
exercises a public read surface; no Myosu state is mutated, no
authority is delegated, and no solver action is sampled (the verifier
checks the bundle's content and emits a single `PROMOTION_GATE_PASS` /
`PROMOTION_GATE_FAIL` line). Operator registration, miner axons, and
solver sampling are chain-side surfaces that the public-bundles
doc does not authorize or cover.

## Scope boundary

Publish the Liar's Dice bundle only. Do NOT publish a public host
(operator's job, not a code change). Do NOT change the bundle
builder, the verifier, the example, the `liars-dice` tier in
`ops/solver_promotion.yaml`, the F-005 Bitino code, or the
SEC-001 / CI-SEC-001 / DX-001 surfaces. Do NOT attempt to publish a
full-encoder NLHE bundle (still blocked on external artifact supply
per the F-002 / PROMOTE-001 blocker notes; out of scope by design).
Do NOT generalize the bundle publisher to all `promotable_local`
rows; `liars-dice` is the only current row, so defer generalization
until a second `promotable_local` row lands.

## Cross-references

- `ops/bundles/liars-dice/README.md` — the canonical-triple layout, the
  pinned hashes, the byte-for-byte reproduction command
- `tests/e2e/public_bundles_manifest.sh` — the 5-sub-check proof
  harness
- `.github/workflows/ci.yml` `public-bundles-manifest` job — the CI
  gate that runs the harness on every push / PR to trunk
- `docs/operator-guide/public-testnet.md` — the sibling public-testnet
  surface (the W-01 public contract for the chain RPC)
- `docs/operator-guide/agent-api.md` — the W-03 read-only solver
  surface (the sibling JSON-in / line-out public contract)

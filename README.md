# 묘수 myosu

Myosu is a decentralized game-solving chain for imperfect-information games.
Miners produce strategy, validators score it, the chain coordinates emissions,
and gameplay exposes the result to humans and agents through the same surface.

The fastest way to orient yourself is [OS.md](OS.md). It is the current
operating-system document for the repo.

## Prerequisites

Install these before running the repo from a fresh checkout:

- Stable Rust toolchain with edition 2024 support (`rust-toolchain.toml` pins
  `stable` and expects `cargo`, `rustfmt`, and `clippy`)
- WASM targets used by the current chain proof paths:

  ```bash
  rustup target add wasm32v1-none wasm32-unknown-unknown
  ```

- `protoc` / `protobuf-compiler` for Substrate and chain-related builds

## Quick Verify

The fastest meaningful green path in this repo is:

```bash
cargo test -p myosu-games-kuhn --quiet
```

That exercises the shared game-engine stack without requiring the chain,
operator bundle, or runtime build surfaces.

## Developer Path

Use this path when you want to confirm the repo is healthy and explore the
local gameplay surface before touching operator tooling.

The consolidated critical-caveat and first-success contributor guide lives at
[`docs/developer-quickstart.md`](docs/developer-quickstart.md). The executable
gate `tests/e2e/developer_quickstart.sh` enforces the four-step fastest
first-success path; the `developer-quickstart` CI job runs it on every push
and pull request.

Current low-friction proofs:

```bash
cargo test -p myosu-games-kuhn --quiet
SKIP_WASM_BUILD=1 cargo run -p myosu-play --quiet -- --smoke-test
printf 'quit\n' | SKIP_WASM_BUILD=1 cargo run -p myosu-play --quiet -- pipe
```

Current top-level reference docs:

- [OS.md](OS.md) for live doctrine, the current operator loop, and stage-0
  meaning
- [SPEC.md](SPEC.md) for durable repo decisions
- [INVARIANTS.md](INVARIANTS.md) for non-negotiable constraints
- [SECURITY.md](SECURITY.md) for vulnerability disclosure and response guidance
- [CHANGELOG.md](CHANGELOG.md) for operator-facing release history and the
  current `0.1.0` baseline
- [docs/execution-playbooks/README.md](docs/execution-playbooks/README.md) for
  current execution playbooks
- [docs/execution-playbooks/local-advisor.md](docs/execution-playbooks/local-advisor.md)
  for the current local advisor and pipe-mode workflow
- [genesis/plans/001-master-plan.md](genesis/plans/001-master-plan.md) for the
  active plan stack

## Operator Path

Use this path when you want the repo-owned key, bundle, miner, validator, and
devnet flow instead of the developer-only proofs above.

Start with the maintained operator guide:

- [docs/operator-guide/quickstart.md](docs/operator-guide/quickstart.md) for
  the zero-to-running operator path using the current key, bundle, miner, and
  validator surfaces
- [docs/operator-guide/architecture.md](docs/operator-guide/architecture.md) for
  the operator-facing mental model of how chain, miner, validator, gameplay,
  and keys fit together
- [docs/operator-guide/troubleshooting.md](docs/operator-guide/troubleshooting.md)
  for known failure modes and fixes
- [docs/operator-guide/upgrading.md](docs/operator-guide/upgrading.md) for the
  current operator upgrade process, breaking-change communication contract, and
  rollback posture
- [plans/001-master-plan.md](plans/001-master-plan.md) for the active solver
  promotion and Bitino-integration plan
- [fabro/programs/myosu-bootstrap.yaml](fabro/programs/myosu-bootstrap.yaml)
  for the current Raspberry bootstrap program
- [docs/execution-playbooks/README.md](docs/execution-playbooks/README.md) for
  current execution playbooks
- [docs/execution-playbooks/operator-network.md](docs/execution-playbooks/operator-network.md)
  for the current named-network and key-surface playbook

### Public testnet

The operator path also publishes a reference public testnet contract
so an external operator or agent can discover the public Myosu
testnet without reading prose. The contract is a single
machine-readable manifest plus an operator-facing doc:

- [docs/operator-guide/public-testnet.md](docs/operator-guide/public-testnet.md)
  for the public WS/HTTP RPC contract, the `test_finney` chain
  spec, the four authority URIs, the public subnet (`7`), the
  public promotable games, and the `system_health` healthcheck
  body the public endpoints are guaranteed to return
- [ops/testnet/manifest.yaml](ops/testnet/manifest.yaml) for the
  machine-readable public contract (the file an external agent
  `curl`s)
- [ops/testnet/healthcheck.json](ops/testnet/healthcheck.json) for
  the canonical `system_health` response body an operator can diff
  the live response against
- `bash tests/e2e/public_testnet_manifest.sh` and the
  `public-testnet-manifest` CI job for the executable contract
  drift guard

### Public policy bundles

The Liar's Dice `promotable_local` policy bundle is also
published on-disk as a public artifact an external operator or
agent can fetch, verify, and feed straight into
`verify_policy_bundle` without going through the chain RPC, a
wallet, or a token. The canonical on-disk triple is byte-stable,
the `bundle_hash` field is the canonical-hash computed by
`compute_bundle_hash` over the bundle's fields (NOT over the
JSON file bytes — `serde_json` key order is non-canonical), and
the verifier rejects any drift in the canonical triple. This is
the W-02 "Public policy bundles" surface that mirrors the W-01
public-testnet contract for the solver-artifact half of the
operator story:

- [docs/operator-guide/public-bundles.md](docs/operator-guide/public-bundles.md)
  for the Liar's Dice bundle URL contract, the `provenance`
  field contract, the `verify_policy_bundle` roundtrip, and the
  read-only-by-construction guarantee
- [ops/bundles/liars-dice/](ops/bundles/liars-dice/) for the
  canonical on-disk triple (`bundle.json` /
  `benchmark-summary.json` / `artifact-manifest.json`) an
  external agent `curl`s
- `bash tests/e2e/public_bundles_manifest.sh` and the
  `public-bundles-manifest` CI job for the executable
  byte-stability + verifier-roundtrip drift guard

### Agent / operator read-only solver surface

External agents and humans can also dispatch a read-only
recommendation for every `benchmarked` portfolio game through a
single JSON-in / line-out binary, no chain RPC, no wallet, no
token required. The contract and supported-slug table are pinned
here:

- [docs/operator-guide/agent-api.md](docs/operator-guide/agent-api.md)
  for the `myosu-solver-read` binary contract, the input JSON
  shape, the `SOLVER_READ` / `SOLVER_READ_FAIL` line protocol,
  the supported game slugs (every `benchmarked` portfolio game),
  and the read-only-by-construction guarantee
- `bash tests/e2e/solver_read.sh` and the `solver-read` CI job
  for the executable line-protocol and supported-slug drift guard

The two dedicated-solver games (`nlhe-heads-up`, `liars-dice`) are
explicitly rejected by the portfolio binary and have their own
read-only surface:

- [docs/operator-guide/agent-api.md#dedicated-solver-games-liars-dice-nlhe-heads-up](docs/operator-guide/agent-api.md)
  for the `myosu-solver-read-dedicated` binary contract, the
  checkpoint + encoder-dir input shape, the dedicated-game
  `SOLVER_READ` line protocol, and the W-07 proof harness
- `bash tests/e2e/solver_read_dedicated.sh` and the
  `solver-read-dedicated` CI job for the executable drift guard

### Validator scoring observability

Every bounded validator scoring pass emits one grep-friendly
`VALIDATOR_SCORING_METRIC` line so an operator can scrape the
per-run latency (`elapsed_ms`) and L1-distance distribution
(`mean_l1` / `p50_l1` / `p99_l1`) into a CSV without going
through the chain RPC. The line protocol is byte-stable across
hosts (the metric's `percentile` helper uses nearest-rank, the
constructor rejects non-finite L1 distances, and the `f64`
rendering is `%.6` deterministic) so the INV-003 determinism
invariant carries through to the metric.

- [docs/operator-guide/observability.md](docs/operator-guide/observability.md)
  for the `VALIDATOR_SCORING_METRIC` line protocol, the
  field contract, the nearest-rank percentile algorithm, the
  byte-stability guarantee, the failure-mode table, and a
  `grep` / `awk` pipeline an operator can run on the validator's
  stderr to scrape the metric into a CSV
- `bash tests/e2e/validator_scoring_metric.sh` and the
  `validator-scoring-metric` CI job for the executable
  line-protocol and metric-construction drift guard

## Current Runnable Truth

These are the currently proven local surfaces:

- a stripped chain that authors blocks and serves the game-solver RPC surface
- miner and validator binaries that participate in the local stage-0 loop
- a gameplay surface in `myosu-play` with smoke-test and pipe modes
- a second-game proof with Liar's Dice
- a node-owned local loop proving poker and Liar's Dice coexist as distinct
  subnets on one local chain

What this repo does not yet claim as a first-class operator product:

- production deployment
- public multi-node network operations
- polished hosted miner or validator operations
- a broad web product surface

## Broader Proof Commands

When you need stronger local regression coverage than the quick verify:

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

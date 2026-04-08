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
- [docs/execution-playbooks/operator-network.md](docs/execution-playbooks/operator-network.md)
  for the current named-network and key-surface playbook

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

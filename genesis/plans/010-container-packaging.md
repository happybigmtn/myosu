# 010: Container Packaging

## Objective

Produce Docker images for `myosu-chain`, `myosu-miner`, and `myosu-validator`
that an operator can pull and run without installing the Rust toolchain.

## Context

Currently, running any myosu binary requires:
- Rust toolchain with nightly features
- `wasm32-unknown-unknown` or `wasm32v1-none` target
- `protobuf-compiler` system package
- Initial compilation time of 10+ minutes

Container images eliminate this friction for operators who want to run
nodes, miners, or validators without becoming Rust developers.

## Acceptance Criteria

- A `Dockerfile` (multi-stage build) that:
  - Uses a Rust builder stage with cached dependencies
  - Produces a minimal runtime image (~100MB target) for each binary
  - Supports build args for target binary selection
  - Includes the chain spec files in the chain image
- A `docker-compose.yml` for local three-node devnet:
  - Three chain nodes with authority keys
  - Pre-configured ports and peer discovery
  - Health checks for block production
- Images are buildable with `docker build` (no external CI dependency)
- `docker compose up` produces a running three-node devnet within 60 seconds
- A smoke test script that verifies the containerized devnet:
  - Blocks are produced
  - Finality advances
  - RPC endpoints are accessible from host

## Verification

```bash
# Build images
docker build --build-arg BINARY=myosu-chain -t myosu-chain .
docker build --build-arg BINARY=myosu-miner -t myosu-miner .
docker build --build-arg BINARY=myosu-validator -t myosu-validator .

# Start devnet
docker compose up -d

# Verify
bash tests/e2e/docker_smoke.sh

# Cleanup
docker compose down -v
```

## Dependencies

- 009 (decision gate) -- must pass before packaging
- 006 (multi-node devnet) -- compose file reuses the multi-node configuration

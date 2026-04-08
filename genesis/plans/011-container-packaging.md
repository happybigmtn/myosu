# 011 — Container Packaging

## Objective

Create Docker images for the three operator-facing binaries (myosu-chain, myosu-miner, myosu-validator) and a docker-compose file that stands up a local devnet with miner and validator. This is the single highest-impact operator experience improvement.

## Context

Currently, operators must compile from source with Rust nightly, WASM target, and protoc. Cold compilation takes 10-30 minutes. There is no containerized path.

The `check_operator_network_fresh_machine.sh` script already proves the full operator flow inside `ubuntu:22.04`, so the compilation and runtime requirements are well-understood.

**This plan is implementation-oriented but feasibility is unresolved for the WASM build step inside Docker.** The runtime WASM build requires ~8GB RAM and substantial disk. Multi-stage builds with cached layers are recommended but untested in this repo.

## Acceptance Criteria

- A `Dockerfile` exists that builds `myosu-chain`, `myosu-miner`, and `myosu-validator` from source
- Multi-stage build: builder stage compiles, runtime stage contains only binaries + chain specs
- `docker build -t myosu-chain .` succeeds (may take 15+ minutes on first build)
- A `docker-compose.yml` exists that starts:
  - 1 chain node (single authority, `--dev` or devnet spec)
  - 1 miner (registers, trains zero iterations, serves)
  - 1 validator (registers, stakes, scores, submits weights)
- `docker compose up` produces logs showing all three components completing their lifecycle
- Container images are under 500MB each (excluding build cache)
- No secrets baked into images (keys generated at runtime)

## Verification

```bash
# Build images
docker build -t myosu-chain -f Dockerfile.chain .
docker build -t myosu-miner -f Dockerfile.miner .

# Compose up
docker compose up --abort-on-container-exit
# Logs should show: chain block production, miner registration, validator scoring

# Image sizes
docker images myosu-chain --format '{{.Size}}'
docker images myosu-miner --format '{{.Size}}'
```

**Research gate within this plan:** If the WASM build step exceeds 16GB RAM or 30 minutes inside Docker, consider pre-building the runtime WASM and including it as a build artifact rather than building inside Docker. Document the decision.

## Dependencies

- Plan 010 (Phase 2 gate) — ensure the binaries being containerized are in final form

# 011: Operator Documentation

## Objective

Produce a tested quickstart guide that takes a new operator from zero to a
running local devnet with miner and validator in under 30 minutes (excluding
build time). The guide must work on a fresh Ubuntu 24.04 or macOS machine.

## Context

The current `docs/operator-guide/quickstart.md` exists but references `fabro`
and `raspberry` tools that a new operator will not have. The proof commands
in `README.md` are accurate but scattered. What is missing is a single
linear path from "I cloned the repo" to "I see blocks, strategy, and scores."

## Acceptance Criteria

- `docs/operator-guide/quickstart.md` is rewritten to cover:
  - Prerequisites (Rust toolchain, wasm target, protoc)
  - Building all binaries
  - Starting a local devnet node
  - Creating operator keys
  - Running a miner with bounded training
  - Running a validator with scoring
  - Playing a hand against trained strategy
  - Each step has a concrete expected-output block
- A new `docs/operator-guide/multi-node.md` covers:
  - Starting a three-node devnet with docker-compose
  - Verifying block production and finality
  - Registering miners and validators across nodes
- A test script `tests/quickstart_smoke.sh` that:
  - Follows the quickstart steps programmatically
  - Asserts each step produces the documented output
  - Completes within 5 minutes (excluding initial build)
- The quickstart does not reference `fabro`, `raspberry`, or any
  tool not installable via standard package managers

## Verification

```bash
# Quickstart smoke test
bash tests/quickstart_smoke.sh

# Manual review: have someone unfamiliar with the repo follow the guide
```

## Dependencies

- 010 (container packaging) -- docker-compose guide in multi-node doc
- The quickstart can be started before 010 (local devnet path is independent)

# Specification: Miner Subsystem

## Objective

Describe the miner binary (`myosu-miner`), its MCCFR training pipeline, strategy serving over HTTP, axon registration on-chain, and key management integration.

## Evidence Status

### Verified (code-grounded)

- Miner crate: `crates/myosu-miner/` with source files in `src/`:
  - `axon.rs` (20.2K): Miner registration, HTTP endpoint setup, key management.
  - `training.rs` (15.3K): MCCFR strategy computation, checkpoint management.
  - `strategy.rs` (17.3K): Strategy serving, request handling.
  - `chain.rs` (2.9K): Blockchain interaction (balance checks, staking).
  - `cli.rs` (6.0K): Command-line argument parsing.
- The miner depends on:
  - `myosu-games-poker` for the NLHE game engine.
  - `myosu-games` for `GameConfig`, `StrategyQuery`, `StrategyResponse` types.
  - `myosu-chain-client` for RPC calls to the chain.
  - `myosu-keys` for key creation/import/management.
  - `rbp-nlhe` from the robopoker fork (`happybigmtn/robopoker`) for the MCCFR solver.
- The miner registers on subnet 7 (game-solver subnet) by submitting a hotkey registration extrinsic.
- Strategy training uses MCCFR (Monte Carlo Counterfactual Regret Minimization) with checkpointed artifacts.
- The miner serves strategy via an HTTP endpoint (the "axon") that validators query.
- INV-004 (`INVARIANTS.md:43-57`): The miner crate (`myosu-miner`) must have no direct dependency on `myosu-play`. Enforcement: `cargo tree` dependency check.
- INV-006 (`INVARIANTS.md:75-88`): The robopoker fork tracks v1.0.0 as baseline. The workspace `Cargo.toml` pins the fork revision. Core MCCFR algorithm changes require review.
- The miner binary is tested via: `SKIP_WASM_BUILD=1 cargo test -p myosu-miner --quiet` (`README.md:124`).
- The E2E smoke test (`stage0_local_loop.rs`) verifies: miner registration (`alice_uid`), miner emission (`alice_miner_emission`), and miner incentive (`alice_miner_incentive`) are all non-zero.
- Workspace member: `crates/myosu-miner` (`Cargo.toml:10`).

### Recommendations (intended future direction)

- Plan 008 (test gap closure) identifies HTTP axon security as a test gap: malformed requests, oversized payloads, and connection flooding are untested.
- Plan 009 (miner quality benchmark) notes that the current validator self-scores the miner checkpoint at always 1.0 — an independent exploitability-based benchmark surface is needed.
- Plan 011 (container packaging) recommends Docker images for `myosu-miner`.

### Hypotheses / Unresolved

- The exact MCCFR iteration count for stage-0 training is configured at runtime but the default is not documented in the crate.
- Checkpoint format and storage location are implementation details in `training.rs` — no stability guarantee or versioning scheme is documented.
- Whether the HTTP axon validates request payloads or trusts all incoming queries.

## Acceptance Criteria

- `cargo check -p myosu-miner` succeeds
- `cargo test -p myosu-miner --quiet` passes (with `SKIP_WASM_BUILD=1`)
- The miner has no direct dependency on `myosu-play` (INV-004): `cargo tree -p myosu-miner` shows no path to `myosu-play`
- After registration on a local devnet, the miner appears as a neuron on subnet 7
- After training, the miner produces a non-empty strategy checkpoint
- The HTTP axon responds to `StrategyQuery` requests with valid `StrategyResponse` payloads
- In the E2E smoke test, `alice_miner_emission > 0` and `alice_miner_incentive > 0`

## Verification

```bash
# Compile check
SKIP_WASM_BUILD=1 cargo check -p myosu-miner

# Unit tests
SKIP_WASM_BUILD=1 cargo test -p myosu-miner --quiet

# INV-004: no dependency on myosu-play
cargo tree -p myosu-miner 2>/dev/null | grep -c myosu-play
# Expected: 0

# INV-006: robopoker fork pin present
grep 'happybigmtn/robopoker' Cargo.toml

# E2E proof (full loop)
SKIP_WASM_BUILD=1 cargo test -p myosu-chain --test stage0_local_loop --quiet
```

## Open Questions

- What is the default MCCFR iteration count for stage-0 training?
- Is checkpoint format versioned? What happens when the miner binary is upgraded with incompatible checkpoint format?
- Does the axon HTTP server have request size limits or rate limiting?
- What is the miner's behavior when the chain is unreachable (retry policy, backoff)?

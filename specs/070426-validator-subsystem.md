# Specification: Validator Subsystem

## Objective

Describe the validator binary (`myosu-validator`), its deterministic exploitability scoring, weight submission to the chain, and the invariants that guarantee scoring reproducibility.

## Evidence Status

### Verified (code-grounded)

- Validator crate: `crates/myosu-validator/` with source files in `src/`:
  - `validation.rs` (25.5K): Deterministic exploitability scoring, weight computation.
  - `chain.rs` (5.7K): Weight submission, stake management.
  - `cli.rs` (5.8K): Configuration and argument parsing.
- The validator depends on:
  - `myosu-games` for trait types (`StrategyQuery`, `StrategyResponse`, `GameConfig`).
  - `myosu-games-poker` for the NLHE evaluation engine.
  - `myosu-chain-client` for RPC interaction.
  - `myosu-keys` for key management.
- INV-003 (`INVARIANTS.md:33-41`): For any given game state and strategy profile, the exploitability score computed by any validator must produce the same result within epsilon < 1e-6. Violation is severity S0 — emissions freeze until determinism is restored.
- Determinism enforcement mechanisms (from INVARIANTS.md):
  - Deterministic PRNG seeding from game state.
  - Canonical game state serialization.
  - Exact-arithmetic exploitability computation where feasible.
- The validator registers on-chain, acquires a validator permit (stake requirement is netuid-specific), then:
  1. Queries miner HTTP endpoints for strategies.
  2. Runs deterministic MCCFR evaluation (blueprint bot vs trained strategy).
  3. Computes exploitability score (deviation from Nash equilibrium).
  4. Submits weights to chain (stake-weighted influence on emissions).
- The E2E smoke test (`stage0_local_loop.rs`) verifies: `bob_has_validator_permit == true`, `bob_weights` is non-empty, `bob_validator_dividend > 0`.
- The validator binary is tested via: `SKIP_WASM_BUILD=1 cargo test -p myosu-validator --quiet` (`README.md:124`).
- Workspace member: `crates/myosu-validator` (`Cargo.toml:11`).
- The E2E test captures per-game summaries for both poker and Liar's Dice subnets, confirming the validator operates across both game types.

### Recommendations (intended future direction)

- Plan 009 (miner quality benchmark) recommends an independent exploitability benchmark surface that does not depend on the validator self-scoring path (current always-1.0 result for local checkpoint).
- Plan 008 (test gap closure) identifies cross-game scoring fairness as untested — do validators produce comparable quality metrics across different game types?

### Hypotheses / Unresolved

- The exact epsilon threshold (< 1e-6) is stated in INVARIANTS.md but the enforcement mechanism in code (tolerance check, assertion, or alert) has not been verified from `validation.rs`.
- Whether the validator uses `substrate-fixed` types (U96F32) for scoring or standard `f64` — the invariant mentions "exact-arithmetic where feasible."
- The stake requirement for acquiring a validator permit on subnet 7 devnet is not documented.

## Acceptance Criteria

- `cargo check -p myosu-validator` succeeds
- `cargo test -p myosu-validator --quiet` passes (with `SKIP_WASM_BUILD=1`)
- Two independent validators given the same miner strategy and game state produce scores within epsilon (< 1e-6) of each other (INV-003)
- After registration and staking on a local devnet, the validator acquires a validator permit
- Weight submission succeeds and is reflected in the next epoch pass
- In the E2E smoke test, `bob_has_validator_permit == true`, `bob_weights` is non-empty, `bob_validator_dividend > 0`

## Verification

```bash
# Compile check
SKIP_WASM_BUILD=1 cargo check -p myosu-validator

# Unit tests
SKIP_WASM_BUILD=1 cargo test -p myosu-validator --quiet

# E2E proof (full loop with validator participation)
SKIP_WASM_BUILD=1 cargo test -p myosu-chain --test stage0_local_loop --quiet

# Cross-game validation: both poker and liars_dice summaries present
# (verified by the stage0_local_loop test parsing both game prefixes)
```

## Open Questions

- What is the minimum stake to acquire a validator permit on subnet 7?
- Does `validation.rs` use `f64` or fixed-point types for scoring computation?
- How does the validator handle miner endpoints that are unreachable or return invalid responses?
- Is there a timeout for miner queries during the validation pass?
- What happens if all miners produce identical (or trivially bad) strategies — does the validator produce meaningful weight differentiation?

# Specification: Operator Stack — Miner and Validator Binaries

## Objective

Define the current state and intended direction of the two operator binaries (`myosu-miner` and `myosu-validator`) that form the off-chain compute and scoring layer. This spec covers their CLI surface, bootstrap sequences, training/scoring logic, and the constraints that govern their operation.

## Evidence Status

### Verified facts (code-grounded)

#### Miner (`myosu-miner`)

- Bootstrap sequence: probe → register → serve_axon → train → strategy → http_axon — `crates/myosu-miner/src/main.rs:1-93`
- CLI arguments include `--chain` (default `ws://127.0.0.1:9944`), `--subnet`, `--key`/`--key-config-dir`, `--port` (default 8080), `--game`, `--train-iterations`, `--encoder-dir`, `--checkpoint` — `crates/myosu-miner/src/cli.rs`
- `GameSelection` enum mirrors `GameType` with variants: Poker, Kuhn, LiarsDice + 20 portfolio games — `crates/myosu-miner/src/cli.rs:6-51`
- `PORTFOLIO_SELECTIONS` constant lists all 20 portfolio game CLI values — `crates/myosu-miner/src/cli.rs:53-75`
- Training produces `TrainingRunReport` with game, checkpoint_path, epochs, exploitability, quality_summary — `crates/myosu-miner/src/training.rs:66-73`
- `LIARS_DICE_SOLVER_TREES = 1 << 10` — `crates/myosu-miner/src/training.rs`
- Positive-iteration poker training rejects encoder bundles whose summary has `postflop_complete = false` before starting the solver — `crates/myosu-miner/src/training.rs:223-241`
- One-shot strategy response generation from checkpoint — `crates/myosu-miner/src/strategy.rs`
- HTTP axon server serves live `/strategy` queries for poker — `crates/myosu-miner/src/axon.rs`
- Key password sourced from environment variable (default `MYOSU_KEY_PASSWORD`) — `crates/myosu-miner/src/cli.rs`
- `MinerBootstrapError` enum covers chain probe, registration, axon, training, strategy failures — `crates/myosu-miner/src/main.rs`
- Machine-readable report prefixes for each bootstrap stage — `crates/myosu-miner/src/` (startup_report, registration_report, axon_report, training_report, strategy_report)

#### Validator (`myosu-validator`)

- Bootstrap sequence: probe → register → enable_subtoken → stake → validate → submit_weights — `crates/myosu-validator/src/main.rs:1-98`
- CLI arguments include `--chain`, `--subnet`, `--key`/`--key-config-dir`, `--game`, `--register`, `--enable-subtoken`, `--submit-weights`, `--stake-amount`, `--weight-hotkey`, `--checkpoint`, `--response` — `crates/myosu-validator/src/cli.rs`
- `GameSelection` enum identical to miner — `crates/myosu-validator/src/cli.rs`
- Validation computes `exact_match`, `L1 distance`, and `score` against validator-loaded checkpoint — `crates/myosu-validator/src/validation.rs`
- `ValidationReport` includes game, action_count, exact_match, l1_distance, score — `crates/myosu-validator/src/validation.rs`
- Validator loads a local checkpoint, recomputes the expected response, computes union L1 distance, and scores with `1.0 / (1.0 + l1_distance)`; `exact_match=true` / `score=1.0` is a same-checkpoint self-consistency proof, not an independent quality oracle — `crates/myosu-validator/src/validation.rs:225-345`, `crates/myosu-validator/src/validation.rs:617-625`, `crates/myosu-validator/src/validation.rs:755-805`
- Weight submission records validator UID, target UID, version key — `crates/myosu-validator/src/chain.rs`
- Stake enforcement via `--stake-amount` with `ValidatorPermitBootstrapReport` — `crates/myosu-validator/src/cli.rs`
- Instant timing for weight submission — `crates/myosu-validator/src/main.rs`

#### Shared

- Both binaries import game crates directly; no coupling through chain — architecture pattern
- Both use `myosu-chain-client` for RPC communication — `crates/myosu-chain-client/`
- Both use `myosu-keys` for operator key loading — `crates/myosu-keys/`
- INV-004: `myosu-play` has no dependency on `myosu-miner`, and vice versa — CI enforced at `.github/workflows/ci.yml:145-158`
- Both binaries print text report blocks directly and expose no `--json`/`--output-format` CLI flag in their current Clap definitions — `crates/myosu-miner/src/main.rs:36-77`, `crates/myosu-validator/src/main.rs:34-83`, `crates/myosu-miner/src/cli.rs`, `crates/myosu-validator/src/cli.rs`

### Recommendations (intended system)

- Validator should evolve from checkpoint self-consistency to independent quality oracle — plans 004-005 establish the dossier/benchmark path for NLHE
- Typed JSON output mode should be added for automation workflows
- Miner should support serving all game types via HTTP (currently only poker has HTTP `/strategy`) — implied by multi-game architecture

### Hypotheses / unresolved questions

- Whether miner HTTP axon will serve portfolio games or only dedicated solvers is open
- Whether validator scoring will use scenario-pack benchmarks independently from the self-consistency check is proposed (plan 005) but not implemented
- F-007 (miner convergence gate) remains blocked on truthful quality benchmark surface

## Acceptance Criteria

- Miner probe reports chain health, RPC methods, and neuron state without errors
- Miner registration produces a UID for the hotkey on the target subnet
- Miner training with `--train-iterations N` produces a checkpoint file and reports epoch count + exploitability
- Miner rejects positive-iteration poker training when artifacts have `postflop_complete = false`
- Miner serves one-shot strategy response from loaded checkpoint
- Miner HTTP axon server binds to configured port and responds to `/strategy` queries
- Validator probe and registration complete without errors
- Validator scoring against a miner checkpoint produces `exact_match = true` when using identical checkpoint
- Validator scoring produces `score < 1.0` when miner response diverges from validator reference
- Validator weight submission writes on-chain weight for target neuron
- Both binaries produce machine-readable report prefixes for each bootstrap stage
- Both binaries load operator keys from `--key` file or `--key-config-dir` with password from environment
- `cargo tree -p myosu-play --edges normal` does not contain `myosu-miner`
- `cargo tree -p myosu-miner --edges normal` does not contain `myosu-play`

## Verification

```bash
# Compile check (no chain dependency needed)
SKIP_WASM_BUILD=1 cargo check -p myosu-miner -p myosu-validator

# Unit tests
SKIP_WASM_BUILD=1 cargo test -p myosu-miner --quiet
SKIP_WASM_BUILD=1 cargo test -p myosu-validator --quiet

# INV-004 boundary check
play_tree="$(SKIP_WASM_BUILD=1 cargo tree -p myosu-play --edges normal)"
echo "$play_tree" | grep -q 'myosu-miner' && echo "FAIL: INV-004" || echo "PASS: INV-004"
miner_tree="$(SKIP_WASM_BUILD=1 cargo tree -p myosu-miner --edges normal)"
echo "$miner_tree" | grep -q 'myosu-play' && echo "FAIL: INV-004" || echo "PASS: INV-004"

# Clippy
SKIP_WASM_BUILD=1 cargo clippy -p myosu-miner -p myosu-validator -- -D warnings

# E2E validator determinism proof (requires running chain)
bash tests/e2e/validator_determinism.sh
```

## Open Questions

1. **Independent validator quality:** Current validator is a determinism proof (same-checkpoint self-check). Plans 004-005 propose the dossier/benchmark path, but the transition from self-consistency to independent quality measurement is unspecified at the code level. What's the integration point in `validation.rs`?
2. **Multi-game HTTP axon:** Only poker has HTTP `/strategy` serving via axon. Should each game type register its own endpoint? What's the URL scheme for portfolio games?
3. **Typed output mode:** Current binaries print text report blocks but have no structured output mode. Which report types should be structured first? Training reports and validation reports are the highest-value targets for automation.
4. **F-007 convergence gate:** Miner convergence benchmarking (how many iterations produce an acceptable strategy) is blocked on truthful quality benchmark surface. This blocks any automated "train until good enough" loop.
5. **Key password UX:** Environment variable `MYOSU_KEY_PASSWORD` is the only password source. Should stdin/tty prompt be supported for interactive use?
6. **Stake amount defaults:** Validator `--stake-amount` has no documented default. What's the minimum viable stake for stage-0 local proofs?

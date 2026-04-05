# Specification: Validation Surface

Date: 2026-04-05

---

## Objective

Define the behavior, interfaces, and invariants of the `myosu-validator` binary
-- the off-chain scoring surface that measures miner strategy quality and
submits weights to the Myosu chain.

The validator loads a solver checkpoint, recomputes the expected strategy for a
given game state, measures L1 distance between expected and observed probability
distributions, and converts that distance into a score. Optionally, it submits
the resulting weight vector on-chain via commit-reveal.

---

## Evidence Status

| Claim | Source | Status |
|-------|--------|--------|
| Binary is `myosu-validator` | `crates/myosu-validator/src/main.rs` | Verified |
| Modules: `chain`, `cli`, `validation` | `crates/myosu-validator/src/lib.rs` | Verified |
| Score formula: `1.0 / (1.0 + l1_distance)` | `validation.rs:370` | Verified |
| Exact match: `l1_distance < f64::EPSILON` yields score 1.0 | `validation.rs:265` | Verified |
| L1 distance covers both expected and observed action sets | `validation.rs:345-367` | Verified |
| Observed response validated via `is_valid()` before scoring | `validation.rs:256-259` | Verified |
| Multi-game: Poker + Liar's Dice via `GameSelection` enum | `cli.rs:7-10` | Verified |
| Liar's Dice does not require `--encoder-dir` | `validation.rs:157` | Verified |
| Weight submission via `ensure_weights_set` | `chain.rs:144-160`, `main.rs:65-84` | Verified |
| INV-003 unit test: `inv_003_determinism` | `validation.rs:512-550` | Verified |
| E2E test: `validator_determinism.sh` | `tests/e2e/validator_determinism.sh` | Verified |
| Reports: startup, registration, subtoken, permit, validation, weight | `lib.rs:12-96` | Verified |

---

## Architecture

### Bootstrap Sequence

The `main` function executes a linear pipeline. Each step is gated by CLI flags
and runs only when the operator requests it.

```
1. Parse CLI, resolve operator key
2. probe_chain          -- connect, fetch health + neuron list
3. ensure_registered    -- burned registration (if --register)
4. ensure_subtoken      -- enable subnet staking (if --enable-subtoken)
5. ensure_permit_ready  -- add stake, wait for permit (if --stake-amount)
6. score_response       -- local scoring pass (if --query-file + --response-file)
7. ensure_weights_set   -- submit weight vector (if --submit-weights)
```

Each step produces a structured plain-text report printed to stdout with a
fixed prefix (`VALIDATOR`, `REGISTRATION`, `SUBTOKEN`, `PERMIT`, `VALIDATION`,
`WEIGHTS`). This format is machine-parseable by the E2E harness.

### Scoring Pipeline

```
query bytes ─────► decode ─► solver.answer(query) ─► expected distribution
                                                          │
response bytes ──► decode ──► is_valid() check ──────► observed distribution
                                                          │
                                    l1_distance(expected, observed)
                                              │
                                    score = 1.0 / (1.0 + l1)
```

The L1 distance computation is symmetric: it iterates over actions present in
the expected distribution, then over actions present only in the observed
distribution (assigning zero expected probability for novel actions). This
handles the case where expected and observed action sets differ.

### Operator Key Resolution

The CLI requires exactly one key source via a clap `ArgGroup`:

- `--key <uri>`: Direct seed/URI (e.g., `//Bob`).
- `--key-config-dir <path>`: Encrypted key file loaded via
  `myosu_keys::load_active_secret_uri_from_env`, decrypted with the password
  from the environment variable named by `--key-password-env` (default:
  `MYOSU_KEY_PASSWORD`).

---

## CLI Interface

| Flag | Type | Required | Default | Purpose |
|------|------|----------|---------|---------|
| `--chain` | String | No | `ws://127.0.0.1:9944` | WebSocket RPC endpoint |
| `--subnet` | u16 | Yes | -- | Target subnet |
| `--key` | String | One of key/key-config-dir | -- | Operator hotkey URI |
| `--key-config-dir` | Path | One of key/key-config-dir | -- | Encrypted key directory |
| `--key-password-env` | String | No | `MYOSU_KEY_PASSWORD` | Env var for key decryption |
| `--register` | bool | No | false | Register hotkey on subnet |
| `--enable-subtoken` | bool | No | false | Enable subnet staking |
| `--submit-weights` | bool | No | false | Submit weights after scoring |
| `--stake-amount` | u64 | No | -- | Minimum stake for permit |
| `--weight-hotkey` | String | No | self | Target hotkey for weight vector |
| `--game` | Enum | No | poker | Game contract (poker, liars-dice) |
| `--encoder-dir` | Path | Poker only | -- | Encoder artifact directory |
| `--checkpoint` | Path | With query/response | -- | Solver checkpoint |
| `--query-file` | Path | With response-file | -- | Wire-encoded strategy query |
| `--response-file` | Path | With query-file | -- | Wire-encoded miner response |

Constraints:
- `--query-file` and `--response-file` must be provided together.
- `--encoder-dir` is required when `--game poker`, optional for `liars-dice`.
- `--checkpoint` is required when query/response files are provided.
- `--weight-hotkey` defaults to the validator's own key (self-weight bootstrap).

---

## Scoring Formula

**L1 distance** between two probability distributions over game actions:

```
l1 = sum_{a in expected} |P_expected(a) - P_observed(a)|
   + sum_{a in observed \ expected} |P_observed(a)|
```

**Score**:

```
score = 1.0 / (1.0 + max(l1, 0.0))
```

Properties:
- Range: (0.0, 1.0]
- Exact match (l1 < f64::EPSILON): score = 1.0
- Completely wrong distribution (l1 = 2.0, maximum for two valid distributions
  over the same action set): score ~= 0.333
- Monotonically decreasing in l1

**Response validity**: The observed distribution must pass `is_valid()` (sums
to ~1.0, no negative probabilities). Invalid responses produce
`ValidationError::InvalidResponse` and are not scored.

---

## Invariants

### INV-003: Game Verification Determinism

Given identical checkpoint, encoder, and query artifacts, any two validator
processes produce identical scores within floating-point epsilon (< 1e-6).

Enforcement mechanisms:
- Deterministic PRNG seeding in solver training
- Canonical serialization of game state and strategy distributions
- No floating-point non-determinism in the scoring path (pure f64 arithmetic)

Verification:
- Unit test `inv_003_determinism` (`validation.rs:512`): Two scoring passes
  with the same solver, query, and response produce `assert_eq!` identical
  `ValidationReport` structs.
- E2E test `validator_determinism.sh`: Two independent validator processes
  (different keys, same checkpoint/query/response) produce scores within
  configurable epsilon (default 1e-6).

---

## Error Taxonomy

All validation errors are typed via `thiserror` and propagate through
`ValidatorBootstrapError`.

| Variant | Trigger | Operator Action |
|---------|---------|-----------------|
| `IncompleteArtifactPair` | Only one of query/response provided | Supply both paths |
| `MissingEncoderDir` | Poker game without `--encoder-dir` | Add encoder path |
| `MissingCheckpoint` | Query/response without `--checkpoint` | Add checkpoint path |
| `Encoder` | Encoder directory fails to load | Check encoder artifacts |
| `ReadQuery` / `ReadResponse` | File I/O failure | Check file paths and permissions |
| `DecodeQuery` / `DecodeResponse` | Wire format parse failure | Regenerate artifacts |
| `Solver` / `LiarsDiceSolver` | Checkpoint load or query failure | Check checkpoint compatibility |
| `InvalidResponse` | Response is not a valid distribution | Investigate miner output |

---

## Testing

### Unit Tests (cargo test -p myosu-validator)

| Test | What it covers |
|------|----------------|
| `validation_plan_requires_both_artifact_paths` | CLI validation: incomplete pair rejected |
| `exact_match_scores_one` | Poker: self-scored response yields score 1.0 |
| `three_action_mismatch_uses_game_agnostic_normalization` | Poker: mismatched distribution scores correctly |
| `inv_003_determinism` | Two identical scoring passes produce identical reports |
| `liars_dice_validation_plan_does_not_require_encoder_dir` | Liar's Dice: encoder-dir optional |
| `liars_dice_exact_match_scores_one` | Liar's Dice: self-scored response yields score 1.0 |
| `cli_parses_stage_zero_flags` | Full CLI flag parsing |
| `cli_parses_config_backed_key_source` | Config-dir key source parsing |
| `startup_report_includes_probe_summary` | Report format: startup |
| `registration_report_includes_registration_summary` | Report format: registration |
| `subtoken_bootstrap_report_includes_enablement_summary` | Report format: subtoken |
| `permit_bootstrap_report_includes_stake_summary` | Report format: permit |
| `validation_report_includes_score_summary` | Report format: validation |
| `weight_submission_report_includes_submission_summary` | Report format: weights |

### E2E Test (validator_determinism.sh)

Spins up a local devnet, registers a subnet, trains a miner, and runs two
independent validator scoring passes. Asserts:
- Both validators produce identical `action_count`, `exact_match`,
  `expected_action`, and `observed_action`.
- `l1_distance` and `score` agree within configurable epsilon (default 1e-6).

CI command: `SKIP_WASM_BUILD=1 cargo test -p myosu-validator --quiet`

---

## Acceptance Criteria

- **Scoring correctness**: `score_response` returns `score = 1.0` when the
   observed response exactly matches the validator's local expectation, for
   both poker and Liar's Dice.

- **Scoring monotonicity**: Higher L1 distance produces strictly lower score.
   The formula `1.0 / (1.0 + l1)` is monotonically decreasing for l1 >= 0.

- **Invalid response rejection**: Responses that fail `is_valid()` return
   `ValidationError::InvalidResponse` and are never scored.

- **Determinism (INV-003)**: Two validator processes with identical artifacts
   produce scores that differ by less than 1e-6. Verified by unit test and E2E
   test.

- **CLI completeness**: All flags documented above are parsed and enforced.
   Missing required combinations produce typed errors.

- **Report stability**: Each pipeline step produces a structured report with
   a fixed prefix and key=value lines, parseable by the E2E harness.

- **Weight submission**: When `--submit-weights` is set, the validator submits
   a weight vector targeting `--weight-hotkey` (or self) via the chain client.

- **Multi-game support**: The `--game` flag selects between poker and
   liars-dice. Each game loads its own solver type, encoder requirements, and
   wire codec.

---

## Verification

| Criterion | Method | Artifact |
|-----------|--------|----------|
| Scoring correctness | Unit tests: `exact_match_scores_one`, `liars_dice_exact_match_scores_one` | `validation.rs` |
| Scoring monotonicity | Unit test: `three_action_mismatch_uses_game_agnostic_normalization` | `validation.rs` |
| Invalid response rejection | Code path: `is_valid()` check before scoring | `validation.rs:256` |
| Determinism | Unit test: `inv_003_determinism`; E2E: `validator_determinism.sh` | Both |
| CLI completeness | Unit tests: `cli_parses_stage_zero_flags`, `cli_parses_config_backed_key_source` | `cli.rs` |
| Report stability | Unit tests: all `*_report_includes_*` tests | `lib.rs` |
| Weight submission | E2E test with `--submit-weights` flag | `validator_determinism.sh` |
| Multi-game | Unit test: `liars_dice_validation_plan_does_not_require_encoder_dir` | `validation.rs` |

---

## Open Questions

1. **L1 tolerance threshold**: The current scoring formula treats any nonzero
   L1 distance as a degraded score. Should there be a configurable "good
   enough" threshold below which the score is clamped to 1.0, to accommodate
   minor floating-point drift across different hardware?

2. **Batch scoring**: The current binary scores one query/response pair per
   invocation. Should validators score multiple miners per epoch in a single
   pass, or is the one-at-a-time model sufficient for stage-0?

3. **Validator permit acquisition**: The `ensure_validator_permit_ready` flow
   adds stake and waits up to 30 minutes for a permit. What determines the
   top-k validator set size per subnet, and how should operators reason about
   the minimum stake required?

4. **Weight submission mode**: The chain client's `ensure_weights_set` handles
   commit-reveal internally. The `WeightSubmissionReport.mode` field reports
   the submission path used (`set_weights` vs commit-reveal v2). Should the
   validator surface expose control over which mode is used, or is automatic
   selection sufficient?

5. **Cross-node emission agreement (Plan 008)**: INV-003 is verified at the
   unit and E2E level. Network-scale verification (multiple physical nodes
   agreeing on emissions) is deferred to the multi-node devnet plan.

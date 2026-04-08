# Specification: Nemesis Audit Findings and Hardening Requirements

**Repository**: `/home/r/coding/myosu`
**Audit date**: 2026-04-08
**Audit scope**: chain pallet, miner, validator, gameplay, game engine crates
**Repo snapshot**: trunk @ 4e0b37f
**Auditor**: initial pass (Phase 0 + Pass 1 + Pass 2)

---

## Discovery Log

| Finding | Severity | Type | Pass |
|---------|----------|------|------|
| NEM-001 | S1 | State / Business Logic | State |
| NEM-002 | S2 | Latent Risk | State |
| NEM-003 | S2 | Latent Risk | Feynman |
| NEM-004 | S2 | Invariant / Architecture | State |
| NEM-005 | S1 | Invariant | State |
| NEM-006 | S2 | Missing Test Coverage | State |
| NEM-007 | S2 | Missing Test Coverage | State |
| NEM-008 | S2 | Invariant / Test | State |

---

## Finding NEM-001: Validator score is meaningless in stage-0 happy-path E2E

### Affected surfaces
- `crates/myosu-validator/src/validation.rs:score_response()` — the validator's scoring function
- `tests/e2e/local_loop.sh` — the primary E2E integration script
- `tests/e2e/validator_determinism.sh` — the INV-003 wiring proof
- `AGENTS.md` "Current Operator Loop" — documented proof surfaces

### Triggering scenario
1. Miner binary runs with `--train-iterations 0 --query-file Q --response-file R`
2. Checkpoint is produced at iteration 0 (empty strategy)
3. `local_loop.sh` / `validator_determinism.sh` feed the miner-produced checkpoint back into the validator with the same query/response
4. Validator loads the checkpoint and computes `solver.answer(query)` — which is the identical policy that produced `R`
5. L1 distance = 0.0, score = 1.0

**Result**: The E2E "happy path" reports `score=1.0 exact_match=true` on a zero-iteration empty profile. This is not an accurate quality signal — it is a self-consistency proof, not a quality proof.

### Invariant or assumption that breaks
- INV-003 (Game Verification Determinism): The proof tests determinism correctly, but the **happy path does not test meaningful quality**. A miner can score 1.0 with a zero-iteration checkpoint because both miner and validator use the same self-referential checkpoint.
- The AGENTS.md operator loop entry: "`tests/e2e/local_loop.sh` ... and the repo-owned happy-path harnesses pass the miner checkpoint straight into that validator path" confirms the self-referential setup is intentional but undocumented as a limitation.

### Why this matters now
A validator submitting weights for a zero-iteration miner would receive the same high weights as a 512-iteration miner on the current E2E harness. The Yuma Consensus reward distribution is therefore not tested against a genuine quality gradient. The stage-0 emission flow could distribute rewards to garbage strategies with no automated detection.

The Liar's Dice quality benchmark (`quality_benchmark` test) is the exception — it uses `exact_exploitability()` on independently trained solvers, which is a legitimate quality ladder. The Poker validator scoring lacks this path.

### Discovery path
**State** — traced the `score_response()` code path from E2E shell into the validation function, identified that `solver.answer(query)` is computed from the same checkpoint the miner produced, cross-referenced against `tests/e2e/local_loop.sh` and AGENTS.md "current stage-0 happy path" entry.

---

## Finding NEM-002: Stage0NoopSwap has unbounded max_price — slippage protection is absent

### Affected surfaces
- `crates/myosu-chain/runtime/src/lib.rs:99` — `Stage0NoopSwap` with `max_price() = u64::MAX`
- `crates/myosu-chain/pallets/game-solver/src/staking/add_stake.rs:196` — `stage0_swap_tao_for_alpha(..., price_limit=stage0_max_price(), ...)`
- `crates/myosu-chain/pallets/game-solver/src/staking/remove_stake.rs:410` — `stage0_swap_alpha_for_tao(..., price_limit, ...)`
- `crates/myosu-chain/pallets/game-solver/src/coinbase/run_coinbase.rs:141` — `swap_tao_for_alpha(..., stage0_max_price(), ...)` in `inject_and_maybe_swap`
- `crates/myosu-chain/runtime/src/lib.rs:94-97` — documented as intentional with the caveat: "must be revisited before any mainnet-style token economics ship"

### Triggering scenario
1. Stage-0: swaps are identity (1:1, no pool state), so the unbounded price limit causes no damage
2. A future runtime replaces `Stage0NoopSwap` with a real AMM but **forgets** to replace the price limit with a real bound
3. Staking and emission code path still calls `stage0_max_price()` — the new AMM receives an unbounded order and accepts any execution price
4. Attacker exploits unbounded slippage tolerance on large stake/swap operations

### Invariant or assumption that breaks
- INV-005 (Plan And Land Coherence): the documented caveat exists but there is **no CI gate, no compile-time assertion, and no migration guard** that forces a real price limit before any AMM substitution. A future developer could replace `Stage0NoopSwap` with a real `ConstantProductSwap` and the `SwapInterface::max_price()` still returns `u64::MAX`.
- The `runtime/src/lib.rs` comment acknowledges the risk but the acknowledgment is prose-only — no enforcement mechanism.

### Why this matters now
No active damage in stage-0 (identity stub has no price sensitivity). But this is a **hardened time-bomb**: it will compile silently with a real AMM and expose real token amounts to slippage exploitation. The risk grows as the system approaches any token-economics spec work (RES-001, F-003).

### Discovery path
**Feynman** — traced the `add_stake` flow through swap calls to `stage0_swap_tao_for_alpha`, noted that `stage0_max_price()` returns `u64::MAX`, compared against the `runtime/src/lib.rs` comment acknowledging this as intentional-with-catch.

---

## Finding NEM-003: `close_integer_emission_split` floor arithmetic is verified for stage-0 owner/server/validator split but not for epoch individual distribution

### Affected surfaces
- `crates/myosu-chain/pallets/game-solver/src/coinbase/run_coinbase.rs:47-65` — `close_integer_emission_split`
- `crates/myosu-chain/pallets/game-solver/src/tests/stage_0_flow.rs:599-633` — `stage_0_coinbase_truncation_dust_is_closed_exactly_sweep`
- `crates/myosu-chain/pallets/game-solver/src/coinbase/run_coinbase.rs:296-305` — pending accumulation
- `crates/myosu-chain/pallets/game-solver/src/epoch/run_epoch.rs` — epoch emission computation with `I32F32` → `I96F32` → `u64` truncation

### Triggering scenario
The `stage_0_coinbase_truncation_dust_is_closed_exactly_sweep` test sweeps the **coinbase block-emission → owner/server/validator split** and confirms zero drift across representative block counts and emission values. This is correct and well-tested.

However, the test computes:
```
owner_cut_amount = block_emission * owner_cut
server_amount = (block_emission - owner_cut_amount) * 0.5
validator_amount = block_emission - owner_cut_amount - server_amount
```

Each step floors the intermediate `U96F32` → `u64` conversion. The sweep proves the sum of the three floored parts equals the original `block_emission * blocks`. This is solid.

**What the sweep does NOT cover**: the per-UID **epoch emission** values computed in `run_epoch.rs` lines 490-530. These go through:
```
server_emission = normalized_server_emission * rao_emission
                 → I32F32 * I96F32 → I96F32 → saturating_to_num::<u64>
```

The `stage_0_coinbase_truncation_dust_is_closed_exactly_sweep` only exercises the coinbase split, not the Yuma epoch per-UID allocation. When the epoch runs, `server_emission[i]` and `validator_emission[i]` are independently floored per UID. There is no test verifying that `sum(server_emission_per_uid) + sum(validator_emission_per_uid) == total_epoch_emission - owner_cut` within some epsilon.

### Invariant or assumption that breaks
- The emission accounting reads: "sum(distributions) == block_emission * epochs (no-ship gate)"
- If per-UID epoch truncation causes the sum of per-UID emissions to fall below the accrued epoch budget, the gap silently leaks rao. The gap would be bounded by `n_validators + n_miners` rao per epoch, but the gate contract is not confirmed.

### Why this matters now
The coinbase split is proven closed, but the epoch distribution accumulation path is not. At large epoch emission values (I96F32 scale), the per-UID floor rounding could accumulate a non-trivial drift. The stage-0 emission sweep test does not cover this path.

### Discovery path
**Feynman** — traced `stage_0_coinbase_truncation_dust_is_closed_exactly_sweep` to understand its scope, then cross-referenced the `close_integer_emission_split` function body against the epoch per-UID emission computation in `run_epoch.rs`. Identified that the sweep covers the coinbase split but not the epoch per-UID floor.

---

## Finding NEM-004: `epoch_mechanism` and `epoch_dense_mechanism` are both retained — no active proof they remain in sync

### Affected surfaces
- `crates/myosu-chain/pallets/game-solver/src/epoch/run_epoch.rs:130-190` — `epoch` (legacy wrapper)
- `crates/myosu-chain/pallets/game-solver/src/epoch/run_epoch.rs:193-230` — `epoch_dense` (legacy wrapper)
- `crates/myosu-chain/pallets/game-solver/src/epoch/run_epoch.rs:232-1621` — `epoch_mechanism` (sparse)
- `crates/myosu-chain/pallets/game-solver/src/epoch/run_epoch.rs` — `epoch_dense_mechanism` (dense)
- `crates/myosu-chain/pallets/game-solver/src/tests/stage_0_flow.rs:323` — `epoch_dense` called in test
- `tests/stage_0_flow.rs:340` — `epoch_mechanism` implicitly via `run_coinbase`

### Triggering scenario
1. `epoch_dense_mechanism` uses dense `[n, n]` matrix math; `epoch_mechanism` uses sparse `Vec<Vec<(u16, I32F32)>>` math
2. Both paths are retained in the codebase
3. `dense_sparse_epoch_paths_produce_identical_state` test (mentioned in AGENTS.md) is the parity proof — but this is behind `legacy-subtensor-tests` feature flag, not the default build
4. Any future change that modifies one path without updating the other risks silent divergence

### Invariant or assumption that breaks
- INV-003: If the two paths diverge in any scenario, validators using one path and miners/chain using another would disagree on scores
- INV-001: Both paths compute Yuma Consensus output; divergence means "trusted structured outcome" is path-dependent

### Why this matters now
The `dense_sparse_epoch_paths_produce_identical_state` test is the INV-003 parity proof for the Yuma math. It is behind a feature flag. The AGENTS.md references it as a proof that "the retained `epoch_dense()` compatibility path" is parity-proven. But this proof does not run in the default build, meaning CI cannot catch a future regression.

### Discovery path
**State** — cataloged both epoch functions in `run_epoch.rs`, identified the feature-flag dependency of the parity test, cross-referenced against AGENTS.md "dense_sparse_epoch_paths_produce_identical_state" entry.

---

## Finding NEM-005: `INV-004` solver-gameplay boundary enforced only by cargo tree test

### Affected surfaces
- `crates/myosu-play/tests/invariants.rs:inv_004_solver_and_gameplay_bins_do_not_depend_on_each_other` — `cargo tree` check
- INV-004: "no direct imports between miner and play crates"
- CI workflow: `INV-004 solver-gameplay dependency boundary` job

### Triggering scenario
1. A developer adds a shared `myosu-chain-client` re-export to `myosu-miner`'s public API
2. `myosu-play` imports from `myosu-miner` transitively through `myosu-chain-client`
3. `cargo tree` test passes because the dependency is indirect (play → chain-client → miner)
4. INV-004's stated intent ("a gameplay bug must not corrupt training data") is violated by the transitive import, but the test does not catch it

**Actual scenario observed**: `myosu-play` and `myosu-miner` are both in the workspace, both depend on `myosu-games` and `myosu-games-poker`. The `cargo tree` check only prevents direct `myosu-play → myosu-miner` or `myosu-miner → myosu-play` edges. Shared dependencies are permitted.

The current boundary is clean (`cargo tree -p myosu-play` confirms no `myosu-miner` in tree). But the enforcement is **textual pattern-matching on `cargo tree` output**, not a structural ownership boundary in `Cargo.toml` or a workspace feature flag.

### Invariant or assumption that breaks
- INV-004: "no direct imports" is enforced but the anti-pattern of transitive import through shared deps is not caught
- INV-001: "truthful unattended execution" requires that gameplay and solver remain independent

### Why this matters now
Currently clean, but the enforcement is brittle. If the workspace structure changes (e.g., `myosu-chain-client` re-exports miner-specific types), the textual check could pass while the semantic boundary is violated.

### Discovery path
**State** — ran `cargo tree -p myosu-play` locally (confirmed no miner), reviewed the test implementation in `myosu-play/tests/invariants.rs`, traced the enforcement mechanism to textual `cargo tree` pattern-matching.

---

## Finding NEM-006: No integration test for miner HTTP axon + validator HTTP scoring over real chain

### Affected surfaces
- `crates/myosu-miner/src/axon.rs` — HTTP axon server
- `crates/myosu-validator/src/chain.rs` — miner discovery from chain
- `crates/myosu-chain/pallets/game-solver/src/serving.rs` — `serve_axon` extrinsic
- `tests/e2e/local_loop.sh` — uses file-based query/response, not HTTP

### Triggering scenario
1. Miner binary calls `serve_axon` and starts HTTP server
2. Validator binary queries `all_axons` RPC, gets miner's IP/port
3. Validator makes HTTP POST to miner `/strategy` with a wire-encoded `StrategyQuery`
4. Miner responds with a `StrategyResponse`
5. Validator scores the response and submits weights

**This end-to-end flow has no automated test.** The `local_loop.sh` uses file-based query/response (`--query-file`, `--response-file`) which bypasses HTTP entirely. The HTTP axon + scoring path is only exercised manually or in docker compose integration.

### Why this matters now
The HTTP wire protocol between validator and miner is a critical trust boundary. Any codec mismatch, HTTP header issue, or path error in the axon server would not be caught by the file-based E2E harness. The protocol is defined by `myosu-games-poker/src/wire.rs` and implemented by `myosu-miner/src/strategy.rs`, but neither has integration test coverage through the HTTP layer.

### Discovery path
**State** — cataloged the miner-validator HTTP interaction paths, compared against `local_loop.sh`, confirmed no HTTP integration test exists in the crate test suites.

---

## Finding NEM-007: `try_state` emission accounting guard is set to 1 rao but the underlying assertion uses `saturating_sub` without bounds checking

### Affected surfaces
- `crates/myosu-chain/pallets/game-solver/src/utils/try_state.rs:20-30`
- `crates/myosu-chain/pallets/game-solver/src/lib.rs:75` — `TOTAL_ISSUANCE_TRY_STATE_ALERT_DELTA = 1`
- `crates/myosu-chain/pallets/game-solver/src/coinbase/run_coinbase.rs:47-65` — `close_integer_emission_split`

### Triggering scenario
1. `try_state` computes `expected_total_issuance` from the coinbase emission accounting
2. Compares against `TotalIssuance::<T>::get()`
3. Asserts `diff <= TOTAL_ISSUANCE_TRY_STATE_ALERT_DELTA` (1 rao)
4. If `expected_total_issuance > live_total_issuance` due to an off-by-one in `close_integer_emission_split`, `diff` could be larger than 1 rao
5. The test `stage_0_try_state_delta_matches_exact_accounting_policy` confirms zero drift on the coinbase split, so this is currently safe

**However**: the `try_state` assertion has no explicit upper-bound enforcement on `expected_total_issuance`. If a future refactor of the epoch emission accumulation accidentally introduces an off-by-N truncation (not caught by the stage-0 coinbase sweep), the `try_state` would trigger but the diagnostic message would only say "diff > 1 rao" without revealing the magnitude. There is no fuzzing or property test that validates `expected_total_issuance` is always within a small epsilon of the live value.

### Why this matters now
The 1-rao guard is the emission accounting no-ship gate. If it triggers on mainnet, the node would panic on `on_initialize`. The diagnostic quality is poor: the guard tells you there is a problem but not the root cause. This is a hardening requirement, not an active bug.

### Discovery path
**State** — read `utils/try_state.rs`, cross-referenced `TOTAL_ISSUANCE_TRY_STATE_ALERT_DELTA`, traced the `close_integer_emission_split` caller in `run_coinbase.rs`.

---

## Finding NEM-008: Robopoker fork has no formal MCCFR algorithm review requirement in the change process

### Affected surfaces
- `docs/robopoker-fork-changelog.md` — current fork tracking doc
- `INV-006` in `INVARIANTS.md` — "Core MCCFR algorithm changes require review"
- `crates/myosu-games-poker/src/solver.rs` — `PokerSolver` wrapping robopoker MCCFR
- `crates/myosu-chain/Cargo.toml` — robopoker workspace pin

### Triggering scenario
1. A developer adds a commit to the robopoker fork that modifies the MCCFR regret update formula (e.g., changes `regret += utility` to `regret = max(regret, utility)`)
2. The `check_robopoker_fork_status.sh` script runs, compares workspace pin against upstream, reports divergence counts
3. The divergence count increases by N files, but the **content of the MCCFR change is not reviewed**
4. INV-006 states "Core MCCFR algorithm changes require review" but there is no automated gate or checklist item that forces a human review of MCCFR-relevant changes

### Invariant or assumption that breaks
- INV-006: "Core MCCFR algorithm changes require review" — the policy exists but the enforcement is advisory only (CI `continue-on-error`). A silent MCCFR correctness change could ship to mainnet.

### Why this matters now
MCCFR is the core algorithm. Any correctness change to the regret update or averaging would silently change solver quality. The AGENTS.md entry "INV-006 advisory proof" confirms this is treated as advisory. For stage-0 this is acceptable (limited real-economy exposure), but the absence of a blocking review gate is a gap for future hardening.

### Discovery path
**State** — read `INVARIANTS.md`, cross-referenced against `docs/robopoker-fork-changelog.md` and `check_robopoker_fork_status.sh`, noted the `continue-on-error` CI behavior.

---

## Cross-cutting Observations

### O-1: Stage-0 emission accounting is well-hardened for the coinbase split
The `close_integer_emission_split` function explicitly floors each component and computes the validator portion as `alpha_created - sum(other_components)`, ensuring the integer remainder closes to zero. The sweep test covers representative emission values and block counts. The `try_state` 1-rao alert margin is conservative. **Verdict: emission accounting for the coinbase split is solid.**

### O-2: Substrate fixed-point library pin is critical for INV-003
`substrate_fixed` is pinned to the `encointer/substrate-fixed` fork v0.6.0. Any update to the pinned version could silently change Yuma output. The AGENTS.md entry "bit-identical Yuma output requires identical fixed-point lib" documents this dependency. No version pinning enforcement exists in `Cargo.lock` beyond the workspace entry.

### O-3: Storage item count is large but tested
The pallet carries ~80 storage items from subtensor. The stage-0 dispatch surface (8 calls) is intentionally narrow. Storage items used in stage-0 are exercised; many deprecated items (NeuronCertificate, PrometheusInfo, ChainIdentity) are present but inactive. The storage layout is not a risk but represents maintenance surface.

### O-4: No unsafe code found
The grep for `unsafe` across the pallet returned zero matches. The `unsafe_cell`, `unsafe_code` searches across all game-solver pallet source also returned zero matches. The codebase avoids `unsafe` in the pallet layer.

### O-5: The `BlocksSinceLastStep` counter is incremented for ALL subnets on every block
In `run_coinbase::drain_pending()`, the loop iterates over ALL subnets and increments `BlocksSinceLastStep` even for subnets that are not emitting. This is by design (needed for the tempo check), but the increment is unconditional and could cause storage churn at scale with many subnets. No rate limit or batching exists.

### O-6: Registry cleanup on subnet dissolution is verified
`tests/networks.rs` confirms that `PendingServerEmission`, `PendingValidatorEmission`, `PendingOwnerCut`, and `PendingRootAlphaDivs` are all cleared on `dissolve_network`. The subnet dissolution path is well-tested.

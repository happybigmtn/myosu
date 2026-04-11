# Define the Canonical Policy Bundle Contract

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds. This document must be maintained in accordance with root `PLANS.md`.

## Purpose / Big Picture

After this plan is complete, Myosu has a durable, typed, verifiable envelope for representing one solver-backed decision point. A "policy bundle" is a self-contained artifact that describes: the game identity, the decision context (public state + legal actions), the solver's mixed strategy as a probability distribution, the recommended action, the artifact provenance (solver family, checkpoint identity, benchmark summary), and a deterministic bundle hash. A companion "sampling proof" type records how a fairness draw is mapped onto that distribution so a downstream consumer (such as Bitino) can later prove why a non-deterministic house action was chosen.

An operator can verify a policy bundle locally (`verify_policy_bundle`), and given the same entropy bytes, `sample_policy_action` always produces the same sampled action. These types live in `crates/myosu-games-canonical/src/policy.rs` and are exported from the canonical crate's public API.

## Requirements Trace

- R1: `CanonicalPolicyBundle` struct with all fields specified in `plans/001-master-plan.md` Interfaces section
- R2: `CanonicalPolicySamplingProof` struct for auditable action sampling
- R3: `verify_policy_bundle()` returns Ok or typed error
- R4: `sample_policy_action()` is deterministic given identical entropy
- R5: All types derive `Serialize, Deserialize` for JSON wire format
- R6: Bundle hash is computed from canonical content, not from serialization order

## Scope Boundaries

This plan adds `policy.rs` to `crates/myosu-games-canonical/` and wires it into `lib.rs`. It does not change any existing type in the canonical crate, does not change any game solver, does not change the miner or validator, and does not create the promotion ledger (that is plan 002).

## Progress

- [ ] Create `crates/myosu-games-canonical/src/policy.rs` with all types
- [ ] Implement `verify_policy_bundle()` validation logic
- [ ] Implement `sample_policy_action()` with deterministic entropy mapping
- [ ] Implement `compute_bundle_hash()` from canonical content
- [ ] Export policy types from `crates/myosu-games-canonical/src/lib.rs`
- [ ] Add unit tests for verification (valid, malformed, empty distribution)
- [ ] Add unit tests for sampling determinism
- [ ] Add proptest for hash stability across serialization variants
- [ ] Verify `cargo test -p myosu-games-canonical --quiet` passes
- [ ] Verify `cargo clippy -p myosu-games-canonical -- -D warnings` passes

## Surprises & Discoveries

None yet.

## Decision Log

- Decision: Policy types live in `myosu-games-canonical`, not in `myosu-play` or a new crate.
  Rationale: `myosu-games-canonical` already owns canonical state snapshots, transition traces, and replay truth. Policy bundles are part of that same truth layer. `myosu-play` is a consumer surface.
  Date/Author: 2026-04-11 / master plan

- Decision: Probability distribution uses `probability_ppm: u32` (parts per million), not `f64`.
  Rationale: Integer representation avoids floating-point non-determinism in hash computation and cross-platform verification. PPM gives 6 decimal digits of precision, sufficient for game strategy distributions.
  Date/Author: 2026-04-11 / master plan

- Decision: Bundle hash uses SHA-256 over canonical byte representation, not serde JSON.
  Rationale: JSON serialization order is not guaranteed across implementations. A canonical byte representation (sorted fields, fixed encoding) ensures the hash is stable.
  Date/Author: 2026-04-11 / genesis corpus

## Outcomes & Retrospective

None yet.

## Context and Orientation

The `crates/myosu-games-canonical/` crate currently exports these relevant types:

- `CanonicalGameSpec` in `src/lib.rs` -- describes a game identity and its canonical representation
- `CanonicalStateSnapshot` in `src/lib.rs` -- a hashable game state snapshot
- `CanonicalActionSpec` in `src/lib.rs` -- legal action descriptions for a game
- `CanonicalStrategyBinding` in `src/lib.rs` -- binds a query hash to a response hash with engine tier
- `CanonicalTruthError` in `src/lib.rs` -- error type for canonical validation failures
- `canonical_hash()` in `src/lib.rs` -- SHA-256 hasher used for snapshot and binding hashes

The new `policy.rs` module will reuse `CanonicalStateSnapshot`, `CanonicalTruthError`, and `canonical_hash()`. It will not modify any existing type.

The `ResearchGame` enum used by `CanonicalPolicyBundle.game` is defined in `crates/myosu-games-portfolio/src/game.rs`. The canonical crate already depends on the portfolio crate.

## Plan of Work

Create `crates/myosu-games-canonical/src/policy.rs` with the following types:

1. `PolicyPromotionTier` enum: `Routed`, `Benchmarked`, `PromotableLocal`, `PromotableFunded`
2. `CanonicalPolicyDistributionEntry` struct: `action_id: String`, `probability_ppm: u32`
3. `CanonicalPolicyBenchmarkSummary` struct: benchmark id, metric name/value/threshold, passing bool
4. `CanonicalPolicyProvenance` struct: game slug, solver family, engine tier, artifact id/hash, benchmark summary
5. `CanonicalPolicyBundle` struct: game, decision id, canonical public state snapshot, legal action ids, distribution, recommended action, provenance, bundle hash
6. `CanonicalPolicySamplingProof` struct: bundle hash, entropy source/hash, draw u64, sampled action id
7. `compute_bundle_hash()` function: serialize canonical fields in sorted order, SHA-256 hash
8. `verify_policy_bundle()` function: check distribution sums to 1M PPM, recommended action is in distribution, all action IDs are unique, bundle hash matches recomputed hash
9. `sample_policy_action()` function: map entropy bytes to u64, scale into distribution range, select action

Then add `pub mod policy;` to `crates/myosu-games-canonical/src/lib.rs` and re-export the key types.

## Implementation Units

### Unit 1: Policy types and bundle hash

Goal: Define all structs and the hash computation.
Requirements advanced: R1, R2, R5, R6.
Dependencies: `sha2` (already in workspace), `serde` (already in workspace).
Files to create: `crates/myosu-games-canonical/src/policy.rs`.
Files to modify: `crates/myosu-games-canonical/src/lib.rs` (add module + re-exports), `crates/myosu-games-canonical/Cargo.toml` (add `sha2` if not already present).
Tests to add: Hash stability test, serialization roundtrip test.
Approach: Define all types with derive macros, implement `compute_bundle_hash` using the existing `canonical_hash` helper pattern.
Test scenarios: (a) Bundle hash is stable across repeated computation, (b) Changing any field changes the hash, (c) All types serialize/deserialize via serde_json.

### Unit 2: Verification and sampling

Goal: Implement `verify_policy_bundle` and `sample_policy_action`.
Requirements advanced: R3, R4.
Dependencies: None beyond Unit 1.
Files to modify: `crates/myosu-games-canonical/src/policy.rs`.
Tests to add: Verification accepts valid bundles, rejects malformed, sampling determinism.
Approach: Verification checks distribution sum, action uniqueness, recommended action presence, hash match. Sampling maps entropy to u64, uses cumulative distribution selection.
Test scenarios: (a) Valid bundle passes verification, (b) Distribution summing to != 1M PPM fails, (c) Duplicate action IDs fail, (d) Same entropy → same action across 100 calls, (e) Different entropy → potentially different action.

## Concrete Steps

Use the repo-root command sequence below as the execution path for this plan.

## Verification

Run the commands in the next section from the repo root. This plan is only
complete when the new policy module compiles, tests pass, and the crate stays
lint-clean under the existing workspace standards.

From the repository root:

    cargo check -p myosu-games-canonical

This should pass before any changes. After adding policy.rs:

    cargo test -p myosu-games-canonical --quiet
    cargo clippy -p myosu-games-canonical -- -D warnings
    cargo fmt -p myosu-games-canonical -- --check

Expected: all tests pass, no clippy warnings, no format drift.

## Acceptance Criteria

- `crates/myosu-games-canonical/src/policy.rs` exists and is exported publicly.
- Verification rejects malformed bundles and accepts valid ones.
- Sampling is deterministic for identical entropy input.
- Bundle hashing is stable and field-sensitive.

## Validation and Acceptance

Acceptance is behavioral. After this plan:

1. `cargo test -p myosu-games-canonical --quiet policy` runs at least 6 tests covering verification and sampling.
2. A test constructs a `CanonicalPolicyBundle` with a 3-action distribution summing to 1,000,000 PPM, verifies it, samples with fixed entropy, and asserts the sampled action ID.
3. A test constructs a bundle with distribution summing to 999,999 PPM and asserts `verify_policy_bundle` returns an error.
4. A test constructs a bundle, computes its hash, modifies one field, recomputes, and asserts the hashes differ.

## Idempotence and Recovery

Creating `policy.rs` is additive. If the file already exists from a partial run, the implementer can overwrite it safely. No migrations, no destructive operations. The existing canonical crate tests must continue to pass unchanged.

## Artifacts and Notes

Expected new file:
    crates/myosu-games-canonical/src/policy.rs

Expected modification:
    crates/myosu-games-canonical/src/lib.rs (add `pub mod policy;` and re-exports)

## Interfaces and Dependencies

In `crates/myosu-games-canonical/src/policy.rs`, define:

    pub enum PolicyPromotionTier {
        Routed,
        Benchmarked,
        PromotableLocal,
        PromotableFunded,
    }

    pub struct CanonicalPolicyDistributionEntry {
        pub action_id: String,
        pub probability_ppm: u32,
    }

    pub struct CanonicalPolicyBenchmarkSummary {
        pub benchmark_id: String,
        pub metric_name: String,
        pub metric_value: f64,
        pub threshold: f64,
        pub passing: bool,
    }

    pub struct CanonicalPolicyProvenance {
        pub game_slug: String,
        pub solver_family: String,
        pub engine_tier: String,
        pub artifact_id: String,
        pub artifact_hash: String,
        pub benchmark: CanonicalPolicyBenchmarkSummary,
    }

    pub struct CanonicalPolicyBundle {
        pub game: ResearchGame,
        pub decision_id: String,
        pub public_state: CanonicalStateSnapshot,
        pub legal_action_ids: Vec<String>,
        pub distribution: Vec<CanonicalPolicyDistributionEntry>,
        pub recommended_action_id: String,
        pub provenance: CanonicalPolicyProvenance,
        pub bundle_hash: String,
    }

    pub struct CanonicalPolicySamplingProof {
        pub bundle_hash: String,
        pub entropy_source: String,
        pub entropy_hash: String,
        pub draw_u64: u64,
        pub sampled_action_id: String,
    }

    pub fn compute_bundle_hash(bundle: &CanonicalPolicyBundle) -> String;
    pub fn verify_policy_bundle(bundle: &CanonicalPolicyBundle) -> Result<(), CanonicalTruthError>;
    pub fn sample_policy_action(
        bundle: &CanonicalPolicyBundle,
        entropy_source: &str,
        entropy_bytes: &[u8],
    ) -> Result<CanonicalPolicySamplingProof, CanonicalTruthError>;

Dependencies: `serde` (workspace), `sha2` (workspace), `myosu-games-portfolio` (already a dependency of canonical crate).

Revision note (2026-04-11 / Codex review): added explicit verification and
acceptance headings so this ExecPlan matches both the root ExecPlan standard
and the repo's current legacy plan-quality checker.

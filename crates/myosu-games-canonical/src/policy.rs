//! Canonical policy bundle types for solver promotion evidence.

use std::collections::BTreeSet;

use myosu_games::{
    CanonicalStateSnapshot, CanonicalTruthError, validate_action_id, validate_unique_action_ids,
};
use myosu_games_portfolio::ResearchGame;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

pub const TOTAL_PROBABILITY_PPM: u32 = 1_000_000;

/// Promotion tier recorded by the solver promotion ledger.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum PolicyPromotionTier {
    Routed,
    Benchmarked,
    PromotableLocal,
    PromotableFunded,
}

/// One action probability in a canonical policy distribution.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CanonicalPolicyDistributionEntry {
    pub action_id: String,
    pub probability_ppm: u32,
}

/// Benchmark result used to justify a promoted policy bundle.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CanonicalPolicyBenchmarkSummary {
    pub benchmark_id: String,
    pub metric_name: String,
    pub metric_value: f64,
    pub threshold: f64,
    pub passing: bool,
}

/// Provenance linking a policy bundle to a solver artifact and benchmark.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CanonicalPolicyProvenance {
    pub game_slug: String,
    pub solver_family: String,
    pub engine_tier: String,
    pub artifact_id: String,
    pub artifact_hash: String,
    pub benchmark: CanonicalPolicyBenchmarkSummary,
}

/// A verifier-consumable policy distribution for one canonical decision point.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
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

/// Replay evidence for deterministic policy sampling.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CanonicalPolicySamplingProof {
    pub bundle_hash: String,
    pub entropy_source: String,
    pub entropy_hash: String,
    pub draw_u64: u64,
    pub sampled_action_id: String,
}

#[derive(Serialize)]
struct CanonicalPolicyBundleHashInput<'a> {
    encoding: &'static str,
    game: ResearchGame,
    decision_id: &'a str,
    public_state: &'a CanonicalStateSnapshot,
    legal_action_ids: &'a [String],
    distribution: &'a [CanonicalPolicyDistributionEntry],
    recommended_action_id: &'a str,
    provenance: &'a CanonicalPolicyProvenance,
}

/// Compute the bundle hash over all bundle fields except `bundle_hash`.
///
/// Encoding contract: values are first converted to `serde_json::Value`, then
/// serialized as minimal JSON with object keys sorted lexicographically at
/// every depth. Arrays keep their declared order. The resulting bytes are
/// hashed with SHA-256 and returned as lowercase hex. This avoids relying on
/// map iteration order when policy artifacts are produced by different tools.
pub fn compute_bundle_hash(bundle: &CanonicalPolicyBundle) -> Result<String, CanonicalTruthError> {
    let input = CanonicalPolicyBundleHashInput {
        encoding: "myosu-policy-bundle-v1/sorted-json",
        game: bundle.game,
        decision_id: &bundle.decision_id,
        public_state: &bundle.public_state,
        legal_action_ids: &bundle.legal_action_ids,
        distribution: &bundle.distribution,
        recommended_action_id: &bundle.recommended_action_id,
        provenance: &bundle.provenance,
    };
    let bytes = canonical_json_bytes(&input)?;
    let digest = Sha256::digest(bytes);

    Ok(hex::encode(digest))
}

/// Verify the internal consistency and hash binding of a policy bundle.
pub fn verify_policy_bundle(bundle: &CanonicalPolicyBundle) -> Result<(), CanonicalTruthError> {
    if bundle.distribution.is_empty() {
        return Err(policy_error("distribution is empty"));
    }
    if bundle.legal_action_ids.is_empty() {
        return Err(policy_error("legal_action_ids is empty"));
    }
    if bundle.game.slug() != bundle.provenance.game_slug {
        return Err(policy_error("game slug does not match provenance"));
    }
    if bundle.public_state.game_id != bundle.provenance.game_slug {
        return Err(policy_error(
            "public_state game_id does not match provenance",
        ));
    }
    if !bundle.provenance.benchmark.metric_value.is_finite()
        || !bundle.provenance.benchmark.threshold.is_finite()
    {
        return Err(policy_error("benchmark values must be finite"));
    }

    validate_unique_action_ids(
        &bundle.public_state.game_id,
        &bundle.public_state.legal_actions,
    )?;

    let legal_action_ids = validate_unique_ids(
        &bundle.public_state.game_id,
        bundle.legal_action_ids.iter().map(String::as_str),
    )?;
    validate_action_id(&bundle.recommended_action_id)?;
    if !legal_action_ids.contains(bundle.recommended_action_id.as_str()) {
        return Err(policy_error("recommended action is not legal"));
    }

    let mut distribution_action_ids = BTreeSet::new();
    let mut probability_sum = 0_u64;
    for entry in &bundle.distribution {
        validate_action_id(&entry.action_id)?;
        if !distribution_action_ids.insert(entry.action_id.as_str()) {
            return Err(CanonicalTruthError::DuplicateActionId {
                game_id: bundle.public_state.game_id.clone(),
                action_id: entry.action_id.clone(),
            });
        }
        if !legal_action_ids.contains(entry.action_id.as_str()) {
            return Err(policy_error("distribution action is not legal"));
        }
        probability_sum = probability_sum
            .checked_add(u64::from(entry.probability_ppm))
            .ok_or_else(|| policy_error("probability sum overflowed"))?;
    }
    if !distribution_action_ids.contains(bundle.recommended_action_id.as_str()) {
        return Err(policy_error("recommended action is not in distribution"));
    }
    if probability_sum != u64::from(TOTAL_PROBABILITY_PPM) {
        return Err(policy_error(
            "distribution probabilities must sum to 1_000_000 ppm",
        ));
    }

    let expected = compute_bundle_hash(bundle)?;
    if expected != bundle.bundle_hash {
        return Err(CanonicalTruthError::HashMismatch {
            expected,
            found: bundle.bundle_hash.clone(),
        });
    }

    Ok(())
}

/// Deterministically sample one policy action from caller-provided entropy.
pub fn sample_policy_action(
    bundle: &CanonicalPolicyBundle,
    entropy_source: &str,
    entropy_bytes: &[u8],
) -> Result<CanonicalPolicySamplingProof, CanonicalTruthError> {
    verify_policy_bundle(bundle)?;

    let entropy_hash = hex::encode(Sha256::digest(entropy_bytes));
    let draw_u64 = draw_from_entropy(bundle, entropy_source, entropy_bytes);
    let draw_ppm = draw_u64
        .checked_rem(u64::from(TOTAL_PROBABILITY_PPM))
        .ok_or_else(|| policy_error("probability draw failed"))?;
    let draw_ppm = u32::try_from(draw_ppm)
        .map_err(|source| policy_error(format!("probability draw conversion failed: {source}")))?;

    let mut cumulative = 0_u32;
    for entry in &bundle.distribution {
        let next = cumulative
            .checked_add(entry.probability_ppm)
            .ok_or_else(|| policy_error("probability cumulative sum overflowed"))?;
        if draw_ppm < next {
            return Ok(CanonicalPolicySamplingProof {
                bundle_hash: bundle.bundle_hash.clone(),
                entropy_source: entropy_source.to_string(),
                entropy_hash,
                draw_u64,
                sampled_action_id: entry.action_id.clone(),
            });
        }
        cumulative = next;
    }

    Err(policy_error("probability draw did not select an action"))
}

fn validate_unique_ids<'a>(
    game_id: &str,
    ids: impl IntoIterator<Item = &'a str>,
) -> Result<BTreeSet<&'a str>, CanonicalTruthError> {
    let mut seen = BTreeSet::new();
    for action_id in ids {
        validate_action_id(action_id)?;
        if !seen.insert(action_id) {
            return Err(CanonicalTruthError::DuplicateActionId {
                game_id: game_id.to_string(),
                action_id: action_id.to_string(),
            });
        }
    }

    Ok(seen)
}

fn draw_from_entropy(
    bundle: &CanonicalPolicyBundle,
    entropy_source: &str,
    entropy_bytes: &[u8],
) -> u64 {
    let mut hasher = Sha256::new();
    update_length_prefixed(&mut hasher, b"myosu-policy-sampling-v1");
    update_length_prefixed(&mut hasher, bundle.bundle_hash.as_bytes());
    update_length_prefixed(&mut hasher, entropy_source.as_bytes());
    update_length_prefixed(&mut hasher, entropy_bytes);
    let digest = hasher.finalize();
    let mut draw_bytes = [0_u8; 8];
    for (target, source) in draw_bytes.iter_mut().zip(digest.iter().take(8)) {
        *target = *source;
    }

    u64::from_be_bytes(draw_bytes)
}

fn update_length_prefixed(hasher: &mut Sha256, bytes: &[u8]) {
    hasher.update(bytes.len().to_be_bytes());
    hasher.update(bytes);
}

fn canonical_json_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, CanonicalTruthError> {
    let value =
        serde_json::to_value(value).map_err(|source| CanonicalTruthError::Serialization {
            message: source.to_string(),
        })?;
    let mut bytes = Vec::new();
    write_canonical_json_value(&value, &mut bytes)?;

    Ok(bytes)
}

fn write_canonical_json_value(
    value: &Value,
    output: &mut Vec<u8>,
) -> Result<(), CanonicalTruthError> {
    match value {
        Value::Null => output.extend_from_slice(b"null"),
        Value::Bool(true) => output.extend_from_slice(b"true"),
        Value::Bool(false) => output.extend_from_slice(b"false"),
        Value::Number(number) => output.extend_from_slice(number.to_string().as_bytes()),
        Value::String(text) => write_canonical_json_string(text, output)?,
        Value::Array(values) => {
            output.push(b'[');
            let mut needs_separator = false;
            for item in values {
                if needs_separator {
                    output.push(b',');
                }
                write_canonical_json_value(item, output)?;
                needs_separator = true;
            }
            output.push(b']');
        }
        Value::Object(map) => {
            output.push(b'{');
            let mut keys: Vec<_> = map.keys().collect();
            keys.sort();
            let mut needs_separator = false;
            for key in keys {
                if needs_separator {
                    output.push(b',');
                }
                write_canonical_json_string(key, output)?;
                output.push(b':');
                let item = map
                    .get(key)
                    .ok_or_else(|| policy_error("canonical JSON key disappeared"))?;
                write_canonical_json_value(item, output)?;
                needs_separator = true;
            }
            output.push(b'}');
        }
    }

    Ok(())
}

fn write_canonical_json_string(
    text: &str,
    output: &mut Vec<u8>,
) -> Result<(), CanonicalTruthError> {
    serde_json::to_writer(&mut *output, text).map_err(|source| CanonicalTruthError::Serialization {
        message: source.to_string(),
    })
}

fn policy_error(message: impl Into<String>) -> CanonicalTruthError {
    CanonicalTruthError::Serialization {
        message: format!("invalid canonical policy bundle: {}", message.into()),
    }
}

#[cfg(test)]
mod tests {
    use myosu_games::{CanonicalActionSpec, CanonicalStateSnapshot};
    use serde_json::{Map, Value, json};

    use super::*;

    #[test]
    fn ppm_sum_validation_accepts_exact_and_rejects_neighbors() {
        let exact = bridge_bundle(vec![
            distribution_entry("bridge.play.follow-suit", 700_000),
            distribution_entry("bridge.play.no-trump", 300_000),
        ]);
        assert!(verify_policy_bundle(&exact).is_ok());

        let below = bridge_bundle(vec![
            distribution_entry("bridge.play.follow-suit", 700_000),
            distribution_entry("bridge.play.no-trump", 299_999),
        ]);
        assert!(verify_policy_bundle(&below).is_err());

        let above = bridge_bundle(vec![
            distribution_entry("bridge.play.follow-suit", 700_000),
            distribution_entry("bridge.play.no-trump", 300_001),
        ]);
        assert!(verify_policy_bundle(&above).is_err());
    }

    #[test]
    fn empty_distribution_is_rejected() {
        let bundle = bridge_bundle(Vec::new());

        assert!(verify_policy_bundle(&bundle).is_err());
    }

    #[test]
    fn sampling_is_deterministic_for_fixed_entropy() {
        let bundle = weighted_bridge_bundle();
        let first = sample(&bundle, b"fixed-seed");
        let second = sample(&bundle, b"fixed-seed");

        assert_eq!(first, second);
        assert_eq!(first.bundle_hash, bundle.bundle_hash);
    }

    #[test]
    fn sampling_tracks_ppm_weights_over_many_draws() {
        let bundle = weighted_bridge_bundle();
        let mut follow_suit = 0_usize;
        let mut no_trump = 0_usize;

        for draw in 0_u64..10_000 {
            let entropy = draw.to_be_bytes();
            let proof = sample(&bundle, &entropy);
            match proof.sampled_action_id.as_str() {
                "bridge.play.follow-suit" => increment(&mut follow_suit),
                "bridge.play.no-trump" => increment(&mut no_trump),
                action_id => panic!("unexpected sampled action {action_id}"),
            }
        }

        assert!((6_500..7_500).contains(&follow_suit));
        assert!((2_500..3_500).contains(&no_trump));
        assert!(follow_suit > no_trump);
    }

    #[test]
    fn bundle_hash_is_deterministic() {
        let bundle = weighted_bridge_bundle();
        let first = compute_hash(&bundle);
        let second = compute_hash(&bundle);

        assert_eq!(first, second);
        assert_eq!(first.len(), 64);
    }

    #[test]
    fn bundle_hash_is_stable_when_unordered_public_state_keys_change() {
        let left = bridge_bundle_with_public_state(object_value([("b", 2), ("a", 1)]));
        let right = bridge_bundle_with_public_state(object_value([("a", 1), ("b", 2)]));

        assert_eq!(compute_hash(&left), compute_hash(&right));
    }

    #[test]
    fn verify_then_sample_roundtrip_succeeds() {
        let bundle = weighted_bridge_bundle();

        assert!(verify_policy_bundle(&bundle).is_ok());
        let proof = sample(&bundle, b"roundtrip-seed");
        assert_eq!(proof.bundle_hash, bundle.bundle_hash);
        assert!(
            bundle
                .legal_action_ids
                .iter()
                .any(|action_id| action_id == &proof.sampled_action_id)
        );
    }

    fn weighted_bridge_bundle() -> CanonicalPolicyBundle {
        bridge_bundle(vec![
            distribution_entry("bridge.play.follow-suit", 700_000),
            distribution_entry("bridge.play.no-trump", 300_000),
        ])
    }

    fn bridge_bundle(distribution: Vec<CanonicalPolicyDistributionEntry>) -> CanonicalPolicyBundle {
        bridge_bundle_with_public_state_and_distribution(json!({"contract": "3nt"}), distribution)
    }

    fn bridge_bundle_with_public_state(public_state: Value) -> CanonicalPolicyBundle {
        bridge_bundle_with_public_state_and_distribution(
            public_state,
            vec![
                distribution_entry("bridge.play.follow-suit", 700_000),
                distribution_entry("bridge.play.no-trump", 300_000),
            ],
        )
    }

    fn bridge_bundle_with_public_state_and_distribution(
        public_state: Value,
        distribution: Vec<CanonicalPolicyDistributionEntry>,
    ) -> CanonicalPolicyBundle {
        let legal_action_ids = vec![
            "bridge.play.follow-suit".to_string(),
            "bridge.play.no-trump".to_string(),
        ];
        let snapshot = CanonicalStateSnapshot {
            game_id: "bridge".to_string(),
            ruleset_version: 1,
            trace_id: "bridge:policy-test".to_string(),
            phase: "opening-lead".to_string(),
            actor: Some(0),
            public_state,
            private_state_commitments: vec!["hidden-state-commitment".to_string()],
            legal_actions: vec![
                action_spec("bridge.play.follow-suit", "follow-suit"),
                action_spec("bridge.play.no-trump", "no-trump"),
            ],
            terminal: false,
        };
        let mut bundle = CanonicalPolicyBundle {
            game: ResearchGame::Bridge,
            decision_id: "bridge:policy-test".to_string(),
            public_state: snapshot,
            legal_action_ids,
            distribution,
            recommended_action_id: "bridge.play.follow-suit".to_string(),
            provenance: CanonicalPolicyProvenance {
                game_slug: "bridge".to_string(),
                solver_family: "PIMC plus double-dummy-inspired policy".to_string(),
                engine_tier: "benchmarked".to_string(),
                artifact_id: "bridge-policy-test".to_string(),
                artifact_hash: "abc123".to_string(),
                benchmark: CanonicalPolicyBenchmarkSummary {
                    benchmark_id: "bridge-policy-test".to_string(),
                    metric_name: "win_rate".to_string(),
                    metric_value: 0.73,
                    threshold: 0.50,
                    passing: true,
                },
            },
            bundle_hash: String::new(),
        };
        bundle.bundle_hash = compute_hash(&bundle);

        bundle
    }

    fn action_spec(action_id: &str, display_label: &str) -> CanonicalActionSpec {
        CanonicalActionSpec {
            game_id: "bridge".to_string(),
            action_id: action_id.to_string(),
            family: "trick_taking".to_string(),
            display_label: display_label.to_string(),
            legal_phases: vec!["opening-lead".to_string()],
            params_schema: json!({"type": "object", "additionalProperties": false}),
        }
    }

    fn distribution_entry(
        action_id: &str,
        probability_ppm: u32,
    ) -> CanonicalPolicyDistributionEntry {
        CanonicalPolicyDistributionEntry {
            action_id: action_id.to_string(),
            probability_ppm,
        }
    }

    fn sample(bundle: &CanonicalPolicyBundle, entropy: &[u8]) -> CanonicalPolicySamplingProof {
        match sample_policy_action(bundle, "unit-test", entropy) {
            Ok(proof) => proof,
            Err(error) => panic!("policy sample should succeed: {error}"),
        }
    }

    fn compute_hash(bundle: &CanonicalPolicyBundle) -> String {
        match compute_bundle_hash(bundle) {
            Ok(hash) => hash,
            Err(error) => panic!("bundle hash should compute: {error}"),
        }
    }

    fn increment(value: &mut usize) {
        *value = match value.checked_add(1) {
            Some(next) => next,
            None => panic!("counter overflowed"),
        };
    }

    fn object_value<const N: usize>(entries: [(&str, i64); N]) -> Value {
        let mut map = Map::new();
        for (key, value) in entries {
            map.insert(key.to_string(), json!(value));
        }

        Value::Object(map)
    }
}

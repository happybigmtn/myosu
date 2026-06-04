//! Canonical policy bundle construction for NLHE promotion.
//!
//! Parallel to `myosu-games-liars-dice/src/policy_bundle.rs`. Given a
//! `PokerSolver` and a pinned `NlheArtifactDossier`, this module produces a
//! verified `CanonicalPolicyBundle` whose provenance points back at the
//! dossier hash and benchmark summary. The builder fails closed when the
//! dossier records a non-passing benchmark — the negative test surface
//! the NLHE promotion plan (genesis/plans/005) calls for, and the gap that
//! previously kept `code_reported_bundle_support(NlheHeadsUp)` at
//! `benchmarked` even though the Liar's Dice solver had already crossed
//! the bar.
//!
//! The policy-bundle contract is:
//! 1. The bundle's `provenance.game_slug` is always `nlhe-heads-up`.
//! 2. The bundle's `provenance.artifact_hash` equals the dossier's
//!    `artifact_hash` (the build is a faithful recording of the dossier).
//! 3. The bundle's `provenance.benchmark` mirrors the dossier's
//!    `benchmark_summary` so the verifier and the manifest can both point
//!    at the same metric source.
//! 4. The bundle's `recommended_action_id` is in the legal-action set and
//!    in the distribution (verifier-checked).
//! 5. The bundle's `distribution` sums to `TOTAL_PROBABILITY_PPM`
//!    (`1_000_000`) so the deterministic sampler is well-defined.
//! 6. The bundle's `bundle_hash` is a SHA-256 over a sorted-JSON encoding
//!    of every other field; the verifier recomputes the hash and refuses
//!    a bundle that does not match.

use thiserror::Error;

use myosu_games::{
    CanonicalActionSpec, CanonicalStateSnapshot, CanonicalTruthError, validate_action_id,
};
use myosu_games_canonical::{
    CanonicalPolicyBenchmarkSummary, CanonicalPolicyBundle, CanonicalPolicyDistributionEntry,
    CanonicalPolicyProvenance, ResearchGame, compute_bundle_hash, verify_policy_bundle,
};
use rbp_gameplay::Edge as RbpEdge;
use rbp_nlhe::{NlheEdge, NlheInfo};
use serde_json::json;
use sha2::{Digest, Sha256};

use crate::artifacts::{NlheArtifactDossier, NlheBenchmarkDossier};
use crate::request::{NlheStrategyRequest, StrategyRequestError};
use crate::robopoker::{NlheStrategyResponse, recommended_edge};
use crate::solver::PokerSolver;
use crate::state::NlheTablePosition;

const GAME_SLUG: &str = "nlhe-heads-up";
const SOLVER_FAMILY: &str = "nlhe-blueprint-cfr";
const ENGINE_TIER: &str = "promotable_local";
const TOTAL_PROBABILITY_PPM: u64 = 1_000_000;

const PREFLOP_DECISION_OBSERVATION: &str = "AcKh";
const PREFLOP_DECISION_BUCKET: i16 = 0;

/// Bundle plus the pinned artifact dossier that justifies it.
#[derive(Clone, Debug, PartialEq)]
pub struct NlhePolicyBundleEvidence {
    pub bundle: CanonicalPolicyBundle,
    pub artifact_dossier: NlheArtifactDossier,
}

/// Errors returned while constructing an NLHE policy bundle.
#[derive(Debug, Error)]
pub enum NlhePolicyBundleError {
    #[error("failed to construct nlhe strategy request: {0}")]
    Request(#[from] StrategyRequestError),
    #[error("nlhe artifact dossier benchmark `{metric_name}` did not pass threshold {threshold}")]
    BenchmarkThreshold {
        metric_name: String,
        metric_value: f64,
        threshold: f64,
    },
    #[error("nlhe policy decision produced no legal actions")]
    EmptyPolicy,
    #[error("failed to convert nlhe policy probabilities: {reason}")]
    Probability { reason: String },
    #[error("malformed canonical action id `{action_id}`")]
    MalformedActionId { action_id: String },
    #[error("{0}")]
    Canonical(#[from] CanonicalTruthError),
}

/// Build a verified canonical policy bundle from an NLHE solver and dossier.
pub fn build_nlhe_policy_bundle(
    solver: &PokerSolver,
    artifact_dossier: &NlheArtifactDossier,
    decision_label: &str,
) -> Result<CanonicalPolicyBundle, NlhePolicyBundleError> {
    build_nlhe_policy_bundle_evidence(solver, artifact_dossier, decision_label)
        .map(|evidence| evidence.bundle)
}

/// Build a verified bundle and retain the pinned artifact dossier used as provenance.
pub fn build_nlhe_policy_bundle_evidence(
    solver: &PokerSolver,
    artifact_dossier: &NlheArtifactDossier,
    decision_label: &str,
) -> Result<NlhePolicyBundleEvidence, NlhePolicyBundleError> {
    if !artifact_dossier.benchmark_summary.passing {
        let summary = &artifact_dossier.benchmark_summary;
        return Err(NlhePolicyBundleError::BenchmarkThreshold {
            metric_name: summary.metric_name.clone(),
            metric_value: summary.metric_value,
            threshold: summary.threshold,
        });
    }

    let request = NlheStrategyRequest::from_observation_text(
        NlheTablePosition::Button,
        PREFLOP_DECISION_OBSERVATION,
        Vec::new(),
        PREFLOP_DECISION_BUCKET,
    )?;
    let query = request.query_with_encoder(solver.encoder())?;
    let info: NlheInfo = query.info.into_info();
    let response: NlheStrategyResponse = solver.answer(query);
    if response.actions.is_empty() {
        return Err(NlhePolicyBundleError::EmptyPolicy);
    }

    let recommended = recommended_edge(&response).ok_or(NlhePolicyBundleError::EmptyPolicy)?;
    let legal_action_ids = response
        .actions
        .iter()
        .map(|(edge, _)| action_id(*edge))
        .collect::<Result<Vec<_>, _>>()?;
    let legal_actions = response
        .actions
        .iter()
        .map(|(edge, _)| canonical_action_spec(*edge))
        .collect::<Result<Vec<_>, _>>()?;
    let distribution = distribution_entries(&response.actions)?;
    let recommended_action_id = action_id(recommended)?;

    let decision_id = format!("{GAME_SLUG}:{decision_label}");
    let artifact_id = format!(
        "nlhe-checkpoint-{}",
        short_hash(&artifact_dossier.artifact_hash)
    );
    let benchmark = CanonicalPolicyBenchmarkSummary {
        benchmark_id: artifact_dossier.benchmark_summary.benchmark_id.clone(),
        metric_name: artifact_dossier.benchmark_summary.metric_name.clone(),
        metric_value: artifact_dossier.benchmark_summary.metric_value,
        threshold: artifact_dossier.benchmark_summary.threshold,
        passing: artifact_dossier.benchmark_summary.passing,
    };
    let info_commitment = info_commitment_hex(&info);

    let mut bundle = CanonicalPolicyBundle {
        game: ResearchGame::NlheHeadsUp,
        decision_id: decision_id.clone(),
        public_state: CanonicalStateSnapshot {
            game_id: GAME_SLUG.to_string(),
            ruleset_version: 1,
            trace_id: decision_id,
            phase: "preflop-betting".to_string(),
            actor: Some(0),
            public_state: json!({
                "street": "preflop",
                "table_position": "button",
                "hero_hole": ["Ac", "Kh"],
                "board": [],
                "pot": 1.5,
                "stacks": [100.0, 100.0],
                "to_act": 0,
            }),
            private_state_commitments: vec![info_commitment],
            legal_actions,
            terminal: false,
        },
        legal_action_ids,
        distribution,
        recommended_action_id,
        provenance: CanonicalPolicyProvenance {
            game_slug: GAME_SLUG.to_string(),
            solver_family: SOLVER_FAMILY.to_string(),
            engine_tier: ENGINE_TIER.to_string(),
            artifact_id,
            artifact_hash: artifact_dossier.artifact_hash.clone(),
            benchmark,
        },
        bundle_hash: String::new(),
    };
    bundle.bundle_hash = compute_bundle_hash(&bundle)?;
    verify_policy_bundle(&bundle)?;

    Ok(NlhePolicyBundleEvidence {
        bundle,
        artifact_dossier: artifact_dossier.clone(),
    })
}

/// Build a synthetic promotion-eligible `NlheBenchmarkDossier` for tests.
///
/// The real benchmark dossier surface is `NlheBenchmarkDossier` (lives in
/// `artifacts.rs`); this helper only exposes the public constructor that
/// the policy-bundle tests need to feed the builder a passing fixture
/// without going through the disk-based manifest loader. The returned
/// dossier records a `metric_value <= threshold` so the builder accepts it
/// and the verifier roundtrip succeeds.
pub fn passing_benchmark_dossier(
    benchmark_id: impl Into<String>,
    metric_name: impl Into<String>,
    metric_value: f64,
    threshold: f64,
) -> NlheBenchmarkDossier {
    NlheBenchmarkDossier::at_most(benchmark_id, metric_name, metric_value, threshold)
}

/// Build a synthetic sparse-rejection `NlheBenchmarkDossier` for tests.
pub fn sparse_benchmark_dossier(
    benchmark_id: impl Into<String>,
    metric_name: impl Into<String>,
    metric_value: f64,
    threshold: f64,
) -> NlheBenchmarkDossier {
    NlheBenchmarkDossier::at_most(benchmark_id, metric_name, metric_value, threshold)
}

fn action_id(edge: NlheEdge) -> Result<String, NlhePolicyBundleError> {
    let candidate = match RbpEdge::from(edge) {
        RbpEdge::Fold => "nlhe-heads-up.fold".to_string(),
        RbpEdge::Check => "nlhe-heads-up.check".to_string(),
        RbpEdge::Call | RbpEdge::Draw => "nlhe-heads-up.call".to_string(),
        RbpEdge::Open(chips) => format!("nlhe-heads-up.open.{chips}"),
        RbpEdge::Raise(odds) => format!(
            "nlhe-heads-up.raise-to.{}-{}",
            odds.numer(),
            odds.denom()
        ),
        RbpEdge::Shove => "nlhe-heads-up.shove".to_string(),
    };
    validate_action_id(&candidate).map_err(|_| NlhePolicyBundleError::MalformedActionId {
        action_id: candidate.clone(),
    })?;
    Ok(candidate)
}

fn canonical_action_spec(edge: NlheEdge) -> Result<CanonicalActionSpec, NlhePolicyBundleError> {
    let action_id = action_id(edge)?;
    let (display_label, params_schema) = match RbpEdge::from(edge) {
        RbpEdge::Fold => (
            "fold".to_string(),
            json!({"type": "object", "additionalProperties": false}),
        ),
        RbpEdge::Check => (
            "check".to_string(),
            json!({"type": "object", "additionalProperties": false}),
        ),
        RbpEdge::Call | RbpEdge::Draw => (
            "call".to_string(),
            json!({"type": "object", "additionalProperties": false}),
        ),
        RbpEdge::Open(chips) => (
            format!("open.{chips}"),
            json!({
                "type": "object",
                "properties": { "bb": { "const": chips } },
                "additionalProperties": false
            }),
        ),
        RbpEdge::Raise(odds) => (
            format!("raise-to.{}/{}", odds.numer(), odds.denom()),
            json!({
                "type": "object",
                "properties": {
                    "numer": { "const": odds.numer() },
                    "denom": { "const": odds.denom() }
                },
                "additionalProperties": false
            }),
        ),
        RbpEdge::Shove => (
            "shove".to_string(),
            json!({"type": "object", "additionalProperties": false}),
        ),
    };
    Ok(CanonicalActionSpec {
        game_id: GAME_SLUG.to_string(),
        action_id,
        family: "nlhe".to_string(),
        display_label,
        legal_phases: vec![
            "preflop-betting".to_string(),
            "postflop-betting".to_string(),
        ],
        params_schema,
    })
}

fn distribution_entries(
    actions: &[(NlheEdge, f32)],
) -> Result<Vec<CanonicalPolicyDistributionEntry>, NlhePolicyBundleError> {
    if actions.is_empty() {
        return Err(NlhePolicyBundleError::EmptyPolicy);
    }
    let total_probability: f64 = actions.iter().try_fold(0.0_f64, |sum, (_, probability)| {
        if !probability.is_finite() || *probability < 0.0 {
            return Err(NlhePolicyBundleError::Probability {
                reason: format!("invalid probability {probability}"),
            });
        }
        Ok(sum + f64::from(*probability))
    })?;
    if total_probability <= f64::EPSILON {
        return Err(NlhePolicyBundleError::Probability {
            reason: "probability mass is zero".to_string(),
        });
    }

    let mut entries = Vec::with_capacity(actions.len());
    for (edge, probability) in actions {
        let scaled = (f64::from(*probability) / total_probability) * TOTAL_PROBABILITY_PPM as f64;
        let probability_ppm = u32::try_from(scaled.floor() as u64).map_err(|source| {
            NlhePolicyBundleError::Probability {
                reason: source.to_string(),
            }
        })?;
        entries.push(CanonicalPolicyDistributionEntry {
            action_id: action_id(*edge)?,
            probability_ppm,
        });
    }
    let ppm_sum: u64 = entries
        .iter()
        .map(|entry| u64::from(entry.probability_ppm))
        .sum();
    if ppm_sum > TOTAL_PROBABILITY_PPM {
        return Err(NlhePolicyBundleError::Probability {
            reason: "probability ppm sum exceeded total".to_string(),
        });
    }
    let remainder = TOTAL_PROBABILITY_PPM - ppm_sum;
    if let Some(first) = entries.first_mut() {
        let addendum = u32::try_from(remainder).map_err(|source| {
            NlhePolicyBundleError::Probability {
                reason: source.to_string(),
            }
        })?;
        first.probability_ppm = first
            .probability_ppm
            .checked_add(addendum)
            .ok_or_else(|| NlhePolicyBundleError::Probability {
                reason: "probability ppm remainder overflowed".to_string(),
            })?;
    }

    Ok(entries)
}

fn short_hash(hash: &str) -> String {
    hash.chars().take(12).collect()
}

fn info_commitment_hex(info: &NlheInfo) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"myosu-nlhe-policy-bundle-v1");
    hasher.update(u64::from(info.subgame()).to_le_bytes());
    hasher.update(i16::from(info.bucket()).to_le_bytes());
    hasher.update(u64::from(info.choices()).to_le_bytes());
    format!("nlhe-heads-up.info.sha256:{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    use myosu_games_canonical::sample_policy_action;
    use rbp_cards::Isomorphism;
    use rbp_gameplay::{Abstraction, Odds};

    use crate::artifacts::{
        NlheAbstractionStreet, NlheArtifactDossier, NlheArtifactManifestReference,
        NlheEncoderArtifactSummary, bootstrap_encoder_streets, encoder_from_lookup,
    };
    use crate::state::NlheTablePosition;

    const DECISION_LABEL: &str = "preflop-button-ak-offsuite-open";

    fn test_solver() -> PokerSolver {
        let merged_lookup = bootstrap_encoder_streets()
            .into_values()
            .flat_map(|lookup| lookup.into_iter())
            .collect::<BTreeMap<Isomorphism, Abstraction>>();
        let encoder =
            encoder_from_lookup(merged_lookup).expect("bootstrap encoder should build");
        PokerSolver::new(encoder)
    }

    fn synthetic_dossier(metric_value: f64, threshold: f64) -> NlheArtifactDossier {
        let benchmark = passing_benchmark_dossier(
            format!("nlhe-bundle-fixture-{metric_value:.6}"),
            "mean_l1_distance",
            metric_value,
            threshold,
        );
        let mut street_entries = BTreeMap::new();
        street_entries.insert(NlheAbstractionStreet::Preflop, 169_u64);
        let summary = NlheEncoderArtifactSummary {
            version: 1,
            game: "nlhe_hu".to_string(),
            total_sha256: format!("synthetic-{metric_value:.6}"),
            street_entries,
            total_entries: 169,
            postflop_complete: false,
        };
        NlheArtifactDossier {
            artifact_hash: format!("synthetic-{metric_value:.6}"),
            manifest_reference: NlheArtifactManifestReference {
                artifact_dir: format!("synthetic://nlhe-heads-up/{metric_value:.6}"),
                manifest_path: format!(
                    "synthetic://nlhe-heads-up/{metric_value:.6}/manifest.json"
                ),
                manifest_sha256: format!("synthetic-{metric_value:.6}"),
                manifest_total_sha256: format!("synthetic-{metric_value:.6}"),
            },
            manifest_summary: summary,
            benchmark_summary: benchmark,
            provenance_chain: vec![format!("synthetic-fixture:{metric_value:.6}")],
        }
    }

    #[test]
    fn policy_bundle_from_passing_dossier_verifies_and_samples() {
        let solver = test_solver();
        let dossier = synthetic_dossier(0.05, 0.20);

        let bundle = build_nlhe_policy_bundle(&solver, &dossier, DECISION_LABEL)
            .expect("passing dossier should produce a verified bundle");

        verify_policy_bundle(&bundle).expect("bundle should verify");
        let first = sample_policy_action(&bundle, "unit-test", b"fixed-entropy")
            .expect("policy sample should succeed");
        let second = sample_policy_action(&bundle, "unit-test", b"fixed-entropy")
            .expect("policy sample should be repeatable");
        assert_eq!(first.sampled_action_id, second.sampled_action_id);

        assert_eq!(bundle.provenance.game_slug, "nlhe-heads-up");
        assert_eq!(bundle.provenance.engine_tier, "promotable_local");
        assert_eq!(bundle.provenance.artifact_hash, dossier.artifact_hash);
        assert_eq!(
            bundle.provenance.benchmark.metric_name,
            dossier.benchmark_summary.metric_name
        );
        assert!(bundle.provenance.benchmark.passing);
    }

    #[test]
    fn sparse_dossier_below_threshold_is_rejected() {
        let solver = test_solver();
        let benchmark = sparse_benchmark_dossier(
            "nlhe-sparse-fixture",
            "mean_l1_distance",
            0.95,
            0.10,
        );
        let mut street_entries = BTreeMap::new();
        street_entries.insert(NlheAbstractionStreet::Preflop, 169_u64);
        let summary = NlheEncoderArtifactSummary {
            version: 1,
            game: "nlhe_hu".to_string(),
            total_sha256: "synthetic-sparse".to_string(),
            street_entries,
            total_entries: 169,
            postflop_complete: false,
        };
        let dossier = NlheArtifactDossier {
            artifact_hash: "synthetic-sparse".to_string(),
            manifest_reference: NlheArtifactManifestReference {
                artifact_dir: "synthetic://nlhe-heads-up/sparse".to_string(),
                manifest_path: "synthetic://nlhe-heads-up/sparse/manifest.json".to_string(),
                manifest_sha256: "synthetic-sparse".to_string(),
                manifest_total_sha256: "synthetic-sparse".to_string(),
            },
            manifest_summary: summary,
            benchmark_summary: benchmark,
            provenance_chain: vec!["synthetic-fixture:sparse".to_string()],
        };

        let error = build_nlhe_policy_bundle(&solver, &dossier, DECISION_LABEL)
            .expect_err("sparse dossier should fail closed");

        let rendered = error.to_string();
        assert!(
            rendered.contains("did not pass threshold"),
            "unexpected error: {rendered}"
        );
    }

    #[test]
    fn tampered_artifact_hash_is_recorded_in_provenance() {
        let solver = test_solver();
        let mut dossier = synthetic_dossier(0.05, 0.20);
        dossier.artifact_hash = "tampered-hash".to_string();

        let bundle = build_nlhe_policy_bundle(&solver, &dossier, "tampered-hash-decision")
            .expect("builder does not verify external hashes; it records the dossier's claim");

        assert_eq!(bundle.provenance.artifact_hash, "tampered-hash");
        assert!(!bundle.provenance.artifact_id.contains("tampered-hash"));
    }

    #[test]
    fn distribution_invariant_holds_for_emitted_bundle() {
        let solver = test_solver();
        let dossier = synthetic_dossier(0.05, 0.20);

        let bundle = build_nlhe_policy_bundle(&solver, &dossier, DECISION_LABEL)
            .expect("bundle should build");

        let total: u64 = bundle
            .distribution
            .iter()
            .map(|entry| u64::from(entry.probability_ppm))
            .sum();
        assert_eq!(total, TOTAL_PROBABILITY_PPM);
        assert!(!bundle.distribution.is_empty());
    }

    #[test]
    fn recommended_action_is_in_legal_action_set() {
        let solver = test_solver();
        let dossier = synthetic_dossier(0.05, 0.20);

        let bundle = build_nlhe_policy_bundle(&solver, &dossier, DECISION_LABEL)
            .expect("bundle should build");

        assert!(
            bundle
                .legal_action_ids
                .contains(&bundle.recommended_action_id),
            "recommended action must be in legal_action_ids"
        );
        let distribution_ids: std::collections::BTreeSet<&str> = bundle
            .distribution
            .iter()
            .map(|entry| entry.action_id.as_str())
            .collect();
        assert!(distribution_ids.contains(bundle.recommended_action_id.as_str()));
    }

    #[test]
    fn bundled_replay_is_deterministic() {
        let solver = test_solver();
        let dossier = synthetic_dossier(0.05, 0.20);

        let bundle = build_nlhe_policy_bundle(&solver, &dossier, DECISION_LABEL)
            .expect("bundle should build");

        // Same entropy bytes ⇒ same sampled action. Sample twice with the
        // same entropy and confirm the result is byte-stable.
        let first = sample_policy_action(&bundle, "draw-source", b"fixed-entropy")
            .expect("policy draw should succeed");
        let second = sample_policy_action(&bundle, "draw-source", b"fixed-entropy")
            .expect("policy draw should succeed");
        assert_eq!(
            first.sampled_action_id, second.sampled_action_id,
            "same entropy bytes must yield the same sampled action"
        );

        // Different entropy ⇒ different sample space, but the verifier
        // and bundle hash stay byte-stable.
        assert_eq!(first.bundle_hash, second.bundle_hash);
        assert_eq!(first.entropy_hash, second.entropy_hash);
    }

    #[test]
    fn empty_distribution_is_rejected() {
        let result = distribution_entries(&[]);
        assert!(matches!(result, Err(NlhePolicyBundleError::EmptyPolicy)));
    }

    #[test]
    fn action_id_format_is_stable_for_each_edge() {
        let sample = |edge: NlheEdge| action_id(edge).expect("action id should build");
        assert_eq!(sample(NlheEdge::from(RbpEdge::Fold)), "nlhe-heads-up.fold");
        assert_eq!(sample(NlheEdge::from(RbpEdge::Call)), "nlhe-heads-up.call");
        assert_eq!(
            sample(NlheEdge::from(RbpEdge::Check)),
            "nlhe-heads-up.check"
        );
        assert_eq!(
            sample(NlheEdge::from(RbpEdge::Shove)),
            "nlhe-heads-up.shove"
        );
        let raise = NlheEdge::from(RbpEdge::Raise(Odds::new(2, 3)));
        assert_eq!(sample(raise), "nlhe-heads-up.raise-to.2-3");
    }

    #[test]
    fn action_spec_matches_action_id_grammar() {
        let spec = canonical_action_spec(NlheEdge::from(RbpEdge::Fold)).expect("spec should build");
        assert_eq!(spec.action_id, "nlhe-heads-up.fold");
        assert_eq!(spec.family, "nlhe");

        let raise = NlheEdge::from(RbpEdge::Raise(Odds::new(1, 2)));
        let raise_spec = canonical_action_spec(raise).expect("raise spec should build");
        assert_eq!(raise_spec.action_id, "nlhe-heads-up.raise-to.1-2");
        assert_eq!(raise_spec.display_label, "raise-to.1/2");
    }

    #[test]
    fn from_observation_text_rejects_invalid_observation() {
        // Mirror the bootstrap test pattern: an invalid observation text should
        // fail the request before the builder ever gets a chance to fail
        // closed on the dossier. This guards the helper's contract.
        let result = NlheStrategyRequest::from_observation_text(
            NlheTablePosition::Button,
            "ZZ",
            Vec::new(),
            0,
        );
        assert!(result.is_err());
    }

    #[test]
    fn short_hash_truncates_to_12_chars() {
        let hash = "0123456789abcdef0123456789abcdef";
        assert_eq!(short_hash(hash), "0123456789ab");
    }
}

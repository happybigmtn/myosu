//! Canonical policy bundle construction for Liar's Dice promotion.

use myosu_games::{CanonicalActionSpec, CanonicalStateSnapshot, CanonicalTruthError, CfrGame};
use myosu_games_canonical::{
    CanonicalPolicyBenchmarkSummary, CanonicalPolicyBundle, CanonicalPolicyDistributionEntry,
    CanonicalPolicyProvenance, ResearchGame, compute_bundle_hash, policy::TOTAL_PROBABILITY_PPM,
    verify_policy_bundle,
};
use serde_json::json;
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::{
    LiarsDiceArtifactDossier, LiarsDiceDossierError, LiarsDiceEdge, LiarsDiceGame, LiarsDiceSolver,
    protocol::recommended_edge,
};

/// Exact-exploitability threshold required before Liar's Dice may be promoted locally.
pub const LIARS_DICE_PROMOTION_THRESHOLD: f64 = 0.70;

const GAME_SLUG: &str = "liars-dice";
const SOLVER_FAMILY: &str = "liars-dice-cfr";
const PROMOTION_DECISION_P1_DIE: u8 = 2;
const PROMOTION_DECISION_P2_DIE: u8 = 5;

/// Bundle plus the checkpoint dossier used to justify it.
#[derive(Clone, Debug, PartialEq)]
pub struct LiarsDicePolicyBundleEvidence {
    pub bundle: CanonicalPolicyBundle,
    pub artifact_dossier: LiarsDiceArtifactDossier,
}

/// Errors returned while constructing a Liar's Dice policy bundle.
#[derive(Debug, Error)]
pub enum LiarsDicePolicyBundleError {
    #[error("{0}")]
    Dossier(#[from] LiarsDiceDossierError),
    #[error(
        "liar's dice exact exploitability {exploitability:.6} does not clear threshold {threshold:.6}"
    )]
    ExactExploitabilityThreshold { exploitability: f64, threshold: f64 },
    #[error("liar's dice policy decision produced no legal actions")]
    EmptyPolicy,
    #[error("liar's dice policy decision included chance edge")]
    ChanceEdge,
    #[error("failed to convert liar's dice policy probabilities: {reason}")]
    Probability { reason: String },
    #[error("{0}")]
    Canonical(#[from] CanonicalTruthError),
}

/// Build a verified canonical policy bundle from a promotion-grade Liar's Dice solver.
pub fn build_liars_dice_policy_bundle<const N: usize>(
    solver: &LiarsDiceSolver<N>,
    decision_label: &str,
) -> Result<CanonicalPolicyBundle, LiarsDicePolicyBundleError> {
    build_liars_dice_policy_bundle_evidence(solver, decision_label).map(|evidence| evidence.bundle)
}

/// Build a verified bundle and retain the exact checkpoint dossier used as provenance.
pub fn build_liars_dice_policy_bundle_evidence<const N: usize>(
    solver: &LiarsDiceSolver<N>,
    decision_label: &str,
) -> Result<LiarsDicePolicyBundleEvidence, LiarsDicePolicyBundleError> {
    let dossier = LiarsDiceArtifactDossier::from_solver(solver, LIARS_DICE_PROMOTION_THRESHOLD)?;
    if !dossier.passing {
        return Err(LiarsDicePolicyBundleError::ExactExploitabilityThreshold {
            exploitability: dossier.exploitability,
            threshold: dossier.threshold,
        });
    }

    let game = LiarsDiceGame::root().apply(LiarsDiceEdge::Roll {
        p1: PROMOTION_DECISION_P1_DIE,
        p2: PROMOTION_DECISION_P2_DIE,
    });
    let info = game.info().ok_or(LiarsDicePolicyBundleError::EmptyPolicy)?;
    let response = solver.query(info);
    if response.actions.is_empty() {
        return Err(LiarsDicePolicyBundleError::EmptyPolicy);
    }
    let recommended = recommended_edge(&response).ok_or(LiarsDicePolicyBundleError::EmptyPolicy)?;
    let recommended_action_id = action_id(recommended)?;
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
    let decision_id = format!("{GAME_SLUG}:{decision_label}");
    let checkpoint_id = artifact_id(&dossier.checkpoint_hash);

    let mut bundle = CanonicalPolicyBundle {
        game: ResearchGame::LiarsDice,
        decision_id: decision_id.clone(),
        public_state: CanonicalStateSnapshot {
            game_id: GAME_SLUG.to_string(),
            ruleset_version: 1,
            trace_id: decision_id,
            phase: "opening-bid".to_string(),
            actor: Some(0),
            public_state: json!({
                "actor": 0,
                "dice_count": 2,
                "faces": 6,
                "last_claim": null,
                "history": [],
            }),
            private_state_commitments: vec![
                die_commitment("p1", PROMOTION_DECISION_P1_DIE),
                die_commitment("p2", PROMOTION_DECISION_P2_DIE),
            ],
            legal_actions,
            terminal: false,
        },
        legal_action_ids,
        distribution,
        recommended_action_id,
        provenance: CanonicalPolicyProvenance {
            game_slug: GAME_SLUG.to_string(),
            solver_family: SOLVER_FAMILY.to_string(),
            engine_tier: "promotable_local".to_string(),
            artifact_id: checkpoint_id,
            artifact_hash: dossier.checkpoint_hash.clone(),
            benchmark: CanonicalPolicyBenchmarkSummary {
                benchmark_id: dossier.benchmark_summary.benchmark_id.clone(),
                metric_name: dossier.benchmark_summary.metric_name.clone(),
                metric_value: dossier.benchmark_summary.metric_value,
                threshold: dossier.benchmark_summary.threshold,
                passing: dossier.benchmark_summary.passing,
            },
        },
        bundle_hash: String::new(),
    };
    bundle.bundle_hash = compute_bundle_hash(&bundle)?;
    verify_policy_bundle(&bundle)?;

    Ok(LiarsDicePolicyBundleEvidence {
        bundle,
        artifact_dossier: dossier,
    })
}

fn action_id(edge: LiarsDiceEdge) -> Result<String, LiarsDicePolicyBundleError> {
    match edge {
        LiarsDiceEdge::Bid(claim) => Ok(format!("liars-dice.bid.{}x{}", claim.count, claim.face)),
        LiarsDiceEdge::Challenge => Ok("liars-dice.challenge".to_string()),
        LiarsDiceEdge::Roll { .. } => Err(LiarsDicePolicyBundleError::ChanceEdge),
    }
}

fn canonical_action_spec(
    edge: LiarsDiceEdge,
) -> Result<CanonicalActionSpec, LiarsDicePolicyBundleError> {
    match edge {
        LiarsDiceEdge::Bid(claim) => Ok(CanonicalActionSpec {
            game_id: GAME_SLUG.to_string(),
            action_id: action_id(edge)?,
            family: "liars_dice".to_string(),
            display_label: format!("bid {}x{}", claim.count, claim.face),
            legal_phases: vec!["opening-bid".to_string(), "bidding".to_string()],
            params_schema: json!({
                "type": "object",
                "properties": {
                    "count": { "const": claim.count },
                    "face": { "const": claim.face }
                },
                "additionalProperties": false
            }),
        }),
        LiarsDiceEdge::Challenge => Ok(CanonicalActionSpec {
            game_id: GAME_SLUG.to_string(),
            action_id: action_id(edge)?,
            family: "liars_dice".to_string(),
            display_label: "challenge".to_string(),
            legal_phases: vec!["bidding".to_string()],
            params_schema: json!({
                "type": "object",
                "additionalProperties": false
            }),
        }),
        LiarsDiceEdge::Roll { .. } => Err(LiarsDicePolicyBundleError::ChanceEdge),
    }
}

fn distribution_entries(
    actions: &[(LiarsDiceEdge, f32)],
) -> Result<Vec<CanonicalPolicyDistributionEntry>, LiarsDicePolicyBundleError> {
    if actions.is_empty() {
        return Err(LiarsDicePolicyBundleError::EmptyPolicy);
    }
    let total_probability = actions.iter().try_fold(0.0_f64, |sum, (_, probability)| {
        if !probability.is_finite() || *probability < 0.0 {
            return Err(LiarsDicePolicyBundleError::Probability {
                reason: format!("invalid probability {probability}"),
            });
        }
        Ok(sum + f64::from(*probability))
    })?;
    if total_probability <= f64::EPSILON {
        return Err(LiarsDicePolicyBundleError::Probability {
            reason: "probability mass is zero".to_string(),
        });
    }

    let mut entries = Vec::with_capacity(actions.len());
    for (edge, probability) in actions {
        let scaled =
            (f64::from(*probability) / total_probability) * f64::from(TOTAL_PROBABILITY_PPM);
        let probability_ppm = u32::try_from(scaled.floor() as u64).map_err(|source| {
            LiarsDicePolicyBundleError::Probability {
                reason: source.to_string(),
            }
        })?;
        entries.push(CanonicalPolicyDistributionEntry {
            action_id: action_id(*edge)?,
            probability_ppm,
        });
    }

    let ppm_sum = entries.iter().try_fold(0_u32, |sum, entry| {
        sum.checked_add(entry.probability_ppm).ok_or_else(|| {
            LiarsDicePolicyBundleError::Probability {
                reason: "probability ppm sum overflowed".to_string(),
            }
        })
    })?;
    let remainder = TOTAL_PROBABILITY_PPM.checked_sub(ppm_sum).ok_or_else(|| {
        LiarsDicePolicyBundleError::Probability {
            reason: "probability ppm sum exceeded total".to_string(),
        }
    })?;
    if let Some(first) = entries.first_mut() {
        first.probability_ppm = first
            .probability_ppm
            .checked_add(remainder)
            .ok_or_else(|| LiarsDicePolicyBundleError::Probability {
                reason: "probability ppm remainder overflowed".to_string(),
            })?;
    }

    Ok(entries)
}

fn artifact_id(hash: &str) -> String {
    let prefix = hash.chars().take(12).collect::<String>();
    format!("liars-dice-checkpoint-{prefix}")
}

fn die_commitment(label: &str, die: u8) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"myosu-liars-dice-policy-v1");
    hasher.update(label.as_bytes());
    hasher.update([die]);
    format!("liars-dice.{label}.sha256:{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use myosu_games_canonical::{sample_policy_action, verify_policy_bundle};

    use crate::{LiarsDiceSolver, build_liars_dice_policy_bundle};

    const SOLVER_TREES: usize = 1 << 10;

    #[test]
    fn policy_bundle_from_trained_solver_verifies_and_samples() {
        let mut solver = LiarsDiceSolver::<SOLVER_TREES>::new();
        solver
            .train(512)
            .expect("promotion-grade training fixture should run");

        let bundle = build_liars_dice_policy_bundle(&solver, "opening-p1-die-2")
            .expect("trained solver should produce a policy bundle");

        verify_policy_bundle(&bundle).expect("policy bundle should verify");
        let first = sample_policy_action(&bundle, "unit-test", b"fixed-entropy")
            .expect("policy sample should succeed");
        let second = sample_policy_action(&bundle, "unit-test", b"fixed-entropy")
            .expect("policy sample should be repeatable");

        assert_eq!(first, second);
        assert_eq!(bundle.provenance.game_slug, "liars-dice");
        assert_eq!(
            bundle.provenance.benchmark.metric_name,
            "exact_exploitability"
        );
        assert!(bundle.provenance.benchmark.passing);
    }

    #[test]
    fn policy_bundle_rejects_zero_iteration_solver() {
        let solver = LiarsDiceSolver::<SOLVER_TREES>::new();

        let error = build_liars_dice_policy_bundle(&solver, "zero-iteration")
            .expect_err("zero-iteration solver should not clear promotion");

        assert!(error.to_string().contains("exact exploitability"));
    }
}

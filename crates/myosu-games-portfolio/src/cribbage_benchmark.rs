//! Cribbage benchmark dossier surface.
//!
//! The dossier is the promotion artifact the `F-001` / `genesis/plans/009-cribbage-deepening.md`
//! plan calls for: it pins the scenario pack the engine was run against, the
//! `rule-aware` engine family that produced the answers, the recommended action
//! for every scenario, and a deterministic SHA-256 hash over the canonical
//! scenario/answer table so the promotion manifest harness can verify that the
//! evidence attached to a `tier: benchmarked` claim matches the live engine
//! output.
//!
//! The dossier is intentionally `benchmarked`-tier, not `promotable_local`:
//! the policy bundle builder (plan 001) is designed for dedicated games
//! initially, and generalizing it to portfolio games is follow-on work.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::core::cribbage::{CribbageScenario, cribbage_scenario_pack};
use crate::engine::answer_typed_challenge;
use crate::game::ResearchGame;
use crate::protocol::{PortfolioAction, recommended_action};
use crate::state::{CribbageChallenge, PortfolioChallenge, PortfolioChallengeSpot};

const BENCHMARK_METHOD: &str = "cribbage-rule-aware-scenario-pack-v1";
const BENCHMARK_METRIC: &str = "engine_recommendation_count";

/// Hash-pinned evidence that the Cribbage `rule-aware` engine was run against
/// the canonical scenario pack and produced a recommendation for every row.
///
/// The threshold is `0.0` because the rule-aware engine is deterministic and
/// the promotion gate is "every scenario produced a recommendation", not a
/// quality score. Higher-quality promotion tiers (notably `promotable_local`)
/// will swap this dossier for a `CanonicalPolicyBundle` that wraps the same
/// scenario table.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CribbageBenchmarkDossier {
    pub benchmark_id: String,
    pub benchmark_method: String,
    pub metric_name: String,
    pub metric_value: f64,
    pub threshold: f64,
    pub passing: bool,
    pub scenario_count: usize,
    pub recommendation_count: usize,
    pub engine_family: String,
    pub engine_tier: String,
    pub rule_file: String,
    pub scenario_hash: String,
    pub recommendations: BTreeMap<String, String>,
}

/// One row of the canonical scenario/answer table the dossier hashes over.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
struct DossierRow {
    scenario_id: String,
    challenge_id: String,
    decision: String,
    pegging_count: u8,
    run_potential: u8,
    crib_edge: i8,
    pair_trap: bool,
    go_window: bool,
    fifteen_outs: u8,
    max_immediate_points: u8,
    engine_tier: String,
    recommended_action: String,
}

impl CribbageBenchmarkDossier {
    /// Build a dossier by running the live `rule-aware` engine against the
    /// canonical scenario pack.
    pub fn build(benchmark_id: impl Into<String>) -> Result<Self, CribbageDossierError> {
        Self::build_with_scenarios(benchmark_id, cribbage_scenario_pack())
    }

    /// Build a dossier from a custom scenario slice (used by tests and the
    /// negative-fixture harnesses).
    pub fn build_with_scenarios(
        benchmark_id: impl Into<String>,
        scenarios: &[CribbageScenario],
    ) -> Result<Self, CribbageDossierError> {
        let mut rows: Vec<DossierRow> = Vec::with_capacity(scenarios.len());
        let mut recommendations: BTreeMap<String, String> = BTreeMap::new();

        for scenario in scenarios {
            let challenge = PortfolioChallenge::Cribbage(CribbageChallenge {
                spot: PortfolioChallengeSpot::scenario(
                    ResearchGame::Cribbage,
                    scenario.scenario_id,
                    scenario.decision,
                ),
                pegging_count: scenario.pegging_count,
                run_potential: scenario.run_potential,
                crib_edge: scenario.crib_edge,
                pair_trap: scenario.pair_trap,
                go_window: scenario.go_window,
                fifteen_outs: scenario.fifteen_outs,
                max_immediate_points: scenario.max_immediate_points,
            });

            let (challenge_id, decision) = match &challenge {
                PortfolioChallenge::Cribbage(state) => {
                    (state.spot.challenge_id.clone(), state.spot.decision.clone())
                }
                _ => unreachable!(
                    "dispatcher only routes Cribbage challenges to the cribbage engine"
                ),
            };

            let answer = answer_typed_challenge(&challenge, 0).map_err(|error| {
                CribbageDossierError::EngineFailed {
                    scenario_id: scenario.scenario_id.to_string(),
                    error: error.to_string(),
                }
            })?;

            let recommendation = recommended_action(&answer.response).ok_or_else(|| {
                CribbageDossierError::NoRecommendation {
                    scenario_id: scenario.scenario_id.to_string(),
                }
            })?;

            recommendations.insert(
                scenario.scenario_id.to_string(),
                action_token(recommendation).to_string(),
            );

            rows.push(DossierRow {
                scenario_id: scenario.scenario_id.to_string(),
                challenge_id,
                decision,
                pegging_count: scenario.pegging_count,
                run_potential: scenario.run_potential,
                crib_edge: scenario.crib_edge,
                pair_trap: scenario.pair_trap,
                go_window: scenario.go_window,
                fifteen_outs: scenario.fifteen_outs,
                max_immediate_points: scenario.max_immediate_points,
                engine_tier: answer.engine_tier.as_str().to_string(),
                recommended_action: action_token(recommendation).to_string(),
            });
        }

        let scenario_hash = hash_rows(&rows);
        let recommendation_count = recommendations.len();
        let scenario_count = scenarios.len();
        let metric_value = recommendation_count as f64;
        let threshold = scenario_count as f64;
        let passing = recommendation_count == scenario_count;

        Ok(Self {
            benchmark_id: benchmark_id.into(),
            benchmark_method: BENCHMARK_METHOD.to_string(),
            metric_name: BENCHMARK_METRIC.to_string(),
            metric_value,
            threshold,
            passing,
            scenario_count,
            recommendation_count,
            engine_family: "state-aware pegging-crib heuristic".to_string(),
            engine_tier: "rule-aware".to_string(),
            rule_file: ResearchGame::Cribbage.rule_file().to_string(),
            scenario_hash,
            recommendations,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CribbageDossierError {
    #[error("cribbage engine failed for scenario {scenario_id}: {error}")]
    EngineFailed { scenario_id: String, error: String },
    #[error("cribbage engine produced no recommendation for scenario {scenario_id}")]
    NoRecommendation { scenario_id: String },
}

fn action_token(action: PortfolioAction) -> &'static str {
    match action {
        PortfolioAction::PegRun => "peg-run",
        PortfolioAction::KeepCrib => "keep-crib",
        PortfolioAction::DiscardDeadwood => "discard-deadwood",
        // Cribbage engine only emits these three; anything else would be a
        // dispatch regression and is reported as a stable token for tests.
        _ => "unexpected-action",
    }
}

fn hash_rows(rows: &[DossierRow]) -> String {
    // Sort by scenario_id so the hash is independent of the underlying
    // scenario-pack iteration order (the pack is a `&'static []` today but
    // this keeps the dossier verifiable if the pack is later sourced from a
    // non-deterministic iterator).
    let mut sorted: Vec<&DossierRow> = rows.iter().collect();
    sorted.sort_by(|left, right| left.scenario_id.cmp(&right.scenario_id));

    let mut hasher = Sha256::new();
    for row in sorted {
        hasher.update(row.scenario_id.as_bytes());
        hasher.update(b"\0");
        hasher.update(row.challenge_id.as_bytes());
        hasher.update(b"\0");
        hasher.update(row.decision.as_bytes());
        hasher.update(b"\0");
        hasher.update([row.pegging_count]);
        hasher.update([row.run_potential]);
        hasher.update(row.crib_edge.to_le_bytes());
        hasher.update([u8::from(row.pair_trap)]);
        hasher.update([u8::from(row.go_window)]);
        hasher.update([row.fifteen_outs]);
        hasher.update([row.max_immediate_points]);
        hasher.update(b"\0");
        hasher.update(row.engine_tier.as_bytes());
        hasher.update(b"\0");
        hasher.update(row.recommended_action.as_bytes());
        hasher.update(b"\n");
    }
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cribbage_dossier_passes_full_scenario_pack() {
        let dossier = CribbageBenchmarkDossier::build("cribbage-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));

        assert!(dossier.passing, "dossier should pass");
        assert_eq!(
            dossier.scenario_count,
            cribbage_scenario_pack().len(),
            "dossier should cover every scenario"
        );
        assert_eq!(dossier.recommendation_count, dossier.scenario_count);
        assert_eq!(dossier.engine_tier, "rule-aware");
        assert!(
            dossier.scenario_hash.len() == 64,
            "sha-256 hex should be 64 chars"
        );
    }

    #[test]
    fn cribbage_dossier_is_deterministic_across_runs() {
        let first = CribbageBenchmarkDossier::build("cribbage-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));
        let second = CribbageBenchmarkDossier::build("cribbage-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));

        assert_eq!(first.scenario_hash, second.scenario_hash);
        assert_eq!(first.recommendations, second.recommendations);
    }

    #[test]
    fn cribbage_dossier_recommendation_map_is_complete() {
        let dossier = CribbageBenchmarkDossier::build("cribbage-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));

        for scenario in cribbage_scenario_pack() {
            let token = dossier
                .recommendations
                .get(scenario.scenario_id)
                .unwrap_or_else(|| {
                    panic!(
                        "dossier should record a recommendation for {}",
                        scenario.scenario_id
                    )
                });
            assert!(
                matches!(token.as_str(), "peg-run" | "keep-crib" | "discard-deadwood"),
                "scenario {} produced unexpected action token {token}",
                scenario.scenario_id
            );
        }
    }

    #[test]
    fn cribbage_dossier_peg_run_dominates_run_heavy_scenarios() {
        let dossier = CribbageBenchmarkDossier::build("cribbage-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));

        // pegging-run-three and pegging-run-four-setup both have nonzero
        // run_potential with no pair_trap or fifteen_outs, so the rule-aware
        // engine should rank PegRun over KeepCrib / DiscardDeadwood.
        for scenario_id in ["pegging-run-three", "pegging-run-four-setup"] {
            assert_eq!(
                dossier.recommendations.get(scenario_id).map(String::as_str),
                Some("peg-run"),
                "{scenario_id} should recommend peg-run"
            );
        }
    }
}

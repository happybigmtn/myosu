//! Hearts benchmark dossier surface.
//!
//! The dossier is the promotion artifact the `F-008` slice ships: it pins
//! the 22-scenario rule-aware scenario pack for Hearts, runs the live
//! portfolio engine against it, records every engine recommendation, and
//! SHA-256-pins the canonical scenario/answer table so the promotion
//! manifest harness can verify that the evidence attached to a
//! `tier: benchmarked` claim matches the live engine output.
//!
//! The dossier is intentionally `benchmarked`-tier, not `promotable_local`:
//! the policy bundle builder (`genesis/plans/001-master-plan.md`) is
//! designed for dedicated games initially, and generalizing it to
//! portfolio games is follow-on work (the same scope boundary the F-001
//! Cribbage row landed under).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::core::trick_taking::{HeartsScenario, hearts_scenario_pack};
use crate::engine::answer_typed_challenge;
use crate::game::ResearchGame;
use crate::protocol::{PortfolioAction, recommended_action};
use crate::state::{PortfolioChallenge, PortfolioChallengeSpot, TrickTakingChallenge};

const BENCHMARK_METHOD: &str = "hearts-rule-aware-scenario-pack-v1";
const BENCHMARK_METRIC: &str = "engine_recommendation_count";

/// Hash-pinned evidence that the Hearts `rule-aware` engine was run against
/// the canonical scenario pack and produced a recommendation for every row.
///
/// The threshold is `0.0` because the rule-aware engine is deterministic
/// and the promotion gate is "every scenario produced a recommendation", not
/// a quality score. Higher-quality promotion tiers (notably
/// `promotable_local`) will swap this dossier for a `CanonicalPolicyBundle`
/// that wraps the same scenario table.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct HeartsBenchmarkDossier {
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
    penalty_pressure: u8,
    winners: u8,
    void_suits: u8,
    cards_in_trick: u8,
    follow_suit_forced: bool,
    moon_shot_viable: bool,
    engine_tier: String,
    recommended_action: String,
}

impl HeartsBenchmarkDossier {
    /// Build a dossier by running the live `rule-aware` engine against the
    /// canonical Hearts scenario pack.
    pub fn build(benchmark_id: impl Into<String>) -> Result<Self, HeartsDossierError> {
        Self::build_with_scenarios(benchmark_id, hearts_scenario_pack())
    }

    /// Build a dossier from a custom scenario slice (used by tests and the
    /// negative-fixture harnesses).
    pub fn build_with_scenarios(
        benchmark_id: impl Into<String>,
        scenarios: &[HeartsScenario],
    ) -> Result<Self, HeartsDossierError> {
        let mut rows: Vec<DossierRow> = Vec::with_capacity(scenarios.len());
        let mut recommendations: BTreeMap<String, String> = BTreeMap::new();

        for scenario in scenarios {
            let challenge = PortfolioChallenge::Hearts(TrickTakingChallenge {
                spot: PortfolioChallengeSpot::scenario(
                    ResearchGame::Hearts,
                    scenario.scenario_id,
                    scenario.decision,
                ),
                // Hearts-specific pins: the engine does not use trump, contract,
                // or nil fields (feature_view keeps them at the Hearts-correct
                // values), but they are required by the typed challenge shape.
                trump_count: 0,
                winners: scenario.winners,
                void_suits: scenario.void_suits,
                contract_pressure: 0,
                penalty_pressure: scenario.penalty_pressure,
                cards_in_trick: scenario.cards_in_trick,
                follow_suit_forced: scenario.follow_suit_forced,
                nil_viable: false,
                moon_shot_viable: scenario.moon_shot_viable,
            });

            let (challenge_id, decision) = match &challenge {
                PortfolioChallenge::Hearts(state) => {
                    (state.spot.challenge_id.clone(), state.spot.decision.clone())
                }
                _ => unreachable!("dispatcher only routes Hearts challenges to the hearts engine"),
            };

            let answer = answer_typed_challenge(&challenge, 0).map_err(|error| {
                HeartsDossierError::EngineFailed {
                    scenario_id: scenario.scenario_id.to_string(),
                    error: error.to_string(),
                }
            })?;

            let recommendation = recommended_action(&answer.response).ok_or_else(|| {
                HeartsDossierError::NoRecommendation {
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
                penalty_pressure: scenario.penalty_pressure,
                winners: scenario.winners,
                void_suits: scenario.void_suits,
                cards_in_trick: scenario.cards_in_trick,
                follow_suit_forced: scenario.follow_suit_forced,
                moon_shot_viable: scenario.moon_shot_viable,
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
            engine_family: "state-aware hearts penalty heuristic".to_string(),
            engine_tier: "rule-aware".to_string(),
            rule_file: ResearchGame::Hearts.rule_file().to_string(),
            scenario_hash,
            recommendations,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum HeartsDossierError {
    #[error("hearts engine failed for scenario {scenario_id}: {error}")]
    EngineFailed { scenario_id: String, error: String },
    #[error("hearts engine produced no recommendation for scenario {scenario_id}")]
    NoRecommendation { scenario_id: String },
}

fn action_token(action: PortfolioAction) -> &'static str {
    match action {
        PortfolioAction::AvoidPenalty => "avoid-penalty",
        PortfolioAction::FollowSuit => "follow-suit",
        PortfolioAction::ShootMoon => "shoot-moon",
        // Hearts engine only emits these three; anything else would be a
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
        hasher.update([row.penalty_pressure]);
        hasher.update([row.winners]);
        hasher.update([row.void_suits]);
        hasher.update([row.cards_in_trick]);
        hasher.update([u8::from(row.follow_suit_forced)]);
        hasher.update([u8::from(row.moon_shot_viable)]);
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
    fn hearts_dossier_passes_full_scenario_pack() {
        let dossier = HeartsBenchmarkDossier::build("hearts-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));

        assert!(dossier.passing, "dossier should pass");
        assert_eq!(
            dossier.scenario_count,
            hearts_scenario_pack().len(),
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
    fn hearts_dossier_is_deterministic_across_runs() {
        let first = HeartsBenchmarkDossier::build("hearts-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));
        let second = HeartsBenchmarkDossier::build("hearts-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));

        assert_eq!(first.scenario_hash, second.scenario_hash);
        assert_eq!(first.recommendations, second.recommendations);
    }

    #[test]
    fn hearts_dossier_recommendation_map_is_complete() {
        let dossier = HeartsBenchmarkDossier::build("hearts-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));

        for scenario in hearts_scenario_pack() {
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
                matches!(
                    token.as_str(),
                    "avoid-penalty" | "follow-suit" | "shoot-moon"
                ),
                "scenario {} produced unexpected action token {token}",
                scenario.scenario_id
            );
        }
    }

    #[test]
    fn hearts_dossier_moon_viable_scenarios_recommend_shoot_moon() {
        let dossier = HeartsBenchmarkDossier::build("hearts-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));

        // Every pure moon-viable / no-pressure scenario (no follow forced, no
        // penalty pressure, no void-rich follow-suit steal) should pick
        // ShootMoon. This is the engine-level guard for the F-008 promotion:
        // a regression that flipped these scenarios to AvoidPenalty would be
        // a real heuristic bug, not just a hash drift.
        for scenario_id in [
            "moon-viable-fresh",
            "moon-viable-mid",
            "moon-viable-with-winner",
            "moon-viable-with-void",
            "moon-viable-mid-trick",
            "light-penalty-moon-viable",
            "forced-follow-moon-low",
        ] {
            assert_eq!(
                dossier.recommendations.get(scenario_id).map(String::as_str),
                Some("shoot-moon"),
                "{scenario_id} should recommend shoot-moon"
            );
        }
    }

    #[test]
    fn hearts_dossier_forced_follow_scenarios_recommend_follow_suit() {
        let dossier = HeartsBenchmarkDossier::build("hearts-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));

        // The forced-follow bucket (no moon pressure) and the void-rich
        // forced-follow-moon-voided case should all pick FollowSuit. A
        // regression to AvoidPenalty or ShootMoon would be a heuristic bug.
        for scenario_id in [
            "forced-follow-clean",
            "forced-follow-voided",
            "forced-follow-with-winner",
            "forced-follow-trick-end",
            "forced-follow-with-void-low-penalty",
            "forced-follow-moon-voided",
        ] {
            assert_eq!(
                dossier.recommendations.get(scenario_id).map(String::as_str),
                Some("follow-suit"),
                "{scenario_id} should recommend follow-suit"
            );
        }
    }
}

//! Bridge benchmark dossier surface.
//!
//! The dossier is the promotion artifact the `F-011` slice ships: it pins
//! the 22-scenario rule-aware scenario pack for Bridge, runs the live
//! portfolio engine against it, records every engine recommendation, and
//! SHA-256-pins the canonical scenario/answer table so the promotion
//! manifest harness can verify that the evidence attached to a
//! `tier: benchmarked` claim matches the live engine output.
//!
//! Bridge is one of the four games sharing the `trick_taking` engine
//! surface (F-008 Hearts, F-010 Spades, F-011 Bridge, and the future
//! F-012 CallBreak slice). The F-010 plan row explicitly called out
//! Bridge's `epochs` parameter as warranting a dedicated dossier — the
//! seed-bit term in `engines/trick_taking.rs::bridge` is folded into
//! `double_dummy`, so every scenario below is hand-verified to stay
//! stable even when the seed-bit nudge fires.
//!
//! The dossier is intentionally `benchmarked`-tier, not `promotable_local`:
//! the policy bundle builder (`genesis/plans/001-master-plan.md`) is
//! designed for dedicated games initially, and generalizing it to
//! portfolio games is follow-on work (the same scope boundary the F-001
//! Cribbage, F-008 Hearts, F-009 Gin Rummy, and F-010 Spades rows landed
//! under).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::core::trick_taking::{BridgeScenario, bridge_scenario_pack};
use crate::engine::answer_typed_challenge;
use crate::game::ResearchGame;
use crate::protocol::{PortfolioAction, recommended_action};
use crate::state::{PortfolioChallenge, PortfolioChallengeSpot, TrickTakingChallenge};

const BENCHMARK_METHOD: &str = "bridge-rule-aware-scenario-pack-v1";
const BENCHMARK_METRIC: &str = "engine_recommendation_count";

/// Hash-pinned evidence that the Bridge `rule-aware` engine was run against
/// the canonical scenario pack and produced a recommendation for every row.
///
/// The threshold is `0.0` because the rule-aware engine is deterministic
/// and the promotion gate is "every scenario produced a recommendation", not
/// a quality score. Higher-quality promotion tiers (notably
/// `promotable_local`) will swap this dossier for a `CanonicalPolicyBundle`
/// that wraps the same scenario table.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BridgeBenchmarkDossier {
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
    trump_count: u8,
    winners: u8,
    void_suits: u8,
    contract_pressure: i8,
    cards_in_trick: u8,
    follow_suit_forced: bool,
    nil_viable: bool,
    engine_tier: String,
    recommended_action: String,
}

impl BridgeBenchmarkDossier {
    /// Build a dossier by running the live `rule-aware` engine against the
    /// canonical Bridge scenario pack.
    pub fn build(benchmark_id: impl Into<String>) -> Result<Self, BridgeDossierError> {
        Self::build_with_scenarios(benchmark_id, bridge_scenario_pack())
    }

    /// Build a dossier from a custom scenario slice (used by tests and the
    /// negative-fixture harnesses).
    pub fn build_with_scenarios(
        benchmark_id: impl Into<String>,
        scenarios: &[BridgeScenario],
    ) -> Result<Self, BridgeDossierError> {
        let mut rows: Vec<DossierRow> = Vec::with_capacity(scenarios.len());
        let mut recommendations: BTreeMap<String, String> = BTreeMap::new();

        for scenario in scenarios {
            let challenge = PortfolioChallenge::Bridge(TrickTakingChallenge {
                spot: PortfolioChallengeSpot::scenario(
                    ResearchGame::Bridge,
                    scenario.scenario_id,
                    scenario.decision,
                ),
                // Bridge-specific pins: the engine does not use penalty_pressure
                // or nil_viable (feature_view keeps them at the Bridge-correct
                // values), but they are required by the typed challenge shape.
                trump_count: scenario.trump_count,
                winners: scenario.winners,
                void_suits: scenario.void_suits,
                contract_pressure: scenario.contract_pressure,
                penalty_pressure: scenario.penalty_pressure,
                cards_in_trick: scenario.cards_in_trick,
                follow_suit_forced: scenario.follow_suit_forced,
                nil_viable: scenario.nil_viable,
                moon_shot_viable: false,
            });

            let (challenge_id, decision) = match &challenge {
                PortfolioChallenge::Bridge(state) => {
                    (state.spot.challenge_id.clone(), state.spot.decision.clone())
                }
                _ => unreachable!("dispatcher only routes Bridge challenges to the bridge engine"),
            };

            let answer = answer_typed_challenge(&challenge, 0).map_err(|error| {
                BridgeDossierError::EngineFailed {
                    scenario_id: scenario.scenario_id.to_string(),
                    error: error.to_string(),
                }
            })?;

            let recommendation = recommended_action(&answer.response).ok_or_else(|| {
                BridgeDossierError::NoRecommendation {
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
                trump_count: scenario.trump_count,
                winners: scenario.winners,
                void_suits: scenario.void_suits,
                contract_pressure: scenario.contract_pressure,
                cards_in_trick: scenario.cards_in_trick,
                follow_suit_forced: scenario.follow_suit_forced,
                nil_viable: scenario.nil_viable,
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
            engine_family: "state-aware bridge control heuristic".to_string(),
            engine_tier: "rule-aware".to_string(),
            rule_file: ResearchGame::Bridge.rule_file().to_string(),
            scenario_hash,
            recommendations,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BridgeDossierError {
    #[error("bridge engine failed for scenario {scenario_id}: {error}")]
    EngineFailed { scenario_id: String, error: String },
    #[error("bridge engine produced no recommendation for scenario {scenario_id}")]
    NoRecommendation { scenario_id: String },
}

fn action_token(action: PortfolioAction) -> &'static str {
    match action {
        PortfolioAction::DoubleDummyPlay => "double-dummy",
        PortfolioAction::FollowSuit => "follow-suit",
        PortfolioAction::BidContract => "bid-contract",
        // Bridge engine only emits these three; anything else would be a
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
        hasher.update([row.trump_count]);
        hasher.update([row.winners]);
        hasher.update([row.void_suits]);
        hasher.update(row.contract_pressure.to_le_bytes());
        hasher.update([row.cards_in_trick]);
        hasher.update([u8::from(row.follow_suit_forced)]);
        hasher.update([u8::from(row.nil_viable)]);
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
    fn bridge_dossier_passes_full_scenario_pack() {
        let dossier = BridgeBenchmarkDossier::build("bridge-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));

        assert!(dossier.passing, "dossier should pass");
        assert_eq!(
            dossier.scenario_count,
            bridge_scenario_pack().len(),
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
    fn bridge_dossier_is_deterministic_across_runs() {
        let first = BridgeBenchmarkDossier::build("bridge-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));
        let second = BridgeBenchmarkDossier::build("bridge-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));

        assert_eq!(first.scenario_hash, second.scenario_hash);
        assert_eq!(first.recommendations, second.recommendations);
    }

    #[test]
    fn bridge_dossier_recommendation_map_is_complete() {
        let dossier = BridgeBenchmarkDossier::build("bridge-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));

        for scenario in bridge_scenario_pack() {
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
                    "double-dummy" | "follow-suit" | "bid-contract"
                ),
                "scenario {} produced unexpected action token {token}",
                scenario.scenario_id
            );
        }
    }

    #[test]
    fn bridge_dossier_double_dummy_scenarios_recommend_double_dummy() {
        let dossier = BridgeBenchmarkDossier::build("bridge-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));

        // Every double-dummy-bucket scenario (winners-rich and/or
        // cards-in-trick with no forced follow) should pick
        // DoubleDummyPlay. A regression that flipped these scenarios to
        // FollowSuit or BidContract would be a real heuristic bug.
        for scenario_id in [
            "dd-rich-winners",
            "dd-trump-rich",
            "dd-mid-pressure-winners",
            "dd-cards-in-trick-winners",
            "dd-high-winners-voided",
            "dd-trump-cards-winners",
        ] {
            assert_eq!(
                dossier.recommendations.get(scenario_id).map(String::as_str),
                Some("double-dummy"),
                "{scenario_id} should recommend double-dummy"
            );
        }
    }

    #[test]
    fn bridge_dossier_forced_follow_scenarios_recommend_follow_suit() {
        let dossier = BridgeBenchmarkDossier::build("bridge-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));

        // The pure forced-follow bucket (forced=true with no winners and
        // no contract pressure) should pick FollowSuit. A regression to
        // DoubleDummyPlay or BidContract would be a heuristic bug.
        for scenario_id in [
            "fs-clean-forced",
            "fs-forced-voided",
            "fs-forced-with-light-winners",
            "fs-forced-trick-end",
            "fs-forced-cards-in-trick",
            "fs-forced-mixed-trump-void",
        ] {
            assert_eq!(
                dossier.recommendations.get(scenario_id).map(String::as_str),
                Some("follow-suit"),
                "{scenario_id} should recommend follow-suit"
            );
        }
    }

    #[test]
    fn bridge_dossier_bid_contract_scenarios_recommend_bid_contract() {
        let dossier = BridgeBenchmarkDossier::build("bridge-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));

        // Every contract-pressure-bucket scenario (cards_in_trick=0 with
        // contract_pressure>=2 and no winners/trumps enough to flip the
        // double-dummy heuristic) should pick BidContract. A regression to
        // DoubleDummyPlay or FollowSuit would be a heuristic bug.
        for scenario_id in [
            "bc-opening-push",
            "bc-mid-pressure",
            "bc-heavy-pressure",
            "bc-with-light-winners",
            "bc-voided-mid-pressure",
            "bc-trump-rich-pressure",
        ] {
            assert_eq!(
                dossier.recommendations.get(scenario_id).map(String::as_str),
                Some("bid-contract"),
                "{scenario_id} should recommend bid-contract"
            );
        }
    }
}

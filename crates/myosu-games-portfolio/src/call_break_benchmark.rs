//! Call Break benchmark dossier surface.
//!
//! The dossier is the promotion artifact the `F-012` slice ships: it pins
//! the 22-scenario rule-aware scenario pack for Call Break, runs the live
//! portfolio engine against it, records every engine recommendation, and
//! SHA-256-pins the canonical scenario/answer table so the promotion
//! manifest harness can verify that the evidence attached to a
//! `tier: benchmarked` claim matches the live engine output.
//!
//! Call Break is the fourth and final game sharing the `trick_taking` engine
//! surface (F-008 Hearts, F-010 Spades, F-011 Bridge, and the F-010/F-011
//! plan rows explicitly deferred Call Break's "bidding-flavour heuristic"
//! to its own dossier slice). The F-010 plan row noted that Call Break's
//! engine shape "is closer to a bidding game than a trick-play game":
//! `call_trump` is the bidding-style arm keyed to `trump_count` +
//! `contract_pressure` + `cards_in_trick`, while `trump_control` and
//! `follow_suit` are the trick-play arms. Every scenario below is
//! hand-verified against the `state-aware call-break trick heuristic`
//! math in `engines/trick_taking.rs::call_break` so the dominant arm
//! stays dominant by at least 0.04 even on the tightest edge cases
//! (e.g. `mixed-ct-tc-edge-voided` lands at ct≈1.45 vs tcc≈1.40, a
//! 0.05 margin).
//!
//! The dossier is intentionally `benchmarked`-tier, not `promotable_local`:
//! the policy bundle builder (`genesis/plans/001-master-plan.md`) is
//! designed for dedicated games initially, and generalizing it to
//! portfolio games is follow-on work (the same scope boundary the F-001
//! Cribbage, F-008 Hearts, F-009 Gin Rummy, F-010 Spades, and F-011 Bridge
//! rows landed under).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::core::trick_taking::{CallBreakScenario, call_break_scenario_pack};
use crate::engine::answer_typed_challenge;
use crate::game::ResearchGame;
use crate::protocol::{PortfolioAction, recommended_action};
use crate::state::{PortfolioChallenge, PortfolioChallengeSpot, TrickTakingChallenge};

const BENCHMARK_METHOD: &str = "call-break-rule-aware-scenario-pack-v1";
const BENCHMARK_METRIC: &str = "engine_recommendation_count";

/// Hash-pinned evidence that the Call Break `rule-aware` engine was run
/// against the canonical scenario pack and produced a recommendation for
/// every row.
///
/// The threshold is `0.0` because the rule-aware engine is deterministic
/// and the promotion gate is "every scenario produced a recommendation", not
/// a quality score. Higher-quality promotion tiers (notably
/// `promotable_local`) will swap this dossier for a `CanonicalPolicyBundle`
/// that wraps the same scenario table.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CallBreakBenchmarkDossier {
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

impl CallBreakBenchmarkDossier {
    /// Build a dossier by running the live `rule-aware` engine against the
    /// canonical Call Break scenario pack.
    pub fn build(benchmark_id: impl Into<String>) -> Result<Self, CallBreakDossierError> {
        Self::build_with_scenarios(benchmark_id, call_break_scenario_pack())
    }

    /// Build a dossier from a custom scenario slice (used by tests and the
    /// negative-fixture harnesses).
    pub fn build_with_scenarios(
        benchmark_id: impl Into<String>,
        scenarios: &[CallBreakScenario],
    ) -> Result<Self, CallBreakDossierError> {
        let mut rows: Vec<DossierRow> = Vec::with_capacity(scenarios.len());
        let mut recommendations: BTreeMap<String, String> = BTreeMap::new();

        for scenario in scenarios {
            let challenge = PortfolioChallenge::CallBreak(TrickTakingChallenge {
                spot: PortfolioChallengeSpot::scenario(
                    ResearchGame::CallBreak,
                    scenario.scenario_id,
                    scenario.decision,
                ),
                // Call Break-specific pins: the engine does not use
                // penalty_pressure or moon_shot_viable (feature_view keeps
                // them at the Call Break-correct values — penalty_pressure=0,
                // moon_shot_viable=false), but they are required by the typed
                // challenge shape and pinned for shape parity with the
                // dossier hash.
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
                PortfolioChallenge::CallBreak(state) => {
                    (state.spot.challenge_id.clone(), state.spot.decision.clone())
                }
                _ => unreachable!(
                    "dispatcher only routes Call Break challenges to the call_break engine"
                ),
            };

            let answer = answer_typed_challenge(&challenge, 0).map_err(|error| {
                CallBreakDossierError::EngineFailed {
                    scenario_id: scenario.scenario_id.to_string(),
                    error: error.to_string(),
                }
            })?;

            let recommendation = recommended_action(&answer.response).ok_or_else(|| {
                CallBreakDossierError::NoRecommendation {
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
            engine_family: "state-aware call-break trick heuristic".to_string(),
            engine_tier: "rule-aware".to_string(),
            rule_file: ResearchGame::CallBreak.rule_file().to_string(),
            scenario_hash,
            recommendations,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CallBreakDossierError {
    #[error("call-break engine failed for scenario {scenario_id}: {error}")]
    EngineFailed { scenario_id: String, error: String },
    #[error("call-break engine produced no recommendation for scenario {scenario_id}")]
    NoRecommendation { scenario_id: String },
}

fn action_token(action: PortfolioAction) -> &'static str {
    match action {
        PortfolioAction::CallTrump => "call-trump",
        PortfolioAction::TrumpControl => "trump-control",
        PortfolioAction::FollowSuit => "follow-suit",
        // Call Break engine only emits these three; anything else would be
        // a dispatch regression and is reported as a stable token for tests.
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
    fn call_break_dossier_passes_full_scenario_pack() {
        let dossier = CallBreakBenchmarkDossier::build("call-break-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));

        assert!(dossier.passing, "dossier should pass");
        assert_eq!(
            dossier.scenario_count,
            call_break_scenario_pack().len(),
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
    fn call_break_dossier_is_deterministic_across_runs() {
        let first = CallBreakBenchmarkDossier::build("call-break-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));
        let second = CallBreakBenchmarkDossier::build("call-break-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));

        assert_eq!(first.scenario_hash, second.scenario_hash);
        assert_eq!(first.recommendations, second.recommendations);
    }

    #[test]
    fn call_break_dossier_recommendation_map_is_complete() {
        let dossier = CallBreakBenchmarkDossier::build("call-break-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));

        for scenario in call_break_scenario_pack() {
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
                    "call-trump" | "trump-control" | "follow-suit"
                ),
                "scenario {} produced unexpected action token {token}",
                scenario.scenario_id
            );
        }
    }

    #[test]
    fn call_break_dossier_call_trump_scenarios_recommend_call_trump() {
        let dossier = CallBreakBenchmarkDossier::build("call-break-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));

        // Every call_trump-bucket scenario (trump + contract pressure on a
        // free lead, or trump + light contract pressure with two winners)
        // should pick CallTrump. A regression that flipped these scenarios
        // to TrumpControl or FollowSuit would be a real heuristic bug.
        for scenario_id in [
            "ct-call-trump-rich-cp",
            "ct-trump-rich-mid-cp",
            "ct-trump-cp-mid",
            "ct-trump-cp-rich-voided",
            "ct-clean-cp",
            "ct-trump-rich-with-winners",
        ] {
            assert_eq!(
                dossier.recommendations.get(scenario_id).map(String::as_str),
                Some("call-trump"),
                "{scenario_id} should recommend call-trump"
            );
        }
    }

    #[test]
    fn call_break_dossier_trump_control_scenarios_recommend_trump_control() {
        let dossier = CallBreakBenchmarkDossier::build("call-break-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));

        // The pure trump_control bucket (high winners + voids/trumps, with
        // no contract pressure to flip to call_trump) should pick
        // TrumpControl. A regression to CallTrump or FollowSuit would be a
        // heuristic bug.
        for scenario_id in [
            "tc-winners-rich-voided",
            "tc-winners-rich-cp-zero",
            "tc-winners-mid-voided",
            "tc-winners-rich-mixed",
            "tc-voided-rich-winners",
            "tc-cards-in-trick-winners",
        ] {
            assert_eq!(
                dossier.recommendations.get(scenario_id).map(String::as_str),
                Some("trump-control"),
                "{scenario_id} should recommend trump-control"
            );
        }
    }

    #[test]
    fn call_break_dossier_follow_suit_scenarios_recommend_follow_suit() {
        let dossier = CallBreakBenchmarkDossier::build("call-break-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));

        // The forced-follow bucket (follow_suit_forced=true with no
        // contract pressure) should pick FollowSuit. A regression to
        // CallTrump or TrumpControl would be a heuristic bug.
        for scenario_id in [
            "fs-clean-forced",
            "fs-forced-voided",
            "fs-forced-with-light-winners",
            "fs-forced-trick-end",
            "fs-forced-cards-in-trick",
            "fs-forced-mid-cp",
        ] {
            assert_eq!(
                dossier.recommendations.get(scenario_id).map(String::as_str),
                Some("follow-suit"),
                "{scenario_id} should recommend follow-suit"
            );
        }
    }
}

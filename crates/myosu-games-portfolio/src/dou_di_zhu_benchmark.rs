//! Dou-Di-Zhu benchmark dossier surface.
//!
//! The dossier is the promotion artifact the `F-018` slice ships: it
//! pins the 22-scenario rule-aware scenario pack for Dou-Di-Zhu, runs
//! the live portfolio engine against it, records every engine
//! recommendation, and SHA-256-pins the canonical scenario/answer
//! table so the promotion manifest harness can verify that the
//! evidence attached to a `tier: benchmarked` claim matches the live
//! engine output.
//!
//! Dou-Di-Zhu is the F-018 portfolio-game-promotion slice (the
//! F-001 / F-008 / F-009 / F-010 / F-011 / F-012 / F-013 / F-014 /
//! F-015 / F-016 / F-017 twelfth slice: Cribbage / Hearts / Gin Rummy
//! / Spades / Bridge / Call Break / Backgammon / Hanafuda Koi-Koi /
//! Hwatu Go-Stop / Stratego / PLO). F-018 opens the
//! `state-aware bomb-preservation heuristic` engine sub-family in
//! `crate::engines::shedding::dou_di_zhu` — no other portfolio game
//! has a dossier row for the `shedding` engine family (pusoy-dos and
//! tien-len share the `SheddingChallenge` struct, but F-018
//! Dou-Di-Zhu is the first dossier slice for this sub-family).
//!
//! The engine has three ranked arms (`preserve-bomb` /
//! `landlord-bid` / `shed-lowest`) keyed to bomb-preservation /
//! landlord-initiative / finishing-line state. The F-018 scenario
//! pack splits 8/8/6 across preserve-bomb-dominant /
//! landlord-bid-dominant / shed-lowest-dominant so a regression
//! that flips the dominant arm on any scenario is loud in the
//! dossier's recommendation map.
//!
//! The dossier is intentionally `benchmarked`-tier, not
//! `promotable_local`: the policy bundle builder
//! (`genesis/plans/001-master-plan.md`) is designed for dedicated
//! games initially, and generalizing it to portfolio games is
//! follow-on work (the same scope boundary the F-001 Cribbage, F-008
//! Hearts, F-009 Gin Rummy, F-010 Spades, F-011 Bridge, F-012 Call
//! Break, F-013 Backgammon, F-014 HanafudaKoiKoi, F-015 HwatuGoStop,
//! F-016 Stratego, and F-017 PLO rows landed under).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::core::shedding::{DouDiZhuScenario, dou_di_zhu_scenario_pack};
use crate::engine::answer_typed_challenge;
use crate::game::ResearchGame;
use crate::protocol::{PortfolioAction, recommended_action};
use crate::state::{PortfolioChallenge, PortfolioChallengeSpot, SheddingChallenge};

const BENCHMARK_METHOD: &str = "dou-di-zhu-rule-aware-scenario-pack-v1";
const BENCHMARK_METRIC: &str = "engine_recommendation_count";

/// Hash-pinned evidence that the Dou-Di-Zhu `rule-aware` engine was
/// run against the canonical scenario pack and produced a
/// recommendation for every row.
///
/// The threshold is `0.0` because the rule-aware engine is
/// deterministic and the promotion gate is "every scenario produced
/// a recommendation", not a quality score. Higher-quality promotion
/// tiers (notably `promotable_local`) will swap this dossier for a
/// `CanonicalPolicyBundle` that wraps the same scenario table.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DouDiZhuBenchmarkDossier {
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

/// One row of the canonical scenario/answer table the dossier hashes
/// over.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
struct DossierRow {
    scenario_id: String,
    challenge_id: String,
    decision: String,
    bomb_count: u8,
    control_combos: u8,
    low_singles: u8,
    opponents_min_cards: u8,
    danger_opponents: u8,
    next_actor_cards: u8,
    on_lead: bool,
    play_options: u8,
    finishing_plays: u8,
    bomb_only_escape: bool,
    forced_pass: bool,
    lead_rank_pressure: u8,
    engine_tier: String,
    recommended_action: String,
}

impl DouDiZhuBenchmarkDossier {
    /// Build a dossier by running the live `rule-aware` engine
    /// against the canonical Dou-Di-Zhu scenario pack.
    pub fn build(benchmark_id: impl Into<String>) -> Result<Self, DouDiZhuDossierError> {
        Self::build_with_scenarios(benchmark_id, dou_di_zhu_scenario_pack())
    }

    /// Build a dossier from a custom scenario slice (used by tests
    /// and the negative-fixture harnesses).
    pub fn build_with_scenarios(
        benchmark_id: impl Into<String>,
        scenarios: &[DouDiZhuScenario],
    ) -> Result<Self, DouDiZhuDossierError> {
        let mut rows: Vec<DossierRow> = Vec::with_capacity(scenarios.len());
        let mut recommendations: BTreeMap<String, String> = BTreeMap::new();

        for scenario in scenarios {
            let challenge = PortfolioChallenge::DouDiZhu(SheddingChallenge {
                spot: PortfolioChallengeSpot::scenario(
                    ResearchGame::DouDiZhu,
                    scenario.scenario_id,
                    scenario.decision,
                ),
                bomb_count: scenario.bomb_count,
                control_combos: scenario.control_combos,
                low_singles: scenario.low_singles,
                opponents_min_cards: scenario.opponents_min_cards,
                danger_opponents: scenario.danger_opponents,
                next_actor_cards: scenario.next_actor_cards,
                on_lead: scenario.on_lead,
                play_options: scenario.play_options,
                finishing_plays: scenario.finishing_plays,
                bomb_only_escape: scenario.bomb_only_escape,
                forced_pass: scenario.forced_pass,
                lead_rank_pressure: scenario.lead_rank_pressure,
            });

            let (challenge_id, decision) = match &challenge {
                PortfolioChallenge::DouDiZhu(state) => {
                    (state.spot.challenge_id.clone(), state.spot.decision.clone())
                }
                _ => unreachable!("dispatcher only routes DouDiZhu challenges to the dou_di_zhu engine"),
            };

            let answer = answer_typed_challenge(&challenge, 0).map_err(|error| {
                DouDiZhuDossierError::EngineFailed {
                    scenario_id: scenario.scenario_id.to_string(),
                    error: error.to_string(),
                }
            })?;

            let recommendation = recommended_action(&answer.response).ok_or_else(|| {
                DouDiZhuDossierError::NoRecommendation {
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
                bomb_count: scenario.bomb_count,
                control_combos: scenario.control_combos,
                low_singles: scenario.low_singles,
                opponents_min_cards: scenario.opponents_min_cards,
                danger_opponents: scenario.danger_opponents,
                next_actor_cards: scenario.next_actor_cards,
                on_lead: scenario.on_lead,
                play_options: scenario.play_options,
                finishing_plays: scenario.finishing_plays,
                bomb_only_escape: scenario.bomb_only_escape,
                forced_pass: scenario.forced_pass,
                lead_rank_pressure: scenario.lead_rank_pressure,
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
            engine_family: "state-aware bomb-preservation heuristic".to_string(),
            engine_tier: "rule-aware".to_string(),
            rule_file: ResearchGame::DouDiZhu.rule_file().to_string(),
            scenario_hash,
            recommendations,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DouDiZhuDossierError {
    #[error("dou-di-zhu engine failed for scenario {scenario_id}: {error}")]
    EngineFailed { scenario_id: String, error: String },
    #[error("dou-di-zhu engine produced no recommendation for scenario {scenario_id}")]
    NoRecommendation { scenario_id: String },
}

fn action_token(action: PortfolioAction) -> &'static str {
    match action {
        PortfolioAction::PreserveBomb => "preserve-bomb",
        PortfolioAction::LandlordBid => "landlord-bid",
        PortfolioAction::ShedLowest => "shed-lowest",
        // Dou-Di-Zhu engine only emits these three; anything else
        // would be a dispatch regression and is reported as a stable
        // token for tests.
        _ => "unexpected-action",
    }
}

fn hash_rows(rows: &[DossierRow]) -> String {
    // Sort by scenario_id so the hash is independent of the
    // underlying scenario-pack iteration order (the pack is a
    // `&'static []` today but this keeps the dossier verifiable if
    // the pack is later sourced from a non-deterministic iterator).
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
        hasher.update([row.bomb_count]);
        hasher.update([row.control_combos]);
        hasher.update([row.low_singles]);
        hasher.update([row.opponents_min_cards]);
        hasher.update([row.danger_opponents]);
        hasher.update([row.next_actor_cards]);
        hasher.update([u8::from(row.on_lead)]);
        hasher.update([row.play_options]);
        hasher.update([row.finishing_plays]);
        hasher.update([u8::from(row.bomb_only_escape)]);
        hasher.update([u8::from(row.forced_pass)]);
        hasher.update([row.lead_rank_pressure]);
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
    fn dou_di_zhu_dossier_passes_full_scenario_pack() {
        let dossier = DouDiZhuBenchmarkDossier::build("dou-di-zhu-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));

        assert!(dossier.passing, "dossier should pass");
        assert_eq!(
            dossier.scenario_count,
            dou_di_zhu_scenario_pack().len(),
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
    fn dou_di_zhu_dossier_is_deterministic_across_runs() {
        let first = DouDiZhuBenchmarkDossier::build("dou-di-zhu-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));
        let second = DouDiZhuBenchmarkDossier::build("dou-di-zhu-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));

        assert_eq!(first.scenario_hash, second.scenario_hash);
        assert_eq!(first.recommendations, second.recommendations);
    }

    #[test]
    fn dou_di_zhu_dossier_recommendation_map_is_complete() {
        let dossier = DouDiZhuBenchmarkDossier::build("dou-di-zhu-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));

        for scenario in dou_di_zhu_scenario_pack() {
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
                    "preserve-bomb" | "landlord-bid" | "shed-lowest"
                ),
                "scenario {} produced unexpected action token {token}",
                scenario.scenario_id
            );
        }
    }

    #[test]
    fn dou_di_zhu_dossier_preserve_bomb_scenarios_recommend_preserve_bomb() {
        let dossier = DouDiZhuBenchmarkDossier::build("dou-di-zhu-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));

        // Every preserve-bomb-bucket scenario (bomb available, no
        // escape / forced-pass) should pick PreserveBomb. A
        // regression that flipped these to LandlordBid or ShedLowest
        // would be a real heuristic bug.
        for scenario_id in [
            "bomb-rich-mid-race",
            "bomb-bomb-only-just-once",
            "bomb-rich-no-finish",
            "bomb-solo-high-pressure",
            "bomb-clean-mid-rank",
            "bomb-rich-deep",
            "bomb-early-soft",
            "bomb-late-bomb-only",
        ] {
            assert_eq!(
                dossier.recommendations.get(scenario_id).map(String::as_str),
                Some("preserve-bomb"),
                "{scenario_id} should recommend preserve-bomb"
            );
        }
    }

    #[test]
    fn dou_di_zhu_dossier_landlord_bid_scenarios_recommend_landlord_bid() {
        let dossier = DouDiZhuBenchmarkDossier::build("dou-di-zhu-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));

        // Every landlord-bid-bucket scenario (no bombs, on the
        // lead, control-rich) should pick LandlordBid. A regression
        // that flipped these to PreserveBomb or ShedLowest would be
        // a real heuristic bug.
        for scenario_id in [
            "landlord-rich-controls",
            "landlord-mid-controls",
            "landlord-rich-options",
            "landlord-rich-no-double",
            "landlord-mid-pressure",
            "landlord-bid-rich-combo",
            "landlord-clean",
            "landlord-deep-stack",
        ] {
            assert_eq!(
                dossier.recommendations.get(scenario_id).map(String::as_str),
                Some("landlord-bid"),
                "{scenario_id} should recommend landlord-bid"
            );
        }
    }

    #[test]
    fn dou_di_zhu_dossier_shed_lowest_scenarios_recommend_shed_lowest() {
        let dossier = DouDiZhuBenchmarkDossier::build("dou-di-zhu-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));

        // Every shed-lowest-bucket scenario (no bombs, on-lead off,
        // finishing line OR many low singles) should pick
        // ShedLowest. A regression that flipped these to
        // PreserveBomb or LandlordBid would be a real heuristic
        // bug.
        for scenario_id in [
            "shed-finishing-line",
            "shed-many-low-singles",
            "shed-next-actor-short",
            "shed-low-singles-rich",
            "shed-deep-exit",
            "shed-double-finish",
        ] {
            assert_eq!(
                dossier.recommendations.get(scenario_id).map(String::as_str),
                Some("shed-lowest"),
                "{scenario_id} should recommend shed-lowest"
            );
        }
    }

    #[test]
    fn dou_di_zhu_dossier_scenario_count_is_22() {
        let dossier = DouDiZhuBenchmarkDossier::build("dou-di-zhu-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));

        assert_eq!(dossier.scenario_count, 22);
        assert_eq!(dossier.recommendation_count, 22);
        assert!(dossier.passing);
    }
}

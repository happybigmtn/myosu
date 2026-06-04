//! NLHE-six-max benchmark dossier surface.
//!
//! The dossier is the promotion artifact the `F-019` slice ships: it
//! pins the 22-scenario rule-aware scenario pack for NLHE-six-max,
//! runs the live portfolio engine against it, records every engine
//! recommendation, and SHA-256-pins the canonical scenario/answer
//! table so the promotion manifest harness can verify that the
//! evidence attached to a `tier: benchmarked` claim matches the live
//! engine output.
//!
//! NLHE-six-max is the F-019 portfolio-game-promotion slice (the
//! F-001 / F-008 / F-009 / F-010 / F-011 / F-012 / F-013 / F-014 /
//! F-015 / F-016 / F-017 / F-018 thirteenth slice: Cribbage / Hearts /
//! Gin Rummy / Spades / Bridge / Call Break / Backgammon / Hanafuda
//! Koi-Koi / Hwatu Go-Stop / Stratego / PLO / Dou-Di-Zhu). F-019
//! opens the `state-aware poker range heuristic` engine surface in
//! `crate::engines::poker_like::nlhe_six_max` as a second dossier row
//! for the `poker_like` engine sub-family — the F-017 PLO row was
//! the first dossier slice for the `poker_like` family, but its
//! three arms (`draw_to_nuts` / `pot_sized_raise` / `pot_control`)
//! are disjoint from the F-019 NLHE-six-max arms (`value_bet` /
//! `tight_open` / `pot_control`), so a regression that flipped the
//! F-017 PLO math would NOT be caught by the F-019 dossier and vice
//! versa. F-019 is also the first six-max dossier row in the
//! `poker_like` sub-family.
//!
//! The engine has three ranked arms (`value-bet` / `tight-open` /
//! `pot-control`) keyed to made-hand-aggression /
//! steal-folding-pressure / deep-stack-pot-control state. The F-019
//! scenario pack splits 8/8/6 across value-bet-dominant /
//! tight-open-dominant / pot-control-dominant so a regression
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
//! F-016 Stratego, F-017 PLO, and F-018 Dou-Di-Zhu rows landed
//! under).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::core::poker_like::{NlheSixMaxScenario, nlhe_six_max_scenario_pack};
use crate::engine::answer_typed_challenge;
use crate::game::ResearchGame;
use crate::protocol::{PortfolioAction, recommended_action};
use crate::state::{PokerLikeChallenge, PortfolioChallenge, PortfolioChallengeSpot};

const BENCHMARK_METHOD: &str = "nlhe-six-max-rule-aware-scenario-pack-v1";
const BENCHMARK_METRIC: &str = "engine_recommendation_count";

/// Hash-pinned evidence that the NLHE-six-max `rule-aware` engine
/// was run against the canonical scenario pack and produced a
/// recommendation for every row.
///
/// The threshold is `0.0` because the rule-aware engine is
/// deterministic and the promotion gate is "every scenario produced
/// a recommendation", not a quality score. Higher-quality promotion
/// tiers (notably `promotable_local`) will swap this dossier for a
/// `CanonicalPolicyBundle` that wraps the same scenario table.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct NlheSixMaxBenchmarkDossier {
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
    pot_bb: u16,
    effective_stack_bb: u16,
    made_strength: u8,
    draw_strength: u8,
    fold_equity: u8,
    to_call_bb: u16,
    active_players: u8,
    check_available: bool,
    raise_available: bool,
    in_position: bool,
    icm_pressure: u8,
    has_seen_cards: bool,
    engine_tier: String,
    recommended_action: String,
}

impl NlheSixMaxBenchmarkDossier {
    /// Build a dossier by running the live `rule-aware` engine
    /// against the canonical NLHE-six-max scenario pack.
    pub fn build(benchmark_id: impl Into<String>) -> Result<Self, NlheSixMaxDossierError> {
        Self::build_with_scenarios(benchmark_id, nlhe_six_max_scenario_pack())
    }

    /// Build a dossier from a custom scenario slice (used by tests
    /// and the negative-fixture harnesses).
    pub fn build_with_scenarios(
        benchmark_id: impl Into<String>,
        scenarios: &[NlheSixMaxScenario],
    ) -> Result<Self, NlheSixMaxDossierError> {
        let mut rows: Vec<DossierRow> = Vec::with_capacity(scenarios.len());
        let mut recommendations: BTreeMap<String, String> = BTreeMap::new();

        for scenario in scenarios {
            let challenge = PortfolioChallenge::NlheSixMax(PokerLikeChallenge {
                spot: PortfolioChallengeSpot::scenario(
                    ResearchGame::NlheSixMax,
                    scenario.scenario_id,
                    scenario.decision,
                ),
                pot_bb: scenario.pot_bb,
                effective_stack_bb: scenario.effective_stack_bb,
                made_strength: scenario.made_strength,
                draw_strength: scenario.draw_strength,
                fold_equity: scenario.fold_equity,
                to_call_bb: scenario.to_call_bb,
                active_players: scenario.active_players,
                check_available: scenario.check_available,
                raise_available: scenario.raise_available,
                in_position: scenario.in_position,
                icm_pressure: scenario.icm_pressure,
                has_seen_cards: scenario.has_seen_cards,
            });

            let (challenge_id, decision) = match &challenge {
                PortfolioChallenge::NlheSixMax(state) => {
                    (state.spot.challenge_id.clone(), state.spot.decision.clone())
                }
                _ => unreachable!(
                    "dispatcher only routes NlheSixMax challenges to the nlhe_six_max engine"
                ),
            };

            let answer = answer_typed_challenge(&challenge, 0).map_err(|error| {
                NlheSixMaxDossierError::EngineFailed {
                    scenario_id: scenario.scenario_id.to_string(),
                    error: error.to_string(),
                }
            })?;

            let recommendation = recommended_action(&answer.response).ok_or_else(|| {
                NlheSixMaxDossierError::NoRecommendation {
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
                pot_bb: scenario.pot_bb,
                effective_stack_bb: scenario.effective_stack_bb,
                made_strength: scenario.made_strength,
                draw_strength: scenario.draw_strength,
                fold_equity: scenario.fold_equity,
                to_call_bb: scenario.to_call_bb,
                active_players: scenario.active_players,
                check_available: scenario.check_available,
                raise_available: scenario.raise_available,
                in_position: scenario.in_position,
                icm_pressure: scenario.icm_pressure,
                has_seen_cards: scenario.has_seen_cards,
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
            engine_family: "state-aware poker range heuristic".to_string(),
            engine_tier: "rule-aware".to_string(),
            rule_file: ResearchGame::NlheSixMax.rule_file().to_string(),
            scenario_hash,
            recommendations,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum NlheSixMaxDossierError {
    #[error("nlhe-six-max engine failed for scenario {scenario_id}: {error}")]
    EngineFailed { scenario_id: String, error: String },
    #[error("nlhe-six-max engine produced no recommendation for scenario {scenario_id}")]
    NoRecommendation { scenario_id: String },
}

fn action_token(action: PortfolioAction) -> &'static str {
    match action {
        PortfolioAction::ValueBet => "value-bet",
        PortfolioAction::TightOpen => "tight-open",
        PortfolioAction::PotControl => "pot-control",
        // NLHE-six-max engine only emits these three; anything else
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
        hasher.update(row.pot_bb.to_le_bytes());
        hasher.update(row.effective_stack_bb.to_le_bytes());
        hasher.update([row.made_strength]);
        hasher.update([row.draw_strength]);
        hasher.update([row.fold_equity]);
        hasher.update(row.to_call_bb.to_le_bytes());
        hasher.update([row.active_players]);
        hasher.update([u8::from(row.check_available)]);
        hasher.update([u8::from(row.raise_available)]);
        hasher.update([u8::from(row.in_position)]);
        hasher.update([row.icm_pressure]);
        hasher.update([u8::from(row.has_seen_cards)]);
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
    fn nlhe_six_max_dossier_passes_full_scenario_pack() {
        let dossier = NlheSixMaxBenchmarkDossier::build("nlhe-six-max-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));

        assert!(dossier.passing, "dossier should pass");
        assert_eq!(
            dossier.scenario_count,
            nlhe_six_max_scenario_pack().len(),
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
    fn nlhe_six_max_dossier_is_deterministic_across_runs() {
        let first = NlheSixMaxBenchmarkDossier::build("nlhe-six-max-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));
        let second = NlheSixMaxBenchmarkDossier::build("nlhe-six-max-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));

        assert_eq!(first.scenario_hash, second.scenario_hash);
        assert_eq!(first.recommendations, second.recommendations);
    }

    #[test]
    fn nlhe_six_max_dossier_recommendation_map_is_complete() {
        let dossier = NlheSixMaxBenchmarkDossier::build("nlhe-six-max-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));

        for scenario in nlhe_six_max_scenario_pack() {
            let action = dossier
                .recommendations
                .get(scenario.scenario_id)
                .unwrap_or_else(|| panic!("missing recommendation for {}", scenario.scenario_id));
            assert!(
                matches!(
                    action.as_str(),
                    "value-bet" | "tight-open" | "pot-control"
                ),
                "scenario {} has unexpected action {}",
                scenario.scenario_id,
                action
            );
        }
    }

    #[test]
    fn nlhe_six_max_dossier_value_bet_scenarios_recommend_value_bet() {
        let dossier = NlheSixMaxBenchmarkDossier::build("nlhe-six-max-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));

        let value_bet = [
            "value-mvp-flop-bet",
            "value-top-pair-position",
            "value-overpair",
            "value-strong-hand-call",
            "value-set-vs-raise",
            "value-overpair-3way",
            "value-monster-no-raise",
            "value-medium-in-position",
        ];
        for scenario_id in value_bet {
            let action = dossier
                .recommendations
                .get(scenario_id)
                .unwrap_or_else(|| panic!("missing recommendation for {scenario_id}"));
            assert_eq!(
                action, "value-bet",
                "scenario {scenario_id} expected value-bet but got {action}"
            );
        }
    }

    #[test]
    fn nlhe_six_max_dossier_tight_open_scenarios_recommend_tight_open() {
        let dossier = NlheSixMaxBenchmarkDossier::build("nlhe-six-max-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));

        let tight_open = [
            "tight-clean-steal",
            "tight-fold-rich-blind",
            "tight-mid-position-open",
            "tight-3way-steal",
            "tight-fold-rich-small-blind",
            "tight-low-fe-2way",
            "tight-3way-no-blind-bonus",
            "tight-4way-rich-fold",
        ];
        for scenario_id in tight_open {
            let action = dossier
                .recommendations
                .get(scenario_id)
                .unwrap_or_else(|| panic!("missing recommendation for {scenario_id}"));
            assert_eq!(
                action, "tight-open",
                "scenario {scenario_id} expected tight-open but got {action}"
            );
        }
    }

    #[test]
    fn nlhe_six_max_dossier_pot_control_scenarios_recommend_pot_control() {
        let dossier = NlheSixMaxBenchmarkDossier::build("nlhe-six-max-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));

        let pot_control = [
            "control-deep-check",
            "control-3way-deep",
            "control-shallow-deep-no-check",
            "control-3way-mid-stack",
            "control-deep-heavy-draw",
            "control-mid-stack-mild-draw",
        ];
        for scenario_id in pot_control {
            let action = dossier
                .recommendations
                .get(scenario_id)
                .unwrap_or_else(|| panic!("missing recommendation for {scenario_id}"));
            assert_eq!(
                action, "pot-control",
                "scenario {scenario_id} expected pot-control but got {action}"
            );
        }
    }

    #[test]
    fn nlhe_six_max_dossier_scenario_count_is_22() {
        let dossier = NlheSixMaxBenchmarkDossier::build("nlhe-six-max-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));
        assert_eq!(dossier.scenario_count, 22);
    }
}

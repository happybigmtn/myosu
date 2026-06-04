//! PLO benchmark dossier surface.
//!
//! The dossier is the promotion artifact the `F-017` slice ships: it
//! pins the 22-scenario rule-aware scenario pack for PLO, runs the
//! live portfolio engine against it, records every engine
//! recommendation, and SHA-256-pins the canonical scenario/answer
//! table so the promotion manifest harness can verify that the
//! evidence attached to a `tier: benchmarked` claim matches the live
//! engine output.
//!
//! PLO is the F-017 portfolio-game-promotion slice (the
//! F-001/F-008/F-009/F-010/F-011/F-012/F-013/F-014/F-015/F-016
//! eleventh slice: Cribbage / Hearts / Gin Rummy / Spades / Bridge /
//! Call Break / Backgammon / Hanafuda Koi-Koi / Hwatu Go-Stop /
//! Stratego). F-017 opens the `state-aware PLO nut-draw heuristic`
//! engine sub-family in `crate::engines::poker_like::plo` — no other
//! portfolio game has a dossier row for the `poker_like` engine
//! family (nlhe-six-max, plo, nlhe-tournament, short-deck, teen-patti
//! all share the `PokerLikeChallenge` struct, but F-017 PLO is the
//! first dossier slice for this sub-family).
//!
//! The engine has three ranked arms (`draw_to_nuts` /
//! `pot_sized_raise` / `pot_control`) keyed to nut-draw-heavy /
//! made-hand-aggression / deep-stack-pot-control state. The F-017
//! scenario pack splits 8/8/6 across draw-to-nuts-dominant /
//! pot-sized-raise-dominant / pot-control-dominant so a regression
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
//! and F-016 Stratego rows landed under).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::core::poker_like::{PloScenario, plo_scenario_pack};
use crate::engine::answer_typed_challenge;
use crate::game::ResearchGame;
use crate::protocol::{PortfolioAction, recommended_action};
use crate::state::{PokerLikeChallenge, PortfolioChallenge, PortfolioChallengeSpot};

const BENCHMARK_METHOD: &str = "plo-rule-aware-scenario-pack-v1";
const BENCHMARK_METRIC: &str = "engine_recommendation_count";

/// Hash-pinned evidence that the PLO `rule-aware` engine was run
/// against the canonical scenario pack and produced a recommendation
/// for every row.
///
/// The threshold is `0.0` because the rule-aware engine is
/// deterministic and the promotion gate is "every scenario produced
/// a recommendation", not a quality score. Higher-quality promotion
/// tiers (notably `promotable_local`) will swap this dossier for a
/// `CanonicalPolicyBundle` that wraps the same scenario table.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PloBenchmarkDossier {
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

impl PloBenchmarkDossier {
    /// Build a dossier by running the live `rule-aware` engine
    /// against the canonical PLO scenario pack.
    pub fn build(benchmark_id: impl Into<String>) -> Result<Self, PloDossierError> {
        Self::build_with_scenarios(benchmark_id, plo_scenario_pack())
    }

    /// Build a dossier from a custom scenario slice (used by tests
    /// and the negative-fixture harnesses).
    pub fn build_with_scenarios(
        benchmark_id: impl Into<String>,
        scenarios: &[PloScenario],
    ) -> Result<Self, PloDossierError> {
        let mut rows: Vec<DossierRow> = Vec::with_capacity(scenarios.len());
        let mut recommendations: BTreeMap<String, String> = BTreeMap::new();

        for scenario in scenarios {
            let challenge = PortfolioChallenge::Plo(PokerLikeChallenge {
                spot: PortfolioChallengeSpot::scenario(
                    ResearchGame::Plo,
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
                PortfolioChallenge::Plo(state) => {
                    (state.spot.challenge_id.clone(), state.spot.decision.clone())
                }
                _ => unreachable!("dispatcher only routes PLO challenges to the plo engine"),
            };

            let answer = answer_typed_challenge(&challenge, 0).map_err(|error| {
                PloDossierError::EngineFailed {
                    scenario_id: scenario.scenario_id.to_string(),
                    error: error.to_string(),
                }
            })?;

            let recommendation = recommended_action(&answer.response).ok_or_else(|| {
                PloDossierError::NoRecommendation {
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
            engine_family: "state-aware PLO nut-draw heuristic".to_string(),
            engine_tier: "rule-aware".to_string(),
            rule_file: ResearchGame::Plo.rule_file().to_string(),
            scenario_hash,
            recommendations,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PloDossierError {
    #[error("plo engine failed for scenario {scenario_id}: {error}")]
    EngineFailed { scenario_id: String, error: String },
    #[error("plo engine produced no recommendation for scenario {scenario_id}")]
    NoRecommendation { scenario_id: String },
}

fn action_token(action: PortfolioAction) -> &'static str {
    match action {
        PortfolioAction::DrawToNuts => "draw-to-nuts",
        PortfolioAction::PotSizedRaise => "pot-sized-raise",
        PortfolioAction::PotControl => "pot-control",
        // PLO engine only emits these three; anything else would be
        // a dispatch regression and is reported as a stable token
        // for tests.
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
    fn plo_dossier_passes_full_scenario_pack() {
        let dossier = PloBenchmarkDossier::build("plo-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));

        assert!(dossier.passing, "dossier should pass");
        assert_eq!(
            dossier.scenario_count,
            plo_scenario_pack().len(),
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
    fn plo_dossier_is_deterministic_across_runs() {
        let first = PloBenchmarkDossier::build("plo-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));
        let second = PloBenchmarkDossier::build("plo-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));

        assert_eq!(first.scenario_hash, second.scenario_hash);
        assert_eq!(first.recommendations, second.recommendations);
    }

    #[test]
    fn plo_dossier_recommendation_map_is_complete() {
        let dossier = PloBenchmarkDossier::build("plo-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));

        for scenario in plo_scenario_pack() {
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
                    "draw-to-nuts" | "pot-sized-raise" | "pot-control"
                ),
                "scenario {} produced unexpected action token {token}",
                scenario.scenario_id
            );
        }
    }

    #[test]
    fn plo_dossier_draw_to_nuts_scenarios_recommend_draw_to_nuts() {
        let dossier = PloBenchmarkDossier::build("plo-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));

        // Every draw-to-nuts-bucket scenario (high draw_strength,
        // facing a call) should pick DrawToNuts. A regression that
        // flipped these to PotSizedRaise or PotControl would be a
        // real heuristic bug.
        for scenario_id in [
            "draw-nuts-balanced-call",
            "draw-nuts-heavy-draw",
            "draw-nuts-big-pot",
            "draw-nuts-out-of-pos",
            "draw-nuts-small-draw",
            "draw-nuts-monster-draw",
            "draw-nuts-double-suited",
            "draw-nuts-multiway",
        ] {
            assert_eq!(
                dossier.recommendations.get(scenario_id).map(String::as_str),
                Some("draw-to-nuts"),
                "{scenario_id} should recommend draw-to-nuts"
            );
        }
    }

    #[test]
    fn plo_dossier_pot_sized_raise_scenarios_recommend_pot_sized_raise() {
        let dossier = PloBenchmarkDossier::build("plo-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));

        // Every pot-sized-raise-bucket scenario (made hand + fold
        // equity + raise available) should pick PotSizedRaise. A
        // regression that flipped these to DrawToNuts or PotControl
        // would be a real heuristic bug.
        for scenario_id in [
            "raise-mvp-can-raise",
            "raise-strong-made",
            "raise-folding-station",
            "raise-three-way-fold",
            "raise-medium-made",
            "raise-the-nuts",
            "raise-thick-value",
            "raise-pure-fold-equity",
        ] {
            assert_eq!(
                dossier.recommendations.get(scenario_id).map(String::as_str),
                Some("pot-sized-raise"),
                "{scenario_id} should recommend pot-sized-raise"
            );
        }
    }

    #[test]
    fn plo_dossier_pot_control_scenarios_recommend_pot_control() {
        let dossier = PloBenchmarkDossier::build("plo-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));

        // Every pot-control-bucket scenario (deep stack + check
        // available + multi-way) should pick PotControl. A
        // regression that flipped these to DrawToNuts or
        // PotSizedRaise would be a real heuristic bug.
        for scenario_id in [
            "control-deep-checked",
            "control-deep-multi",
            "control-shallow-multi",
            "control-five-way",
            "control-three-way",
            "control-weak-draw",
        ] {
            assert_eq!(
                dossier.recommendations.get(scenario_id).map(String::as_str),
                Some("pot-control"),
                "{scenario_id} should recommend pot-control"
            );
        }
    }

    #[test]
    fn plo_dossier_scenario_count_is_22() {
        let dossier = PloBenchmarkDossier::build("plo-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));

        assert_eq!(dossier.scenario_count, 22);
        assert_eq!(dossier.recommendation_count, 22);
        assert!(dossier.passing);
    }
}

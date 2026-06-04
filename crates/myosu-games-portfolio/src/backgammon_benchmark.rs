//! Backgammon benchmark dossier surface.
//!
//! The dossier is the promotion artifact the `F-013` slice ships: it
//! pins the 22-scenario rule-aware scenario pack for Backgammon, runs
//! the live portfolio engine against it, records every engine
//! recommendation, and SHA-256-pins the canonical scenario/answer
//! table so the promotion manifest harness can verify that the
//! evidence attached to a `tier: benchmarked` claim matches the live
//! engine output.
//!
//! Backgammon is the F-013 portfolio-game-promotion slice, scoped to
//! the self-contained `state-aware race-contact heuristic` engine
//! surface in `crate::engines::backgammon::answer`. The engine has
//! three ranked arms (`bear_off` / `accept_double` / `advance_piece`)
//! keyed to race/contact/cube state; the F-013 scenario pack splits
//! 7/7/7/1 across bear_off-dominant / accept_double-dominant /
//! advance_piece-dominant / mixed-edge so a regression that flips
//! the dominant arm on any scenario is loud in the dossier's
//! recommendation map.
//!
//! Backgammon is the only `routed` game in the `backgammon` engine
//! family (no other portfolio game shares the `state-aware
//! race-contact heuristic` engine surface), so F-013 is the
//! natural next-portfolio-game promotion slice after the
//! F-001/F-008/F-009/F-010/F-011/F-012 trick_taking / gin_rummy /
//! cribbage dossiers landed.
//!
//! The dossier is intentionally `benchmarked`-tier, not
//! `promotable_local`: the policy bundle builder
//! (`genesis/plans/001-master-plan.md`) is designed for dedicated
//! games initially, and generalizing it to portfolio games is
//! follow-on work (the same scope boundary the F-001 Cribbage, F-008
//! Hearts, F-009 Gin Rummy, F-010 Spades, F-011 Bridge, and F-012
//! Call Break rows landed under).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::core::backgammon::{BackgammonScenario, backgammon_scenario_pack};
use crate::engine::answer_typed_challenge;
use crate::game::ResearchGame;
use crate::protocol::{PortfolioAction, recommended_action};
use crate::state::{BackgammonChallenge, PortfolioChallenge, PortfolioChallengeSpot};

const BENCHMARK_METHOD: &str = "backgammon-rule-aware-scenario-pack-v1";
const BENCHMARK_METRIC: &str = "engine_recommendation_count";

/// Hash-pinned evidence that the Backgammon `rule-aware` engine was
/// run against the canonical scenario pack and produced a
/// recommendation for every row.
///
/// The threshold is `0.0` because the rule-aware engine is
/// deterministic and the promotion gate is "every scenario produced
/// a recommendation", not a quality score. Higher-quality promotion
/// tiers (notably `promotable_local`) will swap this dossier for a
/// `CanonicalPolicyBundle` that wraps the same scenario table.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BackgammonBenchmarkDossier {
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
    race_lead_pips: i16,
    borne_off: u8,
    anchors: u8,
    cube_efficiency: u8,
    has_contact: bool,
    bar_count: u8,
    bearoff_ready: bool,
    cube_centered: bool,
    cube_owned_by_actor: bool,
    facing_double: bool,
    move_options: u8,
    off_moves: u8,
    blot_count: u8,
    home_board_points: u8,
    prime_length: u8,
    engine_tier: String,
    recommended_action: String,
}

impl BackgammonBenchmarkDossier {
    /// Build a dossier by running the live `rule-aware` engine
    /// against the canonical Backgammon scenario pack.
    pub fn build(benchmark_id: impl Into<String>) -> Result<Self, BackgammonDossierError> {
        Self::build_with_scenarios(benchmark_id, backgammon_scenario_pack())
    }

    /// Build a dossier from a custom scenario slice (used by tests
    /// and the negative-fixture harnesses).
    pub fn build_with_scenarios(
        benchmark_id: impl Into<String>,
        scenarios: &[BackgammonScenario],
    ) -> Result<Self, BackgammonDossierError> {
        let mut rows: Vec<DossierRow> = Vec::with_capacity(scenarios.len());
        let mut recommendations: BTreeMap<String, String> = BTreeMap::new();

        for scenario in scenarios {
            let challenge = PortfolioChallenge::Backgammon(BackgammonChallenge {
                spot: PortfolioChallengeSpot::scenario(
                    ResearchGame::Backgammon,
                    scenario.scenario_id,
                    scenario.decision,
                ),
                race_lead_pips: scenario.race_lead_pips,
                borne_off: scenario.borne_off,
                anchors: scenario.anchors,
                cube_efficiency: scenario.cube_efficiency,
                has_contact: scenario.has_contact,
                bar_count: scenario.bar_count,
                bearoff_ready: scenario.bearoff_ready,
                cube_centered: scenario.cube_centered,
                cube_owned_by_actor: scenario.cube_owned_by_actor,
                facing_double: scenario.facing_double,
                move_options: scenario.move_options,
                off_moves: scenario.off_moves,
                blot_count: scenario.blot_count,
                home_board_points: scenario.home_board_points,
                prime_length: scenario.prime_length,
            });

            let (challenge_id, decision) = match &challenge {
                PortfolioChallenge::Backgammon(state) => {
                    (state.spot.challenge_id.clone(), state.spot.decision.clone())
                }
                _ => unreachable!(
                    "dispatcher only routes Backgammon challenges to the backgammon engine"
                ),
            };

            let answer = answer_typed_challenge(&challenge, 0).map_err(|error| {
                BackgammonDossierError::EngineFailed {
                    scenario_id: scenario.scenario_id.to_string(),
                    error: error.to_string(),
                }
            })?;

            let recommendation = recommended_action(&answer.response).ok_or_else(|| {
                BackgammonDossierError::NoRecommendation {
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
                race_lead_pips: scenario.race_lead_pips,
                borne_off: scenario.borne_off,
                anchors: scenario.anchors,
                cube_efficiency: scenario.cube_efficiency,
                has_contact: scenario.has_contact,
                bar_count: scenario.bar_count,
                bearoff_ready: scenario.bearoff_ready,
                cube_centered: scenario.cube_centered,
                cube_owned_by_actor: scenario.cube_owned_by_actor,
                facing_double: scenario.facing_double,
                move_options: scenario.move_options,
                off_moves: scenario.off_moves,
                blot_count: scenario.blot_count,
                home_board_points: scenario.home_board_points,
                prime_length: scenario.prime_length,
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
            engine_family: "state-aware race-contact heuristic".to_string(),
            engine_tier: "rule-aware".to_string(),
            rule_file: ResearchGame::Backgammon.rule_file().to_string(),
            scenario_hash,
            recommendations,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BackgammonDossierError {
    #[error("backgammon engine failed for scenario {scenario_id}: {error}")]
    EngineFailed { scenario_id: String, error: String },
    #[error("backgammon engine produced no recommendation for scenario {scenario_id}")]
    NoRecommendation { scenario_id: String },
}

fn action_token(action: PortfolioAction) -> &'static str {
    match action {
        PortfolioAction::BearOff => "bear-off",
        PortfolioAction::AcceptDouble => "accept-double",
        PortfolioAction::AdvancePiece => "advance-piece",
        // Backgammon engine only emits these three; anything else
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
        hasher.update(row.race_lead_pips.to_le_bytes());
        hasher.update([row.borne_off]);
        hasher.update([row.anchors]);
        hasher.update([row.cube_efficiency]);
        hasher.update([u8::from(row.has_contact)]);
        hasher.update([row.bar_count]);
        hasher.update([u8::from(row.bearoff_ready)]);
        hasher.update([u8::from(row.cube_centered)]);
        hasher.update([u8::from(row.cube_owned_by_actor)]);
        hasher.update([u8::from(row.facing_double)]);
        hasher.update([row.move_options]);
        hasher.update([row.off_moves]);
        hasher.update([row.blot_count]);
        hasher.update([row.home_board_points]);
        hasher.update([row.prime_length]);
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
    fn backgammon_dossier_passes_full_scenario_pack() {
        let dossier = BackgammonBenchmarkDossier::build("backgammon-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));

        assert!(dossier.passing, "dossier should pass");
        assert_eq!(
            dossier.scenario_count,
            backgammon_scenario_pack().len(),
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
    fn backgammon_dossier_is_deterministic_across_runs() {
        let first = BackgammonBenchmarkDossier::build("backgammon-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));
        let second = BackgammonBenchmarkDossier::build("backgammon-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));

        assert_eq!(first.scenario_hash, second.scenario_hash);
        assert_eq!(first.recommendations, second.recommendations);
    }

    #[test]
    fn backgammon_dossier_recommendation_map_is_complete() {
        let dossier = BackgammonBenchmarkDossier::build("backgammon-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));

        for scenario in backgammon_scenario_pack() {
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
                    "bear-off" | "accept-double" | "advance-piece"
                ),
                "scenario {} produced unexpected action token {token}",
                scenario.scenario_id
            );
        }
    }

    #[test]
    fn backgammon_dossier_bear_off_scenarios_recommend_bear_off() {
        let dossier = BackgammonBenchmarkDossier::build("backgammon-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));

        // Every bear_off-bucket scenario (strong race with no contact,
        // no facing double) should pick BearOff. A regression that
        // flipped these to AcceptDouble or AdvancePiece would be a
        // real heuristic bug.
        for scenario_id in [
            "bo-pristine-bearoff",
            "bo-clean-rno-double",
            "bo-bearoff-thick",
            "bo-bearoff-medium-race",
            "bo-racing-home-empty",
            "bo-bearoff-cube-owned",
            "bo-bearoff-clean-zero-blot",
        ] {
            assert_eq!(
                dossier.recommendations.get(scenario_id).map(String::as_str),
                Some("bear-off"),
                "{scenario_id} should recommend bear-off"
            );
        }
    }

    #[test]
    fn backgammon_dossier_accept_double_scenarios_recommend_accept_double() {
        let dossier = BackgammonBenchmarkDossier::build("backgammon-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));

        // Every accept_double-bucket scenario (facing a double with a
        // live take: positive race, ≥4 off-moves, no blot disaster)
        // should pick AcceptDouble. A regression that flipped these
        // to BearOff or AdvancePiece would be a real heuristic bug.
        for scenario_id in [
            "ad-live-take",
            "ad-secure-take",
            "ad-deep-take",
            "ad-clean-take-blotless",
            "ad-take-with-prime",
            "ad-cube-efficiency-rich",
            "ad-small-bearoff-take",
        ] {
            assert_eq!(
                dossier.recommendations.get(scenario_id).map(String::as_str),
                Some("accept-double"),
                "{scenario_id} should recommend accept-double"
            );
        }
    }

    #[test]
    fn backgammon_dossier_advance_piece_scenarios_recommend_advance_piece() {
        let dossier = BackgammonBenchmarkDossier::build("backgammon-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));

        // Every advance_piece-bucket scenario (contact position with
        // anchors and/or bar) should pick AdvancePiece. A regression
        // that flipped these to BearOff or AcceptDouble would be a
        // real heuristic bug.
        for scenario_id in [
            "ap-contact-pristine",
            "ap-prime-and-bar",
            "ap-contact-bar-entry",
            "ap-blotty-prime-midgame",
            "ap-crashed-contact-bar",
            "ap-blot-rescue",
            "ap-contact-prime-deep",
        ] {
            assert_eq!(
                dossier.recommendations.get(scenario_id).map(String::as_str),
                Some("advance-piece"),
                "{scenario_id} should recommend advance-piece"
            );
        }
    }
}

//! HwatuGoStop benchmark dossier surface.
//!
//! The dossier is the promotion artifact the `F-015` slice ships: it
//! pins the 22-scenario rule-aware scenario pack for HwatuGoStop,
//! runs the live portfolio engine against it, records every engine
//! recommendation, and SHA-256-pins the canonical scenario/answer
//! table so the promotion manifest harness can verify that the
//! evidence attached to a `tier: benchmarked` claim matches the live
//! engine output.
//!
//! HwatuGoStop is the F-015 portfolio-game-promotion slice (the
//! F-001/F-008/F-009/F-010/F-011/F-012/F-013/F-014 ninth slice:
//! Cribbage / Hearts / Gin Rummy / Spades / Bridge / Call Break /
//! Backgammon / Hanafuda Koi-Koi / Hwatu Go-Stop). F-015 is the
//! second dossier slice for the flower-card-capture family: it
//! reuses the `HanafudaChallenge` feature struct from F-014 (the
//! same 15-field `HanafudaChallenge` is shared because both
//! HanafudaKoiKoi and HwatuGoStop route through the same
//! `crate::state::PortfolioChallenge` enum), but it dispatches to
//! the distinct `hwatu_go_stop` arm of `crate::engines::hanafuda`
//! (F-014 used the `koi_koi` arm). The two arms share no heuristic
//! coefficients — the call_go boost is keyed to `bonus_cards * 0.30
//! + upside_gain * 0.38` (vs the koi_koi arm's `upside_gain * 0.40 +
//! bright_capture_options * 0.24`), so a regression that swapped the
//! arm for the wrong game would produce a different recommendation
//! map and the dossier would catch it.
//!
//! The engine has three ranked arms (`call_go` / `stop_round` /
//! `koi_koi`) keyed to bonus-heavy-fresh / banked-cash-out /
//! stretched-window state. The F-015 scenario pack splits 8/8/6
//! across call_go-dominant / stop_round-dominant / mixed-edge so a
//! regression that flips the dominant arm on any scenario is loud
//! in the dossier's recommendation map.
//!
//! The dossier is intentionally `benchmarked`-tier, not
//! `promotable_local`: the policy bundle builder
//! (`genesis/plans/001-master-plan.md`) is designed for dedicated
//! games initially, and generalizing it to portfolio games is
//! follow-on work (the same scope boundary the F-001 Cribbage, F-008
//! Hearts, F-009 Gin Rummy, F-010 Spades, F-011 Bridge, F-012 Call
//! Break, F-013 Backgammon, and F-014 HanafudaKoiKoi rows landed
//! under).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::core::hanafuda::{HwatuScenario, hwatu_scenario_pack};
use crate::engine::answer_typed_challenge;
use crate::game::ResearchGame;
use crate::protocol::{PortfolioAction, recommended_action};
use crate::state::{HanafudaChallenge, PortfolioChallenge, PortfolioChallengeSpot};

const BENCHMARK_METHOD: &str = "hwatu-go-stop-rule-aware-scenario-pack-v1";
const BENCHMARK_METRIC: &str = "engine_recommendation_count";

/// Hash-pinned evidence that the HwatuGoStop `rule-aware` engine was
/// run against the canonical scenario pack and produced a
/// recommendation for every row.
///
/// The threshold is `0.0` because the rule-aware engine is
/// deterministic and the promotion gate is "every scenario produced
/// a recommendation", not a quality score. Higher-quality promotion
/// tiers (notably `promotable_local`) will swap this dossier for a
/// `CanonicalPolicyBundle` that wraps the same scenario table.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct HwatuBenchmarkDossier {
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
    points: u8,
    bright_count: u8,
    ribbon_yaku: u8,
    animal_yaku: u8,
    bonus_cards: u8,
    yaku_count: u8,
    bright_capture_options: u8,
    opponent_pressure: u8,
    hand_count: u8,
    decision_window: bool,
    locked_points: u8,
    continuation_calls: u8,
    upside_capture_options: u8,
    max_upside_gain: u8,
    engine_tier: String,
    recommended_action: String,
}

impl HwatuBenchmarkDossier {
    /// Build a dossier by running the live `rule-aware` engine
    /// against the canonical HwatuGoStop scenario pack.
    pub fn build(benchmark_id: impl Into<String>) -> Result<Self, HwatuDossierError> {
        Self::build_with_scenarios(benchmark_id, hwatu_scenario_pack())
    }

    /// Build a dossier from a custom scenario slice (used by tests
    /// and the negative-fixture harnesses).
    pub fn build_with_scenarios(
        benchmark_id: impl Into<String>,
        scenarios: &[HwatuScenario],
    ) -> Result<Self, HwatuDossierError> {
        let mut rows: Vec<DossierRow> = Vec::with_capacity(scenarios.len());
        let mut recommendations: BTreeMap<String, String> = BTreeMap::new();

        for scenario in scenarios {
            let challenge = PortfolioChallenge::HwatuGoStop(HanafudaChallenge {
                spot: PortfolioChallengeSpot::scenario(
                    ResearchGame::HwatuGoStop,
                    scenario.scenario_id,
                    scenario.decision,
                ),
                points: scenario.points,
                bright_count: scenario.bright_count,
                ribbon_yaku: scenario.ribbon_yaku,
                animal_yaku: scenario.animal_yaku,
                bonus_cards: scenario.bonus_cards,
                yaku_count: scenario.yaku_count,
                bright_capture_options: scenario.bright_capture_options,
                opponent_pressure: scenario.opponent_pressure,
                hand_count: scenario.hand_count,
                decision_window: scenario.decision_window,
                locked_points: scenario.locked_points,
                continuation_calls: scenario.continuation_calls,
                upside_capture_options: scenario.upside_capture_options,
                max_upside_gain: scenario.max_upside_gain,
            });

            let (challenge_id, decision) = match &challenge {
                PortfolioChallenge::HwatuGoStop(state) => {
                    (state.spot.challenge_id.clone(), state.spot.decision.clone())
                }
                _ => unreachable!(
                    "dispatcher only routes HwatuGoStop challenges to the hanafuda engine"
                ),
            };

            let answer = answer_typed_challenge(&challenge, 0).map_err(|error| {
                HwatuDossierError::EngineFailed {
                    scenario_id: scenario.scenario_id.to_string(),
                    error: error.to_string(),
                }
            })?;

            let recommendation = recommended_action(&answer.response).ok_or_else(|| {
                HwatuDossierError::NoRecommendation {
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
                points: scenario.points,
                bright_count: scenario.bright_count,
                ribbon_yaku: scenario.ribbon_yaku,
                animal_yaku: scenario.animal_yaku,
                bonus_cards: scenario.bonus_cards,
                yaku_count: scenario.yaku_count,
                bright_capture_options: scenario.bright_capture_options,
                opponent_pressure: scenario.opponent_pressure,
                hand_count: scenario.hand_count,
                decision_window: scenario.decision_window,
                locked_points: scenario.locked_points,
                continuation_calls: scenario.continuation_calls,
                upside_capture_options: scenario.upside_capture_options,
                max_upside_gain: scenario.max_upside_gain,
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
            engine_family: "state-aware go-stop bonus heuristic".to_string(),
            engine_tier: "rule-aware".to_string(),
            rule_file: ResearchGame::HwatuGoStop.rule_file().to_string(),
            scenario_hash,
            recommendations,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum HwatuDossierError {
    #[error("hwatu engine failed for scenario {scenario_id}: {error}")]
    EngineFailed { scenario_id: String, error: String },
    #[error("hwatu engine produced no recommendation for scenario {scenario_id}")]
    NoRecommendation { scenario_id: String },
}

fn action_token(action: PortfolioAction) -> &'static str {
    match action {
        PortfolioAction::CallGo => "call-go",
        PortfolioAction::StopRound => "stop-round",
        PortfolioAction::KoiKoi => "koi-koi",
        // HwatuGoStop engine only emits these three; anything else
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
        hasher.update([row.points]);
        hasher.update([row.bright_count]);
        hasher.update([row.ribbon_yaku]);
        hasher.update([row.animal_yaku]);
        hasher.update([row.bonus_cards]);
        hasher.update([row.yaku_count]);
        hasher.update([row.bright_capture_options]);
        hasher.update([row.opponent_pressure]);
        hasher.update([row.hand_count]);
        hasher.update([u8::from(row.decision_window)]);
        hasher.update([row.locked_points]);
        hasher.update([row.continuation_calls]);
        hasher.update([row.upside_capture_options]);
        hasher.update([row.max_upside_gain]);
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
    fn hwatu_dossier_passes_full_scenario_pack() {
        let dossier = HwatuBenchmarkDossier::build("hwatu-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));

        assert!(dossier.passing, "dossier should pass");
        assert_eq!(
            dossier.scenario_count,
            hwatu_scenario_pack().len(),
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
    fn hwatu_dossier_is_deterministic_across_runs() {
        let first = HwatuBenchmarkDossier::build("hwatu-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));
        let second = HwatuBenchmarkDossier::build("hwatu-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));

        assert_eq!(first.scenario_hash, second.scenario_hash);
        assert_eq!(first.recommendations, second.recommendations);
    }

    #[test]
    fn hwatu_dossier_recommendation_map_is_complete() {
        let dossier = HwatuBenchmarkDossier::build("hwatu-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));

        for scenario in hwatu_scenario_pack() {
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
                matches!(token.as_str(), "call-go" | "stop-round" | "koi-koi"),
                "scenario {} produced unexpected action token {token}",
                scenario.scenario_id
            );
        }
    }

    #[test]
    fn hwatu_dossier_call_go_scenarios_recommend_call_go() {
        let dossier = HwatuBenchmarkDossier::build("hwatu-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));

        // Every call_go-bucket scenario (bonus-heavy fresh with live
        // upside) should pick CallGo. A regression that flipped these
        // to StopRound or KoiKoi would be a real heuristic bug.
        for scenario_id in [
            "cg-bonus-heavy-fresh-upside",
            "cg-bright-bonus-press",
            "cg-mixed-yaku-bonus-fresh",
            "cg-bright-bonus-fresh-upside",
            "cg-bright-bonus-fresh-zero-locked",
            "cg-animal-yaku-bonus-fresh",
            "cg-bright-bonus-thick-upside",
            "cg-bonus-fresh-upside",
        ] {
            assert_eq!(
                dossier.recommendations.get(scenario_id).map(String::as_str),
                Some("call-go"),
                "{scenario_id} should recommend call-go"
            );
        }
    }

    #[test]
    fn hwatu_dossier_stop_round_scenarios_recommend_stop_round() {
        let dossier = HwatuBenchmarkDossier::build("hwatu-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));

        // Every stop_round-bucket scenario (banked cash-out with
        // locked points and no live upside) should pick StopRound.
        // A regression that flipped these to CallGo or KoiKoi would
        // be a real heuristic bug.
        for scenario_id in [
            "sr-cashout-bright-no-upside",
            "sr-bright-banked-pressure",
            "sr-yaku-banked-no-upside",
            "sr-mixed-yaku-banked-no-upside",
            "sr-bright-banked-no-upside",
            "sr-bright-yaku-pressure",
            "sr-stretched-bright-bank",
            "sr-bright-bank-lock",
        ] {
            assert_eq!(
                dossier.recommendations.get(scenario_id).map(String::as_str),
                Some("stop-round"),
                "{scenario_id} should recommend stop-round"
            );
        }
    }

    #[test]
    fn hwatu_dossier_scenario_count_is_22() {
        let dossier = HwatuBenchmarkDossier::build("hwatu-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));

        assert_eq!(dossier.scenario_count, 22);
        assert_eq!(dossier.recommendation_count, 22);
        assert!(dossier.passing);
    }
}

//! Stratego benchmark dossier surface.
//!
//! The dossier is the promotion artifact the `F-016` slice ships: it
//! pins the 22-scenario rule-aware scenario pack for Stratego, runs
//! the live portfolio engine against it, records every engine
//! recommendation, and SHA-256-pins the canonical scenario/answer
//! table so the promotion manifest harness can verify that the
//! evidence attached to a `tier: benchmarked` claim matches the live
//! engine output.
//!
//! Stratego is the F-016 portfolio-game-promotion slice (the
//! F-001/F-008/F-009/F-010/F-011/F-012/F-013/F-014/F-015 tenth slice:
//! Cribbage / Hearts / Gin Rummy / Spades / Bridge / Call Break /
//! Backgammon / Hanafuda Koi-Koi / Hwatu Go-Stop). F-016 opens the
//! `state-aware belief-scout heuristic` engine family in
//! `crate::engines::stratego::answer` — no other portfolio game
//! shares that engine surface, so F-016 is the first dossier slice
//! for the belief-scout family (the F-015 HwatuGoStop row is the
//! `hanafuda` family's second slice; F-014 HanafudaKoiKoi is its
//! first; F-013 Backgammon is the `backgammon` family's first and
//! only slice so far).
//!
//! The engine has three ranked arms (`scout` / `place_safe` /
//! `advance_piece`) keyed to open-scouting / bomb-defensive /
//! forced-combat state. The F-016 scenario pack splits 8/8/6 across
//! scout-dominant / advance-piece-dominant / place-safe-dominant so
//! a regression that flips the dominant arm on any scenario is loud
//! in the dossier's recommendation map.
//!
//! The dossier is intentionally `benchmarked`-tier, not
//! `promotable_local`: the policy bundle builder
//! (`genesis/plans/001-master-plan.md`) is designed for dedicated
//! games initially, and generalizing it to portfolio games is
//! follow-on work (the same scope boundary the F-001 Cribbage, F-008
//! Hearts, F-009 Gin Rummy, F-010 Spades, F-011 Bridge, F-012 Call
//! Break, F-013 Backgammon, F-014 HanafudaKoiKoi, and F-015
//! HwatuGoStop rows landed under).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::core::stratego::{StrategoScenario, stratego_scenario_pack};
use crate::engine::answer_typed_challenge;
use crate::game::ResearchGame;
use crate::protocol::{PortfolioAction, recommended_action};
use crate::state::{PortfolioChallenge, PortfolioChallengeSpot, StrategoChallenge};

const BENCHMARK_METHOD: &str = "stratego-rule-aware-scenario-pack-v1";
const BENCHMARK_METRIC: &str = "engine_recommendation_count";

/// Hash-pinned evidence that the Stratego `rule-aware` engine was
/// run against the canonical scenario pack and produced a
/// recommendation for every row.
///
/// The threshold is `0.0` because the rule-aware engine is
/// deterministic and the promotion gate is "every scenario produced
/// a recommendation", not a quality score. Higher-quality promotion
/// tiers (notably `promotable_local`) will swap this dossier for a
/// `CanonicalPolicyBundle` that wraps the same scenario table.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct StrategoBenchmarkDossier {
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
    scout_lanes: u8,
    miners_remaining: u8,
    bombs_suspected: u8,
    attack_targets: u8,
    hidden_targets: u8,
    attack_is_forced: bool,
    engine_tier: String,
    recommended_action: String,
}

impl StrategoBenchmarkDossier {
    /// Build a dossier by running the live `rule-aware` engine
    /// against the canonical Stratego scenario pack.
    pub fn build(benchmark_id: impl Into<String>) -> Result<Self, StrategoDossierError> {
        Self::build_with_scenarios(benchmark_id, stratego_scenario_pack())
    }

    /// Build a dossier from a custom scenario slice (used by tests
    /// and the negative-fixture harnesses).
    pub fn build_with_scenarios(
        benchmark_id: impl Into<String>,
        scenarios: &[StrategoScenario],
    ) -> Result<Self, StrategoDossierError> {
        let mut rows: Vec<DossierRow> = Vec::with_capacity(scenarios.len());
        let mut recommendations: BTreeMap<String, String> = BTreeMap::new();

        for scenario in scenarios {
            let challenge = PortfolioChallenge::Stratego(StrategoChallenge {
                spot: PortfolioChallengeSpot::scenario(
                    ResearchGame::Stratego,
                    scenario.scenario_id,
                    scenario.decision,
                ),
                scout_lanes: scenario.scout_lanes,
                miners_remaining: scenario.miners_remaining,
                bombs_suspected: scenario.bombs_suspected,
                attack_targets: scenario.attack_targets,
                hidden_targets: scenario.hidden_targets,
                attack_is_forced: scenario.attack_is_forced,
            });

            let (challenge_id, decision) = match &challenge {
                PortfolioChallenge::Stratego(state) => {
                    (state.spot.challenge_id.clone(), state.spot.decision.clone())
                }
                _ => unreachable!(
                    "dispatcher only routes Stratego challenges to the stratego engine"
                ),
            };

            let answer = answer_typed_challenge(&challenge, 0).map_err(|error| {
                StrategoDossierError::EngineFailed {
                    scenario_id: scenario.scenario_id.to_string(),
                    error: error.to_string(),
                }
            })?;

            let recommendation = recommended_action(&answer.response).ok_or_else(|| {
                StrategoDossierError::NoRecommendation {
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
                scout_lanes: scenario.scout_lanes,
                miners_remaining: scenario.miners_remaining,
                bombs_suspected: scenario.bombs_suspected,
                attack_targets: scenario.attack_targets,
                hidden_targets: scenario.hidden_targets,
                attack_is_forced: scenario.attack_is_forced,
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
            engine_family: "state-aware belief-scout heuristic".to_string(),
            engine_tier: "rule-aware".to_string(),
            rule_file: ResearchGame::Stratego.rule_file().to_string(),
            scenario_hash,
            recommendations,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StrategoDossierError {
    #[error("stratego engine failed for scenario {scenario_id}: {error}")]
    EngineFailed { scenario_id: String, error: String },
    #[error("stratego engine produced no recommendation for scenario {scenario_id}")]
    NoRecommendation { scenario_id: String },
}

fn action_token(action: PortfolioAction) -> &'static str {
    match action {
        PortfolioAction::Scout => "scout",
        PortfolioAction::PlaceSafe => "place-safe",
        PortfolioAction::AdvancePiece => "advance-piece",
        // Stratego engine only emits these three; anything else
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
        hasher.update([row.scout_lanes]);
        hasher.update([row.miners_remaining]);
        hasher.update([row.bombs_suspected]);
        hasher.update([row.attack_targets]);
        hasher.update([row.hidden_targets]);
        hasher.update([u8::from(row.attack_is_forced)]);
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
    fn stratego_dossier_passes_full_scenario_pack() {
        let dossier = StrategoBenchmarkDossier::build("stratego-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));

        assert!(dossier.passing, "dossier should pass");
        assert_eq!(
            dossier.scenario_count,
            stratego_scenario_pack().len(),
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
    fn stratego_dossier_is_deterministic_across_runs() {
        let first = StrategoBenchmarkDossier::build("stratego-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));
        let second = StrategoBenchmarkDossier::build("stratego-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));

        assert_eq!(first.scenario_hash, second.scenario_hash);
        assert_eq!(first.recommendations, second.recommendations);
    }

    #[test]
    fn stratego_dossier_recommendation_map_is_complete() {
        let dossier = StrategoBenchmarkDossier::build("stratego-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));

        for scenario in stratego_scenario_pack() {
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
                matches!(token.as_str(), "scout" | "place-safe" | "advance-piece"),
                "scenario {} produced unexpected action token {token}",
                scenario.scenario_id
            );
        }
    }

    #[test]
    fn stratego_dossier_scout_scenarios_recommend_scout() {
        let dossier = StrategoBenchmarkDossier::build("stratego-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));

        // Every scout-bucket scenario (open board with hidden targets
        // and live scout lanes) should pick Scout. A regression that
        // flipped these to PlaceSafe or AdvancePiece would be a real
        // heuristic bug.
        for scenario_id in [
            "scout-open-two-lanes-one-hidden",
            "scout-open-three-lanes-one-hidden",
            "scout-open-one-lane-two-hidden",
            "scout-open-two-lanes-two-hidden",
            "scout-light-bomb-pressure",
            "scout-deep-bomb-pressure",
            "scout-mixed-bomb-press",
            "scout-hidden-rich",
        ] {
            assert_eq!(
                dossier.recommendations.get(scenario_id).map(String::as_str),
                Some("scout"),
                "{scenario_id} should recommend scout"
            );
        }
    }

    #[test]
    fn stratego_dossier_advance_scenarios_recommend_advance_piece() {
        let dossier = StrategoBenchmarkDossier::build("stratego-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));

        // Every advance-piece-bucket scenario (forced combat with
        // live attack targets) should pick AdvancePiece. A
        // regression that flipped these to Scout or PlaceSafe would
        // be a real heuristic bug.
        for scenario_id in [
            "advance-forced-one-target",
            "advance-forced-two-targets",
            "advance-forced-three-targets",
            "advance-forced-one-target-bomb",
            "advance-forced-one-target-deep-bomb",
            "advance-forced-one-hidden",
            "advance-forced-two-hidden",
            "advance-forced-one-target-bomb-balanced",
        ] {
            assert_eq!(
                dossier.recommendations.get(scenario_id).map(String::as_str),
                Some("advance-piece"),
                "{scenario_id} should recommend advance-piece"
            );
        }
    }

    #[test]
    fn stratego_dossier_place_safe_scenarios_recommend_place_safe() {
        let dossier = StrategoBenchmarkDossier::build("stratego-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));

        // Every place-safe-bucket scenario (heavy bomb pressure +
        // no attack targets + no miners) should pick PlaceSafe. A
        // regression that flipped these to Scout or AdvancePiece
        // would be a real heuristic bug.
        for scenario_id in [
            "place-safe-heavy-bomb",
            "place-safe-no-miners",
            "place-safe-no-targets-heavy-bomb",
            "place-safe-no-targets-mid-bomb",
            "place-safe-no-miners-deep-bomb",
            "place-safe-no-miners-light-bomb",
        ] {
            assert_eq!(
                dossier.recommendations.get(scenario_id).map(String::as_str),
                Some("place-safe"),
                "{scenario_id} should recommend place-safe"
            );
        }
    }

    #[test]
    fn stratego_dossier_scenario_count_is_22() {
        let dossier = StrategoBenchmarkDossier::build("stratego-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));

        assert_eq!(dossier.scenario_count, 22);
        assert_eq!(dossier.recommendation_count, 22);
        assert!(dossier.passing);
    }
}

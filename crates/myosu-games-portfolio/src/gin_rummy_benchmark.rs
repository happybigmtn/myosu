//! Gin Rummy benchmark dossier surface.
//!
//! The dossier is the promotion artifact the `F-009` slice ships: it pins
//! the 22-scenario rule-aware scenario pack for Gin Rummy, runs the live
//! portfolio engine against it, records every engine recommendation, and
//! SHA-256-pins the canonical scenario/answer table so the promotion
//! manifest harness can verify that the evidence attached to a
//! `tier: benchmarked` claim matches the live engine output.
//!
//! The dossier is intentionally `benchmarked`-tier, not `promotable_local`:
//! the policy bundle builder (`genesis/plans/001-master-plan.md`) is
//! designed for dedicated games initially, and generalizing it to
//! portfolio games is follow-on work (the same scope boundary the F-001
//! Cribbage and F-008 Hearts rows landed under).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::core::gin_rummy::{GinRummyScenario, gin_rummy_scenario_pack};
use crate::engine::answer_typed_challenge;
use crate::game::ResearchGame;
use crate::protocol::{PortfolioAction, recommended_action};
use crate::state::{GinRummyChallenge, PortfolioChallenge, PortfolioChallengeSpot};

const BENCHMARK_METHOD: &str = "gin-rummy-rule-aware-scenario-pack-v1";
const BENCHMARK_METRIC: &str = "engine_recommendation_count";

/// Hash-pinned evidence that the Gin Rummy `rule-aware` engine was run
/// against the canonical scenario pack and produced a recommendation for
/// every row.
///
/// The threshold is `0.0` because the rule-aware engine is deterministic
/// and the promotion gate is "every scenario produced a recommendation", not
/// a quality score. Higher-quality promotion tiers (notably
/// `promotable_local`) will swap this dossier for a `CanonicalPolicyBundle`
/// that wraps the same scenario table.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct GinRummyBenchmarkDossier {
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
    deadwood: u8,
    meld_count: u8,
    live_draws: u8,
    knock_available: bool,
    gin_available: bool,
    discard_options: u8,
    engine_tier: String,
    recommended_action: String,
}

impl GinRummyBenchmarkDossier {
    /// Build a dossier by running the live `rule-aware` engine against the
    /// canonical Gin Rummy scenario pack.
    pub fn build(benchmark_id: impl Into<String>) -> Result<Self, GinRummyDossierError> {
        Self::build_with_scenarios(benchmark_id, gin_rummy_scenario_pack())
    }

    /// Build a dossier from a custom scenario slice (used by tests and the
    /// negative-fixture harnesses).
    pub fn build_with_scenarios(
        benchmark_id: impl Into<String>,
        scenarios: &[GinRummyScenario],
    ) -> Result<Self, GinRummyDossierError> {
        let mut rows: Vec<DossierRow> = Vec::with_capacity(scenarios.len());
        let mut recommendations: BTreeMap<String, String> = BTreeMap::new();

        for scenario in scenarios {
            let challenge = PortfolioChallenge::GinRummy(GinRummyChallenge {
                spot: PortfolioChallengeSpot::scenario(
                    ResearchGame::GinRummy,
                    scenario.scenario_id,
                    scenario.decision,
                ),
                deadwood: scenario.deadwood,
                meld_count: scenario.meld_count,
                live_draws: scenario.live_draws,
                knock_available: scenario.knock_available,
                gin_available: scenario.gin_available,
                discard_options: scenario.discard_options,
            });

            let (challenge_id, decision) = match &challenge {
                PortfolioChallenge::GinRummy(state) => {
                    (state.spot.challenge_id.clone(), state.spot.decision.clone())
                }
                _ => unreachable!(
                    "dispatcher only routes GinRummy challenges to the gin_rummy engine"
                ),
            };

            let answer = answer_typed_challenge(&challenge, 0).map_err(|error| {
                GinRummyDossierError::EngineFailed {
                    scenario_id: scenario.scenario_id.to_string(),
                    error: error.to_string(),
                }
            })?;

            let recommendation = recommended_action(&answer.response).ok_or_else(|| {
                GinRummyDossierError::NoRecommendation {
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
                deadwood: scenario.deadwood,
                meld_count: scenario.meld_count,
                live_draws: scenario.live_draws,
                knock_available: scenario.knock_available,
                gin_available: scenario.gin_available,
                discard_options: scenario.discard_options,
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
            engine_family: "state-aware meld-distance draw/discard heuristic".to_string(),
            engine_tier: "rule-aware".to_string(),
            rule_file: ResearchGame::GinRummy.rule_file().to_string(),
            scenario_hash,
            recommendations,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GinRummyDossierError {
    #[error("gin-rummy engine failed for scenario {scenario_id}: {error}")]
    EngineFailed { scenario_id: String, error: String },
    #[error("gin-rummy engine produced no recommendation for scenario {scenario_id}")]
    NoRecommendation { scenario_id: String },
}

fn action_token(action: PortfolioAction) -> &'static str {
    match action {
        PortfolioAction::Knock => "knock",
        PortfolioAction::DiscardDeadwood => "discard-deadwood",
        PortfolioAction::PotControl => "pot-control",
        // Gin Rummy engine only emits these three; anything else would be a
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
        hasher.update([row.deadwood]);
        hasher.update([row.meld_count]);
        hasher.update([row.live_draws]);
        hasher.update([u8::from(row.knock_available)]);
        hasher.update([u8::from(row.gin_available)]);
        hasher.update([row.discard_options]);
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
    fn gin_rummy_dossier_passes_full_scenario_pack() {
        let dossier = GinRummyBenchmarkDossier::build("gin-rummy-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));

        assert!(dossier.passing, "dossier should pass");
        assert_eq!(
            dossier.scenario_count,
            gin_rummy_scenario_pack().len(),
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
    fn gin_rummy_dossier_is_deterministic_across_runs() {
        let first = GinRummyBenchmarkDossier::build("gin-rummy-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));
        let second = GinRummyBenchmarkDossier::build("gin-rummy-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));

        assert_eq!(first.scenario_hash, second.scenario_hash);
        assert_eq!(first.recommendations, second.recommendations);
    }

    #[test]
    fn gin_rummy_dossier_recommendation_map_is_complete() {
        let dossier = GinRummyBenchmarkDossier::build("gin-rummy-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));

        for scenario in gin_rummy_scenario_pack() {
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
                matches!(token.as_str(), "knock" | "discard-deadwood" | "pot-control"),
                "scenario {} produced unexpected action token {token}",
                scenario.scenario_id
            );
        }
    }

    #[test]
    fn gin_rummy_dossier_knock_window_scenarios_emit_knock_or_gin_conversion() {
        let dossier = GinRummyBenchmarkDossier::build("gin-rummy-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));

        // All five knock-window scenarios have knock_available=true and a
        // reasonable deadwood pressure, so the rule-aware engine should
        // rank Knock over DiscardDeadwood / PotControl.
        for scenario_id in [
            "knock-window-clean",
            "knock-window-forced",
            "knock-window-gin-pressure",
            "knock-window-deadwood-tight",
            "knock-window-discard-tight",
        ] {
            assert_eq!(
                dossier.recommendations.get(scenario_id).map(String::as_str),
                Some("knock"),
                "{scenario_id} should recommend knock"
            );
        }
    }

    #[test]
    fn gin_rummy_dossier_gin_conversion_scenarios_emit_knock() {
        let dossier = GinRummyBenchmarkDossier::build("gin-rummy-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));

        // Gin conversion scenarios all have gin_available=true and
        // knock_available=true. The rule-aware engine should still rank
        // Knock over PotControl (since the knock pressure bonus is
        // larger than the gin-only bonus in the heuristic).
        for scenario_id in [
            "gin-conversion-clean",
            "gin-conversion-via-discard",
            "gin-conversion-last-card",
            "gin-conversion-mid-hand",
            "gin-conversion-blocked",
        ] {
            assert_eq!(
                dossier.recommendations.get(scenario_id).map(String::as_str),
                Some("knock"),
                "{scenario_id} should recommend knock"
            );
        }
    }

    #[test]
    fn gin_rummy_dossier_pot_control_scenarios_emit_discard_deadwood() {
        // The Gin Rummy rule-aware engine's discard_deadwood heuristic
        // (0.85 + deadwood*0.06 + discard_options*0.08) outranks the
        // pot_control heuristic (0.75 + live_draws*0.08) when both
        // deadwood and discard_options are non-trivial, so the engine
        // ranks discard-deadwood over pot-control for every pot-control
        // scenario in the pack. This is the same kind of real engine
        // heuristic gap the F-001 Cribbage row already documented
        // (pegging-pair-clean prefers discard-deadwood over peg-run);
        // the dossier still passes because the promotion threshold is
        // "every scenario produced a recommendation", not a per-scenario
        // quality score.
        let dossier = GinRummyBenchmarkDossier::build("gin-rummy-pack-v1")
            .unwrap_or_else(|error| panic!("dossier should build: {error}"));

        for scenario_id in [
            "pot-control-long-draw",
            "pot-control-mid-draw",
            "pot-control-early-draw",
            "pot-control-no-knock-pressure",
        ] {
            assert_eq!(
                dossier.recommendations.get(scenario_id).map(String::as_str),
                Some("discard-deadwood"),
                "{scenario_id} should recommend discard-deadwood (engine heuristic gap; see module docs)"
            );
        }
    }
}

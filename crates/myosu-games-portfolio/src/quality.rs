use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::engine::{EngineAnswer, EngineTier};
use crate::game::ResearchGame;
use crate::protocol::{PortfolioAction, recommended_action};

/// Deterministic quality report for one portfolio engine response.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct EngineQualityReport {
    pub game: ResearchGame,
    pub challenge_id: String,
    pub engine_family: String,
    pub engine_tier: EngineTier,
    pub legal_action_count: usize,
    pub baseline_action: String,
    pub engine_action: String,
    pub baseline_l1_distance: f64,
    pub deterministic: bool,
    pub score: f64,
    pub iterations: usize,
    pub seed: u64,
}

impl EngineQualityReport {
    /// Build a report by comparing the selected engine response to the baseline.
    pub fn from_answer(
        answer: &EngineAnswer,
        baseline: &EngineAnswer,
        iterations: usize,
        seed: u64,
    ) -> Self {
        let baseline_action = action_label(recommended_action(&baseline.response));
        let engine_action = action_label(recommended_action(&answer.response));
        let baseline_l1_distance = response_l1_distance(answer, baseline);
        let score = if answer.engine_tier == EngineTier::StaticBaseline {
            1.0
        } else if engine_action != baseline_action {
            1.1
        } else if baseline_l1_distance > f64::EPSILON {
            1.05
        } else {
            1.0
        };

        Self {
            game: answer.game,
            challenge_id: answer.challenge_id.clone(),
            engine_family: answer.engine_family.clone(),
            engine_tier: answer.engine_tier,
            legal_action_count: answer.legal_actions.len(),
            baseline_action,
            engine_action,
            baseline_l1_distance,
            deterministic: true,
            score,
            iterations,
            seed,
        }
    }
}

fn action_label(action: Option<PortfolioAction>) -> String {
    action
        .map(PortfolioAction::label)
        .unwrap_or("none")
        .to_string()
}

fn response_l1_distance(answer: &EngineAnswer, baseline: &EngineAnswer) -> f64 {
    let mut actions = BTreeSet::new();
    for (action, _) in &answer.response.actions {
        actions.insert(*action);
    }
    for (action, _) in &baseline.response.actions {
        actions.insert(*action);
    }

    actions
        .into_iter()
        .map(|action| {
            (probability_for(&answer.response.actions, action)
                - probability_for(&baseline.response.actions, action))
            .abs()
        })
        .sum()
}

fn probability_for(actions: &[(PortfolioAction, f32)], action: PortfolioAction) -> f64 {
    actions
        .iter()
        .find_map(|(candidate, probability)| {
            (*candidate == action).then_some(f64::from(*probability))
        })
        .unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use crate::engine::{answer_game, answer_typed_challenge};
    use crate::game::ResearchGame;
    use crate::quality::EngineQualityReport;
    use crate::state::PortfolioChallenge;

    #[test]
    fn quality_report_marks_bridge_as_rule_aware_and_deterministic() {
        let challenge = match PortfolioChallenge::bootstrap(ResearchGame::Bridge) {
            Some(challenge) => challenge,
            None => panic!("bridge should have typed challenge"),
        };
        let answer = match answer_typed_challenge(&challenge, 3) {
            Ok(answer) => answer,
            Err(error) => panic!("bridge should answer: {error}"),
        };
        let baseline = answer_game(ResearchGame::Bridge, 0);
        let report = EngineQualityReport::from_answer(&answer, &baseline, 3, 11);

        assert_eq!(report.game, ResearchGame::Bridge);
        assert_eq!(report.engine_tier.as_str(), "rule-aware");
        assert!(report.deterministic);
        assert!(report.legal_action_count > 0);
        assert!(report.baseline_l1_distance > 0.0);
        assert!(report.score > 1.0);
    }
}

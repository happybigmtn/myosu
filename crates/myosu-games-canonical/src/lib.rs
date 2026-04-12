//! Canonical registry and bootstrap adapters for the initial myosu portfolio.

mod games;
pub mod playtrace;
pub mod policy;

use myosu_games::{
    CanonicalActionSpec, CanonicalGameSpec, CanonicalStateSnapshot, CanonicalStrategyBinding,
    CanonicalTruthError, canonical_hash, validate_unique_action_ids,
};
use myosu_games_portfolio::{PortfolioAction, PortfolioSolver, answer_typed_challenge};
use serde_json::json;

pub use myosu_games_portfolio::ResearchGame;
pub use playtrace::{
    PlaytraceError, PlaytracePolicy, PlaytraceReport, PlaytraceRequest,
    canonical_ten_playtrace_requests, research_playtrace_requests, run_playtrace,
    validate_transition_trace,
};
pub use policy::{
    CanonicalPolicyBenchmarkSummary, CanonicalPolicyBundle, CanonicalPolicyDistributionEntry,
    CanonicalPolicyProvenance, CanonicalPolicySamplingProof, PolicyPromotionTier,
    SolverPromotionEntry, SolverPromotionLedger, SolverPromotionManifestRow, SolverPromotionRoute,
    code_reported_bundle_support, compute_bundle_hash, parse_solver_promotion_ledger,
    read_solver_promotion_ledger, sample_policy_action, solver_promotion_manifest_rows,
    verify_policy_bundle,
};

/// First migration batch for canonical game truth.
pub const CANONICAL_TEN: [ResearchGame; 10] = [
    ResearchGame::NlheSixMax,
    ResearchGame::HanafudaKoiKoi,
    ResearchGame::RiichiMahjong,
    ResearchGame::Bridge,
    ResearchGame::GinRummy,
    ResearchGame::Stratego,
    ResearchGame::OfcChinesePoker,
    ResearchGame::DouDiZhu,
    ResearchGame::Backgammon,
    ResearchGame::Cribbage,
];

/// Canonical specs for every game in the initial migration batch.
pub fn canonical_ten_specs() -> Vec<CanonicalGameSpec> {
    CANONICAL_TEN
        .into_iter()
        .filter_map(canonical_game_spec)
        .collect()
}

/// Canonical spec for a supported portfolio game.
pub fn canonical_game_spec(game: ResearchGame) -> Option<CanonicalGameSpec> {
    if !is_canonical_ten(game) {
        return None;
    }

    Some(CanonicalGameSpec {
        game_type: game.game_type(),
        slug: game.slug().to_string(),
        chain_id: game.chain_id().to_string(),
        ruleset_version: 1,
        display_name: game.display_name().to_string(),
        default_players: game.default_players(),
        rule_file: Some(game.rule_file().to_string()),
    })
}

/// Canonical action specs derived from the typed rule-aware portfolio engine.
pub fn canonical_action_specs(game: ResearchGame) -> Option<Vec<CanonicalActionSpec>> {
    if !is_canonical_ten(game) {
        return None;
    }

    let challenge = myosu_games_portfolio::PortfolioChallenge::bootstrap(game)?;
    let answer = answer_typed_challenge(&challenge, 0).ok()?;
    let family = canonical_family(game);
    let mut actions = Vec::with_capacity(answer.legal_actions.len());
    for action in answer.legal_actions {
        actions.push(action_spec(game, family, action));
    }

    validate_unique_action_ids(game.slug(), &actions).ok()?;

    Some(actions)
}

/// Representative canonical bootstrap snapshot for a supported game.
pub fn canonical_bootstrap_snapshot(
    game: ResearchGame,
) -> Result<CanonicalStateSnapshot, CanonicalTruthError> {
    ensure_canonical_ten(game)?;

    let challenge =
        myosu_games_portfolio::PortfolioChallenge::bootstrap(game).ok_or_else(|| {
            CanonicalTruthError::UnsupportedGame {
                game_id: game.slug().to_string(),
            }
        })?;
    let spot = challenge.spot();
    let legal_actions =
        canonical_action_specs(game).ok_or_else(|| CanonicalTruthError::UnsupportedGame {
            game_id: game.slug().to_string(),
        })?;
    let private_state_commitment = canonical_hash(&json!({
        "game": game.chain_id(),
        "trace_id": spot.challenge_id.as_str(),
        "commitment": "portfolio-bootstrap-hidden-state",
    }))?;

    Ok(CanonicalStateSnapshot {
        game_id: game.slug().to_string(),
        ruleset_version: 1,
        trace_id: spot.challenge_id.clone(),
        phase: "bootstrap".to_string(),
        actor: Some(0),
        public_state: json!({
            "chain_id": game.chain_id(),
            "decision": spot.decision.as_str(),
            "default_players": game.default_players(),
            "engine_family": spot.solver_family.as_str(),
            "rule_file": spot.rule_file.as_str(),
            "state_kind": "portfolio_strength_bootstrap",
        }),
        private_state_commitments: vec![private_state_commitment],
        legal_actions,
        terminal: false,
    })
}

/// Strategy binding for the representative typed strength query.
pub fn canonical_bootstrap_strategy_binding(
    game: ResearchGame,
) -> Result<CanonicalStrategyBinding, CanonicalTruthError> {
    ensure_canonical_ten(game)?;

    let solver = PortfolioSolver::new();
    let query = PortfolioSolver::strength_query(game).map_err(|_| {
        CanonicalTruthError::UnsupportedGame {
            game_id: game.slug().to_string(),
        }
    })?;
    let response = solver.answer_strength_checked(query.clone()).map_err(|_| {
        CanonicalTruthError::UnsupportedGame {
            game_id: game.slug().to_string(),
        }
    })?;
    let quality = solver.strength_quality(query.clone()).map_err(|_| {
        CanonicalTruthError::UnsupportedGame {
            game_id: game.slug().to_string(),
        }
    })?;

    Ok(CanonicalStrategyBinding {
        query_hash: canonical_hash(&query)?,
        response_hash: canonical_hash(&response)?,
        checkpoint_hash: None,
        engine_tier: quality.engine_tier.as_str().to_string(),
        engine_family: quality.engine_family,
        quality_summary: Some(format!(
            "challenge_id={};score={:.6}",
            quality.challenge_id, quality.score
        )),
    })
}

/// Whether a research game is in the first canonical migration batch.
pub fn is_canonical_ten(game: ResearchGame) -> bool {
    CANONICAL_TEN.contains(&game)
}

fn ensure_canonical_ten(game: ResearchGame) -> Result<(), CanonicalTruthError> {
    if is_canonical_ten(game) {
        Ok(())
    } else {
        Err(CanonicalTruthError::UnsupportedGame {
            game_id: game.slug().to_string(),
        })
    }
}

fn action_spec(
    game: ResearchGame,
    family: &'static str,
    action: PortfolioAction,
) -> CanonicalActionSpec {
    CanonicalActionSpec {
        game_id: game.slug().to_string(),
        action_id: format!("{}.{}.{}", game.slug(), family, action.label()),
        family: family.to_string(),
        display_label: action.label().to_string(),
        legal_phases: vec!["bootstrap".to_string()],
        params_schema: json!({"type": "object", "additionalProperties": false}),
    }
}

fn canonical_family(game: ResearchGame) -> &'static str {
    match game {
        ResearchGame::NlheSixMax => games::poker_like::FAMILY,
        ResearchGame::HanafudaKoiKoi => games::hanafuda::FAMILY,
        ResearchGame::RiichiMahjong => games::mahjong::FAMILY,
        ResearchGame::Bridge => games::trick_taking::FAMILY,
        ResearchGame::GinRummy => games::gin_rummy::FAMILY,
        ResearchGame::Stratego => games::stratego::FAMILY,
        ResearchGame::OfcChinesePoker => games::ofc::FAMILY,
        ResearchGame::DouDiZhu => games::shedding::FAMILY,
        ResearchGame::Backgammon => games::backgammon::FAMILY,
        ResearchGame::Cribbage => games::cribbage::FAMILY,
        _ => "unsupported",
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use myosu_games_portfolio::PortfolioSnapshot;

    use super::*;

    #[test]
    fn canonical_ten_has_expected_membership() {
        assert_eq!(CANONICAL_TEN.len(), 10);
        assert!(CANONICAL_TEN.contains(&ResearchGame::Bridge));
        assert!(CANONICAL_TEN.contains(&ResearchGame::NlheSixMax));
        assert!(!CANONICAL_TEN.contains(&ResearchGame::NlheHeadsUp));
        assert!(!CANONICAL_TEN.contains(&ResearchGame::LiarsDice));
    }

    #[test]
    fn canonical_specs_cover_all_ten_games() {
        let specs = canonical_ten_specs();
        let slugs: BTreeSet<_> = specs.iter().map(|spec| spec.slug.as_str()).collect();

        assert_eq!(specs.len(), 10);
        assert_eq!(slugs.len(), 10);
        assert!(slugs.contains("bridge"));
        assert!(slugs.contains("cribbage"));
    }

    #[test]
    fn every_canonical_game_builds_snapshot_and_binding() {
        for game in CANONICAL_TEN {
            let snapshot = match canonical_bootstrap_snapshot(game) {
                Ok(snapshot) => snapshot,
                Err(error) => panic!("{} should build snapshot: {error}", game.slug()),
            };
            let binding = match canonical_bootstrap_strategy_binding(game) {
                Ok(binding) => binding,
                Err(error) => panic!("{} should build binding: {error}", game.slug()),
            };

            assert_eq!(snapshot.game_id, game.slug());
            assert_eq!(snapshot.ruleset_version, 1);
            assert!(!snapshot.legal_actions.is_empty());
            assert!(!binding.query_hash.is_empty());
            assert!(!binding.response_hash.is_empty());
            assert_eq!(binding.engine_tier, "rule-aware");
        }
    }

    #[test]
    fn strategy_response_actions_map_to_one_canonical_action() {
        for game in CANONICAL_TEN {
            let query = match PortfolioSolver::strength_query(game) {
                Ok(query) => query,
                Err(error) => panic!("{} should build strength query: {error}", game.slug()),
            };
            let response = match PortfolioSolver::new().answer_strength_checked(query) {
                Ok(response) => response,
                Err(error) => panic!("{} should answer strength query: {error}", game.slug()),
            };
            let specs = match canonical_action_specs(game) {
                Some(specs) => specs,
                None => panic!("{} should build action specs", game.slug()),
            };

            for (action, _) in response.actions {
                let label = action.label();
                let matching = specs
                    .iter()
                    .filter(|spec| spec.display_label == label)
                    .count();

                assert_eq!(
                    matching,
                    1,
                    "{} action {label} should map once",
                    game.slug()
                );
            }
        }
    }

    #[test]
    fn renderer_completion_labels_map_to_canonical_actions() {
        for game in CANONICAL_TEN {
            let completions = PortfolioSnapshot::demo(game).legal_actions;
            let specs = match canonical_action_specs(game) {
                Some(specs) => specs,
                None => panic!("{} should build action specs", game.slug()),
            };

            for completion in completions {
                let matching = specs
                    .iter()
                    .filter(|spec| spec.display_label == completion)
                    .count();

                assert_eq!(
                    matching,
                    1,
                    "{} renderer completion {completion} should map once",
                    game.slug()
                );
            }
        }
    }

    #[test]
    fn snapshot_and_binding_hashes_are_stable_in_process() {
        let first_snapshot = match canonical_bootstrap_snapshot(ResearchGame::Bridge) {
            Ok(snapshot) => snapshot,
            Err(error) => panic!("bridge should build snapshot: {error}"),
        };
        let second_snapshot = match canonical_bootstrap_snapshot(ResearchGame::Bridge) {
            Ok(snapshot) => snapshot,
            Err(error) => panic!("bridge should build snapshot again: {error}"),
        };
        let first_binding = match canonical_bootstrap_strategy_binding(ResearchGame::Bridge) {
            Ok(binding) => binding,
            Err(error) => panic!("bridge should build binding: {error}"),
        };
        let second_binding = match canonical_bootstrap_strategy_binding(ResearchGame::Bridge) {
            Ok(binding) => binding,
            Err(error) => panic!("bridge should build binding again: {error}"),
        };

        assert_eq!(
            canonical_hash(&first_snapshot),
            canonical_hash(&second_snapshot)
        );
        assert_eq!(first_binding, second_binding);
    }
}

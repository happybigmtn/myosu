use myosu_games::{CanonicalActionSpec, CanonicalStateSnapshot};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::game::ResearchGame;

/// Canonical-facing state for a portfolio game core.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CoreGameState {
    pub game: ResearchGame,
    pub phase: String,
    pub actor: Option<u8>,
    pub public_state: Value,
    pub private_state_commitments: Vec<String>,
    pub legal_actions: Vec<CoreAction>,
    pub terminal: bool,
    pub payoff: Option<Vec<i64>>,
}

/// Canonical-facing action for a portfolio game core.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CoreAction {
    pub action_id: String,
    pub display_label: String,
    pub params: Value,
}

/// Result of applying one action to a core state.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CoreTransition {
    pub before: CoreGameState,
    pub action: CoreAction,
    pub after: CoreGameState,
}

/// Errors returned by core game dispatch and transition logic.
#[derive(Clone, Debug, Error, PartialEq)]
pub enum CoreGameError {
    #[error("game `{game}` does not have a core implementation yet")]
    UnsupportedGame { game: ResearchGame },
    #[error("unknown action `{action_id}` for game `{game}`")]
    UnknownAction {
        game: ResearchGame,
        action_id: String,
    },
    #[error("illegal action `{action_id}` for game `{game}`: {reason}")]
    IllegalAction {
        game: ResearchGame,
        action_id: String,
        reason: String,
    },
    #[error("invalid params for action `{action_id}`: {reason}")]
    InvalidParams { action_id: String, reason: String },
    #[error("payoff requested before game `{game}` reached terminal state")]
    NonTerminalPayoff { game: ResearchGame },
}

/// Build the first core state for a game.
pub fn bootstrap_state(game: ResearchGame) -> Result<CoreGameState, CoreGameError> {
    match game {
        ResearchGame::NlheHeadsUp => crate::core::poker_like::nlhe_heads_up_bootstrap_state(),
        ResearchGame::NlheSixMax => crate::core::poker_like::nlhe6max_bootstrap_state(),
        ResearchGame::Plo => crate::core::poker_like::plo_bootstrap_state(),
        ResearchGame::NlheTournament => crate::core::poker_like::nlhe_tournament_bootstrap_state(),
        ResearchGame::ShortDeck => crate::core::poker_like::short_deck_bootstrap_state(),
        ResearchGame::TeenPatti => crate::core::poker_like::teen_patti_bootstrap_state(),
        ResearchGame::HanafudaKoiKoi => crate::core::hanafuda::hanafuda_bootstrap_state(),
        ResearchGame::HwatuGoStop => crate::core::hanafuda::hwatu_bootstrap_state(),
        ResearchGame::RiichiMahjong => crate::core::mahjong::mahjong_bootstrap_state(),
        ResearchGame::Bridge => crate::core::trick_taking::bridge_bootstrap_state(),
        ResearchGame::Spades => crate::core::trick_taking::spades_bootstrap_state(),
        ResearchGame::CallBreak => crate::core::trick_taking::call_break_bootstrap_state(),
        ResearchGame::Hearts => crate::core::trick_taking::hearts_bootstrap_state(),
        ResearchGame::Cribbage => crate::core::cribbage::cribbage_bootstrap_state(),
        ResearchGame::Backgammon => crate::core::backgammon::backgammon_bootstrap_state(),
        ResearchGame::GinRummy => crate::core::gin_rummy::gin_rummy_bootstrap_state(),
        ResearchGame::OfcChinesePoker => crate::core::ofc::ofc_bootstrap_state(),
        ResearchGame::DouDiZhu => crate::core::shedding::dou_di_zhu_bootstrap_state(),
        ResearchGame::PusoyDos => crate::core::shedding::pusoy_dos_bootstrap_state(),
        ResearchGame::TienLen => crate::core::shedding::tien_len_bootstrap_state(),
        ResearchGame::Stratego => crate::core::stratego::stratego_bootstrap_state(),
        ResearchGame::LiarsDice => crate::core::liars_dice::liars_dice_bootstrap_state(),
    }
}

/// Borrow the legal actions already attached to a state.
pub fn legal_actions(state: &CoreGameState) -> &[CoreAction] {
    &state.legal_actions
}

/// Apply one legal action to a state.
pub fn apply_action(
    state: &CoreGameState,
    action_id: &str,
    params: Value,
) -> Result<CoreTransition, CoreGameError> {
    match state.game {
        ResearchGame::NlheHeadsUp => {
            crate::core::poker_like::apply_nlhe_heads_up_action(state, action_id, params)
        }
        ResearchGame::NlheSixMax => {
            crate::core::poker_like::apply_nlhe6max_action(state, action_id, params)
        }
        ResearchGame::Plo => crate::core::poker_like::apply_plo_action(state, action_id, params),
        ResearchGame::NlheTournament => {
            crate::core::poker_like::apply_nlhe_tournament_action(state, action_id, params)
        }
        ResearchGame::ShortDeck => {
            crate::core::poker_like::apply_short_deck_action(state, action_id, params)
        }
        ResearchGame::TeenPatti => {
            crate::core::poker_like::apply_teen_patti_action(state, action_id, params)
        }
        ResearchGame::HanafudaKoiKoi => {
            crate::core::hanafuda::apply_hanafuda_action(state, action_id, params)
        }
        ResearchGame::HwatuGoStop => {
            crate::core::hanafuda::apply_hwatu_action(state, action_id, params)
        }
        ResearchGame::RiichiMahjong => {
            crate::core::mahjong::apply_mahjong_action(state, action_id, params)
        }
        ResearchGame::Bridge => {
            crate::core::trick_taking::apply_bridge_action(state, action_id, params)
        }
        ResearchGame::Spades => {
            crate::core::trick_taking::apply_spades_action(state, action_id, params)
        }
        ResearchGame::CallBreak => {
            crate::core::trick_taking::apply_call_break_action(state, action_id, params)
        }
        ResearchGame::Hearts => {
            crate::core::trick_taking::apply_hearts_action(state, action_id, params)
        }
        ResearchGame::Cribbage => {
            crate::core::cribbage::apply_cribbage_action(state, action_id, params)
        }
        ResearchGame::Backgammon => {
            crate::core::backgammon::apply_backgammon_action(state, action_id, params)
        }
        ResearchGame::GinRummy => {
            crate::core::gin_rummy::apply_gin_rummy_action(state, action_id, params)
        }
        ResearchGame::OfcChinesePoker => {
            crate::core::ofc::apply_ofc_action(state, action_id, params)
        }
        ResearchGame::DouDiZhu => {
            crate::core::shedding::apply_dou_di_zhu_action(state, action_id, params)
        }
        ResearchGame::PusoyDos => {
            crate::core::shedding::apply_pusoy_dos_action(state, action_id, params)
        }
        ResearchGame::TienLen => {
            crate::core::shedding::apply_tien_len_action(state, action_id, params)
        }
        ResearchGame::Stratego => {
            crate::core::stratego::apply_stratego_action(state, action_id, params)
        }
        ResearchGame::LiarsDice => {
            crate::core::liars_dice::apply_liars_dice_action(state, action_id, params)
        }
    }
}

impl From<CoreGameState> for CanonicalStateSnapshot {
    fn from(state: CoreGameState) -> Self {
        let actor = state.actor;
        let trace_id = format!(
            "{}:{}:{}",
            state.game.chain_id(),
            state.phase,
            actor
                .map(|actor| format!("actor-{actor}"))
                .unwrap_or_else(|| "actor-none".to_string())
        );

        Self {
            game_id: state.game.slug().to_string(),
            ruleset_version: 1,
            trace_id,
            phase: state.phase,
            actor,
            public_state: state.public_state,
            private_state_commitments: state.private_state_commitments,
            legal_actions: state
                .legal_actions
                .into_iter()
                .map(CanonicalActionSpec::from)
                .collect(),
            terminal: state.terminal,
        }
    }
}

impl From<CoreAction> for CanonicalActionSpec {
    fn from(action: CoreAction) -> Self {
        let game_id = action
            .action_id
            .split('.')
            .next()
            .filter(|prefix| !prefix.is_empty())
            .unwrap_or("unknown")
            .to_string();

        Self {
            game_id,
            action_id: action.action_id,
            family: "core".to_string(),
            display_label: action.display_label,
            legal_phases: Vec::new(),
            params_schema: action.params,
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::game::ALL_RESEARCH_GAMES;

    #[test]
    fn bootstrap_state_supports_research_games() {
        let spades = bootstrap_state(ResearchGame::Spades)
            .unwrap_or_else(|error| panic!("spades should have core support: {error}"));

        assert_eq!(spades.game, ResearchGame::Spades);
        assert!(!spades.legal_actions.is_empty());
    }

    #[test]
    fn legal_action_accessor_returns_attached_actions() {
        let state = fake_state(ResearchGame::Bridge);
        let Some(action) = legal_actions(&state).first() else {
            panic!("fake state should have one legal action");
        };

        assert_eq!(legal_actions(&state).len(), 1);
        assert_eq!(action.action_id, "bridge.play.follow-suit");
    }

    #[test]
    fn apply_action_rejects_unknown_action_before_dispatch() {
        let state = bootstrap_state(ResearchGame::Spades)
            .unwrap_or_else(|error| panic!("spades bootstrap should succeed: {error}"));

        assert_eq!(
            apply_action(&state, "spades.play.bad", json!({})),
            Err(CoreGameError::UnknownAction {
                game: ResearchGame::Spades,
                action_id: "spades.play.bad".to_string(),
            })
        );
    }

    #[test]
    fn apply_action_dispatches_to_live_core_implementation() {
        let state = bootstrap_state(ResearchGame::Spades)
            .unwrap_or_else(|error| panic!("spades bootstrap should succeed: {error}"));
        let action_id = state
            .legal_actions
            .first()
            .map(|action| action.action_id.clone())
            .unwrap_or_else(|| panic!("spades bootstrap should expose a legal action"));
        let transition = apply_action(&state, &action_id, json!({}))
            .unwrap_or_else(|error| panic!("spades action should apply: {error}"));

        assert_eq!(transition.before.game, ResearchGame::Spades);
        assert_eq!(transition.action.action_id, action_id);
    }

    #[test]
    fn core_state_converts_to_canonical_snapshot() {
        let snapshot = CanonicalStateSnapshot::from(fake_state(ResearchGame::Bridge));

        assert_eq!(snapshot.game_id, "bridge");
        assert_eq!(snapshot.ruleset_version, 1);
        assert_eq!(snapshot.phase, "play");
        assert_eq!(snapshot.actor, Some(0));
        assert_eq!(snapshot.legal_actions.len(), 1);
        let Some(action) = snapshot.legal_actions.first() else {
            panic!("snapshot should have one legal action");
        };
        assert_eq!(action.action_id, "bridge.play.follow-suit");
    }

    #[test]
    fn core_action_converts_to_canonical_action_spec() {
        let spec = CanonicalActionSpec::from(fake_action("bridge"));

        assert_eq!(spec.game_id, "bridge");
        assert_eq!(spec.action_id, "bridge.play.follow-suit");
        assert_eq!(spec.display_label, "follow-suit");
        assert_eq!(spec.params_schema, json!({"type": "object"}));
    }

    #[test]
    fn core_types_serialize() {
        let state = fake_state(ResearchGame::Bridge);
        let serialized = match serde_json::to_string(&state) {
            Ok(serialized) => serialized,
            Err(error) => panic!("core state should serialize: {error}"),
        };

        assert!(serialized.contains("bridge"));
        assert!(serialized.contains("follow-suit"));
    }

    #[test]
    fn all_research_bootstrap_states_convert_to_canonical_snapshots() {
        for game in ALL_RESEARCH_GAMES {
            let state = bootstrap_state(game).unwrap_or_else(|error| {
                panic!(
                    "{} should have a core bootstrap state: {error}",
                    game.slug()
                )
            });
            assert!(
                !state.legal_actions.is_empty(),
                "{} should expose legal core actions",
                game.slug()
            );
            let snapshot = CanonicalStateSnapshot::from(state);

            assert_eq!(snapshot.game_id, game.slug());
            assert_eq!(snapshot.ruleset_version, 1);
            assert!(!snapshot.legal_actions.is_empty());
        }
    }

    fn fake_state(game: ResearchGame) -> CoreGameState {
        let prefix = game.slug();
        CoreGameState {
            game,
            phase: "play".to_string(),
            actor: Some(0),
            public_state: json!({"led_suit": "spades"}),
            private_state_commitments: vec!["hidden-hand-hash".to_string()],
            legal_actions: vec![fake_action(prefix)],
            terminal: false,
            payoff: None,
        }
    }

    fn fake_action(prefix: &str) -> CoreAction {
        CoreAction {
            action_id: format!("{prefix}.play.follow-suit"),
            display_label: "follow-suit".to_string(),
            params: json!({"type": "object"}),
        }
    }
}

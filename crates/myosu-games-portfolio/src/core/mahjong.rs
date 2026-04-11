use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};

use crate::core::model::{CoreAction, CoreGameError, CoreGameState, CoreTransition};
use crate::game::ResearchGame;

const MAHJONG_DISCARD_PREFIX: &str = "riichi-mahjong.discard.";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
struct MahjongPublicState {
    hand: Vec<String>,
    visible_discards: Vec<String>,
    dora_indicator: String,
    danger_discards: Vec<String>,
    seat_wind: String,
    round_wind: String,
    shanten: u8,
    riichi_declared: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MahjongFeatureView {
    pub shanten: u8,
    pub safe_discards: u8,
    pub discard_options: u8,
    pub ukeire: u8,
    pub pair_count: u8,
    pub run_bases: u8,
    pub dora_count: u8,
    pub yakuhai_pairs: u8,
    pub push_pressure: u8,
    pub riichi_threats: u8,
    pub riichi_available: bool,
}

struct MahjongPushInputs {
    shanten: u8,
    ukeire: u8,
    run_bases: u8,
    pair_count: u8,
    dora_count: u8,
    yakuhai_pairs: u8,
    riichi_threats: u8,
    safe_discards: u8,
    discard_options: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum MahjongAction {
    Discard(String),
    DeclareRiichi,
}

pub fn mahjong_bootstrap_state() -> Result<CoreGameState, CoreGameError> {
    let public = MahjongPublicState {
        hand: vec![
            "1m".to_string(),
            "2m".to_string(),
            "3m".to_string(),
            "4m".to_string(),
            "5m".to_string(),
            "6m".to_string(),
            "7m".to_string(),
            "2p".to_string(),
            "3p".to_string(),
            "4p".to_string(),
            "5s".to_string(),
            "5s".to_string(),
            "east".to_string(),
            "east".to_string(),
        ],
        visible_discards: vec!["9m".to_string(), "north".to_string()],
        dora_indicator: "4s".to_string(),
        danger_discards: vec!["east".to_string()],
        seat_wind: "east".to_string(),
        round_wind: "east".to_string(),
        shanten: 0,
        riichi_declared: false,
    };

    state_from_public(public, Some(0))
}

pub fn apply_mahjong_action(
    state: &CoreGameState,
    action_id: &str,
    _params: serde_json::Value,
) -> Result<CoreTransition, CoreGameError> {
    let before_public: MahjongPublicState = serde_json::from_value(state.public_state.clone())
        .map_err(|source| CoreGameError::InvalidParams {
            action_id: action_id.to_string(),
            reason: source.to_string(),
        })?;
    let action = parse_mahjong_action(action_id)?;
    validate_mahjong_action(&before_public, &action, action_id)?;
    if !state
        .legal_actions
        .iter()
        .any(|candidate| candidate.action_id == action_id)
    {
        return Err(illegal_mahjong_action(
            action_id,
            "action is not legal in this discard decision",
        ));
    }

    let mut after_public = before_public.clone();
    match action.clone() {
        MahjongAction::Discard(tile) => {
            remove_first_tile(&mut after_public.hand, &tile);
            after_public.visible_discards.push(tile);
        }
        MahjongAction::DeclareRiichi => {
            after_public.riichi_declared = true;
        }
    }
    let after = state_from_public(after_public, state.actor)?;
    let action = core_action_for_mahjong(action);

    Ok(CoreTransition {
        before: state.clone(),
        action,
        after,
    })
}

fn state_from_public(
    public: MahjongPublicState,
    actor: Option<u8>,
) -> Result<CoreGameState, CoreGameError> {
    let legal_actions = legal_mahjong_actions(&public)
        .into_iter()
        .map(core_action_for_mahjong)
        .collect();
    let public_state =
        serde_json::to_value(public).map_err(|source| CoreGameError::InvalidParams {
            action_id: "riichi-mahjong.bootstrap".to_string(),
            reason: source.to_string(),
        })?;

    Ok(CoreGameState {
        game: ResearchGame::RiichiMahjong,
        phase: "discard".to_string(),
        actor,
        public_state,
        private_state_commitments: vec!["riichi.wall-and-opponent-hands.bootstrap-v1".to_string()],
        legal_actions,
        terminal: false,
        payoff: None,
    })
}

pub(crate) fn feature_view(state: &CoreGameState) -> Result<MahjongFeatureView, CoreGameError> {
    let public: MahjongPublicState =
        serde_json::from_value(state.public_state.clone()).map_err(|source| {
            CoreGameError::InvalidParams {
                action_id: format!("{}.feature-view", state.game.slug()),
                reason: source.to_string(),
            }
        })?;
    let legal_actions = legal_mahjong_actions(&public);
    let discard_actions = legal_actions
        .iter()
        .filter_map(|action| match action {
            MahjongAction::Discard(tile) => Some(tile),
            MahjongAction::DeclareRiichi => None,
        })
        .collect::<Vec<_>>();
    let tile_counts = tile_counts(&public.hand);
    let safe_discards = usize_to_u8(
        discard_actions
            .iter()
            .filter(|tile| !public.danger_discards.iter().any(|danger| danger == **tile))
            .count(),
    );
    let discard_options = usize_to_u8(discard_actions.len());
    let ukeire = usize_to_u8(
        public
            .hand
            .iter()
            .cloned()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            .saturating_add(usize::from(public.shanten == 0))
            .min(13),
    );
    let pair_count = usize_to_u8(tile_counts.values().filter(|count| **count >= 2).count());
    let run_bases = count_run_bases(&public.hand);
    let dora_count = usize_to_u8(
        public
            .hand
            .iter()
            .filter(|tile| is_dora_tile(tile, &public.dora_indicator))
            .count(),
    );
    let yakuhai_pairs = count_yakuhai_pairs(&tile_counts, &public.seat_wind, &public.round_wind);
    let riichi_threats = usize_to_u8(
        usize::from(!public.danger_discards.is_empty())
            .saturating_add(public.visible_discards.len() / 6)
            .min(5),
    );

    Ok(MahjongFeatureView {
        shanten: public.shanten,
        safe_discards,
        discard_options,
        ukeire,
        pair_count,
        run_bases,
        dora_count,
        yakuhai_pairs,
        push_pressure: push_pressure(MahjongPushInputs {
            shanten: public.shanten,
            ukeire,
            run_bases,
            pair_count,
            dora_count,
            yakuhai_pairs,
            riichi_threats,
            safe_discards,
            discard_options,
        }),
        riichi_threats,
        riichi_available: legal_actions
            .iter()
            .any(|action| matches!(action, MahjongAction::DeclareRiichi)),
    })
}

fn legal_mahjong_actions(public: &MahjongPublicState) -> Vec<MahjongAction> {
    let mut unique_tiles = public.hand.clone();
    unique_tiles.sort();
    unique_tiles.dedup();
    let mut actions = unique_tiles
        .into_iter()
        .map(MahjongAction::Discard)
        .collect::<Vec<_>>();
    if public.shanten == 0 && !public.riichi_declared {
        actions.push(MahjongAction::DeclareRiichi);
    }

    actions
}

fn validate_mahjong_action(
    public: &MahjongPublicState,
    action: &MahjongAction,
    action_id: &str,
) -> Result<(), CoreGameError> {
    match action {
        MahjongAction::Discard(tile) => {
            if !public.hand.contains(tile) {
                return Err(illegal_mahjong_action(action_id, "tile is not in hand"));
            }
        }
        MahjongAction::DeclareRiichi => {
            if public.shanten != 0 || public.riichi_declared {
                return Err(illegal_mahjong_action(
                    action_id,
                    "riichi requires tenpai and no prior riichi declaration",
                ));
            }
        }
    }

    Ok(())
}

fn core_action_for_mahjong(action: MahjongAction) -> CoreAction {
    match action {
        MahjongAction::Discard(tile) => CoreAction {
            action_id: format!("{MAHJONG_DISCARD_PREFIX}{tile}"),
            display_label: format!("discard-{tile}"),
            params: json!({"tile": tile}),
        },
        MahjongAction::DeclareRiichi => CoreAction {
            action_id: "riichi-mahjong.declare-riichi".to_string(),
            display_label: "declare-riichi".to_string(),
            params: json!({}),
        },
    }
}

fn parse_mahjong_action(action_id: &str) -> Result<MahjongAction, CoreGameError> {
    if action_id == "riichi-mahjong.declare-riichi" {
        return Ok(MahjongAction::DeclareRiichi);
    }
    let Some(tile) = action_id.strip_prefix(MAHJONG_DISCARD_PREFIX) else {
        return Err(CoreGameError::UnknownAction {
            game: ResearchGame::RiichiMahjong,
            action_id: action_id.to_string(),
        });
    };

    Ok(MahjongAction::Discard(tile.to_string()))
}

fn remove_first_tile(hand: &mut Vec<String>, tile: &str) {
    let mut removed = false;
    hand.retain(|candidate| {
        if candidate == tile && !removed {
            removed = true;
            false
        } else {
            true
        }
    });
}

fn tile_counts(hand: &[String]) -> BTreeMap<&str, usize> {
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for tile in hand {
        if let Some(count) = counts.get_mut(tile.as_str()) {
            *count = count.saturating_add(1);
        } else {
            counts.insert(tile.as_str(), 1);
        }
    }
    counts
}

fn count_yakuhai_pairs(
    tile_counts: &BTreeMap<&str, usize>,
    seat_wind: &str,
    round_wind: &str,
) -> u8 {
    let mut yakuhai_tiles = BTreeSet::new();
    yakuhai_tiles.insert(seat_wind);
    yakuhai_tiles.insert(round_wind);
    for dragon in ["white", "green", "red"] {
        yakuhai_tiles.insert(dragon);
    }

    usize_to_u8(
        yakuhai_tiles
            .into_iter()
            .filter(|tile| tile_counts.get(tile).copied().unwrap_or(0) >= 2)
            .count(),
    )
}

fn count_run_bases(hand: &[String]) -> u8 {
    let suits = ['m', 'p', 's'];
    let sequences = [
        (1_u8, 2_u8, 3_u8),
        (2_u8, 3_u8, 4_u8),
        (3_u8, 4_u8, 5_u8),
        (4_u8, 5_u8, 6_u8),
        (5_u8, 6_u8, 7_u8),
        (6_u8, 7_u8, 8_u8),
        (7_u8, 8_u8, 9_u8),
    ];
    let mut suited_tiles = BTreeSet::new();
    for tile in hand {
        if let Some((rank, tile_suit)) = parse_suited_tile(tile) {
            suited_tiles.insert((tile_suit, rank));
        }
    }
    let mut total = 0usize;
    for suit in suits {
        total = total.saturating_add(
            sequences
                .iter()
                .filter(|(first, second, third)| {
                    suited_tiles.contains(&(suit, *first))
                        && suited_tiles.contains(&(suit, *second))
                        && suited_tiles.contains(&(suit, *third))
                })
                .count(),
        );
    }

    usize_to_u8(total)
}

fn parse_suited_tile(tile: &str) -> Option<(u8, char)> {
    let mut chars = tile.chars();
    let rank = chars.next()?.to_digit(10)?;
    let suit = chars.next()?;
    if chars.next().is_some() {
        return None;
    }
    if !matches!(suit, 'm' | 'p' | 's') {
        return None;
    }

    let rank = u8::try_from(rank).ok()?;
    if !(1..=9).contains(&rank) {
        return None;
    }

    Some((rank, suit))
}

fn is_dora_tile(tile: &str, indicator: &str) -> bool {
    match (parse_suited_tile(tile), parse_suited_tile(indicator)) {
        (Some((tile_rank, tile_suit)), Some((indicator_rank, indicator_suit))) => {
            tile_suit == indicator_suit && tile_rank == next_suited_rank(indicator_rank)
        }
        _ => next_honor_tile(indicator).is_some_and(|dora| tile == dora),
    }
}

fn next_suited_rank(rank: u8) -> u8 {
    if rank == 9 { 1 } else { rank.saturating_add(1) }
}

fn next_honor_tile(indicator: &str) -> Option<&'static str> {
    match indicator {
        "east" => Some("south"),
        "south" => Some("west"),
        "west" => Some("north"),
        "north" => Some("east"),
        "white" => Some("green"),
        "green" => Some("red"),
        "red" => Some("white"),
        _ => None,
    }
}

fn push_pressure(inputs: MahjongPushInputs) -> u8 {
    let MahjongPushInputs {
        shanten,
        ukeire,
        run_bases,
        pair_count,
        dora_count,
        yakuhai_pairs,
        riichi_threats,
        safe_discards,
        discard_options,
    } = inputs;
    let mut pressure = 0u8;
    if shanten == 0 {
        pressure = pressure.saturating_add(2);
    } else if shanten == 1 {
        pressure = pressure.saturating_add(1);
    }
    if ukeire >= 10 {
        pressure = pressure.saturating_add(2);
    } else if ukeire >= 6 {
        pressure = pressure.saturating_add(1);
    }
    if run_bases >= 4 {
        pressure = pressure.saturating_add(1);
    }
    if dora_count >= 2 {
        pressure = pressure.saturating_add(1);
    }
    if yakuhai_pairs > 0 {
        pressure = pressure.saturating_add(1);
    }
    if pair_count >= 4 {
        pressure = pressure.saturating_sub(1);
    }
    pressure = pressure.saturating_sub(riichi_threats.min(2));
    if discard_options > 0 && safe_discards.saturating_mul(2) < discard_options {
        pressure = pressure.saturating_sub(1);
    }
    pressure.min(5)
}

fn illegal_mahjong_action(action_id: &str, reason: &str) -> CoreGameError {
    CoreGameError::IllegalAction {
        game: ResearchGame::RiichiMahjong,
        action_id: action_id.to_string(),
        reason: reason.to_string(),
    }
}

fn usize_to_u8(value: usize) -> u8 {
    u8::try_from(value).unwrap_or(u8::MAX)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::core::model::{apply_action, bootstrap_state};

    #[test]
    fn mahjong_bootstrap_state_has_legal_actions() {
        let state = mahjong_state();

        assert_eq!(state.game, ResearchGame::RiichiMahjong);
        assert!(
            state
                .legal_actions
                .iter()
                .any(|action| action.action_id == "riichi-mahjong.discard.1m")
        );
    }

    #[test]
    fn mahjong_discard_from_hand_only() {
        let state = mahjong_state();

        assert!(matches!(
            apply_action(&state, "riichi-mahjong.discard.white", json!({})),
            Err(CoreGameError::IllegalAction { reason, .. }) if reason.contains("not in hand")
        ));
    }

    #[test]
    fn mahjong_riichi_gated_on_tenpai() {
        let state = mahjong_state();

        assert!(
            state
                .legal_actions
                .iter()
                .any(|action| action.action_id == "riichi-mahjong.declare-riichi")
        );
        assert!(matches!(
            apply_action(
                &mahjong_state_with_shanten(1),
                "riichi-mahjong.declare-riichi",
                json!({})
            ),
            Err(CoreGameError::IllegalAction { reason, .. }) if reason.contains("tenpai")
        ));
    }

    #[test]
    fn mahjong_transition_is_deterministic() {
        let state = mahjong_state();
        let first = apply_action(&state, "riichi-mahjong.discard.1m", json!({}));
        let second = apply_action(&state, "riichi-mahjong.discard.1m", json!({}));

        assert_eq!(first, second);
    }

    #[test]
    fn mahjong_feature_view_tracks_safety_and_ukeire() {
        let state = mahjong_state();
        let view = feature_view(&state)
            .unwrap_or_else(|error| panic!("mahjong feature view should decode: {error}"));

        assert_eq!(view.shanten, 0);
        assert_eq!(view.safe_discards, 11);
        assert_eq!(view.discard_options, 12);
        assert_eq!(view.ukeire, 13);
        assert_eq!(view.pair_count, 2);
        assert_eq!(view.run_bases, 6);
        assert_eq!(view.dora_count, 2);
        assert_eq!(view.yakuhai_pairs, 1);
        assert_eq!(view.push_pressure, 5);
        assert_eq!(view.riichi_threats, 1);
        assert!(view.riichi_available);
    }

    fn mahjong_state() -> CoreGameState {
        match bootstrap_state(ResearchGame::RiichiMahjong) {
            Ok(state) => state,
            Err(error) => panic!("mahjong bootstrap should succeed: {error}"),
        }
    }

    fn mahjong_state_with_shanten(shanten: u8) -> CoreGameState {
        let mut public: MahjongPublicState =
            match serde_json::from_value(mahjong_state().public_state) {
                Ok(public) => public,
                Err(error) => panic!("mahjong public state should decode: {error}"),
            };
        public.shanten = shanten;

        match state_from_public(public, Some(0)) {
            Ok(state) => state,
            Err(error) => panic!("mahjong state should rebuild: {error}"),
        }
    }
}

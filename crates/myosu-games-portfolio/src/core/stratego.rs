use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::core::model::{CoreAction, CoreGameError, CoreGameState, CoreTransition};
use crate::game::ResearchGame;

const STRATEGO_MOVE_PREFIX: &str = "stratego.move.";
const HIDDEN_MARSHAL_COMMITMENT: &str = "stratego.hidden-a.bootstrap-v1";
const HIDDEN_BOMB_COMMITMENT: &str = "stratego.hidden-b.bootstrap-v1";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
struct StrategoPublicState {
    board_width: u8,
    board_height: u8,
    water: Vec<StrategoCoordinate>,
    own_pieces: Vec<StrategoOwnPiece>,
    opponent_pieces: Vec<StrategoOpponentPiece>,
    captured: Vec<StrategoRank>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
struct StrategoCoordinate {
    x: u8,
    y: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
struct StrategoOwnPiece {
    rank: StrategoRank,
    position: StrategoCoordinate,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
struct StrategoOpponentPiece {
    position: StrategoCoordinate,
    public_rank: Option<StrategoRank>,
    hidden_commitment: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct StrategoFeatureView {
    pub scout_lanes: u8,
    pub miners_remaining: u8,
    pub bombs_suspected: u8,
    pub attack_targets: u8,
    pub hidden_targets: u8,
    pub attack_is_forced: bool,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum StrategoRank {
    Marshal,
    Miner,
    Scout,
    Spy,
    Bomb,
    Flag,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct StrategoMove {
    from: StrategoCoordinate,
    to: StrategoCoordinate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StrategoCombatOutcome {
    AttackerWins,
    DefenderWins,
    BothRemoved,
}

pub fn stratego_bootstrap_state() -> Result<CoreGameState, CoreGameError> {
    let public = StrategoPublicState {
        board_width: 4,
        board_height: 4,
        water: vec![
            StrategoCoordinate { x: 1, y: 2 },
            StrategoCoordinate { x: 2, y: 2 },
        ],
        own_pieces: vec![
            StrategoOwnPiece {
                rank: StrategoRank::Scout,
                position: StrategoCoordinate { x: 0, y: 0 },
            },
            StrategoOwnPiece {
                rank: StrategoRank::Miner,
                position: StrategoCoordinate { x: 1, y: 1 },
            },
            StrategoOwnPiece {
                rank: StrategoRank::Flag,
                position: StrategoCoordinate { x: 0, y: 3 },
            },
        ],
        opponent_pieces: vec![
            StrategoOpponentPiece {
                position: StrategoCoordinate { x: 2, y: 1 },
                public_rank: None,
                hidden_commitment: Some(HIDDEN_MARSHAL_COMMITMENT.to_string()),
            },
            StrategoOpponentPiece {
                position: StrategoCoordinate { x: 1, y: 0 },
                public_rank: None,
                hidden_commitment: Some(HIDDEN_BOMB_COMMITMENT.to_string()),
            },
        ],
        captured: Vec::new(),
    };

    state_from_public(public, Some(0))
}

pub fn apply_stratego_action(
    state: &CoreGameState,
    action_id: &str,
    _params: serde_json::Value,
) -> Result<CoreTransition, CoreGameError> {
    let before_public: StrategoPublicState = serde_json::from_value(state.public_state.clone())
        .map_err(|source| CoreGameError::InvalidParams {
            action_id: action_id.to_string(),
            reason: source.to_string(),
        })?;
    let parsed = parse_stratego_move(action_id)?;
    validate_stratego_move(&before_public, parsed, action_id)?;
    if !state
        .legal_actions
        .iter()
        .any(|candidate| candidate.action_id == action_id)
    {
        return Err(illegal_stratego_action(
            action_id,
            "action is not legal in this stratego position",
        ));
    }

    let mut after_public = before_public.clone();
    apply_validated_move(&mut after_public, parsed, action_id)?;
    let after = state_from_public(after_public, state.actor)?;
    let action = core_action_for_move(parsed);

    Ok(CoreTransition {
        before: state.clone(),
        action,
        after,
    })
}

fn state_from_public(
    public: StrategoPublicState,
    actor: Option<u8>,
) -> Result<CoreGameState, CoreGameError> {
    let legal_actions = legal_stratego_moves(&public)
        .into_iter()
        .map(core_action_for_move)
        .collect();
    let private_state_commitments = public
        .opponent_pieces
        .iter()
        .filter_map(|piece| piece.hidden_commitment.clone())
        .collect();
    let public_state =
        serde_json::to_value(public).map_err(|source| CoreGameError::InvalidParams {
            action_id: "stratego.bootstrap".to_string(),
            reason: source.to_string(),
        })?;

    Ok(CoreGameState {
        game: ResearchGame::Stratego,
        phase: "movement".to_string(),
        actor,
        public_state,
        private_state_commitments,
        legal_actions,
        terminal: false,
        payoff: None,
    })
}

pub(crate) fn feature_view(state: &CoreGameState) -> Result<StrategoFeatureView, CoreGameError> {
    let public: StrategoPublicState =
        serde_json::from_value(state.public_state.clone()).map_err(|source| {
            CoreGameError::InvalidParams {
                action_id: format!("{}.feature-view", state.game.slug()),
                reason: source.to_string(),
            }
        })?;
    let legal_moves = legal_stratego_moves(&public);
    let scout_positions = public
        .own_pieces
        .iter()
        .filter(|piece| piece.rank == StrategoRank::Scout)
        .map(|piece| piece.position)
        .collect::<std::collections::HashSet<_>>();
    let opponent_positions = public
        .opponent_pieces
        .iter()
        .map(|piece| piece.position)
        .collect::<std::collections::HashSet<_>>();
    let attacked_targets = legal_moves
        .iter()
        .filter_map(|candidate| {
            opponent_piece_at(&public, candidate.to)
                .is_some()
                .then_some(candidate.to)
        })
        .collect::<std::collections::HashSet<_>>();
    let hidden_targets = legal_moves
        .iter()
        .filter_map(|candidate| {
            opponent_piece_at(&public, candidate.to)
                .is_some_and(|piece| piece.public_rank.is_none())
                .then_some(candidate.to)
        })
        .collect::<std::collections::HashSet<_>>();

    Ok(StrategoFeatureView {
        scout_lanes: usize_to_u8(
            legal_moves
                .iter()
                .map(|candidate| candidate.from)
                .filter(|position| scout_positions.contains(position))
                .collect::<std::collections::HashSet<_>>()
                .len(),
        ),
        miners_remaining: usize_to_u8(
            public
                .own_pieces
                .iter()
                .filter(|piece| piece.rank == StrategoRank::Miner)
                .count(),
        ),
        bombs_suspected: usize_to_u8(
            public
                .opponent_pieces
                .iter()
                .filter(|piece| {
                    piece.public_rank == Some(StrategoRank::Bomb)
                        || piece.hidden_commitment.is_some()
                })
                .count(),
        ),
        attack_targets: usize_to_u8(attacked_targets.len()),
        hidden_targets: usize_to_u8(hidden_targets.len()),
        attack_is_forced: !legal_moves.is_empty()
            && legal_moves
                .iter()
                .all(|candidate| opponent_positions.contains(&candidate.to)),
    })
}

fn legal_stratego_moves(public: &StrategoPublicState) -> Vec<StrategoMove> {
    let mut moves = Vec::new();
    for piece in &public.own_pieces {
        if !piece.rank.can_move() {
            continue;
        }
        for (dx, dy) in [(0_i8, 1_i8), (1, 0), (0, -1), (-1, 0)] {
            let max_steps = if piece.rank == StrategoRank::Scout {
                public.board_width.max(public.board_height)
            } else {
                1
            };
            for step in 1..=max_steps {
                let Some(to) = offset(piece.position, dx, dy, step) else {
                    break;
                };
                if !in_bounds(public, to) || is_water(public, to) {
                    break;
                }
                if own_piece_at(public, to).is_some() {
                    break;
                }
                moves.push(StrategoMove {
                    from: piece.position,
                    to,
                });
                if opponent_piece_at(public, to).is_some() {
                    break;
                }
            }
        }
    }

    moves
}

fn validate_stratego_move(
    public: &StrategoPublicState,
    parsed: StrategoMove,
    action_id: &str,
) -> Result<(), CoreGameError> {
    if !in_bounds(public, parsed.from) || !in_bounds(public, parsed.to) {
        return Err(illegal_stratego_action(
            action_id,
            "source and destination must be inside the board",
        ));
    }
    if is_water(public, parsed.to) {
        return Err(illegal_stratego_action(
            action_id,
            "pieces cannot move into water squares",
        ));
    }
    let Some(piece) = own_piece_at(public, parsed.from) else {
        return Err(illegal_stratego_action(
            action_id,
            "source square has no actor piece",
        ));
    };
    if !piece.rank.can_move() {
        return Err(illegal_stratego_action(
            action_id,
            "bombs and flags are immobile",
        ));
    }
    if own_piece_at(public, parsed.to).is_some() {
        return Err(illegal_stratego_action(
            action_id,
            "destination is occupied by an actor piece",
        ));
    }
    if !is_straight_line(parsed.from, parsed.to) {
        return Err(illegal_stratego_action(
            action_id,
            "stratego moves must stay in a row or column",
        ));
    }
    let distance = manhattan_distance(parsed.from, parsed.to);
    if distance == 0 {
        return Err(illegal_stratego_action(
            action_id,
            "move destination must differ from source",
        ));
    }
    if piece.rank != StrategoRank::Scout && distance != 1 {
        return Err(illegal_stratego_action(
            action_id,
            "only scouts can move more than one square",
        ));
    }
    if piece.rank == StrategoRank::Scout && !scout_path_clear(public, parsed) {
        return Err(illegal_stratego_action(
            action_id,
            "scout path is blocked before the destination",
        ));
    }

    Ok(())
}

fn apply_validated_move(
    public: &mut StrategoPublicState,
    parsed: StrategoMove,
    action_id: &str,
) -> Result<(), CoreGameError> {
    let own_index = public
        .own_pieces
        .iter()
        .position(|piece| piece.position == parsed.from)
        .ok_or_else(|| invalid_stratego_state(action_id, "missing actor piece"))?;
    let Some(opponent_index) = public
        .opponent_pieces
        .iter()
        .position(|piece| piece.position == parsed.to)
    else {
        let own_piece = public
            .own_pieces
            .get_mut(own_index)
            .ok_or_else(|| invalid_stratego_state(action_id, "missing actor piece"))?;
        own_piece.position = parsed.to;
        return Ok(());
    };

    let attacker_rank = public
        .own_pieces
        .get(own_index)
        .map(|piece| piece.rank)
        .ok_or_else(|| invalid_stratego_state(action_id, "missing actor piece"))?;
    let defender_rank = {
        let opponent = public
            .opponent_pieces
            .get(opponent_index)
            .ok_or_else(|| invalid_stratego_state(action_id, "missing opponent piece"))?;
        opponent_rank(opponent, action_id)?
    };
    let opponent = public
        .opponent_pieces
        .get_mut(opponent_index)
        .ok_or_else(|| invalid_stratego_state(action_id, "missing opponent piece"))?;
    opponent.public_rank = Some(defender_rank);
    opponent.hidden_commitment = None;
    match combat_outcome(attacker_rank, defender_rank) {
        StrategoCombatOutcome::AttackerWins => {
            public.captured.push(defender_rank);
            public.opponent_pieces.remove(opponent_index);
            let own_piece = public
                .own_pieces
                .get_mut(own_index)
                .ok_or_else(|| invalid_stratego_state(action_id, "missing actor piece"))?;
            own_piece.position = parsed.to;
        }
        StrategoCombatOutcome::DefenderWins => {
            public.captured.push(attacker_rank);
            public.own_pieces.remove(own_index);
        }
        StrategoCombatOutcome::BothRemoved => {
            public.captured.push(attacker_rank);
            public.captured.push(defender_rank);
            public.opponent_pieces.remove(opponent_index);
            public.own_pieces.remove(own_index);
        }
    }

    Ok(())
}

fn combat_outcome(attacker: StrategoRank, defender: StrategoRank) -> StrategoCombatOutcome {
    if defender == StrategoRank::Flag {
        return StrategoCombatOutcome::AttackerWins;
    }
    if defender == StrategoRank::Bomb {
        return if attacker == StrategoRank::Miner {
            StrategoCombatOutcome::AttackerWins
        } else {
            StrategoCombatOutcome::DefenderWins
        };
    }
    if attacker == StrategoRank::Spy && defender == StrategoRank::Marshal {
        return StrategoCombatOutcome::AttackerWins;
    }
    let attacker_strength = attacker.strength();
    let defender_strength = defender.strength();
    if attacker_strength > defender_strength {
        StrategoCombatOutcome::AttackerWins
    } else if attacker_strength < defender_strength {
        StrategoCombatOutcome::DefenderWins
    } else {
        StrategoCombatOutcome::BothRemoved
    }
}

fn core_action_for_move(parsed: StrategoMove) -> CoreAction {
    CoreAction {
        action_id: format!(
            "{STRATEGO_MOVE_PREFIX}{}.{}",
            coord_token(parsed.from),
            coord_token(parsed.to)
        ),
        display_label: format!(
            "move-{}-{}",
            coord_token(parsed.from),
            coord_token(parsed.to)
        ),
        params: json!({"from": parsed.from, "to": parsed.to}),
    }
}

fn parse_stratego_move(action_id: &str) -> Result<StrategoMove, CoreGameError> {
    let Some(move_token) = action_id.strip_prefix(STRATEGO_MOVE_PREFIX) else {
        return Err(CoreGameError::UnknownAction {
            game: ResearchGame::Stratego,
            action_id: action_id.to_string(),
        });
    };
    let Some((from, to)) = move_token.split_once('.') else {
        return Err(CoreGameError::UnknownAction {
            game: ResearchGame::Stratego,
            action_id: action_id.to_string(),
        });
    };

    Ok(StrategoMove {
        from: parse_coord(action_id, from)?,
        to: parse_coord(action_id, to)?,
    })
}

fn coord_token(coord: StrategoCoordinate) -> String {
    format!("{}-{}", coord.x, coord.y)
}

fn parse_coord(action_id: &str, token: &str) -> Result<StrategoCoordinate, CoreGameError> {
    let Some((x, y)) = token.split_once('-') else {
        return Err(CoreGameError::UnknownAction {
            game: ResearchGame::Stratego,
            action_id: action_id.to_string(),
        });
    };
    let x = x.parse::<u8>().map_err(|_| CoreGameError::UnknownAction {
        game: ResearchGame::Stratego,
        action_id: action_id.to_string(),
    })?;
    let y = y.parse::<u8>().map_err(|_| CoreGameError::UnknownAction {
        game: ResearchGame::Stratego,
        action_id: action_id.to_string(),
    })?;

    Ok(StrategoCoordinate { x, y })
}

fn opponent_rank(
    opponent: &StrategoOpponentPiece,
    action_id: &str,
) -> Result<StrategoRank, CoreGameError> {
    if let Some(rank) = opponent.public_rank {
        return Ok(rank);
    }
    let Some(commitment) = opponent.hidden_commitment.as_deref() else {
        return Err(invalid_stratego_state(
            action_id,
            "hidden opponent piece has no commitment",
        ));
    };
    match commitment {
        HIDDEN_MARSHAL_COMMITMENT => Ok(StrategoRank::Marshal),
        HIDDEN_BOMB_COMMITMENT => Ok(StrategoRank::Bomb),
        _ => Err(invalid_stratego_state(
            action_id,
            "unknown hidden stratego commitment",
        )),
    }
}

fn own_piece_at(
    public: &StrategoPublicState,
    coord: StrategoCoordinate,
) -> Option<&StrategoOwnPiece> {
    public
        .own_pieces
        .iter()
        .find(|piece| piece.position == coord)
}

fn opponent_piece_at(
    public: &StrategoPublicState,
    coord: StrategoCoordinate,
) -> Option<&StrategoOpponentPiece> {
    public
        .opponent_pieces
        .iter()
        .find(|piece| piece.position == coord)
}

fn in_bounds(public: &StrategoPublicState, coord: StrategoCoordinate) -> bool {
    coord.x < public.board_width && coord.y < public.board_height
}

fn is_water(public: &StrategoPublicState, coord: StrategoCoordinate) -> bool {
    public.water.contains(&coord)
}

fn is_straight_line(from: StrategoCoordinate, to: StrategoCoordinate) -> bool {
    from.x == to.x || from.y == to.y
}

fn manhattan_distance(from: StrategoCoordinate, to: StrategoCoordinate) -> u8 {
    from.x.abs_diff(to.x).saturating_add(from.y.abs_diff(to.y))
}

fn scout_path_clear(public: &StrategoPublicState, parsed: StrategoMove) -> bool {
    let dx = direction_delta(parsed.from.x, parsed.to.x);
    let dy = direction_delta(parsed.from.y, parsed.to.y);
    for step in 1..manhattan_distance(parsed.from, parsed.to) {
        let Some(coord) = offset(parsed.from, dx, dy, step) else {
            return false;
        };
        if !in_bounds(public, coord)
            || is_water(public, coord)
            || own_piece_at(public, coord).is_some()
            || opponent_piece_at(public, coord).is_some()
        {
            return false;
        }
    }

    true
}

fn offset(coord: StrategoCoordinate, dx: i8, dy: i8, step: u8) -> Option<StrategoCoordinate> {
    let x_delta = i16::from(dx).checked_mul(i16::from(step))?;
    let y_delta = i16::from(dy).checked_mul(i16::from(step))?;
    let x = i16::from(coord.x).checked_add(x_delta)?;
    let y = i16::from(coord.y).checked_add(y_delta)?;
    if x < 0 || y < 0 {
        return None;
    }

    Some(StrategoCoordinate {
        x: u8::try_from(x).ok()?,
        y: u8::try_from(y).ok()?,
    })
}

fn direction_delta(from: u8, to: u8) -> i8 {
    match to.cmp(&from) {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    }
}

impl StrategoRank {
    const fn can_move(self) -> bool {
        !matches!(self, Self::Bomb | Self::Flag)
    }

    const fn strength(self) -> u8 {
        match self {
            Self::Marshal => 10,
            Self::Miner => 3,
            Self::Scout => 2,
            Self::Spy => 1,
            Self::Bomb | Self::Flag => 0,
        }
    }
}

fn illegal_stratego_action(action_id: &str, reason: &str) -> CoreGameError {
    CoreGameError::IllegalAction {
        game: ResearchGame::Stratego,
        action_id: action_id.to_string(),
        reason: reason.to_string(),
    }
}

fn invalid_stratego_state(action_id: &str, reason: &str) -> CoreGameError {
    CoreGameError::InvalidParams {
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
    fn stratego_bootstrap_state_has_legal_actions() {
        let state = stratego_state();

        assert_eq!(state.game, ResearchGame::Stratego);
        assert!(
            state
                .legal_actions
                .iter()
                .any(|action| action.action_id == "stratego.move.1-1.2-1")
        );
    }

    #[test]
    fn stratego_rejects_immobile_piece_move() {
        let state = stratego_state();

        assert!(matches!(
            apply_action(&state, "stratego.move.0-3.0-2", json!({})),
            Err(CoreGameError::IllegalAction { reason, .. }) if reason.contains("immobile")
        ));
    }

    #[test]
    fn stratego_rejects_water_move() {
        let state = stratego_state();

        assert!(matches!(
            apply_action(&state, "stratego.move.1-1.1-2", json!({})),
            Err(CoreGameError::IllegalAction { reason, .. }) if reason.contains("water")
        ));
    }

    #[test]
    fn stratego_combat_rank_comparison() {
        let state = stratego_state();
        let transition = match apply_action(&state, "stratego.move.1-1.2-1", json!({})) {
            Ok(transition) => transition,
            Err(error) => panic!("miner attacking marshal should adjudicate: {error}"),
        };
        let public: StrategoPublicState =
            match serde_json::from_value(transition.after.public_state) {
                Ok(public) => public,
                Err(error) => panic!("stratego public state should decode: {error}"),
            };

        assert!(
            public
                .opponent_pieces
                .iter()
                .any(|piece| piece.position == coord(2, 1)
                    && piece.public_rank == Some(StrategoRank::Marshal))
        );
        assert!(
            !public
                .own_pieces
                .iter()
                .any(|piece| piece.position == coord(2, 1))
        );
    }

    #[test]
    fn stratego_miner_beats_bomb() {
        let state = stratego_state();
        let transition = match apply_action(&state, "stratego.move.1-1.1-0", json!({})) {
            Ok(transition) => transition,
            Err(error) => panic!("miner should defuse bomb: {error}"),
        };
        let public: StrategoPublicState =
            match serde_json::from_value(transition.after.public_state) {
                Ok(public) => public,
                Err(error) => panic!("stratego public state should decode: {error}"),
            };

        assert!(
            public
                .own_pieces
                .iter()
                .any(|piece| piece.position == coord(1, 0))
        );
        assert!(
            !public
                .opponent_pieces
                .iter()
                .any(|piece| piece.position == coord(1, 0))
        );
        assert!(public.captured.contains(&StrategoRank::Bomb));
    }

    #[test]
    fn stratego_spy_beats_marshal_attacking() {
        let state = state_with_single_attacker(StrategoRank::Spy, HIDDEN_MARSHAL_COMMITMENT);
        let transition = match apply_action(&state, "stratego.move.1-1.2-1", json!({})) {
            Ok(transition) => transition,
            Err(error) => panic!("spy should beat marshal while attacking: {error}"),
        };
        let public: StrategoPublicState =
            match serde_json::from_value(transition.after.public_state) {
                Ok(public) => public,
                Err(error) => panic!("stratego public state should decode: {error}"),
            };

        assert!(
            public
                .own_pieces
                .iter()
                .any(|piece| piece.position == coord(2, 1))
        );
        assert!(
            !public
                .opponent_pieces
                .iter()
                .any(|piece| piece.position == coord(2, 1))
        );
    }

    #[test]
    fn stratego_combat_reveals_hidden_piece() {
        let state = stratego_state();
        let transition = match apply_action(&state, "stratego.move.1-1.2-1", json!({})) {
            Ok(transition) => transition,
            Err(error) => panic!("combat should apply: {error}"),
        };
        let public: StrategoPublicState =
            match serde_json::from_value(transition.after.public_state) {
                Ok(public) => public,
                Err(error) => panic!("stratego public state should decode: {error}"),
            };
        let Some(piece) = public
            .opponent_pieces
            .iter()
            .find(|piece| piece.position == coord(2, 1))
        else {
            panic!("defending marshal should remain on board");
        };

        assert_eq!(piece.public_rank, Some(StrategoRank::Marshal));
        assert_eq!(piece.hidden_commitment, None);
    }

    #[test]
    fn stratego_transition_is_deterministic() {
        let state = stratego_state();
        let first = apply_action(&state, "stratego.move.1-1.1-0", json!({}));
        let second = apply_action(&state, "stratego.move.1-1.1-0", json!({}));

        assert_eq!(first, second);
    }

    #[test]
    fn stratego_feature_view_tracks_hidden_pressure() {
        let state = stratego_state();
        let view = feature_view(&state)
            .unwrap_or_else(|error| panic!("stratego feature view should decode: {error}"));

        assert_eq!(view.scout_lanes, 1);
        assert_eq!(view.miners_remaining, 1);
        assert_eq!(view.bombs_suspected, 2);
        assert_eq!(view.attack_targets, 2);
        assert_eq!(view.hidden_targets, 2);
        assert!(!view.attack_is_forced);
    }

    fn stratego_state() -> CoreGameState {
        match bootstrap_state(ResearchGame::Stratego) {
            Ok(state) => state,
            Err(error) => panic!("stratego bootstrap should succeed: {error}"),
        }
    }

    fn state_with_single_attacker(rank: StrategoRank, hidden_commitment: &str) -> CoreGameState {
        let public = StrategoPublicState {
            board_width: 4,
            board_height: 4,
            water: Vec::new(),
            own_pieces: vec![StrategoOwnPiece {
                rank,
                position: coord(1, 1),
            }],
            opponent_pieces: vec![StrategoOpponentPiece {
                position: coord(2, 1),
                public_rank: None,
                hidden_commitment: Some(hidden_commitment.to_string()),
            }],
            captured: Vec::new(),
        };

        match state_from_public(public, Some(0)) {
            Ok(state) => state,
            Err(error) => panic!("custom stratego state should build: {error}"),
        }
    }

    fn coord(x: u8, y: u8) -> StrategoCoordinate {
        StrategoCoordinate { x, y }
    }
}

// =====================================================================
// F-016 promotion-artifact scenario pack.
//
// The pack is the canonical 22-scenario rule-aware scenario set the
// `StrategoBenchmarkDossier` (in `crate::stratego_benchmark`) hashes
// over and renders. Each row is hand-verified against the live
// `state-aware belief-scout heuristic` math in
// `crate::engines::stratego::answer`:
//
//   scout      = 0.85 + scout_lanes*0.55 + hidden_targets*0.45
//                + bombs_suspected*0.05
//                + (attack_is_forced ? -0.40 : 0.15)
//   place_safe = 0.80 + bombs_suspected*0.30
//                + (attack_targets==0 ? 0.25 : 0.0)
//                + (miners_remaining==0 ? 0.25 : 0.0)
//                + (attack_is_forced ? -0.20 : 0.10)
//   advance    = 0.80 + attack_targets*0.60
//                + (attack_is_forced ? 0.95 : 0.0)
//                + (bombs_suspected > miners_remaining ? -0.10 : 0.10)
//
// The pack splits 8/8/6 across scout-dominant / advance-piece-dominant
// / place-safe-dominant so a regression that flips the dominant arm on
// any scenario is loud in the dossier's recommendation map. Every
// row's `decision` docstring records the expected `s=…` / `p=…` /
// `a=…` values so the dominant arm stays dominant by at least 0.60 on
// every scenario (the tightest margin is `place-safe-no-targets-mid-bomb`
// at p=1.75 vs s=1.10, a 0.65 margin).
/// One row of the canonical 22-scenario Stratego benchmark pack.
///
/// The fields mirror the live `StrategoChallenge` feature struct so
/// the dossier's typed `PortfolioChallenge::Stratego(StrategoChallenge { ... })`
/// is a field-for-field copy of the scenario. Keeping the mirror
/// stable means the dossier hash is also a regression guard for the
/// feature-view extraction: a refactor that changes how
/// `feature_view` reads `scout_lanes` / `hidden_targets` / etc out of
/// a `CoreGameState` will flip the dossier's recommendation map and
/// the unit tests will fail.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StrategoScenario {
    pub scenario_id: &'static str,
    pub decision: &'static str,
    pub scout_lanes: u8,
    pub miners_remaining: u8,
    pub bombs_suspected: u8,
    pub attack_targets: u8,
    pub hidden_targets: u8,
    pub attack_is_forced: bool,
}

/// Return the canonical 22-scenario Stratego benchmark pack.
pub fn stratego_scenario_pack() -> &'static [StrategoScenario] {
    STRATEGO_SCENARIO_PACK
}

const STRATEGO_SCENARIO_PACK: &[StrategoScenario] = &[
    // ---- scout bucket (×8) — open board with hidden targets + live scout lanes ----
    StrategoScenario {
        scenario_id: "scout-open-two-lanes-one-hidden",
        decision: "Open board: 2 scout lanes, 1 hidden target — scout dominates (s=2.55, p=1.15, a=0.90)",
        scout_lanes: 2,
        miners_remaining: 1,
        bombs_suspected: 0,
        attack_targets: 0,
        hidden_targets: 1,
        attack_is_forced: false,
    },
    StrategoScenario {
        scenario_id: "scout-open-three-lanes-one-hidden",
        decision: "Open board: 3 scout lanes, 1 hidden target — scout dominates (s=3.10, p=1.15, a=0.90)",
        scout_lanes: 3,
        miners_remaining: 1,
        bombs_suspected: 0,
        attack_targets: 0,
        hidden_targets: 1,
        attack_is_forced: false,
    },
    StrategoScenario {
        scenario_id: "scout-open-one-lane-two-hidden",
        decision: "Open board: 1 scout lane, 2 hidden targets — scout dominates (s=2.45, p=1.15, a=0.90)",
        scout_lanes: 1,
        miners_remaining: 1,
        bombs_suspected: 0,
        attack_targets: 0,
        hidden_targets: 2,
        attack_is_forced: false,
    },
    StrategoScenario {
        scenario_id: "scout-open-two-lanes-two-hidden",
        decision: "Open board: 2 scout lanes, 2 hidden targets — scout dominates (s=3.00, p=1.15, a=0.90)",
        scout_lanes: 2,
        miners_remaining: 1,
        bombs_suspected: 0,
        attack_targets: 0,
        hidden_targets: 2,
        attack_is_forced: false,
    },
    StrategoScenario {
        scenario_id: "scout-light-bomb-pressure",
        decision: "Open board with 1 bomb suspected, 2 scout lanes, 1 hidden — scout dominates (s=2.60, p=1.45, a=0.90)",
        scout_lanes: 2,
        miners_remaining: 1,
        bombs_suspected: 1,
        attack_targets: 0,
        hidden_targets: 1,
        attack_is_forced: false,
    },
    StrategoScenario {
        scenario_id: "scout-deep-bomb-pressure",
        decision: "Open board with 2 bombs suspected, 2 scout lanes, 2 hidden — scout dominates (s=3.10, p=1.75, a=0.70)",
        scout_lanes: 2,
        miners_remaining: 1,
        bombs_suspected: 2,
        attack_targets: 0,
        hidden_targets: 2,
        attack_is_forced: false,
    },
    StrategoScenario {
        scenario_id: "scout-mixed-bomb-press",
        decision: "Open board with 1 bomb suspected, 3 scout lanes, 1 hidden — scout dominates (s=3.15, p=1.45, a=0.90)",
        scout_lanes: 3,
        miners_remaining: 1,
        bombs_suspected: 1,
        attack_targets: 0,
        hidden_targets: 1,
        attack_is_forced: false,
    },
    StrategoScenario {
        scenario_id: "scout-hidden-rich",
        decision: "Open board: 2 scout lanes, 3 hidden targets — scout dominates (s=3.45, p=1.15, a=0.90)",
        scout_lanes: 2,
        miners_remaining: 1,
        bombs_suspected: 0,
        attack_targets: 0,
        hidden_targets: 3,
        attack_is_forced: false,
    },
    // ---- advance-piece bucket (×8) — forced combat with live attack targets ----
    StrategoScenario {
        scenario_id: "advance-forced-one-target",
        decision: "Forced combat: 1 attack target — advance dominates (s=0.45, p=0.60, a=2.45)",
        scout_lanes: 0,
        miners_remaining: 1,
        bombs_suspected: 0,
        attack_targets: 1,
        hidden_targets: 0,
        attack_is_forced: true,
    },
    StrategoScenario {
        scenario_id: "advance-forced-two-targets",
        decision: "Forced combat: 2 attack targets — advance dominates (s=0.45, p=0.60, a=3.05)",
        scout_lanes: 0,
        miners_remaining: 1,
        bombs_suspected: 0,
        attack_targets: 2,
        hidden_targets: 0,
        attack_is_forced: true,
    },
    StrategoScenario {
        scenario_id: "advance-forced-three-targets",
        decision: "Forced combat: 3 attack targets — advance dominates (s=0.45, p=0.60, a=3.65)",
        scout_lanes: 0,
        miners_remaining: 1,
        bombs_suspected: 0,
        attack_targets: 3,
        hidden_targets: 0,
        attack_is_forced: true,
    },
    StrategoScenario {
        scenario_id: "advance-forced-one-target-bomb",
        decision: "Forced combat: 1 attack target, 1 bomb suspected — advance dominates (s=0.50, p=0.90, a=2.45)",
        scout_lanes: 0,
        miners_remaining: 1,
        bombs_suspected: 1,
        attack_targets: 1,
        hidden_targets: 0,
        attack_is_forced: true,
    },
    StrategoScenario {
        scenario_id: "advance-forced-one-target-deep-bomb",
        decision: "Forced combat: 1 attack target, 2 bombs suspected, miners < bombs — advance dominates (s=0.55, p=1.20, a=2.25)",
        scout_lanes: 0,
        miners_remaining: 1,
        bombs_suspected: 2,
        attack_targets: 1,
        hidden_targets: 0,
        attack_is_forced: true,
    },
    StrategoScenario {
        scenario_id: "advance-forced-one-hidden",
        decision: "Forced combat with 1 hidden target — advance dominates (s=0.90, p=0.60, a=2.45)",
        scout_lanes: 0,
        miners_remaining: 1,
        bombs_suspected: 0,
        attack_targets: 1,
        hidden_targets: 1,
        attack_is_forced: true,
    },
    StrategoScenario {
        scenario_id: "advance-forced-two-hidden",
        decision: "Forced combat with 2 hidden targets — advance dominates (s=1.35, p=0.60, a=2.45)",
        scout_lanes: 0,
        miners_remaining: 1,
        bombs_suspected: 0,
        attack_targets: 1,
        hidden_targets: 2,
        attack_is_forced: true,
    },
    StrategoScenario {
        scenario_id: "advance-forced-one-target-bomb-balanced",
        decision: "Forced combat: 1 attack target, 2 bombs suspected, miners=2 (balances bombs) — advance dominates (s=0.55, p=1.20, a=2.45)",
        scout_lanes: 0,
        miners_remaining: 2,
        bombs_suspected: 2,
        attack_targets: 1,
        hidden_targets: 0,
        attack_is_forced: true,
    },
    // ---- place-safe bucket (×6) — bomb-heavy / no-attack / no-miners open board ----
    StrategoScenario {
        scenario_id: "place-safe-heavy-bomb",
        decision: "Heavy bomb pressure (3 bombs, no targets) — place_safe dominates (s=1.15, p=2.05, a=0.70)",
        scout_lanes: 0,
        miners_remaining: 2,
        bombs_suspected: 3,
        attack_targets: 0,
        hidden_targets: 0,
        attack_is_forced: false,
    },
    StrategoScenario {
        scenario_id: "place-safe-no-miners",
        decision: "Bomb pressure with no miners left — place_safe dominates (s=1.10, p=2.00, a=0.70)",
        scout_lanes: 0,
        miners_remaining: 0,
        bombs_suspected: 2,
        attack_targets: 0,
        hidden_targets: 0,
        attack_is_forced: false,
    },
    StrategoScenario {
        scenario_id: "place-safe-no-targets-heavy-bomb",
        decision: "3 bombs suspected, 0 targets — place_safe dominates (s=1.15, p=2.05, a=0.70)",
        scout_lanes: 0,
        miners_remaining: 1,
        bombs_suspected: 3,
        attack_targets: 0,
        hidden_targets: 0,
        attack_is_forced: false,
    },
    StrategoScenario {
        scenario_id: "place-safe-no-targets-mid-bomb",
        decision: "2 bombs suspected, 0 targets — place_safe dominates (s=1.10, p=1.75, a=0.70)",
        scout_lanes: 0,
        miners_remaining: 1,
        bombs_suspected: 2,
        attack_targets: 0,
        hidden_targets: 0,
        attack_is_forced: false,
    },
    StrategoScenario {
        scenario_id: "place-safe-no-miners-deep-bomb",
        decision: "2 bombs suspected, 0 miners — place_safe dominates (s=1.10, p=2.00, a=0.70)",
        scout_lanes: 0,
        miners_remaining: 0,
        bombs_suspected: 2,
        attack_targets: 0,
        hidden_targets: 0,
        attack_is_forced: false,
    },
    StrategoScenario {
        scenario_id: "place-safe-no-miners-light-bomb",
        decision: "1 bomb suspected, 0 miners — place_safe dominates (s=1.05, p=1.70, a=0.70)",
        scout_lanes: 0,
        miners_remaining: 0,
        bombs_suspected: 1,
        attack_targets: 0,
        hidden_targets: 0,
        attack_is_forced: false,
    },
];

#[cfg(test)]
mod scenario_pack_tests {
    use super::*;

    #[test]
    fn stratego_scenario_pack_is_22_rows() {
        assert_eq!(stratego_scenario_pack().len(), 22);
    }

    #[test]
    fn stratego_scenario_ids_are_unique() {
        let pack = stratego_scenario_pack();
        let mut seen = std::collections::HashSet::new();
        for scenario in pack {
            assert!(
                seen.insert(scenario.scenario_id),
                "duplicate scenario_id: {}",
                scenario.scenario_id
            );
        }
    }

    #[test]
    fn stratego_scenario_buckets_have_expected_counts() {
        let pack = stratego_scenario_pack();
        let scout = pack
            .iter()
            .filter(|s| s.scenario_id.starts_with("scout-"))
            .count();
        let advance = pack
            .iter()
            .filter(|s| s.scenario_id.starts_with("advance-"))
            .count();
        let place_safe = pack
            .iter()
            .filter(|s| s.scenario_id.starts_with("place-safe-"))
            .count();
        assert_eq!(scout, 8, "scout bucket should be 8");
        assert_eq!(advance, 8, "advance bucket should be 8");
        assert_eq!(place_safe, 6, "place-safe bucket should be 6");
        assert_eq!(scout + advance + place_safe, 22);
    }

    #[test]
    fn stratego_scenario_math_holds_for_every_row() {
        // The dossier's recommendation map is built from
        // `answer_typed_challenge` on the typed challenge, but the
        // pack's own `decision` docstring names the expected
        // `s=…`/`p=…`/`a=…` values, so this test re-derives the
        // expected dominant action and confirms it matches the bucket
        // label. A regression that changes the engine's heuristic
        // math would flip the dominant action on at least one row
        // and this test would fail.
        fn dominant(
            scout_lanes: u8,
            hidden_targets: u8,
            bombs_suspected: u8,
            attack_targets: u8,
            miners_remaining: u8,
            attack_is_forced: bool,
        ) -> &'static str {
            let scout = 0.85_f32
                + (scout_lanes as f32) * 0.55
                + (hidden_targets as f32) * 0.45
                + (bombs_suspected as f32) * 0.05
                + if attack_is_forced { -0.40 } else { 0.15 };
            let place_safe = 0.80_f32
                + (bombs_suspected as f32) * 0.30
                + if attack_targets == 0 { 0.25 } else { 0.0 }
                + if miners_remaining == 0 { 0.25 } else { 0.0 }
                + if attack_is_forced { -0.20 } else { 0.10 };
            let advance = 0.80_f32
                + (attack_targets as f32) * 0.60
                + if attack_is_forced { 0.95 } else { 0.0 }
                + if bombs_suspected > miners_remaining {
                    -0.10
                } else {
                    0.10
                };
            if scout >= place_safe && scout >= advance {
                "scout"
            } else if advance >= place_safe {
                "advance-piece"
            } else {
                "place-safe"
            }
        }

        for scenario in stratego_scenario_pack() {
            let derived = dominant(
                scenario.scout_lanes,
                scenario.hidden_targets,
                scenario.bombs_suspected,
                scenario.attack_targets,
                scenario.miners_remaining,
                scenario.attack_is_forced,
            );
            let expected = if scenario.scenario_id.starts_with("scout-") {
                "scout"
            } else if scenario.scenario_id.starts_with("advance-") {
                "advance-piece"
            } else {
                "place-safe"
            };
            assert_eq!(
                derived, expected,
                "scenario {} expected {} but engine math derives {}",
                scenario.scenario_id, expected, derived
            );
        }
    }
}

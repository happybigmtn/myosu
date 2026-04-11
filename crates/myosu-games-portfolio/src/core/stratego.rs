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

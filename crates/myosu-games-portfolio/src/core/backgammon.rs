use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::core::model::{CoreAction, CoreGameError, CoreGameState, CoreTransition};
use crate::game::ResearchGame;

const BACKGAMMON_MOVE_PREFIX: &str = "backgammon.move.";
const BACKGAMMON_TAKE_DOUBLE_ID: &str = "backgammon.take-double";
const BACKGAMMON_DROP_DOUBLE_ID: &str = "backgammon.drop-double";
const BACKGAMMON_CHECKERS_PER_SIDE: u8 = 15;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
struct BackgammonPublicState {
    own_points: Vec<u8>,
    opponent_points: Vec<u8>,
    bar: [u8; 2],
    borne_off: [u8; 2],
    dice: Vec<u8>,
    cube_owner: Option<u8>,
    cube_value: u8,
    cube_available: bool,
    cube_offered_by_opponent: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BackgammonSource {
    Bar,
    Point(u8),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BackgammonDest {
    Point(u8),
    Off,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct BackgammonMove {
    source: BackgammonSource,
    dest: BackgammonDest,
    die: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BackgammonFeatureView {
    pub race_lead_pips: i16,
    pub borne_off: u8,
    pub anchors: u8,
    pub cube_efficiency: u8,
    pub has_contact: bool,
    pub bar_count: u8,
    pub bearoff_ready: bool,
    pub cube_centered: bool,
    pub cube_owned_by_actor: bool,
    pub facing_double: bool,
    pub move_options: u8,
    pub off_moves: u8,
    pub blot_count: u8,
    pub home_board_points: u8,
    pub prime_length: u8,
}

pub fn backgammon_bootstrap_state() -> Result<CoreGameState, CoreGameError> {
    state_from_public(backgammon_cube_decision_public(), Some(0))
}

pub fn apply_backgammon_action(
    state: &CoreGameState,
    action_id: &str,
    _params: serde_json::Value,
) -> Result<CoreTransition, CoreGameError> {
    let before_public: BackgammonPublicState = serde_json::from_value(state.public_state.clone())
        .map_err(|source| CoreGameError::InvalidParams {
        action_id: action_id.to_string(),
        reason: source.to_string(),
    })?;
    if before_public.cube_offered_by_opponent {
        if !state
            .legal_actions
            .iter()
            .any(|candidate| candidate.action_id == action_id)
        {
            return Err(CoreGameError::UnknownAction {
                game: ResearchGame::Backgammon,
                action_id: action_id.to_string(),
            });
        }
        return apply_cube_decision(state, before_public, action_id);
    }

    let parsed = parse_backgammon_move(action_id)?;
    if before_public.bar.first().copied().unwrap_or(0) > 0
        && !matches!(parsed.source, BackgammonSource::Bar)
    {
        return Err(CoreGameError::IllegalAction {
            game: ResearchGame::Backgammon,
            action_id: action_id.to_string(),
            reason: "bar entry must be played before other checker moves".to_string(),
        });
    }
    if matches!(parsed.dest, BackgammonDest::Off) && !all_checkers_in_home(&before_public) {
        return Err(CoreGameError::IllegalAction {
            game: ResearchGame::Backgammon,
            action_id: action_id.to_string(),
            reason: "bearing off is legal only when all checkers are in the home board".to_string(),
        });
    }
    if !state
        .legal_actions
        .iter()
        .any(|candidate| candidate.action_id == action_id)
    {
        return Err(CoreGameError::UnknownAction {
            game: ResearchGame::Backgammon,
            action_id: action_id.to_string(),
        });
    }

    let mut after_public = before_public.clone();
    apply_checker_move(&mut after_public, parsed, action_id)?;
    let actor = state.actor.unwrap_or_default();
    let after = if side_has_borne_off_all(&after_public, 0) {
        let stake = i64::from(after_public.cube_value.max(1));
        state_from_public_with_terminal(
            after_public,
            Some(payoff_for_loser(opposing_actor(actor), stake)),
        )?
    } else if checker_turn_is_complete(&after_public) {
        rotate_perspective(&mut after_public);
        after_public.dice = representative_roll(&after_public);
        state_from_public(after_public, Some(opposing_actor(actor)))?
    } else {
        state_from_public(after_public, state.actor)?
    };
    let action = core_action_for_move(parsed);

    Ok(CoreTransition {
        before: state.clone(),
        action,
        after,
    })
}

fn apply_cube_decision(
    state: &CoreGameState,
    before_public: BackgammonPublicState,
    action_id: &str,
) -> Result<CoreTransition, CoreGameError> {
    match action_id {
        BACKGAMMON_TAKE_DOUBLE_ID => {
            let mut after_public = before_public.clone();
            after_public.cube_offered_by_opponent = false;
            after_public.cube_value = after_public.cube_value.saturating_mul(2).max(2);
            after_public.cube_owner = state.actor;
            after_public.cube_available = true;
            let next_actor = opposing_actor(state.actor.unwrap_or_default());
            rotate_perspective(&mut after_public);
            let after = state_from_public(after_public, Some(next_actor))?;
            let action = cube_decision_action(BACKGAMMON_TAKE_DOUBLE_ID, "take-double");

            Ok(CoreTransition {
                before: state.clone(),
                action,
                after,
            })
        }
        BACKGAMMON_DROP_DOUBLE_ID => {
            let mut after_public = before_public;
            after_public.cube_offered_by_opponent = false;
            let stake = i64::from(after_public.cube_value.max(1));
            let loser = state.actor.unwrap_or_default();
            let after = state_from_public_with_terminal(
                after_public,
                Some(payoff_for_loser(loser, stake)),
            )?;
            let action = cube_decision_action(BACKGAMMON_DROP_DOUBLE_ID, "drop-double");

            Ok(CoreTransition {
                before: state.clone(),
                action,
                after,
            })
        }
        _ => Err(CoreGameError::UnknownAction {
            game: ResearchGame::Backgammon,
            action_id: action_id.to_string(),
        }),
    }
}

fn state_from_public(
    public: BackgammonPublicState,
    actor: Option<u8>,
) -> Result<CoreGameState, CoreGameError> {
    state_from_public_with_terminal_and_actor(public, actor, None)
}

fn state_from_public_with_terminal(
    public: BackgammonPublicState,
    payoff: Option<Vec<i64>>,
) -> Result<CoreGameState, CoreGameError> {
    state_from_public_with_terminal_and_actor(public, None, payoff)
}

fn state_from_public_with_terminal_and_actor(
    public: BackgammonPublicState,
    actor: Option<u8>,
    payoff: Option<Vec<i64>>,
) -> Result<CoreGameState, CoreGameError> {
    let terminal = payoff.is_some();
    let phase = if public.cube_offered_by_opponent {
        "cube-decision"
    } else {
        "checker-play"
    };
    let legal_actions = if terminal {
        Vec::new()
    } else if public.cube_offered_by_opponent {
        vec![
            cube_decision_action(BACKGAMMON_TAKE_DOUBLE_ID, "take-double"),
            cube_decision_action(BACKGAMMON_DROP_DOUBLE_ID, "drop-double"),
        ]
    } else {
        legal_backgammon_moves(&public)
            .into_iter()
            .map(core_action_for_move)
            .collect()
    };
    let public_state =
        serde_json::to_value(public).map_err(|source| CoreGameError::InvalidParams {
            action_id: "backgammon.bootstrap".to_string(),
            reason: source.to_string(),
        })?;

    Ok(CoreGameState {
        game: ResearchGame::Backgammon,
        phase: phase.to_string(),
        actor: (!terminal).then_some(actor.unwrap_or_default()),
        public_state,
        private_state_commitments: vec!["backgammon.hidden-cube-state.bootstrap-v1".to_string()],
        legal_actions,
        terminal,
        payoff,
    })
}

pub(crate) fn feature_view(state: &CoreGameState) -> Result<BackgammonFeatureView, CoreGameError> {
    let public: BackgammonPublicState = serde_json::from_value(state.public_state.clone())
        .map_err(|source| CoreGameError::InvalidParams {
            action_id: format!("{}.feature-view", state.game.slug()),
            reason: source.to_string(),
        })?;
    let actor = state.actor.unwrap_or_default();
    let own_remaining =
        remaining_pips(&public.own_points, public.bar.first().copied().unwrap_or(0));
    let opponent_remaining = remaining_pips(
        &public.opponent_points,
        public.bar.get(1).copied().unwrap_or(0),
    );
    let moves = legal_backgammon_moves(&public);

    Ok(BackgammonFeatureView {
        race_lead_pips: pip_delta(opponent_remaining, own_remaining),
        borne_off: public.borne_off.first().copied().unwrap_or_default(),
        anchors: usize_to_u8(
            public
                .own_points
                .iter()
                .filter(|count| **count >= 2)
                .count(),
        ),
        cube_efficiency: effective_cube_value(&public),
        has_contact: sides_still_have_contact(&public),
        bar_count: public.bar.first().copied().unwrap_or_default(),
        bearoff_ready: all_checkers_in_home(&public),
        cube_centered: public.cube_owner.is_none(),
        cube_owned_by_actor: public.cube_owner == Some(actor),
        facing_double: public.cube_offered_by_opponent,
        move_options: usize_to_u8(moves.len()),
        off_moves: usize_to_u8(
            moves
                .iter()
                .filter(|candidate| matches!(candidate.dest, BackgammonDest::Off))
                .count(),
        ),
        blot_count: usize_to_u8(
            public
                .own_points
                .iter()
                .filter(|count| **count == 1)
                .count(),
        ),
        home_board_points: made_points_in_range(&public.own_points, 19, 24),
        prime_length: longest_prime(&public.own_points),
    })
}

fn legal_backgammon_moves(public: &BackgammonPublicState) -> Vec<BackgammonMove> {
    if public.bar.first().copied().unwrap_or(0) > 0 {
        return public
            .dice
            .iter()
            .copied()
            .filter_map(|die| {
                let dest = 25u8.checked_sub(die)?;
                Some(BackgammonMove {
                    source: BackgammonSource::Bar,
                    dest: BackgammonDest::Point(dest),
                    die,
                })
            })
            .collect();
    }

    let mut moves = Vec::new();
    for (index, count) in public.own_points.iter().copied().enumerate() {
        if count == 0 {
            continue;
        }
        let Ok(zero_based) = u8::try_from(index) else {
            continue;
        };
        let Some(source) = zero_based.checked_add(1) else {
            continue;
        };
        for die in &public.dice {
            let Some(dest) = source.checked_add(*die) else {
                continue;
            };
            let dest = if dest > 24 {
                if all_checkers_in_home(public) {
                    BackgammonDest::Off
                } else {
                    continue;
                }
            } else {
                BackgammonDest::Point(dest)
            };
            moves.push(BackgammonMove {
                source: BackgammonSource::Point(source),
                dest,
                die: *die,
            });
        }
    }

    moves
}

fn apply_checker_move(
    public: &mut BackgammonPublicState,
    parsed: BackgammonMove,
    action_id: &str,
) -> Result<(), CoreGameError> {
    match parsed.source {
        BackgammonSource::Bar => {
            let Some(bar) = public.bar.first_mut() else {
                return Err(invalid_backgammon_state(action_id, "missing actor bar"));
            };
            if *bar == 0 {
                return Err(invalid_backgammon_state(action_id, "bar is empty"));
            }
            *bar = bar.saturating_sub(1);
        }
        BackgammonSource::Point(point) => {
            let Some(source) = point_mut(&mut public.own_points, point) else {
                return Err(invalid_backgammon_state(
                    action_id,
                    "source point out of range",
                ));
            };
            if *source == 0 {
                return Err(CoreGameError::IllegalAction {
                    game: ResearchGame::Backgammon,
                    action_id: action_id.to_string(),
                    reason: "source point has no checker".to_string(),
                });
            }
            *source = source.saturating_sub(1);
        }
    }

    match parsed.dest {
        BackgammonDest::Point(point) => {
            let Some(dest) = point_mut(&mut public.own_points, point) else {
                return Err(invalid_backgammon_state(
                    action_id,
                    "destination point out of range",
                ));
            };
            *dest = dest.saturating_add(1);
        }
        BackgammonDest::Off => {
            let Some(borne_off) = public.borne_off.first_mut() else {
                return Err(invalid_backgammon_state(
                    action_id,
                    "missing borne-off count",
                ));
            };
            *borne_off = borne_off.saturating_add(1);
        }
    }

    if let Some(position) = public
        .dice
        .iter()
        .position(|candidate| *candidate == parsed.die)
    {
        public.dice.remove(position);
    }

    Ok(())
}

fn core_action_for_move(parsed: BackgammonMove) -> CoreAction {
    CoreAction {
        action_id: backgammon_action_id(parsed),
        display_label: backgammon_action_label(parsed),
        params: json!({
            "source": source_token(parsed.source),
            "dest": dest_token(parsed.dest),
            "die": parsed.die,
        }),
    }
}

fn cube_decision_action(action_id: &str, label: &str) -> CoreAction {
    CoreAction {
        action_id: action_id.to_string(),
        display_label: label.to_string(),
        params: json!({}),
    }
}

fn backgammon_action_id(parsed: BackgammonMove) -> String {
    format!(
        "{BACKGAMMON_MOVE_PREFIX}{}-{}",
        source_token(parsed.source),
        dest_token(parsed.dest)
    )
}

fn backgammon_action_label(parsed: BackgammonMove) -> String {
    format!(
        "move-{}-{}",
        source_token(parsed.source),
        dest_token(parsed.dest)
    )
}

fn source_token(source: BackgammonSource) -> String {
    match source {
        BackgammonSource::Bar => "bar".to_string(),
        BackgammonSource::Point(point) => point.to_string(),
    }
}

fn dest_token(dest: BackgammonDest) -> String {
    match dest {
        BackgammonDest::Point(point) => point.to_string(),
        BackgammonDest::Off => "off".to_string(),
    }
}

fn parse_backgammon_move(action_id: &str) -> Result<BackgammonMove, CoreGameError> {
    let Some(move_token) = action_id.strip_prefix(BACKGAMMON_MOVE_PREFIX) else {
        return Err(CoreGameError::UnknownAction {
            game: ResearchGame::Backgammon,
            action_id: action_id.to_string(),
        });
    };
    let Some((source, dest)) = move_token.split_once('-') else {
        return Err(CoreGameError::UnknownAction {
            game: ResearchGame::Backgammon,
            action_id: action_id.to_string(),
        });
    };
    let source = if source == "bar" {
        BackgammonSource::Bar
    } else {
        BackgammonSource::Point(parse_point(action_id, source)?)
    };
    let dest = if dest == "off" {
        BackgammonDest::Off
    } else {
        BackgammonDest::Point(parse_point(action_id, dest)?)
    };
    let die = move_die(source, dest).ok_or_else(|| CoreGameError::UnknownAction {
        game: ResearchGame::Backgammon,
        action_id: action_id.to_string(),
    })?;

    Ok(BackgammonMove { source, dest, die })
}

fn parse_point(action_id: &str, token: &str) -> Result<u8, CoreGameError> {
    let point = token
        .parse::<u8>()
        .map_err(|_| CoreGameError::UnknownAction {
            game: ResearchGame::Backgammon,
            action_id: action_id.to_string(),
        })?;
    if !(1..=24).contains(&point) {
        return Err(CoreGameError::UnknownAction {
            game: ResearchGame::Backgammon,
            action_id: action_id.to_string(),
        });
    }

    Ok(point)
}

fn move_die(source: BackgammonSource, dest: BackgammonDest) -> Option<u8> {
    match (source, dest) {
        (BackgammonSource::Bar, BackgammonDest::Point(dest)) => 25u8.checked_sub(dest),
        (BackgammonSource::Point(source), BackgammonDest::Point(dest)) => dest.checked_sub(source),
        (BackgammonSource::Point(source), BackgammonDest::Off) => 25u8.checked_sub(source),
        (BackgammonSource::Bar, BackgammonDest::Off) => None,
    }
}

fn remaining_pips(points: &[u8], bar_count: u8) -> u64 {
    let board_pips = points
        .iter()
        .enumerate()
        .fold(0_u64, |total, (index, count)| {
            let point_number = u64::try_from(index).unwrap_or_default().saturating_add(1);
            total.saturating_add(
                u64::from(*count).saturating_mul(25_u64.saturating_sub(point_number)),
            )
        });

    board_pips.saturating_add(u64::from(bar_count).saturating_mul(25))
}

fn pip_delta(opponent_remaining: u64, own_remaining: u64) -> i16 {
    let opponent = i64::try_from(opponent_remaining).unwrap_or(i64::MAX);
    let own = i64::try_from(own_remaining).unwrap_or(i64::MAX);
    let delta = opponent.saturating_sub(own);
    i16::try_from(delta).unwrap_or(if delta.is_negative() {
        i16::MIN
    } else {
        i16::MAX
    })
}

fn all_checkers_in_home(public: &BackgammonPublicState) -> bool {
    side_all_checkers_in_home(&public.own_points, public.bar.first().copied().unwrap_or(0))
}

fn side_all_checkers_in_home(points: &[u8], bar_count: u8) -> bool {
    if bar_count > 0 {
        return false;
    }
    points.iter().copied().enumerate().all(|(index, count)| {
        if count == 0 {
            return true;
        }
        let Ok(zero_based) = u8::try_from(index) else {
            return false;
        };
        let Some(point) = zero_based.checked_add(1) else {
            return false;
        };

        (19..=24).contains(&point)
    })
}

fn sides_still_have_contact(public: &BackgammonPublicState) -> bool {
    !(side_all_checkers_in_home(&public.own_points, public.bar.first().copied().unwrap_or(0))
        && side_all_checkers_in_home(
            &public.opponent_points,
            public.bar.get(1).copied().unwrap_or(0),
        ))
}

fn made_points_in_range(points: &[u8], start: u8, end: u8) -> u8 {
    let start_index = usize::from(start.saturating_sub(1));
    let end_index = usize::from(end.saturating_sub(1));
    usize_to_u8(
        points
            .iter()
            .enumerate()
            .filter(|(index, count)| *index >= start_index && *index <= end_index && **count >= 2)
            .count(),
    )
}

fn longest_prime(points: &[u8]) -> u8 {
    let mut best = 0_usize;
    let mut current = 0_usize;

    for count in points {
        if *count >= 2 {
            current = current.saturating_add(1);
            best = best.max(current);
        } else {
            current = 0;
        }
    }

    usize_to_u8(best)
}

fn point_mut(points: &mut [u8], point: u8) -> Option<&mut u8> {
    points.get_mut(point_index(point)?)
}

fn point_index(point: u8) -> Option<usize> {
    let zero_based = point.checked_sub(1)?;
    Some(usize::from(zero_based))
}

fn set_point(points: &mut [u8], point: u8, count: u8) -> Result<(), CoreGameError> {
    let Some(slot) = point_mut(points, point) else {
        return Err(invalid_backgammon_state(
            "backgammon.bootstrap",
            "point out of range",
        ));
    };
    *slot = count;

    Ok(())
}

fn usize_to_u8(value: usize) -> u8 {
    u8::try_from(value).unwrap_or(u8::MAX)
}

fn invalid_backgammon_state(action_id: &str, reason: &str) -> CoreGameError {
    CoreGameError::InvalidParams {
        action_id: action_id.to_string(),
        reason: reason.to_string(),
    }
}

fn backgammon_cube_decision_public() -> BackgammonPublicState {
    let mut own_points = vec![0; 24];
    set_point(&mut own_points, 20, 1)
        .unwrap_or_else(|error| panic!("point 20 should exist in bootstrap board: {error}"));
    set_point(&mut own_points, 21, 1)
        .unwrap_or_else(|error| panic!("point 21 should exist in bootstrap board: {error}"));
    set_point(&mut own_points, 22, 1)
        .unwrap_or_else(|error| panic!("point 22 should exist in bootstrap board: {error}"));
    set_point(&mut own_points, 23, 1)
        .unwrap_or_else(|error| panic!("point 23 should exist in bootstrap board: {error}"));
    let mut opponent_points = vec![0; 24];
    set_point(&mut opponent_points, 19, 3)
        .unwrap_or_else(|error| panic!("point 19 should exist in bootstrap board: {error}"));
    set_point(&mut opponent_points, 20, 2)
        .unwrap_or_else(|error| panic!("point 20 should exist in bootstrap board: {error}"));

    BackgammonPublicState {
        own_points,
        opponent_points,
        bar: [0, 0],
        borne_off: [11, 10],
        dice: vec![6, 5],
        cube_owner: None,
        cube_value: 1,
        cube_available: false,
        cube_offered_by_opponent: true,
    }
}

#[cfg(test)]
fn backgammon_checker_play_public() -> BackgammonPublicState {
    let mut own_points = vec![0; 24];
    set_point(&mut own_points, 8, 2)
        .unwrap_or_else(|error| panic!("point 8 should exist in bootstrap board: {error}"));
    set_point(&mut own_points, 19, 5)
        .unwrap_or_else(|error| panic!("point 19 should exist in bootstrap board: {error}"));
    BackgammonPublicState {
        own_points,
        opponent_points: vec![0; 24],
        bar: [1, 0],
        borne_off: [7, 6],
        dice: vec![6, 3],
        cube_owner: None,
        cube_value: 1,
        cube_available: false,
        cube_offered_by_opponent: false,
    }
}

fn rotate_perspective(public: &mut BackgammonPublicState) {
    std::mem::swap(&mut public.own_points, &mut public.opponent_points);
    public.bar.swap(0, 1);
    public.borne_off.swap(0, 1);
}

fn checker_turn_is_complete(public: &BackgammonPublicState) -> bool {
    public.dice.is_empty() || legal_backgammon_moves(public).is_empty()
}

fn side_has_borne_off_all(public: &BackgammonPublicState, side: usize) -> bool {
    public.borne_off.get(side).copied().unwrap_or_default() >= BACKGAMMON_CHECKERS_PER_SIDE
}

fn representative_roll(public: &BackgammonPublicState) -> Vec<u8> {
    let mut best_roll = vec![6, 5];
    let mut best_score = (0_u8, 0_u8, 0_u8, 0_u8);

    for die_a in (1..=6).rev() {
        for die_b in (1..=6).rev() {
            let mut candidate = public.clone();
            candidate.dice = vec![die_a, die_b];
            let moves = legal_backgammon_moves(&candidate);
            let off_moves = usize_to_u8(
                moves
                    .iter()
                    .filter(|move_candidate| matches!(move_candidate.dest, BackgammonDest::Off))
                    .count(),
            );
            let move_options = usize_to_u8(moves.len());
            let score = (off_moves, move_options, die_a.max(die_b), die_a.min(die_b));
            if score > best_score {
                best_score = score;
                best_roll = vec![die_a, die_b];
            }
        }
    }

    best_roll
}

fn effective_cube_value(public: &BackgammonPublicState) -> u8 {
    if public.cube_offered_by_opponent {
        public.cube_value.saturating_mul(2).max(2)
    } else {
        public.cube_value.max(1)
    }
}

fn opposing_actor(actor: u8) -> u8 {
    if actor == 0 { 1 } else { 0 }
}

fn payoff_for_loser(loser: u8, stake: i64) -> Vec<i64> {
    if loser == 0 {
        vec![stake.saturating_neg(), stake]
    } else {
        vec![stake, stake.saturating_neg()]
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::core::model::{apply_action, bootstrap_state};

    #[test]
    fn backgammon_bootstrap_state_has_legal_actions() {
        let state = backgammon_state();

        assert_eq!(state.game, ResearchGame::Backgammon);
        assert!(!state.legal_actions.is_empty());
        assert_eq!(state.phase, "cube-decision");
        assert!(
            state
                .legal_actions
                .iter()
                .any(|action| action.action_id == BACKGAMMON_TAKE_DOUBLE_ID)
        );
        assert!(
            state
                .legal_actions
                .iter()
                .any(|action| action.action_id == BACKGAMMON_DROP_DOUBLE_ID)
        );
    }

    #[test]
    fn backgammon_cube_phase_rejects_checker_move() {
        let state = backgammon_state();

        assert!(matches!(
            apply_action(&state, "backgammon.move.8-off", json!({})),
            Err(CoreGameError::UnknownAction { .. })
        ));
    }

    #[test]
    fn backgammon_bar_entry_before_other_moves() {
        let state = checker_play_state();

        assert!(matches!(
            apply_action(&state, "backgammon.move.8-14", json!({})),
            Err(CoreGameError::IllegalAction { reason, .. }) if reason.contains("bar entry")
        ));
    }

    #[test]
    fn backgammon_bearing_off_requires_home_board() {
        let mut public = checker_play_public();
        public.bar = [0, 0];
        let state = match state_from_public(public, Some(0)) {
            Ok(state) => state,
            Err(error) => panic!("state should build: {error}"),
        };

        assert!(matches!(
            apply_action(&state, "backgammon.move.8-off", json!({})),
            Err(CoreGameError::IllegalAction { reason, .. }) if reason.contains("home board")
        ));
    }

    #[test]
    fn backgammon_transition_is_deterministic() {
        let state = checker_play_state();
        let first = apply_action(&state, "backgammon.move.bar-19", json!({}));
        let second = apply_action(&state, "backgammon.move.bar-19", json!({}));

        assert_eq!(first, second);
    }

    #[test]
    fn backgammon_feature_view_tracks_bar_entry_progress() {
        let state = checker_play_state();
        let before = feature_view(&state)
            .unwrap_or_else(|error| panic!("backgammon feature view should decode: {error}"));
        let mut public = checker_play_public();
        public.bar = [0, 0];
        let outside_home = point_mut(&mut public.own_points, 8)
            .unwrap_or_else(|| panic!("point 8 should exist in backgammon board"));
        *outside_home = 0;
        let point = point_mut(&mut public.own_points, 19)
            .unwrap_or_else(|| panic!("point 19 should exist in backgammon board"));
        *point = point.saturating_add(3);
        let after_state = state_from_public(public, Some(0))
            .unwrap_or_else(|error| panic!("post-entry backgammon state should build: {error}"));
        let after = feature_view(&after_state).unwrap_or_else(|error| {
            panic!("backgammon feature view after move should decode: {error}")
        });

        assert_eq!(before.anchors, 2);
        assert_eq!(after.anchors, 1);
        assert_eq!(before.bar_count, 1);
        assert_eq!(after.bar_count, 0);
        assert!(!before.bearoff_ready);
        assert!(after.bearoff_ready);
        assert!(before.cube_centered);
        assert!(!before.cube_owned_by_actor);
        assert!(!before.facing_double);
        assert_eq!(before.blot_count, 0);
        assert_eq!(before.home_board_points, 1);
        assert_eq!(before.prime_length, 1);
        assert_eq!(after.blot_count, 0);
        assert_eq!(after.home_board_points, 1);
        assert_eq!(after.prime_length, 1);
        assert!(after.race_lead_pips > before.race_lead_pips);
    }

    #[test]
    fn backgammon_feature_view_tracks_prime_and_blots() {
        let mut own_points = vec![0; 24];
        set_point(&mut own_points, 6, 2)
            .unwrap_or_else(|error| panic!("point 6 should exist in custom board: {error}"));
        set_point(&mut own_points, 7, 2)
            .unwrap_or_else(|error| panic!("point 7 should exist in custom board: {error}"));
        set_point(&mut own_points, 8, 2)
            .unwrap_or_else(|error| panic!("point 8 should exist in custom board: {error}"));
        set_point(&mut own_points, 20, 2)
            .unwrap_or_else(|error| panic!("point 20 should exist in custom board: {error}"));
        set_point(&mut own_points, 22, 1)
            .unwrap_or_else(|error| panic!("point 22 should exist in custom board: {error}"));
        let public = BackgammonPublicState {
            own_points,
            opponent_points: vec![0; 24],
            bar: [0, 0],
            borne_off: [6, 4],
            dice: vec![4, 3],
            cube_owner: None,
            cube_value: 2,
            cube_available: true,
            cube_offered_by_opponent: false,
        };
        let state = state_from_public(public, Some(0))
            .unwrap_or_else(|error| panic!("prime/blot backgammon state should build: {error}"));
        let view = feature_view(&state)
            .unwrap_or_else(|error| panic!("prime/blot feature view should decode: {error}"));

        assert_eq!(view.blot_count, 1);
        assert_eq!(view.home_board_points, 1);
        assert_eq!(view.prime_length, 3);
    }

    #[test]
    fn backgammon_feature_view_tracks_cube_status() {
        let mut public = checker_play_public();
        public.cube_owner = Some(1);
        public.cube_value = 1;
        public.cube_offered_by_opponent = true;
        let state = state_from_public(public, Some(0))
            .unwrap_or_else(|error| panic!("cube-status backgammon state should build: {error}"));
        let view = feature_view(&state)
            .unwrap_or_else(|error| panic!("cube-status feature view should decode: {error}"));

        assert_eq!(view.cube_efficiency, 2);
        assert!(!view.cube_centered);
        assert!(!view.cube_owned_by_actor);
        assert!(view.facing_double);
        assert_eq!(view.move_options, 2);
        assert_eq!(view.off_moves, 0);
    }

    #[test]
    fn taking_double_hands_turn_back_to_doubler() {
        let state = backgammon_state();
        let transition = apply_action(&state, BACKGAMMON_TAKE_DOUBLE_ID, json!({}))
            .unwrap_or_else(|error| panic!("take-double should apply: {error}"));

        assert_eq!(transition.after.phase, "checker-play");
        assert!(!transition.after.terminal);
        assert_eq!(transition.after.actor, Some(1));
        assert!(
            transition
                .after
                .legal_actions
                .iter()
                .any(|action| action.action_id.ends_with("-off"))
        );
        let view = feature_view(&transition.after)
            .unwrap_or_else(|error| panic!("post-take feature view should decode: {error}"));
        assert_eq!(view.cube_efficiency, 2);
        assert!(!view.cube_centered);
        assert!(!view.cube_owned_by_actor);
        assert!(!view.facing_double);
        assert!(view.bearoff_ready);
        assert!(!view.has_contact);
        assert_eq!(view.move_options, 4);
        assert_eq!(view.off_moves, 3);
        assert_eq!(view.blot_count, 0);
        assert_eq!(view.home_board_points, 2);
        assert_eq!(view.prime_length, 2);
        assert!(view.race_lead_pips < 0);
    }

    #[test]
    fn consuming_last_die_rotates_turn_with_representative_roll() {
        let mut own_points = vec![0; 24];
        set_point(&mut own_points, 24, 2)
            .unwrap_or_else(|error| panic!("point 24 should exist in custom board: {error}"));
        let mut opponent_points = vec![0; 24];
        set_point(&mut opponent_points, 20, 2)
            .unwrap_or_else(|error| panic!("point 20 should exist in custom board: {error}"));
        let public = BackgammonPublicState {
            own_points,
            opponent_points,
            bar: [0, 0],
            borne_off: [13, 13],
            dice: vec![1],
            cube_owner: None,
            cube_value: 2,
            cube_available: true,
            cube_offered_by_opponent: false,
        };
        let state = state_from_public(public, Some(0))
            .unwrap_or_else(|error| panic!("custom checker-play state should build: {error}"));
        let transition = apply_action(&state, "backgammon.move.24-off", json!({}))
            .unwrap_or_else(|error| panic!("bearing off final die should apply: {error}"));

        assert_eq!(transition.after.phase, "checker-play");
        assert_eq!(transition.after.actor, Some(1));
        let after_public: BackgammonPublicState =
            serde_json::from_value(transition.after.public_state)
                .unwrap_or_else(|error| panic!("rolled-over state should decode: {error}"));
        assert_eq!(after_public.dice, vec![6, 6]);
        assert!(
            transition
                .after
                .legal_actions
                .iter()
                .any(|action| action.action_id.ends_with("-off"))
        );
    }

    #[test]
    fn dropping_double_ends_the_game() {
        let state = backgammon_state();
        let transition = apply_action(&state, BACKGAMMON_DROP_DOUBLE_ID, json!({}))
            .unwrap_or_else(|error| panic!("drop-double should apply: {error}"));

        assert!(transition.after.terminal);
        assert_eq!(transition.after.payoff, Some(vec![-1, 1]));
        assert!(transition.after.legal_actions.is_empty());
    }

    fn backgammon_state() -> CoreGameState {
        match bootstrap_state(ResearchGame::Backgammon) {
            Ok(state) => state,
            Err(error) => panic!("backgammon bootstrap should succeed: {error}"),
        }
    }

    fn checker_play_state() -> CoreGameState {
        state_from_public(backgammon_checker_play_public(), Some(0))
            .unwrap_or_else(|error| panic!("checker-play backgammon state should build: {error}"))
    }

    fn checker_play_public() -> BackgammonPublicState {
        backgammon_checker_play_public()
    }
}

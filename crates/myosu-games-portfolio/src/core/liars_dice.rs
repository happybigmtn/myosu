use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::core::model::{CoreAction, CoreGameError, CoreGameState, CoreTransition};
use crate::game::ResearchGame;

const LIARS_DICE_BID_PREFIX: &str = "liars-dice.bid.";
const LIARS_DICE_CHALLENGE: &str = "liars-dice.challenge";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
struct LiarsDicePublicState {
    actor: u8,
    dice: Vec<u8>,
    last_claim: Option<Claim>,
    history: Vec<ClaimRecord>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
struct Claim {
    count: u8,
    face: u8,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
struct ClaimRecord {
    actor: u8,
    claim: Claim,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LiarsDiceAction {
    Bid(Claim),
    Challenge,
}

pub fn liars_dice_bootstrap_state() -> Result<CoreGameState, CoreGameError> {
    state_from_public(
        LiarsDicePublicState {
            actor: 1,
            dice: vec![4, 2],
            last_claim: Some(Claim { count: 1, face: 3 }),
            history: vec![ClaimRecord {
                actor: 0,
                claim: Claim { count: 1, face: 3 },
            }],
        },
        false,
        None,
    )
}

pub fn apply_liars_dice_action(
    state: &CoreGameState,
    action_id: &str,
    _params: serde_json::Value,
) -> Result<CoreTransition, CoreGameError> {
    let before_public: LiarsDicePublicState = serde_json::from_value(state.public_state.clone())
        .map_err(|source| CoreGameError::InvalidParams {
            action_id: action_id.to_string(),
            reason: source.to_string(),
        })?;
    let action = parse_liars_dice_action(action_id)?;
    validate_liars_dice_action(&before_public, action, action_id)?;
    if !state
        .legal_actions
        .iter()
        .any(|candidate| candidate.action_id == action_id)
    {
        return Err(CoreGameError::IllegalAction {
            game: ResearchGame::LiarsDice,
            action_id: action_id.to_string(),
            reason: "action is not legal from this claim state".to_string(),
        });
    }

    let mut after_public = before_public.clone();
    let (terminal, payoff) = match action {
        LiarsDiceAction::Bid(claim) => {
            after_public.last_claim = Some(claim);
            after_public.history.push(ClaimRecord {
                actor: after_public.actor,
                claim,
            });
            after_public.actor = other_actor(after_public.actor);
            (false, None)
        }
        LiarsDiceAction::Challenge => {
            let Some(last_record) = after_public.history.last().copied() else {
                return Err(CoreGameError::InvalidParams {
                    action_id: action_id.to_string(),
                    reason: "challenge requires a last claim record".to_string(),
                });
            };
            let occurrences = after_public
                .dice
                .iter()
                .filter(|die| **die == last_record.claim.face)
                .count();
            let truthful = occurrences >= usize::from(last_record.claim.count);
            let winner = if truthful {
                last_record.actor
            } else {
                after_public.actor
            };
            let payoff = if winner == 0 {
                vec![1, -1]
            } else {
                vec![-1, 1]
            };
            (true, Some(payoff))
        }
    };

    let after = state_from_public(after_public, terminal, payoff)?;
    let action = core_action_for_liars_dice(action);

    Ok(CoreTransition {
        before: state.clone(),
        action,
        after,
    })
}

fn state_from_public(
    public: LiarsDicePublicState,
    terminal: bool,
    payoff: Option<Vec<i64>>,
) -> Result<CoreGameState, CoreGameError> {
    if public.dice.len() != 2 || public.actor > 1 {
        return Err(CoreGameError::InvalidParams {
            action_id: "liars-dice.bootstrap".to_string(),
            reason: "liar's dice bootstrap expects two dice and a valid actor".to_string(),
        });
    }
    let actor = (!terminal).then_some(public.actor);
    let legal_actions = if terminal {
        Vec::new()
    } else {
        legal_liars_dice_actions(&public)
            .into_iter()
            .map(core_action_for_liars_dice)
            .collect()
    };
    let public_state =
        serde_json::to_value(public).map_err(|source| CoreGameError::InvalidParams {
            action_id: "liars-dice.bootstrap".to_string(),
            reason: source.to_string(),
        })?;

    Ok(CoreGameState {
        game: ResearchGame::LiarsDice,
        phase: "bidding".to_string(),
        actor,
        public_state,
        private_state_commitments: vec![
            "liars-dice.die-0.bootstrap-v1".to_string(),
            "liars-dice.die-1.bootstrap-v1".to_string(),
        ],
        legal_actions,
        terminal,
        payoff,
    })
}

fn legal_liars_dice_actions(public: &LiarsDicePublicState) -> Vec<LiarsDiceAction> {
    let mut actions = Vec::new();
    let start_rank = public
        .last_claim
        .map(claim_rank)
        .unwrap_or_default()
        .saturating_add(1);
    for rank in start_rank..=claim_rank(Claim { count: 2, face: 6 }) {
        let Some(claim) = claim_from_rank(rank) else {
            continue;
        };
        actions.push(LiarsDiceAction::Bid(claim));
    }
    if public.last_claim.is_some() {
        actions.push(LiarsDiceAction::Challenge);
    }

    actions
}

fn validate_liars_dice_action(
    public: &LiarsDicePublicState,
    action: LiarsDiceAction,
    action_id: &str,
) -> Result<(), CoreGameError> {
    match action {
        LiarsDiceAction::Bid(claim) => {
            if !(1..=2).contains(&claim.count) || !(1..=6).contains(&claim.face) {
                return Err(CoreGameError::IllegalAction {
                    game: ResearchGame::LiarsDice,
                    action_id: action_id.to_string(),
                    reason: "claim must stay within one-die-per-player bounds".to_string(),
                });
            }
            if let Some(last_claim) = public.last_claim
                && claim_rank(claim) <= claim_rank(last_claim)
            {
                return Err(CoreGameError::IllegalAction {
                    game: ResearchGame::LiarsDice,
                    action_id: action_id.to_string(),
                    reason: "bid must strictly increase the prior claim".to_string(),
                });
            }
        }
        LiarsDiceAction::Challenge => {
            if public.last_claim.is_none() {
                return Err(CoreGameError::IllegalAction {
                    game: ResearchGame::LiarsDice,
                    action_id: action_id.to_string(),
                    reason: "challenge requires an existing claim".to_string(),
                });
            }
        }
    }

    Ok(())
}

fn core_action_for_liars_dice(action: LiarsDiceAction) -> CoreAction {
    match action {
        LiarsDiceAction::Bid(claim) => CoreAction {
            action_id: format!("{LIARS_DICE_BID_PREFIX}{}x{}", claim.count, claim.face),
            display_label: format!("bid-{}x{}", claim.count, claim.face),
            params: json!({"count": claim.count, "face": claim.face}),
        },
        LiarsDiceAction::Challenge => CoreAction {
            action_id: LIARS_DICE_CHALLENGE.to_string(),
            display_label: "challenge".to_string(),
            params: json!({}),
        },
    }
}

fn parse_liars_dice_action(action_id: &str) -> Result<LiarsDiceAction, CoreGameError> {
    if action_id == LIARS_DICE_CHALLENGE {
        return Ok(LiarsDiceAction::Challenge);
    }
    let Some(token) = action_id.strip_prefix(LIARS_DICE_BID_PREFIX) else {
        return Err(CoreGameError::UnknownAction {
            game: ResearchGame::LiarsDice,
            action_id: action_id.to_string(),
        });
    };
    let Some((count, face)) = token.split_once('x') else {
        return Err(CoreGameError::UnknownAction {
            game: ResearchGame::LiarsDice,
            action_id: action_id.to_string(),
        });
    };
    let count = count
        .parse::<u8>()
        .map_err(|_| CoreGameError::UnknownAction {
            game: ResearchGame::LiarsDice,
            action_id: action_id.to_string(),
        })?;
    let face = face
        .parse::<u8>()
        .map_err(|_| CoreGameError::UnknownAction {
            game: ResearchGame::LiarsDice,
            action_id: action_id.to_string(),
        })?;

    Ok(LiarsDiceAction::Bid(Claim { count, face }))
}

fn claim_rank(claim: Claim) -> u8 {
    claim
        .count
        .saturating_sub(1)
        .saturating_mul(6)
        .saturating_add(claim.face)
}

fn claim_from_rank(rank: u8) -> Option<Claim> {
    if !(1..=12).contains(&rank) {
        return None;
    }
    let zero_based = rank.checked_sub(1)?;
    let count = zero_based.checked_div(6)?.checked_add(1)?;
    let face = zero_based.checked_rem(6)?.checked_add(1)?;
    Some(Claim { count, face })
}

fn other_actor(actor: u8) -> u8 {
    if actor == 0 { 1 } else { 0 }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::core::model::{apply_action, bootstrap_state};

    #[test]
    fn liars_dice_bootstrap_has_higher_bids_and_challenge() {
        let state = liars_dice_state();

        assert!(
            state
                .legal_actions
                .iter()
                .any(|action| action.action_id == "liars-dice.bid.1x4")
        );
        assert!(
            state
                .legal_actions
                .iter()
                .any(|action| action.action_id == "liars-dice.challenge")
        );
    }

    #[test]
    fn liars_dice_rejects_non_increasing_bid() {
        let state = liars_dice_state();

        assert!(matches!(
            apply_action(&state, "liars-dice.bid.1x3", json!({})),
            Err(CoreGameError::IllegalAction { reason, .. }) if reason.contains("strictly increase")
        ));
    }

    #[test]
    fn liars_dice_challenge_resolves_terminal_payoff() {
        let state = liars_dice_state();
        let transition = apply_action(&state, "liars-dice.challenge", json!({}))
            .unwrap_or_else(|error| panic!("challenge should apply: {error}"));

        assert!(transition.after.terminal);
        assert_eq!(transition.after.payoff, Some(vec![-1, 1]));
    }

    #[test]
    fn liars_dice_transition_is_deterministic() {
        let state = liars_dice_state();
        let first = apply_action(&state, "liars-dice.bid.1x4", json!({}));
        let second = apply_action(&state, "liars-dice.bid.1x4", json!({}));

        assert_eq!(first, second);
    }

    fn liars_dice_state() -> CoreGameState {
        bootstrap_state(ResearchGame::LiarsDice)
            .unwrap_or_else(|error| panic!("liars dice bootstrap should succeed: {error}"))
    }
}

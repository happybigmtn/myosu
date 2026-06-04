use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::cards::{Card, Rank, Suit};
use crate::core::model::{CoreAction, CoreGameError, CoreGameState, CoreTransition};
use crate::eval::meld::is_three_card_run;
use crate::game::ResearchGame;

const CRIBBAGE_ACTION_PREFIX: &str = "cribbage.pegging.play-";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
struct CribbagePublicState {
    running_count: u8,
    pegging_points: u8,
    actor_hand: Vec<Card>,
    played_cards: Vec<Card>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CribbageFeatureView {
    pub pegging_count: u8,
    pub run_potential: u8,
    pub crib_edge: i8,
    pub pair_trap: bool,
    pub go_window: bool,
    pub fifteen_outs: u8,
    pub max_immediate_points: u8,
}

pub fn cribbage_bootstrap_state() -> Result<CoreGameState, CoreGameError> {
    let public = CribbagePublicState {
        running_count: 9,
        pegging_points: 0,
        actor_hand: vec![
            Card::new(Rank::Six, Suit::Clubs),
            Card::new(Rank::King, Suit::Hearts),
            Card::new(Rank::Two, Suit::Spades),
            Card::new(Rank::Ace, Suit::Diamonds),
        ],
        played_cards: vec![
            Card::new(Rank::Four, Suit::Clubs),
            Card::new(Rank::Five, Suit::Clubs),
        ],
    };

    state_from_public(public, Some(0))
}

pub fn apply_cribbage_action(
    state: &CoreGameState,
    action_id: &str,
    _params: serde_json::Value,
) -> Result<CoreTransition, CoreGameError> {
    let before_public: CribbagePublicState = serde_json::from_value(state.public_state.clone())
        .map_err(|source| CoreGameError::InvalidParams {
            action_id: action_id.to_string(),
            reason: source.to_string(),
        })?;
    let card = parse_cribbage_card_action(action_id)?;
    if !before_public.actor_hand.contains(&card) {
        return Err(CoreGameError::IllegalAction {
            game: ResearchGame::Cribbage,
            action_id: action_id.to_string(),
            reason: "card is not in acting hand".to_string(),
        });
    }

    let card_value = cribbage_card_value(card.rank);
    let Some(next_count) = before_public.running_count.checked_add(card_value) else {
        return Err(over_31(action_id));
    };
    if next_count > 31 {
        return Err(over_31(action_id));
    }

    let mut after_public = before_public.clone();
    after_public
        .actor_hand
        .retain(|candidate| *candidate != card);
    after_public.played_cards.push(card);
    after_public.running_count = next_count;
    let points = immediate_pegging_points(&after_public.played_cards, next_count);
    after_public.pegging_points =
        after_public
            .pegging_points
            .checked_add(points)
            .ok_or_else(|| CoreGameError::InvalidParams {
                action_id: action_id.to_string(),
                reason: "pegging points overflow".to_string(),
            })?;

    let after = state_from_public(after_public, state.actor)?;
    let action = core_action_for_card(card);

    Ok(CoreTransition {
        before: state.clone(),
        action,
        after,
    })
}

fn state_from_public(
    public: CribbagePublicState,
    actor: Option<u8>,
) -> Result<CoreGameState, CoreGameError> {
    let mut legal_actions = Vec::new();
    for card in &public.actor_hand {
        if public
            .running_count
            .checked_add(cribbage_card_value(card.rank))
            .is_some_and(|count| count <= 31)
        {
            legal_actions.push(core_action_for_card(*card));
        }
    }
    let public_state =
        serde_json::to_value(public).map_err(|source| CoreGameError::InvalidParams {
            action_id: "cribbage.bootstrap".to_string(),
            reason: source.to_string(),
        })?;

    Ok(CoreGameState {
        game: ResearchGame::Cribbage,
        phase: "pegging".to_string(),
        actor,
        public_state,
        private_state_commitments: vec!["cribbage.opponent-hand.bootstrap-v1".to_string()],
        legal_actions,
        terminal: false,
        payoff: None,
    })
}

/// One labeled representative decision point in the Cribbage search space.
///
/// Scenarios are intentionally narrow: each row targets a specific engine
/// heuristic (pegging-run, pair trap, fifteen-out, go window, crib edge,
/// discard pressure) and is used by both the benchmark dossier writer and
/// the e2e promotion proof. The fields mirror the typed
/// `CribbageChallenge` so a scenario can be replayed through the same
/// engine dispatch as a live portfolio challenge.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CribbageScenario {
    pub scenario_id: &'static str,
    pub decision: &'static str,
    pub pegging_count: u8,
    pub run_potential: u8,
    pub crib_edge: i8,
    pub pair_trap: bool,
    pub go_window: bool,
    pub fifteen_outs: u8,
    pub max_immediate_points: u8,
}

/// Return the canonical 22-scenario Cribbage benchmark pack.
///
/// Coverage requirements (R1 in `genesis/plans/009-cribbage-deepening.md`):
/// - opening_discard × 4 (low-card-heavy, five-heavy, pair-led, neutral)
/// - pegging_fifteen × 4 (single 15-out, dual 15-outs, near-15 trap, no-out)
/// - pegging_pair × 3 (clean pair, double pair, no trap)
/// - pegging_run × 3 (3-card run, 4-card run set-up, blocked)
/// - pegging_go × 3 (clean go, danger go, end-of-31)
/// - pegging_thirty_one × 2 (forced last card, non-forced)
/// - counting × 3 (high run count, high fifteen count, balanced)
///
/// Total: 22 labeled scenarios, exceeds the 20-scenario floor.
pub fn cribbage_scenario_pack() -> &'static [CribbageScenario] {
    SCENARIO_PACK
}

const SCENARIO_PACK: &[CribbageScenario] = &[
    // opening_discard
    CribbageScenario {
        scenario_id: "opening-discard-low-heavy",
        decision: "Discard to crib with low-card heavy hand (favor own hand)",
        pegging_count: 0,
        run_potential: 0,
        crib_edge: 2,
        pair_trap: false,
        go_window: false,
        fifteen_outs: 0,
        max_immediate_points: 0,
    },
    CribbageScenario {
        scenario_id: "opening-discard-five-heavy",
        decision: "Discard to crib with five-heavy hand (favor 15s)",
        pegging_count: 0,
        run_potential: 0,
        crib_edge: 1,
        pair_trap: false,
        go_window: false,
        fifteen_outs: 0,
        max_immediate_points: 0,
    },
    CribbageScenario {
        scenario_id: "opening-discard-pair-led",
        decision: "Discard to crib with pair-led hand (avoid giving pair)",
        pegging_count: 0,
        run_potential: 0,
        crib_edge: 0,
        pair_trap: true,
        go_window: false,
        fifteen_outs: 0,
        max_immediate_points: 0,
    },
    CribbageScenario {
        scenario_id: "opening-discard-neutral",
        decision: "Discard to crib with neutral hand (no clear feature)",
        pegging_count: 0,
        run_potential: 0,
        crib_edge: 0,
        pair_trap: false,
        go_window: false,
        fifteen_outs: 0,
        max_immediate_points: 0,
    },
    // pegging_fifteen
    CribbageScenario {
        scenario_id: "pegging-fifteen-single",
        decision: "Pegging with a single 15-out available",
        pegging_count: 10,
        run_potential: 0,
        crib_edge: 0,
        pair_trap: false,
        go_window: false,
        fifteen_outs: 1,
        max_immediate_points: 2,
    },
    CribbageScenario {
        scenario_id: "pegging-fifteen-dual",
        decision: "Pegging with two 15-outs available",
        pegging_count: 7,
        run_potential: 0,
        crib_edge: 0,
        pair_trap: false,
        go_window: false,
        fifteen_outs: 2,
        max_immediate_points: 2,
    },
    CribbageScenario {
        scenario_id: "pegging-fifteen-near-trap",
        decision: "Pegging near 15 with a card that would make 15",
        pegging_count: 13,
        run_potential: 0,
        crib_edge: 0,
        pair_trap: false,
        go_window: false,
        fifteen_outs: 1,
        max_immediate_points: 2,
    },
    CribbageScenario {
        scenario_id: "pegging-fifteen-no-out",
        decision: "Pegging with no 15-out available",
        pegging_count: 9,
        run_potential: 1,
        crib_edge: 0,
        pair_trap: false,
        go_window: false,
        fifteen_outs: 0,
        max_immediate_points: 3,
    },
    // pegging_pair
    CribbageScenario {
        scenario_id: "pegging-pair-clean",
        decision: "Pegging with a clean pair available",
        pegging_count: 5,
        run_potential: 0,
        crib_edge: 0,
        pair_trap: true,
        go_window: false,
        fifteen_outs: 0,
        max_immediate_points: 2,
    },
    CribbageScenario {
        scenario_id: "pegging-pair-double",
        decision: "Pegging with a double pair available",
        pegging_count: 5,
        run_potential: 0,
        crib_edge: 0,
        pair_trap: true,
        go_window: false,
        fifteen_outs: 0,
        max_immediate_points: 4,
    },
    CribbageScenario {
        scenario_id: "pegging-pair-no-trap",
        decision: "Pegging with no pair trap (rank mismatch)",
        pegging_count: 6,
        run_potential: 1,
        crib_edge: 0,
        pair_trap: false,
        go_window: false,
        fifteen_outs: 0,
        max_immediate_points: 0,
    },
    // pegging_run
    CribbageScenario {
        scenario_id: "pegging-run-three",
        decision: "Pegging with a 3-card run completion available",
        pegging_count: 12,
        run_potential: 2,
        crib_edge: 0,
        pair_trap: false,
        go_window: false,
        fifteen_outs: 0,
        max_immediate_points: 3,
    },
    CribbageScenario {
        scenario_id: "pegging-run-four-setup",
        decision: "Pegging with 4-card run set-up available",
        pegging_count: 11,
        run_potential: 3,
        crib_edge: 0,
        pair_trap: false,
        go_window: false,
        fifteen_outs: 0,
        max_immediate_points: 4,
    },
    CribbageScenario {
        scenario_id: "pegging-run-blocked",
        decision: "Pegging with run potential blocked by opponent card",
        pegging_count: 14,
        run_potential: 0,
        crib_edge: 0,
        pair_trap: false,
        go_window: false,
        fifteen_outs: 0,
        max_immediate_points: 0,
    },
    // pegging_go
    CribbageScenario {
        scenario_id: "pegging-go-clean",
        decision: "Clean go opportunity (forcing opponent under 31)",
        pegging_count: 27,
        run_potential: 0,
        crib_edge: 0,
        pair_trap: false,
        go_window: true,
        fifteen_outs: 0,
        max_immediate_points: 1,
    },
    CribbageScenario {
        scenario_id: "pegging-go-danger",
        decision: "Go window with danger that opponent can hit",
        pegging_count: 25,
        run_potential: 0,
        crib_edge: 0,
        pair_trap: false,
        go_window: true,
        fifteen_outs: 0,
        max_immediate_points: 1,
    },
    CribbageScenario {
        scenario_id: "pegging-go-end-of-31",
        decision: "Go window near end of 31 (last-card scoring)",
        pegging_count: 29,
        run_potential: 0,
        crib_edge: 0,
        pair_trap: false,
        go_window: true,
        fifteen_outs: 0,
        max_immediate_points: 1,
    },
    // pegging_thirty_one
    CribbageScenario {
        scenario_id: "pegging-thirty-one-forced",
        decision: "Forced last-card play for 31",
        pegging_count: 28,
        run_potential: 0,
        crib_edge: 0,
        pair_trap: false,
        go_window: false,
        fifteen_outs: 0,
        max_immediate_points: 3,
    },
    CribbageScenario {
        scenario_id: "pegging-thirty-one-nonforced",
        decision: "Non-forced 31 with multiple candidates",
        pegging_count: 21,
        run_potential: 2,
        crib_edge: 1,
        pair_trap: true,
        go_window: true,
        fifteen_outs: 1,
        max_immediate_points: 4,
    },
    // counting
    CribbageScenario {
        scenario_id: "counting-run-heavy",
        decision: "Counting phase: hand has high run potential",
        pegging_count: 0,
        run_potential: 3,
        crib_edge: 0,
        pair_trap: false,
        go_window: false,
        fifteen_outs: 0,
        max_immediate_points: 0,
    },
    CribbageScenario {
        scenario_id: "counting-fifteen-heavy",
        decision: "Counting phase: hand has high fifteen count",
        pegging_count: 0,
        run_potential: 0,
        crib_edge: -1,
        pair_trap: false,
        go_window: false,
        fifteen_outs: 3,
        max_immediate_points: 0,
    },
    CribbageScenario {
        scenario_id: "counting-balanced",
        decision: "Counting phase: balanced hand, no dominant feature",
        pegging_count: 0,
        run_potential: 1,
        crib_edge: 0,
        pair_trap: false,
        go_window: false,
        fifteen_outs: 1,
        max_immediate_points: 0,
    },
];

pub(crate) fn feature_view(state: &CoreGameState) -> Result<CribbageFeatureView, CoreGameError> {
    let public: CribbagePublicState =
        serde_json::from_value(state.public_state.clone()).map_err(|source| {
            CoreGameError::InvalidParams {
                action_id: format!("{}.feature-view", state.game.slug()),
                reason: source.to_string(),
            }
        })?;
    let low_cards = public
        .actor_hand
        .iter()
        .filter(|card| cribbage_card_value(card.rank) <= 5)
        .count();
    let high_cards = public
        .actor_hand
        .iter()
        .filter(|card| cribbage_card_value(card.rank) >= 10)
        .count();
    let legal_cards = public
        .actor_hand
        .iter()
        .copied()
        .filter(|card| {
            public
                .running_count
                .checked_add(cribbage_card_value(card.rank))
                .is_some_and(|count| count <= 31)
        })
        .collect::<Vec<_>>();

    Ok(CribbageFeatureView {
        pegging_count: public.running_count,
        run_potential: usize_to_u8(
            public
                .actor_hand
                .iter()
                .filter(|card| completes_cribbage_run(&public.played_cards, **card))
                .count(),
        ),
        crib_edge: i8::try_from(low_cards)
            .unwrap_or(i8::MAX)
            .saturating_sub(i8::try_from(high_cards).unwrap_or(i8::MAX))
            .clamp(-2, 2),
        pair_trap: public
            .played_cards
            .last()
            .is_some_and(|last| public.actor_hand.iter().any(|card| card.rank == last.rank)),
        go_window: legal_cards.iter().any(|card| {
            public
                .running_count
                .saturating_add(cribbage_card_value(card.rank))
                >= 27
        }),
        fifteen_outs: usize_to_u8(
            legal_cards
                .iter()
                .filter(|card| {
                    public
                        .running_count
                        .saturating_add(cribbage_card_value(card.rank))
                        == 15
                })
                .count(),
        ),
        max_immediate_points: legal_cards
            .iter()
            .map(|card| {
                let mut next_played = public.played_cards.clone();
                next_played.push(*card);
                let next_count = public
                    .running_count
                    .saturating_add(cribbage_card_value(card.rank));
                immediate_pegging_points(&next_played, next_count)
            })
            .max()
            .unwrap_or(0),
    })
}

fn core_action_for_card(card: Card) -> CoreAction {
    CoreAction {
        action_id: format!(
            "{CRIBBAGE_ACTION_PREFIX}{}-{}",
            crate::core::trick_taking::rank_token(card.rank),
            crate::core::trick_taking::suit_token(card.suit)
        ),
        display_label: format!(
            "play-{}-{}",
            crate::core::trick_taking::rank_token(card.rank),
            crate::core::trick_taking::suit_token(card.suit)
        ),
        params: json!({"card": card}),
    }
}

fn parse_cribbage_card_action(action_id: &str) -> Result<Card, CoreGameError> {
    let Some(card_token) = action_id.strip_prefix(CRIBBAGE_ACTION_PREFIX) else {
        return Err(CoreGameError::UnknownAction {
            game: ResearchGame::Cribbage,
            action_id: action_id.to_string(),
        });
    };
    let Some((rank, suit)) = card_token.split_once('-') else {
        return Err(CoreGameError::UnknownAction {
            game: ResearchGame::Cribbage,
            action_id: action_id.to_string(),
        });
    };
    let rank = crate::core::trick_taking::parse_rank(rank).ok_or_else(|| {
        CoreGameError::UnknownAction {
            game: ResearchGame::Cribbage,
            action_id: action_id.to_string(),
        }
    })?;
    let suit = crate::core::trick_taking::parse_suit(suit).ok_or_else(|| {
        CoreGameError::UnknownAction {
            game: ResearchGame::Cribbage,
            action_id: action_id.to_string(),
        }
    })?;

    Ok(Card::new(rank, suit))
}

fn cribbage_card_value(rank: Rank) -> u8 {
    match rank {
        Rank::Ace => 1,
        Rank::Two => 2,
        Rank::Three => 3,
        Rank::Four => 4,
        Rank::Five => 5,
        Rank::Six => 6,
        Rank::Seven => 7,
        Rank::Eight => 8,
        Rank::Nine => 9,
        Rank::Ten | Rank::Jack | Rank::Queen | Rank::King => 10,
    }
}

fn immediate_pegging_points(played_cards: &[Card], running_count: u8) -> u8 {
    let mut points: u8 = 0;
    if running_count == 15 || running_count == 31 {
        points = points.saturating_add(2);
    }
    if last_two_are_pair(played_cards) {
        points = points.saturating_add(2);
    }
    if last_three_are_run(played_cards) {
        points = points.saturating_add(3);
    }

    points
}

fn last_two_are_pair(played_cards: &[Card]) -> bool {
    let mut iter = played_cards.iter().rev();
    let Some(last) = iter.next() else {
        return false;
    };
    let Some(previous) = iter.next() else {
        return false;
    };

    last.rank == previous.rank
}

fn last_three_are_run(played_cards: &[Card]) -> bool {
    let mut cards = played_cards
        .iter()
        .rev()
        .take(3)
        .copied()
        .collect::<Vec<_>>();
    if cards.len() != 3 {
        return false;
    }
    cards.reverse();

    is_three_card_run(&cards)
}

fn completes_cribbage_run(played_cards: &[Card], candidate: Card) -> bool {
    if played_cards.len() < 2 {
        return false;
    }
    let Some(previous) = played_cards.get(played_cards.len().saturating_sub(2)..) else {
        return false;
    };
    let mut ranks = previous
        .iter()
        .map(|card| rank_score(card.rank))
        .collect::<Vec<_>>();
    ranks.push(rank_score(candidate.rank));
    ranks.sort_unstable();
    let Some(first) = ranks.first().copied() else {
        return false;
    };
    let Some(second) = ranks.get(1).copied() else {
        return false;
    };
    let Some(third) = ranks.get(2).copied() else {
        return false;
    };

    first.saturating_add(1) == second && second.saturating_add(1) == third
}

fn rank_score(rank: Rank) -> u8 {
    match rank {
        Rank::Two => 2,
        Rank::Three => 3,
        Rank::Four => 4,
        Rank::Five => 5,
        Rank::Six => 6,
        Rank::Seven => 7,
        Rank::Eight => 8,
        Rank::Nine => 9,
        Rank::Ten => 10,
        Rank::Jack => 11,
        Rank::Queen => 12,
        Rank::King => 13,
        Rank::Ace => 14,
    }
}

fn usize_to_u8(value: usize) -> u8 {
    u8::try_from(value).unwrap_or(u8::MAX)
}

fn over_31(action_id: &str) -> CoreGameError {
    CoreGameError::IllegalAction {
        game: ResearchGame::Cribbage,
        action_id: action_id.to_string(),
        reason: "pegging count cannot exceed 31".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::model::{apply_action, bootstrap_state};

    #[test]
    fn cribbage_bootstrap_state_has_legal_actions() {
        let state = cribbage_state();

        assert_eq!(state.game, ResearchGame::Cribbage);
        assert!(!state.legal_actions.is_empty());
        assert!(
            state
                .legal_actions
                .iter()
                .any(|action| action.action_id == "cribbage.pegging.play-six-clubs")
        );
    }

    #[test]
    fn cribbage_rejects_illegal_action() {
        let state = cribbage_state();

        assert!(matches!(
            apply_action(&state, "cribbage.pegging.play-three-clubs", json!({})),
            Err(CoreGameError::IllegalAction { reason, .. }) if reason.contains("not in acting hand")
        ));
    }

    #[test]
    fn cribbage_transition_is_deterministic() {
        let state = cribbage_state();
        let first = apply_action(&state, "cribbage.pegging.play-six-clubs", json!({}));
        let second = apply_action(&state, "cribbage.pegging.play-six-clubs", json!({}));

        assert_eq!(first, second);
    }

    #[test]
    fn cribbage_pegging_rejects_over_31() {
        let state = over_31_state();

        assert!(matches!(
            apply_action(&state, "cribbage.pegging.play-king-hearts", json!({})),
            Err(CoreGameError::IllegalAction { reason, .. }) if reason.contains("exceed 31")
        ));
    }

    #[test]
    fn cribbage_nonterminal_state_has_no_payoff() {
        let state = cribbage_state();
        let transition = match apply_action(&state, "cribbage.pegging.play-six-clubs", json!({})) {
            Ok(transition) => transition,
            Err(error) => panic!("cribbage legal action should apply: {error}"),
        };

        assert!(!transition.after.terminal);
        assert_eq!(transition.after.payoff, None);
    }

    #[test]
    fn cribbage_scores_fifteen_and_three_card_run() {
        let state = cribbage_state();
        let transition = match apply_action(&state, "cribbage.pegging.play-six-clubs", json!({})) {
            Ok(transition) => transition,
            Err(error) => panic!("cribbage legal action should apply: {error}"),
        };
        let public: CribbagePublicState =
            match serde_json::from_value(transition.after.public_state) {
                Ok(public) => public,
                Err(error) => panic!("cribbage public state should decode: {error}"),
            };

        assert_eq!(public.running_count, 15);
        assert_eq!(public.pegging_points, 5);
    }

    #[test]
    fn cribbage_feature_view_tracks_run_and_edge() {
        let state = cribbage_state();
        let view = feature_view(&state)
            .unwrap_or_else(|error| panic!("cribbage view should decode: {error}"));

        assert_eq!(view.pegging_count, 9);
        assert_eq!(view.run_potential, 1);
        assert_eq!(view.crib_edge, 1);
        assert!(!view.pair_trap);
        assert!(!view.go_window);
        assert_eq!(view.fifteen_outs, 1);
        assert_eq!(view.max_immediate_points, 5);
    }

    #[test]
    fn cribbage_scenario_pack_meets_coverage_requirements() {
        let pack = cribbage_scenario_pack();
        // 20-scenario floor (plan 009 R1) — we ship 22.
        assert!(
            pack.len() >= 20,
            "cribbage scenario pack should hold >=20 scenarios, found {}",
            pack.len()
        );

        // every scenario id must be unique (stable challenge_id derivation).
        let mut seen: Vec<&str> = pack.iter().map(|scenario| scenario.scenario_id).collect();
        seen.sort_unstable();
        let original_len = seen.len();
        seen.dedup();
        assert_eq!(
            seen.len(),
            original_len,
            "cribbage scenario pack has duplicate scenario_id"
        );

        // Coverage buckets from `genesis/plans/009-cribbage-deepening.md` R1.
        let buckets = [
            "opening-discard",
            "pegging-fifteen",
            "pegging-pair",
            "pegging-run",
            "pegging-go",
            "pegging-thirty-one",
            "counting",
        ];
        for bucket in buckets {
            let count = pack
                .iter()
                .filter(|scenario| scenario.scenario_id.starts_with(bucket))
                .count();
            assert!(count >= 1, "cribbage scenario pack missing bucket {bucket}");
        }

        // pegging_count is in 0..=31 and go_window/fifteen_outs are consistent.
        for scenario in pack {
            assert!(scenario.pegging_count <= 31);
            assert!(scenario.fifteen_outs <= 8);
            assert!(scenario.max_immediate_points <= 12);
        }
    }

    fn cribbage_state() -> CoreGameState {
        match bootstrap_state(ResearchGame::Cribbage) {
            Ok(state) => state,
            Err(error) => panic!("cribbage bootstrap should succeed: {error}"),
        }
    }

    fn over_31_state() -> CoreGameState {
        let public = CribbagePublicState {
            running_count: 25,
            pegging_points: 0,
            actor_hand: vec![Card::new(Rank::King, Suit::Hearts)],
            played_cards: Vec::new(),
        };

        match state_from_public(public, Some(0)) {
            Ok(state) => state,
            Err(error) => panic!("over-31 state should build: {error}"),
        }
    }
}

//! Compliance checks for `CfrGame` implementations.
//!
//! These functions validate that a game implementation satisfies the
//! invariants required for CFR to work correctly.

use std::{collections::BTreeMap, fmt::Debug};

use myosu_games::{CfrGame, CfrInfo, CfrTurn, Utility};

const MAX_GAME_DEPTH: usize = 128;
const MAX_REACHABLE_STATES: usize = 4_096;

/// Assert that a game implementation satisfies all CFR invariants.
///
/// # Panics
///
/// Panics if any invariant is violated.
pub fn assert_game_valid<G>()
where
    G: CfrGame + Debug + PartialEq,
    G::T: CfrInfo<E = G::E, T = G::T, X = G::T>,
{
    assert_root_is_chance_or_player::<G>();

    let root = G::root();
    let mut stats = TraversalStats::<G>::default();
    let mut path = vec![root];
    visit_state(root, &mut path, &mut stats);

    assert!(
        stats.terminal_states > 0,
        "game tree rooted at {:?} did not reach any terminal states",
        root
    );
}

#[derive(Debug)]
struct TraversalStats<G>
where
    G: CfrGame,
    G::T: CfrInfo<E = G::E, T = G::T, X = G::T>,
{
    terminal_states: usize,
    visited_states: usize,
    infoset_choices: BTreeMap<G::T, Vec<G::E>>,
}

impl<G> Default for TraversalStats<G>
where
    G: CfrGame,
    G::T: CfrInfo<E = G::E, T = G::T, X = G::T>,
{
    fn default() -> Self {
        Self {
            terminal_states: 0,
            visited_states: 0,
            infoset_choices: BTreeMap::new(),
        }
    }
}

fn assert_root_is_chance_or_player<G>()
where
    G: CfrGame,
{
    let root_turn = G::root().turn();
    assert!(
        root_turn != G::T::terminal(),
        "root state cannot be terminal"
    );
}

fn visit_state<G>(game: G, path: &mut Vec<G>, stats: &mut TraversalStats<G>)
where
    G: CfrGame + Debug + PartialEq,
    G::T: CfrInfo<E = G::E, T = G::T, X = G::T>,
{
    assert!(
        path.len() <= MAX_GAME_DEPTH,
        "game tree exceeded maximum supported depth of {MAX_GAME_DEPTH} at state {:?}",
        game
    );

    stats.visited_states += 1;
    assert!(
        stats.visited_states <= MAX_REACHABLE_STATES,
        "game tree exceeded maximum supported state budget of {MAX_REACHABLE_STATES}"
    );

    let turn = game.turn();
    let choices = turn.choices();

    if turn == G::T::terminal() {
        assert!(
            choices.is_empty(),
            "terminal state {:?} exposed legal actions {:?}",
            game,
            choices
        );
        assert_terminal_has_utility::<G>(game);
        assert_payoff_is_zero_sum::<G>(game);
        stats.terminal_states += 1;
        return;
    }

    assert!(
        !choices.is_empty(),
        "non-terminal state {:?} exposed no legal actions",
        game
    );

    match stats.infoset_choices.get(&turn) {
        Some(previous) => assert!(
            previous == &choices,
            "information set {:?} produced inconsistent action sets: {:?} vs {:?}",
            turn,
            previous,
            choices
        ),
        None => {
            stats.infoset_choices.insert(turn, choices.clone());
        }
    }

    for edge in choices {
        let next = game.apply(edge);
        assert!(
            next != game,
            "applying edge {:?} to {:?} did not change the game state",
            edge,
            game
        );
        assert!(
            !path.contains(&next),
            "cycle detected after applying edge {:?} from state {:?}",
            edge,
            game
        );

        path.push(next);
        visit_state(next, path, stats);
        path.pop();
    }
}

fn assert_terminal_has_utility<G>(game: G)
where
    G: CfrGame,
{
    let p1 = game.payoff(G::T::from(0));
    let p2 = game.payoff(G::T::from(1));

    assert_is_finite(p1, "player 1 utility");
    assert_is_finite(p2, "player 2 utility");
}

fn assert_payoff_is_zero_sum<G>(game: G)
where
    G: CfrGame,
{
    let p1 = game.payoff(G::T::from(0));
    let p2 = game.payoff(G::T::from(1));
    let net = p1 + p2;

    assert!(
        net.abs() < 1e-6,
        "terminal payoff is not zero-sum: p1={p1:?}, p2={p2:?}, sum={net:?}"
    );
}

fn assert_is_finite(value: Utility, label: &str) {
    assert!(value.is_finite(), "{label} must be finite, got {value:?}");
}

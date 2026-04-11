use thiserror::Error;

use crate::cards::{Card, Rank, validate_unique_cards};

/// Compact hand-strength summary for lightweight poker-family engines.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SimplePokerClass {
    HighCard,
    Pair,
    Trips,
}

/// Result of evaluating a tiny poker challenge hand.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SimplePokerStrength {
    pub class: SimplePokerClass,
    pub high_rank: Rank,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PokerEvalError {
    #[error("poker evaluator requires at least one card")]
    EmptyHand,
    #[error(transparent)]
    Cards(#[from] crate::cards::CardCollectionError),
}

pub fn simple_strength(cards: &[Card]) -> Result<SimplePokerStrength, PokerEvalError> {
    validate_unique_cards(cards)?;
    let high_rank = cards
        .iter()
        .map(|card| card.rank)
        .max()
        .ok_or(PokerEvalError::EmptyHand)?;
    let class = if has_rank_count(cards, 3) {
        SimplePokerClass::Trips
    } else if has_rank_count(cards, 2) {
        SimplePokerClass::Pair
    } else {
        SimplePokerClass::HighCard
    };

    Ok(SimplePokerStrength { class, high_rank })
}

fn has_rank_count(cards: &[Card], target: usize) -> bool {
    cards.iter().any(|candidate| {
        cards
            .iter()
            .filter(|card| card.rank == candidate.rank)
            .count()
            == target
    })
}

#[cfg(test)]
mod tests {
    use crate::cards::{Card, Rank, Suit};
    use crate::eval::poker::{PokerEvalError, SimplePokerClass, simple_strength};

    #[test]
    fn simple_strength_detects_pair_and_high_rank() {
        let strength = match simple_strength(&[
            Card::new(Rank::Ace, Suit::Clubs),
            Card::new(Rank::Ace, Suit::Spades),
            Card::new(Rank::King, Suit::Hearts),
        ]) {
            Ok(strength) => strength,
            Err(error) => panic!("hand should evaluate: {error}"),
        };

        assert_eq!(strength.class, SimplePokerClass::Pair);
        assert_eq!(strength.high_rank, Rank::Ace);
    }

    #[test]
    fn empty_hands_are_rejected() {
        assert_eq!(simple_strength(&[]), Err(PokerEvalError::EmptyHand));
    }
}

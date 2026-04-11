use crate::cards::{Card, validate_unique_cards};

pub fn is_three_card_set(cards: &[Card]) -> bool {
    if cards.len() != 3 || validate_unique_cards(cards).is_err() {
        return false;
    }

    let Some(first) = cards.first() else {
        return false;
    };

    cards.iter().all(|card| card.rank == first.rank)
}

pub fn is_three_card_run(cards: &[Card]) -> bool {
    if cards.len() != 3 || validate_unique_cards(cards).is_err() {
        return false;
    }

    let mut sorted = cards.to_vec();
    sorted.sort();

    match sorted.as_slice() {
        [first, second, third] => {
            first.suit == second.suit
                && second.suit == third.suit
                && first.rank.next() == Some(second.rank)
                && second.rank.next() == Some(third.rank)
        }
        _ => false,
    }
}

pub fn is_simple_meld(cards: &[Card]) -> bool {
    is_three_card_set(cards) || is_three_card_run(cards)
}

#[cfg(test)]
mod tests {
    use crate::cards::{Card, Rank, Suit};
    use crate::eval::meld::{is_simple_meld, is_three_card_run, is_three_card_set};

    #[test]
    fn detects_three_card_set() {
        let cards = [
            Card::new(Rank::Seven, Suit::Clubs),
            Card::new(Rank::Seven, Suit::Diamonds),
            Card::new(Rank::Seven, Suit::Hearts),
        ];

        assert!(is_three_card_set(&cards));
        assert!(is_simple_meld(&cards));
    }

    #[test]
    fn detects_suited_three_card_run() {
        let cards = [
            Card::new(Rank::Seven, Suit::Spades),
            Card::new(Rank::Eight, Suit::Spades),
            Card::new(Rank::Nine, Suit::Spades),
        ];

        assert!(is_three_card_run(&cards));
        assert!(is_simple_meld(&cards));
    }

    #[test]
    fn rejects_unsuited_runs() {
        let cards = [
            Card::new(Rank::Seven, Suit::Spades),
            Card::new(Rank::Eight, Suit::Hearts),
            Card::new(Rank::Nine, Suit::Spades),
        ];

        assert!(!is_three_card_run(&cards));
    }
}

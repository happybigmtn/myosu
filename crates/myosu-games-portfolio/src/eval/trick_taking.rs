use crate::cards::{Card, Suit, validate_unique_cards};

pub fn legal_follow_suit_cards(hand: &[Card], led_suit: Option<Suit>) -> Vec<Card> {
    let Some(led_suit) = led_suit else {
        return hand.to_vec();
    };
    let suited_cards: Vec<Card> = hand
        .iter()
        .copied()
        .filter(|card| card.suit == led_suit)
        .collect();

    if suited_cards.is_empty() {
        hand.to_vec()
    } else {
        suited_cards
    }
}

pub fn highest_legal_card(hand: &[Card], led_suit: Option<Suit>) -> Option<Card> {
    validate_unique_cards(hand).ok()?;
    legal_follow_suit_cards(hand, led_suit).into_iter().max()
}

#[cfg(test)]
mod tests {
    use crate::cards::{Card, Rank, Suit};
    use crate::eval::trick_taking::{highest_legal_card, legal_follow_suit_cards};

    #[test]
    fn follow_suit_keeps_only_led_suit_when_available() {
        let spade = Card::new(Rank::Ace, Suit::Spades);
        let heart = Card::new(Rank::King, Suit::Hearts);
        let legal = legal_follow_suit_cards(&[spade, heart], Some(Suit::Hearts));

        assert_eq!(legal, vec![heart]);
    }

    #[test]
    fn follow_suit_allows_any_card_when_void() {
        let spade = Card::new(Rank::Ace, Suit::Spades);
        let legal = legal_follow_suit_cards(&[spade], Some(Suit::Hearts));

        assert_eq!(legal, vec![spade]);
    }

    #[test]
    fn highest_legal_card_rejects_duplicate_hands() {
        let spade = Card::new(Rank::Ace, Suit::Spades);

        assert_eq!(highest_legal_card(&[spade, spade], None), None);
    }
}

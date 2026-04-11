use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Four suits used by the Anglo-American card games in the research corpus.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Suit {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
}

impl Suit {
    pub const ALL: [Self; 4] = [Self::Clubs, Self::Diamonds, Self::Hearts, Self::Spades];
}

/// Ordered rank representation for standard-card evaluators.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Rank {
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}

impl Rank {
    pub const STANDARD: [Self; 13] = [
        Self::Two,
        Self::Three,
        Self::Four,
        Self::Five,
        Self::Six,
        Self::Seven,
        Self::Eight,
        Self::Nine,
        Self::Ten,
        Self::Jack,
        Self::Queen,
        Self::King,
        Self::Ace,
    ];

    pub const SHORT_DECK: [Self; 9] = [
        Self::Six,
        Self::Seven,
        Self::Eight,
        Self::Nine,
        Self::Ten,
        Self::Jack,
        Self::Queen,
        Self::King,
        Self::Ace,
    ];

    pub const fn next(self) -> Option<Self> {
        match self {
            Self::Two => Some(Self::Three),
            Self::Three => Some(Self::Four),
            Self::Four => Some(Self::Five),
            Self::Five => Some(Self::Six),
            Self::Six => Some(Self::Seven),
            Self::Seven => Some(Self::Eight),
            Self::Eight => Some(Self::Nine),
            Self::Nine => Some(Self::Ten),
            Self::Ten => Some(Self::Jack),
            Self::Jack => Some(Self::Queen),
            Self::Queen => Some(Self::King),
            Self::King => Some(Self::Ace),
            Self::Ace => None,
        }
    }
}

/// Standard playing card.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Card {
    pub rank: Rank,
    pub suit: Suit,
}

impl Card {
    pub const fn new(rank: Rank, suit: Suit) -> Self {
        Self { rank, suit }
    }
}

/// Supported deterministic deck families.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DeckKind {
    Standard52,
    ShortDeck36,
}

impl DeckKind {
    pub fn cards(self) -> Vec<Card> {
        let ranks: &[Rank] = match self {
            Self::Standard52 => &Rank::STANDARD,
            Self::ShortDeck36 => &Rank::SHORT_DECK,
        };

        ranks
            .iter()
            .copied()
            .flat_map(|rank| Suit::ALL.into_iter().map(move |suit| Card::new(rank, suit)))
            .collect()
    }

    pub fn contains(self, card: Card) -> bool {
        let ranks: &[Rank] = match self {
            Self::Standard52 => &Rank::STANDARD,
            Self::ShortDeck36 => &Rank::SHORT_DECK,
        };

        ranks.contains(&card.rank)
    }
}

/// One physical Hanafuda card in a 48-card deck.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HanafudaCard {
    pub month: HanafudaMonth,
    pub kind: HanafudaKind,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum HanafudaMonth {
    January,
    February,
    March,
    April,
    May,
    June,
    July,
    August,
    September,
    October,
    November,
    December,
}

impl HanafudaMonth {
    pub const ALL: [Self; 12] = [
        Self::January,
        Self::February,
        Self::March,
        Self::April,
        Self::May,
        Self::June,
        Self::July,
        Self::August,
        Self::September,
        Self::October,
        Self::November,
        Self::December,
    ];
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum HanafudaKind {
    Bright,
    Animal,
    Ribbon,
    Chaff,
}

impl HanafudaKind {
    pub const ALL: [Self; 4] = [Self::Bright, Self::Animal, Self::Ribbon, Self::Chaff];
}

pub fn hanafuda_deck() -> Vec<HanafudaCard> {
    HanafudaMonth::ALL
        .into_iter()
        .flat_map(|month| {
            HanafudaKind::ALL
                .into_iter()
                .map(move |kind| HanafudaCard { month, kind })
        })
        .collect()
}

/// Errors raised when card collections are malformed.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum CardCollectionError {
    #[error("duplicate card in collection: {card:?}")]
    DuplicateCard { card: Card },
    #[error("card {card:?} is not part of {deck:?}")]
    WrongDeck { deck: DeckKind, card: Card },
}

pub fn validate_unique_cards(cards: &[Card]) -> Result<(), CardCollectionError> {
    let mut seen = BTreeSet::new();
    for card in cards {
        if !seen.insert(*card) {
            return Err(CardCollectionError::DuplicateCard { card: *card });
        }
    }

    Ok(())
}

pub fn validate_deck_membership(deck: DeckKind, cards: &[Card]) -> Result<(), CardCollectionError> {
    validate_unique_cards(cards)?;
    for card in cards {
        if !deck.contains(*card) {
            return Err(CardCollectionError::WrongDeck { deck, card: *card });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::cards::{
        Card, CardCollectionError, DeckKind, Rank, Suit, hanafuda_deck, validate_deck_membership,
        validate_unique_cards,
    };

    #[test]
    fn standard_and_short_decks_have_expected_unique_cards() {
        let standard = DeckKind::Standard52.cards();
        let short_deck = DeckKind::ShortDeck36.cards();

        assert_eq!(standard.len(), 52);
        assert_eq!(short_deck.len(), 36);
        assert!(validate_unique_cards(&standard).is_ok());
        assert!(validate_unique_cards(&short_deck).is_ok());
        assert!(standard.contains(&Card::new(Rank::Two, Suit::Clubs)));
        assert!(!short_deck.contains(&Card::new(Rank::Two, Suit::Clubs)));
    }

    #[test]
    fn hanafuda_deck_has_four_cards_per_month() {
        let deck = hanafuda_deck();

        assert_eq!(deck.len(), 48);
        assert_eq!(
            deck.into_iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            48
        );
    }

    #[test]
    fn duplicate_cards_are_rejected() {
        let card = Card::new(Rank::Ace, Suit::Spades);
        let error = validate_unique_cards(&[card, card]);

        assert!(matches!(
            error,
            Err(CardCollectionError::DuplicateCard { card: duplicate }) if duplicate == card
        ));
    }

    #[test]
    fn short_deck_membership_rejects_low_cards() {
        let card = Card::new(Rank::Five, Suit::Hearts);
        let error = validate_deck_membership(DeckKind::ShortDeck36, &[card]);

        assert!(matches!(
            error,
            Err(CardCollectionError::WrongDeck {
                deck: DeckKind::ShortDeck36,
                card: rejected,
            }) if rejected == card
        ));
    }
}

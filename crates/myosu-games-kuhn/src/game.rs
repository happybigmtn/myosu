use myosu_games::{CfrEdge, CfrGame, CfrInfo, CfrTurn, Utility};
use rbp_mccfr::{CfrPublic, CfrSecret};
use rbp_transport::Support;
use serde::{Deserialize, Serialize};

const UNKNOWN_CARD: u8 = u8::MAX;

/// The three cards in standard Kuhn poker.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
pub enum KuhnCard {
    Jack = 0,
    Queen = 1,
    King = 2,
}

impl KuhnCard {
    /// Return every card in the standard Kuhn deck.
    pub fn all() -> [Self; 3] {
        [Self::Jack, Self::Queen, Self::King]
    }

    /// Return whether this card beats another at showdown.
    pub fn beats(self, other: Self) -> bool {
        self.rank() > other.rank()
    }

    fn rank(self) -> u8 {
        self as u8
    }
}

/// Turn classification for Kuhn poker.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum KuhnTurn {
    /// Chance deals the two private cards.
    Chance,
    /// Player one to act.
    PlayerOne,
    /// Player two to act.
    PlayerTwo,
    /// Terminal payoff node.
    Terminal,
}

impl From<usize> for KuhnTurn {
    fn from(player: usize) -> Self {
        match player {
            0 => Self::PlayerOne,
            1 => Self::PlayerTwo,
            _ => panic!("kuhn poker only has 2 players"),
        }
    }
}

impl Support for KuhnTurn {}

impl CfrTurn for KuhnTurn {
    fn chance() -> Self {
        Self::Chance
    }

    fn terminal() -> Self {
        Self::Terminal
    }
}

/// An action in Kuhn poker.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum KuhnEdge {
    /// Chance deals one distinct card to each player.
    Deal { p1: KuhnCard, p2: KuhnCard },
    /// Pass without adding to the pot.
    Check,
    /// Add one more chip to the pot.
    Bet,
    /// Match the outstanding bet.
    Call,
    /// Fold to the outstanding bet.
    Fold,
}

impl Support for KuhnEdge {}
impl CfrEdge for KuhnEdge {}

/// Public action history for a Kuhn poker hand.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum KuhnHistory {
    /// No player action has happened yet.
    Opening,
    /// Player one checked.
    P1Checked,
    /// Player one bet.
    P1Bet,
    /// Player one checked and player two bet.
    P1CheckP2Bet,
    /// Both players checked and went to showdown.
    CheckCheck,
    /// Player one bet and player two called.
    BetCall,
    /// Player one bet and player two folded.
    BetFold,
    /// Player one checked, player two bet, and player one called.
    CheckBetCall,
    /// Player one checked, player two bet, and player one folded.
    CheckBetFold,
}

impl KuhnHistory {
    /// Return the public action sequence without the chance deal.
    pub fn actions(self) -> Vec<KuhnEdge> {
        match self {
            Self::Opening => Vec::new(),
            Self::P1Checked => vec![KuhnEdge::Check],
            Self::P1Bet => vec![KuhnEdge::Bet],
            Self::P1CheckP2Bet => vec![KuhnEdge::Check, KuhnEdge::Bet],
            Self::CheckCheck => vec![KuhnEdge::Check, KuhnEdge::Check],
            Self::BetCall => vec![KuhnEdge::Bet, KuhnEdge::Call],
            Self::BetFold => vec![KuhnEdge::Bet, KuhnEdge::Fold],
            Self::CheckBetCall => vec![KuhnEdge::Check, KuhnEdge::Bet, KuhnEdge::Call],
            Self::CheckBetFold => vec![KuhnEdge::Check, KuhnEdge::Bet, KuhnEdge::Fold],
        }
    }

    fn showdown_stake(self) -> Option<Utility> {
        match self {
            Self::CheckCheck => Some(1.0),
            Self::BetCall | Self::CheckBetCall => Some(2.0),
            _ => None,
        }
    }

    fn folded_winner(self) -> Option<KuhnTurn> {
        match self {
            Self::BetFold => Some(KuhnTurn::PlayerOne),
            Self::CheckBetFold => Some(KuhnTurn::PlayerTwo),
            _ => None,
        }
    }

    fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::CheckCheck
                | Self::BetCall
                | Self::BetFold
                | Self::CheckBetCall
                | Self::CheckBetFold
        )
    }
}

/// Public information visible at an information set.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct KuhnPublic {
    actor: KuhnTurn,
    history: KuhnHistory,
}

impl KuhnPublic {
    fn new(actor: KuhnTurn, history: KuhnHistory) -> Self {
        Self { actor, history }
    }

    /// Return the active public history.
    pub fn history(self) -> KuhnHistory {
        self.history
    }
}

impl CfrPublic for KuhnPublic {
    type E = KuhnEdge;
    type T = KuhnTurn;

    fn choices(&self) -> Vec<Self::E> {
        match (self.actor, self.history) {
            (KuhnTurn::Chance, KuhnHistory::Opening) => {
                let mut deals = Vec::with_capacity(6);
                for p1 in KuhnCard::all() {
                    for p2 in KuhnCard::all() {
                        if p1 != p2 {
                            deals.push(KuhnEdge::Deal { p1, p2 });
                        }
                    }
                }
                deals
            }
            (KuhnTurn::PlayerOne, KuhnHistory::Opening)
            | (KuhnTurn::PlayerTwo, KuhnHistory::P1Checked) => {
                vec![KuhnEdge::Check, KuhnEdge::Bet]
            }
            (KuhnTurn::PlayerTwo, KuhnHistory::P1Bet)
            | (KuhnTurn::PlayerOne, KuhnHistory::P1CheckP2Bet) => {
                vec![KuhnEdge::Call, KuhnEdge::Fold]
            }
            (KuhnTurn::Terminal, history) if history.is_terminal() => Vec::new(),
            _ => panic!("invalid actor/history pairing for public choices"),
        }
    }

    fn history(&self) -> Vec<Self::E> {
        self.history.actions()
    }
}

/// Private information for the acting player.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct KuhnSecret(pub KuhnCard);

impl Support for KuhnSecret {}
impl CfrSecret for KuhnSecret {}

/// Information set for Kuhn poker.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct KuhnInfo {
    public: KuhnPublic,
    secret: KuhnSecret,
}

impl KuhnInfo {
    fn new(public: KuhnPublic, secret: KuhnSecret) -> Self {
        Self { public, secret }
    }

    /// Return the acting player's private card.
    pub fn card(self) -> KuhnCard {
        self.secret.0
    }
}

impl CfrInfo for KuhnInfo {
    type E = KuhnEdge;
    type T = KuhnTurn;
    type X = KuhnPublic;
    type Y = KuhnSecret;

    fn public(&self) -> Self::X {
        self.public
    }

    fn secret(&self) -> Self::Y {
        self.secret
    }
}

/// Standard two-player Kuhn poker game state.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct KuhnGame {
    p1_card: u8,
    p2_card: u8,
    history: KuhnHistory,
    actor: KuhnTurn,
}

impl KuhnGame {
    /// Return the acting player's information set when the game is at a decision node.
    pub fn info(self) -> Option<KuhnInfo> {
        let public = KuhnPublic::new(self.actor, self.history);
        let secret = match self.actor {
            KuhnTurn::PlayerOne => KuhnSecret(self.p1_card()),
            KuhnTurn::PlayerTwo => KuhnSecret(self.p2_card()),
            KuhnTurn::Chance | KuhnTurn::Terminal => return None,
        };
        Some(KuhnInfo::new(public, secret))
    }

    /// Return the public history.
    pub fn history(self) -> KuhnHistory {
        self.history
    }

    fn p1_card(self) -> KuhnCard {
        decode_card(self.p1_card)
    }

    fn p2_card(self) -> KuhnCard {
        decode_card(self.p2_card)
    }

    fn showdown_winner(self) -> KuhnTurn {
        if self.p1_card().beats(self.p2_card()) {
            KuhnTurn::PlayerOne
        } else {
            KuhnTurn::PlayerTwo
        }
    }
}

impl CfrGame for KuhnGame {
    type E = KuhnEdge;
    type T = KuhnTurn;

    fn root() -> Self {
        Self {
            p1_card: UNKNOWN_CARD,
            p2_card: UNKNOWN_CARD,
            history: KuhnHistory::Opening,
            actor: KuhnTurn::Chance,
        }
    }

    fn turn(&self) -> Self::T {
        self.actor
    }

    fn apply(&self, edge: Self::E) -> Self {
        match (self.actor, self.history, edge) {
            (KuhnTurn::Chance, KuhnHistory::Opening, KuhnEdge::Deal { p1, p2 }) => {
                assert_ne!(p1, p2, "chance must deal distinct cards");
                Self {
                    p1_card: encode_card(p1),
                    p2_card: encode_card(p2),
                    history: KuhnHistory::Opening,
                    actor: KuhnTurn::PlayerOne,
                }
            }
            (KuhnTurn::PlayerOne, KuhnHistory::Opening, KuhnEdge::Check) => Self {
                p1_card: self.p1_card,
                p2_card: self.p2_card,
                history: KuhnHistory::P1Checked,
                actor: KuhnTurn::PlayerTwo,
            },
            (KuhnTurn::PlayerOne, KuhnHistory::Opening, KuhnEdge::Bet) => Self {
                p1_card: self.p1_card,
                p2_card: self.p2_card,
                history: KuhnHistory::P1Bet,
                actor: KuhnTurn::PlayerTwo,
            },
            (KuhnTurn::PlayerTwo, KuhnHistory::P1Checked, KuhnEdge::Check) => Self {
                p1_card: self.p1_card,
                p2_card: self.p2_card,
                history: KuhnHistory::CheckCheck,
                actor: KuhnTurn::Terminal,
            },
            (KuhnTurn::PlayerTwo, KuhnHistory::P1Checked, KuhnEdge::Bet) => Self {
                p1_card: self.p1_card,
                p2_card: self.p2_card,
                history: KuhnHistory::P1CheckP2Bet,
                actor: KuhnTurn::PlayerOne,
            },
            (KuhnTurn::PlayerTwo, KuhnHistory::P1Bet, KuhnEdge::Call) => Self {
                p1_card: self.p1_card,
                p2_card: self.p2_card,
                history: KuhnHistory::BetCall,
                actor: KuhnTurn::Terminal,
            },
            (KuhnTurn::PlayerTwo, KuhnHistory::P1Bet, KuhnEdge::Fold) => Self {
                p1_card: self.p1_card,
                p2_card: self.p2_card,
                history: KuhnHistory::BetFold,
                actor: KuhnTurn::Terminal,
            },
            (KuhnTurn::PlayerOne, KuhnHistory::P1CheckP2Bet, KuhnEdge::Call) => Self {
                p1_card: self.p1_card,
                p2_card: self.p2_card,
                history: KuhnHistory::CheckBetCall,
                actor: KuhnTurn::Terminal,
            },
            (KuhnTurn::PlayerOne, KuhnHistory::P1CheckP2Bet, KuhnEdge::Fold) => Self {
                p1_card: self.p1_card,
                p2_card: self.p2_card,
                history: KuhnHistory::CheckBetFold,
                actor: KuhnTurn::Terminal,
            },
            (KuhnTurn::Terminal, history, _) if history.is_terminal() => {
                panic!("terminal states do not accept actions")
            }
            _ => panic!("invalid Kuhn action for current actor/history state"),
        }
    }

    fn payoff(&self, turn: Self::T) -> Utility {
        assert_eq!(
            self.actor,
            KuhnTurn::Terminal,
            "payoff requires terminal state"
        );
        let winner = self
            .history
            .folded_winner()
            .unwrap_or_else(|| self.showdown_winner());
        let stake = self.history.showdown_stake().unwrap_or(1.0);

        match turn {
            KuhnTurn::PlayerOne | KuhnTurn::PlayerTwo => {
                if turn == winner {
                    stake
                } else {
                    -stake
                }
            }
            KuhnTurn::Chance | KuhnTurn::Terminal => {
                panic!("payoff only applies to player turns")
            }
        }
    }
}

fn encode_card(card: KuhnCard) -> u8 {
    card as u8
}

fn decode_card(raw: u8) -> KuhnCard {
    match raw {
        0 => KuhnCard::Jack,
        1 => KuhnCard::Queen,
        2 => KuhnCard::King,
        _ => panic!("invalid stored Kuhn card"),
    }
}

#[cfg(test)]
mod tests {
    use super::{KuhnCard, KuhnEdge, KuhnGame, KuhnHistory, KuhnPublic, KuhnTurn};
    use myosu_games::CfrGame;
    use rbp_mccfr::{CfrInfo, CfrPublic};
    use std::collections::BTreeSet;

    #[test]
    fn root_is_chance_with_six_unique_deals() {
        let root = KuhnGame::root();
        let public = KuhnPublic::new(KuhnTurn::Chance, KuhnHistory::Opening);
        let deals = public.choices();

        assert_eq!(root.turn(), KuhnTurn::Chance);
        assert_eq!(deals.len(), 6);
        assert_eq!(deals.iter().collect::<BTreeSet<_>>().len(), 6);
    }

    #[test]
    fn check_bet_call_reaches_two_chip_showdown() {
        let game = KuhnGame::root()
            .apply(KuhnEdge::Deal {
                p1: KuhnCard::King,
                p2: KuhnCard::Queen,
            })
            .apply(KuhnEdge::Check)
            .apply(KuhnEdge::Bet)
            .apply(KuhnEdge::Call);

        assert_eq!(game.turn(), KuhnTurn::Terminal);
        assert_eq!(game.payoff(KuhnTurn::PlayerOne), 2.0);
        assert_eq!(game.payoff(KuhnTurn::PlayerTwo), -2.0);
    }

    #[test]
    fn bet_fold_awards_one_chip_to_bettor() {
        let game = KuhnGame::root()
            .apply(KuhnEdge::Deal {
                p1: KuhnCard::Jack,
                p2: KuhnCard::Queen,
            })
            .apply(KuhnEdge::Bet)
            .apply(KuhnEdge::Fold);

        assert_eq!(game.turn(), KuhnTurn::Terminal);
        assert_eq!(game.payoff(KuhnTurn::PlayerOne), 1.0);
        assert_eq!(game.payoff(KuhnTurn::PlayerTwo), -1.0);
    }

    #[test]
    fn info_exposes_private_card_and_legal_choices() {
        let game = KuhnGame::root().apply(KuhnEdge::Deal {
            p1: KuhnCard::Queen,
            p2: KuhnCard::Jack,
        });
        let info = game.info().expect("player turn should expose info");

        assert_eq!(info.card(), KuhnCard::Queen);
        assert_eq!(info.public().history(), KuhnHistory::Opening);
        assert_eq!(
            info.public().choices(),
            vec![KuhnEdge::Check, KuhnEdge::Bet]
        );
    }

    #[test]
    fn reachable_information_sets_total_twelve() {
        fn walk(game: KuhnGame, infos: &mut BTreeSet<super::KuhnInfo>) {
            if let Some(info) = game.info() {
                infos.insert(info);
            }

            let public = match game.turn() {
                KuhnTurn::Chance | KuhnTurn::Terminal => {
                    KuhnPublic::new(game.turn(), game.history())
                }
                _ => game
                    .info()
                    .expect("player turns should expose info")
                    .public(),
            };

            for edge in public.choices() {
                walk(game.apply(edge), infos);
            }
        }

        let mut infos = BTreeSet::new();
        walk(KuhnGame::root(), &mut infos);

        assert_eq!(infos.len(), 12);
    }
}

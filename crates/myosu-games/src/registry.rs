use crate::traits::GameType;

/// Human-readable description of a registered game type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameDescriptor {
    /// Canonical game identifier.
    pub game_type: GameType,
    /// Expected player count for the game.
    pub num_players: u8,
    /// Short operator-facing description.
    pub description: &'static str,
    /// Whether the game is built into the current repo.
    pub builtin: bool,
}

impl GameDescriptor {
    fn builtin(game_type: GameType, description: &'static str) -> Self {
        let num_players = game_type.num_players();

        Self {
            game_type,
            num_players,
            description,
            builtin: true,
        }
    }

    fn custom(name: String) -> Self {
        Self {
            game_type: GameType::Custom(name),
            num_players: 2,
            description: "Custom game type",
            builtin: false,
        }
    }
}

/// Runtime registry for supported myosu game types.
pub struct GameRegistry;

impl GameRegistry {
    /// Return the built-in game types known to this repo.
    pub fn supported() -> Vec<GameDescriptor> {
        vec![
            GameDescriptor::builtin(GameType::NlheHeadsUp, "Heads-up no-limit hold'em"),
            GameDescriptor::builtin(GameType::NlheSixMax, "Six-max no-limit hold'em"),
            GameDescriptor::builtin(GameType::KuhnPoker, "Two-player Kuhn poker"),
            GameDescriptor::builtin(GameType::LiarsDice, "Two-player liar's dice"),
            GameDescriptor::builtin(GameType::Plo, "Pot-limit Omaha"),
            GameDescriptor::builtin(GameType::NlheTournament, "No-limit hold'em tournament"),
            GameDescriptor::builtin(GameType::ShortDeck, "Short Deck hold'em"),
            GameDescriptor::builtin(GameType::TeenPatti, "Teen Patti"),
            GameDescriptor::builtin(GameType::HanafudaKoiKoi, "Hanafuda Koi-Koi"),
            GameDescriptor::builtin(GameType::HwatuGoStop, "Hwatu Go-Stop"),
            GameDescriptor::builtin(GameType::RiichiMahjong, "Riichi Mahjong"),
            GameDescriptor::builtin(GameType::Bridge, "Contract Bridge"),
            GameDescriptor::builtin(GameType::GinRummy, "Gin Rummy"),
            GameDescriptor::builtin(GameType::Stratego, "Stratego"),
            GameDescriptor::builtin(GameType::OfcChinesePoker, "Open Face Chinese Poker"),
            GameDescriptor::builtin(GameType::Spades, "Spades"),
            GameDescriptor::builtin(GameType::DouDiZhu, "Dou Di Zhu"),
            GameDescriptor::builtin(GameType::PusoyDos, "Pusoy Dos"),
            GameDescriptor::builtin(GameType::TienLen, "Tien Len"),
            GameDescriptor::builtin(GameType::CallBreak, "Call Break"),
            GameDescriptor::builtin(GameType::Backgammon, "Backgammon"),
            GameDescriptor::builtin(GameType::Hearts, "Hearts"),
            GameDescriptor::builtin(GameType::Cribbage, "Cribbage"),
        ]
    }

    /// Describe a single game type.
    pub fn describe(game_type: GameType) -> GameDescriptor {
        match game_type {
            GameType::NlheHeadsUp => {
                GameDescriptor::builtin(GameType::NlheHeadsUp, "Heads-up no-limit hold'em")
            }
            GameType::NlheSixMax => {
                GameDescriptor::builtin(GameType::NlheSixMax, "Six-max no-limit hold'em")
            }
            GameType::KuhnPoker => {
                GameDescriptor::builtin(GameType::KuhnPoker, "Two-player Kuhn poker")
            }
            GameType::LiarsDice => {
                GameDescriptor::builtin(GameType::LiarsDice, "Two-player liar's dice")
            }
            GameType::Plo => GameDescriptor::builtin(GameType::Plo, "Pot-limit Omaha"),
            GameType::NlheTournament => {
                GameDescriptor::builtin(GameType::NlheTournament, "No-limit hold'em tournament")
            }
            GameType::ShortDeck => {
                GameDescriptor::builtin(GameType::ShortDeck, "Short Deck hold'em")
            }
            GameType::TeenPatti => GameDescriptor::builtin(GameType::TeenPatti, "Teen Patti"),
            GameType::HanafudaKoiKoi => {
                GameDescriptor::builtin(GameType::HanafudaKoiKoi, "Hanafuda Koi-Koi")
            }
            GameType::HwatuGoStop => {
                GameDescriptor::builtin(GameType::HwatuGoStop, "Hwatu Go-Stop")
            }
            GameType::RiichiMahjong => {
                GameDescriptor::builtin(GameType::RiichiMahjong, "Riichi Mahjong")
            }
            GameType::Bridge => GameDescriptor::builtin(GameType::Bridge, "Contract Bridge"),
            GameType::GinRummy => GameDescriptor::builtin(GameType::GinRummy, "Gin Rummy"),
            GameType::Stratego => GameDescriptor::builtin(GameType::Stratego, "Stratego"),
            GameType::OfcChinesePoker => {
                GameDescriptor::builtin(GameType::OfcChinesePoker, "Open Face Chinese Poker")
            }
            GameType::Spades => GameDescriptor::builtin(GameType::Spades, "Spades"),
            GameType::DouDiZhu => GameDescriptor::builtin(GameType::DouDiZhu, "Dou Di Zhu"),
            GameType::PusoyDos => GameDescriptor::builtin(GameType::PusoyDos, "Pusoy Dos"),
            GameType::TienLen => GameDescriptor::builtin(GameType::TienLen, "Tien Len"),
            GameType::CallBreak => GameDescriptor::builtin(GameType::CallBreak, "Call Break"),
            GameType::Backgammon => GameDescriptor::builtin(GameType::Backgammon, "Backgammon"),
            GameType::Hearts => GameDescriptor::builtin(GameType::Hearts, "Hearts"),
            GameType::Cribbage => GameDescriptor::builtin(GameType::Cribbage, "Cribbage"),
            GameType::Custom(name) => GameDescriptor::custom(name),
        }
    }

    /// Parse and describe an on-chain game type byte string.
    pub fn from_bytes(bytes: &[u8]) -> Option<GameDescriptor> {
        GameType::from_bytes(bytes).map(Self::describe)
    }

    /// Report whether a game type is built into the current repo.
    pub fn is_builtin(game_type: &GameType) -> bool {
        !matches!(game_type, GameType::Custom(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_game_types() {
        let heads_up = GameRegistry::from_bytes(b"nlhe_hu").expect("known game should resolve");
        let six_max = GameRegistry::from_bytes(b"nlhe_6max").expect("known game should resolve");
        let kuhn_poker =
            GameRegistry::from_bytes(b"kuhn_poker").expect("known game should resolve");
        let liars_dice =
            GameRegistry::from_bytes(b"liars_dice").expect("known game should resolve");
        let bridge = GameRegistry::from_bytes(b"bridge").expect("known game should resolve");
        let cribbage = GameRegistry::from_bytes(b"cribbage").expect("known game should resolve");

        assert_eq!(heads_up.game_type, GameType::NlheHeadsUp);
        assert_eq!(six_max.game_type, GameType::NlheSixMax);
        assert_eq!(kuhn_poker.game_type, GameType::KuhnPoker);
        assert_eq!(liars_dice.game_type, GameType::LiarsDice);
        assert_eq!(bridge.game_type, GameType::Bridge);
        assert_eq!(cribbage.game_type, GameType::Cribbage);
        assert!(heads_up.builtin);
        assert!(six_max.builtin);
        assert!(kuhn_poker.builtin);
        assert!(liars_dice.builtin);
        assert!(bridge.builtin);
        assert!(cribbage.builtin);
    }

    #[test]
    fn unknown_game_type_custom() {
        let descriptor =
            GameRegistry::from_bytes(b"draw_mahjong").expect("custom game should resolve");

        assert_eq!(
            descriptor,
            GameDescriptor {
                game_type: GameType::Custom("draw_mahjong".to_string()),
                num_players: 2,
                description: "Custom game type",
                builtin: false,
            }
        );
    }

    #[test]
    fn custom_game_registration_requires_no_registry_changes() {
        let custom_game = GameType::Custom("test_game".to_string());
        let descriptor = GameRegistry::describe(custom_game.clone());
        let roundtrip = GameRegistry::from_bytes(&custom_game.to_bytes())
            .expect("custom game bytes should resolve");

        assert_eq!(descriptor.game_type, custom_game);
        assert_eq!(descriptor.num_players, 2);
        assert_eq!(descriptor.description, "Custom game type");
        assert!(!descriptor.builtin);
        assert_eq!(roundtrip, descriptor);
        assert!(!GameRegistry::is_builtin(&custom_game));
        assert!(
            !GameRegistry::supported()
                .iter()
                .any(|candidate| candidate.game_type == custom_game)
        );
    }

    #[test]
    fn roundtrip_bytes() {
        for descriptor in GameRegistry::supported() {
            let bytes = descriptor.game_type.to_bytes();
            let decoded = GameRegistry::from_bytes(&bytes).expect("supported game should decode");

            assert_eq!(decoded.game_type, descriptor.game_type);
        }
    }

    #[test]
    fn num_players_correct() {
        let descriptors = GameRegistry::supported();

        assert_eq!(descriptors[0].num_players, 2);
        assert_eq!(descriptors[1].num_players, 6);
        assert_eq!(descriptors[2].num_players, 2);
        assert_eq!(descriptors[3].num_players, 2);
        assert!(
            descriptors
                .iter()
                .any(|descriptor| descriptor.game_type == GameType::RiichiMahjong
                    && descriptor.num_players == 4)
        );
    }
}

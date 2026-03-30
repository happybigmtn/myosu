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
    pub fn supported() -> [GameDescriptor; 3] {
        [
            GameDescriptor::builtin(GameType::NlheHeadsUp, "Heads-up no-limit hold'em"),
            GameDescriptor::builtin(GameType::NlheSixMax, "Six-max no-limit hold'em"),
            GameDescriptor::builtin(GameType::LiarsDice, "Two-player liar's dice"),
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
            GameType::LiarsDice => {
                GameDescriptor::builtin(GameType::LiarsDice, "Two-player liar's dice")
            }
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
        let liars_dice =
            GameRegistry::from_bytes(b"liars_dice").expect("known game should resolve");

        assert_eq!(heads_up.game_type, GameType::NlheHeadsUp);
        assert_eq!(six_max.game_type, GameType::NlheSixMax);
        assert_eq!(liars_dice.game_type, GameType::LiarsDice);
        assert!(heads_up.builtin);
        assert!(six_max.builtin);
        assert!(liars_dice.builtin);
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
    }
}

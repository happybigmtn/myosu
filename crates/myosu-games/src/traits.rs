//! Core CFR trait re-exports and myosu-specific game types.
//!
//! This module provides:
//! - Re-exports of robopoker's game-agnostic CFR traits
//! - `GameConfig` for typed game parameter configuration
//! - `StrategyQuery`/`StrategyResponse` for miner-validator communication

pub use rbp_core::{Probability, Utility};
pub use rbp_mccfr::{CfrEdge, CfrGame, CfrInfo, CfrTurn, Encoder, Profile};

use serde::{Deserialize, Serialize};

/// Top-level configuration for a game instance.
///
/// This struct is passed when initializing game engines and solvers.
/// The `game_type` and `params` together define the exact game variant.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct GameConfig {
    /// The type of game being played
    pub game_type: GameType,
    /// Number of players in the game
    pub num_players: u8,
    /// Game-specific parameters
    pub params: GameParams,
}

impl GameConfig {
    /// Create a new game configuration
    pub fn new(game_type: GameType, num_players: u8, params: GameParams) -> Self {
        Self {
            game_type,
            num_players,
            params,
        }
    }

    /// Create a standard NLHE heads-up configuration
    pub fn nlhe_heads_up(stack_bb: u32) -> Self {
        Self {
            game_type: GameType::NlheHeadsUp,
            num_players: 2,
            params: GameParams::NlheHeadsUp {
                stack_bb,
                ante_bb: None,
            },
        }
    }

    /// Create the standard Kuhn poker configuration.
    pub fn kuhn_poker() -> Self {
        Self {
            game_type: GameType::KuhnPoker,
            num_players: 2,
            params: GameParams::KuhnPoker,
        }
    }
}

/// Known game types supported by myosu.
///
/// The `Custom` variant allows for extensibility without code changes.
/// New game types can be registered on-chain using a unique string identifier.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum GameType {
    /// No-Limit Hold'em heads-up (2 players)
    NlheHeadsUp,
    /// No-Limit Hold'em 6-max
    NlheSixMax,
    /// Kuhn Poker
    KuhnPoker,
    /// Liar's Dice
    LiarsDice,
    /// Pot-Limit Omaha
    Plo,
    /// No-Limit Hold'em tournament
    NlheTournament,
    /// Short Deck Hold'em
    ShortDeck,
    /// Teen Patti
    TeenPatti,
    /// Hanafuda Koi-Koi
    HanafudaKoiKoi,
    /// Hwatu Go-Stop
    HwatuGoStop,
    /// Riichi Mahjong
    RiichiMahjong,
    /// Contract Bridge
    Bridge,
    /// Gin Rummy
    GinRummy,
    /// Stratego
    Stratego,
    /// Open Face Chinese Poker
    OfcChinesePoker,
    /// Spades
    Spades,
    /// Dou Di Zhu
    DouDiZhu,
    /// Pusoy Dos
    PusoyDos,
    /// Tien Len
    TienLen,
    /// Call Break
    CallBreak,
    /// Backgammon
    Backgammon,
    /// Hearts
    Hearts,
    /// Cribbage
    Cribbage,
    /// Custom game type identified by string
    Custom(String),
}

impl GameType {
    /// Parse a game type from a byte string (as stored on-chain).
    ///
    /// Known game types are mapped to their variants; unknown types
    /// are stored as `Custom`.
    ///
    /// # Examples
    ///
    /// ```
    /// use myosu_games::traits::GameType;
    ///
    /// assert_eq!(GameType::from_bytes(b"nlhe_hu"), Some(GameType::NlheHeadsUp));
    /// assert_eq!(GameType::from_bytes(b"nlhe_6max"), Some(GameType::NlheSixMax));
    /// assert_eq!(GameType::from_bytes(b"kuhn_poker"), Some(GameType::KuhnPoker));
    /// assert_eq!(GameType::from_bytes(b"liars_dice"), Some(GameType::LiarsDice));
    /// assert_eq!(GameType::from_bytes(b"bridge"), Some(GameType::Bridge));
    /// assert_eq!(GameType::from_bytes(b"unknown"), Some(GameType::Custom("unknown".to_string())));
    /// ```
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        match bytes {
            b"nlhe_hu" => Some(Self::NlheHeadsUp),
            b"nlhe_6max" => Some(Self::NlheSixMax),
            b"kuhn_poker" => Some(Self::KuhnPoker),
            b"liars_dice" => Some(Self::LiarsDice),
            b"plo" => Some(Self::Plo),
            b"nlhe_tournament" => Some(Self::NlheTournament),
            b"short_deck" => Some(Self::ShortDeck),
            b"teen_patti" => Some(Self::TeenPatti),
            b"hanafuda_koi_koi" => Some(Self::HanafudaKoiKoi),
            b"hwatu_go_stop" => Some(Self::HwatuGoStop),
            b"riichi_mahjong" => Some(Self::RiichiMahjong),
            b"bridge" => Some(Self::Bridge),
            b"gin_rummy" => Some(Self::GinRummy),
            b"stratego" => Some(Self::Stratego),
            b"ofc_chinese_poker" => Some(Self::OfcChinesePoker),
            b"spades" => Some(Self::Spades),
            b"dou_di_zhu" => Some(Self::DouDiZhu),
            b"pusoy_dos" => Some(Self::PusoyDos),
            b"tien_len" => Some(Self::TienLen),
            b"call_break" => Some(Self::CallBreak),
            b"backgammon" => Some(Self::Backgammon),
            b"hearts" => Some(Self::Hearts),
            b"cribbage" => Some(Self::Cribbage),
            _ => {
                // Try to parse as UTF-8 string for custom types
                String::from_utf8(bytes.to_vec()).ok().map(Self::Custom)
            }
        }
    }

    /// Convert the game type to a canonical byte representation.
    ///
    /// # Examples
    ///
    /// ```
    /// use myosu_games::traits::GameType;
    ///
    /// assert_eq!(GameType::NlheHeadsUp.to_bytes(), b"nlhe_hu".to_vec());
    /// assert_eq!(GameType::NlheSixMax.to_bytes(), b"nlhe_6max".to_vec());
    /// assert_eq!(GameType::KuhnPoker.to_bytes(), b"kuhn_poker".to_vec());
    /// ```
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Self::NlheHeadsUp => b"nlhe_hu".to_vec(),
            Self::NlheSixMax => b"nlhe_6max".to_vec(),
            Self::KuhnPoker => b"kuhn_poker".to_vec(),
            Self::LiarsDice => b"liars_dice".to_vec(),
            Self::Plo => b"plo".to_vec(),
            Self::NlheTournament => b"nlhe_tournament".to_vec(),
            Self::ShortDeck => b"short_deck".to_vec(),
            Self::TeenPatti => b"teen_patti".to_vec(),
            Self::HanafudaKoiKoi => b"hanafuda_koi_koi".to_vec(),
            Self::HwatuGoStop => b"hwatu_go_stop".to_vec(),
            Self::RiichiMahjong => b"riichi_mahjong".to_vec(),
            Self::Bridge => b"bridge".to_vec(),
            Self::GinRummy => b"gin_rummy".to_vec(),
            Self::Stratego => b"stratego".to_vec(),
            Self::OfcChinesePoker => b"ofc_chinese_poker".to_vec(),
            Self::Spades => b"spades".to_vec(),
            Self::DouDiZhu => b"dou_di_zhu".to_vec(),
            Self::PusoyDos => b"pusoy_dos".to_vec(),
            Self::TienLen => b"tien_len".to_vec(),
            Self::CallBreak => b"call_break".to_vec(),
            Self::Backgammon => b"backgammon".to_vec(),
            Self::Hearts => b"hearts".to_vec(),
            Self::Cribbage => b"cribbage".to_vec(),
            Self::Custom(s) => s.as_bytes().to_vec(),
        }
    }

    /// Return the default number of players for this game type.
    ///
    /// # Examples
    ///
    /// ```
    /// use myosu_games::traits::GameType;
    ///
    /// assert_eq!(GameType::NlheHeadsUp.num_players(), 2);
    /// assert_eq!(GameType::NlheSixMax.num_players(), 6);
    /// assert_eq!(GameType::KuhnPoker.num_players(), 2);
    /// assert_eq!(GameType::LiarsDice.num_players(), 2);
    /// ```
    pub fn num_players(&self) -> u8 {
        match self {
            Self::NlheHeadsUp => 2,
            Self::NlheSixMax => 6,
            Self::KuhnPoker => 2,
            Self::LiarsDice => 2,
            Self::Plo => 6,
            Self::NlheTournament => 9,
            Self::ShortDeck => 6,
            Self::TeenPatti => 6,
            Self::HanafudaKoiKoi => 2,
            Self::HwatuGoStop => 3,
            Self::RiichiMahjong => 4,
            Self::Bridge => 4,
            Self::GinRummy => 2,
            Self::Stratego => 2,
            Self::OfcChinesePoker => 3,
            Self::Spades => 4,
            Self::DouDiZhu => 3,
            Self::PusoyDos => 4,
            Self::TienLen => 4,
            Self::CallBreak => 4,
            Self::Backgammon => 2,
            Self::Hearts => 4,
            Self::Cribbage => 2,
            // Default to 2 for custom games
            Self::Custom(_) => 2,
        }
    }
}

/// Game-specific parameters for different game types.
///
/// This enum provides compile-time validation for known games while
/// allowing extensibility via the `Custom` variant for new game types.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case", tag = "game", content = "params")]
#[non_exhaustive]
pub enum GameParams {
    /// No-Limit Hold'em parameters
    NlheHeadsUp {
        /// Starting stack size in big blinds
        stack_bb: u32,
        /// Optional ante size in big blinds (None for no ante)
        #[serde(skip_serializing_if = "Option::is_none")]
        ante_bb: Option<u32>,
    },
    /// Liar's Dice parameters
    LiarsDice {
        /// Number of dice per player
        num_dice: u8,
        /// Number of faces on each die
        num_faces: u8,
    },
    /// Standard two-player Kuhn poker parameters
    KuhnPoker,
    /// Custom parameters as opaque JSON
    Custom(serde_json::Value),
}

/// Query sent to a miner to request a strategy for a given information set.
///
/// Generic over the game's Info type for type safety within a specific
/// game implementation, but serializable for network transport.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct StrategyQuery<I: Serialize> {
    /// The information set (game state from the acting player's perspective)
    pub info: I,
}

impl<I: Serialize> StrategyQuery<I> {
    /// Create a new strategy query
    pub fn new(info: I) -> Self {
        Self { info }
    }
}

/// Response from a miner containing action probabilities for a given information set.
///
/// The `actions` vector contains pairs of (action, probability) where probabilities
/// sum to 1.0 (or very close, within floating-point tolerance).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct StrategyResponse<E: Serialize> {
    /// Action distribution: list of (action, probability) pairs
    pub actions: Vec<(E, Probability)>,
}

impl<E: Serialize> StrategyResponse<E> {
    /// Create a new strategy response
    pub fn new(actions: Vec<(E, Probability)>) -> Self {
        Self { actions }
    }

    /// Verify that action probabilities sum to approximately 1.0
    ///
    /// Returns true if the sum is within epsilon (0.001) of 1.0.
    /// Empty action lists are considered valid (terminal states have no actions).
    pub fn is_valid(&self) -> bool {
        if self.actions.is_empty() {
            return true;
        }
        let sum: Probability = self.actions.iter().map(|(_, p)| p).sum();
        (sum - 1.0).abs() < 0.001
    }

    /// Get the probability for a specific action
    ///
    /// Returns 0.0 if the action is not in the distribution.
    pub fn probability_for(&self, action: &E) -> Probability
    where
        E: PartialEq,
    {
        self.actions
            .iter()
            .find(|(a, _)| a == action)
            .map(|(_, p)| *p)
            .unwrap_or(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn reexports_compile() {
        // Just verify that the re-exports are available
        let _: Probability = 0.5;
        let _: Utility = 1.0;
    }

    #[test]
    fn game_config_serializes() {
        let config = GameConfig::nlhe_heads_up(100);
        let json = serde_json::to_string(&config).expect("should serialize");
        let deserialized: GameConfig = serde_json::from_str(&json).expect("should deserialize");
        assert_eq!(config, deserialized);
    }

    #[test]
    fn game_config_nlhe_params() {
        let config = GameConfig {
            game_type: GameType::NlheHeadsUp,
            num_players: 2,
            params: GameParams::NlheHeadsUp {
                stack_bb: 200,
                ante_bb: Some(1),
            },
        };

        let json = serde_json::to_string_pretty(&config).expect("should serialize");
        assert!(json.contains("nlhe_heads_up"));
        assert!(json.contains("200"));
    }

    #[test]
    fn game_type_from_bytes_known() {
        assert_eq!(
            GameType::from_bytes(b"nlhe_hu"),
            Some(GameType::NlheHeadsUp)
        );
        assert_eq!(
            GameType::from_bytes(b"nlhe_6max"),
            Some(GameType::NlheSixMax)
        );
        assert_eq!(
            GameType::from_bytes(b"kuhn_poker"),
            Some(GameType::KuhnPoker)
        );
        assert_eq!(
            GameType::from_bytes(b"liars_dice"),
            Some(GameType::LiarsDice)
        );
        assert_eq!(GameType::from_bytes(b"bridge"), Some(GameType::Bridge));
        assert_eq!(GameType::from_bytes(b"cribbage"), Some(GameType::Cribbage));
    }

    #[test]
    fn game_type_from_bytes_custom() {
        assert_eq!(
            GameType::from_bytes(b"custom_game"),
            Some(GameType::Custom("custom_game".to_string()))
        );
    }

    #[test]
    fn game_type_to_bytes_roundtrip() {
        let types = vec![
            GameType::NlheHeadsUp,
            GameType::NlheSixMax,
            GameType::KuhnPoker,
            GameType::LiarsDice,
            GameType::Bridge,
            GameType::Cribbage,
            GameType::Custom("my_game".to_string()),
        ];

        for gt in types {
            let bytes = gt.to_bytes();
            let recovered = GameType::from_bytes(&bytes).expect("should parse");
            assert_eq!(gt, recovered);
        }
    }

    #[test]
    fn game_type_unicode_custom_roundtrips() {
        let game_type = GameType::Custom("liars_dice_묘수".to_string());
        let bytes = game_type.to_bytes();
        let recovered = GameType::from_bytes(&bytes).expect("unicode custom type should parse");

        assert_eq!(recovered, game_type);
    }

    #[test]
    fn game_type_num_players() {
        assert_eq!(GameType::NlheHeadsUp.num_players(), 2);
        assert_eq!(GameType::NlheSixMax.num_players(), 6);
        assert_eq!(GameType::KuhnPoker.num_players(), 2);
        assert_eq!(GameType::LiarsDice.num_players(), 2);
        assert_eq!(GameType::Bridge.num_players(), 4);
        assert_eq!(GameType::Cribbage.num_players(), 2);
        assert_eq!(GameType::Custom("anything".to_string()).num_players(), 2);
    }

    #[test]
    fn strategy_query_response_roundtrip() {
        // Using simple types for testing
        #[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
        struct MockInfo {
            player: u8,
            observation: String,
        }

        #[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
        enum MockAction {
            Fold,
            Call,
            Raise,
        }

        let query = StrategyQuery::new(MockInfo {
            player: 0,
            observation: "test".to_string(),
        });

        let query_json = serde_json::to_string(&query).expect("should serialize query");
        let query_deserialized: StrategyQuery<MockInfo> =
            serde_json::from_str(&query_json).expect("should deserialize query");
        assert_eq!(query.info, query_deserialized.info);

        let response = StrategyResponse::new(vec![
            (MockAction::Fold, 0.2),
            (MockAction::Call, 0.5),
            (MockAction::Raise, 0.3),
        ]);

        let response_json = serde_json::to_string(&response).expect("should serialize response");
        let response_deserialized: StrategyResponse<MockAction> =
            serde_json::from_str(&response_json).expect("should deserialize response");
        assert_eq!(response.actions.len(), response_deserialized.actions.len());
    }

    #[test]
    fn strategy_response_validates() {
        // Valid: probabilities sum to 1.0
        let valid = StrategyResponse::new(vec![('a', 0.5), ('b', 0.5)]);
        assert!(valid.is_valid());

        // Valid: empty (terminal state)
        let empty = StrategyResponse::<char>::new(vec![]);
        assert!(empty.is_valid());

        // Valid: within epsilon of 1.0
        let near_one = StrategyResponse::new(vec![('a', 0.3333), ('b', 0.3333), ('c', 0.3334)]);
        assert!(near_one.is_valid());

        // Invalid: sums to wrong value
        let invalid = StrategyResponse::new(vec![('a', 0.5), ('b', 0.3)]);
        assert!(!invalid.is_valid());
    }

    #[test]
    fn strategy_response_probability_for() {
        let response = StrategyResponse::new(vec![('a', 0.5), ('b', 0.3), ('c', 0.2)]);

        assert_eq!(response.probability_for(&'a'), 0.5);
        assert_eq!(response.probability_for(&'b'), 0.3);
        assert_eq!(response.probability_for(&'c'), 0.2);
        assert_eq!(response.probability_for(&'z'), 0.0); // not present
    }

    #[test]
    fn strategy_response_zero_probability_edges_can_still_be_valid() {
        let response = StrategyResponse::new(vec![('a', 0.0), ('b', 1.0), ('c', 0.0)]);

        assert!(response.is_valid());
        assert_eq!(response.probability_for(&'a'), 0.0);
        assert_eq!(response.probability_for(&'b'), 1.0);
        assert_eq!(response.probability_for(&'c'), 0.0);
    }

    fn arb_game_type() -> impl Strategy<Value = GameType> {
        prop_oneof![
            Just(GameType::NlheHeadsUp),
            Just(GameType::NlheSixMax),
            Just(GameType::KuhnPoker),
            Just(GameType::LiarsDice),
            "[a-z0-9_]{1,16}".prop_map(GameType::Custom),
        ]
    }

    fn arb_game_config() -> impl Strategy<Value = GameConfig> {
        prop_oneof![
            (1u32..=10_000u32, proptest::option::of(0u32..=100u32)).prop_map(
                |(stack_bb, ante_bb)| GameConfig::new(
                    GameType::NlheHeadsUp,
                    2,
                    GameParams::NlheHeadsUp { stack_bb, ante_bb },
                ),
            ),
            (1u8..=8u8, 2u8..=20u8).prop_map(|(num_dice, num_faces)| {
                GameConfig::new(
                    GameType::LiarsDice,
                    2,
                    GameParams::LiarsDice {
                        num_dice,
                        num_faces,
                    },
                )
            }),
            Just(GameConfig::new(
                GameType::KuhnPoker,
                2,
                GameParams::KuhnPoker,
            )),
            "[a-z0-9_]{1,16}".prop_map(|name| {
                GameConfig::new(
                    GameType::Custom(name.clone()),
                    2,
                    GameParams::Custom(serde_json::json!({ "game": name })),
                )
            }),
        ]
    }

    fn arb_strategy_response() -> impl Strategy<Value = StrategyResponse<String>> {
        prop::collection::vec(("[a-z]{1,12}", 0.0f64..=1.0f64), 0..=8).prop_map(|actions| {
            let actions = actions
                .into_iter()
                .map(|(action, probability)| (action, probability as Probability))
                .collect();
            StrategyResponse::new(actions)
        })
    }

    proptest! {
        #[test]
        fn serialization_roundtrip_game_type(game_type in arb_game_type()) {
            let bytes = game_type.to_bytes();
            let decoded = GameType::from_bytes(&bytes);
            prop_assert_eq!(decoded, Some(game_type));
        }

        #[test]
        fn serialization_roundtrip_game_config(config in arb_game_config()) {
            let json = serde_json::to_string(&config).expect("config should serialize");
            let decoded: GameConfig =
                serde_json::from_str(&json).expect("config should deserialize");
            prop_assert_eq!(decoded, config);
        }

        #[test]
        fn serialization_roundtrip_strategy_response(
            response in arb_strategy_response()
        ) {
            let json = serde_json::to_string(&response)
                .expect("response should serialize");
            let decoded: StrategyResponse<String> =
                serde_json::from_str(&json).expect("response should deserialize");
            prop_assert_eq!(decoded, response);
        }
    }
}

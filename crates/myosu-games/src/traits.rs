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
}

/// Known game types supported by myosu.
///
/// The `Custom` variant allows for extensibility without code changes.
/// New game types can be registered on-chain using a unique string identifier.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum GameType {
    /// No-Limit Hold'em heads-up (2 players)
    NlheHeadsUp,
    /// No-Limit Hold'em 6-max
    NlheSixMax,
    /// Liar's Dice
    LiarsDice,
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
    /// assert_eq!(GameType::from_bytes(b"liars_dice"), Some(GameType::LiarsDice));
    /// assert_eq!(GameType::from_bytes(b"unknown"), Some(GameType::Custom("unknown".to_string())));
    /// ```
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        match bytes {
            b"nlhe_hu" => Some(Self::NlheHeadsUp),
            b"nlhe_6max" => Some(Self::NlheSixMax),
            b"liars_dice" => Some(Self::LiarsDice),
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
    /// ```
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Self::NlheHeadsUp => b"nlhe_hu".to_vec(),
            Self::NlheSixMax => b"nlhe_6max".to_vec(),
            Self::LiarsDice => b"liars_dice".to_vec(),
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
    /// assert_eq!(GameType::LiarsDice.num_players(), 2);
    /// ```
    pub fn num_players(&self) -> u8 {
        match self {
            Self::NlheHeadsUp => 2,
            Self::NlheSixMax => 6,
            Self::LiarsDice => 2,
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
            GameType::from_bytes(b"liars_dice"),
            Some(GameType::LiarsDice)
        );
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
            GameType::LiarsDice,
            GameType::Custom("my_game".to_string()),
        ];

        for gt in types {
            let bytes = gt.to_bytes();
            let recovered = GameType::from_bytes(&bytes).expect("should parse");
            assert_eq!(gt, recovered);
        }
    }

    #[test]
    fn game_type_num_players() {
        assert_eq!(GameType::NlheHeadsUp.num_players(), 2);
        assert_eq!(GameType::NlheSixMax.num_players(), 6);
        assert_eq!(GameType::LiarsDice.num_players(), 2);
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
}

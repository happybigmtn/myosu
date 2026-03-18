//! Game State JSON Schema — Universal machine-readable game state for all 20 games.
//!
//! This module defines the JSON schema for game state that enables programmatic
//! agents (LLMs, bots, scripts) to parse and act in any myosu game. The schema
//! is game-agnostic at the top level but game-specific in the `state` field.
//!
//! ## Key Contract: Exhaustive `legal_actions`
//!
//! The `legal_actions` array is EXHAUSTIVE. An agent never needs to guess
//! what's legal. Every valid action is enumerated with its parameters.
//!
//! ## Example NLHE State
//!
//! ```json
//! {
//!   "game_type": "nlhe_hu",
//!   "hand_number": 47,
//!   "phase": "action",
//!   "state": {
//!     "board": ["Ts", "7h", "2c"],
//!     "your_hand": ["As", "Kh"],
//!     "your_stack": 94,
//!     "your_position": "BB",
//!     "opponents": [
//!       {"seat": "SB", "stack": 94, "hand": null}
//!     ],
//!     "pot": 12,
//!     "to_act": "you",
//!     "last_action": {"player": "SB", "action": "raise", "amount": 6}
//!   },
//!   "legal_actions": [
//!     {"action": "fold"},
//!     {"action": "call", "amount": 6},
//!     {"action": "raise", "min": 12, "max": 94},
//!     {"action": "shove", "amount": 94}
//!   ],
//!   "meta": {
//!     "solver_source": "miner-12",
//!     "solver_exploitability": 13.2,
//!     "subnet_id": 1
//!   }
//! }
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The universal game state structure.
///
/// This struct represents the complete state of a game at any point,
/// designed to be fully serializable to JSON for agent consumption.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct GameState {
    /// Game type identifier (e.g., "nlhe_hu", "liars_dice", "riichi")
    pub game_type: String,

    /// Hand/round number (0-indexed or 1-indexed, game-dependent)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hand_number: Option<u32>,

    /// Current game phase
    pub phase: GamePhase,

    /// Game-specific state (board, hands, stacks, etc.)
    #[serde(flatten)]
    pub state_wrapper: GameStateWrapper,

    /// Exhaustive list of legal actions available to the current player
    pub legal_actions: Vec<LegalAction>,

    /// Metadata about the game session and solver
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<MetaInfo>,
}

/// Wrapper to handle the game-specific state field.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct GameStateWrapper {
    /// Game-specific state payload
    pub state: serde_json::Value,
}

/// Game phase enumeration.
///
/// These phases are common across most games. Individual games may
/// have additional phases encoded in their state.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum GamePhase {
    /// Waiting for game to start
    Waiting,
    /// Active play, waiting for a player decision
    Action,
    /// Betting round in progress (poker-specific)
    Betting,
    /// Showdown/reveal phase
    Showdown,
    /// Hand/round complete, finalizing
    Complete,
    /// Game over, session ended
    Ended,
    /// Custom phase for game-specific states
    #[serde(untagged)]
    Custom(String),
}

/// A legal action with its parameters.
///
/// This enum covers all action types across all 20 games. Each variant
/// includes the parameters needed to execute that action.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum LegalAction {
    /// Fold (give up the hand)
    Fold,

    /// Call (match the current bet)
    Call {
        /// Amount to call (in game units, usually big blinds for poker)
        amount: u32,
    },

    /// Check (pass when no bet to call)
    Check,

    /// Raise (increase the bet)
    Raise {
        /// Minimum raise amount
        min: u32,
        /// Maximum raise amount
        max: u32,
    },

    /// Bet (make the first bet in a round)
    Bet {
        /// Minimum bet amount
        min: u32,
        /// Maximum bet amount
        max: u32,
    },

    /// Shove/All-in (bet all remaining chips)
    Shove {
        /// Total amount being shoved
        amount: u32,
    },

    /// Discard a tile/card (Mahjong, card games)
    Discard {
        /// The item to discard (tile notation like "4s", "1m")
        item: String,
    },

    /// Declare riichi (Mahjong)
    Riichi {
        /// Tile to discard when declaring riichi
        #[serde(skip_serializing_if = "Option::is_none")]
        discard: Option<String>,
    },

    /// Declare tsumo (Mahjong self-draw win)
    Tsumo,

    /// Declare ron (Mahjong claim win)
    Ron,

    /// Pass (skip an optional action)
    Pass,

    /// Bid (Liar's Dice, auction games)
    Bid {
        /// Quantity being bid
        quantity: u8,
        /// Face value being bid (1-6 for dice)
        face: u8,
    },

    /// Challenge the previous bid (Liar's Dice)
    Challenge,

    /// Play cards from hand
    Play {
        /// Cards/tiles being played
        cards: Vec<String>,
    },

    /// Draw a card/tile
    Draw,

    /// Custom action for game-specific moves
    #[serde(untagged)]
    Custom {
        /// Action type identifier
        #[serde(rename = "action")]
        action_type: String,
        /// Additional parameters as key-value pairs
        #[serde(flatten)]
        params: HashMap<String, serde_json::Value>,
    },
}

/// Metadata about the game session.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct MetaInfo {
    /// Source of the solver strategy (e.g., "miner-12", "blueprint", "heuristic")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub solver_source: Option<String>,

    /// Solver exploitability in millibig blinds per hand (lower is better)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub solver_exploitability: Option<f64>,

    /// Subnet ID on the myosu chain
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subnet_id: Option<u16>,

    /// Miner UID that served this strategy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub miner_uid: Option<u16>,

    /// Block height when this strategy was published
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_height: Option<u64>,

    /// Additional metadata as key-value pairs
    #[serde(flatten, skip_serializing_if = "HashMap::is_empty")]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Opponent information (for games with hidden information).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct OpponentInfo {
    /// Seat/position identifier (e.g., "SB", "BB", "east", "south")
    pub seat: String,

    /// Current stack/chip count (if visible)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack: Option<u32>,

    /// Number of tiles/cards in hand (when hidden)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hand_count: Option<u8>,

    /// Visible hand (when revealed or in showdown)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hand: Option<Vec<String>>,

    /// Recent discards/actions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discards: Option<Vec<String>>,

    /// Whether this opponent is in riichi (Mahjong)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub riichi: Option<bool>,
}

/// Last action taken in the game.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct LastAction {
    /// Player who took the action ("you" or seat identifier)
    pub player: String,
    /// Action type
    pub action: String,
    /// Action amount (for bet/raise/call)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<u32>,
    /// Additional action data
    #[serde(flatten, skip_serializing_if = "HashMap::is_empty")]
    pub extra: HashMap<String, serde_json::Value>,
}

/// NLHE-specific game state.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct NlheState {
    /// Community cards (0-5 cards in standard notation like "As", "Th", "2c")
    pub board: Vec<String>,

    /// Hero's hole cards
    pub your_hand: Vec<String>,

    /// Hero's remaining stack (in big blinds)
    pub your_stack: u32,

    /// Hero's position ("SB" or "BB" for heads-up)
    pub your_position: String,

    /// Opponent information
    pub opponents: Vec<OpponentInfo>,

    /// Current pot size (in big blinds)
    pub pot: u32,

    /// Whose turn it is ("you" or seat identifier)
    pub to_act: String,

    /// Last action taken
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_action: Option<LastAction>,

    /// Amount needed to call (0 if no bet to call)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_call: Option<u32>,

    /// Hero's hand strength description (e.g., "top pair", "flush draw")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hand_strength: Option<String>,

    /// Current street ("preflop", "flop", "turn", "river")
    pub street: String,
}

/// Mahjong (Riichi) specific game state.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct RiichiState {
    /// Your hand tiles (e.g., ["1m", "2m", "3m", "5p", "Ew"])
    pub your_hand: Vec<String>,

    /// Most recent draw (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub draw: Option<String>,

    /// Opponent information
    pub opponents: Vec<OpponentInfo>,

    /// Dora indicator tiles
    pub dora: Vec<String>,

    /// Whether you are in riichi
    #[serde(skip_serializing_if = "Option::is_none")]
    pub riichi: Option<bool>,

    /// Your current points
    pub points: i32,

    /// Current round (e.g., "East 1", "South 2")
    pub round: String,

    /// Number of tiles remaining in the wall
    pub wall_tiles: u8,
}

/// Liar's Dice specific game state.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct LiarsDiceState {
    /// Your dice values (hidden from opponents)
    pub your_dice: Vec<u8>,

    /// Number of dice per opponent (hidden values)
    pub opponent_dice_counts: Vec<u8>,

    /// Current bid (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_bid: Option<BidInfo>,

    /// Whose turn it is
    pub to_act: String,

    /// Total dice remaining in the game
    pub total_dice: u8,
}

/// Bid information for Liar's Dice.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct BidInfo {
    /// Player who made the bid
    pub player: String,
    /// Quantity bid
    pub quantity: u8,
    /// Face value bid (1-6)
    pub face: u8,
}

/// Action submitted by an agent.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum AgentAction {
    /// Fold
    Fold,
    /// Call
    Call,
    /// Check
    Check,
    /// Raise with amount
    Raise { amount: u32 },
    /// Bet with amount
    Bet { amount: u32 },
    /// All-in
    Shove,
    /// Discard a tile/card
    Discard { tile: String },
    /// Declare riichi
    Riichi,
    /// Tsumo (self-draw win)
    Tsumo,
    /// Ron (claim win)
    Ron,
    /// Pass
    Pass,
    /// Bid in Liar's Dice
    Bid { quantity: u8, face: u8 },
    /// Challenge in Liar's Dice
    Challenge,
    /// Play cards
    Play { cards: Vec<String> },
    /// Custom action
    #[serde(untagged)]
    Custom(serde_json::Value),
}

/// Error response when an invalid action is submitted.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct ActionError {
    /// Error code
    pub error: String,
    /// Human-readable error message
    pub message: String,
    /// Current legal actions (so agent can retry)
    pub legal_actions: Vec<LegalAction>,
}

impl ActionError {
    /// Create a new action error
    pub fn new(
        error: impl Into<String>,
        message: impl Into<String>,
        legal_actions: Vec<LegalAction>,
    ) -> Self {
        Self {
            error: error.into(),
            message: message.into(),
            legal_actions,
        }
    }
}

/// Builder for creating game state instances.
#[derive(Clone, Debug, Default)]
pub struct GameStateBuilder {
    game_type: String,
    hand_number: Option<u32>,
    phase: Option<GamePhase>,
    state: Option<serde_json::Value>,
    legal_actions: Vec<LegalAction>,
    meta: Option<MetaInfo>,
}

impl GameStateBuilder {
    /// Create a new builder for the given game type
    pub fn new(game_type: impl Into<String>) -> Self {
        Self {
            game_type: game_type.into(),
            ..Default::default()
        }
    }

    /// Set the hand number
    pub fn hand_number(mut self, num: u32) -> Self {
        self.hand_number = Some(num);
        self
    }

    /// Set the game phase
    pub fn phase(mut self, phase: GamePhase) -> Self {
        self.phase = Some(phase);
        self
    }

    /// Set the game-specific state
    pub fn state(mut self, state: impl Serialize) -> Result<Self, serde_json::Error> {
        self.state = Some(serde_json::to_value(state)?);
        Ok(self)
    }

    /// Set the game-specific state from a JSON value
    pub fn state_value(mut self, state: serde_json::Value) -> Self {
        self.state = Some(state);
        self
    }

    /// Add a legal action
    pub fn legal_action(mut self, action: LegalAction) -> Self {
        self.legal_actions.push(action);
        self
    }

    /// Set all legal actions
    pub fn legal_actions(mut self, actions: Vec<LegalAction>) -> Self {
        self.legal_actions = actions;
        self
    }

    /// Set metadata
    pub fn meta(mut self, meta: MetaInfo) -> Self {
        self.meta = Some(meta);
        self
    }

    /// Build the GameState
    pub fn build(self) -> Result<GameState, SchemaError> {
        Ok(GameState {
            game_type: self.game_type,
            hand_number: self.hand_number,
            phase: self.phase.ok_or(SchemaError::MissingPhase)?,
            state_wrapper: GameStateWrapper {
                state: self.state.ok_or(SchemaError::MissingState)?,
            },
            legal_actions: self.legal_actions,
            meta: self.meta,
        })
    }
}

/// Errors that can occur when building game state.
#[derive(Clone, Debug, PartialEq)]
pub enum SchemaError {
    /// Missing required phase field
    MissingPhase,
    /// Missing required state field
    MissingState,
}

impl std::fmt::Display for SchemaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SchemaError::MissingPhase => write!(f, "GameState requires a phase"),
            SchemaError::MissingState => write!(f, "GameState requires a state"),
        }
    }
}

impl std::error::Error for SchemaError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nlhe_state_serializes() {
        let state = NlheState {
            board: vec!["Ts".to_string(), "7h".to_string(), "2c".to_string()],
            your_hand: vec!["As".to_string(), "Kh".to_string()],
            your_stack: 94,
            your_position: "BB".to_string(),
            opponents: vec![OpponentInfo {
                seat: "SB".to_string(),
                stack: Some(94),
                hand_count: None,
                hand: None,
                discards: None,
                riichi: None,
            }],
            pot: 12,
            to_act: "you".to_string(),
            last_action: Some(LastAction {
                player: "SB".to_string(),
                action: "raise".to_string(),
                amount: Some(6),
                extra: HashMap::new(),
            }),
            to_call: Some(6),
            hand_strength: Some("top pair".to_string()),
            street: "flop".to_string(),
        };

        let game_state = GameStateBuilder::new("nlhe_hu")
            .hand_number(47)
            .phase(GamePhase::Action)
            .state(&state)
            .unwrap()
            .legal_action(LegalAction::Fold)
            .legal_action(LegalAction::Call { amount: 6 })
            .legal_action(LegalAction::Raise { min: 12, max: 94 })
            .legal_action(LegalAction::Shove { amount: 94 })
            .meta(MetaInfo {
                solver_source: Some("miner-12".to_string()),
                solver_exploitability: Some(13.2),
                subnet_id: Some(1),
                miner_uid: None,
                block_height: None,
                extra: HashMap::new(),
            })
            .build()
            .unwrap();

        let json = serde_json::to_string_pretty(&game_state).expect("should serialize");

        // Verify key fields are present
        assert!(json.contains("nlhe_hu"));
        assert!(json.contains("Ts"));
        assert!(json.contains("As"));
        assert!(json.contains("top pair"));
        assert!(json.contains("fold"));
        assert!(json.contains("call"));
        assert!(json.contains("raise"));
        assert!(json.contains("miner-12"));
        assert!(json.contains("13.2"));

        // Verify it deserializes back
        let deserialized: GameState = serde_json::from_str(&json).expect("should deserialize");
        assert_eq!(deserialized.game_type, "nlhe_hu");
        assert_eq!(deserialized.hand_number, Some(47));
    }

    #[test]
    fn legal_actions_exhaustive() {
        // Test that all legal action variants serialize correctly
        let actions = vec![
            LegalAction::Fold,
            LegalAction::Call { amount: 10 },
            LegalAction::Check,
            LegalAction::Raise { min: 20, max: 100 },
            LegalAction::Bet { min: 10, max: 100 },
            LegalAction::Shove { amount: 94 },
            LegalAction::Discard {
                item: "4s".to_string(),
            },
            LegalAction::Riichi {
                discard: Some("1m".to_string()),
            },
            LegalAction::Tsumo,
            LegalAction::Ron,
            LegalAction::Pass,
            LegalAction::Bid {
                quantity: 3,
                face: 5,
            },
            LegalAction::Challenge,
            LegalAction::Play {
                cards: vec!["2h".to_string(), "2d".to_string()],
            },
            LegalAction::Draw,
        ];

        for action in &actions {
            let json = serde_json::to_string(action).expect("should serialize");
            let deserialized: LegalAction =
                serde_json::from_str(&json).expect("should deserialize");
            assert_eq!(*action, deserialized);
        }
    }

    #[test]
    fn all_game_types_have_schema() {
        // Verify that the schema supports all planned game types
        let game_types = vec![
            "nlhe_hu",
            "nlhe_6max",
            "short_deck",
            "plo_hu",
            "liars_dice",
            "riichi",
            "backgammon",
            "bridge",
            "hanabi",
            "leduc",
        ];

        for game_type in game_types {
            let state = serde_json::json!({
                "test_field": "test_value",
                "game": game_type
            });

            let game_state = GameStateBuilder::new(game_type)
                .phase(GamePhase::Waiting)
                .state_value(state)
                .legal_action(LegalAction::Pass)
                .build()
                .unwrap();

            let json = serde_json::to_string(&game_state).expect("should serialize");
            assert!(json.contains(game_type));
        }
    }

    #[test]
    fn riichi_state_example() {
        let state = RiichiState {
            your_hand: vec![
                "1m".to_string(),
                "2m".to_string(),
                "3m".to_string(),
                "5p".to_string(),
                "6p".to_string(),
                "7p".to_string(),
                "3s".to_string(),
                "4s".to_string(),
                "9s".to_string(),
                "9s".to_string(),
                "Ew".to_string(),
                "Ew".to_string(),
            ],
            draw: Some("5s".to_string()),
            opponents: vec![
                OpponentInfo {
                    seat: "south".to_string(),
                    stack: None,
                    hand_count: Some(13),
                    hand: None,
                    discards: Some(vec![
                        "1m".to_string(),
                        "9p".to_string(),
                        "5s".to_string(),
                        "Nw".to_string(),
                    ]),
                    riichi: Some(false),
                },
                OpponentInfo {
                    seat: "west".to_string(),
                    stack: None,
                    hand_count: Some(11),
                    hand: None,
                    discards: Some(vec!["2m".to_string(), "3p".to_string(), "7s".to_string()]),
                    riichi: Some(false),
                },
                OpponentInfo {
                    seat: "north".to_string(),
                    stack: None,
                    hand_count: Some(13),
                    hand: None,
                    discards: Some(vec!["4p".to_string()]),
                    riichi: Some(false),
                },
            ],
            dora: vec!["3m".to_string()],
            riichi: Some(false),
            points: 25000,
            round: "East 1".to_string(),
            wall_tiles: 70,
        };

        let game_state = GameStateBuilder::new("riichi")
            .phase(GamePhase::Action)
            .state(&state)
            .unwrap()
            .legal_action(LegalAction::Discard {
                item: "1m".to_string(),
            })
            .legal_action(LegalAction::Discard {
                item: "5s".to_string(),
            })
            .legal_action(LegalAction::Discard {
                item: "4s".to_string(),
            })
            .legal_action(LegalAction::Tsumo)
            .legal_action(LegalAction::Riichi {
                discard: Some("4s".to_string()),
            })
            .build()
            .unwrap();

        let json = serde_json::to_string_pretty(&game_state).unwrap();

        // Verify Mahjong-specific fields
        assert!(json.contains("1m"));
        assert!(json.contains("Ew"));
        assert!(json.contains("dora"));
        assert!(json.contains("East 1"));
        assert!(json.contains("tsumo"));
    }

    #[test]
    fn liars_dice_state_example() {
        let state = LiarsDiceState {
            your_dice: vec![2, 4, 4, 6, 1],
            opponent_dice_counts: vec![5, 5],
            current_bid: Some(BidInfo {
                player: "player_1".to_string(),
                quantity: 4,
                face: 3,
            }),
            to_act: "you".to_string(),
            total_dice: 15,
        };

        let game_state = GameStateBuilder::new("liars_dice")
            .phase(GamePhase::Action)
            .state(&state)
            .unwrap()
            .legal_action(LegalAction::Bid {
                quantity: 5,
                face: 3,
            })
            .legal_action(LegalAction::Bid {
                quantity: 4,
                face: 4,
            })
            .legal_action(LegalAction::Challenge)
            .build()
            .unwrap();

        let json = serde_json::to_string_pretty(&game_state).unwrap();
        assert!(json.contains("liars_dice"));
        assert!(json.contains("current_bid"));
        assert!(json.contains("challenge"));
    }

    #[test]
    fn valid_action_accepted() {
        let action = AgentAction::Raise { amount: 15 };
        let json = serde_json::to_string(&action).unwrap();
        assert_eq!(json, r#"{"action":"raise","amount":15}"#);

        let deserialized: AgentAction = serde_json::from_str(&json).unwrap();
        match deserialized {
            AgentAction::Raise { amount } => assert_eq!(amount, 15),
            _ => panic!("Expected Raise action"),
        }
    }

    #[test]
    fn invalid_action_returns_legal() {
        let legal_actions = vec![
            LegalAction::Fold,
            LegalAction::Call { amount: 6 },
            LegalAction::Raise { min: 12, max: 94 },
        ];

        let error = ActionError::new(
            "invalid_action",
            "raise amount must be between 12 and 94",
            legal_actions.clone(),
        );

        let json = serde_json::to_string_pretty(&error).unwrap();
        assert!(json.contains("invalid_action"));
        assert!(json.contains("raise amount must be"));
        assert!(json.contains("legal_actions"));

        // Verify legal_actions is present and has the right number of items
        let deserialized: ActionError = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.legal_actions.len(), 3);
    }

    #[test]
    fn all_action_types_roundtrip() {
        let actions = vec![
            AgentAction::Fold,
            AgentAction::Call,
            AgentAction::Check,
            AgentAction::Raise { amount: 20 },
            AgentAction::Bet { amount: 10 },
            AgentAction::Shove,
            AgentAction::Discard {
                tile: "5s".to_string(),
            },
            AgentAction::Riichi,
            AgentAction::Tsumo,
            AgentAction::Ron,
            AgentAction::Pass,
            AgentAction::Bid {
                quantity: 4,
                face: 3,
            },
            AgentAction::Challenge,
            AgentAction::Play {
                cards: vec!["Ah".to_string(), "Kh".to_string()],
            },
        ];

        for original in &actions {
            let json = serde_json::to_string(original).expect("should serialize");
            let deserialized: AgentAction =
                serde_json::from_str(&json).expect("should deserialize");
            assert_eq!(*original, deserialized);
        }
    }

    #[test]
    fn builder_missing_phase_fails() {
        let result = GameStateBuilder::new("nlhe_hu")
            .state_value(serde_json::json!({"test": true}))
            .build();

        assert!(matches!(result, Err(SchemaError::MissingPhase)));
    }

    #[test]
    fn builder_missing_state_fails() {
        let result = GameStateBuilder::new("nlhe_hu")
            .phase(GamePhase::Waiting)
            .build();

        assert!(matches!(result, Err(SchemaError::MissingState)));
    }

    #[test]
    fn phase_roundtrip() {
        let phases = vec![
            GamePhase::Waiting,
            GamePhase::Action,
            GamePhase::Betting,
            GamePhase::Showdown,
            GamePhase::Complete,
            GamePhase::Ended,
            GamePhase::Custom("special_phase".to_string()),
        ];

        for phase in phases {
            let json = serde_json::to_string(&phase).unwrap();
            let deserialized: GamePhase = serde_json::from_str(&json).unwrap();
            assert_eq!(phase, deserialized);
        }
    }

    #[test]
    fn meta_info_serializes() {
        let meta = MetaInfo {
            solver_source: Some("miner-42".to_string()),
            solver_exploitability: Some(8.5),
            subnet_id: Some(7),
            miner_uid: Some(42),
            block_height: Some(1234567),
            extra: {
                let mut map = HashMap::new();
                map.insert("custom_key".to_string(), serde_json::json!("custom_value"));
                map
            },
        };

        let json = serde_json::to_string_pretty(&meta).unwrap();
        assert!(json.contains("miner-42"));
        assert!(json.contains("8.5"));
        assert!(json.contains("custom_key"));

        let deserialized: MetaInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.solver_source, Some("miner-42".to_string()));
    }
}

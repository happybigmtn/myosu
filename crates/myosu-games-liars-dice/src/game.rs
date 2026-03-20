//! Liar's Dice game state — `CfrGame` implementation.
//!
//! Implements the root node, action space, terminal detection,
//! and terminal utility computation for 2-player 1-die Liar's Dice.

/// LiarsDiceGame implements the CFR game tree root node.
///
/// State: `dice: [u8; 2]` (die faces for each player, 1-6),
/// `current_player: u8` (0 or 1), and bid history.
///
/// # Constraints
///
/// - `CfrGame: Copy` — game state must be `Copy` for MCCFR
/// - Bid history uses fixed-size array with sentinel to avoid heap allocation
/// - Max 12 bids for 1-die game (after which challenge is forced)
#[derive(Clone, Copy, Debug)]
pub struct LiarsDiceGame {
    /// Dice faces for each player (1-6), set at game start
    pub dice: [u8; 2],
    /// Current player to act (0 or 1)
    pub current_player: u8,
    /// Bid history — fixed-size array, filled slots are valid bids
    pub bids: [Option<Bid>; 12],
    /// Number of valid bids in the array
    pub num_bids: u8,
    /// Whether the game has ended
    pub terminal: bool,
}

/// A bid in the Liar's Dice game.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bid {
    /// Number of dice with the specified face (must be > previous quantity, or same quantity with higher face)
    pub quantity: u8,
    /// Face value being claimed (1-6)
    pub face: u8,
}

impl LiarsDiceGame {
    /// Create a new game with random dice rolls.
    pub fn new() -> Self {
        todo!("Slice 2: implement LiarsDiceGame::new() with dice rolling")
    }

    /// Create a game with given dice (for testing).
    pub fn with_dice(_dice: [u8; 2]) -> Self {
        todo!("Slice 2: implement LiarsDiceGame::with_dice()")
    }
}

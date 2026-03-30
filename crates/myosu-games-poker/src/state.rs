use serde::{Deserialize, Serialize};

use crate::action::NlheAction;

/// Current betting street.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NlheStreet {
    Preflop,
    Flop,
    Turn,
    River,
}

impl NlheStreet {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Preflop => "PREFLOP",
            Self::Flop => "FLOP",
            Self::Turn => "TURN",
            Self::River => "RIVER",
        }
    }
}

/// Acting perspective for the rendered hand.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NlheActor {
    Hero,
    Villain,
}

impl NlheActor {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Hero => "HERO",
            Self::Villain => "VILLAIN",
        }
    }
}

/// Table position label for the two-player wedge.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NlheTablePosition {
    Button,
    BigBlind,
}

impl NlheTablePosition {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Button => "BTN",
            Self::BigBlind => "BB",
        }
    }
}

/// Public player stack and contribution state.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct NlhePlayerState {
    pub label: String,
    pub position: NlheTablePosition,
    pub stack_bb: u32,
    pub committed_bb: u32,
}

impl NlhePlayerState {
    /// Create a new player state.
    pub fn new(label: impl Into<String>, position: NlheTablePosition, stack_bb: u32) -> Self {
        Self {
            label: label.into(),
            position,
            stack_bb,
            committed_bb: 0,
        }
    }
}

/// Renderable snapshot of an NLHE decision point.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct NlheSnapshot {
    pub hand_number: u32,
    pub street: NlheStreet,
    pub pot_bb: u32,
    pub board: Vec<String>,
    pub hero_hole: [String; 2],
    pub action_on: NlheActor,
    pub to_call_bb: u32,
    pub min_raise_to_bb: Option<u32>,
    pub legal_actions: Vec<NlheAction>,
    pub hero: NlhePlayerState,
    pub villain: NlhePlayerState,
}

impl NlheSnapshot {
    /// Return a compact header context for the hand.
    pub fn context_label(&self) -> String {
        format!("HAND {}", self.hand_number)
    }

    /// Return the board as a compact text string.
    pub fn board_label(&self) -> String {
        if self.board.is_empty() {
            return "--".to_string();
        }
        self.board.join(" ")
    }

    /// Return the legal actions as human-readable labels.
    pub fn action_labels(&self) -> Vec<String> {
        self.legal_actions.iter().map(ToString::to_string).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn board_label_collapses_empty_board() {
        let snapshot = sample_snapshot();
        assert_eq!(snapshot.board_label(), "--");
    }

    #[test]
    fn action_labels_follow_display() {
        let mut snapshot = sample_snapshot();
        snapshot.legal_actions = vec![NlheAction::Fold, NlheAction::RaiseTo { amount_bb: 18 }];
        assert_eq!(snapshot.action_labels(), vec!["fold", "raise 18"]);
    }

    fn sample_snapshot() -> NlheSnapshot {
        NlheSnapshot {
            hand_number: 17,
            street: NlheStreet::Preflop,
            pot_bb: 3,
            board: Vec::new(),
            hero_hole: ["Ac".to_string(), "Kh".to_string()],
            action_on: NlheActor::Hero,
            to_call_bb: 1,
            min_raise_to_bb: Some(6),
            legal_actions: vec![NlheAction::Fold, NlheAction::Call],
            hero: NlhePlayerState::new("Hero", NlheTablePosition::Button, 99),
            villain: NlhePlayerState::new("Villain", NlheTablePosition::BigBlind, 98),
        }
    }
}

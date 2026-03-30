use core::fmt;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Player action for a no-limit hold'em decision point.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NlheAction {
    Fold,
    Check,
    Call,
    Bet { amount_bb: u32 },
    RaiseTo { amount_bb: u32 },
    AllIn,
}

impl fmt::Display for NlheAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Fold => f.write_str("fold"),
            Self::Check => f.write_str("check"),
            Self::Call => f.write_str("call"),
            Self::Bet { amount_bb } => write!(f, "bet {amount_bb}"),
            Self::RaiseTo { amount_bb } => write!(f, "raise {amount_bb}"),
            Self::AllIn => f.write_str("all-in"),
        }
    }
}

/// Parse error for user-entered poker actions.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum ParseActionError {
    #[error("unknown action")]
    UnknownAction,
    #[error("missing amount for sized action")]
    MissingAmount,
    #[error("invalid amount")]
    InvalidAmount,
}

impl NlheAction {
    /// Parse a shell input string into a normalized poker action.
    pub fn parse(input: &str) -> Result<Self, ParseActionError> {
        let normalized = input.trim().to_ascii_lowercase();
        if normalized.is_empty() {
            return Err(ParseActionError::UnknownAction);
        }

        let mut parts = normalized.split_whitespace();
        let Some(head) = parts.next() else {
            return Err(ParseActionError::UnknownAction);
        };

        match head {
            "f" | "fold" => Ok(Self::Fold),
            "k" | "x" | "check" => Ok(Self::Check),
            "c" | "call" => Ok(Self::Call),
            "jam" | "allin" | "all-in" => Ok(Self::AllIn),
            "b" | "bet" => {
                let amount = parse_amount(parts.next())?;
                Ok(Self::Bet { amount_bb: amount })
            }
            "r" | "raise" => {
                let amount = parse_amount(parts.next())?;
                Ok(Self::RaiseTo { amount_bb: amount })
            }
            _ => Err(ParseActionError::UnknownAction),
        }
    }
}

fn parse_amount(value: Option<&str>) -> Result<u32, ParseActionError> {
    let Some(value) = value else {
        return Err(ParseActionError::MissingAmount);
    };

    value
        .parse::<u32>()
        .map_err(|_| ParseActionError::InvalidAmount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_actions() {
        assert_eq!(NlheAction::parse("f"), Ok(NlheAction::Fold));
        assert_eq!(NlheAction::parse("check"), Ok(NlheAction::Check));
        assert_eq!(NlheAction::parse("call"), Ok(NlheAction::Call));
        assert_eq!(NlheAction::parse("all-in"), Ok(NlheAction::AllIn));
    }

    #[test]
    fn parses_sized_actions() {
        assert_eq!(
            NlheAction::parse("bet 12"),
            Ok(NlheAction::Bet { amount_bb: 12 })
        );
        assert_eq!(
            NlheAction::parse("r 30"),
            Ok(NlheAction::RaiseTo { amount_bb: 30 })
        );
    }

    #[test]
    fn rejects_bad_inputs() {
        assert_eq!(NlheAction::parse(""), Err(ParseActionError::UnknownAction));
        assert_eq!(
            NlheAction::parse("raise"),
            Err(ParseActionError::MissingAmount)
        );
        assert_eq!(
            NlheAction::parse("bet nope"),
            Err(ParseActionError::InvalidAmount)
        );
    }
}

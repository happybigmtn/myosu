//! Canonical game truth types shared by strategy, validation, and replay code.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::traits::GameType;

/// Canonical registry entry for one game ruleset.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CanonicalGameSpec {
    pub game_type: GameType,
    pub slug: String,
    pub chain_id: String,
    pub ruleset_version: u32,
    pub display_name: String,
    pub default_players: u8,
    pub rule_file: Option<String>,
}

/// Canonical legal-action declaration for one game.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CanonicalActionSpec {
    pub game_id: String,
    pub action_id: String,
    pub family: String,
    pub display_label: String,
    pub legal_phases: Vec<String>,
    pub params_schema: Value,
}

/// Public state snapshot for one canonical decision point.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CanonicalStateSnapshot {
    pub game_id: String,
    pub ruleset_version: u32,
    pub trace_id: String,
    pub phase: String,
    pub actor: Option<u8>,
    pub public_state: Value,
    pub private_state_commitments: Vec<String>,
    pub legal_actions: Vec<CanonicalActionSpec>,
    pub terminal: bool,
}

/// Hash binding between a strategy query, strategy response, and engine tier.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CanonicalStrategyBinding {
    pub query_hash: String,
    pub response_hash: String,
    pub checkpoint_hash: Option<String>,
    pub engine_tier: String,
    pub engine_family: String,
    pub quality_summary: Option<String>,
}

/// Replayable transition record for one canonical action.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CanonicalTransitionTrace {
    pub trace_id: String,
    pub game_id: String,
    pub ruleset_version: u32,
    pub state_hash_before: String,
    pub action_id: String,
    pub action_params: Value,
    pub state_hash_after: String,
    pub strategy_binding: CanonicalStrategyBinding,
    pub payoff: Option<Vec<i64>>,
}

/// Errors returned by canonical truth helpers.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum CanonicalTruthError {
    #[error("failed to serialize canonical value: {message}")]
    Serialization { message: String },
    #[error("malformed canonical action id `{action_id}`")]
    MalformedActionId { action_id: String },
    #[error("duplicate canonical action id `{action_id}` for game `{game_id}`")]
    DuplicateActionId { game_id: String, action_id: String },
    #[error("canonical hash mismatch: expected `{expected}`, found `{found}`")]
    HashMismatch { expected: String, found: String },
    #[error("unsupported canonical game `{game_id}`")]
    UnsupportedGame { game_id: String },
    #[error("terminal payoff requested for non-terminal state `{trace_id}`")]
    NonTerminalPayoff { trace_id: String },
}

/// Produce a deterministic SHA-256 hash of a canonical serializable value.
pub fn canonical_hash<T: Serialize>(value: &T) -> Result<String, CanonicalTruthError> {
    let bytes = serde_json::to_vec(value).map_err(|source| CanonicalTruthError::Serialization {
        message: source.to_string(),
    })?;
    let digest = Sha256::digest(bytes);

    Ok(hex::encode(digest))
}

/// Validate the stable game-scoped canonical action-id grammar.
pub fn validate_action_id(action_id: &str) -> Result<(), CanonicalTruthError> {
    let has_scope_separator = action_id.contains('.');
    let has_only_allowed_chars = action_id
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || matches!(ch, '.' | '-' | '_'));
    if action_id.is_empty() || !has_scope_separator || !has_only_allowed_chars {
        return Err(CanonicalTruthError::MalformedActionId {
            action_id: action_id.to_string(),
        });
    }

    Ok(())
}

/// Verify that a canonical action list has valid, unique action IDs.
pub fn validate_unique_action_ids(
    game_id: &str,
    actions: &[CanonicalActionSpec],
) -> Result<(), CanonicalTruthError> {
    let mut seen = BTreeSet::new();
    for action in actions {
        validate_action_id(&action.action_id)?;
        if !seen.insert(action.action_id.as_str()) {
            return Err(CanonicalTruthError::DuplicateActionId {
                game_id: game_id.to_string(),
                action_id: action.action_id.clone(),
            });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::canonical::{
        CanonicalActionSpec, CanonicalGameSpec, CanonicalStrategyBinding, CanonicalTransitionTrace,
        CanonicalTruthError, canonical_hash, validate_action_id, validate_unique_action_ids,
    };
    use crate::traits::GameType;

    #[test]
    fn canonical_game_spec_hash_is_stable() {
        let spec = CanonicalGameSpec {
            game_type: GameType::Bridge,
            slug: "bridge".to_string(),
            chain_id: "bridge".to_string(),
            ruleset_version: 1,
            display_name: "Contract Bridge".to_string(),
            default_players: 4,
            rule_file: Some("research/game-rules/10-bridge.md".to_string()),
        };

        let first = match canonical_hash(&spec) {
            Ok(hash) => hash,
            Err(error) => panic!("spec should hash: {error}"),
        };
        let second = match canonical_hash(&spec) {
            Ok(hash) => hash,
            Err(error) => panic!("spec should hash again: {error}"),
        };

        assert_eq!(first, second);
        assert_eq!(first.len(), 64);
    }

    #[test]
    fn action_id_validation_rejects_malformed_ids() {
        assert!(validate_action_id("bridge.play.follow-suit").is_ok());
        assert_eq!(
            validate_action_id("Bridge Play"),
            Err(CanonicalTruthError::MalformedActionId {
                action_id: "Bridge Play".to_string()
            })
        );
        assert_eq!(
            validate_action_id("noscope"),
            Err(CanonicalTruthError::MalformedActionId {
                action_id: "noscope".to_string()
            })
        );
    }

    #[test]
    fn duplicate_action_ids_are_rejected() {
        let action = CanonicalActionSpec {
            game_id: "bridge".to_string(),
            action_id: "bridge.play.follow-suit".to_string(),
            family: "trick_taking".to_string(),
            display_label: "follow-suit".to_string(),
            legal_phases: vec!["bootstrap".to_string()],
            params_schema: json!({"type": "object"}),
        };
        let actions = vec![action.clone(), action];

        assert_eq!(
            validate_unique_action_ids("bridge", &actions),
            Err(CanonicalTruthError::DuplicateActionId {
                game_id: "bridge".to_string(),
                action_id: "bridge.play.follow-suit".to_string()
            })
        );
    }

    #[test]
    fn terminal_payoff_is_optional_on_transition_trace() {
        let binding = CanonicalStrategyBinding {
            query_hash: "query".to_string(),
            response_hash: "response".to_string(),
            checkpoint_hash: None,
            engine_tier: "rule-aware".to_string(),
            engine_family: "trick-taking".to_string(),
            quality_summary: None,
        };
        let trace = CanonicalTransitionTrace {
            trace_id: "bridge:bootstrap-v1".to_string(),
            game_id: "bridge".to_string(),
            ruleset_version: 1,
            state_hash_before: "before".to_string(),
            action_id: "bridge.play.follow-suit".to_string(),
            action_params: json!({}),
            state_hash_after: "after".to_string(),
            strategy_binding: binding,
            payoff: None,
        };

        assert!(trace.payoff.is_none());
        assert!(canonical_hash(&trace).is_ok());
    }
}

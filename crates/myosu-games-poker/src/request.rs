use std::panic::{AssertUnwindSafe, catch_unwind};

use rbp_cards::{Hand, Observation};
use rbp_gameplay::{Action, Partial, Turn};
use rbp_nlhe::NlheEncoder;
use rbp_nlhe::NlheInfo;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::robopoker::{NlheInfoKey, NlheStrategyQuery};
use crate::state::{NlheSnapshot, NlheTablePosition};

/// Exact replay action used to rebuild a robopoker inference-time recall.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NlheHistoryAction {
    Fold,
    Check,
    Call { amount: i16 },
    Raise { amount: i16 },
    Shove { amount: i16 },
    Deal { cards: Vec<String> },
}

impl NlheHistoryAction {
    fn to_action(&self) -> Result<Action, StrategyRequestError> {
        match self {
            Self::Fold => Ok(Action::Fold),
            Self::Check => Ok(Action::Check),
            Self::Call { amount } => positive_amount(*amount, "call").map(Action::Call),
            Self::Raise { amount } => positive_amount(*amount, "raise").map(Action::Raise),
            Self::Shove { amount } => positive_amount(*amount, "shove").map(Action::Shove),
            Self::Deal { cards } => {
                let hand = parse_cards(cards, "deal cards")?;
                Ok(Action::Draw(hand))
            }
        }
    }
}

/// Myosu-side inference request for a single poker strategy lookup.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct NlheStrategyRequest {
    pub hero_position: NlheTablePosition,
    pub hero_hole: [String; 2],
    pub board: Vec<String>,
    pub action_history: Vec<NlheHistoryAction>,
    pub abstraction_bucket: i16,
}

impl NlheStrategyRequest {
    /// Create a request from a renderable snapshot plus the missing replay inputs.
    pub fn from_snapshot(
        snapshot: &NlheSnapshot,
        action_history: Vec<NlheHistoryAction>,
        abstraction_bucket: i16,
    ) -> Self {
        Self {
            hero_position: snapshot.hero.position,
            hero_hole: snapshot.hero_hole.clone(),
            board: snapshot.board.clone(),
            action_history,
            abstraction_bucket,
        }
    }

    /// Create a request from a canonical observation string and replay inputs.
    pub fn from_observation_text(
        hero_position: NlheTablePosition,
        observation_text: &str,
        action_history: Vec<NlheHistoryAction>,
        abstraction_bucket: i16,
    ) -> Result<Self, StrategyRequestError> {
        Observation::try_from(observation_text).map_err(|message| {
            StrategyRequestError::Observation {
                input: observation_text.to_string(),
                message,
            }
        })?;
        let (hole_text, board_text) = observation_text
            .split_once('~')
            .unwrap_or((observation_text, ""));
        let hero_hole = parse_hero_hole(hole_text)?;
        let board = split_card_text(board_text, "board")?;

        Ok(Self {
            hero_position,
            hero_hole,
            board,
            action_history,
            abstraction_bucket,
        })
    }

    /// Convert the request into robopoker's card observation.
    pub fn observation(&self) -> Result<Observation, StrategyRequestError> {
        let pocket = self.hero_hole.join("");
        let board = self.board.join("");
        let text = if board.is_empty() {
            pocket
        } else {
            format!("{pocket}~{board}")
        };

        Observation::try_from(text.as_str()).map_err(|message| StrategyRequestError::Observation {
            input: text,
            message,
        })
    }

    /// Convert the request history into robopoker replay actions.
    pub fn actions(&self) -> Result<Vec<Action>, StrategyRequestError> {
        self.action_history
            .iter()
            .map(NlheHistoryAction::to_action)
            .collect()
    }

    /// Build the upstream inference-time recall for this request.
    pub fn partial(&self) -> Result<Partial, StrategyRequestError> {
        let observation = self.observation()?;
        let actions = self.actions()?;

        Partial::try_build(self.hero_turn(), observation, actions).map_err(|source| {
            StrategyRequestError::Recall {
                context: "nlhe partial recall",
                message: source.to_string(),
            }
        })
    }

    /// Build the upstream robopoker information set for this request.
    pub fn info(&self) -> Result<NlheInfo, StrategyRequestError> {
        let partial = self.partial()?;
        let abstraction = self.abstraction();

        Ok(NlheInfo::from((&partial, abstraction)))
    }

    /// Build the wire-safe information-set key for this request.
    pub fn info_key(&self) -> Result<NlheInfoKey, StrategyRequestError> {
        self.info().map(NlheInfoKey::from)
    }

    /// Build the wire-safe strategy query for this request.
    pub fn query(&self) -> Result<NlheStrategyQuery, StrategyRequestError> {
        self.info_key().map(NlheStrategyQuery::new)
    }

    /// Derive the abstraction bucket from an encoder artifact.
    pub fn abstraction_with_encoder(
        &self,
        encoder: &NlheEncoder,
    ) -> Result<rbp_gameplay::Abstraction, StrategyRequestError> {
        let observation = self.observation()?;
        let display = format!("{}", observation);

        catch_unwind(AssertUnwindSafe(|| encoder.abstraction(&observation))).map_err(|_| {
            StrategyRequestError::MissingAbstraction {
                observation: display,
            }
        })
    }

    /// Build an information set by deriving the abstraction from an encoder.
    pub fn info_with_encoder(
        &self,
        encoder: &NlheEncoder,
    ) -> Result<NlheInfo, StrategyRequestError> {
        let partial = self.partial()?;
        let abstraction = self.abstraction_with_encoder(encoder)?;

        Ok(NlheInfo::from((&partial, abstraction)))
    }

    /// Build an information-set key by deriving the abstraction from an encoder.
    pub fn info_key_with_encoder(
        &self,
        encoder: &NlheEncoder,
    ) -> Result<NlheInfoKey, StrategyRequestError> {
        self.info_with_encoder(encoder).map(NlheInfoKey::from)
    }

    /// Build a strategy query by deriving the abstraction from an encoder.
    pub fn query_with_encoder(
        &self,
        encoder: &NlheEncoder,
    ) -> Result<NlheStrategyQuery, StrategyRequestError> {
        self.info_key_with_encoder(encoder)
            .map(NlheStrategyQuery::new)
    }

    fn abstraction(&self) -> rbp_gameplay::Abstraction {
        rbp_gameplay::Abstraction::from(self.abstraction_bucket)
    }

    fn hero_turn(&self) -> Turn {
        match self.hero_position {
            NlheTablePosition::Button => Turn::Choice(0),
            NlheTablePosition::BigBlind => Turn::Choice(1),
        }
    }
}

/// Error returned when lowering a Myosu request into robopoker lookup types.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum StrategyRequestError {
    #[error("invalid {context} amount: {amount}")]
    InvalidAmount { context: &'static str, amount: i16 },
    #[error("invalid {context}: {message}")]
    Cards {
        context: &'static str,
        message: String,
    },
    #[error("invalid observation `{input}`: {message}")]
    Observation { input: String, message: String },
    #[error("failed to build {context}: {message}")]
    Recall {
        context: &'static str,
        message: String,
    },
    #[error("encoder missing abstraction for observation `{observation}`")]
    MissingAbstraction { observation: String },
}

fn positive_amount(amount: i16, context: &'static str) -> Result<i16, StrategyRequestError> {
    if amount > 0 {
        return Ok(amount);
    }

    Err(StrategyRequestError::InvalidAmount { context, amount })
}

fn parse_cards(cards: &[String], context: &'static str) -> Result<Hand, StrategyRequestError> {
    let joined = cards.join("");
    Hand::try_from(joined.as_str())
        .map_err(|message| StrategyRequestError::Cards { context, message })
}

fn parse_hero_hole(text: &str) -> Result<[String; 2], StrategyRequestError> {
    let cards = split_card_text(text, "hero hole")?;
    if cards.len() != 2 {
        return Err(StrategyRequestError::Cards {
            context: "hero hole",
            message: format!("expected exactly 2 cards, got {}", cards.len()),
        });
    }

    let [first, second] = cards
        .try_into()
        .expect("hero hole length already checked above");
    Ok([first, second])
}

fn split_card_text(text: &str, context: &'static str) -> Result<Vec<String>, StrategyRequestError> {
    if text.is_empty() {
        return Ok(Vec::new());
    }
    if !text.len().is_multiple_of(2) {
        return Err(StrategyRequestError::Cards {
            context,
            message: format!("expected an even card string length, got {}", text.len()),
        });
    }

    let cards = text
        .as_bytes()
        .chunks_exact(2)
        .map(|chunk| String::from_utf8(chunk.to_vec()).expect("card bytes should be ASCII"))
        .collect::<Vec<_>>();
    parse_cards(&cards, context)?;
    Ok(cards)
}

#[cfg(test)]
mod tests {
    use rbp_cards::{Isomorphism, Observation};
    use rbp_gameplay::{Abstraction, Edge, Recall};
    use rbp_mccfr::Encounter;
    use rbp_nlhe::{NlheEdge, NlheEncoder, NlheProfile};
    use std::collections::BTreeMap;

    use super::*;
    use crate::artifacts::{bootstrap_encoder_streets, bootstrap_scenarios, encoder_from_lookup};
    use crate::robopoker::NlheBlueprint;
    use crate::state::{NlheActor, NlhePlayerState, NlheStreet};

    #[test]
    fn request_matches_manual_partial_conversion() {
        let request = sample_preflop_request();
        let partial = request.partial().expect("request should build recall");
        let manual = Partial::try_build(
            Turn::Choice(0),
            Observation::try_from("AcKh").expect("observation should parse"),
            vec![],
        )
        .expect("manual recall should build");

        assert_eq!(partial.history(), manual.history());
        assert_eq!(partial.subgame(), manual.subgame());
        assert_eq!(partial.choices(), manual.choices());
    }

    #[test]
    fn request_builds_expected_info_key() {
        let request = sample_preflop_request();
        let partial = request.partial().expect("request should build recall");
        let manual = NlheInfo::from((&partial, Abstraction::from(42_i16)));
        let key = request.info_key().expect("request should build info key");

        assert_eq!(key, NlheInfoKey::from(manual));
    }

    #[test]
    fn request_from_snapshot_answers_known_profile() {
        let snapshot = sample_snapshot();
        let request = NlheStrategyRequest::from_snapshot(&snapshot, Vec::new(), 42);
        let info = request.info().expect("request should build info");
        let blueprint = blueprint_with_weighted_policy(info);
        let response = blueprint.answer(request.query().expect("request should build query"));

        assert!(
            response
                .actions
                .iter()
                .any(|(edge, _)| *edge == NlheEdge::from(Edge::Call)),
            "response actions were {:?}",
            response.actions
        );

        let recommended = crate::recommended_edge(&response).expect("known profile should act");

        assert_eq!(recommended, NlheEdge::from(Edge::Call));
    }

    #[test]
    fn request_can_derive_info_from_encoder() {
        let request = sample_preflop_request();
        let encoder = sample_encoder().expect("encoder should build");

        let manual = request.info().expect("manual bucket path should work");
        let derived = request
            .info_with_encoder(&encoder)
            .expect("encoder-derived path should work");

        assert_eq!(derived, manual);
    }

    #[test]
    fn request_can_query_with_encoder() {
        let request = sample_preflop_request();
        let encoder = sample_encoder().expect("encoder should build");
        let info = request
            .info_with_encoder(&encoder)
            .expect("encoder-derived info should work");
        let blueprint = blueprint_with_weighted_policy(info);

        let recommended = blueprint
            .recommend_query(
                request
                    .query_with_encoder(&encoder)
                    .expect("encoder-derived query should work"),
            )
            .expect("known profile should have action");

        assert_eq!(recommended, NlheEdge::from(Edge::Call));
    }

    #[test]
    fn bootstrap_scenarios_query_with_encoder() {
        let merged_lookup = bootstrap_encoder_streets()
            .into_values()
            .flat_map(|lookup| lookup.into_iter())
            .collect();
        let encoder = encoder_from_lookup(merged_lookup).expect("encoder should build");

        for scenario in bootstrap_scenarios() {
            let request = NlheStrategyRequest::from_observation_text(
                NlheTablePosition::Button,
                scenario.observation,
                Vec::new(),
                0,
            )
            .unwrap_or_else(|error| panic!("scenario {} should parse: {error:?}", scenario.label));
            let query = request
                .query_with_encoder(&encoder)
                .expect("bootstrap scenario should derive a query");
            assert!(
                !query.info.bucket.is_negative(),
                "scenario {} should derive a non-negative bucket",
                scenario.label
            );
        }
    }

    #[test]
    fn request_rejects_bad_cards() {
        let mut request = sample_preflop_request();
        request.hero_hole = ["Ac".to_string(), "Ac".to_string()];

        let error = request
            .observation()
            .expect_err("duplicate cards should fail");

        assert!(matches!(error, StrategyRequestError::Observation { .. }));
    }

    #[test]
    fn request_rejects_non_positive_amounts() {
        let request = NlheStrategyRequest {
            action_history: vec![NlheHistoryAction::Raise { amount: 0 }],
            ..sample_preflop_request()
        };

        let error = request.actions().expect_err("zero raise should fail");

        assert_eq!(
            error,
            StrategyRequestError::InvalidAmount {
                context: "raise",
                amount: 0,
            }
        );
    }

    #[test]
    fn request_reports_missing_encoder_lookup() {
        let request = sample_preflop_request();
        let encoder = NlheEncoder::default();

        let error = request
            .info_with_encoder(&encoder)
            .expect_err("missing lookup should fail");

        assert!(matches!(
            error,
            StrategyRequestError::MissingAbstraction { .. }
        ));
    }

    fn sample_preflop_request() -> NlheStrategyRequest {
        NlheStrategyRequest {
            hero_position: NlheTablePosition::Button,
            hero_hole: ["Ac".to_string(), "Kh".to_string()],
            board: Vec::new(),
            action_history: Vec::new(),
            abstraction_bucket: 42,
        }
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
            min_raise_to_bb: Some(4),
            legal_actions: vec![
                crate::NlheAction::Fold,
                crate::NlheAction::Call,
                crate::NlheAction::RaiseTo { amount_bb: 4 },
            ],
            hero: NlhePlayerState::new("Hero", NlheTablePosition::Button, 99),
            villain: NlhePlayerState::new("Villain", NlheTablePosition::BigBlind, 98),
        }
    }

    fn blueprint_with_weighted_policy(info: NlheInfo) -> NlheBlueprint {
        let encounters = BTreeMap::from([(
            info,
            info.choices()
                .into_iter()
                .map(NlheEdge::from)
                .map(|edge| {
                    let weight = if edge == NlheEdge::from(Edge::Call) {
                        0.80
                    } else {
                        0.15
                    };
                    (edge, Encounter::new(weight, 0.0, 0.0, 1))
                })
                .collect(),
        )]);
        let profile = NlheProfile {
            iterations: 1,
            encounters,
            metrics: Default::default(),
        };

        NlheBlueprint::new(NlheEncoder::default(), profile)
    }

    fn sample_encoder() -> Result<NlheEncoder, crate::ArtifactCodecError> {
        let observation = Observation::try_from("AcKh").expect("observation should parse");
        encoder_from_lookup(BTreeMap::from([(
            Isomorphism::from(observation),
            Abstraction::from(42_i16),
        )]))
    }
}

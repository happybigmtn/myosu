use myosu_games::{Profile, StrategyQuery, StrategyResponse};
use rbp_gameplay::{Abstraction, Path};
use rbp_mccfr::Decision;
use rbp_nlhe::{Flagship, NlheEdge, NlheEncoder, NlheGame, NlheInfo, NlheProfile, Strategy};
use serde::{Deserialize, Serialize};

/// Re-exported robopoker NLHE edge type.
pub type RbpNlheEdge = NlheEdge;
/// Re-exported robopoker NLHE encoder type.
pub type RbpNlheEncoder = NlheEncoder;
/// Re-exported robopoker NLHE game type.
pub type RbpNlheGame = NlheGame;
/// Re-exported robopoker NLHE information-set type.
pub type RbpNlheInfo = NlheInfo;
/// Re-exported robopoker NLHE profile type.
pub type RbpNlheProfile = NlheProfile;
/// Re-exported robopoker NLHE strategy type.
pub type RbpNlheStrategy = Strategy;
/// Re-exported robopoker flagship solver alias.
pub type NlheFlagshipSolver = Flagship;
/// Wire-safe poker strategy query type.
pub type NlheStrategyQuery = StrategyQuery<NlheInfoKey>;
/// Wire-safe poker strategy response type.
pub type NlheStrategyResponse = StrategyResponse<NlheEdge>;

/// Pick the highest-probability action from a strategy response.
pub fn recommended_edge(response: &NlheStrategyResponse) -> Option<NlheEdge> {
    response
        .actions
        .iter()
        .copied()
        .max_by(|(left_edge, left_prob), (right_edge, right_prob)| {
            left_prob
                .total_cmp(right_prob)
                .then_with(|| left_edge.cmp(right_edge))
        })
        .map(|(edge, _)| edge)
}

/// Wire-safe identifier for a robopoker NLHE information set.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct NlheInfoKey {
    /// Packed current-street history path.
    pub subgame: u64,
    /// Packed abstraction bucket.
    pub bucket: i16,
    /// Packed available-choice path.
    pub choices: u64,
}

impl NlheInfoKey {
    /// Convert the key back into a robopoker NLHE information set.
    pub fn into_info(self) -> NlheInfo {
        NlheInfo::from((
            Path::from(self.subgame),
            Abstraction::from(self.bucket),
            Path::from(self.choices),
        ))
    }
}

impl From<&NlheInfo> for NlheInfoKey {
    fn from(info: &NlheInfo) -> Self {
        Self {
            subgame: u64::from(info.subgame()),
            bucket: i16::from(info.bucket()),
            choices: u64::from(info.choices()),
        }
    }
}

impl From<NlheInfo> for NlheInfoKey {
    fn from(info: NlheInfo) -> Self {
        Self::from(&info)
    }
}

/// Thin wrapper around a trained robopoker blueprint.
pub struct NlheBlueprint {
    encoder: NlheEncoder,
    profile: NlheProfile,
}

impl NlheBlueprint {
    /// Create a blueprint from robopoker encoder/profile parts.
    pub fn new(encoder: NlheEncoder, profile: NlheProfile) -> Self {
        Self { encoder, profile }
    }

    /// Borrow the underlying encoder.
    pub fn encoder(&self) -> &NlheEncoder {
        &self.encoder
    }

    /// Borrow the underlying profile.
    pub fn profile(&self) -> &NlheProfile {
        &self.profile
    }

    /// Query the average strategy for an already-constructed information set.
    pub fn query(&self, info: NlheInfo) -> NlheStrategyResponse {
        StrategyResponse::new(self.profile.averaged_distribution(&info))
    }

    /// Query the average strategy using a wire-safe info key.
    pub fn query_key(&self, key: NlheInfoKey) -> NlheStrategyResponse {
        self.query(key.into_info())
    }

    /// Build a wire-safe query from an upstream robopoker information set.
    pub fn query_for_info(info: &NlheInfo) -> NlheStrategyQuery {
        StrategyQuery::new(NlheInfoKey::from(info))
    }

    /// Answer a wire-safe strategy query.
    pub fn answer(&self, query: NlheStrategyQuery) -> NlheStrategyResponse {
        self.query_key(query.info)
    }

    /// Return the highest-probability action for an information set.
    pub fn recommend(&self, info: NlheInfo) -> Option<NlheEdge> {
        recommended_edge(&self.query(info))
    }

    /// Return the highest-probability action for a wire-safe info key.
    pub fn recommend_key(&self, key: NlheInfoKey) -> Option<NlheEdge> {
        recommended_edge(&self.query_key(key))
    }

    /// Return the highest-probability action for a wire-safe strategy query.
    pub fn recommend_query(&self, query: NlheStrategyQuery) -> Option<NlheEdge> {
        recommended_edge(&self.answer(query))
    }

    /// Build robopoker's richer strategy view for an information set.
    pub fn strategy(&self, info: NlheInfo) -> Strategy {
        let decisions = info
            .choices()
            .into_iter()
            .map(NlheEdge::from)
            .map(|edge| {
                let mass = self.profile.cum_weight(&info, &edge);
                let counts = self.profile.cum_counts(&info, &edge);
                Decision { edge, mass, counts }
            })
            .collect::<Vec<_>>();

        Strategy::from((info, decisions))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rbp_gameplay::{Edge, Odds};
    use rbp_mccfr::Encounter;
    use std::collections::BTreeMap;

    #[test]
    fn info_key_roundtrips() {
        let info = sample_info();
        let key = NlheInfoKey::from(info);

        assert_eq!(key.into_info(), info);
    }

    #[test]
    fn query_returns_normalized_strategy_response() {
        let blueprint = NlheBlueprint::new(NlheEncoder::default(), NlheProfile::default());
        let response = blueprint.query(sample_info());

        assert!(response.is_valid());
        assert_eq!(response.actions.len(), 3);
    }

    #[test]
    fn strategy_view_matches_query_support() {
        let blueprint = NlheBlueprint::new(NlheEncoder::default(), NlheProfile::default());
        let info = sample_info();
        let response = blueprint.query(info);
        let strategy = blueprint.strategy(info);

        assert_eq!(strategy.info(), &info);
        assert_eq!(strategy.policy().len(), response.actions.len());
    }

    #[test]
    fn answer_roundtrips_through_wire_query() {
        let blueprint = NlheBlueprint::new(NlheEncoder::default(), NlheProfile::default());
        let info = sample_info();
        let query = NlheBlueprint::query_for_info(&info);
        let response = blueprint.answer(query);

        assert!(response.is_valid());
        assert_eq!(response.actions.len(), 3);
    }

    #[test]
    fn wire_types_serialize() {
        let info = sample_info();
        let query = NlheBlueprint::query_for_info(&info);
        let encoded_query = serde_json::to_string(&query).expect("query should serialize");
        let decoded_query: NlheStrategyQuery =
            serde_json::from_str(&encoded_query).expect("query should deserialize");
        let blueprint = NlheBlueprint::new(NlheEncoder::default(), NlheProfile::default());
        let response = blueprint.answer(decoded_query);
        let encoded_response = serde_json::to_string(&response).expect("response should serialize");
        let decoded_response: NlheStrategyResponse =
            serde_json::from_str(&encoded_response).expect("response should deserialize");

        assert!(decoded_response.is_valid());
        assert_eq!(decoded_response.actions.len(), response.actions.len());
    }

    #[test]
    fn recommend_picks_highest_probability_edge() {
        let info = sample_info();
        let blueprint = blueprint_with_weighted_policy(info);

        let recommended = blueprint.recommend(info);

        assert_eq!(recommended, Some(NlheEdge::from(Edge::Call)));
    }

    #[test]
    fn recommend_query_uses_wire_safe_key() {
        let info = sample_info();
        let blueprint = blueprint_with_weighted_policy(info);
        let query = NlheBlueprint::query_for_info(&info);

        let recommended = blueprint.recommend_query(query);

        assert_eq!(recommended, Some(NlheEdge::from(Edge::Call)));
    }

    fn sample_info() -> NlheInfo {
        let subgame = vec![Edge::Check, Edge::Raise(Odds::new(1, 2))]
            .into_iter()
            .collect::<Path>();
        let choices = vec![Edge::Fold, Edge::Call, Edge::Raise(Odds::new(1, 1))]
            .into_iter()
            .collect::<Path>();
        let bucket = Abstraction::from(0.42_f32);

        NlheInfo::from((subgame, bucket, choices))
    }

    fn blueprint_with_weighted_policy(info: NlheInfo) -> NlheBlueprint {
        let encounters = BTreeMap::from([(
            info,
            BTreeMap::from([
                (
                    NlheEdge::from(Edge::Fold),
                    Encounter::new(0.05, 0.0, 0.0, 1),
                ),
                (
                    NlheEdge::from(Edge::Call),
                    Encounter::new(0.80, 0.0, 0.0, 1),
                ),
                (
                    NlheEdge::from(Edge::Raise(Odds::new(1, 1))),
                    Encounter::new(0.15, 0.0, 0.0, 1),
                ),
            ]),
        )]);
        let profile = NlheProfile {
            iterations: 1,
            encounters,
            metrics: Default::default(),
        };

        NlheBlueprint::new(NlheEncoder::default(), profile)
    }
}

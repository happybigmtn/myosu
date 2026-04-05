use bincode::Options;
use thiserror::Error;

use crate::robopoker::{NlheInfoKey, NlheStrategyQuery, NlheStrategyResponse};

const MAX_DECODE_BYTES: u64 = 1_048_576;

/// Error returned when binary wire encoding or decoding fails.
#[derive(Debug, Error)]
pub enum WireCodecError {
    #[error("failed to encode {context}: {source}")]
    Encode {
        context: &'static str,
        #[source]
        source: bincode::Error,
    },
    #[error("failed to decode {context}: {source}")]
    Decode {
        context: &'static str,
        #[source]
        source: bincode::Error,
    },
}

/// Encode a wire-safe robopoker information-set key.
pub fn encode_info_key(key: &NlheInfoKey) -> Result<Vec<u8>, WireCodecError> {
    encode_codec()
        .serialize(key)
        .map_err(|source| WireCodecError::Encode {
            context: "nlhe info key",
            source,
        })
}

/// Decode a wire-safe robopoker information-set key.
pub fn decode_info_key(bytes: &[u8]) -> Result<NlheInfoKey, WireCodecError> {
    decode_codec(MAX_DECODE_BYTES)
        .deserialize(bytes)
        .map_err(|source| WireCodecError::Decode {
            context: "nlhe info key",
            source,
        })
}

/// Encode an NLHE strategy query for transport between services.
pub fn encode_strategy_query(query: &NlheStrategyQuery) -> Result<Vec<u8>, WireCodecError> {
    encode_codec()
        .serialize(query)
        .map_err(|source| WireCodecError::Encode {
            context: "nlhe strategy query",
            source,
        })
}

/// Decode an NLHE strategy query from its binary wire form.
pub fn decode_strategy_query(bytes: &[u8]) -> Result<NlheStrategyQuery, WireCodecError> {
    decode_codec(MAX_DECODE_BYTES)
        .deserialize(bytes)
        .map_err(|source| WireCodecError::Decode {
            context: "nlhe strategy query",
            source,
        })
}

/// Encode an NLHE strategy response for transport between services.
pub fn encode_strategy_response(
    response: &NlheStrategyResponse,
) -> Result<Vec<u8>, WireCodecError> {
    encode_codec()
        .serialize(response)
        .map_err(|source| WireCodecError::Encode {
            context: "nlhe strategy response",
            source,
        })
}

/// Decode an NLHE strategy response from its binary wire form.
pub fn decode_strategy_response(bytes: &[u8]) -> Result<NlheStrategyResponse, WireCodecError> {
    decode_codec(MAX_DECODE_BYTES)
        .deserialize(bytes)
        .map_err(|source| WireCodecError::Decode {
            context: "nlhe strategy response",
            source,
        })
}

fn encode_codec() -> impl Options {
    bincode::DefaultOptions::new()
        .with_fixint_encoding()
        .reject_trailing_bytes()
}

fn decode_codec(limit: u64) -> impl Options {
    bincode::DefaultOptions::new()
        .with_fixint_encoding()
        .with_limit(limit)
        .reject_trailing_bytes()
}

#[cfg(test)]
mod tests {
    use std::panic::{AssertUnwindSafe, catch_unwind};

    use proptest::collection::vec;
    use proptest::prelude::*;
    use rbp_gameplay::{Abstraction, Edge, Odds, Path};
    use rbp_nlhe::NlheInfo;

    use super::*;
    use crate::robopoker::{
        NlheBlueprint, NlheInfoKey, NlheStrategyQuery, NlheStrategyResponse, RbpNlheEncoder,
        RbpNlheProfile,
    };

    #[test]
    fn info_key_roundtrips_through_bincode() {
        let key = NlheInfoKey::from(sample_info());
        let encoded = encode_info_key(&key).expect("key should encode");
        let decoded = decode_info_key(&encoded).expect("key should decode");

        assert_eq!(decoded, key);
    }

    #[test]
    fn strategy_query_roundtrips_through_bincode() {
        let query = NlheBlueprint::query_for_info(&sample_info());
        let encoded = encode_strategy_query(&query).expect("query should encode");
        let decoded = decode_strategy_query(&encoded).expect("query should decode");

        assert_eq!(decoded, query);
    }

    #[test]
    fn strategy_response_roundtrips_through_bincode() {
        let blueprint = NlheBlueprint::new(RbpNlheEncoder::default(), RbpNlheProfile::default());
        let response = blueprint.query(sample_info());
        let encoded = encode_strategy_response(&response).expect("response should encode");
        let decoded = decode_strategy_response(&encoded).expect("response should decode");

        assert_eq!(decoded, response);
        assert!(decoded.is_valid());
    }

    #[test]
    fn empty_strategy_response_roundtrips_through_bincode() {
        let response = NlheStrategyResponse::new(Vec::new());
        let encoded = encode_strategy_response(&response).expect("response should encode");
        let decoded = decode_strategy_response(&encoded).expect("response should decode");

        assert_eq!(decoded, response);
        assert!(decoded.is_valid());
    }

    #[test]
    fn zero_probability_edges_roundtrip_through_bincode() {
        let response = NlheStrategyResponse::new(vec![
            (crate::RbpNlheEdge::from(Edge::Fold), 0.0),
            (crate::RbpNlheEdge::from(Edge::Call), 1.0),
            (crate::RbpNlheEdge::from(Edge::Raise(Odds::new(1, 1))), 0.0),
        ]);
        let encoded = encode_strategy_response(&response).expect("response should encode");
        let decoded = decode_strategy_response(&encoded).expect("response should decode");

        assert_eq!(decoded, response);
        assert!(decoded.is_valid());
        assert_eq!(
            decoded.probability_for(&crate::RbpNlheEdge::from(Edge::Fold)),
            0.0
        );
        assert_eq!(
            decoded.probability_for(&crate::RbpNlheEdge::from(Edge::Call)),
            1.0
        );
    }

    #[test]
    fn max_info_key_roundtrips_through_bincode() {
        let key = NlheInfoKey {
            subgame: u64::MAX,
            bucket: i16::MAX,
            choices: u64::MAX,
        };
        let encoded = encode_info_key(&key).expect("key should encode");
        let decoded = decode_info_key(&encoded).expect("key should decode");

        assert_eq!(decoded, key);
    }

    #[test]
    fn decode_rejects_trailing_bytes() {
        let query = NlheBlueprint::query_for_info(&sample_info());
        let mut encoded = encode_strategy_query(&query).expect("query should encode");
        encoded.push(0);

        let error = decode_strategy_query(&encoded).expect_err("trailing bytes should fail");

        assert!(matches!(
            error,
            WireCodecError::Decode {
                context: "nlhe strategy query",
                ..
            }
        ));
    }

    #[test]
    fn decode_rejects_truncated_strategy_response() {
        let blueprint = NlheBlueprint::new(RbpNlheEncoder::default(), RbpNlheProfile::default());
        let response = blueprint.query(sample_info());
        let mut encoded = encode_strategy_response(&response).expect("response should encode");
        encoded.pop().expect("encoded response should not be empty");

        let error = decode_strategy_response(&encoded).expect_err("truncated response should fail");

        assert!(matches!(
            error,
            WireCodecError::Decode {
                context: "nlhe strategy response",
                ..
            }
        ));
    }

    #[test]
    fn decode_rejects_oversized_strategy_response() {
        let oversized = vec![0_u8; MAX_DECODE_BYTES as usize + 1];

        let error = decode_strategy_response(&oversized).expect_err("oversized response should fail");

        assert!(matches!(
            error,
            WireCodecError::Decode {
                context: "nlhe strategy response",
                ..
            }
        ));
    }

    #[test]
    fn decode_codec_carries_a_real_byte_limit() {
        let response = NlheStrategyResponse::new(vec![(crate::RbpNlheEdge::from(Edge::Call), 1.0)]);
        let result = decode_codec(0).serialized_size(&response);

        assert!(
            result.is_err(),
            "bounded codec should reject over-budget values"
        );
    }

    fn arb_strategy_query() -> impl Strategy<Value = NlheStrategyQuery> {
        (any::<u64>(), any::<i16>(), any::<u64>()).prop_map(|(subgame, bucket, choices)| {
            NlheStrategyQuery::new(NlheInfoKey {
                subgame,
                bucket,
                choices,
            })
        })
    }

    fn arb_nlhe_edge() -> impl Strategy<Value = crate::RbpNlheEdge> {
        prop_oneof![
            Just(crate::RbpNlheEdge::from(Edge::Fold)),
            Just(crate::RbpNlheEdge::from(Edge::Call)),
            (1u8..=3, 1u8..=3)
                .prop_map(|(numerator, denominator)| {
                    crate::RbpNlheEdge::from(Edge::Raise(Odds::new(
                        i16::from(numerator),
                        i16::from(denominator),
                    )))
                }),
        ]
    }

    fn arb_strategy_response() -> impl Strategy<Value = NlheStrategyResponse> {
        vec((arb_nlhe_edge(), 0u16..=1000), 0..=6).prop_map(|actions| {
            let total = actions
                .iter()
                .map(|(_, weight)| f32::from(*weight))
                .sum::<f32>();
            if total == 0.0 {
                return NlheStrategyResponse::new(Vec::new());
            }

            let normalized = actions
                .into_iter()
                .map(|(edge, weight)| (edge, f32::from(weight) / total))
                .collect();
            NlheStrategyResponse::new(normalized)
        })
    }

    proptest! {
        #[test]
        fn fuzz_strategy_query_roundtrips_through_bincode(query in arb_strategy_query()) {
            let encoded = encode_strategy_query(&query).expect("query should encode");
            let decoded = decode_strategy_query(&encoded).expect("query should decode");

            prop_assert_eq!(decoded, query);
        }

        #[test]
        fn fuzz_strategy_response_roundtrips_through_bincode(
            response in arb_strategy_response()
        ) {
            let encoded = encode_strategy_response(&response).expect("response should encode");
            let decoded = decode_strategy_response(&encoded).expect("response should decode");

            prop_assert_eq!(&decoded, &response);
            prop_assert!(decoded.is_valid());
        }

        #[test]
        fn prop_decode_info_key_rejects_truncated_payloads(trim_seed in any::<usize>()) {
            let key = NlheInfoKey {
                subgame: u64::MAX,
                bucket: i16::MAX,
                choices: u64::MAX,
            };
            let encoded = encode_info_key(&key).expect("key should encode");
            let trim = trim_seed % encoded.len();
            let truncated = &encoded[..trim];

            let result = catch_unwind(AssertUnwindSafe(|| decode_info_key(truncated)));

            prop_assert!(result.is_ok());
            prop_assert!(result.expect("decode should not panic").is_err());
        }

        #[test]
        fn prop_decode_strategy_query_rejects_trailing_suffixes(
            suffix in vec(any::<u8>(), 1..8)
        ) {
            let query = NlheBlueprint::query_for_info(&sample_info());
            let mut encoded = encode_strategy_query(&query).expect("query should encode");
            encoded.extend_from_slice(&suffix);

            let result = catch_unwind(AssertUnwindSafe(|| decode_strategy_query(&encoded)));

            prop_assert!(result.is_ok());
            prop_assert!(result.expect("decode should not panic").is_err());
        }

        #[test]
        fn prop_decode_strategy_response_rejects_truncated_payloads(
            trim_seed in any::<usize>()
        ) {
            let blueprint = NlheBlueprint::new(RbpNlheEncoder::default(), RbpNlheProfile::default());
            let response = blueprint.query(sample_info());
            let encoded = encode_strategy_response(&response).expect("response should encode");
            let trim = trim_seed % encoded.len();
            let truncated = &encoded[..trim];

            let result = catch_unwind(AssertUnwindSafe(|| decode_strategy_response(truncated)));

            prop_assert!(result.is_ok());
            prop_assert!(result.expect("decode should not panic").is_err());
        }

        #[test]
        fn fuzz_random_strategy_query_bytes_never_panic(bytes in vec(any::<u8>(), 0..=1024)) {
            let result = catch_unwind(AssertUnwindSafe(|| decode_strategy_query(&bytes)));

            prop_assert!(result.is_ok());
        }

        #[test]
        fn fuzz_random_strategy_response_bytes_never_panic(bytes in vec(any::<u8>(), 0..=1024)) {
            let result = catch_unwind(AssertUnwindSafe(|| decode_strategy_response(&bytes)));

            prop_assert!(result.is_ok());
        }
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
}

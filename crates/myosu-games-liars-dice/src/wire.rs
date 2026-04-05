use bincode::Options;
use thiserror::Error;

use crate::game::LiarsDiceInfo;
use crate::protocol::{LiarsDiceStrategyQuery, LiarsDiceStrategyResponse};

const MAX_DECODE_BYTES: u64 = 256 * 1024 * 1024;

/// Error returned when Liar's Dice wire encoding or decoding fails.
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

/// Encode a Liar's Dice information set for transport.
pub fn encode_info(info: &LiarsDiceInfo) -> Result<Vec<u8>, WireCodecError> {
    encode_codec()
        .serialize(info)
        .map_err(|source| WireCodecError::Encode {
            context: "liar's dice info",
            source,
        })
}

/// Decode a Liar's Dice information set from transport bytes.
pub fn decode_info(bytes: &[u8]) -> Result<LiarsDiceInfo, WireCodecError> {
    decode_codec(MAX_DECODE_BYTES)
        .deserialize(bytes)
        .map_err(|source| WireCodecError::Decode {
            context: "liar's dice info",
            source,
        })
}

/// Encode a Liar's Dice strategy query for transport between services.
pub fn encode_strategy_query(query: &LiarsDiceStrategyQuery) -> Result<Vec<u8>, WireCodecError> {
    encode_codec()
        .serialize(query)
        .map_err(|source| WireCodecError::Encode {
            context: "liar's dice strategy query",
            source,
        })
}

/// Decode a Liar's Dice strategy query from transport bytes.
pub fn decode_strategy_query(bytes: &[u8]) -> Result<LiarsDiceStrategyQuery, WireCodecError> {
    decode_codec(MAX_DECODE_BYTES)
        .deserialize(bytes)
        .map_err(|source| WireCodecError::Decode {
            context: "liar's dice strategy query",
            source,
        })
}

/// Encode a Liar's Dice strategy response for transport between services.
pub fn encode_strategy_response(
    response: &LiarsDiceStrategyResponse,
) -> Result<Vec<u8>, WireCodecError> {
    encode_codec()
        .serialize(response)
        .map_err(|source| WireCodecError::Encode {
            context: "liar's dice strategy response",
            source,
        })
}

/// Decode a Liar's Dice strategy response from transport bytes.
pub fn decode_strategy_response(bytes: &[u8]) -> Result<LiarsDiceStrategyResponse, WireCodecError> {
    decode_codec(MAX_DECODE_BYTES)
        .deserialize(bytes)
        .map_err(|source| WireCodecError::Decode {
            context: "liar's dice strategy response",
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

    use bincode::Options;
    use proptest::collection::vec;
    use proptest::prelude::*;

    use crate::game::{
        LiarsDiceClaim, LiarsDiceEdge, LiarsDiceInfo, LiarsDicePublic, LiarsDiceSecret,
        LiarsDiceTurn, NO_CLAIM,
    };
    use crate::protocol::recommended_edge;
    use crate::protocol::{LiarsDiceStrategyQuery, LiarsDiceStrategyResponse};
    use crate::wire::{
        WireCodecError, decode_info, decode_strategy_query, decode_strategy_response, encode_info,
        encode_strategy_query, encode_strategy_response,
    };

    #[test]
    fn info_roundtrips_through_bincode() {
        let info = sample_info();
        let encoded = encode_info(&info).expect("info should encode");
        let decoded = decode_info(&encoded).expect("info should decode");

        assert_eq!(decoded, info);
    }

    #[test]
    fn strategy_query_roundtrips_through_bincode() {
        let query = LiarsDiceStrategyQuery::new(sample_info());
        let encoded = encode_strategy_query(&query).expect("query should encode");
        let decoded = decode_strategy_query(&encoded).expect("query should decode");

        assert_eq!(decoded, query);
    }

    #[test]
    fn strategy_response_roundtrips_through_bincode() {
        let response = LiarsDiceStrategyResponse::new(vec![
            (
                crate::game::LiarsDiceEdge::Bid(
                    LiarsDiceClaim::new(1, 4).expect("claim should build"),
                ),
                0.75,
            ),
            (crate::game::LiarsDiceEdge::Challenge, 0.25),
        ]);
        let encoded = encode_strategy_response(&response).expect("response should encode");
        let decoded = decode_strategy_response(&encoded).expect("response should decode");

        assert_eq!(decoded, response);
        assert!(decoded.is_valid());
        assert_eq!(
            recommended_edge(&decoded),
            Some(crate::game::LiarsDiceEdge::Bid(
                LiarsDiceClaim::new(1, 4).expect("claim should build")
            ))
        );
    }

    #[test]
    fn decode_rejects_trailing_query_bytes() {
        let mut encoded =
            encode_strategy_query(&LiarsDiceStrategyQuery::new(sample_info())).expect("encode");
        encoded.push(0);

        let error = decode_strategy_query(&encoded).expect_err("trailing bytes should fail");

        assert!(matches!(
            error,
            WireCodecError::Decode {
                context: "liar's dice strategy query",
                ..
            }
        ));
    }

    #[test]
    fn decode_rejects_truncated_response() {
        let response = LiarsDiceStrategyResponse::new(vec![(
            crate::game::LiarsDiceEdge::Bid(LiarsDiceClaim::new(1, 2).expect("claim should build")),
            1.0,
        )]);
        let mut encoded = encode_strategy_response(&response).expect("response should encode");
        encoded.pop().expect("encoded response should not be empty");

        let error = decode_strategy_response(&encoded).expect_err("truncated bytes should fail");

        assert!(matches!(
            error,
            WireCodecError::Decode {
                context: "liar's dice strategy response",
                ..
            }
        ));
    }

    #[test]
    fn decode_codec_carries_a_real_byte_limit() {
        let response = LiarsDiceStrategyResponse::new(vec![(
            crate::game::LiarsDiceEdge::Bid(LiarsDiceClaim::new(2, 6).expect("claim should build")),
            1.0,
        )]);
        let result = super::decode_codec(0).serialized_size(&response);

        assert!(
            result.is_err(),
            "bounded codec should reject over-budget values"
        );
    }

    fn arb_turn() -> impl Strategy<Value = LiarsDiceTurn> {
        prop_oneof![Just(LiarsDiceTurn::P1), Just(LiarsDiceTurn::P2),]
    }

    fn arb_claim() -> impl Strategy<Value = LiarsDiceClaim> {
        (1u8..=2, 1u8..=6).prop_map(|(count, face)| LiarsDiceClaim { count, face })
    }

    fn arb_info() -> impl Strategy<Value = LiarsDiceInfo> {
        (arb_turn(), prop::option::of(arb_claim()), 1u8..=6).prop_map(|(actor, claim, die)| {
            let last_claim_rank = claim.map_or(NO_CLAIM, LiarsDiceClaim::rank);
            LiarsDiceInfo::new(
                LiarsDicePublic::new(actor, last_claim_rank),
                LiarsDiceSecret(die),
            )
        })
    }

    fn arb_strategy_query() -> impl Strategy<Value = LiarsDiceStrategyQuery> {
        arb_info().prop_map(LiarsDiceStrategyQuery::new)
    }

    fn arb_liars_dice_edge() -> impl Strategy<Value = LiarsDiceEdge> {
        prop_oneof![
            Just(LiarsDiceEdge::Challenge),
            arb_claim().prop_map(LiarsDiceEdge::Bid),
        ]
    }

    fn arb_strategy_response() -> impl Strategy<Value = LiarsDiceStrategyResponse> {
        vec((arb_liars_dice_edge(), 0u16..=1000), 0..=8).prop_map(|actions| {
            let total = actions
                .iter()
                .map(|(_, weight)| f32::from(*weight))
                .sum::<f32>();
            if total == 0.0 {
                return LiarsDiceStrategyResponse::new(Vec::new());
            }

            let normalized = actions
                .into_iter()
                .map(|(edge, weight)| (edge, f32::from(weight) / total))
                .collect();
            LiarsDiceStrategyResponse::new(normalized)
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

    fn sample_info() -> LiarsDiceInfo {
        LiarsDiceInfo::new(
            LiarsDicePublic::new(LiarsDiceTurn::P1, NO_CLAIM),
            LiarsDiceSecret(4),
        )
    }
}

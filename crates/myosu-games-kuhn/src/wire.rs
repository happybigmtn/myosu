use bincode::Options;
use thiserror::Error;

use crate::game::KuhnInfo;
use crate::protocol::{KuhnStrategyQuery, KuhnStrategyResponse};

const MAX_DECODE_BYTES: u64 = 256 * 1024 * 1024;

/// Error returned when Kuhn poker wire encoding or decoding fails.
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

/// Encode a Kuhn poker information set for transport.
pub fn encode_info(info: &KuhnInfo) -> Result<Vec<u8>, WireCodecError> {
    encode_codec()
        .serialize(info)
        .map_err(|source| WireCodecError::Encode {
            context: "kuhn info",
            source,
        })
}

/// Decode a Kuhn poker information set from transport bytes.
pub fn decode_info(bytes: &[u8]) -> Result<KuhnInfo, WireCodecError> {
    decode_codec(MAX_DECODE_BYTES)
        .deserialize(bytes)
        .map_err(|source| WireCodecError::Decode {
            context: "kuhn info",
            source,
        })
}

/// Encode a Kuhn poker strategy query for transport between services.
pub fn encode_strategy_query(query: &KuhnStrategyQuery) -> Result<Vec<u8>, WireCodecError> {
    encode_codec()
        .serialize(query)
        .map_err(|source| WireCodecError::Encode {
            context: "kuhn strategy query",
            source,
        })
}

/// Decode a Kuhn poker strategy query from transport bytes.
pub fn decode_strategy_query(bytes: &[u8]) -> Result<KuhnStrategyQuery, WireCodecError> {
    decode_codec(MAX_DECODE_BYTES)
        .deserialize(bytes)
        .map_err(|source| WireCodecError::Decode {
            context: "kuhn strategy query",
            source,
        })
}

/// Encode a Kuhn poker strategy response for transport between services.
pub fn encode_strategy_response(
    response: &KuhnStrategyResponse,
) -> Result<Vec<u8>, WireCodecError> {
    encode_codec()
        .serialize(response)
        .map_err(|source| WireCodecError::Encode {
            context: "kuhn strategy response",
            source,
        })
}

/// Decode a Kuhn poker strategy response from transport bytes.
pub fn decode_strategy_response(bytes: &[u8]) -> Result<KuhnStrategyResponse, WireCodecError> {
    decode_codec(MAX_DECODE_BYTES)
        .deserialize(bytes)
        .map_err(|source| WireCodecError::Decode {
            context: "kuhn strategy response",
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
    use bincode::Options;

    use crate::game::{KuhnCard, KuhnEdge, KuhnGame};
    use crate::protocol::{KuhnStrategyQuery, KuhnStrategyResponse, recommended_edge};
    use crate::wire::{
        WireCodecError, decode_info, decode_strategy_query, decode_strategy_response, encode_info,
        encode_strategy_query, encode_strategy_response,
    };
    use myosu_games::CfrGame;

    #[test]
    fn info_roundtrips_through_bincode() {
        let info = sample_info();
        let encoded = encode_info(&info).expect("info should encode");
        let decoded = decode_info(&encoded).expect("info should decode");

        assert_eq!(decoded, info);
    }

    #[test]
    fn strategy_query_roundtrips_through_bincode() {
        let query = KuhnStrategyQuery::new(sample_info());
        let encoded = encode_strategy_query(&query).expect("query should encode");
        let decoded = decode_strategy_query(&encoded).expect("query should decode");

        assert_eq!(decoded, query);
    }

    #[test]
    fn strategy_response_roundtrips_through_bincode() {
        let response =
            KuhnStrategyResponse::new(vec![(KuhnEdge::Check, 0.7), (KuhnEdge::Bet, 0.3)]);
        let encoded = encode_strategy_response(&response).expect("response should encode");
        let decoded = decode_strategy_response(&encoded).expect("response should decode");

        assert_eq!(decoded, response);
        assert!(decoded.is_valid());
        assert_eq!(recommended_edge(&decoded), Some(KuhnEdge::Check));
    }

    #[test]
    fn decode_rejects_trailing_query_bytes() {
        let mut encoded =
            encode_strategy_query(&KuhnStrategyQuery::new(sample_info())).expect("encode");
        encoded.push(0);

        let error = decode_strategy_query(&encoded).expect_err("trailing bytes should fail");

        assert!(matches!(
            error,
            WireCodecError::Decode {
                context: "kuhn strategy query",
                ..
            }
        ));
    }

    #[test]
    fn decode_rejects_truncated_response() {
        let response = KuhnStrategyResponse::new(vec![(KuhnEdge::Call, 1.0)]);
        let mut encoded = encode_strategy_response(&response).expect("response should encode");
        encoded.pop().expect("encoded response should not be empty");

        let error = decode_strategy_response(&encoded).expect_err("truncated bytes should fail");

        assert!(matches!(
            error,
            WireCodecError::Decode {
                context: "kuhn strategy response",
                ..
            }
        ));
    }

    #[test]
    fn decode_codec_carries_a_real_byte_limit() {
        let response = KuhnStrategyResponse::new(vec![(KuhnEdge::Fold, 1.0)]);
        let result = super::decode_codec(0).serialized_size(&response);

        assert!(
            result.is_err(),
            "bounded codec should reject over-budget values"
        );
    }

    fn sample_info() -> crate::game::KuhnInfo {
        KuhnGame::root()
            .apply(KuhnEdge::Deal {
                p1: KuhnCard::King,
                p2: KuhnCard::Queen,
            })
            .info()
            .expect("dealt opening state should expose info")
    }
}

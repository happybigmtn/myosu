use bincode::Options;
use thiserror::Error;

use crate::protocol::{
    PortfolioInfo, PortfolioStrategyQuery, PortfolioStrategyResponse, PortfolioStrengthQuery,
};

const MAX_DECODE_BYTES: u64 = 1_048_576;

/// Error returned when portfolio wire encoding or decoding fails.
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

/// Encode a portfolio information set for transport.
pub fn encode_info(info: &PortfolioInfo) -> Result<Vec<u8>, WireCodecError> {
    encode_codec()
        .serialize(info)
        .map_err(|source| WireCodecError::Encode {
            context: "portfolio info",
            source,
        })
}

/// Decode a portfolio information set from transport bytes.
pub fn decode_info(bytes: &[u8]) -> Result<PortfolioInfo, WireCodecError> {
    decode_codec(MAX_DECODE_BYTES)
        .deserialize(bytes)
        .map_err(|source| WireCodecError::Decode {
            context: "portfolio info",
            source,
        })
}

/// Encode a portfolio strategy query for transport between services.
pub fn encode_strategy_query(query: &PortfolioStrategyQuery) -> Result<Vec<u8>, WireCodecError> {
    encode_codec()
        .serialize(query)
        .map_err(|source| WireCodecError::Encode {
            context: "portfolio strategy query",
            source,
        })
}

/// Decode a portfolio strategy query from transport bytes.
pub fn decode_strategy_query(bytes: &[u8]) -> Result<PortfolioStrategyQuery, WireCodecError> {
    decode_codec(MAX_DECODE_BYTES)
        .deserialize(bytes)
        .map_err(|source| WireCodecError::Decode {
            context: "portfolio strategy query",
            source,
        })
}

/// Encode a typed portfolio strength query for transport between services.
pub fn encode_strength_query(query: &PortfolioStrengthQuery) -> Result<Vec<u8>, WireCodecError> {
    encode_codec()
        .serialize(query)
        .map_err(|source| WireCodecError::Encode {
            context: "portfolio strength query",
            source,
        })
}

/// Decode a typed portfolio strength query from transport bytes.
pub fn decode_strength_query(bytes: &[u8]) -> Result<PortfolioStrengthQuery, WireCodecError> {
    decode_codec(MAX_DECODE_BYTES)
        .deserialize(bytes)
        .map_err(|source| WireCodecError::Decode {
            context: "portfolio strength query",
            source,
        })
}

/// Encode a portfolio strategy response for transport between services.
pub fn encode_strategy_response(
    response: &PortfolioStrategyResponse,
) -> Result<Vec<u8>, WireCodecError> {
    encode_codec()
        .serialize(response)
        .map_err(|source| WireCodecError::Encode {
            context: "portfolio strategy response",
            source,
        })
}

/// Decode a portfolio strategy response from transport bytes.
pub fn decode_strategy_response(bytes: &[u8]) -> Result<PortfolioStrategyResponse, WireCodecError> {
    decode_codec(MAX_DECODE_BYTES)
        .deserialize(bytes)
        .map_err(|source| WireCodecError::Decode {
            context: "portfolio strategy response",
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
    use crate::game::ResearchGame;
    use crate::protocol::{
        PortfolioAction, PortfolioInfo, PortfolioStrategyResponse, PortfolioStrengthInfo,
    };
    use crate::wire::{
        WireCodecError, decode_strategy_query, decode_strategy_response, decode_strength_query,
        encode_strategy_query, encode_strategy_response, encode_strength_query,
    };
    use myosu_games::StrategyQuery;

    #[test]
    fn strategy_query_roundtrips_through_bincode() {
        let query = StrategyQuery::new(PortfolioInfo::bootstrap(ResearchGame::Stratego));
        let encoded = match encode_strategy_query(&query) {
            Ok(encoded) => encoded,
            Err(error) => panic!("query should encode: {error}"),
        };
        let decoded = match decode_strategy_query(&encoded) {
            Ok(decoded) => decoded,
            Err(error) => panic!("query should decode: {error}"),
        };

        assert_eq!(decoded, query);
    }

    #[test]
    fn strength_query_roundtrips_through_bincode() {
        let info = match PortfolioStrengthInfo::bootstrap(ResearchGame::Bridge) {
            Some(info) => info,
            None => panic!("bridge should have strength info"),
        };
        let query = StrategyQuery::new(info);
        let encoded = match encode_strength_query(&query) {
            Ok(encoded) => encoded,
            Err(error) => panic!("query should encode: {error}"),
        };
        let decoded = match decode_strength_query(&encoded) {
            Ok(decoded) => decoded,
            Err(error) => panic!("query should decode: {error}"),
        };

        assert_eq!(decoded, query);
        assert_eq!(decoded.info.challenge.game(), ResearchGame::Bridge);
    }

    #[test]
    fn strategy_response_roundtrips_through_bincode() {
        let response = PortfolioStrategyResponse::new(vec![
            (PortfolioAction::Scout, 0.6),
            (PortfolioAction::AdvancePiece, 0.4),
        ]);
        let encoded = match encode_strategy_response(&response) {
            Ok(encoded) => encoded,
            Err(error) => panic!("response should encode: {error}"),
        };
        let decoded = match decode_strategy_response(&encoded) {
            Ok(decoded) => decoded,
            Err(error) => panic!("response should decode: {error}"),
        };

        assert_eq!(decoded, response);
        assert!(decoded.is_valid());
    }

    #[test]
    fn decode_rejects_trailing_query_bytes() {
        let query = StrategyQuery::new(PortfolioInfo::bootstrap(ResearchGame::Backgammon));
        let mut encoded = match encode_strategy_query(&query) {
            Ok(encoded) => encoded,
            Err(error) => panic!("query should encode: {error}"),
        };
        encoded.push(0);

        let error = decode_strategy_query(&encoded).err();

        assert!(matches!(
            error,
            Some(WireCodecError::Decode {
                context: "portfolio strategy query",
                ..
            })
        ));
    }

    #[test]
    fn decode_rejects_trailing_strength_query_bytes() {
        let info = match PortfolioStrengthInfo::bootstrap(ResearchGame::Bridge) {
            Some(info) => info,
            None => panic!("bridge should have strength info"),
        };
        let query = StrategyQuery::new(info);
        let mut encoded = match encode_strength_query(&query) {
            Ok(encoded) => encoded,
            Err(error) => panic!("query should encode: {error}"),
        };
        encoded.push(0);

        let error = decode_strength_query(&encoded).err();

        assert!(matches!(
            error,
            Some(WireCodecError::Decode {
                context: "portfolio strength query",
                ..
            })
        ));
    }
}

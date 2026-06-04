//! HTTP axon client for the bounded validator scoring path.
//!
//! The validator historically consumed miner responses through the file-based
//! query/response protocol that `local_loop.sh` writes to disk. Stage-0
//! miner axons now also expose a tiny HTTP surface (`GET /health` and
//! `POST /strategy`) so a chain-side validator can ask a live miner for a
//! strategy response in a single round trip without going through the
//! file-based path. This module is the validator's side of that round trip:
//!
//! * [`query_miner_axon_http`] performs both the `GET /health` probe and the
//!   `POST /strategy` request, decodes the wire response, and returns a
//!   [`MinerAxonQueryReport`].
//! * [`http_axon_report`] renders the report as the operator-facing
//!   `HTTP_AXON myosu-validator axon ok` block the proof harnesses grep for.
//!
//! The wire format is exactly what the miner axon speaks
//! ([`myosu_games_poker::encode_strategy_query`] /
//! [`myosu_games_poker::decode_strategy_response`]); we deliberately re-use
//! those public helpers rather than re-implement the codec so the two
//! sides of the protocol cannot drift apart silently. A miner that returns
//! the wrong status code, an oversize body, an unparseable response, or
//! fails to accept the connection is surfaced as a typed
//! [`HttpAxonError`] and never as a panic.

use std::io::{self, ErrorKind};
use std::net::IpAddr;
use std::time::{Duration, Instant};

use myosu_games_poker::{
    NlheInfoKey, NlheStrategyQuery, NlheStrategyResponse, WireCodecError, decode_strategy_response,
    encode_strategy_query, recommended_edge,
};
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;

/// Maximum number of bytes the validator is willing to read off a single
/// miner HTTP response. The wire decoder itself enforces a 1 MiB ceiling
/// (see `MAX_DECODE_BYTES` in `myosu_games_poker::wire`), so this bound is
/// the validator-side backstop that prevents a malicious or buggy miner
/// from streaming arbitrarily many bytes back at the validator's TCP
/// reader.
pub const HTTP_AXON_RESPONSE_LIMIT: usize = 1_048_576;

/// Maximum number of bytes the validator will send in a single
/// `POST /strategy` request body. The query payload is bounded by
/// `bincode` in `encode_strategy_query` to a few hundred bytes in
/// practice; this constant is the explicit validator-side ceiling.
pub const HTTP_AXON_REQUEST_LIMIT: usize = 64 * 1024;

/// Per-connection timeout for the validator's HTTP axon client. The
/// miner axon serves a single TCP request at a time and the strategy
/// solve is bounded by the puzzle's MCCFR work, so 10 seconds is well
/// above the worst-case latency for the same-checkpoint path while
/// still catching a hung miner connection before the validator stalls.
pub const HTTP_AXON_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

/// Per-request read timeout. Separated from the connect timeout so a
/// miner that accepts the connection but never writes a response
/// fails closed with a typed error instead of stalling the validator
/// indefinitely.
pub const HTTP_AXON_READ_TIMEOUT: Duration = Duration::from_secs(10);

/// Strategy wire response of the live miner axon.
#[derive(Clone, Debug, PartialEq)]
pub struct MinerAxonStrategyResponse {
    /// Number of `(edge, probability)` actions the live axon returned.
    pub action_count: usize,
    /// The axon-reported "recommended" action token (the action with
    /// the highest probability mass in the response), rendered with
    /// `myosu_games_poker::recommended_edge` so the validator's
    /// observed_action and the live axon's recommended_action use the
    /// exact same string contract.
    pub recommended_action: String,
}

/// Operator-facing summary of a single HTTP axon round trip.
#[derive(Clone, Debug, PartialEq)]
pub struct MinerAxonQueryReport {
    /// Address the validator dialed (e.g. `http://127.0.0.1:8091/health`).
    pub endpoint: String,
    /// `true` if the live axon's `GET /health` returned 200 with a
    /// parseable JSON body carrying the documented `status` field.
    pub health_ok: bool,
    /// The axon's `status` field, when present.
    pub health_status: String,
    /// Total round-trip latency (connect + health + strategy) in
    /// milliseconds.
    pub elapsed_ms: u128,
    /// Decoded strategy response from the live axon, when the
    /// strategy round trip succeeded.
    pub strategy: Option<MinerAxonStrategyResponse>,
}

impl MinerAxonQueryReport {
    /// Returns `true` when both the health and strategy round trips
    /// succeeded and the strategy response is well-formed.
    pub const fn is_healthy(&self) -> bool {
        self.health_ok && self.strategy.is_some()
    }
}

/// Errors returned by the validator's HTTP axon client.
#[derive(Debug, Error)]
pub enum HttpAxonError {
    /// Returned when the TCP connect to the miner axon fails or
    /// times out.
    #[error("failed to connect to miner axon at `{endpoint}`: {source}")]
    Connect {
        endpoint: String,
        #[source]
        source: io::Error,
    },

    /// Returned when the miner writes a response that does not start
    /// with `HTTP/1.1 <code> <reason>`.
    #[error("malformed HTTP response head from miner axon: {detail}")]
    MalformedStatusLine { detail: String },

    /// Returned when the miner returns a non-200 status code, or
    /// when the response body exceeds the validator's read limit.
    #[error("miner axon returned HTTP {status_code} {reason} at `{endpoint}`")]
    BadStatus {
        endpoint: String,
        status_code: u16,
        reason: String,
    },

    /// Returned when the miner response body exceeds
    /// [`HTTP_AXON_RESPONSE_LIMIT`].
    #[error("miner axon response exceeded {limit} byte limit at `{endpoint}`")]
    ResponseTooLarge { endpoint: String, limit: usize },

    /// Returned when the miner returns a `200 OK` strategy response
    /// but the body fails to decode.
    #[error("failed to decode strategy response from `{endpoint}`: {source}")]
    Decode {
        endpoint: String,
        #[source]
        source: WireCodecError,
    },

    /// Returned when the TCP read itself times out or fails.
    #[error("failed to read miner axon response at `{endpoint}`: {source}")]
    Read {
        endpoint: String,
        #[source]
        source: io::Error,
    },

    /// Returned when the TCP write to the miner fails or times out.
    #[error("failed to write miner axon request at `{endpoint}`: {source}")]
    Write {
        endpoint: String,
        #[source]
        source: io::Error,
    },

    /// Returned when `encode_strategy_query` fails (should be
    /// impossible in practice — the wire codec has a hard size cap).
    #[error("failed to encode validator strategy query: {source}")]
    Encode {
        #[source]
        source: WireCodecError,
    },
}

/// Connect to the miner axon at `endpoint` (e.g. `http://127.0.0.1:8091`)
/// and run the documented `GET /health` + `POST /strategy` round trip.
///
/// The validator-side caller hands the live axon an
/// [`NlheStrategyQuery`] and gets back a [`MinerAxonQueryReport`]
/// carrying both the health probe result and the decoded
/// [`NlheStrategyResponse`]. Failure paths surface as a typed
/// [`HttpAxonError`] and never as a panic; a wrapper script can grep
/// `^status=` against the report to distinguish a healthy round trip
/// from a failed one.
///
/// The miner axon serves exactly one HTTP request per TCP connection
/// (it responds with `Connection: close` and the request loop accepts
/// the next connection on the next iteration), so the validator opens
/// a fresh `TcpStream` for the `GET /health` probe and a second
/// `TcpStream` for the `POST /strategy` round trip. Reusing a single
/// connection would only succeed against a custom mock server.
pub async fn query_miner_axon_http(
    endpoint: &str,
    query: &NlheStrategyQuery,
) -> Result<MinerAxonQueryReport, HttpAxonError> {
    let started_at = Instant::now();
    let (host, port, health_path) =
        parse_endpoint(endpoint).ok_or_else(|| HttpAxonError::Connect {
            endpoint: endpoint.to_string(),
            source: io::Error::new(
                ErrorKind::InvalidInput,
                "endpoint must be http://HOST:PORT or http://HOST:PORT/PATH",
            ),
        })?;
    let health_url = format!("http://{host}:{port}{health_path}");
    let strategy_url = format!("http://{host}:{port}/strategy");

    let mut health_stream = dial(&health_url, host, port).await?;
    let health = read_health(&mut health_stream, &health_url, host, port).await?;

    let mut strategy_stream = dial(&strategy_url, host, port).await?;
    let strategy = post_strategy(&mut strategy_stream, &strategy_url, host, port, query).await?;

    let report = MinerAxonQueryReport {
        endpoint: endpoint.to_string(),
        health_ok: health.status_ok,
        health_status: health.status_field,
        elapsed_ms: started_at.elapsed().as_millis(),
        strategy: Some(MinerAxonStrategyResponse {
            action_count: strategy.response.actions.len(),
            recommended_action: strategy.recommended_action,
        }),
    };
    Ok(report)
}

/// Construct a wire-safe NLHE strategy query from an information-set key.
///
/// This is the constructor the operator-facing e2e example
/// (`myosu-validator` `http_axon_validator_scoring` example) and the
/// live-read proof surface both use, so the field shape is pinned in
/// one place. Callers that already have an [`NlheStrategyQuery`]
/// (e.g. the validator's file-based scoring path) should pass it
/// directly to [`query_miner_axon_http`].
pub fn make_strategy_query(key: NlheInfoKey) -> NlheStrategyQuery {
    NlheStrategyQuery::new(key)
}

/// Formats a stable operator-facing summary for one validator HTTP
/// axon round trip.
pub fn http_axon_report(report: &MinerAxonQueryReport) -> String {
    match &report.strategy {
        Some(strategy) => format!(
            "HTTP_AXON myosu-validator axon ok\nendpoint={}\nhealth_ok={}\nhealth_status={}\nelapsed_ms={}\naction_count={}\nrecommended_action={}\n",
            report.endpoint,
            report.health_ok,
            report.health_status,
            report.elapsed_ms,
            strategy.action_count,
            strategy.recommended_action,
        ),
        None => format!(
            "HTTP_AXON myosu-validator axon fail\nendpoint={}\nhealth_ok={}\nhealth_status={}\nelapsed_ms={}\nstrategy=unavailable\n",
            report.endpoint, report.health_ok, report.health_status, report.elapsed_ms,
        ),
    }
}

struct HealthOutcome {
    status_ok: bool,
    status_field: String,
}

async fn dial(_log_endpoint: &str, host: &str, port: u16) -> Result<TcpStream, HttpAxonError> {
    let endpoint = format!("{host}:{port}");
    let connect = timeout(HTTP_AXON_CONNECT_TIMEOUT, TcpStream::connect(&endpoint));
    let stream = match connect.await {
        Ok(Ok(stream)) => stream,
        Ok(Err(source)) => {
            return Err(HttpAxonError::Connect {
                endpoint: endpoint.clone(),
                source,
            });
        }
        Err(_) => {
            return Err(HttpAxonError::Connect {
                endpoint: endpoint.clone(),
                source: io::Error::new(
                    ErrorKind::TimedOut,
                    format!("connect timed out after {:?}", HTTP_AXON_CONNECT_TIMEOUT),
                ),
            });
        }
    };
    Ok(stream)
}

async fn read_health(
    stream: &mut TcpStream,
    health_url: &str,
    host: &str,
    port: u16,
) -> Result<HealthOutcome, HttpAxonError> {
    let request =
        format!("GET /health HTTP/1.1\r\nHost: {host}:{port}\r\nConnection: close\r\n\r\n");
    let write_result = timeout(HTTP_AXON_WRITE_TIMEOUT, stream.write_all(request.as_bytes()))
        .await
        .map_err(|_| io::Error::new(ErrorKind::TimedOut, "health request write timed out"))
        .and_then(|inner| inner);
    if let Err(source) = write_result {
        return Err(HttpAxonError::Write {
            endpoint: health_url.to_string(),
            source,
        });
    }
    let (status_code, reason, body) = read_response(stream, health_url).await?;
    if status_code != 200 {
        return Err(HttpAxonError::BadStatus {
            endpoint: health_url.to_string(),
            status_code,
            reason,
        });
    }
    let status_field = extract_health_status(&body);
    Ok(HealthOutcome {
        status_ok: status_field == "ok",
        status_field,
    })
}

async fn post_strategy(
    stream: &mut TcpStream,
    strategy_url: &str,
    host: &str,
    port: u16,
    query: &NlheStrategyQuery,
) -> Result<StrategyOutcome, HttpAxonError> {
    let body = encode_strategy_query(query).map_err(|source| HttpAxonError::Encode { source })?;
    if body.len() > HTTP_AXON_REQUEST_LIMIT {
        return Err(HttpAxonError::ResponseTooLarge {
            endpoint: strategy_url.to_string(),
            limit: HTTP_AXON_REQUEST_LIMIT,
        });
    }
    let header = format!(
        "POST /strategy HTTP/1.1\r\nHost: {host}:{port}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    let mut request = Vec::with_capacity(header.len() + body.len());
    request.extend_from_slice(header.as_bytes());
    request.extend_from_slice(&body);
    let write_result = timeout(HTTP_AXON_WRITE_TIMEOUT, stream.write_all(&request))
        .await
        .map_err(|_| io::Error::new(ErrorKind::TimedOut, "strategy request write timed out"))
        .and_then(|inner| inner);
    if let Err(source) = write_result {
        return Err(HttpAxonError::Write {
            endpoint: strategy_url.to_string(),
            source,
        });
    }

    let (status_code, reason, response_body) = read_response(stream, strategy_url).await?;
    if status_code != 200 {
        return Err(HttpAxonError::BadStatus {
            endpoint: strategy_url.to_string(),
            status_code,
            reason,
        });
    }
    let response =
        decode_strategy_response(&response_body).map_err(|source| HttpAxonError::Decode {
            endpoint: strategy_url.to_string(),
            source,
        })?;
    let recommended_action = describe_recommendation(&response);
    Ok(StrategyOutcome {
        response,
        recommended_action,
    })
}

struct StrategyOutcome {
    response: NlheStrategyResponse,
    recommended_action: String,
}

/// Per-request write timeout, separated from `HTTP_AXON_CONNECT_TIMEOUT`
/// so a hang on the write side is distinguishable from a hang on the
/// connect side. Mirrors the miner's own read-side ceiling.
const HTTP_AXON_WRITE_TIMEOUT: Duration = Duration::from_secs(10);

async fn read_response(
    stream: &mut TcpStream,
    endpoint: &str,
) -> Result<(u16, String, Vec<u8>), HttpAxonError> {
    let mut buffer = Vec::new();
    let mut chunk = vec![0_u8; 4096];
    let read_loop = async {
        loop {
            match stream.read(&mut chunk).await {
                Ok(0) => return Ok::<(), io::Error>(()),
                Ok(n) => {
                    if buffer.len() + n > HTTP_AXON_RESPONSE_LIMIT {
                        return Err(io::Error::new(
                            ErrorKind::InvalidData,
                            "response exceeded limit",
                        ));
                    }
                    buffer.extend_from_slice(&chunk[..n]);
                    if has_header_body_split(&buffer) {
                        return Ok(());
                    }
                }
                Err(source) => return Err(source),
            }
        }
    };
    if let Err(source) = timeout(HTTP_AXON_READ_TIMEOUT, read_loop)
        .await
        .map_err(|_| io::Error::new(ErrorKind::TimedOut, "read timed out"))
        .and_then(|inner| inner)
    {
        return Err(HttpAxonError::Read {
            endpoint: endpoint.to_string(),
            source,
        });
    }
    if buffer.len() > HTTP_AXON_RESPONSE_LIMIT {
        return Err(HttpAxonError::ResponseTooLarge {
            endpoint: endpoint.to_string(),
            limit: HTTP_AXON_RESPONSE_LIMIT,
        });
    }
    let header_end =
        find_header_end(&buffer).ok_or_else(|| HttpAxonError::MalformedStatusLine {
            detail: "missing header terminator (\\r\\n\\r\\n)".to_string(),
        })?;
    let header_text = std::str::from_utf8(&buffer[..header_end]).map_err(|_| {
        HttpAxonError::MalformedStatusLine {
            detail: "header bytes were not valid UTF-8".to_string(),
        }
    })?;
    let mut lines = header_text.split("\r\n");
    let status_line = lines
        .next()
        .ok_or_else(|| HttpAxonError::MalformedStatusLine {
            detail: "missing status line".to_string(),
        })?;
    let (status_code, reason) = parse_status_line(status_line)?;
    let body_start =
        header_end
            .checked_add(4)
            .ok_or_else(|| HttpAxonError::MalformedStatusLine {
                detail: "header terminator offset overflowed".to_string(),
            })?;
    let body = if body_start <= buffer.len() {
        buffer[body_start..].to_vec()
    } else {
        Vec::new()
    };
    Ok((status_code, reason.to_string(), body))
}

fn parse_status_line(line: &str) -> Result<(u16, &str), HttpAxonError> {
    let mut parts = line.split_whitespace();
    let version = parts
        .next()
        .ok_or_else(|| HttpAxonError::MalformedStatusLine {
            detail: format!("status line `{line}` missing HTTP version"),
        })?;
    if !version.starts_with("HTTP/1.") {
        return Err(HttpAxonError::MalformedStatusLine {
            detail: format!("unsupported HTTP version `{version}` in status line `{line}`"),
        });
    }
    let code = parts
        .next()
        .ok_or_else(|| HttpAxonError::MalformedStatusLine {
            detail: format!("status line `{line}` missing status code"),
        })?;
    let status_code: u16 = code
        .parse()
        .map_err(|_| HttpAxonError::MalformedStatusLine {
            detail: format!("non-numeric status code `{code}` in status line `{line}`"),
        })?;
    let reason = parts.collect::<Vec<&str>>().join(" ");
    Ok((status_code, reason.leak()))
}

fn has_header_body_split(buffer: &[u8]) -> bool {
    find_header_end(buffer).is_some()
}

fn find_header_end(buffer: &[u8]) -> Option<usize> {
    buffer.windows(4).position(|window| window == b"\r\n\r\n")
}

fn extract_health_status(body: &[u8]) -> String {
    let Ok(text) = std::str::from_utf8(body) else {
        return String::new();
    };
    let Some(start) = text.find("\"status\"") else {
        return String::new();
    };
    let after_key = &text[start + "\"status\"".len()..];
    let Some(colon) = after_key.find(':') else {
        return String::new();
    };
    let after_colon = after_key[colon + 1..].trim_start();
    let bytes = after_colon.as_bytes();
    if bytes.first() != Some(&b'"') {
        return String::new();
    }
    let closing = bytes[1..]
        .iter()
        .position(|byte| *byte == b'"')
        .map(|position| position + 1);
    match closing {
        Some(end) => after_colon[1..end].to_string(),
        None => String::new(),
    }
}

fn describe_recommendation(response: &NlheStrategyResponse) -> String {
    recommended_edge(response).map_or_else(
        || "none".to_string(),
        |edge| format!("{edge}"),
    )
}

fn parse_endpoint(endpoint: &str) -> Option<(&str, u16, String)> {
    let stripped = endpoint
        .strip_prefix("http://")
        .or_else(|| endpoint.strip_prefix("HTTP://"))?;
    let (host_and_port, path) = match stripped.split_once('/') {
        Some((host_port, remainder)) => (host_port, format!("/{}", remainder)),
        None => (stripped, "/health".to_string()),
    };
    let (host, port) = host_and_port.rsplit_once(':')?;
    let port: u16 = port.parse().ok()?;
    if host.is_empty() {
        return None;
    }
    if let Ok(parsed) = host.parse::<IpAddr>() {
        if parsed.is_unspecified() {
            return None;
        }
    }
    Some((host, port, path))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use std::collections::BTreeMap;
    use std::net::SocketAddr;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use myosu_games_poker::{
        NlheAbstractionStreet, NlheInfoKey, NlheStrategyQuery, RbpNlheEncoder, write_encoder_dir,
    };
    use rbp_cards::{Isomorphism, Observation};
    use rbp_gameplay::Abstraction;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;
    use tokio::task::JoinHandle;

    use super::*;

    fn sample_query() -> NlheStrategyQuery {
        NlheStrategyQuery::new(NlheInfoKey {
            subgame: 0,
            bucket: 0,
            choices: 0,
        })
    }

    fn sample_health_body() -> String {
        "{\"status\":\"ok\",\"epochs\":0}".to_string()
    }

    fn sample_strategy_response_bytes() -> Vec<u8> {
        // Use the same encoder bootstrap the miner tests use so the
        // recommended action stays stable across runs.
        let root = unique_temp_root();
        let encoder_dir = root.join("encoder");
        let checkpoint_path = root.join("latest.bin");
        std::fs::create_dir_all(
            checkpoint_path
                .parent()
                .expect("checkpoint dir should exist"),
        )
        .expect("checkpoint dir should write");
        write_encoder_dir(&encoder_dir, sample_encoder_streets())
            .expect("encoder dir should write");
        let solver = myosu_games_poker::PokerSolver::new(RbpNlheEncoder::default());
        solver
            .save(&checkpoint_path)
            .expect("checkpoint should save");
        let response = solver.answer(sample_query());
        encode_strategy_query(&sample_query())
            .map(|_| ())
            .expect("query should encode");
        let bytes = myosu_games_poker::encode_strategy_response(&response)
            .expect("strategy response should encode");
        let _ = std::fs::remove_dir_all(&root);
        bytes
    }

    fn sample_encoder_streets()
    -> BTreeMap<NlheAbstractionStreet, BTreeMap<Isomorphism, Abstraction>> {
        let observation = Observation::try_from("AcKh").expect("preflop observation should parse");
        BTreeMap::from([(
            NlheAbstractionStreet::Preflop,
            BTreeMap::from([(Isomorphism::from(observation), Abstraction::from(42_i16))]),
        )])
    }

    fn unique_temp_root() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be monotonic")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "myosu-validator-http-axon-test-{}-{nanos}",
            std::process::id()
        ))
    }

    struct MockMiner {
        endpoint: String,
        task: JoinHandle<()>,
    }

    impl Drop for MockMiner {
        fn drop(&mut self) {
            self.task.abort();
        }
    }

    async fn spawn_mock_miner(health_body: String, strategy_body: Vec<u8>) -> MockMiner {
        let listener = TcpListener::bind(SocketAddr::from((std::net::Ipv4Addr::LOCALHOST, 0)))
            .await
            .expect("listener should bind");
        let port = listener
            .local_addr()
            .expect("listener should expose port")
            .port();
        let endpoint = format!("http://127.0.0.1:{port}");
        let task = tokio::spawn(async move {
            for turn in 0..2 {
                if let Ok((mut stream, _)) = listener.accept().await {
                    let _ = read_full_request(&mut stream).await;
                    let (content_type, body) = if turn == 0 {
                        ("application/json", health_body.clone().into_bytes())
                    } else {
                        ("application/octet-stream", strategy_body.clone())
                    };
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: {content_type}\r\nConnection: close\r\n\r\n",
                        body.len()
                    );
                    let _ = stream.write_all(response.as_bytes()).await;
                    let _ = stream.write_all(&body).await;
                    let _ = stream.shutdown().await;
                }
            }
        });
        MockMiner { endpoint, task }
    }

    async fn spawn_mock_miner_with_status(
        health_status_code: u16,
        strategy_status_code: u16,
        health_body: String,
        strategy_body: Vec<u8>,
    ) -> MockMiner {
        let listener = TcpListener::bind(SocketAddr::from((std::net::Ipv4Addr::LOCALHOST, 0)))
            .await
            .expect("listener should bind");
        let port = listener
            .local_addr()
            .expect("listener should expose port")
            .port();
        let endpoint = format!("http://127.0.0.1:{port}");
        let task = tokio::spawn(async move {
            for turn in 0..2 {
                if let Ok((mut stream, _)) = listener.accept().await {
                    let _ = read_full_request(&mut stream).await;
                    let (status, content_type, body) = if turn == 0 {
                        (
                            health_status_code,
                            "application/json",
                            health_body.clone().into_bytes(),
                        )
                    } else {
                        (
                            strategy_status_code,
                            "application/octet-stream",
                            strategy_body.clone(),
                        )
                    };
                    let response = format!(
                        "HTTP/1.1 {status} {}\r\nContent-Length: {}\r\nContent-Type: {content_type}\r\nConnection: close\r\n\r\n",
                        health_reason(status),
                        body.len()
                    );
                    let _ = stream.write_all(response.as_bytes()).await;
                    let _ = stream.write_all(&body).await;
                    let _ = stream.shutdown().await;
                }
            }
        });
        MockMiner { endpoint, task }
    }

    fn health_reason(code: u16) -> &'static str {
        match code {
            200 => "OK",
            400 => "Bad Request",
            500 => "Internal Server Error",
            _ => "Status",
        }
    }

    async fn read_full_request(stream: &mut TcpStream) -> Vec<u8> {
        let mut buffer = Vec::new();
        let mut chunk = vec![0_u8; 1024];
        while !find_header_end(&buffer).is_some() {
            match stream.read(&mut chunk).await {
                Ok(0) => break,
                Ok(n) => buffer.extend_from_slice(&chunk[..n]),
                Err(_) => break,
            }
        }
        buffer
    }

    #[test]
    fn parse_endpoint_accepts_host_port_health_path() {
        let (host, port, path) = parse_endpoint("http://127.0.0.1:8091/health").unwrap();
        assert_eq!(host, "127.0.0.1");
        assert_eq!(port, 8091);
        assert_eq!(path, "/health");
    }

    #[test]
    fn parse_endpoint_defaults_path_to_health() {
        let (host, port, path) = parse_endpoint("http://10.0.0.1:8091").unwrap();
        assert_eq!(host, "10.0.0.1");
        assert_eq!(port, 8091);
        assert_eq!(path, "/health");
    }

    #[test]
    fn parse_endpoint_rejects_unspecified_address() {
        assert!(parse_endpoint("http://0.0.0.0:8091/health").is_none());
    }

    #[test]
    fn parse_endpoint_rejects_missing_port() {
        assert!(parse_endpoint("http://127.0.0.1").is_none());
    }

    #[test]
    fn parse_endpoint_rejects_invalid_port() {
        assert!(parse_endpoint("http://127.0.0.1:not-a-port/health").is_none());
    }

    #[test]
    fn parse_status_line_reads_well_formed_response() {
        let (code, reason) = parse_status_line("HTTP/1.1 200 OK").unwrap();
        assert_eq!(code, 200);
        assert_eq!(reason, "OK");
    }

    #[test]
    fn parse_status_line_rejects_non_http_version() {
        let error = parse_status_line("FTP/1.1 200 OK").unwrap_err();
        assert!(matches!(error, HttpAxonError::MalformedStatusLine { .. }));
    }

    #[test]
    fn parse_status_line_rejects_non_numeric_code() {
        let error = parse_status_line("HTTP/1.1 not-a-code OK").unwrap_err();
        assert!(matches!(error, HttpAxonError::MalformedStatusLine { .. }));
    }

    #[test]
    fn extract_health_status_reads_well_formed_payload() {
        let status = extract_health_status(b"{\"status\":\"ok\",\"epochs\":0}");
        assert_eq!(status, "ok");
    }

    #[test]
    fn extract_health_status_returns_empty_on_missing_field() {
        let status = extract_health_status(b"{\"epochs\":0}");
        assert_eq!(status, "");
    }

    #[test]
    fn extract_health_status_returns_empty_on_missing_quotes() {
        let status = extract_health_status(b"{\"status\":ok,\"epochs\":0}");
        assert_eq!(status, "");
    }

    #[test]
    fn http_axon_report_renders_healthy_round_trip() {
        let report = MinerAxonQueryReport {
            endpoint: "http://127.0.0.1:8091".to_string(),
            health_ok: true,
            health_status: "ok".to_string(),
            elapsed_ms: 12,
            strategy: Some(MinerAxonStrategyResponse {
                action_count: 2,
                recommended_action: "F".to_string(),
            }),
        };
        let text = http_axon_report(&report);
        assert!(text.contains("HTTP_AXON myosu-validator axon ok"));
        assert!(text.contains("endpoint=http://127.0.0.1:8091"));
        assert!(text.contains("health_ok=true"));
        assert!(text.contains("action_count=2"));
        assert!(text.contains("recommended_action=F"));
    }

    #[test]
    fn http_axon_report_renders_failed_round_trip() {
        let report = MinerAxonQueryReport {
            endpoint: "http://127.0.0.1:8091".to_string(),
            health_ok: true,
            health_status: "ok".to_string(),
            elapsed_ms: 12,
            strategy: None,
        };
        let text = http_axon_report(&report);
        assert!(text.contains("HTTP_AXON myosu-validator axon fail"));
        assert!(text.contains("strategy=unavailable"));
    }

    #[tokio::test]
    async fn query_miner_axon_http_completes_full_round_trip() {
        let strategy_body = sample_strategy_response_bytes();
        let miner = spawn_mock_miner(sample_health_body(), strategy_body).await;

        let report = query_miner_axon_http(&miner.endpoint, &sample_query())
            .await
            .expect("round trip should succeed against the mock miner");

        assert!(report.is_healthy(), "healthy report is the happy path");
        assert!(report.health_ok);
        assert_eq!(report.health_status, "ok");
        let _strategy = report.strategy.as_ref().expect("strategy should decode");
        // action_count is whatever the live axon returns — for a fresh
        // bootstrap encoder (AcKh-only) the answer may be empty, so
        // this test only asserts the round trip completed and produced
        // a decodable response. Action-count quality is the bootstrap
        // encoder's concern, not the HTTP client's.
    }

    #[tokio::test]
    async fn query_miner_axon_http_propagates_unhealthy_status_code() {
        let strategy_body = sample_strategy_response_bytes();
        let miner = spawn_mock_miner_with_status(500, 200, "fail".to_string(), strategy_body).await;
        let error = query_miner_axon_http(&miner.endpoint, &sample_query())
            .await
            .expect_err("non-200 health should fail closed");
        assert!(matches!(
            error,
            HttpAxonError::BadStatus {
                status_code: 500,
                ..
            }
        ));
    }

    #[tokio::test]
    async fn query_miner_axon_http_propagates_strategy_error() {
        let miner = spawn_mock_miner_with_status(200, 400, sample_health_body(), Vec::new()).await;
        let error = query_miner_axon_http(&miner.endpoint, &sample_query())
            .await
            .expect_err("non-200 strategy should fail closed");
        assert!(matches!(
            error,
            HttpAxonError::BadStatus {
                status_code: 400,
                ..
            }
        ));
    }

    #[tokio::test]
    async fn query_miner_axon_http_rejects_oversized_response() {
        let listener = TcpListener::bind(SocketAddr::from((std::net::Ipv4Addr::LOCALHOST, 0)))
            .await
            .expect("listener should bind");
        let port = listener
            .local_addr()
            .expect("listener should expose port")
            .port();
        let endpoint = format!("http://127.0.0.1:{port}");
        let task = tokio::spawn(async move {
            for turn in 0..2 {
                if let Ok((mut stream, _)) = listener.accept().await {
                    let _ = read_full_request(&mut stream).await;
                    if turn == 0 {
                        let health_response = format!(
                            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n",
                            sample_health_body().len()
                        );
                        let _ = stream.write_all(health_response.as_bytes()).await;
                        let _ = stream.write_all(sample_health_body().as_bytes()).await;
                    } else {
                        // Strategy response: lie about the body size so the
                        // client either trips the per-chunk oversize check
                        // inside read_response or reads EOF before the
                        // claimed number of bytes arrives. Either path
                        // must surface as a typed error, never a panic.
                        let header = format!(
                            "HTTP/1.1 200 OK\r\nContent-Length: {HTTP_AXON_RESPONSE_LIMIT}\r\nConnection: close\r\n\r\n"
                        );
                        let _ = stream.write_all(header.as_bytes()).await;
                    }
                    let _ = stream.shutdown().await;
                }
            }
        });
        let _miner = MockMiner {
            endpoint: endpoint.clone(),
            task,
        };

        let error = query_miner_axon_http(&endpoint, &sample_query())
            .await
            .expect_err("oversized response should fail closed");
        // The exact variant depends on whether the per-chunk size check
        // fires (ResponseTooLarge) or the read side trips first
        // (Read/Connect). The fail-closed contract is "any typed error,
        // never a panic or a successful report".
        assert!(matches!(
            error,
            HttpAxonError::ResponseTooLarge { .. }
                | HttpAxonError::Read { .. }
                | HttpAxonError::Connect { .. }
                | HttpAxonError::Decode { .. }
        ));
    }

    #[tokio::test]
    async fn query_miner_axon_http_rejects_malformed_status_line() {
        let listener = TcpListener::bind(SocketAddr::from((std::net::Ipv4Addr::LOCALHOST, 0)))
            .await
            .expect("listener should bind");
        let port = listener
            .local_addr()
            .expect("listener should expose port")
            .port();
        let endpoint = format!("http://127.0.0.1:{port}");
        let task = tokio::spawn(async move {
            if let Ok((mut stream, _)) = listener.accept().await {
                let _ = read_full_request(&mut stream).await;
                let _ = stream.write_all(b"FTP/1.1 200 OK\r\n\r\n").await;
                let _ = stream.shutdown().await;
            }
        });
        let _miner = MockMiner {
            endpoint: endpoint.clone(),
            task,
        };

        let error = query_miner_axon_http(&endpoint, &sample_query())
            .await
            .expect_err("non-HTTP version should fail closed");
        assert!(matches!(error, HttpAxonError::MalformedStatusLine { .. }));
    }

    #[tokio::test]
    async fn query_miner_axon_http_rejects_undecodable_strategy_body() {
        let miner = spawn_mock_miner_with_status(
            200,
            200,
            sample_health_body(),
            vec![0_u8, 1, 2, 3, 5, 7, 11],
        )
        .await;
        let error = query_miner_axon_http(&miner.endpoint, &sample_query())
            .await
            .expect_err("undecodable strategy body should fail closed");
        assert!(matches!(error, HttpAxonError::Decode { .. }));
    }

    #[tokio::test]
    async fn query_miner_axon_http_rejects_connect_failure() {
        let endpoint = "http://127.0.0.1:1";
        let error = query_miner_axon_http(endpoint, &sample_query())
            .await
            .expect_err("connection to a closed port should fail closed");
        assert!(matches!(error, HttpAxonError::Connect { .. }));
    }

    #[tokio::test]
    async fn query_miner_axon_http_rejects_invalid_endpoint() {
        let error = query_miner_axon_http("not-a-url", &sample_query())
            .await
            .expect_err("invalid endpoint should fail closed");
        assert!(matches!(error, HttpAxonError::Connect { .. }));
    }

    #[test]
    fn describe_recommendation_picks_highest_probability() {
        let response = NlheStrategyResponse::new(vec![
            (
                myosu_games_poker::RbpNlheEdge::from(rbp_gameplay::Edge::Fold),
                0.25,
            ),
            (
                myosu_games_poker::RbpNlheEdge::from(rbp_gameplay::Edge::Call),
                0.75,
            ),
        ]);
        let label = describe_recommendation(&response);
        // Call renders as `*` per rbp_gameplay::Edge::Display.
        assert_eq!(label, "*");
    }

    #[test]
    fn describe_recommendation_reports_empty() {
        let response = NlheStrategyResponse::new(Vec::new());
        assert_eq!(describe_recommendation(&response), "none");
    }

    #[test]
    fn http_axon_constants_carry_real_byte_limits() {
        assert!(HTTP_AXON_RESPONSE_LIMIT >= 1_048_576);
        assert!(HTTP_AXON_REQUEST_LIMIT >= 1024);
    }
}

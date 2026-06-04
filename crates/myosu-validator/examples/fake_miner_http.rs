//! Minimal HTTP server that mimics the live `myosu-miner --serve-http`
//! surface for the validator-scoring e2e proof.
//!
//! NEM-004's `tests/e2e/http_axon_validator_scoring.sh` exercise is
//! the validator side of the HTTP axon round trip (the `GET /health`
//! + `POST /strategy` path the validator's
//! `myosu_validator::http_axon::query_miner_axon_http` consumes). The
//! real miner HTTP server is already covered by its own unit tests
//! (`crates/myosu-miner/src/axon.rs::tests`), so the proof harness
//! only needs a deterministic wire-format server it can launch
//! without a running chain.
//!
//! This binary serves the same wire format the live miner speaks:
//!
//!   * `GET /health` -> `200 OK` with body `{"status":"ok","epochs":0}`
//!   * `POST /strategy` -> `200 OK` with a wire-encoded
//!     `NlheStrategyResponse` decoded from the request body via the
//!     public `myosu_games_poker` codec helpers.
//!
//! The server is intentionally tiny: it binds to 127.0.0.1 on the
//! caller-supplied port, serves one request per connection (the same
//! one-request-per-connection contract the live miner uses), and
//! runs until the e2e harness kills the process on cleanup. The
//! e2e harness drives the lifecycle.

use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::thread;

use myosu_games_poker::{decode_strategy_query, encode_strategy_response};

const HEALTH_BODY: &[u8] = b"{\"status\":\"ok\",\"epochs\":0}";

#[derive(Debug)]
enum RequestKind {
    Health,
    Strategy,
    NotFound,
    BadRequest(&'static str),
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let port: u16 = args
        .get(1)
        .ok_or("missing port argument")?
        .parse()
        .map_err(|e| format!("invalid port: {e}"))?;
    let bind = SocketAddr::from((Ipv4Addr::LOCALHOST, port));
    let listener = TcpListener::bind(bind)?;
    eprintln!("fake_miner_http: bound to {bind}");

    // Note: the e2e harness redirects `< /dev/null` so any watcher
    // thread that reads from stdin would observe an immediate EOF and
    // kill the server before the proof could run. The server is meant
    // to live for the lifetime of the proof and is killed explicitly
    // by the harness on cleanup, so no stdin watcher is wired here.
    //
    // The server dispatches on the request method + path (parsed from
    // the raw request line) rather than a turn counter, because the
    // proof harness's readiness probe + the validator's two
    // round-trip requests all need correct, type-driven responses
    // regardless of the order they arrive in.

    for stream in listener.incoming() {
        let stream = stream?;
        thread::spawn(move || {
            if let Err(error) = handle_client(stream) {
                eprintln!("fake_miner_http: client error: {error}");
            }
        });
    }
    Ok(())
}

fn handle_client(mut stream: TcpStream) -> std::io::Result<()> {
    // The validator's documented request shape is: full request line +
    // headers + body sent in a single TCP write (the wire codec's
    // `HTTP_AXON_REQUEST_LIMIT` of 64 KiB is well above the actual
    // query payload, so the body always fits in the first 64 KiB
    // chunk). Read until we've seen the header terminator
    // (`\r\n\r\n`), then verify the body length against the parsed
    // `Content-Length` header and dispatch on the method + path. The
    // `Content-Length` check guards against a future caller that
    // upgrades the body size past the single-packet envelope.
    let mut buffer = Vec::new();
    let mut chunk = [0_u8; 1024];
    loop {
        let n = stream.read(&mut chunk)?;
        if n == 0 {
            return Err(std::io::Error::other(
                "connection closed before request terminator",
            ));
        }
        buffer.extend_from_slice(&chunk[..n]);
        if buffer.windows(4).any(|w| w == b"\r\n\r\n") {
            break;
        }
        if buffer.len() > 64 * 1024 {
            return Err(std::io::Error::other("request too large"));
        }
    }
    let header_end = buffer
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .ok_or_else(|| std::io::Error::other("missing header terminator"))?;
    let header_bytes = &buffer[..header_end];
    let body_bytes = &buffer[header_end + 4..];
    let content_length = parse_content_length(header_bytes).unwrap_or(0);
    if body_bytes.len() < content_length {
        return Err(std::io::Error::other(
            "request body shorter than Content-Length",
        ));
    }
    let request_line = std::str::from_utf8(
        header_bytes
            .split(|&b| b == b'\r' || b == b'\n')
            .next()
            .unwrap_or(b""),
    )
    .map_err(|_| std::io::Error::other("request line is not valid UTF-8"))?
    .to_string();
    let response = match classify_request(&request_line) {
        RequestKind::Health => respond_health(),
        RequestKind::Strategy => respond_strategy(&body_bytes[..content_length]),
        RequestKind::NotFound => response_error(404, "not found"),
        RequestKind::BadRequest(message) => response_error(400, message),
    };
    stream.write_all(&response)?;
    stream.flush()?;
    Ok(())
}

fn parse_content_length(header_bytes: &[u8]) -> Option<usize> {
    // Walk through the header bytes as ASCII lines. Skip the request
    // line, then on each subsequent line look for `Content-Length: N`.
    // The header set is small, so a linear scan is fine.
    let mut lines = header_bytes.split(|&b| b == b'\n');
    let _request_line = lines.next();
    for line in lines {
        let trimmed = line.strip_suffix(b"\r").unwrap_or(line);
        if let Some(value) = trimmed.strip_prefix(b"Content-Length:") {
            let value = value
                .iter()
                .skip_while(|&&b| b == b' ')
                .copied()
                .take_while(|&b| b.is_ascii_digit())
                .collect::<Vec<_>>();
            if let Ok(s) = std::str::from_utf8(&value) {
                if let Ok(n) = s.parse::<usize>() {
                    return Some(n);
                }
            }
        }
    }
    None
}

fn classify_request(request_line: &str) -> RequestKind {
    // Request line: "METHOD SP PATH SP HTTP/VERSION"
    let mut parts = request_line.split(' ').take(3);
    let method = match parts.next() {
        Some(method) => method,
        None => return RequestKind::BadRequest("missing HTTP method"),
    };
    let path = match parts.next() {
        Some(path) => path,
        None => return RequestKind::BadRequest("missing HTTP path"),
    };
    match (method, path) {
        ("GET", "/health") => RequestKind::Health,
        ("POST", "/strategy") => RequestKind::Strategy,
        ("GET", _) | ("POST", _) => RequestKind::NotFound,
        _ => RequestKind::BadRequest("unsupported HTTP method"),
    }
}

fn respond_health() -> Vec<u8> {
    let mut out = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n",
        HEALTH_BODY.len()
    )
    .into_bytes();
    out.extend_from_slice(HEALTH_BODY);
    out
}

fn respond_strategy(body: &[u8]) -> Vec<u8> {
    // Verify the request body is a decodable strategy query (so the
    // validator's wire-format contract is exercised on the way in)
    // and respond with a deterministic strategy response. The
    // validator side checks action_count and recommended_action; the
    // Fold edge is the simplest deterministic choice.
    match decode_strategy_query(body) {
        Ok(_query) => {
            let stub = myosu_games_poker::NlheStrategyResponse::new(vec![(
                myosu_games_poker::RbpNlheEdge::from(rbp_gameplay::Edge::Fold),
                1.0_f32,
            )]);
            match encode_strategy_response(&stub) {
                Ok(encoded) => response_ok(encoded),
                Err(error) => response_error(500, &format!("encode failed: {error}")),
            }
        }
        Err(error) => response_error(400, &format!("decode failed: {error}")),
    }
}

fn response_ok(body: Vec<u8>) -> Vec<u8> {
    let mut out = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: application/octet-stream\r\nConnection: close\r\n\r\n",
        body.len()
    )
    .into_bytes();
    out.extend_from_slice(&body);
    out
}

fn response_error(status: u16, body: &str) -> Vec<u8> {
    let body_bytes = body.as_bytes();
    let reason = match status {
        200 => "OK",
        400 => "Bad Request",
        500 => "Internal Server Error",
        _ => "Status",
    };
    let mut out = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Length: {}\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\n",
        body_bytes.len()
    )
    .into_bytes();
    out.extend_from_slice(body_bytes);
    out
}

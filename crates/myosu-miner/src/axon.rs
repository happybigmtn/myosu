use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use myosu_games_poker::{
    ArtifactCodecError, PokerSolver, PokerSolverError, WireCodecError, decode_strategy_query,
    encode_strategy_response, load_encoder_dir,
};
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tracing::warn;

use crate::cli::Cli;
use crate::cli::GameSelection;

const REQUEST_LIMIT_BYTES: usize = 64 * 1024;

/// Configuration for the live HTTP miner axon server.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AxonServePlan {
    pub game: GameSelection,
    pub encoder_dir: PathBuf,
    pub checkpoint_path: PathBuf,
    pub bind_endpoint: SocketAddr,
    pub connect_endpoint: SocketAddr,
}

/// Operator-visible summary of the live HTTP miner axon server.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AxonServeReport {
    pub game: GameSelection,
    pub bind_endpoint: String,
    pub connect_endpoint: String,
    pub checkpoint_path: PathBuf,
    pub epochs: usize,
}

/// Errors returned while preparing or serving the live HTTP miner axon.
#[derive(Debug, Error)]
pub enum AxonServeError {
    /// Returned when HTTP serving is requested without the required encoder directory.
    #[error("--serve-http requires --encoder-dir")]
    MissingEncoderDir,

    /// Returned when HTTP serving has no checkpoint to load.
    #[error("--serve-http requires --checkpoint or --train-iterations")]
    MissingCheckpoint,

    /// Returned when a game has no live HTTP path yet.
    #[error("--serve-http is not implemented yet for --game liars-dice")]
    UnsupportedGame,

    /// Returned when the encoder artifact directory fails to load.
    #[error("failed to load encoder directory `{path}`: {source}")]
    Encoder {
        path: String,
        #[source]
        source: ArtifactCodecError,
    },

    /// Returned when the solver checkpoint fails to load.
    #[error("failed to load strategy checkpoint `{path}`: {source}")]
    SolverLoad {
        path: String,
        #[source]
        source: PokerSolverError,
    },

    /// Returned when the axon TCP listener cannot bind.
    #[error("failed to bind HTTP axon `{endpoint}`: {source}")]
    Bind {
        endpoint: String,
        #[source]
        source: std::io::Error,
    },

    /// Returned when the listener cannot accept the next connection.
    #[error("failed to accept HTTP axon connection on `{endpoint}`: {source}")]
    Accept {
        endpoint: String,
        #[source]
        source: std::io::Error,
    },
}

/// Loaded live HTTP miner axon, ready to serve requests.
pub struct LoadedAxonServer {
    listener: TcpListener,
    solver: PokerSolver,
    report: AxonServeReport,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct HttpRequest {
    method: String,
    path: String,
    body: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct HttpResponse {
    status_code: u16,
    reason: &'static str,
    content_type: &'static str,
    body: Vec<u8>,
}

#[derive(Debug, Error)]
enum RequestReadError {
    #[error("failed to read request bytes: {0}")]
    Read(#[source] std::io::Error),

    #[error("request exceeded {REQUEST_LIMIT_BYTES} byte limit")]
    RequestTooLarge,

    #[error("request headers were not valid UTF-8")]
    HeaderEncoding,

    #[error("missing HTTP request line")]
    MissingRequestLine,

    #[error("invalid HTTP request line `{0}`")]
    InvalidRequestLine(String),

    #[error("invalid content-length header `{0}`")]
    InvalidContentLength(String),
}

/// Builds an optional live HTTP serving plan from the current CLI flags.
///
/// Args:
///     cli: Parsed miner CLI arguments.
///     checkpoint_hint: Fresh checkpoint path produced earlier in the same run.
///
/// Returns:
///     `Ok(None)` when live HTTP serving was not requested, otherwise the
///     validated `AxonServePlan`.
pub fn axon_plan_from_cli(
    cli: &Cli,
    checkpoint_hint: Option<&Path>,
) -> Result<Option<AxonServePlan>, AxonServeError> {
    if !cli.serve_http {
        return Ok(None);
    }

    let Some(encoder_dir) = cli.encoder_dir.clone() else {
        if cli.game == GameSelection::Poker {
            return Err(AxonServeError::MissingEncoderDir);
        }
        return Err(AxonServeError::UnsupportedGame);
    };
    if cli.game == GameSelection::LiarsDice {
        return Err(AxonServeError::UnsupportedGame);
    }
    let checkpoint_path = cli
        .checkpoint
        .clone()
        .or_else(|| checkpoint_hint.map(Path::to_path_buf))
        .ok_or(AxonServeError::MissingCheckpoint)?;
    let bind_endpoint = SocketAddr::from((Ipv4Addr::UNSPECIFIED, cli.port));
    let connect_endpoint = SocketAddr::from((Ipv4Addr::LOCALHOST, cli.port));

    Ok(Some(AxonServePlan {
        game: cli.game,
        encoder_dir,
        checkpoint_path,
        bind_endpoint,
        connect_endpoint,
    }))
}

/// Loads a checkpoint-backed live HTTP miner axon.
///
/// Args:
///     plan: Validated serving plan from the miner CLI.
///
/// Returns:
///     A loaded server with a bound listener plus an operator-facing report.
pub async fn load_axon_server(plan: &AxonServePlan) -> Result<LoadedAxonServer, AxonServeError> {
    let encoder =
        load_encoder_dir(&plan.encoder_dir).map_err(|source| AxonServeError::Encoder {
            path: plan.encoder_dir.display().to_string(),
            source,
        })?;
    let solver = PokerSolver::load(&plan.checkpoint_path, encoder).map_err(|source| {
        AxonServeError::SolverLoad {
            path: plan.checkpoint_path.display().to_string(),
            source,
        }
    })?;
    let listener = TcpListener::bind(plan.bind_endpoint)
        .await
        .map_err(|source| AxonServeError::Bind {
            endpoint: plan.bind_endpoint.to_string(),
            source,
        })?;
    let bound_endpoint = listener
        .local_addr()
        .map_err(|source| AxonServeError::Bind {
            endpoint: plan.bind_endpoint.to_string(),
            source,
        })?;
    let connect_endpoint = connect_endpoint_for_bind(SocketAddr::new(
        plan.connect_endpoint.ip(),
        bound_endpoint.port(),
    ));
    let report = AxonServeReport {
        game: plan.game,
        bind_endpoint: bound_endpoint.to_string(),
        connect_endpoint: connect_endpoint.to_string(),
        checkpoint_path: plan.checkpoint_path.clone(),
        epochs: solver.epochs(),
    };

    Ok(LoadedAxonServer {
        listener,
        solver,
        report,
    })
}

impl LoadedAxonServer {
    /// Return the stable operator-facing startup report for this loaded server.
    pub const fn report(&self) -> &AxonServeReport {
        &self.report
    }

    /// Serve live HTTP miner requests until the process exits.
    pub async fn serve(self) -> Result<(), AxonServeError> {
        loop {
            let endpoint = self.report.bind_endpoint.clone();
            let (mut stream, _) = self
                .listener
                .accept()
                .await
                .map_err(|source| AxonServeError::Accept { endpoint, source })?;
            let response = match read_http_request(&mut stream).await {
                Ok(request) => response_for_request(&self.solver, request),
                Err(error) => {
                    warn!(error = %error, "rejected invalid miner axon request");
                    HttpResponse::text(400, "Bad Request", error.to_string())
                }
            };
            if let Err(error) = write_http_response(&mut stream, response).await {
                warn!(error = %error, "failed to write miner axon response");
            }
        }
    }
}

fn response_for_request(solver: &PokerSolver, request: HttpRequest) -> HttpResponse {
    match (request.method.as_str(), request.path.as_str()) {
        ("GET", "/health") => HttpResponse::json(format!(
            "{{\"status\":\"ok\",\"epochs\":{}}}",
            solver.epochs()
        )),
        ("POST", "/strategy") => response_for_strategy(solver, request.body),
        _ => HttpResponse::text(404, "Not Found", "unknown miner axon route".to_string()),
    }
}

fn response_for_strategy(solver: &PokerSolver, body: Vec<u8>) -> HttpResponse {
    let query = match decode_strategy_query(&body) {
        Ok(query) => query,
        Err(error) => {
            return HttpResponse::text(400, "Bad Request", format_strategy_error(&error));
        }
    };
    let response = solver.answer(query);
    match encode_strategy_response(&response) {
        Ok(body) => HttpResponse::binary(200, "OK", body),
        Err(error) => {
            HttpResponse::text(500, "Internal Server Error", format_strategy_error(&error))
        }
    }
}

fn read_prefix(bytes: &[u8], len: usize) -> Result<&[u8], RequestReadError> {
    bytes.get(..len).ok_or_else(|| {
        RequestReadError::Read(std::io::Error::other(
            "request parser read beyond available prefix",
        ))
    })
}

fn split_header_and_body(
    buffer: &[u8],
    index: usize,
) -> Result<(Vec<u8>, Vec<u8>), RequestReadError> {
    let header = buffer.get(..index).ok_or_else(|| {
        RequestReadError::Read(std::io::Error::other("request header offset out of bounds"))
    })?;
    let body_start = index.checked_add(4).ok_or_else(|| {
        RequestReadError::Read(std::io::Error::other("request body offset overflow"))
    })?;
    let body = buffer.get(body_start..).ok_or_else(|| {
        RequestReadError::Read(std::io::Error::other("request body offset out of bounds"))
    })?;
    Ok((header.to_vec(), body.to_vec()))
}

async fn read_http_request(stream: &mut TcpStream) -> Result<HttpRequest, RequestReadError> {
    let (header_bytes, body_prefix) = read_header_bytes(stream).await?;
    let header_text =
        std::str::from_utf8(&header_bytes).map_err(|_| RequestReadError::HeaderEncoding)?;
    let (method, path, content_length) = parse_request_head(header_text)?;
    let body = read_request_body(stream, body_prefix, content_length).await?;
    Ok(HttpRequest { method, path, body })
}

async fn read_header_bytes(stream: &mut TcpStream) -> Result<(Vec<u8>, Vec<u8>), RequestReadError> {
    let mut buffer = Vec::new();
    loop {
        let mut chunk = [0_u8; 1024];
        let read = stream
            .read(&mut chunk)
            .await
            .map_err(RequestReadError::Read)?;
        if read == 0 {
            return Err(RequestReadError::MissingRequestLine);
        }
        buffer.extend_from_slice(read_prefix(&chunk, read)?);
        if buffer.len() > REQUEST_LIMIT_BYTES {
            return Err(RequestReadError::RequestTooLarge);
        }
        if let Some(index) = header_end(&buffer) {
            let (header, body) = split_header_and_body(&buffer, index)?;
            return Ok((header, body));
        }
    }
}

fn header_end(bytes: &[u8]) -> Option<usize> {
    bytes.windows(4).position(|window| window == b"\r\n\r\n")
}

fn parse_request_head(header_text: &str) -> Result<(String, String, usize), RequestReadError> {
    let mut lines = header_text.lines();
    let request_line = lines.next().ok_or(RequestReadError::MissingRequestLine)?;
    let mut parts = request_line.split_whitespace();
    let method = parts
        .next()
        .map(ToOwned::to_owned)
        .ok_or_else(|| RequestReadError::InvalidRequestLine(request_line.to_string()))?;
    let path = parts
        .next()
        .map(ToOwned::to_owned)
        .ok_or_else(|| RequestReadError::InvalidRequestLine(request_line.to_string()))?;
    let Some(version) = parts.next() else {
        return Err(RequestReadError::InvalidRequestLine(
            request_line.to_string(),
        ));
    };
    if !version.starts_with("HTTP/1.") || parts.next().is_some() {
        return Err(RequestReadError::InvalidRequestLine(
            request_line.to_string(),
        ));
    }

    let content_length = parse_content_length(lines)?;
    Ok((method, path, content_length))
}

fn parse_content_length<'a>(
    headers: impl Iterator<Item = &'a str>,
) -> Result<usize, RequestReadError> {
    for header in headers {
        let Some((name, value)) = header.split_once(':') else {
            continue;
        };
        if !name.trim().eq_ignore_ascii_case("content-length") {
            continue;
        }
        return usize::from_str(value.trim())
            .map_err(|_| RequestReadError::InvalidContentLength(value.trim().to_string()));
    }
    Ok(0)
}

async fn read_request_body(
    stream: &mut TcpStream,
    mut body: Vec<u8>,
    content_length: usize,
) -> Result<Vec<u8>, RequestReadError> {
    if content_length > REQUEST_LIMIT_BYTES {
        return Err(RequestReadError::RequestTooLarge);
    }
    while body.len() < content_length {
        let remaining = content_length.checked_sub(body.len()).ok_or_else(|| {
            RequestReadError::Read(std::io::Error::other(
                "request body length underflow while reading",
            ))
        })?;
        let mut chunk = vec![0_u8; remaining.min(1024)];
        let read = stream
            .read(&mut chunk)
            .await
            .map_err(RequestReadError::Read)?;
        if read == 0 {
            return Err(RequestReadError::MissingRequestLine);
        }
        body.extend_from_slice(read_prefix(&chunk, read)?);
    }
    body.truncate(content_length);
    Ok(body)
}

async fn write_http_response(
    stream: &mut TcpStream,
    response: HttpResponse,
) -> Result<(), std::io::Error> {
    let header = format!(
        "HTTP/1.1 {} {}\r\nContent-Length: {}\r\nContent-Type: {}\r\nConnection: close\r\n\r\n",
        response.status_code,
        response.reason,
        response.body.len(),
        response.content_type,
    );
    stream.write_all(header.as_bytes()).await?;
    stream.write_all(&response.body).await?;
    stream.shutdown().await
}

fn format_strategy_error(error: &WireCodecError) -> String {
    error.to_string()
}

impl HttpResponse {
    fn text(status_code: u16, reason: &'static str, body: String) -> Self {
        Self {
            status_code,
            reason,
            content_type: "text/plain; charset=utf-8",
            body: body.into_bytes(),
        }
    }

    fn json(body: String) -> Self {
        Self {
            status_code: 200,
            reason: "OK",
            content_type: "application/json",
            body: body.into_bytes(),
        }
    }

    fn binary(status_code: u16, reason: &'static str, body: Vec<u8>) -> Self {
        Self {
            status_code,
            reason,
            content_type: "application/octet-stream",
            body,
        }
    }
}

fn connect_endpoint_for_bind(endpoint: SocketAddr) -> SocketAddr {
    match endpoint.ip() {
        IpAddr::V4(ip) if ip.is_unspecified() => {
            SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), endpoint.port())
        }
        _ => endpoint,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::time::{SystemTime, UNIX_EPOCH};

    use myosu_games_poker::{
        NlheAbstractionStreet, NlheInfoKey, NlheStrategyQuery, RbpNlheEncoder, write_encoder_dir,
    };
    use rbp_cards::{Isomorphism, Observation};
    use rbp_gameplay::Abstraction;

    use super::*;

    #[test]
    fn plan_requires_checkpoint_for_http_serving() {
        let cli = Cli {
            chain: "ws://127.0.0.1:9944".to_string(),
            subnet: 1,
            key: Some("//Alice".to_string()),
            key_config_dir: None,
            key_password_env: "MYOSU_KEY_PASSWORD".to_string(),
            port: 8080,
            register: false,
            serve_axon: false,
            serve_http: true,
            data_dir: PathBuf::from("/tmp/miner-data"),
            game: GameSelection::Poker,
            encoder_dir: Some(PathBuf::from("/tmp/encoder")),
            checkpoint: None,
            train_iterations: 0,
            query_file: None,
            response_file: None,
        };

        let error =
            axon_plan_from_cli(&cli, None).expect_err("http serving should require a checkpoint");
        assert!(matches!(error, AxonServeError::MissingCheckpoint));
    }

    #[tokio::test]
    async fn server_answers_health_and_strategy() {
        let root = unique_temp_root();
        let encoder_dir = root.join("encoder");
        let checkpoint_path = root.join("checkpoints").join("latest.bin");
        std::fs::create_dir_all(
            checkpoint_path
                .parent()
                .expect("checkpoint dir should exist"),
        )
        .expect("checkpoint dir should write");
        write_encoder_dir(&encoder_dir, sample_encoder_streets())
            .expect("encoder dir should write");
        PokerSolver::new(RbpNlheEncoder::default())
            .save(&checkpoint_path)
            .expect("checkpoint should save");

        let plan = AxonServePlan {
            game: GameSelection::Poker,
            encoder_dir,
            checkpoint_path,
            bind_endpoint: SocketAddr::from((Ipv4Addr::LOCALHOST, 0)),
            connect_endpoint: SocketAddr::from((Ipv4Addr::LOCALHOST, 0)),
        };
        let server = load_axon_server(&plan)
            .await
            .expect("server should load from checkpoint");
        let endpoint = server.report().connect_endpoint.clone();
        let task = tokio::spawn(server.serve());

        let health = issue_request(
            &endpoint,
            b"GET /health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
        )
        .await;
        assert!(health.starts_with(b"HTTP/1.1 200 OK"));
        assert!(String::from_utf8_lossy(&health).contains("\"status\":\"ok\""));

        let query = NlheStrategyQuery::new(NlheInfoKey {
            subgame: 0,
            bucket: 0,
            choices: 0,
        });
        let query_body =
            myosu_games_poker::encode_strategy_query(&query).expect("query should encode");
        let request = format!(
            "POST /strategy HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            query_body.len()
        );
        let mut strategy_request = request.into_bytes();
        strategy_request.extend_from_slice(&query_body);
        let strategy = issue_request(&endpoint, &strategy_request).await;
        assert!(strategy.starts_with(b"HTTP/1.1 200 OK"));

        task.abort();
        let _ = std::fs::remove_dir_all(root);
    }

    async fn issue_request(endpoint: &str, request: &[u8]) -> Vec<u8> {
        let mut stream = TcpStream::connect(endpoint)
            .await
            .expect("server should accept connection");
        stream
            .write_all(request)
            .await
            .expect("request should write");
        let mut response = Vec::new();
        stream
            .read_to_end(&mut response)
            .await
            .expect("response should read");
        response
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
            "myosu-miner-axon-test-{}-{nanos}",
            std::process::id()
        ))
    }

    #[test]
    fn liars_dice_http_plan_is_explicitly_unsupported() {
        let cli = Cli {
            chain: "ws://127.0.0.1:9944".to_string(),
            subnet: 1,
            key: Some("//Alice".to_string()),
            key_config_dir: None,
            key_password_env: "MYOSU_KEY_PASSWORD".to_string(),
            port: 8080,
            register: false,
            serve_axon: false,
            serve_http: true,
            data_dir: PathBuf::from("/tmp/miner-data"),
            game: GameSelection::LiarsDice,
            encoder_dir: None,
            checkpoint: Some(PathBuf::from("/tmp/checkpoint.bin")),
            train_iterations: 0,
            query_file: None,
            response_file: None,
        };

        let error =
            axon_plan_from_cli(&cli, None).expect_err("liar's dice http serving should be gated");
        assert!(matches!(error, AxonServeError::UnsupportedGame));
    }
}

use std::io;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::str::FromStr;
use std::time::Duration;

use myosu_games_poker::{
    NlheRenderer, RbpNlheEdge, decode_strategy_response, encode_strategy_query, recommended_edge,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;

use crate::discovery::DiscoveredMiner;

const LIVE_QUERY_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiveMinerStrategy {
    pub advertised_endpoint: String,
    pub connect_endpoint: String,
    pub action_count: usize,
    pub recommended_edge: String,
    pub recommended_action: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct HttpResponse {
    status_code: u16,
    body: Vec<u8>,
}

pub async fn query_live_miner(
    miner: &DiscoveredMiner,
    renderer: &NlheRenderer,
) -> io::Result<LiveMinerStrategy> {
    let Some(query) = renderer
        .strategy_request()
        .and_then(|request| request.query().ok())
    else {
        return Err(io::Error::other(
            "renderer did not expose a live strategy query",
        ));
    };
    let connect_endpoint = connect_endpoint(&miner.endpoint)?;
    check_health(&connect_endpoint).await?;
    let query_body = encode_strategy_query(&query).map_err(io::Error::other)?;
    let response = post_strategy(&connect_endpoint, &query_body).await?;
    let strategy = decode_strategy_response(&response.body).map_err(io::Error::other)?;
    let recommended = recommended_edge(&strategy);
    let recommended_edge = recommended
        .map(|edge| format!("{edge:?}"))
        .unwrap_or_else(|| "none".to_string());
    let recommended_action = recommended
        .and_then(|edge: RbpNlheEdge| renderer.recommendation_action(edge))
        .unwrap_or("unknown")
        .to_string();

    Ok(LiveMinerStrategy {
        advertised_endpoint: miner.endpoint.clone(),
        connect_endpoint,
        action_count: strategy.actions.len(),
        recommended_edge,
        recommended_action,
    })
}

fn connect_endpoint(endpoint: &str) -> io::Result<String> {
    let endpoint = SocketAddr::from_str(endpoint).map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("invalid discovered miner endpoint `{endpoint}`: {error}"),
        )
    })?;
    let host = match endpoint.ip() {
        IpAddr::V4(ip) if ip.is_unspecified() => IpAddr::V4(Ipv4Addr::LOCALHOST),
        IpAddr::V6(ip) if ip.is_unspecified() => IpAddr::V6(Ipv6Addr::LOCALHOST),
        ip => ip,
    };
    Ok(SocketAddr::new(host, endpoint.port()).to_string())
}

async fn check_health(endpoint: &str) -> io::Result<()> {
    let response = issue_request(
        endpoint,
        b"GET /health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
    )
    .await?;
    if response.status_code != 200 {
        return Err(io::Error::other(format!(
            "miner health check failed with status {}",
            response.status_code
        )));
    }
    let body = String::from_utf8(response.body).map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("miner health response was not UTF-8: {error}"),
        )
    })?;
    if !body.contains("\"status\":\"ok\"") {
        return Err(io::Error::other(format!(
            "miner health response was not ok: {body}"
        )));
    }
    Ok(())
}

async fn post_strategy(endpoint: &str, body: &[u8]) -> io::Result<HttpResponse> {
    let mut request = format!(
        "POST /strategy HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    )
    .into_bytes();
    request.extend_from_slice(body);
    issue_request(endpoint, &request).await
}

async fn issue_request(endpoint: &str, request: &[u8]) -> io::Result<HttpResponse> {
    let mut stream = timeout(LIVE_QUERY_TIMEOUT, TcpStream::connect(endpoint))
        .await
        .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "timed out connecting to miner"))?
        .map_err(|error| io::Error::other(format!("failed to connect to miner: {error}")))?;
    timeout(LIVE_QUERY_TIMEOUT, stream.write_all(request))
        .await
        .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "timed out writing miner request"))?
        .map_err(|error| io::Error::other(format!("failed to write miner request: {error}")))?;
    let mut response = Vec::new();
    timeout(LIVE_QUERY_TIMEOUT, stream.read_to_end(&mut response))
        .await
        .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "timed out reading miner reply"))?
        .map_err(|error| io::Error::other(format!("failed to read miner reply: {error}")))?;
    parse_http_response(&response)
}

fn parse_http_response(response: &[u8]) -> io::Result<HttpResponse> {
    let Some(index) = response.windows(4).position(|window| window == b"\r\n\r\n") else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "miner response missing HTTP header terminator",
        ));
    };
    let header = std::str::from_utf8(&response[..index]).map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("miner response headers were not UTF-8: {error}"),
        )
    })?;
    let status_code = parse_status_code(header)?;
    Ok(HttpResponse {
        status_code,
        body: response[index + 4..].to_vec(),
    })
}

fn parse_status_code(header: &str) -> io::Result<u16> {
    let status_line = header.lines().next().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "miner response missing HTTP status line",
        )
    })?;
    let mut parts = status_line.split_whitespace();
    let Some(version) = parts.next() else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "miner response had empty HTTP status line",
        ));
    };
    if !version.starts_with("HTTP/1.") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("miner response used unsupported HTTP version `{version}`"),
        ));
    }
    let Some(code) = parts.next() else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("miner response missing status code in `{status_line}`"),
        ));
    };
    code.parse::<u16>().map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("miner response had invalid status code `{code}`: {error}"),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::connect_endpoint;
    use super::parse_http_response;

    #[test]
    fn rewrites_unspecified_ipv4_endpoint_to_loopback() {
        let endpoint =
            connect_endpoint("0.0.0.0:8080").expect("unspecified endpoint should normalize");

        assert_eq!(endpoint, "127.0.0.1:8080");
    }

    #[test]
    fn parses_http_response_status_and_body() {
        let response = parse_http_response(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok")
            .expect("response should parse");

        assert_eq!(response.status_code, 200);
        assert_eq!(response.body, b"ok");
    }
}

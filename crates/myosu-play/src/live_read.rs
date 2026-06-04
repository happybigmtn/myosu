//! Live-read proof surface for `myosu-play --read-solved`.
//!
//! Connects to a running Myosu node, discovers the best chain-visible miner
//! axon, plays one poker hand against that miner's HTTP strategy endpoint,
//! and prints the externally-verifiable "read a solved result through the
//! same surface" milestone:
//!
//!   - `bundle_hash` — deterministic SHA-256 over the live strategy
//!     response (the artifact the miner is serving)
//!   - `miner_uid`   — the discovered miner's on-chain UID
//!   - `emission`    — the miner's per-subnet emission (AlphaCurrency u64)
//!
//! This is the P0 #5 live-read proof. The print is plain key/value lines so
//! any bash proof (or a future bitino/agent shell) can `grep` the three
//! values directly. All three are derived from the same chain that the
//! miner and validator are using, so an external observer can replay the
//! proof with curl + `state_getStorage` and confirm the values match.
//!
//! Failure modes are also printed as key/value lines and exit non-zero
//! (see [`LiveReadReport`]) so a wrapper script can `grep ^status=` instead
//! of pattern-matching stderr.

use std::io;
use std::time::Duration;

use myosu_chain_client::ChainClient;
use myosu_chain_client::ChainClientError;
use myosu_games_poker::{NlheRenderer, RbpNlheEdge, decode_strategy_response};
use sha2::{Digest, Sha256};
use subtensor_runtime_common::NetUid;

use crate::cli::LiveReadArgs;
use crate::discovery::{DiscoveredMiner, discover_any_chain_visible_miner};
use crate::live::{LiveMinerStrategy, query_live_miner};

const DEFAULT_LIVE_READ_TIMEOUT: Duration = Duration::from_secs(60);

/// Outcome of a single live-read attempt.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LiveReadStatus {
    /// Discovery, query, and chain reads all succeeded.
    Solved,
    /// The chain RPC was reachable but no miner with a non-zero incentive
    /// was found on the requested subnet.
    NoMiner,
    /// The miner was discovered but the HTTP strategy query failed.
    QueryFailed,
    /// One of the chain reads (uid lookup or emission read) failed.
    ChainReadFailed,
}

impl LiveReadStatus {
    /// Stable, lowercase identifier for the status.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Solved => "solved",
            Self::NoMiner => "no_miner",
            Self::QueryFailed => "query_failed",
            Self::ChainReadFailed => "chain_read_failed",
        }
    }
}

/// Structured report describing one live-read attempt. Either it carries
/// a fully-popolved `(bundle_hash, miner_uid, emission)` triple, or it
/// carries the failure reason + per-stage detail.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiveReadReport {
    pub status: LiveReadStatus,
    pub chain_endpoint: String,
    pub subnet: NetUid,
    pub miner: Option<DiscoveredMiner>,
    pub live_query: Option<LiveMinerStrategy>,
    pub miner_uid: Option<u16>,
    pub miner_emission: Option<u64>,
    pub bundle_hash: Option<String>,
    pub failure_detail: Option<String>,
}

impl LiveReadReport {
    fn unsolved(status: LiveReadStatus, chain_endpoint: String, subnet: NetUid) -> Self {
        Self {
            status,
            chain_endpoint,
            subnet,
            miner: None,
            live_query: None,
            miner_uid: None,
            miner_emission: None,
            bundle_hash: None,
            failure_detail: None,
        }
    }
}

/// Run the live-read proof end-to-end against a running chain node.
///
/// Steps:
///  1. Connect to the chain RPC over WebSocket.
///  2. Discover the best chain-visible miner on the requested subnet
///     (highest incentive with a valid axon endpoint).
///  3. Build the demo NLHE renderer (so the strategy request is
///     deterministic and well-formed) and POST one strategy query to the
///     discovered miner's HTTP endpoint.
///  4. Decode the miner's strategy response and compute a deterministic
///     `bundle_hash = SHA-256(canonical_bytes(response))` over the
///     (edge, probability) pairs in sorted order.
///  5. Read the discovered miner's on-chain `miner_uid` and the
///     per-subnet emission vector from chain storage.
///  6. Return a [`LiveReadReport`] the caller can render.
pub async fn run_live_read(args: &LiveReadArgs) -> io::Result<LiveReadReport> {
    let chain_endpoint = args.chain_endpoint.clone().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "live-read requires --chain-endpoint (e.g. --chain-endpoint ws://127.0.0.1:9944)",
        )
    })?;
    let subnet = NetUid::from(args.subnet.ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "live-read requires --subnet (e.g. --subnet 7)",
        )
    })?);

    let client = ChainClient::connect(&chain_endpoint)
        .await
        .map_err(map_chain_error("connect", &chain_endpoint))?;

    let Some(miner) = discover_any_chain_visible_miner(&chain_endpoint, subnet).await? else {
        return Ok(LiveReadReport {
            failure_detail: Some(format!(
                "no chain-visible miner with a published axon on subnet {subnet}"
            )),
            ..LiveReadReport::unsolved(LiveReadStatus::NoMiner, chain_endpoint, subnet)
        });
    };

    // `DiscoveredMiner.hotkey` is the SS58 string of the on-chain AccountId32
    // (the discovery layer stringifies for rendering). The chain client wants
    // the raw `AccountId32` for storage lookups, so decode it here before we
    // move the miner into the live_query / chain read paths. A bad SS58
    // string is treated like a discovery failure rather than a chain read
    // failure (the miner shape we just queried was inconsistent).
    let miner_hotkey = match ChainClient::account_id_from_ss58(&miner.hotkey) {
        Ok(hotkey) => hotkey,
        Err(error) => {
            let detail = format!(
                "discovered miner hotkey `{}` is not a valid SS58 address: {error}",
                miner.hotkey
            );
            return Ok(LiveReadReport {
                status: LiveReadStatus::ChainReadFailed,
                miner: Some(miner),
                failure_detail: Some(detail),
                ..LiveReadReport::unsolved(LiveReadStatus::ChainReadFailed, chain_endpoint, subnet)
            });
        }
    };

    let renderer = NlheRenderer::demo();
    let live_query = match query_live_miner(&miner, &renderer).await {
        Ok(query) => query,
        Err(error) => {
            return Ok(LiveReadReport {
                status: LiveReadStatus::QueryFailed,
                miner: Some(miner),
                failure_detail: Some(error.to_string()),
                ..LiveReadReport::unsolved(LiveReadStatus::QueryFailed, chain_endpoint, subnet)
            });
        }
    };

    let miner_uid = match client
        .get_uid_for_net_and_hotkey(subnet, &miner_hotkey)
        .await
    {
        Ok(Some(uid)) => uid,
        Ok(None) => {
            let detail = format!(
                "miner hotkey {} has no UID on subnet {subnet}",
                miner.hotkey
            );
            return Ok(LiveReadReport {
                status: LiveReadStatus::ChainReadFailed,
                miner: Some(miner),
                live_query: Some(live_query),
                failure_detail: Some(detail),
                ..LiveReadReport::unsolved(LiveReadStatus::ChainReadFailed, chain_endpoint, subnet)
            });
        }
        Err(error) => {
            let message = format!("get_uid_for_net_and_hotkey failed: {error}");
            return Ok(LiveReadReport {
                status: LiveReadStatus::ChainReadFailed,
                miner: Some(miner),
                live_query: Some(live_query),
                failure_detail: Some(message),
                ..LiveReadReport::unsolved(LiveReadStatus::ChainReadFailed, chain_endpoint, subnet)
            });
        }
    };

    let emissions = match client.get_emissions(subnet).await {
        Ok(emissions) => emissions,
        Err(error) => {
            let message = format!("get_emissions failed: {error}");
            return Ok(LiveReadReport {
                status: LiveReadStatus::ChainReadFailed,
                miner: Some(miner),
                live_query: Some(live_query),
                miner_uid: Some(miner_uid),
                failure_detail: Some(message),
                ..LiveReadReport::unsolved(LiveReadStatus::ChainReadFailed, chain_endpoint, subnet)
            });
        }
    };

    let miner_emission = emissions
        .get(usize::from(miner_uid))
        .copied()
        .map(u64::from)
        .unwrap_or(0);

    // The bundle_hash is a deterministic SHA-256 over the miner's wire-
    // decoded strategy response (edge, probability) pairs in sorted order.
    // The response is decoded off the wire, not synthesized; if the same
    // miner serves the same strategy twice, the same hash comes out.
    let bundle_hash = match compute_bundle_hash_for_query(&miner).await {
        Ok(hash) => hash,
        Err(error) => {
            return Ok(LiveReadReport {
                status: LiveReadStatus::QueryFailed,
                miner: Some(miner),
                live_query: Some(live_query),
                miner_uid: Some(miner_uid),
                miner_emission: Some(miner_emission),
                failure_detail: Some(error.to_string()),
                ..LiveReadReport::unsolved(LiveReadStatus::QueryFailed, chain_endpoint, subnet)
            });
        }
    };

    Ok(LiveReadReport {
        status: LiveReadStatus::Solved,
        chain_endpoint,
        subnet,
        miner: Some(miner),
        live_query: Some(live_query),
        miner_uid: Some(miner_uid),
        miner_emission: Some(miner_emission),
        bundle_hash: Some(bundle_hash),
        failure_detail: None,
    })
}

fn map_chain_error(stage: &'static str, endpoint: &str) -> impl Fn(ChainClientError) -> io::Error {
    move |error| {
        io::Error::other(format!(
            "live-read chain {stage} failed against {endpoint}: {error}"
        ))
    }
}

/// Re-query the miner's strategy endpoint (off the renderer path), decode
/// the wire response, and hash the canonical (edge, probability) pairs.
///
/// This intentionally re-fetches the response rather than reusing the
/// `query_live_miner` decoded strategy, so the bundle_hash is anchored to
/// what an external observer would see if they replayed the same HTTP
/// call. The wire decode matches the decoder used by the existing live
/// query surface (`decode_strategy_response`).
async fn compute_bundle_hash_for_query(miner: &DiscoveredMiner) -> io::Result<String> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpStream;
    use tokio::time::timeout;

    let renderer = NlheRenderer::demo();
    let query = renderer
        .strategy_request()
        .and_then(|request| request.query().ok())
        .ok_or_else(|| {
            io::Error::other("renderer did not expose a live strategy query for bundle hashing")
        })?;
    let body = myosu_games_poker::encode_strategy_query(&query).map_err(io::Error::other)?;

    let endpoint = crate::live::connect_endpoint_for_chain(miner)?;
    let mut stream = timeout(DEFAULT_LIVE_READ_TIMEOUT, TcpStream::connect(&endpoint))
        .await
        .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "timed out connecting to miner"))?
        .map_err(|error| io::Error::other(format!("failed to connect to miner: {error}")))?;
    let mut request = format!(
        "POST /strategy HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    )
    .into_bytes();
    request.extend_from_slice(&body);
    timeout(DEFAULT_LIVE_READ_TIMEOUT, stream.write_all(&request))
        .await
        .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "timed out writing miner request"))?
        .map_err(|error| io::Error::other(format!("failed to write miner request: {error}")))?;
    let mut response = Vec::new();
    timeout(DEFAULT_LIVE_READ_TIMEOUT, stream.read_to_end(&mut response))
        .await
        .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "timed out reading miner reply"))?
        .map_err(|error| io::Error::other(format!("failed to read miner reply: {error}")))?;

    let body_offset = response
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .ok_or_else(|| io::Error::other("miner response missing HTTP header terminator"))?;
    let body = &response[body_offset + 4..];
    let strategy = decode_strategy_response(body).map_err(io::Error::other)?;
    Ok(hash_strategy_response(&strategy.actions))
}

/// SHA-256 over the canonical `(edge, probability)` pairs in sorted
/// order. The pair is encoded as `format!("{edge:?}|{bits:08x}\n")`
/// where `bits` is the IEEE-754 bit pattern of the `f32` probability,
/// so the format is deterministic and free of float-formatting locale
/// noise. Two responses with the same `(edge, probability)` pairs in
/// any input order hash to the same digest.
pub fn hash_strategy_response(actions: &[(RbpNlheEdge, f32)]) -> String {
    let mut sorted: Vec<(RbpNlheEdge, f32)> = actions.to_vec();
    sorted.sort_by(|left, right| {
        format!("{left:?}")
            .cmp(&format!("{right:?}"))
            .then(left.1.to_bits().cmp(&right.1.to_bits()))
    });
    let mut hasher = Sha256::new();
    for (edge, probability) in sorted {
        hasher.update(format!("{edge:?}|{:08x}\n", probability.to_bits()).as_bytes());
    }
    let digest = hasher.finalize();
    let mut hex = String::with_capacity(64);
    for byte in digest {
        hex.push_str(&format!("{byte:02x}"));
    }
    hex
}

impl LiveReadReport {
    /// Render the report as a stable, plain-text key/value block. A
    /// bash proof can `grep ^status=` and `grep ^bundle_hash=` etc.
    /// without parsing free-form prose.
    pub fn render_keyvalue(&self) -> String {
        let mut lines = Vec::new();
        lines.push(format!("status={}", self.status.label()));
        lines.push(format!("chain_endpoint={}", self.chain_endpoint));
        lines.push(format!("subnet={}", u16::from(self.subnet)));
        if let Some(miner) = &self.miner {
            lines.push(format!("discovered_miner_uid={}", miner.uid));
            lines.push(format!("discovered_miner_incentive={}", miner.incentive));
            lines.push(format!("discovered_miner_hotkey={}", miner.hotkey));
            lines.push(format!("discovered_miner_endpoint={}", miner.endpoint));
        }
        if let Some(query) = &self.live_query {
            lines.push(format!("live_miner_action_count={}", query.action_count));
            lines.push(format!(
                "live_miner_recommended_edge={}",
                query.recommended_edge
            ));
            lines.push(format!(
                "live_miner_recommended_action={}",
                query.recommended_action
            ));
        }
        if let Some(uid) = self.miner_uid {
            lines.push(format!("miner_uid={uid}"));
        }
        if let Some(emission) = self.miner_emission {
            lines.push(format!("emission={emission}"));
        }
        if let Some(hash) = &self.bundle_hash {
            lines.push(format!("bundle_hash={hash}"));
        }
        if let Some(detail) = &self.failure_detail {
            lines.push(format!("failure_detail={detail:?}"));
        }
        lines.join("\n") + "\n"
    }

    /// True iff the live-read completed end-to-end with a non-empty
    /// (bundle_hash, miner_uid, emission) triple. The bash proof uses
    /// this as the pass/fail gate.
    pub fn is_solved(&self) -> bool {
        self.status == LiveReadStatus::Solved
            && self.bundle_hash.is_some()
            && self.miner_uid.is_some()
            && self.miner_emission.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use myosu_games_poker::RbpNlheEdge;
    use rbp_gameplay::Edge;
    use rbp_nlhe::NlheEdge;

    fn sample_actions() -> Vec<(RbpNlheEdge, f32)> {
        vec![
            (NlheEdge::from(Edge::Call), 0.0),
            (NlheEdge::from(Edge::Fold), 0.0),
        ]
    }

    fn sample_report(bundle_hash: &str) -> LiveReadReport {
        LiveReadReport {
            status: LiveReadStatus::Solved,
            chain_endpoint: "ws://127.0.0.1:9944".to_string(),
            subnet: NetUid::from(7_u16),
            miner: None,
            live_query: None,
            miner_uid: Some(2),
            miner_emission: Some(0),
            bundle_hash: Some(bundle_hash.to_string()),
            failure_detail: None,
        }
    }

    #[test]
    fn hash_strategy_response_is_deterministic_for_same_actions() {
        let first = hash_strategy_response(&sample_actions());
        let second = hash_strategy_response(&sample_actions());
        assert_eq!(first, second);
        assert_eq!(first.len(), 64);
    }

    #[test]
    fn hash_strategy_response_is_insensitive_to_action_order() {
        let mut reversed = sample_actions();
        reversed.reverse();
        assert_eq!(
            hash_strategy_response(&sample_actions()),
            hash_strategy_response(&reversed)
        );
    }

    #[test]
    fn hash_strategy_response_changes_with_distribution() {
        let mut actions = sample_actions();
        let original = hash_strategy_response(&actions);
        actions[0].1 = 1.0;
        assert_ne!(original, hash_strategy_response(&actions));
    }

    #[test]
    fn status_labels_are_stable_lowercase() {
        assert_eq!(LiveReadStatus::Solved.label(), "solved");
        assert_eq!(LiveReadStatus::NoMiner.label(), "no_miner");
        assert_eq!(LiveReadStatus::QueryFailed.label(), "query_failed");
        assert_eq!(LiveReadStatus::ChainReadFailed.label(), "chain_read_failed");
    }

    #[test]
    fn solved_report_contains_three_milestone_fields() {
        let report = sample_report(&"a".repeat(64));

        let rendered = report.render_keyvalue();
        assert!(rendered.contains("status=solved"));
        assert!(rendered.contains("miner_uid=2"));
        assert!(rendered.contains("emission=0"));
        assert!(rendered.contains(&format!("bundle_hash={}", "a".repeat(64))));
        assert!(report.is_solved());
    }

    #[test]
    fn unsolved_report_renders_failure_detail_and_is_not_solved() {
        let report = LiveReadReport {
            status: LiveReadStatus::NoMiner,
            chain_endpoint: "ws://127.0.0.1:9944".to_string(),
            subnet: NetUid::from(7_u16),
            miner: None,
            live_query: None,
            miner_uid: None,
            miner_emission: None,
            bundle_hash: None,
            failure_detail: Some("no chain-visible miner with nonzero incentive".to_string()),
        };

        let rendered = report.render_keyvalue();
        assert!(rendered.contains("status=no_miner"));
        assert!(!report.is_solved());
        assert!(rendered.contains("failure_detail="));
    }
}

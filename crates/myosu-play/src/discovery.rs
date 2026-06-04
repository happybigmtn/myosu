use std::io;

use myosu_chain_client::ChainClient;
use subtensor_runtime_common::NetUid;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiscoveredMiner {
    pub subnet: NetUid,
    pub uid: u16,
    pub hotkey: String,
    pub incentive: u16,
    pub endpoint: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct DiscoveryCandidate {
    subnet: NetUid,
    uid: u16,
    hotkey: String,
    incentive: u16,
    endpoint: Option<String>,
}

pub async fn discover_best_chain_visible_miner(
    endpoint: &str,
    subnet: NetUid,
) -> io::Result<Option<DiscoveredMiner>> {
    let client = ChainClient::connect(endpoint)
        .await
        .map_err(|error| io::Error::other(format!("failed to connect to chain: {error}")))?;
    let miners = client
        .get_chain_visible_miners(subnet)
        .await
        .map_err(|error| io::Error::other(format!("failed to discover miners: {error}")))?;
    let candidates = miners
        .into_iter()
        .map(|miner| DiscoveryCandidate {
            subnet: miner.subnet,
            uid: miner.uid,
            hotkey: miner.hotkey.to_string(),
            incentive: miner.incentive,
            endpoint: miner.endpoint_hint(),
        })
        .collect();
    Ok(select_best_candidate(candidates))
}

/// Discover the best chain-visible miner for live read, including
/// miners whose `Incentive` is still zero.
///
/// The on-chain `Incentive` vector is only updated at the end of each
/// epoch, so a freshly-registered miner in a test/devnet chain (or any
/// chain that has not yet completed its first post-weights epoch) will
/// have `incentive == 0` even though the miner has a published axon
/// and is otherwise live. The production "best" path
/// (`discover_best_chain_visible_miner`) keeps the strict filter
/// because the live advice surface is only meaningful when the chain
/// has scored the miner; the live-read proof, in contrast, is
/// asserting that we can *read a solved result through the same
/// surface* (bundle_hash/miner_uid/emission), which only needs the
/// miner axon to be present and a strategy response to be available.
///
/// The fallback ranking is the same as the strict path
/// (incentive desc, then uid asc) but accepts zero-incentive
/// candidates as long as their axon endpoint is present.
pub async fn discover_any_chain_visible_miner(
    endpoint: &str,
    subnet: NetUid,
) -> io::Result<Option<DiscoveredMiner>> {
    let client = ChainClient::connect(endpoint)
        .await
        .map_err(|error| io::Error::other(format!("failed to connect to chain: {error}")))?;
    let miner_list = client
        .get_chain_visible_miner_axons(subnet)
        .await
        .map_err(|error| io::Error::other(format!("failed to discover miner axons: {error}")))?;
    let candidates = miner_list
        .into_iter()
        .map(|miner| DiscoveryCandidate {
            subnet: miner.subnet,
            uid: miner.uid,
            hotkey: miner.hotkey.to_string(),
            incentive: miner.incentive,
            endpoint: miner.endpoint_hint(),
        })
        .collect();
    Ok(select_any_candidate_with_endpoint(candidates))
}

fn select_best_candidate(mut candidates: Vec<DiscoveryCandidate>) -> Option<DiscoveredMiner> {
    candidates.sort_by(|left, right| {
        right
            .incentive
            .cmp(&left.incentive)
            .then_with(|| left.uid.cmp(&right.uid))
    });

    candidates.into_iter().find_map(|candidate| {
        if candidate.incentive == 0 {
            return None;
        }
        Some(DiscoveredMiner {
            subnet: candidate.subnet,
            uid: candidate.uid,
            hotkey: candidate.hotkey,
            incentive: candidate.incentive,
            endpoint: candidate.endpoint?,
        })
    })
}

/// Like `select_best_candidate` but accepts zero-incentive candidates
/// as long as a `format_axon_endpoint` is present. Returns the
/// highest-ranked (incentive desc, uid asc) miner that published an
/// axon endpoint, or `None` if no miner has a published axon.
fn select_any_candidate_with_endpoint(
    mut candidates: Vec<DiscoveryCandidate>,
) -> Option<DiscoveredMiner> {
    candidates.sort_by(|left, right| {
        right
            .incentive
            .cmp(&left.incentive)
            .then_with(|| left.uid.cmp(&right.uid))
    });

    candidates.into_iter().find_map(|candidate| {
        Some(DiscoveredMiner {
            subnet: candidate.subnet,
            uid: candidate.uid,
            hotkey: candidate.hotkey,
            incentive: candidate.incentive,
            endpoint: candidate.endpoint?,
        })
    })
}

#[cfg(test)]
mod tests {
    use super::DiscoveryCandidate;
    use super::select_any_candidate_with_endpoint;
    use super::select_best_candidate;
    use subtensor_runtime_common::NetUid;

    fn candidate(uid: u16, incentive: u16, endpoint: Option<&str>) -> DiscoveryCandidate {
        DiscoveryCandidate {
            subnet: NetUid::from(7_u16),
            uid,
            hotkey: format!("hotkey-{uid}"),
            incentive,
            endpoint: endpoint.map(ToOwned::to_owned),
        }
    }

    #[test]
    fn finds_best_miner() {
        let best = select_best_candidate(vec![
            candidate(1, 32000, Some("127.0.0.2:8080")),
            candidate(0, 65535, Some("127.0.0.1:8080")),
        ])
        .expect("best miner should be selected");

        assert_eq!(best.uid, 0);
        assert_eq!(best.incentive, 65535);
        assert_eq!(best.endpoint, "127.0.0.1:8080");
    }

    #[test]
    fn fallback_to_second_best() {
        let best = select_best_candidate(vec![
            candidate(0, 65535, None),
            candidate(1, 32000, Some("127.0.0.2:8080")),
        ])
        .expect("second-best visible miner should be selected");

        assert_eq!(best.uid, 1);
        assert_eq!(best.incentive, 32000);
        assert_eq!(best.endpoint, "127.0.0.2:8080");
    }

    #[test]
    fn no_miners_uses_random() {
        let best = select_best_candidate(vec![
            candidate(0, 0, Some("127.0.0.1:8080")),
            candidate(1, 0, None),
        ]);

        assert_eq!(best, None);
    }

    #[test]
    fn any_picks_zero_incentive_miner_with_axon() {
        // The strict `select_best_candidate` skips zero-incentive
        // candidates; the permissive `select_any_candidate_with_endpoint`
        // accepts them as long as the axon endpoint is present.
        let strict = select_best_candidate(vec![candidate(0, 0, Some("127.0.0.1:8080"))]);
        let permissive =
            select_any_candidate_with_endpoint(vec![candidate(0, 0, Some("127.0.0.1:8080"))])
                .expect("permissive path should accept zero-incentive miner with axon");

        assert_eq!(strict, None);
        assert_eq!(permissive.uid, 0);
        assert_eq!(permissive.incentive, 0);
        assert_eq!(permissive.endpoint, "127.0.0.1:8080");
    }

    #[test]
    fn any_prefers_nonzero_incentive_when_available() {
        // When at least one miner has nonzero incentive, the
        // permissive path still ranks it first (same as strict path).
        let best = select_any_candidate_with_endpoint(vec![
            candidate(0, 0, Some("127.0.0.1:8080")),
            candidate(1, 12345, Some("127.0.0.2:8080")),
        ])
        .expect("nonzero-incentive miner with axon should win");

        assert_eq!(best.uid, 1);
        assert_eq!(best.incentive, 12345);
    }

    #[test]
    fn any_skips_miners_without_axon_endpoint() {
        // The permissive path still requires a published axon
        // endpoint. A zero-incentive miner without an axon is not
        // "live-readable".
        let best =
            select_any_candidate_with_endpoint(vec![candidate(0, 0, None), candidate(1, 0, None)]);

        assert_eq!(best, None);
    }
}

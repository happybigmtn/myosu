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

#[cfg(test)]
mod tests {
    use super::DiscoveryCandidate;
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
}

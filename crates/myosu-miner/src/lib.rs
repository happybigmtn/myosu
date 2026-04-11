pub mod axon;
pub mod chain;
pub mod cli;
pub mod strategy;
pub mod training;

/// Formats a stable operator-facing startup report for the current chain probe.
///
/// Args:
///     report: Result of the initial chain connectivity probe.
///
/// Returns:
///     A multi-line plain-text report suitable for stdout.
pub fn startup_report(report: &crate::chain::ChainProbeReport) -> String {
    format!(
        "MINER myosu-miner bootstrap ok\nchain_endpoint={}\nsubnet={}\npeers={}\nis_syncing={}\nshould_have_peers={}\nrpc_method_count={}\nneuron_lite_bytes={}\n",
        report.endpoint,
        report.subnet,
        report.health.peers,
        report.health.is_syncing,
        report.health.should_have_peers,
        report.rpc_methods.methods.len(),
        report.neuron_lite_bytes.len(),
    )
}

/// Formats a stable operator-facing summary for on-chain miner registration.
pub fn registration_report(report: &myosu_chain_client::RegistrationReport) -> String {
    format!(
        "REGISTRATION myosu-miner subnet ok\nhotkey={}\nsubnet={}\nuid={}\nalready_registered={}\nextrinsic_hash={}\n",
        report.hotkey,
        report.subnet,
        report.uid,
        report.already_registered,
        report.extrinsic_hash.as_deref().unwrap_or("none"),
    )
}

/// Formats a stable operator-facing summary for miner axon publication.
pub fn axon_report(report: &myosu_chain_client::AxonServeReport) -> String {
    format!(
        "AXON myosu-miner publish ok\nhotkey={}\nsubnet={}\nversion={}\nip={}\nport={}\nalready_published={}\nextrinsic_hash={}\n",
        report.hotkey,
        report.subnet,
        report.version,
        report.ip,
        report.port,
        report.already_published,
        report.extrinsic_hash.as_deref().unwrap_or("none"),
    )
}

/// Formats a stable operator-facing summary for the live HTTP miner axon.
pub fn http_axon_report(report: &crate::axon::AxonServeReport) -> String {
    format!(
        "HTTP myosu-miner axon ok\ngame={:?}\nbind_endpoint={}\nconnect_endpoint={}\ncheckpoint_path={}\nepochs={}\n",
        report.game,
        report.bind_endpoint,
        report.connect_endpoint,
        report.checkpoint_path.display(),
        report.epochs,
    )
}

/// Formats a stable operator-facing summary for one bounded training batch.
///
/// Args:
///     report: Result of a bounded solver training batch.
///
/// Returns:
///     A multi-line plain-text report suitable for stdout.
pub fn training_report(report: &crate::training::TrainingRunReport) -> String {
    format!(
        "TRAINING myosu-miner batch ok\ngame={:?}\ncheckpoint_path={}\nepochs={}\nexploitability={}\nquality_summary={}\n",
        report.game,
        report.checkpoint_path.display(),
        report.epochs,
        report.exploitability,
        report.quality_summary,
    )
}

/// Formats a stable operator-facing summary for one bounded strategy response.
///
/// Args:
///     report: Result of a single strategy-response batch.
///
/// Returns:
///     A multi-line plain-text report suitable for stdout.
pub fn strategy_report(report: &crate::strategy::StrategyServeReport) -> String {
    format!(
        "STRATEGY myosu-miner query ok\ngame={:?}\nresponse_path={}\naction_count={}\nrecommended_action={}\nquality_summary={}\n",
        report.game,
        report.response_path.display(),
        report.action_count,
        report.recommended_action,
        report.quality_summary,
    )
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use myosu_chain_client::AxonServeReport;
    use myosu_chain_client::RegistrationReport;
    use myosu_chain_client::RpcMethods;
    use myosu_chain_client::SystemHealth;
    use subtensor_runtime_common::NetUid;

    use crate::axon::AxonServeReport as HttpAxonServeReport;
    use crate::axon_report;
    use crate::chain::ChainProbeReport;
    use crate::cli::GameSelection;
    use crate::http_axon_report;
    use crate::registration_report;
    use crate::startup_report;
    use crate::strategy::StrategyServeReport;
    use crate::strategy_report;
    use crate::training::TrainingRunReport;
    use crate::training_report;

    #[test]
    fn startup_report_includes_probe_summary() {
        let report = ChainProbeReport {
            endpoint: "ws://127.0.0.1:9944".to_string(),
            subnet: NetUid::from(1u16),
            health: SystemHealth {
                peers: 0,
                is_syncing: false,
                should_have_peers: false,
            },
            rpc_methods: RpcMethods {
                version: Some(1),
                methods: vec!["system_health".to_string(), "rpc_methods".to_string()],
            },
            neuron_lite_bytes: vec![1, 2, 3],
        };

        let report = startup_report(&report);
        assert!(report.contains("MINER myosu-miner bootstrap ok"));
        assert!(report.contains("subnet=1"));
        assert!(report.contains("rpc_method_count=2"));
        assert!(report.contains("neuron_lite_bytes=3"));
    }

    #[test]
    fn registration_report_includes_registration_summary() {
        let report = RegistrationReport {
            hotkey: myosu_chain_client::ChainClient::account_id_from_uri("//Alice")
                .expect("Alice account should parse"),
            subnet: NetUid::from(1_u16),
            uid: 5,
            extrinsic_hash: Some("0xabc".to_string()),
            already_registered: false,
        };

        let report = registration_report(&report);
        assert!(report.contains("REGISTRATION myosu-miner subnet ok"));
        assert!(report.contains("uid=5"));
        assert!(report.contains("extrinsic_hash=0xabc"));
    }

    #[test]
    fn axon_report_includes_endpoint_summary() {
        let report = AxonServeReport {
            hotkey: myosu_chain_client::ChainClient::account_id_from_uri("//Alice")
                .expect("Alice account should parse"),
            subnet: NetUid::from(1_u16),
            version: 1,
            ip: u128::from(u32::from_be_bytes([127, 0, 0, 1])),
            port: 8080,
            extrinsic_hash: Some("0xdef".to_string()),
            already_published: false,
        };

        let report = axon_report(&report);
        assert!(report.contains("AXON myosu-miner publish ok"));
        assert!(report.contains("port=8080"));
        assert!(report.contains("extrinsic_hash=0xdef"));
    }

    #[test]
    fn http_axon_report_includes_endpoint_summary() {
        let report = HttpAxonServeReport {
            game: GameSelection::Poker,
            bind_endpoint: "0.0.0.0:8080".to_string(),
            connect_endpoint: "127.0.0.1:8080".to_string(),
            checkpoint_path: std::path::PathBuf::from("/tmp/miner/latest.bin"),
            epochs: 12,
        };

        let report = http_axon_report(&report);
        assert!(report.contains("HTTP myosu-miner axon ok"));
        assert!(report.contains("connect_endpoint=127.0.0.1:8080"));
        assert!(report.contains("epochs=12"));
    }

    #[test]
    fn training_report_includes_checkpoint_summary() {
        let report = TrainingRunReport {
            game: GameSelection::LiarsDice,
            checkpoint_path: std::path::PathBuf::from("/tmp/miner/latest.bin"),
            epochs: 12,
            exploitability: "unavailable: sparse encoder".to_string(),
            quality_summary: "engine_tier=dedicated-mccfr engine_family=robopoker-nlhe".to_string(),
        };

        let report = training_report(&report);
        assert!(report.contains("TRAINING myosu-miner batch ok"));
        assert!(report.contains("checkpoint_path=/tmp/miner/latest.bin"));
        assert!(report.contains("epochs=12"));
        assert!(report.contains("quality_summary=engine_tier=dedicated-mccfr"));
    }

    #[test]
    fn strategy_report_includes_response_summary() {
        let report = StrategyServeReport {
            game: GameSelection::LiarsDice,
            response_path: std::path::PathBuf::from("/tmp/miner/response.bin"),
            action_count: 3,
            recommended_action: "Raise(1/2)".to_string(),
            quality_summary: "engine_tier=dedicated-mccfr engine_family=liars-dice-cfr".to_string(),
        };

        let report = strategy_report(&report);
        assert!(report.contains("STRATEGY myosu-miner query ok"));
        assert!(report.contains("response_path=/tmp/miner/response.bin"));
        assert!(report.contains("action_count=3"));
        assert!(report.contains("quality_summary=engine_tier=dedicated-mccfr"));
    }
}

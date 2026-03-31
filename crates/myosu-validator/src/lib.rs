pub mod chain;
pub mod cli;
pub mod validation;

/// Formats a stable operator-facing startup report for the current chain probe.
///
/// Args:
///     report: Result of the initial chain connectivity probe.
///
/// Returns:
///     A multi-line plain-text report suitable for stdout.
pub fn startup_report(report: &crate::chain::ChainProbeReport) -> String {
    format!(
        "VALIDATOR myosu-validator bootstrap ok\nchain_endpoint={}\nsubnet={}\npeers={}\nis_syncing={}\nshould_have_peers={}\nrpc_method_count={}\nneuron_lite_bytes={}\n",
        report.endpoint,
        report.subnet,
        report.health.peers,
        report.health.is_syncing,
        report.health.should_have_peers,
        report.rpc_methods.methods.len(),
        report.neuron_lite_bytes.len(),
    )
}

/// Formats a stable operator-facing summary for on-chain validator registration.
pub fn registration_report(report: &myosu_chain_client::RegistrationReport) -> String {
    format!(
        "REGISTRATION myosu-validator subnet ok\nhotkey={}\nsubnet={}\nuid={}\nalready_registered={}\nextrinsic_hash={}\n",
        report.hotkey,
        report.subnet,
        report.uid,
        report.already_registered,
        report.extrinsic_hash.as_deref().unwrap_or("none"),
    )
}

/// Formats a stable operator-facing summary for subnet staking enablement.
pub fn subtoken_bootstrap_report(report: &crate::chain::SubtokenBootstrapReport) -> String {
    format!(
        "SUBTOKEN myosu-validator subnet ok\nsubnet={}\nalready_enabled={}\nextrinsic_hash={}\n",
        report.subnet,
        report.already_enabled,
        report.extrinsic_hash.as_deref().unwrap_or("none"),
    )
}

/// Formats a stable operator-facing summary for subnet tempo changes.
pub fn subnet_tempo_bootstrap_report(report: &crate::chain::SubnetTempoBootstrapReport) -> String {
    format!(
        "TEMPO myosu-validator subnet ok\nsubnet={}\ntempo={}\nalready_set={}\nextrinsic_hash={}\n",
        report.subnet,
        report.tempo,
        report.already_set,
        report.extrinsic_hash.as_deref().unwrap_or("none"),
    )
}

/// Formats a stable operator-facing summary for subnet weights-rate-limit changes.
pub fn weights_rate_limit_bootstrap_report(
    report: &crate::chain::WeightsRateLimitBootstrapReport,
) -> String {
    format!(
        "WEIGHTS_RATE_LIMIT myosu-validator subnet ok\nsubnet={}\nweights_set_rate_limit={}\nalready_set={}\nextrinsic_hash={}\n",
        report.subnet,
        report.weights_set_rate_limit,
        report.already_set,
        report.extrinsic_hash.as_deref().unwrap_or("none"),
    )
}

/// Formats a stable operator-facing summary for commit-reveal toggles.
pub fn commit_reveal_bootstrap_report(
    report: &crate::chain::CommitRevealBootstrapReport,
) -> String {
    format!(
        "COMMIT_REVEAL myosu-validator subnet ok\nsubnet={}\nenabled={}\nalready_set={}\nextrinsic_hash={}\n",
        report.subnet,
        report.enabled,
        report.already_set,
        report.extrinsic_hash.as_deref().unwrap_or("none"),
    )
}

/// Formats a stable operator-facing summary for stake bootstrap and permit readiness.
pub fn permit_bootstrap_report(report: &crate::chain::ValidatorPermitBootstrapReport) -> String {
    format!(
        "PERMIT myosu-validator ready ok\nhotkey={}\nsubnet={}\nuid={}\nrequested_minimum_stake={}\nfinal_stake={}\nadded_stake={}\nalready_staked={}\nextrinsic_hash={}\n",
        report.hotkey,
        report.subnet,
        report.uid,
        report.requested_minimum_stake,
        report.final_stake,
        report.added_stake,
        report.already_staked,
        report.extrinsic_hash.as_deref().unwrap_or("none"),
    )
}

/// Formats a stable operator-facing summary for on-chain weight submission.
pub fn weight_submission_report(report: &myosu_chain_client::WeightSubmissionReport) -> String {
    format!(
        "WEIGHTS myosu-validator submission ok\nvalidator_hotkey={}\nvalidator_uid={}\ntarget_hotkey={}\ntarget_uid={}\nsubnet={}\nversion_key={}\nmode={}\nalready_submitted={}\nextrinsic_hash={}\n",
        report.validator_hotkey,
        report.validator_uid,
        report.target_hotkey,
        report.target_uid,
        report.subnet,
        report.version_key,
        report.mode,
        report.already_submitted,
        report.extrinsic_hash.as_deref().unwrap_or("none"),
    )
}

/// Formats a stable operator-facing summary for one bounded validation pass.
///
/// Args:
///     report: Result of one validator scoring pass.
///
/// Returns:
///     A multi-line plain-text report suitable for stdout.
pub fn validation_report(report: &crate::validation::ValidationReport) -> String {
    format!(
        "VALIDATION myosu-validator score ok\ngame={:?}\naction_count={}\nexact_match={}\nl1_distance={:.6}\nscore={:.6}\nexpected_action={}\nobserved_action={}\n",
        report.game,
        report.action_count,
        report.exact_match,
        report.l1_distance,
        report.score,
        report.expected_action,
        report.observed_action,
    )
}

#[cfg(test)]
mod tests {
    use myosu_chain_client::RegistrationReport;
    use myosu_chain_client::RpcMethods;
    use myosu_chain_client::SystemHealth;
    use myosu_chain_client::WeightSubmissionReport;
    use subtensor_runtime_common::NetUid;

    use crate::chain::ChainProbeReport;
    use crate::chain::CommitRevealBootstrapReport;
    use crate::chain::SubnetTempoBootstrapReport;
    use crate::chain::SubtokenBootstrapReport;
    use crate::chain::ValidatorPermitBootstrapReport;
    use crate::chain::WeightsRateLimitBootstrapReport;
    use crate::cli::GameSelection;
    use crate::commit_reveal_bootstrap_report;
    use crate::permit_bootstrap_report;
    use crate::registration_report;
    use crate::startup_report;
    use crate::subnet_tempo_bootstrap_report;
    use crate::subtoken_bootstrap_report;
    use crate::validation::ValidationReport;
    use crate::validation_report;
    use crate::weight_submission_report;
    use crate::weights_rate_limit_bootstrap_report;

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
        assert!(report.contains("VALIDATOR myosu-validator bootstrap ok"));
        assert!(report.contains("subnet=1"));
        assert!(report.contains("rpc_method_count=2"));
        assert!(report.contains("neuron_lite_bytes=3"));
    }

    #[test]
    fn registration_report_includes_registration_summary() {
        let report = RegistrationReport {
            hotkey: myosu_chain_client::ChainClient::account_id_from_uri("//Bob")
                .expect("Bob account should parse"),
            subnet: NetUid::from(1_u16),
            uid: 6,
            extrinsic_hash: Some("0x123".to_string()),
            already_registered: false,
        };

        let report = registration_report(&report);
        assert!(report.contains("REGISTRATION myosu-validator subnet ok"));
        assert!(report.contains("uid=6"));
        assert!(report.contains("extrinsic_hash=0x123"));
    }

    #[test]
    fn subtoken_bootstrap_report_includes_enablement_summary() {
        let report = SubtokenBootstrapReport {
            subnet: NetUid::from(1_u16),
            extrinsic_hash: Some("0xsudo".to_string()),
            already_enabled: false,
        };

        let report = subtoken_bootstrap_report(&report);
        assert!(report.contains("SUBTOKEN myosu-validator subnet ok"));
        assert!(report.contains("subnet=1"));
        assert!(report.contains("extrinsic_hash=0xsudo"));
    }

    #[test]
    fn subnet_tempo_bootstrap_report_includes_tempo_summary() {
        let report = SubnetTempoBootstrapReport {
            subnet: NetUid::from(2_u16),
            tempo: 2,
            extrinsic_hash: Some("0xtempo".to_string()),
            already_set: false,
        };

        let report = subnet_tempo_bootstrap_report(&report);
        assert!(report.contains("TEMPO myosu-validator subnet ok"));
        assert!(report.contains("tempo=2"));
        assert!(report.contains("extrinsic_hash=0xtempo"));
    }

    #[test]
    fn weights_rate_limit_bootstrap_report_includes_rate_limit_summary() {
        let report = WeightsRateLimitBootstrapReport {
            subnet: NetUid::from(2_u16),
            weights_set_rate_limit: 0,
            extrinsic_hash: Some("0xweights".to_string()),
            already_set: false,
        };

        let report = weights_rate_limit_bootstrap_report(&report);
        assert!(report.contains("WEIGHTS_RATE_LIMIT myosu-validator subnet ok"));
        assert!(report.contains("weights_set_rate_limit=0"));
        assert!(report.contains("extrinsic_hash=0xweights"));
    }

    #[test]
    fn commit_reveal_bootstrap_report_includes_toggle_summary() {
        let report = CommitRevealBootstrapReport {
            subnet: NetUid::from(2_u16),
            enabled: false,
            extrinsic_hash: Some("0xcommit".to_string()),
            already_set: false,
        };

        let report = commit_reveal_bootstrap_report(&report);
        assert!(report.contains("COMMIT_REVEAL myosu-validator subnet ok"));
        assert!(report.contains("enabled=false"));
        assert!(report.contains("extrinsic_hash=0xcommit"));
    }

    #[test]
    fn permit_bootstrap_report_includes_stake_summary() {
        let report = ValidatorPermitBootstrapReport {
            hotkey: myosu_chain_client::ChainClient::account_id_from_uri("//Bob")
                .expect("Bob account should parse")
                .to_string(),
            subnet: NetUid::from(1_u16),
            uid: 2,
            requested_minimum_stake: 100,
            final_stake: 100,
            added_stake: 25,
            extrinsic_hash: Some("0xstake".to_string()),
            already_staked: false,
        };

        let report = permit_bootstrap_report(&report);
        assert!(report.contains("PERMIT myosu-validator ready ok"));
        assert!(report.contains("requested_minimum_stake=100"));
        assert!(report.contains("added_stake=25"));
        assert!(report.contains("extrinsic_hash=0xstake"));
    }

    #[test]
    fn validation_report_includes_score_summary() {
        let report = ValidationReport {
            game: GameSelection::LiarsDice,
            action_count: 3,
            exact_match: true,
            l1_distance: 0.0,
            score: 1.0,
            expected_action: "Call".to_string(),
            observed_action: "Call".to_string(),
        };

        let report = validation_report(&report);
        assert!(report.contains("VALIDATION myosu-validator score ok"));
        assert!(report.contains("action_count=3"));
        assert!(report.contains("score=1.000000"));
    }

    #[test]
    fn weight_submission_report_includes_submission_summary() {
        let report = WeightSubmissionReport {
            validator_hotkey: myosu_chain_client::ChainClient::account_id_from_uri("//Bob")
                .expect("Bob account should parse"),
            validator_uid: 2,
            target_hotkey: myosu_chain_client::ChainClient::account_id_from_uri("//Alice")
                .expect("Alice account should parse"),
            target_uid: 1,
            subnet: NetUid::from(1_u16),
            version_key: 0,
            mode: "set_weights",
            extrinsic_hash: Some("0xabc".to_string()),
            already_submitted: false,
        };

        let report = weight_submission_report(&report);
        assert!(report.contains("WEIGHTS myosu-validator submission ok"));
        assert!(report.contains("validator_uid=2"));
        assert!(report.contains("target_uid=1"));
        assert!(report.contains("extrinsic_hash=0xabc"));
    }
}

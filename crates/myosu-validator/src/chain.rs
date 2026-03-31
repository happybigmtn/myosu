use myosu_chain_client::ChainClient;
use myosu_chain_client::ChainClientError;
use myosu_chain_client::CommitRevealToggleReport;
use myosu_chain_client::RegistrationReport;
use myosu_chain_client::RpcMethods;
use myosu_chain_client::StakeAddReport;
use myosu_chain_client::SubtokenEnableReport;
use myosu_chain_client::SystemHealth;
use myosu_chain_client::TempoSetReport;
use myosu_chain_client::WeightSubmissionReport;
use myosu_chain_client::WeightsRateLimitReport;
use std::time::Duration;
use subtensor_runtime_common::NetUid;
use thiserror::Error;

/// Startup connectivity summary for the bootstrap validator.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChainProbeReport {
    pub endpoint: String,
    pub subnet: NetUid,
    pub health: SystemHealth,
    pub rpc_methods: RpcMethods,
    pub neuron_lite_bytes: Vec<u8>,
}

/// Explicit permit bootstrap summary for non-self validator weighting.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidatorPermitBootstrapReport {
    pub hotkey: String,
    pub subnet: NetUid,
    pub uid: u16,
    pub requested_minimum_stake: u64,
    pub final_stake: u64,
    pub added_stake: u64,
    pub extrinsic_hash: Option<String>,
    pub already_staked: bool,
}

/// Explicit subnet enablement summary for local devnet bootstrap.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubtokenBootstrapReport {
    pub subnet: NetUid,
    pub extrinsic_hash: Option<String>,
    pub already_enabled: bool,
}

/// Explicit subnet tempo summary for local devnet bootstrap.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubnetTempoBootstrapReport {
    pub subnet: NetUid,
    pub tempo: u16,
    pub extrinsic_hash: Option<String>,
    pub already_set: bool,
}

/// Explicit subnet weights-rate-limit summary for local devnet bootstrap.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WeightsRateLimitBootstrapReport {
    pub subnet: NetUid,
    pub weights_set_rate_limit: u64,
    pub extrinsic_hash: Option<String>,
    pub already_set: bool,
}

/// Explicit commit-reveal toggle summary for local devnet bootstrap.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommitRevealBootstrapReport {
    pub subnet: NetUid,
    pub enabled: bool,
    pub extrinsic_hash: Option<String>,
    pub already_set: bool,
}

/// Errors returned by the validator's initial chain probe.
#[derive(Debug, Error)]
pub enum ChainProbeError {
    /// Returned when the underlying shared chain client fails.
    #[error("failed to probe chain endpoint: {0}")]
    Client(#[from] ChainClientError),
}

/// Errors returned by the validator's on-chain bootstrap actions.
#[derive(Debug, Error)]
pub enum ChainActionError {
    /// Returned when the underlying shared chain client fails.
    #[error("failed to execute validator chain action: {0}")]
    Client(#[from] ChainClientError),
}

/// Connects to the chain and gathers the minimum startup facts the validator needs.
///
/// Args:
///     endpoint: WebSocket RPC endpoint for the node.
///     subnet: Subnet the operator intends to validate on.
///
/// Returns:
///     A `ChainProbeReport` with node health, advertised RPC methods, and the
///     current lite-neuron payload for the requested subnet.
pub async fn probe_chain(
    endpoint: &str,
    subnet: NetUid,
) -> Result<ChainProbeReport, ChainProbeError> {
    let client = ChainClient::connect(endpoint).await?;
    let health = client.system_health().await?;
    let rpc_methods = client.rpc_methods().await?;
    let neuron_lite_bytes = client.neuron_info_get_neurons_lite(subnet).await?;

    Ok(ChainProbeReport {
        endpoint: client.endpoint().to_string(),
        subnet,
        health,
        rpc_methods,
        neuron_lite_bytes,
    })
}

/// Registers the validator hotkey on-chain if it is not already a subnet member.
pub async fn ensure_registered(
    endpoint: &str,
    key_uri: &str,
    subnet: NetUid,
) -> Result<RegistrationReport, ChainActionError> {
    let client = ChainClient::connect(endpoint).await?;
    client
        .ensure_burned_registration(key_uri, subnet, Duration::from_secs(20))
        .await
        .map_err(Into::into)
}

/// Starts the subnet and enables staking through the owner path.
pub async fn ensure_subtoken_enabled(
    endpoint: &str,
    key_uri: &str,
    subnet: NetUid,
) -> Result<SubtokenBootstrapReport, ChainActionError> {
    let client = ChainClient::connect(endpoint).await?;
    let report = client
        .ensure_subtoken_enabled(key_uri, subnet, Duration::from_secs(20))
        .await?;

    Ok(subtoken_bootstrap_report(report))
}

/// Sets a subnet tempo override through the local sudo path.
pub async fn ensure_subnet_tempo(
    endpoint: &str,
    key_uri: &str,
    subnet: NetUid,
    tempo: u16,
) -> Result<SubnetTempoBootstrapReport, ChainActionError> {
    let client = ChainClient::connect(endpoint).await?;
    let report = client
        .ensure_subnet_tempo_via_sudo(key_uri, subnet, tempo, Duration::from_secs(20))
        .await?;

    Ok(subnet_tempo_bootstrap_report(report))
}

/// Sets a subnet weights-set rate limit override through the local sudo path.
pub async fn ensure_weights_set_rate_limit(
    endpoint: &str,
    key_uri: &str,
    subnet: NetUid,
    weights_set_rate_limit: u64,
) -> Result<WeightsRateLimitBootstrapReport, ChainActionError> {
    let client = ChainClient::connect(endpoint).await?;
    let report = client
        .ensure_weights_set_rate_limit_via_sudo(
            key_uri,
            subnet,
            weights_set_rate_limit,
            Duration::from_secs(20),
        )
        .await?;

    Ok(weights_rate_limit_bootstrap_report(report))
}

/// Enables or disables commit-reveal through the subnet owner admin path.
pub async fn ensure_commit_reveal_enabled(
    endpoint: &str,
    key_uri: &str,
    subnet: NetUid,
    enabled: bool,
) -> Result<CommitRevealBootstrapReport, ChainActionError> {
    let client = ChainClient::connect(endpoint).await?;
    let report = client
        .ensure_commit_reveal_weights_enabled(key_uri, subnet, enabled, Duration::from_secs(20))
        .await?;

    Ok(commit_reveal_bootstrap_report(report))
}

/// Ensures the validator has enough stake to acquire validator permit and waits for it.
pub async fn ensure_validator_permit_ready(
    endpoint: &str,
    key_uri: &str,
    subnet: NetUid,
    requested_minimum_stake: u64,
) -> Result<ValidatorPermitBootstrapReport, ChainActionError> {
    let client = ChainClient::connect(endpoint).await?;
    let hotkey = ChainClient::account_id_from_uri(key_uri)?;
    let stake_report = client
        .ensure_stake_added(
            key_uri,
            subnet,
            requested_minimum_stake,
            Duration::from_secs(20),
        )
        .await?;
    let uid = client
        .wait_for_validator_permit(subnet, &hotkey, Duration::from_secs(30 * 60))
        .await?;

    Ok(validator_permit_bootstrap_report(
        hotkey.to_string(),
        subnet,
        uid,
        stake_report,
    ))
}

/// Submits one bootstrap weight vector for this validator on-chain.
pub async fn ensure_weights_set(
    endpoint: &str,
    key_uri: &str,
    subnet: NetUid,
    target_hotkey_uri: &str,
) -> Result<WeightSubmissionReport, ChainActionError> {
    let client = ChainClient::connect(endpoint).await?;
    client
        .ensure_weights_set(
            key_uri,
            subnet,
            target_hotkey_uri,
            Duration::from_secs(30 * 60),
        )
        .await
        .map_err(Into::into)
}

fn validator_permit_bootstrap_report(
    hotkey: String,
    subnet: NetUid,
    uid: u16,
    stake_report: StakeAddReport,
) -> ValidatorPermitBootstrapReport {
    ValidatorPermitBootstrapReport {
        hotkey,
        subnet,
        uid,
        requested_minimum_stake: stake_report.requested_minimum_stake,
        final_stake: stake_report.final_stake,
        added_stake: stake_report.added_stake,
        extrinsic_hash: stake_report.extrinsic_hash,
        already_staked: stake_report.already_staked,
    }
}

fn subtoken_bootstrap_report(report: SubtokenEnableReport) -> SubtokenBootstrapReport {
    SubtokenBootstrapReport {
        subnet: report.subnet,
        extrinsic_hash: report.extrinsic_hash,
        already_enabled: report.already_enabled,
    }
}

fn subnet_tempo_bootstrap_report(report: TempoSetReport) -> SubnetTempoBootstrapReport {
    SubnetTempoBootstrapReport {
        subnet: report.subnet,
        tempo: report.tempo,
        extrinsic_hash: report.extrinsic_hash,
        already_set: report.already_set,
    }
}

fn weights_rate_limit_bootstrap_report(
    report: WeightsRateLimitReport,
) -> WeightsRateLimitBootstrapReport {
    WeightsRateLimitBootstrapReport {
        subnet: report.subnet,
        weights_set_rate_limit: report.weights_set_rate_limit,
        extrinsic_hash: report.extrinsic_hash,
        already_set: report.already_set,
    }
}

fn commit_reveal_bootstrap_report(report: CommitRevealToggleReport) -> CommitRevealBootstrapReport {
    CommitRevealBootstrapReport {
        subnet: report.subnet,
        enabled: report.enabled,
        extrinsic_hash: report.extrinsic_hash,
        already_set: report.already_set,
    }
}

use myosu_chain_client::AxonServeReport;
use myosu_chain_client::ChainClient;
use myosu_chain_client::ChainClientError;
use myosu_chain_client::RegistrationReport;
use myosu_chain_client::RpcMethods;
use myosu_chain_client::SystemHealth;
use std::time::Duration;
use subtensor_runtime_common::NetUid;
use thiserror::Error;

const OPERATOR_CHAIN_ACTION_TIMEOUT: Duration = Duration::from_secs(180);

/// Startup connectivity summary for the bootstrap miner.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChainProbeReport {
    pub endpoint: String,
    pub subnet: NetUid,
    pub health: SystemHealth,
    pub rpc_methods: RpcMethods,
    pub neuron_lite_bytes: Vec<u8>,
}

/// Errors returned by the miner's initial chain probe.
#[derive(Debug, Error)]
pub enum ChainProbeError {
    /// Returned when the underlying shared chain client fails.
    #[error("failed to probe chain endpoint: {0}")]
    Client(#[from] ChainClientError),
}

/// Errors returned by the miner's on-chain bootstrap actions.
#[derive(Debug, Error)]
pub enum ChainActionError {
    /// Returned when the underlying shared chain client fails.
    #[error("failed to execute miner chain action: {0}")]
    Client(#[from] ChainClientError),
}

/// Connects to the chain and gathers the minimum startup facts the miner needs.
///
/// Args:
///     endpoint: WebSocket RPC endpoint for the node.
///     subnet: Subnet the operator intends to mine on.
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

/// Registers the miner hotkey on-chain if it is not already a subnet member.
pub async fn ensure_registered(
    endpoint: &str,
    key_uri: &str,
    subnet: NetUid,
) -> Result<RegistrationReport, ChainActionError> {
    let client = ChainClient::connect(endpoint).await?;
    client
        .ensure_burned_registration(key_uri, subnet, OPERATOR_CHAIN_ACTION_TIMEOUT)
        .await
        .map_err(Into::into)
}

/// Publishes the miner axon endpoint on-chain if it is not already current.
pub async fn ensure_serving(
    endpoint: &str,
    key_uri: &str,
    subnet: NetUid,
    port: u16,
) -> Result<AxonServeReport, ChainActionError> {
    let client = ChainClient::connect(endpoint).await?;
    client
        .ensure_axon_served(key_uri, subnet, 1, 0, port, OPERATOR_CHAIN_ACTION_TIMEOUT)
        .await
        .map_err(Into::into)
}

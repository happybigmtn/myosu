use std::borrow::Cow;
use std::collections::VecDeque;
use std::net::Ipv4Addr;
use std::net::Ipv6Addr;
use std::num::ParseIntError;
use std::time::Duration;
use std::time::Instant;

use codec::Decode;
use codec::Encode;
use frame_metadata_hash_extension::CheckMetadataHash;
use frame_system::CheckEra;
use frame_system::CheckGenesis;
use frame_system::CheckNonZeroSender;
use frame_system::CheckSpecVersion;
use frame_system::CheckTxVersion;
use frame_system::CheckWeight;
use frame_system::EventRecord;
use frame_system::Phase;
use jsonrpsee::core::client::ClientT;
use jsonrpsee::rpc_params;
use jsonrpsee::ws_client::WsClient;
use jsonrpsee::ws_client::WsClientBuilder;
use myosu_chain_runtime as runtime;
use pallet_game_solver::AxonInfo;
use pallet_game_solver::Call as GameSolverCall;
use pallet_transaction_payment::ChargeTransactionPayment;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use sp_core::H256;
use sp_core::Pair;
use sp_core::hashing::blake2_128;
use sp_core::hashing::twox_64;
use sp_core::hashing::twox_128;
use sp_core::sr25519;
use sp_runtime::AccountId32;
use sp_runtime::generic::Era;
use sp_runtime::traits::BlakeTwo256;
use sp_runtime::traits::Hash;
use subtensor_runtime_common::AlphaCurrency;
use subtensor_runtime_common::NetUid;
use subtensor_runtime_common::NetUidStorageIndex;
use subtensor_runtime_common::Signature;
use subtensor_runtime_common::TaoCurrency;
use thiserror::Error;

const DEFAULT_WS_SCHEME: &str = "ws://";
const DEFAULT_POLL_INTERVAL: Duration = Duration::from_millis(500);
const DEFAULT_NETWORK_RATE_LIMIT: u64 = runtime::SubtensorInitialNetworkRateLimit::get();
const DEFAULT_SUBNET_TEMPO: u16 = runtime::SubtensorInitialTempo::get();
const DEFAULT_WEIGHTS_SET_RATE_LIMIT: u64 = 100;

/// Errors returned by the shared chain client.
#[derive(Debug, Error)]
pub enum ChainClientError {
    /// Returned when the provided RPC endpoint is empty after trimming.
    #[error("chain RPC endpoint is empty")]
    EmptyEndpoint,

    /// Returned when the JSON-RPC transport or method call fails.
    #[error("chain RPC request failed: {0}")]
    Rpc(#[from] jsonrpsee::core::ClientError),

    /// Returned when one specific JSON-RPC method call fails.
    #[error("chain RPC `{method}` failed: {source}")]
    RpcMethod {
        method: &'static str,
        #[source]
        source: jsonrpsee::core::ClientError,
    },

    /// Returned when a secret URI cannot be parsed as an sr25519 key.
    #[error("failed to parse sr25519 secret URI `{uri}`: {detail}")]
    InvalidSecretUri { uri: String, detail: String },

    /// Returned when a chain hash response is malformed.
    #[error("failed to parse chain hash `{value}`: {source}")]
    InvalidHash {
        value: String,
        source: hex::FromHexError,
    },

    /// Returned when a chain header number is not valid hexadecimal.
    #[error("failed to parse chain header number `{value}`: {source}")]
    InvalidHeaderNumber {
        value: String,
        #[source]
        source: ParseIntError,
    },

    /// Returned when a hex-encoded RPC response cannot be decoded.
    #[error("failed to decode hex payload from `{method}`: {source}")]
    InvalidHexPayload {
        method: &'static str,
        #[source]
        source: hex::FromHexError,
    },

    /// Returned when SCALE decoding of a storage value fails.
    #[error("failed to decode SCALE value for storage key `{key}`: {detail}")]
    StorageDecode { key: String, detail: String },

    /// Returned when a storage poll did not converge before timeout.
    #[error("timed out waiting for `{operation}` after {timeout_secs}s")]
    Timeout {
        operation: String,
        timeout_secs: u64,
    },

    /// Returned when block-scoped extrinsic outcome inspection fails.
    #[error("extrinsic watch for `{operation}` failed: {detail}")]
    ExtrinsicWatchFailed { operation: String, detail: String },

    /// Returned when a required subnet member is missing from chain state.
    #[error("missing subnet member `{hotkey}` on subnet {subnet} while preparing {operation}")]
    MissingSubnetMember {
        operation: &'static str,
        hotkey: String,
        subnet: NetUid,
    },

    /// Returned when a validator lacks permission to submit non-self weights.
    #[error("validator uid {validator_uid} does not have validator permit on subnet {subnet}")]
    ValidatorPermitMissing { validator_uid: u16, subnet: NetUid },
}

/// Response from the node `system_health` RPC method.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemHealth {
    pub peers: usize,
    pub is_syncing: bool,
    pub should_have_peers: bool,
}

/// Response from the node `rpc_methods` RPC method.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct RpcMethods {
    #[serde(default)]
    pub version: Option<u32>,
    pub methods: Vec<String>,
}

/// Response from the node `chain_getHeader` RPC method.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChainHeader {
    pub parent_hash: String,
    pub number: String,
    pub state_root: String,
    pub extrinsics_root: String,
}

/// Response from the node `state_getRuntimeVersion` RPC method.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeVersion {
    pub spec_version: u32,
    pub transaction_version: u32,
}

/// Report describing a successful on-chain burned registration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegistrationReport {
    pub hotkey: AccountId32,
    pub subnet: NetUid,
    pub uid: u16,
    pub extrinsic_hash: Option<String>,
    pub already_registered: bool,
}

/// Report describing a successful on-chain axon publication.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AxonServeReport {
    pub hotkey: AccountId32,
    pub subnet: NetUid,
    pub version: u32,
    pub ip: u128,
    pub port: u16,
    pub extrinsic_hash: Option<String>,
    pub already_published: bool,
}

/// Report describing a successful subnet registration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NetworkRegistrationReport {
    pub hotkey: AccountId32,
    pub subnet: NetUid,
    pub extrinsic_hash: Option<String>,
    pub inclusion_block: H256,
    pub network_last_registered_at_inclusion: u64,
    pub network_last_registered_at_head: u64,
}

/// Report describing a successful on-chain stake top-up.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StakeAddReport {
    pub hotkey: AccountId32,
    pub subnet: NetUid,
    pub requested_minimum_stake: u64,
    pub final_stake: u64,
    pub added_stake: u64,
    pub extrinsic_hash: Option<String>,
    pub already_staked: bool,
}

/// Report describing a successful subnet start and staking enablement.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubtokenEnableReport {
    pub subnet: NetUid,
    pub extrinsic_hash: Option<String>,
    pub already_enabled: bool,
}

/// Report describing a successful on-chain weight submission.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WeightSubmissionReport {
    pub validator_hotkey: AccountId32,
    pub validator_uid: u16,
    pub target_hotkey: AccountId32,
    pub target_uid: u16,
    pub subnet: NetUid,
    pub version_key: u64,
    pub mode: &'static str,
    pub extrinsic_hash: Option<String>,
    pub already_submitted: bool,
}

/// Report describing observed post-weight epoch outputs for one subnet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EpochOutcomeReport {
    pub subnet: NetUid,
    pub miner_uid: u16,
    pub validator_uid: u16,
    pub miner_incentive: u16,
    pub validator_dividend: u16,
    pub miner_emission: u64,
}

/// Chain-visible miner metadata derived from incentive, key, and axon state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChainVisibleMiner {
    pub subnet: NetUid,
    pub uid: u16,
    pub hotkey: AccountId32,
    pub incentive: u16,
    pub axon: AxonInfo,
}

impl ChainVisibleMiner {
    /// Returns the miner's advertised axon endpoint when it can be formatted.
    pub fn endpoint_hint(&self) -> Option<String> {
        format_axon_endpoint(&self.axon)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CommitWindow {
    first_reveal_block: u64,
}

#[derive(Clone, Debug)]
struct CommitWeightsRequest<'a> {
    signer_uri: &'a str,
    netuid: NetUid,
    hotkey: &'a AccountId32,
    uids: &'a [u16],
    values: &'a [u16],
    salt: &'a [u16],
    version_key: u64,
}

#[derive(Clone, Debug)]
struct SigningContext {
    genesis_hash: H256,
    best_hash: H256,
    best_number: u64,
    nonce: u32,
    spec_version: u32,
    transaction_version: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
struct ChainBlockResponse {
    block: ChainBlockData,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ChainBlockData {
    extrinsics: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct WatchedExtrinsicOutcome {
    extrinsic_hash: String,
    inclusion_block: H256,
    extrinsic_hex: String,
}

/// Shared JSON-RPC client for the local Myosu chain node.
#[derive(Debug)]
pub struct ChainClient {
    endpoint: String,
    client: WsClient,
}

impl ChainClient {
    /// Connects to the chain node over WebSocket JSON-RPC.
    ///
    /// Args:
    ///     endpoint: Raw operator-provided RPC endpoint. If it omits a scheme,
    ///         `ws://` is assumed.
    ///
    /// Returns:
    ///     A connected `ChainClient`.
    pub async fn connect(endpoint: &str) -> Result<Self, ChainClientError> {
        let endpoint = normalize_ws_endpoint(endpoint)?;
        let client = WsClientBuilder::default().build(endpoint.as_ref()).await?;
        Ok(Self {
            endpoint: endpoint.into_owned(),
            client,
        })
    }

    /// Returns the normalized WebSocket endpoint used by this client.
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    /// Parses an sr25519 secret URI into the corresponding account ID.
    pub fn account_id_from_uri(uri: &str) -> Result<AccountId32, ChainClientError> {
        let pair = sr25519::Pair::from_string(uri, None).map_err(|source| {
            ChainClientError::InvalidSecretUri {
                uri: uri.to_string(),
                detail: source.to_string(),
            }
        })?;
        Ok(AccountId32::from(pair.public()))
    }

    /// Calls `system_health` and returns the typed node health payload.
    pub async fn system_health(&self) -> Result<SystemHealth, ChainClientError> {
        self.request("system_health", rpc_params![]).await
    }

    /// Calls `rpc_methods` and returns the advertised RPC method list.
    pub async fn rpc_methods(&self) -> Result<RpcMethods, ChainClientError> {
        self.request("rpc_methods", rpc_params![]).await
    }

    /// Calls `chain_getHeader` for the current best head.
    pub async fn chain_get_header(&self) -> Result<ChainHeader, ChainClientError> {
        self.request("chain_getHeader", rpc_params![]).await
    }

    /// Returns the current best block number.
    pub async fn best_block_number(&self) -> Result<u64, ChainClientError> {
        let header = self.chain_get_header().await?;
        parse_header_number(&header.number)
    }

    /// Returns the decoded event records for the current best block.
    pub async fn system_events(
        &self,
    ) -> Result<Vec<EventRecord<runtime::RuntimeEvent, H256>>, ChainClientError> {
        let key = storage_prefix("System", "Events");
        Ok(self
            .state_get_storage_decoded::<Vec<EventRecord<runtime::RuntimeEvent, H256>>>(
                "state_getStorage",
                &key,
            )
            .await?
            .unwrap_or_default())
    }

    async fn system_events_at(
        &self,
        block_hash: H256,
    ) -> Result<Vec<EventRecord<runtime::RuntimeEvent, H256>>, ChainClientError> {
        let key = storage_prefix("System", "Events");
        Ok(self
            .state_get_storage_decoded_at::<Vec<EventRecord<runtime::RuntimeEvent, H256>>>(
                "state_getStorage",
                &key,
                block_hash,
            )
            .await?
            .unwrap_or_default())
    }

    /// Calls `chain_getBlockHash` for the requested block number.
    pub async fn chain_get_block_hash(&self, block: u64) -> Result<H256, ChainClientError> {
        let hash: String = self
            .request("chain_getBlockHash", rpc_params![block])
            .await?;
        parse_h256(hash)
    }

    async fn chain_get_block(
        &self,
        block_hash: H256,
    ) -> Result<Option<ChainBlockResponse>, ChainClientError> {
        self.request("chain_getBlock", rpc_params![h256_hex(&block_hash)])
            .await
    }

    /// Calls `state_getRuntimeVersion`.
    pub async fn state_get_runtime_version(&self) -> Result<RuntimeVersion, ChainClientError> {
        self.request("state_getRuntimeVersion", rpc_params![]).await
    }

    /// Calls `system_accountNextIndex`.
    pub async fn system_account_next_index(
        &self,
        account: &AccountId32,
    ) -> Result<u32, ChainClientError> {
        self.request("system_accountNextIndex", rpc_params![account.to_string()])
            .await
    }

    /// Returns SCALE-encoded lite neuron info for the requested subnet.
    ///
    /// The byte payload matches the node's custom `neuronInfo_getNeuronsLite`
    /// RPC surface and is decoded by higher layers once they know the target
    /// runtime types.
    pub async fn neuron_info_get_neurons_lite(
        &self,
        netuid: NetUid,
    ) -> Result<Vec<u8>, ChainClientError> {
        self.request(
            "neuronInfo_getNeuronsLite",
            rpc_params![netuid, Option::<String>::None],
        )
        .await
    }

    /// Returns the UID for one subnet member when present.
    pub async fn get_uid_for_net_and_hotkey(
        &self,
        netuid: NetUid,
        hotkey: &AccountId32,
    ) -> Result<Option<u16>, ChainClientError> {
        let key = uid_storage_key(netuid, hotkey);
        self.state_get_storage_decoded("state_getStorage", &key)
            .await
    }

    /// Returns the hotkey for one subnet UID when present.
    pub async fn get_hotkey_for_net_and_uid(
        &self,
        netuid: NetUid,
        uid: u16,
    ) -> Result<Option<AccountId32>, ChainClientError> {
        let key = key_storage_key(netuid, uid);
        self.state_get_storage_decoded("state_getStorage", &key)
            .await
    }

    /// Returns the published axon info for one subnet member when present.
    pub async fn get_axon_for_net_and_hotkey(
        &self,
        netuid: NetUid,
        hotkey: &AccountId32,
    ) -> Result<Option<AxonInfo>, ChainClientError> {
        let key = axon_storage_key(netuid, hotkey);
        self.state_get_storage_decoded("state_getStorage", &key)
            .await
    }

    /// Returns the current subnet alpha stake for one hotkey.
    pub async fn get_hotkey_subnet_stake(
        &self,
        netuid: NetUid,
        hotkey: &AccountId32,
    ) -> Result<u64, ChainClientError> {
        let key = hotkey_alpha_storage_key(hotkey, netuid);
        Ok(self
            .state_get_storage_decoded::<AlphaCurrency>("state_getStorage", &key)
            .await?
            .map(u64::from)
            .unwrap_or_default())
    }

    /// Returns the currently allocated subnet ids from `NetworksAdded`.
    pub async fn get_existing_subnets(&self) -> Result<Vec<NetUid>, ChainClientError> {
        let prefix = storage_prefix("SubtensorModule", "NetworksAdded");
        let prefix_hex = format!("0x{}", hex::encode(prefix));
        let keys = self
            .request::<Vec<String>, _>("state_getKeys", rpc_params![prefix_hex])
            .await?;
        let mut subnets = Vec::new();
        for key in keys {
            let bytes = decode_hex_payload("state_getKeys", &key)?;
            let Some(encoded_netuid) = bytes.get(32..) else {
                continue;
            };
            let mut slice = encoded_netuid;
            let netuid =
                NetUid::decode(&mut slice).map_err(|source| ChainClientError::StorageDecode {
                    key: key.clone(),
                    detail: source.to_string(),
                })?;
            subnets.push(netuid);
        }
        subnets.sort_unstable();
        Ok(subnets)
    }

    /// Returns the active weights version key for one subnet.
    pub async fn get_weights_version_key(&self, netuid: NetUid) -> Result<u64, ChainClientError> {
        let key = map_identity_storage_key("SubtensorModule", "WeightsVersionKey", &netuid);
        Ok(self
            .state_get_storage_decoded::<u64>("state_getStorage", &key)
            .await?
            .unwrap_or_default())
    }

    /// Returns the active weights-set rate limit for one subnet.
    pub async fn get_weights_set_rate_limit(
        &self,
        netuid: NetUid,
    ) -> Result<u64, ChainClientError> {
        let key = map_identity_storage_key("SubtensorModule", "WeightsSetRateLimit", &netuid);
        Ok(self
            .state_get_storage_decoded::<u64>("state_getStorage", &key)
            .await?
            .unwrap_or(DEFAULT_WEIGHTS_SET_RATE_LIMIT))
    }

    /// Returns the current subnet tempo.
    pub async fn get_subnet_tempo(&self, netuid: NetUid) -> Result<u16, ChainClientError> {
        let key = map_identity_storage_key("SubtensorModule", "Tempo", &netuid);
        Ok(self
            .state_get_storage_decoded::<u16>("state_getStorage", &key)
            .await?
            .unwrap_or(DEFAULT_SUBNET_TEMPO))
    }

    /// Returns the global admin freeze window length.
    pub async fn get_admin_freeze_window(&self) -> Result<u16, ChainClientError> {
        let key = storage_prefix("SubtensorModule", "AdminFreezeWindow");
        Ok(self
            .state_get_storage_decoded::<u16>("state_getStorage", &key)
            .await?
            .unwrap_or_default())
    }

    /// Returns the global network registration rate limit.
    pub async fn get_network_rate_limit(&self) -> Result<u64, ChainClientError> {
        let key = storage_prefix("SubtensorModule", "NetworkRateLimit");
        Ok(self
            .state_get_storage_decoded::<u64>("state_getStorage", &key)
            .await?
            .unwrap_or(DEFAULT_NETWORK_RATE_LIMIT))
    }

    /// Returns the global network registration rate limit at one block.
    pub async fn get_network_rate_limit_at(
        &self,
        block_hash: H256,
    ) -> Result<u64, ChainClientError> {
        let key = storage_prefix("SubtensorModule", "NetworkRateLimit");
        Ok(self
            .state_get_storage_decoded_at::<u64>("state_getStorage", &key, block_hash)
            .await?
            .unwrap_or(DEFAULT_NETWORK_RATE_LIMIT))
    }

    /// Returns the block number recorded for the most recent subnet registration.
    pub async fn get_network_last_registered_block(&self) -> Result<u64, ChainClientError> {
        let key = network_last_registered_storage_key();
        Ok(self
            .state_get_storage_decoded::<u64>("state_getStorage", &key)
            .await?
            .unwrap_or_default())
    }

    /// Returns the block number recorded for the most recent subnet registration at one block.
    pub async fn get_network_last_registered_block_at(
        &self,
        block_hash: H256,
    ) -> Result<u64, ChainClientError> {
        let key = network_last_registered_storage_key();
        Ok(self
            .state_get_storage_decoded_at::<u64>("state_getStorage", &key, block_hash)
            .await?
            .unwrap_or_default())
    }

    /// Returns the global subnet limit.
    pub async fn get_subnet_limit(&self) -> Result<u16, ChainClientError> {
        let key = storage_prefix("SubtensorModule", "SubnetLimit");
        Ok(self
            .state_get_storage_decoded::<u16>("state_getStorage", &key)
            .await?
            .unwrap_or_default())
    }

    /// Returns whether one subnet uid currently has validator permit.
    pub async fn has_validator_permit(
        &self,
        netuid: NetUid,
        uid: u16,
    ) -> Result<bool, ChainClientError> {
        let key = map_identity_storage_key("SubtensorModule", "ValidatorPermit", &netuid);
        let permits = self
            .state_get_storage_decoded::<Vec<bool>>("state_getStorage", &key)
            .await?
            .unwrap_or_default();
        Ok(permits.get(usize::from(uid)).copied().unwrap_or(false))
    }

    /// Returns one validator's current on-chain weights for the subnet.
    pub async fn get_weights_for_uid(
        &self,
        netuid: NetUid,
        uid: u16,
    ) -> Result<Vec<(u16, u16)>, ChainClientError> {
        let key = weights_storage_key(netuid, uid);
        Ok(self
            .state_get_storage_decoded::<Vec<(u16, u16)>>("state_getStorage", &key)
            .await?
            .unwrap_or_default())
    }

    /// Returns the current subnet incentive vector.
    pub async fn get_incentives(&self, netuid: NetUid) -> Result<Vec<u16>, ChainClientError> {
        let key = map_identity_storage_key(
            "SubtensorModule",
            "Incentive",
            &NetUidStorageIndex::from(netuid),
        );
        Ok(self
            .state_get_storage_decoded::<Vec<u16>>("state_getStorage", &key)
            .await?
            .unwrap_or_default())
    }

    /// Returns chain-visible miners ranked by incentive descending then UID ascending.
    pub async fn get_chain_visible_miners(
        &self,
        netuid: NetUid,
    ) -> Result<Vec<ChainVisibleMiner>, ChainClientError> {
        let incentives = self.get_incentives(netuid).await?;
        let mut miners = Vec::new();
        for (uid_index, incentive) in incentives.into_iter().enumerate() {
            let Ok(uid) = u16::try_from(uid_index) else {
                continue;
            };
            if incentive == 0 {
                continue;
            }
            let Some(hotkey) = self.get_hotkey_for_net_and_uid(netuid, uid).await? else {
                continue;
            };
            let Some(axon) = self.get_axon_for_net_and_hotkey(netuid, &hotkey).await? else {
                continue;
            };
            if format_axon_endpoint(&axon).is_none() {
                continue;
            }
            miners.push(ChainVisibleMiner {
                subnet: netuid,
                uid,
                hotkey,
                incentive,
                axon,
            });
        }
        miners.sort_by(|left, right| {
            right
                .incentive
                .cmp(&left.incentive)
                .then_with(|| left.uid.cmp(&right.uid))
        });
        Ok(miners)
    }

    /// Returns the current subnet dividend vector.
    pub async fn get_dividends(&self, netuid: NetUid) -> Result<Vec<u16>, ChainClientError> {
        let key = map_identity_storage_key("SubtensorModule", "Dividends", &netuid);
        Ok(self
            .state_get_storage_decoded::<Vec<u16>>("state_getStorage", &key)
            .await?
            .unwrap_or_default())
    }

    /// Returns the current subnet emission vector.
    pub async fn get_emissions(
        &self,
        netuid: NetUid,
    ) -> Result<Vec<AlphaCurrency>, ChainClientError> {
        let key = map_identity_storage_key("SubtensorModule", "Emission", &netuid);
        Ok(self
            .state_get_storage_decoded::<Vec<AlphaCurrency>>("state_getStorage", &key)
            .await?
            .unwrap_or_default())
    }

    /// Returns whether commit-reveal is enabled for one subnet.
    pub async fn get_commit_reveal_weights_enabled(
        &self,
        netuid: NetUid,
    ) -> Result<bool, ChainClientError> {
        let key =
            map_identity_storage_key("SubtensorModule", "CommitRevealWeightsEnabled", &netuid);
        Ok(self
            .state_get_storage_decoded::<bool>("state_getStorage", &key)
            .await?
            .unwrap_or(true))
    }

    /// Returns whether subnet staking is enabled for one subnet.
    pub async fn get_subtoken_enabled(&self, netuid: NetUid) -> Result<bool, ChainClientError> {
        let key = map_identity_storage_key("SubtensorModule", "SubtokenEnabled", &netuid);
        Ok(self
            .state_get_storage_decoded::<bool>("state_getStorage", &key)
            .await?
            .unwrap_or_default())
    }

    /// Returns the queued weight commits for one hotkey on the subnet.
    pub async fn get_weight_commits(
        &self,
        netuid: NetUid,
        hotkey: &AccountId32,
    ) -> Result<VecDeque<(H256, u64, u64, u64)>, ChainClientError> {
        let key = weight_commits_storage_key(netuid, hotkey);
        Ok(self
            .state_get_storage_decoded::<VecDeque<(H256, u64, u64, u64)>>("state_getStorage", &key)
            .await?
            .unwrap_or_default())
    }

    /// Ensures a hotkey is registered on-chain through `burned_register`.
    pub async fn ensure_burned_registration(
        &self,
        signer_uri: &str,
        netuid: NetUid,
        timeout: Duration,
    ) -> Result<RegistrationReport, ChainClientError> {
        let hotkey = Self::account_id_from_uri(signer_uri)?;
        if let Some(uid) = self.get_uid_for_net_and_hotkey(netuid, &hotkey).await? {
            return Ok(RegistrationReport {
                hotkey,
                subnet: netuid,
                uid,
                extrinsic_hash: None,
                already_registered: true,
            });
        }

        let call = runtime::RuntimeCall::SubtensorModule(
            GameSolverCall::<runtime::Runtime>::burned_register {
                netuid,
                hotkey: hotkey.clone(),
            },
        );
        let extrinsic_hash = self.submit_signed_call(signer_uri, call).await?;
        let uid = self
            .poll_for_uid(netuid, &hotkey, timeout)
            .await?
            .ok_or_else(|| ChainClientError::Timeout {
                operation: format!("burned_register for subnet {netuid}"),
                timeout_secs: timeout.as_secs(),
            })?;

        Ok(RegistrationReport {
            hotkey,
            subnet: netuid,
            uid,
            extrinsic_hash: Some(extrinsic_hash),
            already_registered: false,
        })
    }

    /// Ensures a hotkey has a published axon for the requested endpoint.
    pub async fn ensure_axon_served(
        &self,
        signer_uri: &str,
        netuid: NetUid,
        version: u32,
        ip: u128,
        port: u16,
        timeout: Duration,
    ) -> Result<AxonServeReport, ChainClientError> {
        let hotkey = Self::account_id_from_uri(signer_uri)?;
        if let Some(existing) = self.get_axon_for_net_and_hotkey(netuid, &hotkey).await?
            && existing.version == version
            && existing.ip == ip
            && existing.port == port
        {
            return Ok(AxonServeReport {
                hotkey,
                subnet: netuid,
                version,
                ip,
                port,
                extrinsic_hash: None,
                already_published: true,
            });
        }

        let call =
            runtime::RuntimeCall::SubtensorModule(GameSolverCall::<runtime::Runtime>::serve_axon {
                netuid,
                version,
                ip,
                port,
                ip_type: 4,
                protocol: 0,
                placeholder1: 0,
                placeholder2: 0,
            });
        let extrinsic_hash = self.submit_signed_call(signer_uri, call).await?;
        self.poll_for_axon(netuid, &hotkey, version, ip, port, timeout)
            .await?;

        Ok(AxonServeReport {
            hotkey,
            subnet: netuid,
            version,
            ip,
            port,
            extrinsic_hash: Some(extrinsic_hash),
            already_published: false,
        })
    }

    /// Registers a fresh dynamic subnet and returns the new subnet id.
    pub async fn register_network(
        &self,
        signer_uri: &str,
        timeout: Duration,
    ) -> Result<NetworkRegistrationReport, ChainClientError> {
        let hotkey = Self::account_id_from_uri(signer_uri)?;
        let before = self.get_existing_subnets().await?;
        let call = runtime::RuntimeCall::SubtensorModule(
            GameSolverCall::<runtime::Runtime>::register_network {
                hotkey: hotkey.clone(),
            },
        );
        let watched = match self
            .submit_signed_call_and_watch_inclusion(signer_uri, call, timeout)
            .await
        {
            Ok(watched) => watched,
            Err(ChainClientError::Timeout { timeout_secs, .. }) => {
                return Err(self.register_network_timeout(timeout_secs).await);
            }
            Err(error) => return Err(error),
        };
        let subnet = match self
            .extract_registered_subnet_from_block(
                &before,
                &watched.extrinsic_hash,
                &watched.extrinsic_hex,
                watched.inclusion_block,
            )
            .await
        {
            Ok(subnet) => subnet,
            Err(ChainClientError::Timeout { timeout_secs, .. }) => {
                return Err(self.register_network_timeout(timeout_secs).await);
            }
            Err(error) => return Err(error),
        };
        let network_last_registered_at_inclusion = self
            .get_network_last_registered_block_at(watched.inclusion_block)
            .await?;
        let network_last_registered_at_head = self.get_network_last_registered_block().await?;

        Ok(NetworkRegistrationReport {
            hotkey,
            subnet,
            extrinsic_hash: Some(watched.extrinsic_hash),
            inclusion_block: watched.inclusion_block,
            network_last_registered_at_inclusion,
            network_last_registered_at_head,
        })
    }

    /// Ensures a subnet member has at least the requested subnet stake.
    pub async fn ensure_stake_added(
        &self,
        signer_uri: &str,
        netuid: NetUid,
        requested_minimum_stake: u64,
        timeout: Duration,
    ) -> Result<StakeAddReport, ChainClientError> {
        let hotkey = Self::account_id_from_uri(signer_uri)?;
        self.get_uid_for_net_and_hotkey(netuid, &hotkey)
            .await?
            .ok_or_else(|| ChainClientError::MissingSubnetMember {
                operation: "add_stake hotkey lookup",
                hotkey: hotkey.to_string(),
                subnet: netuid,
            })?;
        let current_stake = self.get_hotkey_subnet_stake(netuid, &hotkey).await?;
        if current_stake >= requested_minimum_stake {
            return Ok(StakeAddReport {
                hotkey,
                subnet: netuid,
                requested_minimum_stake,
                final_stake: current_stake,
                added_stake: 0,
                extrinsic_hash: None,
                already_staked: true,
            });
        }

        let added_stake = requested_minimum_stake.saturating_sub(current_stake);
        let call =
            runtime::RuntimeCall::SubtensorModule(GameSolverCall::<runtime::Runtime>::add_stake {
                hotkey: hotkey.clone(),
                netuid,
                amount_staked: TaoCurrency::from(added_stake),
            });
        let extrinsic_hash = self.submit_signed_call(signer_uri, call).await?;
        let final_stake = self
            .poll_for_hotkey_subnet_stake(netuid, &hotkey, requested_minimum_stake, timeout)
            .await?;

        Ok(StakeAddReport {
            hotkey,
            subnet: netuid,
            requested_minimum_stake,
            final_stake,
            added_stake,
            extrinsic_hash: Some(extrinsic_hash),
            already_staked: false,
        })
    }

    /// Waits until a subnet member has validator permit on-chain.
    pub async fn wait_for_validator_permit(
        &self,
        netuid: NetUid,
        hotkey: &AccountId32,
        timeout: Duration,
    ) -> Result<u16, ChainClientError> {
        let validator_uid = self
            .get_uid_for_net_and_hotkey(netuid, hotkey)
            .await?
            .ok_or_else(|| ChainClientError::MissingSubnetMember {
                operation: "validator permit lookup",
                hotkey: hotkey.to_string(),
                subnet: netuid,
            })?;
        let deadline = Instant::now()
            .checked_add(timeout)
            .unwrap_or_else(Instant::now);
        while Instant::now() < deadline {
            if self.has_validator_permit(netuid, validator_uid).await? {
                return Ok(validator_uid);
            }
            tokio::time::sleep(DEFAULT_POLL_INTERVAL).await;
        }
        Err(ChainClientError::Timeout {
            operation: format!("validator permit for subnet {netuid}"),
            timeout_secs: timeout.as_secs(),
        })
    }

    /// Ensures subnet staking is enabled through the owner `start_call` path.
    pub async fn ensure_subtoken_enabled(
        &self,
        signer_uri: &str,
        netuid: NetUid,
        timeout: Duration,
    ) -> Result<SubtokenEnableReport, ChainClientError> {
        if self.get_subtoken_enabled(netuid).await? {
            return Ok(SubtokenEnableReport {
                subnet: netuid,
                extrinsic_hash: None,
                already_enabled: true,
            });
        }

        let call =
            runtime::RuntimeCall::SubtensorModule(GameSolverCall::<runtime::Runtime>::start_call {
                netuid,
            });
        let extrinsic_hash = self.submit_signed_call(signer_uri, call).await?;
        self.poll_for_subtoken_enabled(netuid, timeout).await?;

        Ok(SubtokenEnableReport {
            subnet: netuid,
            extrinsic_hash: Some(extrinsic_hash),
            already_enabled: false,
        })
    }

    /// Ensures a validator has one live `set_weights` entry on-chain.
    pub async fn ensure_weights_set(
        &self,
        signer_uri: &str,
        netuid: NetUid,
        target_hotkey_uri: &str,
        timeout: Duration,
    ) -> Result<WeightSubmissionReport, ChainClientError> {
        let validator_hotkey = Self::account_id_from_uri(signer_uri)?;
        let target_hotkey = Self::account_id_from_uri(target_hotkey_uri)?;
        let validator_uid = self
            .get_uid_for_net_and_hotkey(netuid, &validator_hotkey)
            .await?
            .ok_or_else(|| ChainClientError::MissingSubnetMember {
                operation: "set_weights validator lookup",
                hotkey: validator_hotkey.to_string(),
                subnet: netuid,
            })?;
        let target_uid = self
            .get_uid_for_net_and_hotkey(netuid, &target_hotkey)
            .await?
            .ok_or_else(|| ChainClientError::MissingSubnetMember {
                operation: "set_weights target lookup",
                hotkey: target_hotkey.to_string(),
                subnet: netuid,
            })?;
        let desired_weights = vec![(target_uid, u16::MAX)];
        let existing = self.get_weights_for_uid(netuid, validator_uid).await?;
        if existing == desired_weights {
            return Ok(WeightSubmissionReport {
                validator_hotkey,
                validator_uid,
                target_hotkey,
                target_uid,
                subnet: netuid,
                version_key: self.get_weights_version_key(netuid).await?,
                mode: "set_weights",
                extrinsic_hash: None,
                already_submitted: true,
            });
        }
        if validator_uid != target_uid && !self.has_validator_permit(netuid, validator_uid).await? {
            return Err(ChainClientError::ValidatorPermitMissing {
                validator_uid,
                subnet: netuid,
            });
        }

        let version_key = self.get_weights_version_key(netuid).await?;
        if self.get_commit_reveal_weights_enabled(netuid).await? {
            let salt = commit_reveal_salt(validator_uid, target_uid);
            let commit_window = self
                .commit_weights(
                    CommitWeightsRequest {
                        signer_uri,
                        netuid,
                        hotkey: &validator_hotkey,
                        uids: &[target_uid],
                        values: &[u16::MAX],
                        salt: &salt,
                        version_key,
                    },
                    timeout,
                )
                .await?;
            self.wait_for_reveal_block(commit_window.first_reveal_block, timeout)
                .await?;
            let call = runtime::RuntimeCall::SubtensorModule(
                GameSolverCall::<runtime::Runtime>::reveal_weights {
                    netuid,
                    uids: vec![target_uid],
                    values: vec![u16::MAX],
                    salt,
                    version_key,
                },
            );
            let extrinsic_hash = self.submit_signed_call(signer_uri, call).await?;
            self.poll_for_weights(netuid, validator_uid, &desired_weights, timeout)
                .await?;

            return Ok(WeightSubmissionReport {
                validator_hotkey,
                validator_uid,
                target_hotkey,
                target_uid,
                subnet: netuid,
                version_key,
                mode: "commit_reveal",
                extrinsic_hash: Some(extrinsic_hash),
                already_submitted: false,
            });
        }

        let call = runtime::RuntimeCall::SubtensorModule(
            GameSolverCall::<runtime::Runtime>::set_weights {
                netuid,
                dests: vec![target_uid],
                weights: vec![u16::MAX],
                version_key,
            },
        );
        let extrinsic_hash = self.submit_signed_call(signer_uri, call).await?;
        self.poll_for_weights(netuid, validator_uid, &desired_weights, timeout)
            .await?;

        Ok(WeightSubmissionReport {
            validator_hotkey,
            validator_uid,
            target_hotkey,
            target_uid,
            subnet: netuid,
            version_key,
            mode: "set_weights",
            extrinsic_hash: Some(extrinsic_hash),
            already_submitted: false,
        })
    }

    /// Waits until post-weight epoch outputs are visible for one miner/validator pair.
    pub async fn wait_for_epoch_outcome(
        &self,
        netuid: NetUid,
        miner_uid: u16,
        validator_uid: u16,
        timeout: Duration,
    ) -> Result<EpochOutcomeReport, ChainClientError> {
        let deadline = Instant::now()
            .checked_add(timeout)
            .unwrap_or_else(Instant::now);
        while Instant::now() < deadline {
            let incentives = self.get_incentives(netuid).await?;
            let dividends = self.get_dividends(netuid).await?;
            let emissions = self.get_emissions(netuid).await?;
            let miner_incentive = incentives
                .get(usize::from(miner_uid))
                .copied()
                .unwrap_or_default();
            let validator_dividend = dividends
                .get(usize::from(validator_uid))
                .copied()
                .unwrap_or_default();
            let miner_emission = emissions
                .get(usize::from(miner_uid))
                .copied()
                .map(u64::from)
                .unwrap_or_default();
            if miner_incentive > 0 && validator_dividend > 0 && miner_emission > 0 {
                return Ok(EpochOutcomeReport {
                    subnet: netuid,
                    miner_uid,
                    validator_uid,
                    miner_incentive,
                    validator_dividend,
                    miner_emission,
                });
            }
            tokio::time::sleep(DEFAULT_POLL_INTERVAL).await;
        }
        Err(ChainClientError::Timeout {
            operation: format!("post-weight epoch outcome for subnet {netuid}"),
            timeout_secs: timeout.as_secs(),
        })
    }

    /// Issues a typed JSON-RPC request against the connected node.
    ///
    /// Args:
    ///     method: JSON-RPC method name.
    ///     params: Prebuilt `rpc_params![]` payload for the target method.
    ///
    /// Returns:
    ///     The deserialized response payload.
    pub async fn request<T, Params>(
        &self,
        method: &'static str,
        params: Params,
    ) -> Result<T, ChainClientError>
    where
        T: DeserializeOwned + Send,
        Params: jsonrpsee::core::traits::ToRpcParams + Send,
    {
        self.client
            .request(method, params)
            .await
            .map_err(|source| ChainClientError::RpcMethod { method, source })
    }

    async fn state_get_storage_decoded<T: Decode>(
        &self,
        method: &'static str,
        key: &[u8],
    ) -> Result<Option<T>, ChainClientError> {
        let key_hex = format!("0x{}", hex::encode(key));
        let value: Option<String> = self.request(method, rpc_params![key_hex]).await?;
        let Some(value) = value else {
            return Ok(None);
        };
        let bytes = decode_hex_payload(method, &value)?;
        let mut slice = bytes.as_slice();
        T::decode(&mut slice)
            .map(Some)
            .map_err(|source| ChainClientError::StorageDecode {
                key: format!("0x{}", hex::encode(key)),
                detail: source.to_string(),
            })
    }

    async fn state_get_storage_decoded_at<T: Decode>(
        &self,
        method: &'static str,
        key: &[u8],
        block_hash: H256,
    ) -> Result<Option<T>, ChainClientError> {
        let key_hex = format!("0x{}", hex::encode(key));
        let value: Option<String> = self
            .request(method, rpc_params![key_hex, h256_hex(&block_hash)])
            .await?;
        let Some(value) = value else {
            return Ok(None);
        };
        let bytes = decode_hex_payload(method, &value)?;
        let mut slice = bytes.as_slice();
        T::decode(&mut slice)
            .map(Some)
            .map_err(|source| ChainClientError::StorageDecode {
                key: format!("0x{}", hex::encode(key)),
                detail: source.to_string(),
            })
    }

    async fn submit_signed_call(
        &self,
        signer_uri: &str,
        call: runtime::RuntimeCall,
    ) -> Result<String, ChainClientError> {
        let pair = sr25519::Pair::from_string(signer_uri, None).map_err(|source| {
            ChainClientError::InvalidSecretUri {
                uri: signer_uri.to_string(),
                detail: source.to_string(),
            }
        })?;
        let signer = AccountId32::from(pair.public());
        let signing_context = self.signing_context(&signer).await?;
        let extrinsic_hex = signed_extrinsic_hex(pair, &signing_context, call)?;
        let extrinsic_hash: String = self
            .request("author_submitExtrinsic", rpc_params![extrinsic_hex])
            .await?;
        self.poll_for_account_nonce(
            &signer,
            signing_context.nonce.saturating_add(1),
            Duration::from_secs(20),
        )
        .await?;
        Ok(extrinsic_hash)
    }

    async fn submit_signed_call_and_watch_inclusion(
        &self,
        signer_uri: &str,
        call: runtime::RuntimeCall,
        timeout: Duration,
    ) -> Result<WatchedExtrinsicOutcome, ChainClientError> {
        let pair = sr25519::Pair::from_string(signer_uri, None).map_err(|source| {
            ChainClientError::InvalidSecretUri {
                uri: signer_uri.to_string(),
                detail: source.to_string(),
            }
        })?;
        let signer = AccountId32::from(pair.public());
        let signing_context = self.signing_context(&signer).await?;
        let extrinsic_hex = signed_extrinsic_hex(pair, &signing_context, call)?;
        let extrinsic_hash: String = self
            .request("author_submitExtrinsic", rpc_params![extrinsic_hex.clone()])
            .await?;
        let inclusion_block = self
            .poll_for_extrinsic_inclusion_block(
                &extrinsic_hash,
                &extrinsic_hex,
                signing_context.best_number.saturating_add(1),
                timeout,
            )
            .await?;

        Ok(WatchedExtrinsicOutcome {
            extrinsic_hash,
            inclusion_block,
            extrinsic_hex,
        })
    }

    async fn signing_context(
        &self,
        signer: &AccountId32,
    ) -> Result<SigningContext, ChainClientError> {
        let genesis_hash = self.chain_get_block_hash(0).await?;
        let header = self.chain_get_header().await?;
        let best_hash_string: String = self.request("chain_getBlockHash", rpc_params![]).await?;
        let best_hash = parse_h256(best_hash_string)?;
        let runtime_version = self.state_get_runtime_version().await?;
        let nonce = self.system_account_next_index(signer).await?;

        Ok(SigningContext {
            genesis_hash,
            best_hash,
            best_number: parse_header_number(&header.number)?,
            nonce,
            spec_version: runtime_version.spec_version,
            transaction_version: runtime_version.transaction_version,
        })
    }

    async fn poll_for_uid(
        &self,
        netuid: NetUid,
        hotkey: &AccountId32,
        timeout: Duration,
    ) -> Result<Option<u16>, ChainClientError> {
        let deadline = Instant::now()
            .checked_add(timeout)
            .unwrap_or_else(Instant::now);
        while Instant::now() < deadline {
            if let Some(uid) = self.get_uid_for_net_and_hotkey(netuid, hotkey).await? {
                return Ok(Some(uid));
            }
            tokio::time::sleep(DEFAULT_POLL_INTERVAL).await;
        }
        Ok(None)
    }

    async fn poll_for_extrinsic_inclusion_block(
        &self,
        extrinsic_hash: &str,
        extrinsic_hex: &str,
        first_block: u64,
        timeout: Duration,
    ) -> Result<H256, ChainClientError> {
        let deadline = Instant::now()
            .checked_add(timeout)
            .unwrap_or_else(Instant::now);
        let mut next_block = first_block;
        while Instant::now() < deadline {
            let header = self.chain_get_header().await?;
            let best_number = parse_header_number(&header.number)?;
            while next_block <= best_number {
                let block_hash = self.chain_get_block_hash(next_block).await?;
                let Some(block) = self.chain_get_block(block_hash).await? else {
                    next_block = next_block.saturating_add(1);
                    continue;
                };
                if locate_block_extrinsic_index(
                    &block.block.extrinsics,
                    extrinsic_hex,
                    extrinsic_hash,
                )?
                .is_some()
                {
                    return Ok(block_hash);
                }
                next_block = next_block.saturating_add(1);
            }
            tokio::time::sleep(DEFAULT_POLL_INTERVAL).await;
        }
        Err(ChainClientError::Timeout {
            operation: "submit_signed_call_and_watch_inclusion".to_string(),
            timeout_secs: timeout.as_secs(),
        })
    }

    async fn poll_for_account_nonce(
        &self,
        account: &AccountId32,
        minimum_nonce: u32,
        timeout: Duration,
    ) -> Result<(), ChainClientError> {
        let deadline = Instant::now()
            .checked_add(timeout)
            .unwrap_or_else(Instant::now);
        while Instant::now() < deadline {
            if self.system_account_next_index(account).await? >= minimum_nonce {
                return Ok(());
            }
            tokio::time::sleep(DEFAULT_POLL_INTERVAL).await;
        }
        Err(ChainClientError::Timeout {
            operation: format!("extrinsic inclusion for account {account}"),
            timeout_secs: timeout.as_secs(),
        })
    }

    async fn poll_for_axon(
        &self,
        netuid: NetUid,
        hotkey: &AccountId32,
        version: u32,
        ip: u128,
        port: u16,
        timeout: Duration,
    ) -> Result<(), ChainClientError> {
        let deadline = Instant::now()
            .checked_add(timeout)
            .unwrap_or_else(Instant::now);
        while Instant::now() < deadline {
            if let Some(axon) = self.get_axon_for_net_and_hotkey(netuid, hotkey).await?
                && axon.version == version
                && axon.ip == ip
                && axon.port == port
            {
                return Ok(());
            }
            tokio::time::sleep(DEFAULT_POLL_INTERVAL).await;
        }
        Err(ChainClientError::Timeout {
            operation: format!("serve_axon for subnet {netuid}"),
            timeout_secs: timeout.as_secs(),
        })
    }

    async fn poll_for_weights(
        &self,
        netuid: NetUid,
        validator_uid: u16,
        expected: &[(u16, u16)],
        timeout: Duration,
    ) -> Result<(), ChainClientError> {
        let deadline = Instant::now()
            .checked_add(timeout)
            .unwrap_or_else(Instant::now);
        while Instant::now() < deadline {
            let weights = self.get_weights_for_uid(netuid, validator_uid).await?;
            if weights == expected {
                return Ok(());
            }
            tokio::time::sleep(DEFAULT_POLL_INTERVAL).await;
        }
        Err(ChainClientError::Timeout {
            operation: format!("set_weights for subnet {netuid}"),
            timeout_secs: timeout.as_secs(),
        })
    }

    async fn poll_for_hotkey_subnet_stake(
        &self,
        netuid: NetUid,
        hotkey: &AccountId32,
        minimum_stake: u64,
        timeout: Duration,
    ) -> Result<u64, ChainClientError> {
        let deadline = Instant::now()
            .checked_add(timeout)
            .unwrap_or_else(Instant::now);
        while Instant::now() < deadline {
            let current_stake = self.get_hotkey_subnet_stake(netuid, hotkey).await?;
            if current_stake >= minimum_stake {
                return Ok(current_stake);
            }
            tokio::time::sleep(DEFAULT_POLL_INTERVAL).await;
        }
        Err(ChainClientError::Timeout {
            operation: format!("add_stake for subnet {netuid}"),
            timeout_secs: timeout.as_secs(),
        })
    }

    async fn poll_for_subtoken_enabled(
        &self,
        netuid: NetUid,
        timeout: Duration,
    ) -> Result<(), ChainClientError> {
        let deadline = Instant::now()
            .checked_add(timeout)
            .unwrap_or_else(Instant::now);
        while Instant::now() < deadline {
            if self.get_subtoken_enabled(netuid).await? {
                return Ok(());
            }
            tokio::time::sleep(DEFAULT_POLL_INTERVAL).await;
        }
        Err(ChainClientError::Timeout {
            operation: format!("start_call for subnet {netuid}"),
            timeout_secs: timeout.as_secs(),
        })
    }

    async fn latest_system_extrinsic_failure(&self) -> Result<Option<String>, ChainClientError> {
        let events = self.system_events().await?;
        Ok(events
            .into_iter()
            .rev()
            .find_map(|record| match record.event {
                runtime::RuntimeEvent::System(frame_system::Event::ExtrinsicFailed {
                    dispatch_error,
                    ..
                }) => Some(format!("{dispatch_error:?}")),
                _ => None,
            }))
    }

    async fn extract_registered_subnet_from_block(
        &self,
        before: &[NetUid],
        extrinsic_hash: &str,
        extrinsic_hex: &str,
        block_hash: H256,
    ) -> Result<NetUid, ChainClientError> {
        let Some(block) = self.chain_get_block(block_hash).await? else {
            return Err(ChainClientError::ExtrinsicWatchFailed {
                operation: "register_network".to_string(),
                detail: format!("missing inclusion block {block_hash}"),
            });
        };
        let Some(extrinsic_index) =
            locate_block_extrinsic_index(&block.block.extrinsics, extrinsic_hex, extrinsic_hash)?
        else {
            return Err(ChainClientError::ExtrinsicWatchFailed {
                operation: "register_network".to_string(),
                detail: format!(
                    "included at block {block_hash} but could not locate submitted extrinsic"
                ),
            });
        };
        let events = self.system_events_at(block_hash).await?;
        let mut dispatch_error = None;
        for record in events {
            let Phase::ApplyExtrinsic(index) = record.phase else {
                continue;
            };
            if index != extrinsic_index {
                continue;
            }
            match record.event {
                runtime::RuntimeEvent::SubtensorModule(
                    pallet_game_solver::Event::NetworkAdded(netuid, _),
                ) => {
                    return Ok(netuid);
                }
                runtime::RuntimeEvent::System(frame_system::Event::ExtrinsicFailed {
                    dispatch_error: error,
                    ..
                }) => {
                    dispatch_error = Some(format!("{error:?}"));
                }
                _ => {}
            }
        }
        if let Some(dispatch_error) = dispatch_error {
            let mut detail = format!("dispatch_error={dispatch_error} at block {block_hash}");
            match self.get_network_rate_limit_at(block_hash).await {
                Ok(network_rate_limit) => {
                    detail.push_str(&format!(
                        " network_rate_limit_at_block={network_rate_limit}"
                    ));
                }
                Err(error) => {
                    detail.push_str(&format!(" network_rate_limit_at_block_error={error}"));
                }
            }
            match self.get_network_last_registered_block_at(block_hash).await {
                Ok(network_last_registered) => {
                    detail.push_str(&format!(
                        " network_last_registered_at_block={network_last_registered}"
                    ));
                }
                Err(error) => {
                    detail.push_str(&format!(" network_last_registered_at_block_error={error}"));
                }
            }
            return Err(ChainClientError::ExtrinsicWatchFailed {
                operation: "register_network".to_string(),
                detail,
            });
        }
        let after = self.get_existing_subnets().await?;
        if let Some(netuid) = after.into_iter().find(|netuid| !before.contains(netuid)) {
            return Ok(netuid);
        }
        Err(ChainClientError::ExtrinsicWatchFailed {
            operation: "register_network".to_string(),
            detail: format!(
                "block {block_hash} extrinsic {extrinsic_index} emitted no NetworkAdded event"
            ),
        })
    }

    async fn register_network_timeout(&self, timeout_secs: u64) -> ChainClientError {
        let latest_failure = self.latest_system_extrinsic_failure().await.ok().flatten();
        let network_rate_limit = self.get_network_rate_limit().await.ok();
        let subnet_limit = self.get_subnet_limit().await.ok();
        let subnets = self.get_existing_subnets().await.ok();
        let mut detail = Vec::new();
        if let Some(network_rate_limit) = network_rate_limit {
            detail.push(format!("network_rate_limit={network_rate_limit}"));
        }
        if let Some(subnet_limit) = subnet_limit {
            detail.push(format!("subnet_limit={subnet_limit}"));
        }
        if let Some(subnets) = subnets {
            detail.push(format!("existing_subnets={subnets:?}"));
        }
        if let Some(latest_failure) = latest_failure {
            detail.push(format!("latest_failure={latest_failure}"));
        }
        let suffix = if detail.is_empty() {
            String::new()
        } else {
            format!(" ({})", detail.join(", "))
        };
        ChainClientError::Timeout {
            operation: format!("register_network{suffix}"),
            timeout_secs,
        }
    }

    async fn commit_weights(
        &self,
        request: CommitWeightsRequest<'_>,
        timeout: Duration,
    ) -> Result<CommitWindow, ChainClientError> {
        let commit_hash = weight_commit_hash(
            request.hotkey,
            request.netuid,
            request.uids,
            request.values,
            request.salt,
            request.version_key,
        );
        let call = runtime::RuntimeCall::SubtensorModule(
            GameSolverCall::<runtime::Runtime>::commit_weights {
                netuid: request.netuid,
                commit_hash,
            },
        );
        let _ = self.submit_signed_call(request.signer_uri, call).await?;
        self.poll_for_commit_window(request.netuid, request.hotkey, commit_hash, timeout)
            .await
    }

    async fn poll_for_commit_window(
        &self,
        netuid: NetUid,
        hotkey: &AccountId32,
        expected_hash: H256,
        timeout: Duration,
    ) -> Result<CommitWindow, ChainClientError> {
        let deadline = Instant::now()
            .checked_add(timeout)
            .unwrap_or_else(Instant::now);
        while Instant::now() < deadline {
            let commits = self.get_weight_commits(netuid, hotkey).await?;
            if let Some((_hash, _commit_block, first_reveal_block, _last_reveal_block)) = commits
                .into_iter()
                .find(|(hash, _, _, _)| *hash == expected_hash)
            {
                return Ok(CommitWindow { first_reveal_block });
            }
            tokio::time::sleep(DEFAULT_POLL_INTERVAL).await;
        }
        Err(ChainClientError::Timeout {
            operation: format!("commit_weights for subnet {netuid}"),
            timeout_secs: timeout.as_secs(),
        })
    }

    async fn wait_for_reveal_block(
        &self,
        first_reveal_block: u64,
        timeout: Duration,
    ) -> Result<(), ChainClientError> {
        let deadline = Instant::now()
            .checked_add(timeout)
            .unwrap_or_else(Instant::now);
        while Instant::now() < deadline {
            let header = self.chain_get_header().await?;
            if parse_header_number(&header.number)? >= first_reveal_block {
                return Ok(());
            }
            tokio::time::sleep(DEFAULT_POLL_INTERVAL).await;
        }
        Err(ChainClientError::Timeout {
            operation: format!("reveal window at block {first_reveal_block}"),
            timeout_secs: timeout.as_secs(),
        })
    }
}

fn signed_extrinsic_hex(
    signer: sr25519::Pair,
    signing_context: &SigningContext,
    call: runtime::RuntimeCall,
) -> Result<String, ChainClientError> {
    let period = runtime::BlockHashCount::get()
        .checked_next_power_of_two()
        .map(|count| count / 2)
        .unwrap_or(2) as u64;
    let extra: runtime::TransactionExtensions = (
        CheckNonZeroSender::<runtime::Runtime>::new(),
        CheckSpecVersion::<runtime::Runtime>::new(),
        CheckTxVersion::<runtime::Runtime>::new(),
        CheckGenesis::<runtime::Runtime>::new(),
        CheckEra::<runtime::Runtime>::from(Era::mortal(period, signing_context.best_number)),
        runtime::check_nonce::CheckNonce::<runtime::Runtime>::from(signing_context.nonce),
        CheckWeight::<runtime::Runtime>::new(),
        ChargeTransactionPayment::<runtime::Runtime>::from(0),
        CheckMetadataHash::<runtime::Runtime>::new(false),
    );
    let additional_signed = (
        (),
        signing_context.spec_version,
        signing_context.transaction_version,
        signing_context.genesis_hash,
        signing_context.best_hash,
        (),
        (),
        (),
        None,
    );
    let payload = runtime::SignedPayload::from_raw(call.clone(), extra.clone(), additional_signed);
    let signature = payload.using_encoded(|bytes| signer.sign(bytes));
    let extrinsic = runtime::UncheckedExtrinsic::new_signed(
        call,
        AccountId32::from(signer.public()).into(),
        Signature::Sr25519(signature),
        extra,
    );

    Ok(format!("0x{}", hex::encode(extrinsic.encode())))
}

fn normalize_ws_endpoint(endpoint: &str) -> Result<Cow<'_, str>, ChainClientError> {
    let endpoint = endpoint.trim();
    if endpoint.is_empty() {
        return Err(ChainClientError::EmptyEndpoint);
    }
    if endpoint.contains("://") {
        return Ok(Cow::Borrowed(endpoint));
    }
    Ok(Cow::Owned(format!("{DEFAULT_WS_SCHEME}{endpoint}")))
}

fn parse_h256(value: String) -> Result<H256, ChainClientError> {
    let bytes = hex::decode(value.trim_start_matches("0x")).map_err(|source| {
        ChainClientError::InvalidHash {
            value: value.clone(),
            source,
        }
    })?;
    Ok(H256::from_slice(&bytes))
}

fn h256_hex(value: &H256) -> String {
    format!("0x{}", hex::encode(value.as_bytes()))
}

fn parse_header_number(value: &str) -> Result<u64, ChainClientError> {
    let trimmed = value.trim_start_matches("0x");
    u64::from_str_radix(trimmed, 16).map_err(|source| ChainClientError::InvalidHeaderNumber {
        value: value.to_string(),
        source,
    })
}

fn decode_hex_payload(method: &'static str, value: &str) -> Result<Vec<u8>, ChainClientError> {
    hex::decode(value.trim_start_matches("0x"))
        .map_err(|source| ChainClientError::InvalidHexPayload { method, source })
}

fn uid_storage_key(netuid: NetUid, hotkey: &AccountId32) -> Vec<u8> {
    double_map_identity_blake2_storage_key("SubtensorModule", "Uids", &netuid, hotkey)
}

fn key_storage_key(netuid: NetUid, uid: u16) -> Vec<u8> {
    double_map_identity_identity_storage_key("SubtensorModule", "Keys", &netuid, &uid)
}

fn storage_prefix(pallet: &str, storage: &str) -> Vec<u8> {
    let mut key = Vec::new();
    key.extend_from_slice(&twox_128(pallet.as_bytes()));
    key.extend_from_slice(&twox_128(storage.as_bytes()));
    key
}

fn axon_storage_key(netuid: NetUid, hotkey: &AccountId32) -> Vec<u8> {
    double_map_identity_blake2_storage_key("SubtensorModule", "Axons", &netuid, hotkey)
}

fn weights_storage_key(netuid: NetUid, uid: u16) -> Vec<u8> {
    double_map_identity_identity_storage_key(
        "SubtensorModule",
        "Weights",
        &NetUidStorageIndex::from(netuid),
        &uid,
    )
}

fn hotkey_alpha_storage_key(hotkey: &AccountId32, netuid: NetUid) -> Vec<u8> {
    double_map_blake2_identity_storage_key("SubtensorModule", "TotalHotkeyAlpha", hotkey, &netuid)
}

fn weight_commits_storage_key(netuid: NetUid, hotkey: &AccountId32) -> Vec<u8> {
    double_map_twox64_twox64_storage_key(
        "SubtensorModule",
        "WeightCommits",
        &NetUidStorageIndex::from(netuid),
        hotkey,
    )
}

fn network_last_registered_storage_key() -> Vec<u8> {
    map_identity_storage_key(
        "SubtensorModule",
        "LastRateLimitedBlock",
        &pallet_game_solver::RateLimitKey::<AccountId32>::NetworkLastRegistered,
    )
}

fn map_identity_storage_key<K: Encode>(pallet: &str, storage: &str, key1: &K) -> Vec<u8> {
    let mut key = Vec::new();
    key.extend_from_slice(&twox_128(pallet.as_bytes()));
    key.extend_from_slice(&twox_128(storage.as_bytes()));
    key.extend_from_slice(&key1.encode());
    key
}

/// Formats one axon entry as a socket endpoint when the stored fields are usable.
pub fn format_axon_endpoint(axon: &AxonInfo) -> Option<String> {
    if axon.port == 0 {
        return None;
    }

    match axon.ip_type {
        4 => {
            let ip = Ipv4Addr::from(u32::try_from(axon.ip).ok()?);
            Some(format!("{ip}:{}", axon.port))
        }
        6 => {
            let ip = Ipv6Addr::from(axon.ip);
            Some(format!("[{ip}]:{}", axon.port))
        }
        _ => None,
    }
}

fn double_map_identity_blake2_storage_key<K1: Encode, K2: Encode>(
    pallet: &str,
    storage: &str,
    key1: &K1,
    key2: &K2,
) -> Vec<u8> {
    let mut key = Vec::new();
    key.extend_from_slice(&twox_128(pallet.as_bytes()));
    key.extend_from_slice(&twox_128(storage.as_bytes()));
    key.extend_from_slice(&key1.encode());
    let key2_bytes = key2.encode();
    key.extend_from_slice(&blake2_128(&key2_bytes));
    key.extend_from_slice(&key2_bytes);
    key
}

fn double_map_identity_identity_storage_key<K1: Encode, K2: Encode>(
    pallet: &str,
    storage: &str,
    key1: &K1,
    key2: &K2,
) -> Vec<u8> {
    let mut key = Vec::new();
    key.extend_from_slice(&twox_128(pallet.as_bytes()));
    key.extend_from_slice(&twox_128(storage.as_bytes()));
    key.extend_from_slice(&key1.encode());
    key.extend_from_slice(&key2.encode());
    key
}

fn double_map_blake2_identity_storage_key<K1: Encode, K2: Encode>(
    pallet: &str,
    storage: &str,
    key1: &K1,
    key2: &K2,
) -> Vec<u8> {
    let mut key = Vec::new();
    key.extend_from_slice(&twox_128(pallet.as_bytes()));
    key.extend_from_slice(&twox_128(storage.as_bytes()));
    let key1_bytes = key1.encode();
    key.extend_from_slice(&blake2_128(&key1_bytes));
    key.extend_from_slice(&key1_bytes);
    key.extend_from_slice(&key2.encode());
    key
}

fn double_map_twox64_twox64_storage_key<K1: Encode, K2: Encode>(
    pallet: &str,
    storage: &str,
    key1: &K1,
    key2: &K2,
) -> Vec<u8> {
    let mut key = Vec::new();
    let key1_bytes = key1.encode();
    let key2_bytes = key2.encode();
    key.extend_from_slice(&twox_128(pallet.as_bytes()));
    key.extend_from_slice(&twox_128(storage.as_bytes()));
    key.extend_from_slice(&twox_64(&key1_bytes));
    key.extend_from_slice(&key1_bytes);
    key.extend_from_slice(&twox_64(&key2_bytes));
    key.extend_from_slice(&key2_bytes);
    key
}

fn weight_commit_hash(
    hotkey: &AccountId32,
    netuid: NetUid,
    uids: &[u16],
    values: &[u16],
    salt: &[u16],
    version_key: u64,
) -> H256 {
    BlakeTwo256::hash_of(&(
        hotkey.clone(),
        NetUidStorageIndex::from(netuid),
        uids,
        values,
        salt,
        version_key,
    ))
}

fn commit_reveal_salt(validator_uid: u16, target_uid: u16) -> Vec<u16> {
    vec![validator_uid, target_uid, u16::MAX]
}

#[cfg(test)]
fn extrinsic_hash_hex(extrinsic_hex: &str) -> Result<String, ChainClientError> {
    let bytes = decode_hex_payload("author_submitAndWatchExtrinsic", extrinsic_hex)?;
    Ok(format!("0x{}", hex::encode(BlakeTwo256::hash(&bytes))))
}

fn locate_block_extrinsic_index(
    extrinsics: &[String],
    target_extrinsic_hex: &str,
    target_extrinsic_hash: &str,
) -> Result<Option<u32>, ChainClientError> {
    let normalized_target = target_extrinsic_hex.to_ascii_lowercase();
    if let Some(index) = extrinsics
        .iter()
        .position(|extrinsic| extrinsic.to_ascii_lowercase() == normalized_target)
    {
        return Ok(u32::try_from(index).ok());
    }
    let target_hash = parse_h256(target_extrinsic_hash.to_string())?;
    for (index, extrinsic) in extrinsics.iter().enumerate() {
        let bytes = decode_hex_payload("chain_getBlock", extrinsic)?;
        if BlakeTwo256::hash(&bytes) == target_hash {
            return Ok(u32::try_from(index).ok());
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::ChainClient;
    use super::ChainClientError;
    use super::DEFAULT_NETWORK_RATE_LIMIT;
    use super::DEFAULT_SUBNET_TEMPO;
    use super::RpcMethods;
    use super::axon_storage_key;
    use super::extrinsic_hash_hex;
    use super::format_axon_endpoint;
    use super::hotkey_alpha_storage_key;
    use super::key_storage_key;
    use super::locate_block_extrinsic_index;
    use super::map_identity_storage_key;
    use super::normalize_ws_endpoint;
    use super::parse_header_number;
    use super::storage_prefix;
    use super::uid_storage_key;
    use super::weights_storage_key;
    use pallet_game_solver::AxonInfo;
    use pallet_game_solver::RateLimitKey;
    use sp_runtime::AccountId32;
    use subtensor_runtime_common::NetUid;

    #[test]
    fn normalize_ws_endpoint_rejects_empty_values() {
        let error = normalize_ws_endpoint("   ").expect_err("empty endpoint should fail");
        assert!(matches!(error, ChainClientError::EmptyEndpoint));
    }

    #[test]
    fn normalize_ws_endpoint_preserves_existing_scheme() {
        let endpoint =
            normalize_ws_endpoint("wss://rpc.myosu.example").expect("endpoint should normalize");
        assert_eq!(endpoint.as_ref(), "wss://rpc.myosu.example");
    }

    #[test]
    fn normalize_ws_endpoint_adds_default_scheme() {
        let endpoint = normalize_ws_endpoint("127.0.0.1:9944").expect("endpoint should normalize");
        assert_eq!(endpoint.as_ref(), "ws://127.0.0.1:9944");
    }

    #[test]
    fn parse_header_number_accepts_hex_values() {
        let number = parse_header_number("0x28").expect("header number should parse");
        assert_eq!(number, 40);
    }

    #[test]
    fn storage_keys_use_distinct_prefixes() {
        let hotkey = AccountId32::new([7; 32]);
        let uid_key = uid_storage_key(NetUid::from(1_u16), &hotkey);
        let key_key = key_storage_key(NetUid::from(1_u16), 7);
        let axon_key = axon_storage_key(NetUid::from(1_u16), &hotkey);
        let weights_key = weights_storage_key(NetUid::from(1_u16), 7);
        let stake_key = hotkey_alpha_storage_key(&hotkey, NetUid::from(1_u16));
        let version_key =
            map_identity_storage_key("SubtensorModule", "WeightsVersionKey", &NetUid::from(1_u16));

        assert_ne!(uid_key, axon_key);
        assert_ne!(key_key, uid_key);
        assert_ne!(key_key, weights_key);
        assert_ne!(uid_key, weights_key);
        assert_ne!(axon_key, weights_key);
        assert_ne!(stake_key, uid_key);
        assert_ne!(stake_key, axon_key);
        assert!(uid_key.len() > 32);
        assert!(axon_key.len() > 32);
        assert!(weights_key.len() > version_key.len());
        assert!(stake_key.len() > version_key.len());
    }

    #[test]
    fn account_id_from_uri_uses_sr25519_dev_phrase() {
        let account = ChainClient::account_id_from_uri("//Alice")
            .expect("Alice URI should decode to account");
        assert_eq!(
            account.to_string(),
            "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
        );
    }

    #[test]
    fn rpc_methods_version_is_optional() {
        let payload = serde_json::from_str::<RpcMethods>(r#"{"methods":["system_health"]}"#)
            .expect("rpc_methods payload should decode without version");
        assert_eq!(payload.version, None);
        assert_eq!(payload.methods, vec!["system_health".to_string()]);
    }

    #[test]
    fn storage_prefix_matches_map_prefix_length() {
        let prefix = storage_prefix("SubtensorModule", "NetworksAdded");
        assert_eq!(prefix.len(), 32);
    }

    #[test]
    fn network_last_registered_storage_key_matches_pallet() {
        let client_key = map_identity_storage_key(
            "SubtensorModule",
            "LastRateLimitedBlock",
            &RateLimitKey::<AccountId32>::NetworkLastRegistered,
        );
        let pallet_key =
            pallet_game_solver::LastRateLimitedBlock::<crate::runtime::Runtime>::hashed_key_for(
                &RateLimitKey::<AccountId32>::NetworkLastRegistered,
            );

        assert_eq!(client_key, pallet_key);
    }

    #[test]
    fn default_network_rate_limit_matches_runtime_constant() {
        assert_eq!(
            DEFAULT_NETWORK_RATE_LIMIT,
            crate::runtime::SubtensorInitialNetworkRateLimit::get()
        );
        assert_eq!(DEFAULT_NETWORK_RATE_LIMIT, 7200);
    }

    #[test]
    fn default_subnet_tempo_matches_runtime_constant() {
        assert_eq!(
            DEFAULT_SUBNET_TEMPO,
            crate::runtime::SubtensorInitialTempo::get()
        );
        assert_eq!(DEFAULT_SUBNET_TEMPO, 360);
    }

    #[test]
    fn format_axon_endpoint_supports_ipv4_and_ipv6() {
        let ipv4 = AxonInfo {
            ip: u128::from(0x7f000002_u32),
            port: 8080,
            ip_type: 4,
            ..AxonInfo::default()
        };
        let ipv6 = AxonInfo {
            ip: 1,
            port: 9090,
            ip_type: 6,
            ..AxonInfo::default()
        };

        assert_eq!(
            format_axon_endpoint(&ipv4).as_deref(),
            Some("127.0.0.2:8080")
        );
        assert_eq!(format_axon_endpoint(&ipv6).as_deref(), Some("[::1]:9090"));
    }

    #[test]
    fn format_axon_endpoint_rejects_invalid_entries() {
        let zero_port = AxonInfo {
            ip: u128::from(0x7f000002_u32),
            port: 0,
            ip_type: 4,
            ..AxonInfo::default()
        };
        let invalid_type = AxonInfo {
            ip: 9,
            port: 8080,
            ip_type: 9,
            ..AxonInfo::default()
        };

        assert_eq!(format_axon_endpoint(&zero_port), None);
        assert_eq!(format_axon_endpoint(&invalid_type), None);
    }

    #[test]
    fn watched_extrinsic_helpers_recover_hash_and_index() {
        let extrinsic_hex = "0x04030201";
        let extrinsic_hash =
            extrinsic_hash_hex(extrinsic_hex).expect("extrinsic hash should derive from bytes");
        let index = locate_block_extrinsic_index(
            &[extrinsic_hex.to_string(), "0xdeadbeef".to_string()],
            extrinsic_hex,
            &extrinsic_hash,
        )
        .expect("extrinsic index should resolve");
        assert_eq!(index, Some(0));
    }
}

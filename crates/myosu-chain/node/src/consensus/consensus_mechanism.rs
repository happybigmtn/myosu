use jsonrpsee::Methods;
use myosu_chain_runtime::opaque::Block;
use sc_client_api::{AuxStore, BlockOf};
use sc_consensus::BlockImport;
use sc_consensus_aura::AuraApi;
use sc_consensus_slots::{BackoffAuthoringBlocksStrategy, InherentDataProviderExt, SlotProportion};
use sc_network_sync::SyncingService;
use sc_service::{TaskManager, error::Error as ServiceError};
use sc_telemetry::TelemetryHandle;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::{HeaderBackend, HeaderMetadata};
use sp_consensus::{Environment, Proposer, SelectChain, SyncOracle};
use sp_consensus_aura::sr25519::AuthorityId as AuraAuthorityId;
use sp_consensus_babe::BabeApi;
use sp_consensus_slots::SlotDuration;
use sp_inherents::CreateInherentDataProviders;
use sp_keystore::KeystorePtr;
use sp_runtime::traits::NumberFor;
use std::sync::{Arc, atomic::AtomicBool};

use crate::client::FullClient;
use crate::service::{BIQ, FullSelectChain};

/// Parameters shared by consensus-specific authoring startup.
// The restart skeleton keeps the full authoring parameter surface so service code stays honest,
// even though the temporary consensus implementations do not read every field yet.
#[allow(dead_code)]
pub struct StartAuthoringParams<C, SC, I, PF, SO, L, CIDP, BS> {
    /// The duration of a slot.
    pub slot_duration: SlotDuration,
    /// The client to interact with the chain.
    pub client: Arc<C>,
    /// A select chain implementation to select the best block.
    pub select_chain: SC,
    /// The block import.
    pub block_import: I,
    /// The proposer factory to build proposer instances.
    pub proposer_factory: PF,
    /// The sync oracle that can give us the current sync status.
    pub sync_oracle: SO,
    /// Hook into the sync module to control the justification sync process.
    pub justification_sync_link: L,
    /// Something that can create the inherent data providers.
    pub create_inherent_data_providers: CIDP,
    /// Whether to force authoring of blocks.
    pub force_authoring: bool,
    /// The backoff strategy when slots are missed.
    pub backoff_authoring_blocks: Option<BS>,
    /// The keystore used by the node.
    pub keystore: KeystorePtr,
    /// The proportion of the slot dedicated to proposing.
    pub block_proposal_slot_portion: SlotProportion,
    /// The max proportion of the slot dedicated to proposing with lenience.
    pub max_block_proposal_slot_portion: Option<SlotProportion>,
    /// Optional telemetry handle.
    pub telemetry: Option<TelemetryHandle>,
}

/// Shared interface for the node's supported consensus mechanisms.
pub trait ConsensusMechanism {
    /// Inherent data providers inserted into blocks by this mechanism.
    type InherentDataProviders: sp_inherents::InherentDataProvider
        + InherentDataProviderExt
        + 'static;

    /// Creates a new instance of the consensus mechanism.
    fn new() -> Self;

    /// Builds the import queue closure for this mechanism.
    fn build_biq(&mut self) -> Result<BIQ<'_>, sc_service::Error>;

    /// Returns the slot duration.
    fn slot_duration(&self, client: &FullClient) -> Result<SlotDuration, ServiceError>;

    /// Creates inherent data providers for authored blocks.
    fn create_inherent_data_providers(
        slot_duration: SlotDuration,
    ) -> Result<Self::InherentDataProviders, Box<dyn std::error::Error + Send + Sync>>;

    /// Starts the consensus-specific authoring process.
    fn start_authoring<C, SC, I, PF, SO, L, CIDP, BS, Error>(
        self,
        task_manager: &mut TaskManager,
        params: StartAuthoringParams<C, SC, I, PF, SO, L, CIDP, BS>,
    ) -> Result<(), sp_consensus::Error>
    where
        C: ProvideRuntimeApi<Block>
            + BlockOf
            + AuxStore
            + HeaderBackend<Block>
            + HeaderMetadata<Block, Error = sp_blockchain::Error>
            + Send
            + Sync
            + 'static,
        C::Api: AuraApi<Block, AuraAuthorityId> + BabeApi<Block>,
        SC: SelectChain<Block> + 'static,
        I: BlockImport<Block, Error = sp_consensus::Error> + Send + Sync + 'static,
        PF: Environment<Block, Error = Error> + Send + Sync + 'static,
        PF::Proposer: Proposer<Block, Error = Error>,
        SO: SyncOracle + Send + Sync + Clone + 'static,
        L: sc_consensus::JustificationSyncLink<Block> + 'static,
        CIDP: CreateInherentDataProviders<Block, ()> + Send + Sync + 'static,
        CIDP::InherentDataProviders: InherentDataProviderExt + Send,
        BS: BackoffAuthoringBlocksStrategy<NumberFor<Block>> + Send + Sync + 'static,
        Error: std::error::Error + Send + From<sp_consensus::Error> + From<I::Error> + 'static;

    /// Spawns any consensus-specific essential handles.
    fn spawn_essential_handles(
        &self,
        task_manager: &mut TaskManager,
        client: Arc<FullClient>,
        custom_service_signal: Option<Arc<AtomicBool>>,
        sync_service: Arc<SyncingService<Block>>,
    ) -> Result<(), ServiceError>;

    /// Returns consensus-specific RPC methods to register.
    fn rpc_methods(
        &self,
        client: Arc<FullClient>,
        keystore: KeystorePtr,
        select_chain: FullSelectChain,
    ) -> Result<Vec<Methods>, sc_service::Error>;
}

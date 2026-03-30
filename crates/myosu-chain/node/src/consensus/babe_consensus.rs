use jsonrpsee::Methods;
use myosu_chain_runtime::opaque::Block;
use sc_consensus::BoxBlockImport;
use sc_consensus_babe::{BabeLink, BabeWorkerHandle};
use sc_network_sync::SyncingService;
use sc_service::{TaskManager, error::Error as ServiceError};
use sc_telemetry::TelemetryHandle;
use sc_transaction_pool::TransactionPoolHandle;
use sc_transaction_pool_api::OffchainTransactionPoolFactory;
use sp_consensus_slots::SlotDuration;
use sp_keystore::KeystorePtr;
use std::sync::{Arc, atomic::AtomicBool};

use crate::client::{FullBackend, FullClient};
use crate::consensus::{ConsensusMechanism, StartAuthoringParams};
use crate::service::{BIQ, FullSelectChain, GrandpaBlockImport};

/// Babe consensus implementation for the restarted node surface.
pub struct BabeConsensus {
    babe_link: Option<BabeLink<Block>>,
    babe_worker_handle: Option<BabeWorkerHandle<Block>>,
}

impl ConsensusMechanism for BabeConsensus {
    type InherentDataProviders = (
        sp_consensus_babe::inherents::InherentDataProvider,
        sp_timestamp::InherentDataProvider,
    );

    fn new() -> Self {
        Self {
            babe_link: None,
            babe_worker_handle: None,
        }
    }

    fn build_biq(&mut self) -> Result<BIQ<'_>, sc_service::Error> {
        let build_import_queue = Box::new(
            move |client: Arc<FullClient>,
                  backend: Arc<FullBackend>,
                  config: &sc_service::Configuration,
                  task_manager: &TaskManager,
                  telemetry: Option<TelemetryHandle>,
                  grandpa_block_import: GrandpaBlockImport,
                  transaction_pool: Arc<TransactionPoolHandle<Block, FullClient>>| {
                let configuration = sc_consensus_babe::configuration(&*client)?;
                if configuration.slot_duration == 0 {
                    return Err(sc_service::Error::Client(
                        sp_blockchain::Error::VersionInvalid(
                            "Unsupported or invalid BabeApi version".to_string(),
                        ),
                    ));
                }

                let (babe_import, babe_link) = sc_consensus_babe::block_import(
                    configuration,
                    grandpa_block_import.clone(),
                    client.clone(),
                )?;

                let slot_duration = babe_link.config().slot_duration();
                let create_inherent_data_providers = move |_, ()| async move {
                    let timestamp = sp_timestamp::InherentDataProvider::from_system_time();
                    let slot = sp_consensus_babe::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
                        *timestamp,
                        slot_duration,
                    );
                    Ok((slot, timestamp))
                };

                let (import_queue, babe_worker_handle) =
                    sc_consensus_babe::import_queue(sc_consensus_babe::ImportQueueParams {
                        link: babe_link.clone(),
                        block_import: babe_import.clone(),
                        justification_import: Some(Box::new(grandpa_block_import)),
                        client,
                        select_chain: sc_consensus::LongestChain::new(backend.clone()),
                        create_inherent_data_providers,
                        spawner: &task_manager.spawn_essential_handle(),
                        registry: config.prometheus_registry(),
                        telemetry,
                        offchain_tx_pool_factory: OffchainTransactionPoolFactory::new(
                            transaction_pool,
                        ),
                    })?;

                self.babe_link = Some(babe_link);
                self.babe_worker_handle = Some(babe_worker_handle);
                Ok((import_queue, Box::new(babe_import) as BoxBlockImport<Block>))
            },
        );

        Ok(build_import_queue)
    }

    fn slot_duration(&self, client: &FullClient) -> Result<SlotDuration, ServiceError> {
        if let Some(link) = &self.babe_link {
            Ok(link.config().slot_duration())
        } else {
            let configuration = sc_consensus_babe::configuration(client)?;
            Ok(configuration.slot_duration())
        }
    }

    fn create_inherent_data_providers(
        slot_duration: SlotDuration,
    ) -> Result<Self::InherentDataProviders, Box<dyn std::error::Error + Send + Sync>> {
        let timestamp = sp_timestamp::InherentDataProvider::from_system_time();
        let slot =
            sp_consensus_babe::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
                *timestamp,
                slot_duration,
            );
        Ok((slot, timestamp))
    }

    fn start_authoring<C, SC, I, PF, SO, L, CIDP, BS, Error>(
        self,
        task_manager: &mut TaskManager,
        params: StartAuthoringParams<C, SC, I, PF, SO, L, CIDP, BS>,
    ) -> Result<(), sp_consensus::Error>
    where
        C: sp_api::ProvideRuntimeApi<Block>
            + sc_client_api::BlockOf
            + sc_client_api::AuxStore
            + sp_blockchain::HeaderBackend<Block>
            + sp_blockchain::HeaderMetadata<Block, Error = sp_blockchain::Error>
            + Send
            + Sync
            + 'static,
        C::Api: sp_consensus_aura::AuraApi<Block, sp_consensus_aura::sr25519::AuthorityId>
            + sp_consensus_babe::BabeApi<Block>,
        SC: sp_consensus::SelectChain<Block> + 'static,
        I: sc_consensus::BlockImport<Block, Error = sp_consensus::Error> + Send + Sync + 'static,
        PF: sp_consensus::Environment<Block, Error = Error> + Send + Sync + 'static,
        PF::Proposer: sp_consensus::Proposer<Block, Error = Error>,
        SO: sp_consensus::SyncOracle + Send + Sync + Clone + 'static,
        L: sc_consensus::JustificationSyncLink<Block> + 'static,
        CIDP: sp_inherents::CreateInherentDataProviders<Block, ()> + Send + Sync + 'static,
        CIDP::InherentDataProviders: sc_consensus_slots::InherentDataProviderExt + Send,
        BS: sc_consensus_slots::BackoffAuthoringBlocksStrategy<sp_runtime::traits::NumberFor<Block>>
            + Send
            + Sync
            + 'static,
        Error: std::error::Error + Send + From<sp_consensus::Error> + From<I::Error> + 'static,
    {
        let Some(babe_link) = self.babe_link else {
            return Err(sp_consensus::Error::ClientImport(
                "build the Babe import queue before authoring".to_string(),
            ));
        };

        let babe = sc_consensus_babe::start_babe::<Block, C, SC, PF, I, SO, CIDP, BS, L, Error>(
            sc_consensus_babe::BabeParams {
                keystore: params.keystore,
                client: params.client,
                select_chain: params.select_chain,
                env: params.proposer_factory,
                block_import: params.block_import,
                sync_oracle: params.sync_oracle,
                justification_sync_link: params.justification_sync_link,
                create_inherent_data_providers: params.create_inherent_data_providers,
                force_authoring: params.force_authoring,
                backoff_authoring_blocks: params.backoff_authoring_blocks,
                babe_link,
                block_proposal_slot_portion: params.block_proposal_slot_portion,
                max_block_proposal_slot_portion: params.max_block_proposal_slot_portion,
                telemetry: params.telemetry,
            },
        )?;

        task_manager.spawn_essential_handle().spawn_blocking(
            "babe-proposer",
            Some("block-authoring"),
            babe,
        );

        Ok(())
    }

    fn spawn_essential_handles(
        &self,
        _task_manager: &mut TaskManager,
        _client: Arc<FullClient>,
        _custom_service_signal: Option<Arc<AtomicBool>>,
        _sync_service: Arc<SyncingService<Block>>,
    ) -> Result<(), ServiceError> {
        Ok(())
    }

    fn rpc_methods(
        &self,
        client: Arc<FullClient>,
        keystore: KeystorePtr,
        select_chain: FullSelectChain,
    ) -> Result<Vec<Methods>, sc_service::Error> {
        use sc_consensus_babe_rpc::{Babe, BabeApiServer};

        if let Some(handle) = &self.babe_worker_handle {
            Ok(vec![
                Babe::new(client, handle.clone(), keystore, select_chain)
                    .into_rpc()
                    .into(),
            ])
        } else {
            Err(sc_service::Error::Other(
                "Babe worker handle not initialized yet".to_string(),
            ))
        }
    }
}

use jsonrpsee::Methods;
use jsonrpsee::tokio;
use myosu_chain_runtime::opaque::Block;
use sc_client_api::AuxStore;
use sc_consensus::BoxBlockImport;
use sc_network_sync::SyncingService;
use sc_service::{TaskManager, error::Error as ServiceError};
use sc_telemetry::TelemetryHandle;
use sc_transaction_pool::TransactionPoolHandle;
use sp_api::ProvideRuntimeApi;
use sp_consensus_aura::AuraApi;
use sp_consensus_aura::sr25519::{AuthorityId as AuraAuthorityId, AuthorityPair as AuraPair};
use sp_consensus_babe::{AuthorityId as BabeAuthorityId, BabeConfiguration};
use sp_consensus_slots::SlotDuration;
use sp_keystore::KeystorePtr;
use sp_runtime::traits::Block as BlockT;
use std::sync::{Arc, atomic::AtomicBool};

use crate::client::{FullBackend, FullClient};
use crate::consensus::{ConsensusMechanism, StartAuthoringParams};
use crate::service::{BIQ, FullSelectChain, GrandpaBlockImport};

/// Aura consensus implementation for the restarted node surface.
pub struct AuraConsensus;

impl ConsensusMechanism for AuraConsensus {
    type InherentDataProviders = (
        sp_consensus_aura::inherents::InherentDataProvider,
        sp_timestamp::InherentDataProvider,
    );

    fn new() -> Self {
        Self
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
                let slot_duration = sc_consensus_aura::slot_duration(&*client)?;
                let create_inherent_data_providers = move |_, ()| async move {
                    let timestamp = sp_timestamp::InherentDataProvider::from_system_time();
                    let slot = sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
                        *timestamp,
                        slot_duration,
                    );
                    Ok((slot, timestamp))
                };

                let import_queue = sc_consensus_aura::import_queue::<AuraPair, _, _, _, _, _>(
                    sc_consensus_aura::ImportQueueParams {
                        block_import: grandpa_block_import.clone(),
                        justification_import: Some(Box::new(grandpa_block_import.clone())),
                        client: client.clone(),
                        create_inherent_data_providers,
                        spawner: &task_manager.spawn_essential_handle(),
                        registry: config.prometheus_registry(),
                        check_for_equivocation: Default::default(),
                        telemetry,
                        compatibility_mode: Default::default(),
                    },
                )
                .map_err(sc_service::Error::from)?;

                let block_import = Box::new(grandpa_block_import.clone()) as BoxBlockImport<Block>;

                let _ = get_expected_babe_configuration::<Block, _>(&*client);
                let _ = backend;
                let _ = transaction_pool;
                Ok((import_queue, block_import))
            },
        );

        Ok(build_import_queue)
    }

    fn slot_duration(&self, client: &FullClient) -> Result<SlotDuration, ServiceError> {
        sc_consensus_aura::slot_duration(client).map_err(Into::into)
    }

    fn create_inherent_data_providers(
        slot_duration: SlotDuration,
    ) -> Result<Self::InherentDataProviders, Box<dyn std::error::Error + Send + Sync>> {
        let timestamp = sp_timestamp::InherentDataProvider::from_system_time();
        let slot =
            sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
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
        let aura = sc_consensus_aura::start_aura::<AuraPair, Block, _, _, _, _, _, _, _, _, _>(
            sc_consensus_aura::StartAuraParams {
                slot_duration: params.slot_duration,
                client: params.client,
                select_chain: params.select_chain,
                block_import: params.block_import,
                proposer_factory: params.proposer_factory,
                sync_oracle: params.sync_oracle,
                justification_sync_link: params.justification_sync_link,
                create_inherent_data_providers: params.create_inherent_data_providers,
                force_authoring: params.force_authoring,
                backoff_authoring_blocks: params.backoff_authoring_blocks,
                keystore: params.keystore,
                block_proposal_slot_portion: params.block_proposal_slot_portion,
                max_block_proposal_slot_portion: params.max_block_proposal_slot_portion,
                telemetry: params.telemetry,
                compatibility_mode: Default::default(),
            },
        )?;

        task_manager
            .spawn_essential_handle()
            .spawn_blocking("aura", Some("block-authoring"), aura);

        Ok(())
    }

    fn spawn_essential_handles(
        &self,
        task_manager: &mut TaskManager,
        client: Arc<FullClient>,
        custom_service_signal: Option<Arc<AtomicBool>>,
        sync_service: Arc<SyncingService<Block>>,
    ) -> Result<(), ServiceError> {
        let client_clone = Arc::clone(&client);
        let signal = custom_service_signal.clone();
        let slot_duration = self.slot_duration(&client)?;
        task_manager.spawn_essential_handle().spawn(
            "babe-switch",
            None,
            Box::pin(async move {
                loop {
                    if let Ok(configuration) = sc_consensus_babe::configuration(&*client_clone) {
                        let syncing = sync_service.status().await.is_ok_and(|status| {
                            status.warp_sync.is_some() || status.state_sync.is_some()
                        });
                        if !configuration.authorities.is_empty() && !syncing {
                            log::info!(
                                "Babe runtime detected; exiting Aura service to allow Babe restart."
                            );
                            if let Some(signal) = &signal {
                                signal.store(true, std::sync::atomic::Ordering::SeqCst);
                            }
                            break;
                        }
                    }
                    tokio::time::sleep(slot_duration.as_duration()).await;
                }
            }),
        );
        Ok(())
    }

    fn rpc_methods(
        &self,
        _client: Arc<FullClient>,
        _keystore: KeystorePtr,
        _select_chain: FullSelectChain,
    ) -> Result<Vec<Methods>, sc_service::Error> {
        Ok(Vec::new())
    }
}

fn get_expected_babe_configuration<B, C>(client: &C) -> sp_blockchain::Result<BabeConfiguration>
where
    B: BlockT,
    C: AuxStore + ProvideRuntimeApi<B> + sc_client_api::UsageProvider<B>,
    C::Api: AuraApi<B, AuraAuthorityId>,
{
    let usage_info = client.usage_info();
    let at_hash = if usage_info.chain.finalized_state.is_some() {
        usage_info.chain.best_hash
    } else {
        usage_info.chain.genesis_hash
    };

    let runtime_api = client.runtime_api();
    let authorities = runtime_api
        .authorities(at_hash)?
        .into_iter()
        .map(|authority| (BabeAuthorityId::from(authority.into_inner()), 1))
        .collect();

    let slot_duration = runtime_api.slot_duration(at_hash)?.as_millis();
    let epoch_config = myosu_chain_runtime::BABE_GENESIS_EPOCH_CONFIG;
    Ok(BabeConfiguration {
        slot_duration,
        epoch_length: myosu_chain_runtime::EPOCH_DURATION_IN_SLOTS,
        c: epoch_config.c,
        authorities,
        randomness: Default::default(),
        allowed_slots: epoch_config.allowed_slots,
    })
}

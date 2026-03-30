//! A collection of node-specific RPC methods.
//! Substrate provides the `sc-rpc` crate, which defines the core RPC layer
//! used by Substrate nodes. This file extends those RPC definitions with
//! capabilities that are specific to this project's runtime configuration.

#![warn(missing_docs)]

use std::sync::Arc;
use std::time::Instant;

use futures::channel::mpsc;

use crate::client::FullClient;
use jsonrpsee::{Methods, RpcModule};
use myosu_chain_runtime::opaque::Block;
use sc_consensus_manual_seal::EngineCommand;
use sc_rpc::SubscriptionTaskExecutor;
use sc_transaction_pool_api::TransactionPool;
use sp_runtime::{OpaqueExtrinsic, traits::BlakeTwo256, traits::Block as BlockT};
use subtensor_runtime_common::Hash;

const LOG_TARGET: &str = "node-rpc";

/// Full client dependencies.
pub struct FullDeps<P> {
    /// The client instance to use.
    pub client: Arc<FullClient>,
    /// Transaction pool instance.
    pub pool: Arc<P>,
    /// Manual seal command sink
    pub command_sink: Option<mpsc::Sender<EngineCommand<Hash>>>,
}

/// Instantiate all full RPC extensions.
pub fn create_full<P>(
    deps: FullDeps<P>,
    _subscription_task_executor: SubscriptionTaskExecutor,
    other_methods: &[Methods],
) -> Result<RpcModule<()>, Box<dyn std::error::Error + Send + Sync>>
where
    P: TransactionPool<
            Block = Block,
            Hash = <sp_runtime::generic::Block<
                sp_runtime::generic::Header<u32, BlakeTwo256>,
                OpaqueExtrinsic,
            > as BlockT>::Hash,
        > + 'static,
{
    let build_started_at = Instant::now();
    use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApiServer};
    use sc_consensus_manual_seal::rpc::{ManualSeal, ManualSealApiServer};
    use substrate_frame_rpc_system::{System, SystemApiServer};
    use subtensor_custom_rpc::{SubtensorCustom, SubtensorCustomApiServer};

    let mut module = RpcModule::new(());
    let FullDeps {
        client,
        pool,
        command_sink,
    } = deps;
    let manual_seal_enabled = command_sink.is_some();

    // Custom RPC methods for Paratensor
    module.merge(SubtensorCustom::new(client.clone()).into_rpc())?;

    module.merge(System::new(client.clone(), pool.clone()).into_rpc())?;
    module.merge(TransactionPayment::new(client.clone()).into_rpc())?;

    // Extend this RPC with a custom API by using the following syntax.
    // `YourRpcStruct` should have a reference to a client, which is needed
    // to call into the runtime.
    // `module.merge(YourRpcTrait::into_rpc(YourRpcStruct::new(ReferenceToClient, ...)))?;`

    if let Some(command_sink) = command_sink {
        module.merge(
            // We provide the rpc handler with the sending end of the channel to allow the rpc
            // send EngineCommands to the background block authorship task.
            ManualSeal::new(command_sink).into_rpc(),
        )?;
    }

    // Other methods provided by the caller
    for m in other_methods {
        module.merge(m.clone())?;
    }

    let core_module_count = 3usize;
    let consensus_module_count = other_methods.len();
    log::info!(
        target: LOG_TARGET,
        "rpc_module_ready build_ms={} core_modules={} consensus_modules={} manual_seal={} method_count={}",
        build_started_at.elapsed().as_millis(),
        core_module_count,
        consensus_module_count,
        manual_seal_enabled,
        module.method_names().count(),
    );

    Ok(module)
}

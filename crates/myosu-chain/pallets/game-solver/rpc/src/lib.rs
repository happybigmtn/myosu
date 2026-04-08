//! RPC interface for the custom Subtensor rpc methods

use codec::Encode;
use jsonrpsee::{
    core::RpcResult,
    proc_macros::rpc,
    types::{ErrorObjectOwned, error::ErrorObject},
};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block as BlockT;
use std::sync::Arc;
use subtensor_runtime_common::NetUid;

pub use subtensor_custom_rpc_runtime_api::NeuronInfoRuntimeApi;

#[rpc(client, server)]
pub trait SubtensorCustomApi<BlockHash> {
    #[method(name = "neuronInfo_getNeuronsLite")]
    fn get_neurons_lite(&self, netuid: NetUid, at: Option<BlockHash>) -> RpcResult<Vec<u8>>;
}

pub struct SubtensorCustom<C, P> {
    /// Shared reference to the client.
    client: Arc<C>,
    _marker: std::marker::PhantomData<P>,
}

impl<C, P> SubtensorCustom<C, P> {
    /// Creates a new instance of the TransactionPayment Rpc helper.
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}

/// Error type of this RPC api.
pub enum Error {
    /// The call to runtime failed.
    RuntimeError(String),
}

impl From<Error> for ErrorObjectOwned {
    fn from(e: Error) -> Self {
        match e {
            Error::RuntimeError(e) => ErrorObject::owned(1, e, None::<()>),
        }
    }
}

impl From<Error> for i32 {
    fn from(e: Error) -> i32 {
        match e {
            Error::RuntimeError(_) => 1,
        }
    }
}

impl<C, Block> SubtensorCustomApiServer<<Block as BlockT>::Hash> for SubtensorCustom<C, Block>
where
    Block: BlockT,
    C: ProvideRuntimeApi<Block> + HeaderBackend<Block> + Send + Sync + 'static,
    C::Api: NeuronInfoRuntimeApi<Block>,
{
    fn get_neurons_lite(
        &self,
        netuid: NetUid,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<u8>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);

        match api.get_neurons_lite(at, netuid) {
            Ok(result) => Ok(result.encode()),
            Err(e) => {
                Err(Error::RuntimeError(format!("Unable to get neurons lite info: {e:?}")).into())
            }
        }
    }
}

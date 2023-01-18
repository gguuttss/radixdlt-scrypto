use radix_engine_interface::api::api::{EngineApi, Invokable};
use radix_engine_interface::api::types::RENodeId;
use radix_engine_interface::constants::EPOCH_MANAGER;
use radix_engine_interface::data::{ScryptoCategorize, ScryptoDecode};
use radix_engine_interface::model::*;
use sbor::rust::fmt::Debug;

#[derive(Debug)]
pub struct Runtime {}

impl Runtime {
    pub fn sys_current_epoch<Y, E>(api: &mut Y) -> Result<u64, E>
    where
        Y: Invokable<EpochManagerGetCurrentEpochInvocation, E>,
        E: Debug + ScryptoCategorize + ScryptoDecode,
    {
        api.invoke(EpochManagerGetCurrentEpochInvocation {
            receiver: EPOCH_MANAGER,
        })
    }

    /// Generates a UUID.
    pub fn generate_uuid<Y, E>(api: &mut Y) -> Result<u128, E>
    where
        Y: EngineApi<E> + Invokable<TransactionRuntimeGenerateUuidInvocation, E>,
        E: Debug + ScryptoCategorize + ScryptoDecode,
    {
        let visible_node_ids = api.sys_get_visible_nodes()?;
        let node_id = visible_node_ids
            .into_iter()
            .find(|n| matches!(n, RENodeId::TransactionRuntime(..)))
            .expect("TransactionHash does not exist");

        api.invoke(TransactionRuntimeGenerateUuidInvocation {
            receiver: node_id.into(),
        })
    }
}

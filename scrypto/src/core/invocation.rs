use sbor::rust::string::ToString;
use sbor::rust::vec::Vec;
use sbor::*;

use crate::core::ScryptoActor;
use crate::engine::types::{BucketId, KeyValueStoreId, ProofId, RENodeId, VaultId};
use crate::prelude::ComponentAddress;
use crate::resource::ResourceAddress;

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub enum TypeName {
    Package,
    ResourceManager,
    TransactionProcessor,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub enum Receiver {
    Consumed(RENodeId),
    Scrypto(ScryptoActor),
    ComponentMetaRef(ComponentAddress),
    ResourceManagerRef(ResourceAddress),
    BucketRef(BucketId),
    ProofRef(ProofId),
    VaultRef(VaultId),
    SystemRef,
    WorktopRef,
    AuthZoneRef,
}

#[derive(Debug, Clone, Eq, Hash, PartialEq, TypeId, Encode, Decode)]
pub enum DataAddress {
    KeyValueEntry(KeyValueStoreId, Vec<u8>),
    ComponentInfo(ComponentAddress, bool),
    ComponentState(ComponentAddress),
}

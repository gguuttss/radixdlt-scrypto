use crate::types::*;
use scrypto::core::FunctionIdentifier;

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct REActor {
    pub function_identifier: FunctionIdentifier,
}

impl REActor {
    pub fn is_substate_readable(&self, substate_id: &SubstateId) -> bool {
        match &self.function_identifier {
            FunctionIdentifier::Function(FnIdentifier::Native(..))
            | FunctionIdentifier::Method(_, FnIdentifier::Native(..)) => true,
            FunctionIdentifier::Function(FnIdentifier::Scrypto { .. }) => match substate_id {
                SubstateId::KeyValueStoreEntry(..) => true,
                SubstateId::ComponentInfo(..) => true,
                _ => false,
            },
            FunctionIdentifier::Method(
                Receiver::Ref(RENodeId::Component(component_address)),
                FnIdentifier::Scrypto { .. },
            ) => match substate_id {
                SubstateId::KeyValueStoreEntry(..) => true,
                SubstateId::ComponentInfo(..) => true,
                SubstateId::ComponentState(addr) => addr.eq(component_address),
                _ => false,
            },
            _ => false,
        }
    }

    pub fn is_substate_writeable(&self, substate_id: &SubstateId) -> bool {
        match &self.function_identifier {
            FunctionIdentifier::Function(FnIdentifier::Native(..))
            | FunctionIdentifier::Method(_, FnIdentifier::Native(..)) => true,
            FunctionIdentifier::Function(FnIdentifier::Scrypto { .. }) => match substate_id {
                SubstateId::KeyValueStoreEntry(..) => true,
                _ => false,
            },
            FunctionIdentifier::Method(
                Receiver::Ref(RENodeId::Component(component_address)),
                FnIdentifier::Scrypto { .. },
            ) => match substate_id {
                SubstateId::KeyValueStoreEntry(..) => true,
                SubstateId::ComponentState(addr) => addr.eq(component_address),
                _ => false,
            },
            _ => false,
        }
    }
}

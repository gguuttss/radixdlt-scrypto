use crate::kernel::actor::Actor;
use crate::kernel::call_frame::Message;
use crate::kernel::kernel_api::{KernelInternalApi, KernelInvocation};
use crate::system::module::SystemModule;
use crate::system::system_callback::SystemConfig;
use crate::system::system_callback_api::SystemCallbackObject;
use crate::types::*;
use crate::{errors::RuntimeError, kernel::kernel_api::KernelApi};
use colored::Colorize;
use radix_engine_interface::api::field_api::LockFlags;
use radix_engine_interface::types::{LockHandle, NodeId, SubstateKey};
use sbor::rust::collections::BTreeMap;
use crate::kernel::kernel_callback_api::{CloseSubstateEvent, CreateNodeEvent, OpenSubstateEvent, ReadSubstateEvent};

#[derive(Debug, Clone)]
pub struct KernelTraceModule {}

#[macro_export]
macro_rules! log {
    ( $api: expr, $msg: expr $( , $arg:expr )* ) => {
        #[cfg(not(feature = "alloc"))]
        println!("{}[{}] {}", "    ".repeat($api.kernel_get_current_depth()), $api.kernel_get_current_depth(), sbor::rust::format!($msg, $( $arg ),*));
    };
}

#[allow(unused_variables)] // for no_std
impl<V: SystemCallbackObject> SystemModule<SystemConfig<V>> for KernelTraceModule {
    fn before_invoke<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        invocation: &KernelInvocation,
    ) -> Result<(), RuntimeError> {
        let message = format!(
            "Invoking: fn = {:?}, input size = {}",
            invocation.actor,
            invocation.len(),
        )
        .green();

        log!(api, "{}", message);
        Ok(())
    }

    fn before_push_frame<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        callee: &Actor,
        message: &mut Message,
        _args: &IndexedScryptoValue,
    ) -> Result<(), RuntimeError> {
        log!(api, "Sending nodes: {:?}", message.move_nodes);
        log!(api, "Sending refs: {:?}", message.copy_references);
        Ok(())
    }

    fn on_execution_finish<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        message: &Message,
    ) -> Result<(), RuntimeError> {
        log!(api, "Returning nodes: {:?}", message.move_nodes);
        log!(api, "Returning refs: {:?}", message.copy_references);
        Ok(())
    }

    fn after_invoke<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        output_size: usize,
    ) -> Result<(), RuntimeError> {
        log!(api, "Exiting: output size = {}", output_size);
        Ok(())
    }

    fn on_allocate_node_id<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        entity_type: EntityType,
    ) -> Result<(), RuntimeError> {
        log!(api, "Allocating node id: entity_type = {:?}", entity_type);
        Ok(())
    }

    fn on_create_node<Y: KernelInternalApi<SystemConfig<V>>>(
        api: &mut Y,
        event: &CreateNodeEvent
    ) -> Result<(), RuntimeError> {
        match event {
            CreateNodeEvent::Start(node_id, node_module_init) => {
                let mut module_substate_keys = BTreeMap::<&PartitionNumber, Vec<&SubstateKey>>::new();
                for (module_id, m) in *node_module_init {
                    for (substate_key, _) in m {
                        module_substate_keys
                            .entry(module_id)
                            .or_default()
                            .push(substate_key);
                    }
                }
                let message = format!(
                    "Creating node: id = {:?}, type = {:?}, substates = {:?}, module 0 = {:?}",
                    node_id,
                    node_id.entity_type(),
                    module_substate_keys,
                    node_module_init.get(&PartitionNumber(0))
                )
                    .red();
                log!(api, "{}", message);
            }
            _ => {}
        }

        Ok(())
    }

    fn before_drop_node<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        node_id: &NodeId,
    ) -> Result<(), RuntimeError> {
        log!(api, "Dropping node: id = {:?}", node_id);
        Ok(())
    }

    fn on_open_substate<Y: KernelInternalApi<SystemConfig<V>>>(
        api: &mut Y,
        event: &OpenSubstateEvent
    ) -> Result<(), RuntimeError> {
        match event {
            OpenSubstateEvent::Start {
                node_id, partition_num, substate_key, flags
            } => {
                log!(
                    api,
                    "Locking substate: node id = {:?}, partition_num = {:?}, substate_key = {:?}, flags = {:?}",
                    node_id,
                    partition_num,
                    substate_key,
                    flags
                );
            }
            OpenSubstateEvent::StoreAccess(..) => {}
            OpenSubstateEvent::End {
                handle, node_id, size
            } => {
                log!(
                    api,
                    "Substate locked: node id = {:?}, handle = {:?}",
                    node_id,
                    handle
                );
            }
        }

        Ok(())
    }

    fn on_read_substate<Y: KernelInternalApi<SystemConfig<V>>>(
        api: &mut Y,
        event: &ReadSubstateEvent,
    ) -> Result<(), RuntimeError> {
        match event {
            ReadSubstateEvent::End {
                handle,
                value
            } => {
                log!(
                    api,
                    "Reading substate: handle = {}, size = {}",
                    handle,
                    value.len()
                );
            }
        }

        Ok(())
    }

    fn on_write_substate<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        lock_handle: LockHandle,
        value_size: usize,
    ) -> Result<(), RuntimeError> {
        log!(
            api,
            "Writing substate: handle = {}, size = {}",
            lock_handle,
            value_size
        );
        Ok(())
    }

    fn on_close_substate<Y: KernelInternalApi<SystemConfig<V>>>(
        api: &mut Y,
        event: &CloseSubstateEvent,
    ) -> Result<(), RuntimeError> {
        match event {
            CloseSubstateEvent::End(lock_handle) => {
                log!(api, "Dropping lock: handle = {} ", lock_handle);
            }
            CloseSubstateEvent::StoreAccess(..) => {
            }
        }
        Ok(())
    }
}

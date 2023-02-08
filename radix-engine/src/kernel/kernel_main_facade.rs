use crate::errors::*;
use crate::kernel::kernel_api::{
    Invokable, KernelNodeApi, KernelSubstateApi, KernelWasmApi, LockFlags, LockInfo,
};
use crate::kernel::KernelModule;
use crate::kernel::*;
use crate::system::global::GlobalAddressSubstate;
use crate::system::node::{RENodeInit, RENodeModuleInit};
use crate::system::node_properties::VisibilityProperties;
use crate::system::node_substates::{SubstateRef, SubstateRefMut};
use crate::types::*;
use crate::wasm::WasmEngine;
use radix_engine_interface::api::types::*;

impl<'g, 's, W, M> KernelActorApi<RuntimeError> for Kernel<'g, 's, W, M>
where
    W: WasmEngine,
    M: KernelModule,
{
    fn fn_identifier(&mut self) -> Result<FnIdentifier, RuntimeError> {
        Ok(self.current_frame.actor.identifier.clone())
    }
}

impl<'g, 's, W, M> KernelNodeApi for Kernel<'g, 's, W, M>
where
    W: WasmEngine,
    M: KernelModule,
{
    fn get_visible_node_data(
        &mut self,
        node_id: RENodeId,
    ) -> Result<RENodeVisibilityOrigin, RuntimeError> {
        let visibility = self.current_frame.get_node_visibility(node_id)?;
        Ok(visibility)
    }

    fn drop_node(&mut self, node_id: RENodeId) -> Result<HeapRENode, RuntimeError> {
        M::pre_drop_node(self, &node_id)?;

        // Change to kernel mode
        let current_mode = self.execution_mode;
        self.execution_mode = ExecutionMode::Kernel;

        if !VisibilityProperties::check_drop_node_visibility(
            current_mode,
            &self.current_frame.actor,
            node_id,
        ) {
            return Err(RuntimeError::KernelError(
                KernelError::InvalidDropNodeVisibility {
                    mode: current_mode,
                    actor: self.current_frame.actor.clone(),
                    node_id,
                },
            ));
        }

        let node = self.drop_node_internal(node_id)?;

        // Restore current mode
        self.execution_mode = current_mode;

        M::post_drop_node(self)?;

        Ok(node)
    }

    fn allocate_node_id(&mut self, node_type: RENodeType) -> Result<RENodeId, RuntimeError> {
        // TODO: Add costing
        let node_id = self.id_allocator.allocate_node_id(node_type)?;

        Ok(node_id)
    }

    fn create_node(
        &mut self,
        node_id: RENodeId,
        re_node: RENodeInit,
        module_init: BTreeMap<NodeModuleId, RENodeModuleInit>,
    ) -> Result<(), RuntimeError> {
        M::pre_create_node(self, &node_id, &re_node, &module_init)?;

        // Change to kernel mode
        let current_mode = self.execution_mode;
        self.execution_mode = ExecutionMode::Kernel;

        if !VisibilityProperties::check_create_node_visibility(
            current_mode,
            &self.current_frame.actor,
            &re_node,
            &module_init,
        ) {
            return Err(RuntimeError::KernelError(
                KernelError::InvalidCreateNodeVisibility {
                    mode: current_mode,
                    actor: self.current_frame.actor.clone(),
                },
            ));
        }

        match (node_id, &re_node) {
            (
                RENodeId::Global(GlobalAddress::Package(..)),
                RENodeInit::Global(GlobalAddressSubstate::Package(..)),
            ) => {}
            (
                RENodeId::Global(GlobalAddress::Resource(..)),
                RENodeInit::Global(GlobalAddressSubstate::Resource(..)),
            ) => {}
            (
                RENodeId::Global(GlobalAddress::Component(..)),
                RENodeInit::Global(GlobalAddressSubstate::EpochManager(..)),
            ) => {}
            (
                RENodeId::Global(GlobalAddress::Component(..)),
                RENodeInit::Global(GlobalAddressSubstate::Clock(..)),
            ) => {}
            (
                RENodeId::Global(GlobalAddress::Component(..)),
                RENodeInit::Global(GlobalAddressSubstate::Validator(..)),
            ) => {}
            (
                RENodeId::Global(GlobalAddress::Component(..)),
                RENodeInit::Global(GlobalAddressSubstate::Identity(..)),
            ) => {}
            (
                RENodeId::Global(GlobalAddress::Component(..)),
                RENodeInit::Global(GlobalAddressSubstate::AccessController(..)),
            ) => {}
            (RENodeId::Global(..), RENodeInit::Global(GlobalAddressSubstate::Component(..))) => {}
            (RENodeId::Global(..), RENodeInit::Global(GlobalAddressSubstate::Account(..))) => {}
            (RENodeId::Bucket(..), RENodeInit::Bucket(..)) => {}
            (RENodeId::TransactionRuntime, RENodeInit::TransactionRuntime(..)) => {}
            (RENodeId::Proof(..), RENodeInit::Proof(..)) => {}
            (RENodeId::AuthZoneStack, RENodeInit::AuthZoneStack(..)) => {}
            (RENodeId::Vault(..), RENodeInit::Vault(..)) => {}
            (RENodeId::Component(..), RENodeInit::Component(..)) => {}
            (RENodeId::Worktop, RENodeInit::Worktop(..)) => {}
            (RENodeId::Logger, RENodeInit::Logger(..)) => {}
            (RENodeId::Package(..), RENodeInit::Package(..)) => {}
            (RENodeId::KeyValueStore(..), RENodeInit::KeyValueStore) => {}
            (RENodeId::NonFungibleStore(..), RENodeInit::NonFungibleStore(..)) => {}
            (RENodeId::ResourceManager(..), RENodeInit::ResourceManager(..)) => {}
            (RENodeId::EpochManager(..), RENodeInit::EpochManager(..)) => {}
            (RENodeId::Validator(..), RENodeInit::Validator(..)) => {}
            (RENodeId::Clock(..), RENodeInit::Clock(..)) => {}
            (RENodeId::Identity(..), RENodeInit::Identity(..)) => {}
            (RENodeId::AccessController(..), RENodeInit::AccessController(..)) => {}
            (RENodeId::Account(..), RENodeInit::Account(..)) => {}
            _ => return Err(RuntimeError::KernelError(KernelError::InvalidId(node_id))),
        }

        // TODO: For Scrypto components, check state against blueprint schema

        let push_to_store = match re_node {
            RENodeInit::Global(..) => true,
            _ => false,
        };

        self.id_allocator.take_node_id(node_id)?;
        self.current_frame.create_node(
            node_id,
            re_node,
            module_init,
            &mut self.heap,
            &mut self.track,
            push_to_store,
        )?;

        // Restore current mode
        self.execution_mode = current_mode;

        M::post_create_node(self, &node_id)?;

        Ok(())
    }

    fn get_module_state<T: KernelModuleState>(&mut self) -> &mut T {
        todo!()
    }
}

impl<'g, 's, W, M> KernelSubstateApi for Kernel<'g, 's, W, M>
where
    W: WasmEngine,
    M: KernelModule,
{
    fn lock_substate(
        &mut self,
        node_id: RENodeId,
        module_id: NodeModuleId,
        offset: SubstateOffset,
        flags: LockFlags,
    ) -> Result<LockHandle, RuntimeError> {
        M::on_lock_substate(self, &node_id, &module_id, &offset, &flags)?;

        // Change to kernel mode
        let current_mode = self.execution_mode;
        self.execution_mode = ExecutionMode::Kernel;

        // Deref
        let (node_id, derefed_lock) =
            if let Some((node_id, derefed_lock)) = self.node_offset_deref(node_id, &offset)? {
                (node_id, Some(derefed_lock))
            } else {
                (node_id, None)
            };

        // TODO: Check if valid offset for node_id

        // Authorization
        let actor = &self.current_frame.actor;
        if !VisibilityProperties::check_substate_visibility(
            current_mode,
            actor,
            node_id,
            offset.clone(),
            flags,
        ) {
            return Err(RuntimeError::KernelError(
                KernelError::InvalidSubstateVisibility {
                    mode: current_mode,
                    actor: actor.clone(),
                    node_id,
                    offset,
                    flags,
                },
            ));
        }

        let maybe_lock_handle = self.current_frame.acquire_lock(
            &mut self.heap,
            &mut self.track,
            node_id,
            module_id,
            offset.clone(),
            flags,
        );

        let lock_handle = match maybe_lock_handle {
            Ok(lock_handle) => lock_handle,
            Err(RuntimeError::KernelError(KernelError::TrackError(TrackError::NotFound(
                SubstateId(node_id, module_id, ref offset),
            )))) => {
                if self.try_virtualize(node_id, &offset)? {
                    self.current_frame.acquire_lock(
                        &mut self.heap,
                        &mut self.track,
                        node_id,
                        module_id,
                        offset.clone(),
                        flags,
                    )?
                } else {
                    return maybe_lock_handle;
                }
            }
            Err(err) => {
                match &err {
                    // TODO: This is a hack to allow for package imports to be visible
                    // TODO: Remove this once we are able to get this information through the Blueprint ABI
                    RuntimeError::CallFrameError(CallFrameError::RENodeNotVisible(
                        RENodeId::Global(GlobalAddress::Package(package_address)),
                    )) => {
                        let node_id = RENodeId::Global(GlobalAddress::Package(*package_address));
                        let module_id = NodeModuleId::SELF;
                        let offset = SubstateOffset::Global(GlobalOffset::Global);
                        self.track
                            .acquire_lock(
                                SubstateId(node_id, module_id, offset.clone()),
                                LockFlags::read_only(),
                            )
                            .map_err(|_| err.clone())?;
                        self.track
                            .release_lock(SubstateId(node_id, module_id, offset.clone()), false)
                            .map_err(|_| err)?;
                        self.current_frame
                            .add_stored_ref(node_id, RENodeVisibilityOrigin::Normal);
                        self.current_frame.acquire_lock(
                            &mut self.heap,
                            &mut self.track,
                            node_id,
                            module_id,
                            offset.clone(),
                            flags,
                        )?
                    }
                    _ => return Err(err),
                }
            }
        };

        if let Some(lock_handle) = derefed_lock {
            self.current_frame
                .drop_lock(&mut self.heap, &mut self.track, lock_handle)?;
        }

        // Restore current mode
        self.execution_mode = current_mode;

        Ok(lock_handle)
    }

    fn get_lock_info(&mut self, lock_handle: LockHandle) -> Result<LockInfo, RuntimeError> {
        self.current_frame.get_lock_info(lock_handle)
    }

    fn drop_lock(&mut self, lock_handle: LockHandle) -> Result<(), RuntimeError> {
        M::on_drop_lock(self, lock_handle)?;

        self.current_frame
            .drop_lock(&mut self.heap, &mut self.track, lock_handle)?;

        Ok(())
    }

    fn get_ref(&mut self, lock_handle: LockHandle) -> Result<SubstateRef, RuntimeError> {
        // A little hacky: this post sys call is called before the sys call happens due to
        // a mutable borrow conflict for substate ref.
        // Some modules (specifically: ExecutionTraceModule) require that all
        // pre/post callbacks are balanced.
        // TODO: Move post sys call to substate_ref drop() so that it's actually
        // after the sys call processing, not before.

        M::on_read_substate(
            self,
            lock_handle,
            0, //  TODO: pass the right size
        )?;

        let substate_ref =
            self.current_frame
                .get_ref(lock_handle, &mut self.heap, &mut self.track)?;

        Ok(substate_ref)
    }

    fn get_ref_mut(&mut self, lock_handle: LockHandle) -> Result<SubstateRefMut, RuntimeError> {
        // A little hacky: this post sys call is called before the sys call happens due to
        // a mutable borrow conflict for substate ref.
        // Some modules (specifically: ExecutionTraceModule) require that all
        // pre/post callbacks are balanced.
        // TODO: Move post sys call to substate_ref drop() so that it's actually
        // after the sys call processing, not before.

        M::on_write_substate(
            self,
            lock_handle,
            0, //  TODO: pass the right size
        )?;

        let substate_ref_mut =
            self.current_frame
                .get_ref_mut(lock_handle, &mut self.heap, &mut self.track)?;

        Ok(substate_ref_mut)
    }
}

impl<'g, 's, W, M> KernelWasmApi<W> for Kernel<'g, 's, W, M>
where
    W: WasmEngine,
    M: KernelModule,
{
    fn scrypto_interpreter(&mut self) -> &ScryptoInterpreter<W> {
        self.scrypto_interpreter
    }

    fn emit_wasm_instantiation_event(&mut self, code: &[u8]) -> Result<(), RuntimeError> {
        M::on_wasm_instantiation(self, code)?;

        Ok(())
    }
}

impl<'g, 's, W, M, N> Invokable<N, RuntimeError> for Kernel<'g, 's, W, M>
where
    W: WasmEngine,
    M: KernelModule,
    N: ExecutableInvocation,
{
    fn invoke(&mut self, invocation: N) -> Result<<N as Invocation>::Output, RuntimeError> {
        M::pre_kernel_invoke(
            self,
            &invocation.fn_identifier(),
            0, // TODO: Pass the right size
        )?;

        // Change to kernel mode
        let saved_mode = self.execution_mode;

        self.execution_mode = ExecutionMode::Resolver;
        let (actor, call_frame_update, executor) = invocation.resolve(self)?;

        self.execution_mode = ExecutionMode::Kernel;
        let rtn = self.invoke_internal(executor, actor, call_frame_update)?;

        // Restore previous mode
        self.execution_mode = saved_mode;

        M::post_kernel_invoke(
            self, 0, // TODO: Pass the right size
        )?;

        Ok(rtn)
    }
}

impl<'g, 's, W, M> KernelInvokeApi<RuntimeError> for Kernel<'g, 's, W, M>
where
    W: WasmEngine,
    M: KernelModule,
{
}

impl<'g, 's, W, M> KernelApi<W, RuntimeError> for Kernel<'g, 's, W, M>
where
    W: WasmEngine,
    M: KernelModule,
{
}

use radix_common::constants::CONSENSUS_MANAGER;
use radix_common::data::scrypto::*;
use radix_common::time::*;
use radix_common::traits::ScryptoEvent;
use radix_common::types::{NodeId, PackageAddress};
use radix_engine_interface::api::actor_api::EventFlags;
use radix_engine_interface::api::*;
use radix_engine_interface::blueprints::consensus_manager::*;
use radix_engine_interface::blueprints::resource::{
    AccessRule, AuthZoneAssertAccessRuleInput, AUTH_ZONE_ASSERT_ACCESS_RULE_IDENT,
};
use radix_engine_interface::types::Epoch;
use sbor::rust::prelude::*;

#[derive(Debug)]
pub struct Runtime {}

impl Runtime {
    pub fn emit_event<T: ScryptoEncode + ScryptoDescribe + ScryptoEvent, Y, E>(
        api: &mut Y,
        event: T,
    ) -> Result<(), E>
    where
        Y: SystemApi<E>,
        E: Debug + ScryptoCategorize + ScryptoDecode,
    {
        api.actor_emit_event(
            T::EVENT_NAME.to_string(),
            scrypto_encode_to_payload(&event)
                .unwrap()
                .into_unvalidated(),
            EventFlags::empty(),
        )
    }

    pub fn emit_event_no_revert<T: ScryptoEncode + ScryptoDescribe + ScryptoEvent, Y, E>(
        api: &mut Y,
        event: T,
    ) -> Result<(), E>
    where
        Y: SystemApi<E>,
        E: Debug + ScryptoCategorize + ScryptoDecode,
    {
        api.actor_emit_event(
            T::EVENT_NAME.to_string(),
            scrypto_encode_to_payload(&event)
                .unwrap()
                .into_unvalidated(),
            EventFlags::FORCE_WRITE,
        )
    }

    pub fn current_epoch<Y, E>(api: &mut Y) -> Result<Epoch, E>
    where
        Y: SystemObjectApi<E>,
        E: Debug + ScryptoCategorize + ScryptoDecode,
    {
        let rtn = api.call_method(
            CONSENSUS_MANAGER.as_node_id(),
            CONSENSUS_MANAGER_GET_CURRENT_EPOCH_IDENT,
            scrypto_encode(&ConsensusManagerGetCurrentEpochInput).unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    pub fn current_time<Y, E>(api: &mut Y, precision: TimePrecision) -> Result<Instant, E>
    where
        Y: SystemObjectApi<E>,
        E: Debug + ScryptoCategorize + ScryptoDecode,
    {
        let rtn = api.call_method(
            CONSENSUS_MANAGER.as_node_id(),
            CONSENSUS_MANAGER_GET_CURRENT_TIME_IDENT,
            scrypto_encode(&ConsensusManagerGetCurrentTimeInputV2 { precision }).unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    pub fn compare_against_current_time<Y, E>(
        api: &mut Y,
        instant: Instant,
        precision: TimePrecision,
        operator: TimeComparisonOperator,
    ) -> Result<bool, E>
    where
        Y: SystemObjectApi<E>,
        E: Debug + ScryptoCategorize + ScryptoDecode,
    {
        let rtn = api.call_method(
            CONSENSUS_MANAGER.as_node_id(),
            CONSENSUS_MANAGER_COMPARE_CURRENT_TIME_IDENT,
            scrypto_encode(&ConsensusManagerCompareCurrentTimeInputV2 {
                precision,
                instant,
                operator,
            })
            .unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    pub fn generate_ruid<Y, E>(api: &mut Y) -> Result<[u8; 32], E>
    where
        Y: SystemApi<E>,
        E: Debug + ScryptoCategorize + ScryptoDecode,
    {
        api.generate_ruid()
    }

    pub fn assert_access_rule<Y, E>(rule: AccessRule, api: &mut Y) -> Result<(), E>
    where
        Y: SystemApi<E>,
        E: Debug + ScryptoCategorize + ScryptoDecode,
    {
        let auth_zone = api.actor_get_node_id(ACTOR_REF_AUTH_ZONE)?;
        let _rtn = api.call_method(
            &auth_zone,
            AUTH_ZONE_ASSERT_ACCESS_RULE_IDENT,
            scrypto_encode(&AuthZoneAssertAccessRuleInput { rule }).unwrap(),
        )?;

        Ok(())
    }

    pub fn get_node_id<Y, E>(api: &mut Y) -> Result<NodeId, E>
    where
        Y: SystemApi<E>,
        E: Debug + ScryptoCategorize + ScryptoDecode,
    {
        api.actor_get_node_id(ACTOR_REF_SELF)
    }

    pub fn package_address<Y, E>(api: &mut Y) -> Result<PackageAddress, E>
    where
        Y: SystemApi<E>,
        E: Debug,
    {
        api.actor_get_blueprint_id().map(|x| x.package_address)
    }
}

use crate::api::node_modules::auth::*;
use crate::api::node_modules::metadata::*;
use crate::api::node_modules::royalty::ComponentSetRoyaltyConfigInvocation;
use crate::api::package::PackageAddress;
use crate::api::package::*;
use crate::api::types::*;
use crate::data::scrypto_encode;
use crate::data::ScryptoValue;
use crate::*;
use radix_engine_interface::api::node_modules::royalty::ComponentClaimRoyaltyInvocation;
use sbor::rust::collections::HashSet;
use sbor::rust::fmt::Debug;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use transaction_data::*;

// TODO: Remove enum
#[derive(Debug, ScryptoSbor)]
pub enum CallTableInvocation {
    Native(NativeInvocation),
    Scrypto(ScryptoInvocation),
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct ScryptoInvocation {
    pub package_address: PackageAddress,
    pub blueprint_name: String,
    pub fn_name: String,
    pub receiver: Option<ScryptoReceiver>,
    pub args: Vec<u8>,
}

impl Invocation for ScryptoInvocation {
    type Output = ScryptoValue;

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Scrypto(ScryptoFnIdentifier {
            package_address: self.package_address,
            blueprint_name: self.blueprint_name.clone(),
            ident: self.fn_name.clone(),
        })
    }
}

impl Into<CallTableInvocation> for ScryptoInvocation {
    fn into(self) -> CallTableInvocation {
        CallTableInvocation::Scrypto(self)
    }
}

#[derive(
    Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestCategorize, ManifestEncode, ManifestDecode,
)]
pub enum NativeInvocation {
    AccessRulesChain(AccessRulesChainInvocation),
    Metadata(MetadataInvocation),
    Package(PackageInvocation),
    Component(ComponentInvocation),
}

impl Into<CallTableInvocation> for NativeInvocation {
    fn into(self) -> CallTableInvocation {
        CallTableInvocation::Native(self)
    }
}

#[derive(
    Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestCategorize, ManifestEncode, ManifestDecode,
)]
pub enum AccessRulesChainInvocation {
    AddAccessCheck(AccessRulesAddAccessCheckInvocation),
    SetMethodAccessRule(AccessRulesSetMethodAccessRuleInvocation),
    SetMethodMutability(AccessRulesSetMethodMutabilityInvocation),
    SetGroupAccessRule(AccessRulesSetGroupAccessRuleInvocation),
    SetGroupMutability(AccessRulesSetGroupMutabilityInvocation),
    GetLength(AccessRulesGetLengthInvocation),
}

#[derive(
    Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestCategorize, ManifestEncode, ManifestDecode,
)]
pub enum MetadataInvocation {
    Set(MetadataSetInvocation),
    Get(MetadataGetInvocation),
}

#[derive(
    Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestCategorize, ManifestEncode, ManifestDecode,
)]
pub enum ComponentInvocation {
    SetRoyaltyConfig(ComponentSetRoyaltyConfigInvocation),
    ClaimRoyalty(ComponentClaimRoyaltyInvocation),
}

#[derive(
    Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestCategorize, ManifestEncode, ManifestDecode,
)]
pub enum PackageInvocation {
    Publish(PackagePublishInvocation),
    PublishNative(PackagePublishNativeInvocation),
    SetRoyaltyConfig(PackageSetRoyaltyConfigInvocation),
    ClaimRoyalty(PackageClaimRoyaltyInvocation),
}

impl NativeInvocation {
    pub fn refs(&self) -> HashSet<RENodeId> {
        let mut refs = HashSet::new();
        match self {
            NativeInvocation::Component(invocation) => match invocation {
                ComponentInvocation::SetRoyaltyConfig(invocation) => {
                    refs.insert(invocation.receiver);
                }
                ComponentInvocation::ClaimRoyalty(invocation) => {
                    refs.insert(invocation.receiver);
                }
            },
            NativeInvocation::Package(package_method) => match package_method {
                PackageInvocation::Publish(..) => {}
                PackageInvocation::PublishNative(..) => {}
                PackageInvocation::SetRoyaltyConfig(invocation) => {
                    refs.insert(RENodeId::Global(Address::Package(invocation.receiver)));
                }
                PackageInvocation::ClaimRoyalty(invocation) => {
                    refs.insert(RENodeId::Global(Address::Package(invocation.receiver)));
                }
            },
            NativeInvocation::AccessRulesChain(access_rules_method) => match access_rules_method {
                AccessRulesChainInvocation::AddAccessCheck(invocation) => {
                    refs.insert(invocation.receiver);
                }
                AccessRulesChainInvocation::SetMethodAccessRule(invocation) => {
                    refs.insert(invocation.receiver);
                }
                AccessRulesChainInvocation::SetMethodMutability(invocation) => {
                    refs.insert(invocation.receiver);
                }
                AccessRulesChainInvocation::SetGroupAccessRule(invocation) => {
                    refs.insert(invocation.receiver);
                }
                AccessRulesChainInvocation::SetGroupMutability(invocation) => {
                    refs.insert(invocation.receiver);
                }
                AccessRulesChainInvocation::GetLength(invocation) => {
                    refs.insert(invocation.receiver);
                }
            },
            NativeInvocation::Metadata(metadata_method) => match metadata_method {
                MetadataInvocation::Set(invocation) => {
                    refs.insert(invocation.receiver);
                }
                MetadataInvocation::Get(invocation) => {
                    refs.insert(invocation.receiver);
                }
            },
        }

        refs
    }
}

fn get_native_fn<T: SerializableInvocation>(_: &T) -> NativeFn {
    T::native_fn()
}

impl NativeInvocation {
    pub fn flatten(&self) -> (NativeFn, Vec<u8>) {
        let (native_fn, encoding) = match self {
            NativeInvocation::AccessRulesChain(i) => match i {
                AccessRulesChainInvocation::AddAccessCheck(i) => {
                    (get_native_fn(i), scrypto_encode(i))
                }
                AccessRulesChainInvocation::SetMethodAccessRule(i) => {
                    (get_native_fn(i), scrypto_encode(i))
                }
                AccessRulesChainInvocation::SetMethodMutability(i) => {
                    (get_native_fn(i), scrypto_encode(i))
                }
                AccessRulesChainInvocation::SetGroupAccessRule(i) => {
                    (get_native_fn(i), scrypto_encode(i))
                }
                AccessRulesChainInvocation::SetGroupMutability(i) => {
                    (get_native_fn(i), scrypto_encode(i))
                }
                AccessRulesChainInvocation::GetLength(i) => (get_native_fn(i), scrypto_encode(i)),
            },
            NativeInvocation::Metadata(i) => match i {
                MetadataInvocation::Set(i) => (get_native_fn(i), scrypto_encode(i)),
                MetadataInvocation::Get(i) => (get_native_fn(i), scrypto_encode(i)),
            },
            NativeInvocation::Package(i) => match i {
                PackageInvocation::Publish(i) => (get_native_fn(i), scrypto_encode(i)),
                PackageInvocation::PublishNative(i) => (get_native_fn(i), scrypto_encode(i)),
                PackageInvocation::SetRoyaltyConfig(i) => (get_native_fn(i), scrypto_encode(i)),
                PackageInvocation::ClaimRoyalty(i) => (get_native_fn(i), scrypto_encode(i)),
            },
            NativeInvocation::Component(i) => match i {
                ComponentInvocation::SetRoyaltyConfig(i) => (get_native_fn(i), scrypto_encode(i)),
                ComponentInvocation::ClaimRoyalty(i) => (get_native_fn(i), scrypto_encode(i)),
            },
        };

        (
            native_fn,
            encoding.expect("Failed to encode native invocation"),
        )
    }
}

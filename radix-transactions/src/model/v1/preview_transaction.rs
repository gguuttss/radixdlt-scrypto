use crate::internal_prelude::*;
use radix_common::constants::AuthAddresses;

#[derive(Debug, Clone, Sbor, PartialEq, Eq, Default)]
pub struct PreviewFlags {
    pub use_free_credit: bool,
    pub assume_all_signature_proofs: bool,
    pub skip_epoch_check: bool,
    pub disable_auth: bool,
}

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor)]
pub struct PreviewIntentV1 {
    pub intent: IntentV1,
    pub signer_public_keys: Vec<PublicKey>,
    pub flags: PreviewFlags,
}

pub struct ValidatedPreviewIntent {
    pub intent: PreparedIntentV1,
    pub encoded_instructions: Vec<u8>,
    pub signer_public_keys: Vec<PublicKey>,
    pub flags: PreviewFlags,
}

impl ValidatedPreviewIntent {
    pub fn get_executable<'a>(&'a self) -> Executable<'a> {
        let intent = &self.intent;
        let flags = &self.flags;

        let mut virtual_resources = BTreeSet::new();
        if self.flags.assume_all_signature_proofs {
            virtual_resources.insert(SECP256K1_SIGNATURE_RESOURCE);
            virtual_resources.insert(ED25519_SIGNATURE_RESOURCE);
        }

        let header = &intent.header.inner;
        let fee_payment = TransactionCostingParameters {
            tip_percentage: header.tip_percentage,
            free_credit_in_xrd: if self.flags.use_free_credit {
                Decimal::try_from(PREVIEW_CREDIT_IN_XRD).unwrap()
            } else {
                Decimal::ZERO
            },
            abort_when_loan_repaid: false,
        };

        let mut initial_proofs = AuthAddresses::signer_set(&self.signer_public_keys);
        if header.notary_is_signatory {
            initial_proofs.insert(NonFungibleGlobalId::from_public_key(
                &header.notary_public_key,
            ));
        }

        let intent_hash = intent.intent_hash();

        Executable::new(
            &self.encoded_instructions,
            &intent.instructions.references,
            &intent.blobs.blobs_by_hash,
            ExecutionContext {
                intent_hash: if flags.skip_epoch_check {
                    TransactionIntentHash::NotToCheck {
                        intent_hash: intent_hash.into_hash(),
                    }
                } else {
                    TransactionIntentHash::ToCheck {
                        intent_hash: intent_hash.into_hash(),
                        expiry_epoch: intent.header.inner.end_epoch_exclusive,
                    }
                },
                epoch_range: if flags.skip_epoch_check {
                    None
                } else {
                    Some(EpochRange {
                        start_epoch_inclusive: intent.header.inner.start_epoch_inclusive,
                        end_epoch_exclusive: intent.header.inner.end_epoch_exclusive,
                    })
                },
                payload_size: self.intent.summary.effective_length,
                num_of_signature_validations: 0, // Accounted for by tests in `common_transformation_costs.rs`.
                auth_zone_params: AuthZoneParams::single_thread(initial_proofs, virtual_resources),
                costing_parameters: fee_payment,
                pre_allocated_addresses: vec![],
            },
            false,
        )
    }
}

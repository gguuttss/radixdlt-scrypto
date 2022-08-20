use sbor::rust::vec::Vec;
use sbor::*;
use scrypto::buffer::{scrypto_decode, scrypto_encode};
use scrypto::core::NetworkDefinition;
use scrypto::crypto::{hash, EcdsaPublicKey, EcdsaSignature, Hash};

use crate::manifest::{compile, CompileError};
use crate::model::Instruction;

// TODO: add versioning of transaction schema

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct TransactionHeader {
    pub version: u8,
    pub network_id: u8,
    pub start_epoch_inclusive: u64,
    pub end_epoch_exclusive: u64,
    pub nonce: u64,
    pub notary_public_key: EcdsaPublicKey,
    pub notary_as_signatory: bool,
    pub cost_unit_limit: u32,
    pub tip_percentage: u32,
}

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct TransactionManifest {
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct TransactionIntent {
    pub header: TransactionHeader,
    pub manifest: TransactionManifest,
}

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct SignedTransactionIntent {
    pub intent: TransactionIntent,
    pub intent_signatures: Vec<(EcdsaPublicKey, EcdsaSignature)>,
}

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct NotarizedTransaction {
    pub signed_intent: SignedTransactionIntent,
    pub notary_signature: EcdsaSignature,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntentCreationError {
    CompileErr(CompileError),
    ConfigErr(IntentConfigError),
}

impl From<CompileError> for IntentCreationError {
    fn from(compile_error: CompileError) -> Self {
        IntentCreationError::CompileErr(compile_error)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntentConfigError {
    MismatchedNetwork { expected: u8, actual: u8 },
}

impl TransactionIntent {
    pub fn new(
        network: &NetworkDefinition,
        header: TransactionHeader,
        manifest: &str,
    ) -> Result<Self, IntentCreationError> {
        if network.id != header.network_id {
            return Err(IntentCreationError::ConfigErr(
                IntentConfigError::MismatchedNetwork {
                    expected: network.id,
                    actual: header.network_id,
                },
            ));
        }
        Ok(Self {
            header,
            manifest: compile(manifest, &network)?,
        })
    }

    pub fn from_slice(slice: &[u8]) -> Result<Self, DecodeError> {
        scrypto_decode(slice)
    }

    pub fn hash(&self) -> Hash {
        hash(self.to_bytes())
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        scrypto_encode(self)
    }
}

impl SignedTransactionIntent {
    pub fn from_slice(slice: &[u8]) -> Result<Self, DecodeError> {
        scrypto_decode(slice)
    }

    pub fn hash(&self) -> Hash {
        hash(self.to_bytes())
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        scrypto_encode(self)
    }
}

impl NotarizedTransaction {
    pub fn from_slice(slice: &[u8]) -> Result<Self, DecodeError> {
        scrypto_decode(slice)
    }

    pub fn hash(&self) -> Hash {
        hash(self.to_bytes())
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        scrypto_encode(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signing::*;
    use scrypto::buffer::scrypto_encode;
    use scrypto::core::Network;

    #[test]
    fn construct_sign_and_notarize() {
        // create a key pair
        let sk1 = EcdsaPrivateKey::from_u64(1).unwrap();
        let sk2 = EcdsaPrivateKey::from_u64(2).unwrap();
        let sk_notary = EcdsaPrivateKey::from_u64(3).unwrap();

        // construct
        let intent = TransactionIntent::new(
            &Network::LocalSimulator.get_definition(),
            TransactionHeader {
                version: 1,
                network_id: Network::LocalSimulator.get_id(),
                start_epoch_inclusive: 0,
                end_epoch_exclusive: 100,
                nonce: 5,
                notary_public_key: sk_notary.public_key(),
                notary_as_signatory: false,
                cost_unit_limit: 1_000_000,
                tip_percentage: 5,
            },
            "CLEAR_AUTH_ZONE;",
        )
        .unwrap();

        // sign
        let signature1 = (sk1.public_key(), sk1.sign(&intent.to_bytes()));
        let signature2 = (sk2.public_key(), sk2.sign(&intent.to_bytes()));
        let signed_intent = SignedTransactionIntent {
            intent,
            intent_signatures: vec![signature1, signature2],
        };

        // notarize
        let signature3 = sk_notary.sign(&signed_intent.to_bytes());
        let transaction = NotarizedTransaction {
            signed_intent,
            notary_signature: signature3,
        };

        assert_eq!(
            "e2ad3c72620ef90cd633df835e9632669b004e40e94dd316c10cd9d285ec9471",
            transaction.signed_intent.intent.hash().to_string()
        );
        assert_eq!(
            "9dfda4ed6ebfe032e6e577f75611a3e1c571b56edab42ec68257ba14e71c1e09",
            transaction.signed_intent.hash().to_string()
        );
        assert_eq!(
            "e62bf0a7d63ba2ef9d015fb0a0febd82cc99eae02594b279b74374b144436442",
            transaction.hash().to_string()
        );
        assert_eq!("10020000001002000000100200000010090000000701110f000000496e7465726e616c546573746e6574000000000a00000000000000000a64000000000000000a0500000000000000912100000002f9308a019258c31049344f85f89d5229b531c845836f99b08601f113bce036f901000940420f00090500000010010000003011010000000d000000436c656172417574685a6f6e65000000003021020000000200000091210000000279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798924000000028ad4c978705c7857bd2b71498d3bd8fcefd825f4b09af21909381a09204ce3a792a7217bdbe51c4cf54532954aeb15831cabe5cf394d2df321440796ceec65302000000912100000002c6047f9441ed7d6d3045406e95c07cd85c778e4b8cef3ca7abac09b95c709ee59240000000bd02fd19ebb958ab1f500790b70520c0fe79e67882d492c11289d41e3c9c16214f09f3fa59d5d2ffcd797a0fefa3993e607d01afb2f9e47ff6792e9f87064eab924000000023086c21e536ef12d4b97b051f29cbf8b2ad07b77927cccd142f6b4dc316f65c7b1fb7ae0273e0cbab14214d43a2635aabfdb2228128b7c60404416901d1c2e6", hex::encode(scrypto_encode(&transaction)));
    }
}

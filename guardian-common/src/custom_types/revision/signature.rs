use ethaddr::Address;

use super::super::{hash::Hash, public_key::PublicKey, signature::Signature};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
/// Represents a sp256k1 public key that has been used to sign an Aqua-Chain
pub struct RevisionSignature {
    pub signature: Signature,
    pub public_key: PublicKey,
    pub signature_hash: Hash,
    // todo: remove with v1.2
    pub wallet_address: Address,
}

#[test]
fn wallet_address() {
    let addr: Address = serde_json::from_str("\"0x13Ddb9f9dEDE0903Ed6F40D4FB273632d19CaED1\"").unwrap();
    dbg!(addr);
}
use super::super::{hash::Hash, tx_hash::TxHash};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
/// Contains the information stored on the blockchain
pub struct RevisionWitness {
    pub domain_snapshot_genesis_hash: Hash,
    pub merkle_root: Hash,
    pub witness_network: String,
    pub witness_event_transaction_hash: TxHash,
    pub witness_hash: Hash,
    pub structured_merkle_proof: Vec<MerkleNode>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MerkleNode {
    pub left_leaf: Hash,
    pub right_leaf: Hash,
}

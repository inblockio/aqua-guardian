use super::*;
use std::collections::HashSet;

pub fn witness_hash(
    domain_snapshot_genesis_hash: &Hash,
    merkle_root: &Hash,
    witness_network: &str,
    witness_event_transaction_hash: &TxHash,
) -> Hash {
    // 2.a create hasher {w}
    let mut w = crypt::Hasher::default();
    // 2.b add rev.witness.domain_snapshot_genesis_hash to hasher {w}
    w.update(domain_snapshot_genesis_hash.to_stackstr());
    // 2.c add rev.witness.merkle_root to hasher {w}
    w.update(merkle_root.to_stackstr());
    // 2.d add rev.witness.witness_network to hasher {w}
    w.update(witness_network);
    // 2.e add rev.witness.witness_event_transaction_hash to hasher {w}
    w.update(witness_event_transaction_hash.to_stackstr());
    Hash::from(w.finalize())
}

/// [c] witness_hash integrity
///
/// IMPORTANT: what does this verify?
/// - the merkle_tree forms a merkle_tree
/// - the merkle_tree has merkle_root as its root node
/// - the merkle_tree contains this rev's verification_hash
/// - the witness_hash describes this merkle_tree publication
///
/// prerequisites: rev [trusted verification_hash]
pub(super) fn only_witness_hash_integrity(rev: &Revision, prev: Option<&Revision>) -> flagset::FlagSet<RevisionIntegrity> {
    use RevisionIntegrity::*;
    let mut integrity = flagset::FlagSet::default();

    let Some(witness) = &rev.witness else {
        return flagset::FlagSet::from(NoWitness);
    };
    let Some(prev_hash) = &prev.map(|a|a.metadata.verification_hash) else {
        return flagset::FlagSet::from(NoPrevRevision);
    };

    // 1 merkle_tree
    // 1.a create set {a} "free leafs"
    let mut a = HashSet::<crypt::Hash>::new();
    // 1.b create set {b} "free roots"
    let mut b = HashSet::<crypt::Hash>::new();
    // 1.c create set {c} "matched"
    let mut c = HashSet::<crypt::Hash>::new();
    // 1.d add merkle_root to set {b}
    a.insert(*witness.merkle_root);
    // 1.e for each node in the merkle_tree
    for node in &witness.structured_merkle_proof {
        // 1.e.i    node.left_leaf is not in set {c} or set {a}
        if c.contains(&*node.left_leaf) || a.contains(&*node.left_leaf) {
            dbg!(&node.left_leaf);
            integrity |= DuplicateMerkleLeaf;
        }
        // 1.e.ii   node.right_leaf is not in set {c} or set {a}
        if c.contains(&*node.right_leaf) || a.contains(&*node.right_leaf) {
            dbg!(&node.right_leaf);
            integrity |= DuplicateMerkleLeaf;
        }
        // 1.e.iii  remove node.left_leaf from set {b} and insert into set {c} *or* insert into set {a}
        if b.remove(&*node.left_leaf) {
            c.insert(*node.left_leaf);
        } else {
            a.insert(*node.left_leaf);
        }
        // 1.e.iv   remove node.right_leaf from set {b} and insert into set {c} *or* insert into set {a}
        if b.remove(&*node.right_leaf) {
            c.insert(*node.right_leaf);
        } else {
            a.insert(*node.right_leaf);
        }
        // 1.e.v    create hasher {p}
        let mut p = crypt::Hasher::default();
        // 1.e.vi   add node.left_leaf to hasher {p}
        p.update(node.left_leaf.to_stackstr());
        // 1.e.vii  add node.right_leaf to hasher {p}
        p.update(node.right_leaf.to_stackstr());
        // 1.e.viii output of hasher {p} is not in set {c} or set {b}
        let p_output = p.finalize();
        if c.contains(&p_output) || b.contains(&p_output) {
            // dbg!(hash::Hash::from(p_output));
            integrity |= DuplicateMerkleLeaf;
        }
        // 1.e.ix   remove output of hasher {p} from set {a} and insert into set {c} *or* insert into set {b}
        if a.remove(&p_output) {
            c.insert(p_output);
        } else {
            b.insert(p_output);
        }
    }
    // 1.f verification_hash is in set {a}
    if !a.contains(&**prev_hash) {
        integrity |= VerificationHashNotInMerkleTree;
    }
    // 1.g set {b} is empty
    if !b.is_empty() {
        integrity |= MerkleTreeIncomplete;
    }
    // 1.h merkle_root is in set {c}
    if !c.contains(&*witness.merkle_root) {
        integrity |= MerkleTreeIncomplete;
    }
    // 2 witness_hash
    // 2.f output of hasher {w} equals rev.witness.witness_hash
    if witness_hash(
        &witness.domain_snapshot_genesis_hash,
        &witness.merkle_root,
        &witness.witness_network,
        &witness.witness_event_transaction_hash,
    ) != witness.witness_hash
    {
        integrity |= WitnessHashNotMatching;
    }

    integrity
}

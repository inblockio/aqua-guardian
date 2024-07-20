use guardian_common::prelude::*;

#[cfg(test)]
mod tests;

pub trait Verify: AquaHashAttached {
    fn verify(&self) -> bool {
        self.read_hash() == self.calculate_hash()
    }
}
pub trait AquaHashAttached: AquaHashable {
    fn read_hash(&self) -> Hash;
}
pub trait AquaHashable {
    fn calculate_hash(&self) -> Hash;
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct RevisionMetadata {
    pub metadata_hash: Hash,

    pub domain_id: String,
    pub timestamp: Timestamp,
}
pub fn rev_v1_1_to_rev_v1_2(
    rev: &guardian_common::custom_types::Revision,
    prev: Option<&guardian_common::custom_types::Revision>,
    merge: Option<&guardian_common::custom_types::Revision>,
) -> Revision {
    Revision {
        verification_hash: rev.metadata.verification_hash,
        content: rev.content.clone(),
        metadata: RevisionMetadata {
            metadata_hash: rev.metadata.metadata_hash,
            domain_id: rev.metadata.domain_id.clone(),
            timestamp: rev.metadata.time_stamp.clone(),
        },
        prev: prev.map(|a| RevisionReference {
            reference_hash: Hash::default(),
            verification_hash: a.metadata.verification_hash,
            signature: rev.signature.clone(),
            witness: rev.witness.clone(),
        }),
        merge: merge.map(|a| RevisionReference {
            reference_hash: Hash::default(),
            verification_hash: a.metadata.verification_hash,
            signature: None,
            witness: None,
        }),
    }
}

pub fn prev_v1_1_to_ref_v1_2(prev: &guardian_common::custom_types::Revision) -> RevisionReference {
    RevisionReference {
        reference_hash: Hash::default(),
        verification_hash: prev.metadata.verification_hash,
        signature: prev.signature.clone(),
        witness: prev.witness.clone(),
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct Revision {
    pub verification_hash: Hash,

    pub content: RevisionContent,
    pub metadata: RevisionMetadata,
    pub prev: Option<RevisionReference>,
    pub merge: Option<RevisionReference>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct RevisionReference {
    pub reference_hash: Hash,

    pub verification_hash: Hash,
    pub signature: Option<RevisionSignature>,
    pub witness: Option<RevisionWitness>,
}

macro_rules! hash_attached {
    ($($DataType:ident :: $field:ident);* $(;)?) => { $(
        impl AquaHashAttached for $DataType {
            fn read_hash(&self) -> Hash {
                self.$field
            }
        }
    )* };
}
hash_attached! {
    Revision::verification_hash;
    RevisionContent::content_hash;
    RevisionMetadata::metadata_hash;
    RevisionSignature::signature_hash;
    RevisionWitness::witness_hash;
    RevisionReference::reference_hash;
}

impl AquaHashable for Revision {
    fn calculate_hash(&self) -> Hash {
        let mut hasher = crypt::Hasher::default();
        hasher.update(self.content.read_hash().to_stackstr());
        hasher.update(self.metadata.read_hash().to_stackstr());
        if let Some(ref prev) = self.prev {
            hasher.update(prev.read_hash().to_stackstr());
        }
        if let Some(ref merge) = self.merge {
            hasher.update(merge.read_hash().to_stackstr());
        }
        Hash::from(hasher.finalize())
    }
}
impl Verify for Revision {
    fn verify(&self) -> bool {
        if let Some(prev) = &self.prev {
            if !prev.verify() {
                return false;
            }
        }
        if let Some(merge) = &self.merge {
            if !merge.verify() {
                return false;
            }
        }
        self.read_hash() == self.calculate_hash() && self.metadata.verify() && self.content.verify()
    }
}

impl AquaHashable for RevisionContent {
    fn calculate_hash(&self) -> Hash {
        let mut hasher = crypt::Hasher::default();
        for value in self.content.values() {
            hasher.update(value);
        }
        Hash::from(hasher.finalize())
    }
}
impl Verify for RevisionContent {
    fn verify(&self) -> bool {
        match (&self.file, self.content.get("file_hash")) {
            (None, None) => (),
            (Some(file), Some(file_hash)) => {
                let Ok(hash): Result<Hash, _> = file_hash.parse() else {
                    return false;
                };
                let mut hasher = crypt::Hasher::default();
                hasher.update(&file.data);
                if hasher.finalize() != *hash {
                    return false;
                }
            }
            _ => return false,
        }

        self.read_hash() == self.calculate_hash()
    }
}

impl AquaHashable for RevisionMetadata {
    fn calculate_hash(&self) -> Hash {
        let mut hasher = crypt::Hasher::default();
        hasher.update(&self.domain_id);
        hasher.update(self.timestamp.to_string());
        Hash::from(hasher.finalize())
    }
}
impl Verify for RevisionMetadata {}

impl AquaHashable for RevisionSignature {
    fn calculate_hash(&self) -> Hash {
        let mut hasher = crypt::Hasher::default();
        hasher.update(self.signature.to_stackstr());
        hasher.update(self.public_key.to_stackstr());
        Hash::from(hasher.finalize())
    }
}
impl Verify for RevisionSignature {}

impl AquaHashable for RevisionWitness {
    fn calculate_hash(&self) -> Hash {
        let mut hasher = crypt::Hasher::default();
        hasher.update(self.domain_snapshot_genesis_hash.to_stackstr());
        hasher.update(self.merkle_root.to_stackstr());
        hasher.update(&self.witness_network);
        hasher.update(self.witness_event_transaction_hash.to_stackstr());
        Hash::from(hasher.finalize())
    }
}
impl Verify for RevisionWitness {}

impl AquaHashable for RevisionReference {
    fn calculate_hash(&self) -> Hash {
        let mut hasher = crypt::Hasher::default();
        hasher.update(self.verification_hash.to_stackstr());
        if let Some(ref sig) = self.signature {
            hasher.update(sig.read_hash().to_stackstr())
        }
        if let Some(ref wit) = self.witness {
            hasher.update(wit.read_hash().to_stackstr())
        }
        Hash::from(hasher.finalize())
    }
}
impl Verify for RevisionReference {
    fn verify(&self) -> bool {
        if let Some(sig) = &self.signature {
            let mut k = crypt::Keccak256::default();
            k.update(
                "\x19Ethereum Signed Message:\n177I sign the following page verification_hash: [0x",
            );
            k.update(self.verification_hash.to_stackstr());
            k.update("]");
            let message = libsecp256k1::Message::parse(&k.finalize().into());
            let Ok(public_key) = libsecp256k1::recover(
                &message,
                &sig.signature.signature,
                &sig.signature.recovery_id,
            ) else {
                return false;
            };
            if public_key != *sig.public_key {
                return false;
            }
        }
        if let Some(wit) = &self.witness {
            let mut a = std::collections::HashSet::<crypt::Hash>::new();
            let mut b = std::collections::HashSet::<crypt::Hash>::new();
            let mut c = std::collections::HashSet::<crypt::Hash>::new();
            a.insert(*wit.merkle_root);
            for node in &wit.structured_merkle_proof {
                if c.contains(&*node.left_leaf) || a.contains(&*node.left_leaf) {
                    return false;
                }
                if c.contains(&*node.right_leaf) || a.contains(&*node.right_leaf) {
                    return false;
                }
                if b.remove(&*node.left_leaf) {
                    c.insert(*node.left_leaf);
                } else {
                    a.insert(*node.left_leaf);
                }
                if b.remove(&*node.right_leaf) {
                    c.insert(*node.right_leaf);
                } else {
                    a.insert(*node.right_leaf);
                }
                let mut p = crypt::Hasher::default();
                p.update(node.left_leaf.to_stackstr());
                p.update(node.right_leaf.to_stackstr());
                let p_output = p.finalize();
                if c.contains(&p_output) || b.contains(&p_output) {
                    return false;
                }
                if a.remove(&p_output) {
                    c.insert(p_output);
                } else {
                    b.insert(p_output);
                }
            }
            if !a.contains(&*self.verification_hash) {
                return false;
            }
            if !b.is_empty() {
                return false;
            }
            if !c.contains(&*wit.merkle_root) {
                return false;
            }
        }

        self.read_hash() == self.calculate_hash()
    }
}

mod signature;
mod verification;
mod witness;

pub mod hashes {
    pub use super::signature::signature_hash;
    pub use super::verification::content_hash;
    pub use super::verification::metadata_hash;
    pub use super::verification::verification_hash;
    pub use super::witness::witness_hash;
}

#[cfg(test)]
mod tests;

use super::*;
use guardian_common::{prelude::*, storage::Storage};

pub fn revision_integrity_ignore_absent(
    rev: &Revision,
    prev: Option<&Revision>,
) -> flagset::FlagSet<RevisionIntegrity> {
    ignore_absent(revision_integrity(rev, prev))
}

pub fn revision_integrity(
    rev: &Revision,
    prev: Option<&Revision>,
) -> flagset::FlagSet<RevisionIntegrity> {
    let mut integrity = verification::only_verification_hash_integrity(rev, prev);
    integrity |= signature::only_signature_hash_integrity(rev, prev);
    integrity |= witness::only_witness_hash_integrity(rev, prev);
    
    integrity
}

pub fn ignore_absent(
    mut revision_integrity: flagset::FlagSet<RevisionIntegrity>,
) -> flagset::FlagSet<RevisionIntegrity> {
    use RevisionIntegrity::*;
    revision_integrity -= NoSignature;
    revision_integrity -= NoWitness;
    revision_integrity
}

#[cfg(test)]
pub fn hash_chain_integrity(
    hash_chain: &pkc_api::da::HashChain,
) -> (
    flagset::FlagSet<HashChainIntegrity>,
    Vec<(&Revision, flagset::FlagSet<RevisionIntegrity>)>,
) {
    let (mut integrity, chain) = only_hash_chain_extract_revision_chain(hash_chain);

    let revision_integrities = chain
        .iter()
        .copied()
        .map(|(rev, prev)| {
            let rev_integ = revision_integrity(rev, prev);
            if !ignore_absent(rev_integ).is_empty() {
                integrity |= HashChainIntegrity::RevisionIntegrityFatal;
            }
            (rev, rev_integ)
        })
        .collect();

    (integrity, revision_integrities)
}

#[cfg(test)]
fn only_hash_chain_extract_revision_chain(
    hash_chain: &pkc_api::da::HashChain,
) -> (
    flagset::FlagSet<HashChainIntegrity>,
    Vec<(&Revision, Option<&Revision>)>,
) {
    use HashChainIntegrity::*;
    let mut integrity: flagset::FlagSet<_> = Default::default();

    for (key, rev) in &hash_chain.revisions {
        if key != &rev.metadata.verification_hash {
            integrity |= KeyValueMismatch;
        }
    }

    let revisions = {
        let mut revisions = std::collections::HashMap::with_capacity(hash_chain.revisions.len());
        for (key, rev) in &hash_chain.revisions {
            use std::collections::hash_map::Entry::*;
            match revisions.entry(&*rev.metadata.verification_hash) {
                Occupied(mut o) => {
                    integrity |= UnusedRevisions;
                    if key == &rev.metadata.verification_hash {
                        o.insert(rev);
                    }
                }
                Vacant(v) => {
                    v.insert(rev);
                }
            }
        }
        revisions
    };

    // maybe construct using with_capacity for the happy path
    let mut chain: Vec<(&Revision, Option<&Revision>)> = Vec::new();

    'chain_reconstruction: {
        let Some(mut rev) = revisions
            .get(&*hash_chain.hash_chain_info.latest_verification_hash)
            .copied()
        else {
            integrity |= ChainHeadMissing;
            break 'chain_reconstruction;
        };
        loop {
            let Some(prev_key) = rev.metadata.previous_verification_hash.as_deref() else {
                chain.push((rev, None));
                break;
            };

            if rev.metadata.verification_hash == hash_chain.hash_chain_info.genesis_hash {
                integrity |= ChainGenesisIsLink;
            }

            let prev: Option<&Revision> = revisions.get(prev_key).copied();

            chain.push((rev, prev));

            let Some(prev) = prev else {
                integrity |= ChainLinkMissing;
                break;
            };

            rev = prev;
        }

        if rev.metadata.verification_hash != hash_chain.hash_chain_info.genesis_hash {
            integrity |= ChainRootMissing;
        }
    }

    if hash_chain.revisions.len() != chain.len() {
        integrity |= UnusedRevisions;
    }

    (integrity, chain)
}

pub async fn verify_all<S: Storage>(
    pkc: S,
) -> std::collections::HashMap<Hash, flagset::FlagSet<RevisionIntegrity>> {
    let mut integrity: std::collections::HashMap<Hash, flagset::FlagSet<RevisionIntegrity>> =
        std::collections::HashMap::new();

    let last_revisions = pkc.list().await.unwrap();

    for last_revision in last_revisions {
        let branch = pkc.get_branch(last_revision).await.unwrap();
        for revision_hash in branch.hashes {
            let revision = pkc.read(revision_hash).await.unwrap();
            let prev;
            let prev = if let Some(prev_hash) = revision.metadata.previous_verification_hash {
                prev = pkc.read(prev_hash).await.unwrap();
                Some(&prev)
            } else {
                None
            };
            let rev_integrity = revision_integrity(&revision, prev);
            integrity.insert(revision_hash, rev_integrity);
        }
    }

    #[allow(clippy::needless_return)]
    return integrity;
}

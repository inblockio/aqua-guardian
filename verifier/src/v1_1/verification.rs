use std::collections::BTreeMap;

use super::*;

pub fn content_hash(content: &BTreeMap<String, String>) -> Hash {
    // 3.a create hasher {c}
    let mut c = crypt::Hasher::default();
    // 3.b iterate over rev.content.content by its keys
    for value in content.values() {
        // 3.c add each value of rev.content.content to hasher {c}
        c.update(value);
    }
    Hash::from(c.finalize())
}

pub fn metadata_hash(
    domain_id: &str,
    time_stamp: &Timestamp,
    previous_verification_hash: Option<&Hash>,
) -> Hash {
    // 4.a create hasher {m}
    let mut m = crypt::Hasher::default();
    // 4.b add rev.metadata.domain_id to hasher {m}
    m.update(domain_id);
    // 4.c add rev.metadata.time_stamp (in format %Y%m%d%H%M%S) to hasher {m}
    m.update(time_stamp.to_string());
    // 4.d if rev.metadata.previous_verification_hash exists then add rev.metadata.previous_verification_hash to hasher {m}
    if let Some(prev_verification_hash) = previous_verification_hash {
        m.update(prev_verification_hash.to_stackstr());
    }
    Hash::from(m.finalize())
}

pub fn verification_hash(
    content_hash: &Hash,
    metadata_hash: &Hash,
    signature_hash: Option<&Hash>,
    witness_hash: Option<&Hash>,
) -> Hash {
    let mut v = crypt::Hasher::default();
    // 5.b add rev.content.content_hash to hasher {v}
    v.update(content_hash.to_stackstr());
    // 5.c add rev.metadata.metadata_hash to hasher {v}
    v.update(metadata_hash.to_stackstr());
    // 5.d if prev?.signature exists then add prev.signature.signature_hash to hasher {v}
    if let Some(prev_signature_hash) = signature_hash {
        v.update(prev_signature_hash.to_stackstr());
    }
    // 5.e if prev?.witness exists then add prev.witness.witness_hash to hasher {v}
    if let Some(prev_witness_hash) = witness_hash {
        v.update(prev_witness_hash.to_stackstr());
    }
    Hash::from(v.finalize())
}

/// [a] verification_hash integrity
///
/// IMPORTANT: what does this verify?
/// - content and metadata have not been messed with
/// - this does not tell you anything about signature or witness
///
/// IMPORTANT: how to verify prev?
/// - recursively is possible but make sure not to get caught by a loop where two or more revisions point to one another as their prevs
///
/// prerequisites: rev, prev? [trusted verification_hash, signature_hash and witness_hash]
pub(super) fn only_verification_hash_integrity(
    rev: &Revision,
    prev: Option<&Revision>,
) -> flagset::FlagSet<RevisionIntegrity> {
    use RevisionIntegrity::*;
    let mut integrity = flagset::FlagSet::default();

    // 1 previous_verification_hash
    // 1.a either both or neither of prev and rev.metadata.previous_verification_hash exist
    'prev_verification_hash: {
        if let Some(prev_verification_hash) = &rev.metadata.previous_verification_hash {
            let Some(prev) = prev else {
                integrity |= NoPrevRevision;
                break 'prev_verification_hash;
            };
            // 1.b if both exist then prev.metadata.verification_hash == rev.metadata.previous_verification_hash
            if prev.metadata.verification_hash != *prev_verification_hash {
                integrity |= PrevVerificationHashNotMatching;
            }
        } else {
            #[allow(clippy::collapsible_else_if)]
            if prev.is_some() {
                integrity |= PrevVerificationHashNotMatching;
            }
        }
    }

    // 2 file_hash
    // 2.a make sure both or neither of rev.content.file and rev.content.content["file_hash"] exist
    // 2.b if neither exist move on to [3]
    'file_hash: {
        if let Some(file) = &rev.content.file {
            let Some(file_hash) = rev.content.content.get("file_hash") else {
                integrity |= FileHashNotMatching;
                break 'file_hash;
            };
            // 2.c create hasher {f}
            let mut f = crypt::Hasher::default();
            // 2.d add rev.content.file.data as raw data (not base64) to hasher {f}
            f.update(file.data.as_ref());
            // 2.e output of hasher {f} equals rev.content.content["file_hash"]
            //todo: should we instead try to parse the file_hash to a hash?
            // what about differently formatted hashes in file_hash?
            if Hash::from(f.finalize()).to_string() != file_hash.as_str() {
                integrity |= FileHashNotMatching;
            }
        } else {
            #[allow(clippy::collapsible_else_if)]
            if rev.content.content.contains_key("file_hash") {
                integrity |= NoFile;
            }
        }
    }

    // 3 content_hash
    // 3.d output of hasher {c} equals rev.content.content_hash
    if content_hash(&rev.content.content) != rev.content.content_hash {
        integrity |= ContentHashNotMatching;
    }

    // 4 metadata_hash
    // 4.e output of hasher {m} equals rev.metadata.metadata_hash
    if metadata_hash(
        &rev.metadata.domain_id,
        &rev.metadata.time_stamp,
        rev.metadata.previous_verification_hash.as_ref(),
    ) != rev.metadata.metadata_hash
    {
        integrity |= MetadataHashNotMatching;
    }

    // 5 verification_hash
    // 5.a create hasher {v}
    // 5.f output of hasher {v} equals rev.metadata.verification_hash
    if verification_hash(
        &rev.content.content_hash,
        &rev.metadata.metadata_hash,
        rev.signature.as_ref().map(|s| &s.signature_hash),
        rev.witness.as_ref().map(|w| &w.witness_hash),
    ) != rev.metadata.verification_hash
    {
        integrity |= VerificationHashNotMatching;
    }

    integrity
}

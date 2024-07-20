use super::*;

pub fn signature_hash(signature: &Signature, public_key: &PublicKey) -> Hash {
    // 4.a create hasher {s}
    let mut s = crypt::Hasher::default();
    // 4.b add rev.signature.signature to hasher {s}
    s.update(signature.to_stackstr());
    // 4.c add rev.signature.public_key to hasher {s}
    s.update(public_key.to_stackstr());
    Hash::from(s.finalize())
}

/// [b] signature_hash integrity
///
/// IMPORTANT: what does this verify?
/// - it verifies that the revision's verification_hash has been signed using the listed public_key
/// - it verifies that the signature_hash hashed that signature with that public_key
///
/// prerequisites: rev [trusted verification_hash]
pub(super) fn only_signature_hash_integrity(rev: &Revision, prev: Option<&Revision>) -> flagset::FlagSet<RevisionIntegrity> {
    use RevisionIntegrity::*;
    let mut integrity = flagset::FlagSet::default();

    let Some(sign) = &rev.signature else {
        return flagset::FlagSet::from(NoSignature);
    };
    let Some(prev_hash) = &prev.map(|a|a.metadata.verification_hash) else {
        return flagset::FlagSet::from(NoPrevRevision);
    };

    // 1 signature + recovery_id
    // 1.a deserialize the hex-encoded rev.signature.signature into exactly 65 bytes
    // 1.b declare first 64 bytes of decoded rev.signature.signature as signature
    // 1.c declare last byte of decoded rev.signature.signature as recovery_id
    // 1.d apply secp256k1 parse_standard to 64 byte signature
    // 1.e parse recovery_id as ethereum rpc format (make sure it is equal to 27, 28, 29 or 30)

    // 2 public_key
    // 2.a parse the hex-encoded rev.signature.public_key as secp256k1 public key

    // already done during parsing

    // 3 verifying the signature
    // 3.a create sha3 Keccak256 hasher {k}
    let mut k = crypt::Keccak256::default();
    // 3.b add "\x19Ethereum Signed Message:\n177I sign the following page verification_hash: [0x" to hasher {k}
    k.update("\x19Ethereum Signed Message:\n177I sign the following page verification_hash: [0x");
    // 3.c add rev.metadata.verification_hash to hasher {k}
    k.update(prev_hash.to_stackstr());
    // 3.d add "]" to hasher {k}
    k.update("]");
    // 3.e parse output of hasher {k} as secp256k1 message
    let message = libsecp256k1::Message::parse(&k.finalize().into());
    // 3.f recover public_key with secp256k1's recover using message, signature and recovery_id
    let Signature {
        signature,
        recovery_id,
    } = sign.signature;
    let Ok(public_key) = libsecp256k1::recover(&message, &signature, &recovery_id) else {
        return flagset::FlagSet::from(SignatureError);
    };
    // 3.g check equality of public_key and parsed public_key
    if public_key != *sign.public_key {
        integrity |= PublicKeyNotMatching;
    }

    // 4 signature_hash
    // 4.d output of hasher {s} equals rev.signature.signature_hash
    if signature_hash(&sign.signature, &sign.public_key) != sign.signature_hash {
        integrity |= SignatureHashNotMatching;
    }

    integrity
}

// todo: do elsewhere

// /// [d] signature identity
// ///
// /// IMPORTANT: what does this verify?
// /// - the revision has been signed by a public_key on the provided list
// ///
// /// prerequisites: rev [trusted verification_hash, signature_hash], pubkey_list
// pub(super) fn signature_identity(rev: &Revision, trusted_keys: &[libsecp256k1::PublicKey]) -> bool {
//     let Some(signature) = &rev.signature else {
//         return false;
//     };
//     // 1 rev.signature.public_key is in pubkey_list (watch out for different pubkey formats)
//     trusted_keys.contains(&signature.public_key)
// }

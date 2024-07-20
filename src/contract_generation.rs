use ethaddr::Address;
use guardian_common::custom_types::{Revision, RevisionContent, Timestamp};
use verifier::v1_1::hashes::{metadata_hash, verification_hash};

fn make_new_cert(
    key_pair: rcgen::KeyPair,
    ip: std::net::IpAddr,
    ethaddr: Address,
) -> Result<rcgen::Certificate, rcgen::Error> {
    // let key_pair = rcgen::KeyPair::generate()?;
    let ethaddr = ethaddr.to_string();
    let mut params = rcgen::CertificateParams::new([ip.to_string(), ethaddr.clone()])?;
    params.distinguished_name.push(
        rcgen::DnType::CommonName,
        rcgen::DnValue::Utf8String(ethaddr),
    );
    params.self_signed(&key_pair)
}

fn make_genesis(content: RevisionContent, time: Timestamp, domain_id: String) -> Revision {
    use guardian_common::prelude::*;
    let metadata_hash = metadata_hash(&domain_id, &time, None);

    let verification_hash = verification_hash(&content.content_hash, &metadata_hash, None, None);

    Revision {
        content,
        metadata: RevisionMetadata {
            time_stamp: time.clone(),
            verification_hash,
            previous_verification_hash: None,
            metadata_hash,
            domain_id,
        },
        signature: None,
        witness: None,
    }
}

fn signed_revision_v1_1<S: guardian_common::signing::Signer>(
    rev: &Revision,
    s: S,
    domain_id: String,
    time_stamp: Timestamp,
) -> Revision {
    use guardian_common::prelude::*;

    let public_key = s.identity();

    let signature = guardian_common::signing::sign_revision_hash(s, rev.metadata.verification_hash);

    let signature_hash = {
        let mut s = crypt::Hasher::default();
        s.update(signature.to_stackstr());
        s.update(public_key.to_stackstr());
        Hash::from(s.finalize())
    };

    let sig = RevisionSignature {
        signature,
        public_key,
        signature_hash,
        wallet_address: public_key.into(),
    };

    let content_content = {
        let mut content = rev.content.content.clone();
        #[derive(serde::Deserialize, serde::Serialize)]
        struct SignatureSlot {
            user: Address,
            timestamp: Timestamp,
        }
        let (key, mut existing_signatures): (_, Vec<SignatureSlot>) = content
            .remove_entry("signature-slot")
            .and_then(|(slot, s)| serde_json::from_str(&s[..]).ok().map(|sig| (slot, sig)))
            .unwrap_or_else(|| ("signature-slot".to_string(), vec![]));
        existing_signatures.push(SignatureSlot {
            user: public_key.into(),
            timestamp: time_stamp.clone(),
        });
        content.insert(
            key,
            serde_json::to_string(&existing_signatures).expect("error serializing, shit's broken"),
        );

        content
    };
    let content_hash = verifier::v1_1::hashes::content_hash(&content_content);
    let metadata_hash = verifier::v1_1::hashes::metadata_hash(
        &domain_id,
        &time_stamp,
        Some(&rev.metadata.verification_hash),
    );
    let verification_hash = verifier::v1_1::hashes::verification_hash(
        &content_hash,
        &metadata_hash,
        Some(&signature_hash),
        None,
    );
    Revision {
        content: RevisionContent {
            file: rev.content.file.clone(),
            content: content_content,
            content_hash,
        },
        metadata: RevisionMetadata {
            domain_id,
            time_stamp,
            verification_hash,
            previous_verification_hash: Some(rev.metadata.verification_hash),
            metadata_hash,
        },
        signature: Some(sig),
        witness: None,
    }
}

pub fn make_guardian_cert<S: guardian_common::signing::Signer>(
    key_pair: rcgen::KeyPair,
    ip: std::net::IpAddr,
    port: u16,
    s: S,
) -> Result<(rcgen::Certificate, Vec<Revision>), rcgen::Error> {
    let guardian = Address::from(s.identity());
    let cert = make_new_cert(key_pair, ip, guardian)?;
    let now = chrono::Utc::now().naive_utc();

    let contract_content =
        contract_interpreter::Contract::TlsIdentityClaim(contract_interpreter::TlsIdentityClaim {
            host: ip.to_string(),
            port,
            cert: cert.der().to_vec().into(),
            guardian,
        })
        .make_content();

    let genesis = make_genesis(contract_content, now.into(), guardian.to_string());
    let guardian_signed =
        signed_revision_v1_1(&genesis, s, guardian.to_string(), now.into());

    Ok((cert, vec![genesis, guardian_signed]))
}

pub fn make_tls_cert_contract<S: guardian_common::signing::Signer>(
    ip: std::net::IpAddr,
    port: u16,
    s: S,
    cert: webpki::types::CertificateDer<'static>,
) -> Result<Vec<Revision>, rcgen::Error> {
    let guardian = Address::from(s.identity());
    let now = chrono::Utc::now().naive_utc();

    let contract_content =
        contract_interpreter::Contract::TlsIdentityClaim(contract_interpreter::TlsIdentityClaim {
            host: ip.to_string(),
            port,
            cert: cert.to_vec().into(),
            guardian,
        })
        .make_content();

    let genesis = make_genesis(contract_content, now.into(), guardian.to_string());
    let guardian_signed =
        signed_revision_v1_1(&genesis, s, guardian.to_string(), now.into());
    // genesis.signature = Some(sig); // this seems to no longer be required? see make_guardian_cert
    Ok(vec![genesis, guardian_signed])
}

pub fn make_guardian_servitude<S: guardian_common::signing::Signer>(
    user: Address,
    s: S,
) -> Vec<Revision> {
    let guardian = Address::from(s.identity());
    let now = chrono::Utc::now().naive_utc();
    let contract_content = contract_interpreter::Contract::GuardianServitude(
        contract_interpreter::GuardianServitude { guardian, user },
    )
    .make_content();
    let genesis = make_genesis(contract_content, now.into(), guardian.to_string());
    let guardian_signed =
        signed_revision_v1_1(&genesis, s, guardian.to_string(), now.into());
    vec![genesis, guardian_signed]
}

#[test]
fn generate_contracts() {
    make_new_cert(
        rcgen::KeyPair::generate().unwrap(),
        ([172, 0, 0, 0]).into(),
        ethaddr::Address::default(),
    )
    .unwrap();
    let genesis = make_genesis(
        RevisionContent::default(),
        chrono::Utc::now().naive_utc().into(),
        "100".to_string(),
    );
    let signer: guardian_common::signing::SimpleSigner =
        ("0x284750bbd0425ce597494511b7a4d579d0b366633af7584050610d64971141a7")
            .parse()
            .expect("failed to parse private key");
    signed_revision_v1_1(
        &genesis,
        signer,
        "100".to_string(),
        chrono::Utc::now().naive_utc().into(),
    ); // ?????
}

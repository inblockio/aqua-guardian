/// Guardian TLS Certificate
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TlsIdentityClaim {
    /// The bytes of a [`webpki::types::CertificateDer<'static>`].
    pub cert: std::sync::Arc<[u8]>,
    /// The address of the Guardian who declares this one of its certificates by signature.
    pub guardian: ethaddr::Address,
    /// The hostname under which to reach the Guardian.
    pub host: String,
    /// The port on which the Guardian is listening.
    pub port: u16,
}

impl std::fmt::Debug for TlsIdentityClaim {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TlsIdentityClaim")
            .field(
                "cert",
                &base64::display::Base64Display::new(
                    &self.cert,
                    &base64::engine::general_purpose::STANDARD,
                ).to_string(),
            )
            .field("guardian", &self.guardian)
            .field("host", &self.host)
            .field("port", &self.port)
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum TlsIdentityClaimEffects {
    /// The Identity Claim is effective. This is where to contact the Guardian
    IdentityClaimed,
}

const DECLARATION: Option<u8> = Some(0);
const SIGNATURE: Option<u8> = Some(1);

impl super::SequencedContract for TlsIdentityClaim {
    type Effect = TlsIdentityClaimEffects;

    /// Checks the effectiveness of the given revisions of the Guardian TLS Certificate (passed as Iterator).
    #[allow(clippy::question_mark)]
    fn is_effective(
        &self,
        mut revisions: impl std::iter::Iterator<Item = Option<u8>>,
    ) -> Option<TlsIdentityClaimEffects> {
        // if there are terms, signatures of both receiver and sender are needed
        let Some(SIGNATURE) = revisions.next() else {
            return None;
        };
        let Some(DECLARATION) = revisions.next() else {
            return None;
        };
        Some(TlsIdentityClaimEffects::IdentityClaimed)
    }

    /// Determines the number of the ''effectiveness'' state of the Guardian TLS Certificate revision based on the presence or absence of guardian signature.
    fn sequence_number(&self, rev: &verifier::v1_2::Revision) -> Option<u8> {
        let Some(prev) = &rev.prev else {
            return DECLARATION;
        };
        if let Some(signature) = &prev.signature {
            if self.guardian == ethaddr::Address::from(signature.public_key) {
                return SIGNATURE;
            }
        }
        None
    }
}
/// Enumeration of possible errors for the Guardian TLS Certificate.
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum TlsIdentityClaimError {
    #[error("certificate file missing")]
    CertMissing,
    #[error("cert not base64")]
    CertNotBase64,
    #[error("certificate file malformatted {0}")]
    CertMalformatted(webpki::Error),
    #[error("guardian not certificate subject")]
    CertSubjectMismatch(webpki::Error),

    #[error("guardian address missing")]
    GuardianMissing,
    #[error("guardian address malformatted {0}")]
    GuardianMalformatted(ethaddr::ParseAddressError),

    #[error("host missing")]
    HostMissing,

    #[error("guardian address missing")]
    PortMissing,
    #[error("guardian address malformatted {0}")]
    PortMalformatted(std::num::ParseIntError),

    #[error("unknown options specified")]
    AdditionalKeys,
}

use super::GenericContractInfo;

impl TryFrom<GenericContractInfo<'_>> for TlsIdentityClaim {
    type Error = TlsIdentityClaimError;
    /// Tries to generate a Guardian TLS Certificate from the [`GenericContractInfo`].
    fn try_from(info: GenericContractInfo) -> Result<Self, Self::Error> {
        let GenericContractInfo {
            mut params,
            // file,
            ..
        } = info;

        use TlsIdentityClaimError::*;

        let guardian_str = params.remove("guardian").ok_or(GuardianMissing)?;
        let guardian =
            ethaddr::Address::from_str_checksum(&guardian_str).map_err(GuardianMalformatted)?;

        let host = params.remove("host").ok_or(HostMissing)?;

        let port_str = params.remove("port").ok_or(PortMissing)?;
        let port = str::parse(&port_str).map_err(PortMalformatted)?;

        // let cert_file = file.ok_or(CertMissing)?;
        let cert_file: guardian_common::custom_types::Base64 = params
            .remove("file")
            .ok_or(CertMissing)?
            .parse()
            .map_err(|_| CertNotBase64)?;
        let cert_der = webpki::types::CertificateDer::from(Vec::from(cert_file.to_owned()));
        let end_entity_cert =
            webpki::EndEntityCert::try_from(&cert_der).map_err(CertMalformatted)?;
        end_entity_cert
            .verify_is_valid_for_subject_name(&webpki::types::ServerName::DnsName(
                webpki::types::DnsName::try_from(guardian_str)
                    .expect("weird, an address should be a valid dns subject name"),
            ))
            .map_err(CertSubjectMismatch)?;

        if !params.is_empty() {
            // > after all params must be empty, correct?
            // yes.
            return Err(AdditionalKeys);
        }

        Ok(TlsIdentityClaim {
            guardian,
            host,
            port,
            cert: cert_file.to_vec().into(),
        })
    }
}

#[test]
fn address_is_dnsname() {
    webpki::types::DnsName::try_from("0x8B6488E003B81ecF69f108f7D10A62eB1D40afb6")
        .expect("weird, an address should be a valid dns subject name");
}

/// Guardian Servitude
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct GuardianServitude {
    /// the guardian who wants to serve the [`user`]
    pub guardian: ethaddr::Address,
    /// the user who accepts the [`guardian`] serving them
    pub user: ethaddr::Address,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum GuardianServitudeEffects {
    /// first revision, unsigned, a suggestion to be accepted (signed) by the Guardian.
    Suggested,
    /// has a signature of the Guardian, this declaration is to be accepted (signed) by the user.
    Declared,
    /// has signatures of the Guardian and of the user. This confirms that the Guardian, that signed this contract,  serves to the user, who signed this contract.
    Accepted,
}

/// Enumeration of error types for the Guardian Servitude
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum GuardianServitudeError {
    #[error("guardian address missing")]
    GuardianMissing,
    #[error("guardian address malformatted {0}")]
    GuardianMalformatted(ethaddr::ParseAddressError),

    #[error("user address missing")]
    UserMissing,
    #[error("user address malformatted {0}")]
    UserMalformatted(ethaddr::ParseAddressError),

    #[error("unknown options specified")]
    AdditionalKeys,
}

const DECLARATION: Option<u8> = Some(0);
const GUARDIAN_SIGNATURE: Option<u8> = Some(1);
const USER_SIGNATURE: Option<u8> = Some(2);

impl super::SequencedContract for GuardianServitude {
    type Effect = GuardianServitudeEffects;

    /// Checks the effectiveness of the given revisions of the Guardian Servitude (passed as Iterator).
    fn is_effective(
        &self,
        mut revisions: impl std::iter::Iterator<Item = Option<u8>>,
    ) -> Option<GuardianServitudeEffects> {
        match (
            revisions.next(),
            revisions.next(),
            revisions.next(),
            revisions.next(),
        ) {
            (Some(USER_SIGNATURE), Some(GUARDIAN_SIGNATURE), Some(DECLARATION), None) => {
                Some(GuardianServitudeEffects::Accepted)
            }
            (Some(GUARDIAN_SIGNATURE), Some(DECLARATION), None, ..) => {
                Some(GuardianServitudeEffects::Declared)
            }
            (Some(DECLARATION), None, ..) => Some(GuardianServitudeEffects::Suggested),
            _ => None,
        }
    }

    /// Determines the number of the ''effectiveness'' state of the Guardian Servitude revision based on the presence or absence of guardian and (authoritative) user signatures.
    fn sequence_number(&self, rev: &verifier::v1_2::Revision) -> Option<u8> {
        let Some(prev) = &rev.prev else {
            return DECLARATION;
        };
        if let Some(signature) = &prev.signature {
            let signing_addr = ethaddr::Address::from(signature.public_key);
            if self.guardian == signing_addr {
                return GUARDIAN_SIGNATURE;
            }
            if self.user == signing_addr {
                return USER_SIGNATURE;
            }
        }
        None
    }
}

use super::GenericContractInfo;

impl TryFrom<GenericContractInfo<'_>> for GuardianServitude {
    type Error = GuardianServitudeError;

    /// Tries to generate a Guardian Servitude from the [`GenericContractInfo`].
    fn try_from(info: GenericContractInfo) -> Result<Self, Self::Error> {
        let GenericContractInfo { mut params, .. } = info;

        use GuardianServitudeError::*;

        let guardian =
            ethaddr::Address::from_str_checksum(&params.remove("guardian").ok_or(GuardianMissing)?)
                .map_err(GuardianMalformatted)?;

        let user = ethaddr::Address::from_str_checksum(&params.remove("user").ok_or(UserMissing)?)
            .map_err(UserMalformatted)?;

        if !params.is_empty() {
            return Err(AdditionalKeys);
        }

        Ok(GuardianServitude { guardian, user })
    }
}

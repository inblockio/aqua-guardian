use super::*;

/// Data Access Agreement
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct AccessAgreement {
    pub sender: Address,
    pub receiver: Address,
    pub pages: Vec<(String, Hash)>,
    pub terms: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AccessAgreementEffects {
    /// no terms, signed by declaring user. this should share the file to the receiver
    Granted,
    /// Has terms, signed by the declaring user. This should share the contract itself to the receiver but not the file.
    Offered,
    /// Has terms, signed by the receiver. This should share the contract back to the declaring user and then share the file from the declaring user to the receiver.
    Accepted,
}
/// Enumeration of error types for the Data Access Agreement
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum AccessAgreementError {
    #[error("sender missing")]
    SenderMissing,
    #[error("sender malformatted {0}")]
    SenderMalformatted(ethaddr::ParseAddressError),

    #[error("receiver missing")]
    ReceiverMissing,
    #[error("receiver malformatted {0}")]
    ReceiverMalformatted(ethaddr::ParseAddressError),

    #[error("pages missing")]
    PagesMissing,
    #[error("page not transcluded")]
    PageNotTranscluded,

    #[error("unknown options specified")]
    AdditionalKeys,
}

const DECLARATION: Option<u8> = Some(0);
const SENDER_SIGNATURE: Option<u8> = Some(1);
const RECEIVER_SIGNATURE: Option<u8> = Some(2);

impl super::SequencedContract for AccessAgreement {
    type Effect = AccessAgreementEffects;

    /// Checks the effectiveness of the given revisions of the Data Access Agreement (passed as Iterator).
    fn is_effective(
        &self,
        revisions: impl std::iter::Iterator<Item = Option<u8>>,
    ) -> Option<AccessAgreementEffects> {
        let mut states = revisions.flatten();
        // revisions are traversed in descending order, i.e.  [receiver_sig, sender_sig, declaration],
        // and so are the corresponding states [2, 1, 0]

        match (
            self.terms.is_some(),
            states.next(),
            states.next(),
            states.next(),
            states.next(),
        ) {
            (true, RECEIVER_SIGNATURE, SENDER_SIGNATURE, DECLARATION, None) => {
                Some(AccessAgreementEffects::Accepted)
            }
            (terms, SENDER_SIGNATURE, DECLARATION, None, ..) => Some(if terms {
                AccessAgreementEffects::Offered
            } else {
                AccessAgreementEffects::Granted
            }),
            _ => None,
        }
    }

    /// Determines the number of the ''effectiveness'' state of the Data Access Agreement revision based on the presence or absence of sender and receiver signatures.
    fn sequence_number(&self, rev: &verifier::v1_2::Revision) -> Option<u8> {
        let Some(prev) = &rev.prev else {
            return DECLARATION;
        };
        if let Some(signature) = &prev.signature {
            let signing_addr = ethaddr::Address::from(signature.public_key);
            if self.sender == signing_addr {
                return SENDER_SIGNATURE;
            }
            if self.receiver == signing_addr {
                return RECEIVER_SIGNATURE;
            }
        }
        None
    }
}

use super::GenericContractInfo;

impl TryFrom<GenericContractInfo<'_>> for AccessAgreement {
    type Error = AccessAgreementError;

    /// Tries to generate a Data Access Agreement from the [`GenericContractInfo`].
    fn try_from(info: GenericContractInfo) -> Result<Self, Self::Error> {
        let GenericContractInfo {
            transclusions,
            mut params,
            ..
        } = info;

        use AccessAgreementError::*;

        let sender = params
            .remove("sender")
            .ok_or(SenderMissing)?
            .parse()
            .map_err(SenderMalformatted)?;

        let receiver = params
            .remove("receiver")
            .ok_or(ReceiverMissing)?
            .parse()
            .map_err(ReceiverMalformatted)?;

        let pages = params
            .remove("pages")
            .ok_or(PagesMissing)?
            .split(", ")
            .map(|name| {
                // Replace of "Media:" to allow correct mapping of transcluded file title to its transclusion hash (MediaWiki limitation)
                let tmp_name = name.replace(" ", "_").replace("Media:", "File:");
                let name = tmp_name.as_str();
                transclusions
                    .get(name)
                    .copied()
                    .map(|hash| (name.to_string(), hash))
            })
            .collect::<Option<_>>()
            .ok_or(PageNotTranscluded)?;

        let terms = params.remove("terms");

        if !params.is_empty() {
            // after all params must be empty, correct?

            return Err(AdditionalKeys);
        }

        Ok(AccessAgreement {
            sender,
            receiver,
            pages,
            terms,
        })
    }
}

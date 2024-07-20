use std::collections::HashMap;

use guardian_common::{
    crypt::Digest,
    custom_types::Base64,
    prelude::{Address, Hash},
};
use verifier::v1_2::Revision;

mod access_agreement;
pub use access_agreement::*;
mod guardian_servitude;
pub use guardian_servitude::*;
mod tls_identity_claim;
pub use tls_identity_claim::*;

/// Trait for the contracts whose ''effectiveness'' depends on the correct order of their revisions.
pub trait SequencedContract {
    type Effect;
    /// Checks if given list of revisions forms a completed contract.
    fn is_effective(
        &self,
        revisions: impl std::iter::Iterator<Item = Option<u8>>,
    ) -> Option<Self::Effect>;
    /// Figures out if revision is part of contract, and if yes returns Some(u8) for use with the [`SequencedContract::is_effective`] function.
    fn sequence_number(&self, rev: &verifier::v1_2::Revision) -> Option<u8>;
}

/// Enumeration of possible contract types.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[non_exhaustive]
pub enum Contract {
    /// Data Access Agreement that is used to share Aqua-Chains with other users.
    AccessAgreement(AccessAgreement),
    /// Guardian Servitude that is used to bind a Guardian to a user.
    GuardianServitude(GuardianServitude),
    /// [mTLS](https://en.wikipedia.org/wiki/Mutual_authentication#mTLS) Certificate of the Guardian.
    TlsIdentityClaim(TlsIdentityClaim),
}

macro_rules! matchhash {
    ($($contract:ident <-> $hex:literal),* $(,)?) => {
        impl TryFrom<GenericContractInfo<'_>> for Contract {
            type Error = ContractParseError;

            fn try_from(gci: GenericContractInfo) -> Result<Self, Self::Error> {
                match <[u8; 64]>::from(*gci.hash) {
                    $(
                        hash if hash == ::hex_literal::hex!($hex) => {
                            let ret = $contract::try_from(gci);
                            dbg!(&ret);
                            Ok(Contract::$contract(ret?))
                        },
                    )*
                    _ => Err(ContractParseError::UnknownContractHash)
                }
            }
        }
        impl Contract {
            fn contract_hash(&self) -> Hash {
                match self {
                    $(
                        Contract::$contract(_) => {
                            ::hex_literal::hex!($hex).into()
                        },
                    )*
                }
            }
        }
    };

}
// Hashes of contracts need to be adjusted here for the contract to be recognized as valid contracts.
// When templates are changed, the hashes need to be updated here.
// %todo this be imported more elegantly in the future.
matchhash! {
    AccessAgreement <-> "725c2b99a955a690e50a1f22f356a64b02c144dd5adcbc09ac09f861fe2cc45a47185d7a9f5ecc60af86c0e60545aabe8c8c9c34feff92ea1da511ec0e2ef2ac",
    GuardianServitude <-> "2c82d270181179987518d620c102a0fc9db1d5ed7238795cc87d9e1de70ed3b6f67236dd3152881d620f9270b7dcb7fea72bd7e9b859dc2478a3058b078f5204",
    TlsIdentityClaim <-> "95ce4ec4bf2b92019feff4843ddd7b849db8c7c0bd2afe325566dee7c6d5bcc6d1870032d3fa5230bb2f184a689f9b758f8282a2a1984238178581fb7895df13",
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ContractEffect {
    AccessAgreement(
        (
            access_agreement::AccessAgreement,
            access_agreement::AccessAgreementEffects,
        ),
    ),
    GuardianServitude(
        (
            guardian_servitude::GuardianServitude,
            guardian_servitude::GuardianServitudeEffects,
        ),
    ),
    TlsIdentityClaim(
        (
            tls_identity_claim::TlsIdentityClaim,
            tls_identity_claim::TlsIdentityClaimEffects,
        ),
    ),
}

impl Contract {
    /// Creates the content of a revision from a generic Contract data.\
    /// This can be used to construct a revision in form of JSON file for this to be then pushed in the PKC.
    pub fn make_content(&self) -> guardian_common::custom_types::RevisionContent {
        let mut content = std::collections::BTreeMap::default();

        let name = match self {
            Contract::AccessAgreement(_) => "AccessAgreement",
            Contract::GuardianServitude(_) => "GuardianServitude",
            Contract::TlsIdentityClaim(_) => "TlsIdentityClaim",
        };

        lazy_static::lazy_static! {
            static ref REPLACE: aho_corasick::AhoCorasick = aho_corasick::AhoCorasick::new(["|", "{", "}"]).unwrap();
        };

        let mut transclusions = vec![Transclusion {
            dbkey: name,
            ns: 10,
            verification_hash: self.contract_hash(),
        }];
        let mut main = format!("{{{{{}\n|", name);
        let /*mut*/ file = None;

        match self {
            Contract::AccessAgreement(aa) => {
                main += &format!("sender={}\n|", aa.sender);
                main += &format!("receiver={}\n|", aa.receiver);
                let mut filesiter = aa.pages.iter();
                main += &format!(
                    "files={}",
                    filesiter.next().map(|(k, _v)| &k[..]).unwrap_or_default()
                );
                for (filename, transcluded_hash) in filesiter {
                    main += ",";
                    main += filename;
                    transclusions.push(Transclusion {
                        dbkey: filename,
                        // todo: aaaaaaaa namespace
                        ns: 0,
                        verification_hash: *transcluded_hash,
                    });
                }
                if let Some(terms) = &aa.terms {
                    main += &format!(
                        "\n|terms={}",
                        REPLACE
                            .try_replace_all(terms, &["{{|}}", "{{(}}", "{{)}}"])
                            .expect("what the fuck is a kilometer")
                    );
                }
                main += "\n}}";
            }
            Contract::GuardianServitude(gs) => {
                main += &format!("guardian={}\n|", gs.guardian);
                main += &format!("user={}\n}}}}", gs.user);
            }
            Contract::TlsIdentityClaim(tic) => {
                // file = Some(guardian_common::custom_types::FileContent {
                //     size: tic.cert.len() as u32,
                //     data: tic.cert.to_vec().into(),
                //     filename: tic.guardian.to_string(),
                //     comment: Default::default(),
                // });
                main += &format!("guardian={}\n|", tic.guardian);
                main += &format!("file={}\n|", Base64::from(tic.cert.to_vec()));
                main += &format!("host={}\n|", tic.host);
                main += &format!("port={}\n}}}}", tic.port);
                // let file_hash = guardian_common::crypt::Hasher::digest(&tic.cert[..]);
                // content.insert(
                //     "file_hash".to_string(),
                //     Hash::from(file_hash).to_stackstr().to_string(),
                // );
            }
        }

        content.insert(
            "transclusion-hashes".to_string(),
            serde_json::to_string(&transclusions).unwrap(),
        );
        content.insert("main".to_string(), main);

        let content_hash = {
            let mut c = guardian_common::crypt::Hasher::default();
            for value in content.values() {
                c.update(value);
            }
            Hash::from(c.finalize())
        };

        guardian_common::custom_types::RevisionContent {
            file,
            content,
            content_hash,
        }
    }
}

impl SequencedContract for Contract {
    type Effect = ContractEffect;

    fn is_effective(
        &self,
        revisions: impl std::iter::Iterator<Item = Option<u8>>,
    ) -> Option<ContractEffect> {
        Some(match self {
            Contract::AccessAgreement(aa) => {
                ContractEffect::AccessAgreement((aa.clone(), aa.is_effective(revisions)?))
            }
            Contract::GuardianServitude(gs) => {
                ContractEffect::GuardianServitude((gs.clone(), gs.is_effective(revisions)?))
            }
            Contract::TlsIdentityClaim(tic) => {
                ContractEffect::TlsIdentityClaim((tic.clone(), tic.is_effective(revisions)?))
            }
        })
    }

    fn sequence_number(&self, revision: &verifier::v1_2::Revision) -> Option<u8> {
        match self {
            Contract::AccessAgreement(aa) => aa.sequence_number(revision),
            Contract::GuardianServitude(gs) => gs.sequence_number(revision),
            Contract::TlsIdentityClaim(tic) => tic.sequence_number(revision),
        }
    }
}

/// Enumeration of possible errors while parsing a contract.
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum ContractParseError {
    #[error("unknown contract hash")]
    UnknownContractHash,
    #[error("access agreement")]
    AccessAgreement(#[from] AccessAgreementError),
    #[error("guardian servitude")]
    GuardianServitude(#[from] GuardianServitudeError),
    #[error("tls identity claim")]
    TlsIdentityClaim(#[from] TlsIdentityClaimError),
}

/// This structure represents a generic contract
#[non_exhaustive]
pub struct GenericContractInfo<'a> {
    // rev: &'a Revision,
    /// Verification hash of the contract.\
    /// More specifically it is a verification hash of the contract template used to create a contract.
    pub hash: Hash,
    /// Name of the file linked to the revision
    pub file: Option<&'a [u8]>,
    /// The transclusion hashes can be found in content -> content -> *transclusion-hashes*.\
    /// These are the transclusion hashes of the pages linked to a revision.
    pub transclusions: Transclusions<'a>,
    /// The parameters can be found in content -> content -> main of a single revision.\
    /// They are the fields of a specific contract (i.e. *sender*, *receiver*, *user* etc.)
    pub params: ContractParams<'a>,
}

/// Trait to access a contract.
pub trait ContractInfo {
    /// Returns a contract, which contains all of the fields of a specific contract (i.e. *sender*, *receiver*, *user* etc.)
    fn get_contract_data(&self) -> Option<&Contract>;
    /// Returns sequence number of a contract.\
    /// See more here: ´[SequencedContract::sequence_number]´
    fn get_contract_seqno(&self) -> Option<u8>;
}
impl ContractInfo for (&Contract, Option<u8>) {
    fn get_contract_data(&self) -> Option<&Contract> {
        Some(self.0)
    }
    fn get_contract_seqno(&self) -> Option<u8> {
        self.1
    }
}
impl<T: ContractInfo + ?Sized> ContractInfo for std::sync::Arc<T> {
    fn get_contract_data(&self) -> Option<&Contract> {
        self.as_ref().get_contract_data()
    }
    fn get_contract_seqno(&self) -> Option<u8> {
        self.as_ref().get_contract_seqno()
    }
}
impl<T: ContractInfo + ?Sized> ContractInfo for Box<T> {
    fn get_contract_data(&self) -> Option<&Contract> {
        self.as_ref().get_contract_data()
    }
    fn get_contract_seqno(&self) -> Option<u8> {
        self.as_ref().get_contract_seqno()
    }
}

/// Checks if list of the states of **the same** contract corresponds to the state of executable (effective) contract.
/// ## Example for DAA:
/// + No terms of usage are provided: only signature of sender is needed -> only list of 2 revisions with states 1,0 is acceptable
/// + Terms of usage are provided: signatures of sender **and** receiver are needed -> only list of 3 revisions with states 2,1,0 is acceptable
///
///  See more about effectiveness of a contract here: [`Contract::is_effective`]
pub fn is_contract_effective(
    contract_and_state: impl Iterator<Item = impl ContractInfo> + Clone,
) -> Option<ContractEffect> {
    let mut iter = contract_and_state.clone();
    let realfirst = iter.next()?;
    let same_contract = iter.fold(realfirst.get_contract_data(), |acc, elem| {
        acc.and_then(|acc| (acc == elem.get_contract_data()?).then_some(acc))
    })?;

    let rev_iter = contract_and_state.map(|tuple| tuple.get_contract_seqno());
    // let tmp: Vec<_> = rev_iter.clone().collect();
    // println!("state: {:?}", tmp);
    same_contract.is_effective(rev_iter)
}

impl Contract {
    /// Extracts data needed for a contract from revision and returns a contract if it can be detected.
    pub fn from_revision(rev: &Revision) -> Option<Result<Self, ContractParseError>> {
        let (hash, transclusions, params) = contract_content(rev)?;
        let generic_contract_info = GenericContractInfo {
            hash,
            file: rev.content.file.as_ref().map(|x| x.data.as_ref()),
            transclusions,
            params,
        };
        Some(Contract::try_from(generic_contract_info))
    }

    /// Identifies the revision of the contract and returns a figure that describes possible contract state.
    /// ## Example for states of DAA:
    /// + None  => DAA is not signed by sender or receiver
    /// + 1     => DAA is signed by sender
    /// + 2     => DAA is signed by receiver
    #[deprecated = "use self.sequence_number(rev) instead"]
    pub fn identify_revision(&self, rev: &Revision) -> Option<u8> {
        self.sequence_number(rev)
    }
}

#[test]
fn ethaddr_is_webpki_dnsname() {
    let dnsname =
        webpki::types::DnsName::try_from("0x13Ddb9f9dEDE0903Ed6F40D4FB273632d19CaED1").unwrap();
    dbg!(dnsname);
}

type Transclusions<'a> = HashMap<&'a str, Hash>;
type ContractParams<'a> = HashMap<&'a str, String>;

/// A single transclusion
#[allow(dead_code)]
#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct Transclusion<'a> {
    dbkey: &'a str,
    ns: i32,
    verification_hash: Hash,
}
/// Extracts from a given revision [verification hash][`GenericContractInfo::hash`] of the contract's template,\
/// [transclusion hashes][`GenericContractInfo::transclusions`] of the pages linked to the revision and\
/// [parameters of the contract][`GenericContractInfo::params`].
fn contract_content(rev: &Revision) -> Option<(Hash, Transclusions, ContractParams)> {
    let mediawiki_text = rev.content.content.get("main")?;
    let mediawiki_text = mediawiki_text.strip_prefix("{{")?;
    let mediawiki_text = mediawiki_text.strip_suffix("\n}}")?;
    let mut item_iter = mediawiki_text.split("\n|");
    let tmp_template_name = item_iter.next()?.replace(" ", "_");
    let template_name = tmp_template_name.as_str();

    let transclusions: Vec<Transclusion> =
        serde_json::from_str(rev.content.content.get("transclusion-hashes")?).ok()?;

    let transclusion_lookup: HashMap<&str, Hash> = transclusions
        .into_iter()
        .map(
            |Transclusion {
                 dbkey,
                 verification_hash,
                 ..
             }| { (dbkey, verification_hash) },
        )
        .collect();

    let template_hash = *transclusion_lookup.get(template_name)?;

    lazy_static::lazy_static! {
        static ref REPLACE: aho_corasick::AhoCorasick = aho_corasick::AhoCorasick::new(["{{|}}", "{{(}}", "{{)}}"]).unwrap();
    };
    let map = item_iter
        .map(|item| {
            let (key, value) = item.split_once("=")?;
            let value_replaced = REPLACE.try_replace_all(value, &["|", "{", "}"]).unwrap();
            Some((key, value_replaced))
        })
        .collect::<Option<_>>()?;

    Some((template_hash, transclusion_lookup, map))
}

// #[test]
// fn a() {
//     const DATA: &str =
//         include_str!(stringify!(../tests/test_data/DAA_SIG_SENDER_NO_WIT_NO_TERM.json));

//     let rev = serde_json::from_str(DATA).unwrap();
//     let rev2 = verifier::v1_2::rev_v1_1_to_rev_v1_2(&rev, None, None);
//     dbg!(&rev2);
//     let contract = Contract::from_revision(&rev2).unwrap().unwrap();
//     dbg!(contract);
// }

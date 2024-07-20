use super::super::{hash::Hash, timestamp::Timestamp};

// todo! remove, this is an abomination
#[doc(hidden)]
fn opt_hash_de<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<Hash>, D::Error> {
    use serde::Deserialize;
    use std::str::FromStr;
    let Some(s) = <&str>::deserialize(deserializer).ok() else {
        return Ok(None);
    };
    Ok(Hash::from_str(s).ok())
}

// todo! remove, this is an abomination
#[doc(hidden)]
fn opt_hash_ser<S: serde::Serializer>(
    opt_hash: &Option<Hash>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    use serde::Serialize;
    match &opt_hash {
        Some(hash) => hash.serialize(serializer),
        None => serializer.serialize_none(),
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, Default)]
/// Contains context information for this revision 
pub struct RevisionMetadata {
    pub domain_id: String,
    pub time_stamp: Timestamp,
    // todo! remove, this is an abomination
    #[serde(deserialize_with = "opt_hash_de", serialize_with = "opt_hash_ser")]
    pub previous_verification_hash: Option<Hash>,
    pub metadata_hash: Hash,
    pub verification_hash: Hash,
}

// this is hopefully temporary. revisions do not have verification_hash on export.
/// Contains context information of the revision on export.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, Default)]
pub struct ExportRevisionMetadata {
    pub domain_id: String,
    pub time_stamp: Timestamp,
    // todo! remove, this is an abomination
    #[serde(deserialize_with = "opt_hash_de", serialize_with = "opt_hash_ser")]
    pub previous_verification_hash: Option<Hash>,
    pub metadata_hash: Hash,
    pub signature: Option<super::signature::RevisionSignature>,
    pub witness: Option<super::witness::RevisionWitness>,
}

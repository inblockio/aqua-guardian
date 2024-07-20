use super::super::{base64::Base64, hash::Hash};
use std::collections::BTreeMap;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, Default)]
/// The user visible content
pub struct RevisionContent {
    /// File in the revision. See: [`FileContent`]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<FileContent>,
    /// (key, value) map for the content `revision` -> `content`->`content` in JSON file.\
    /// Keys (i.e. `main`, `transclusion_hashes`) need to be sorted, thus using a [`BTreeMap`]
    pub content: BTreeMap<String, String>,
    /// Value of `content_hash` key of a revision in JSON file 
    pub content_hash: Hash,
}

/// The content of the file
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct FileContent {
    /// The content of the file in Base64 encoding
    pub data: Base64,
    pub filename: String,
    pub size: u32,
    pub comment: String,
}

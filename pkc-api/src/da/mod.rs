use super::*;

use guardian_common::prelude::*;

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct VerificationEntity {
    pub page_title: String,
    pub page_id: i32,
    pub rev_id: i32,
    pub domain_id: String,
    pub time_stamp: Timestamp,
    //signature
    // pub signature: Option<Signature>,
    // pub public_key: Option<PublicKey>,
    // pub wallet_address: Option<Address>,
    //witness
    // pub witness_event_id: Option<String>,
    // pub source: Option<String>,
    pub verification_hash: Hash,
    pub content_hash: Hash,
    pub metadata_hash: Hash,
    // pub signature_hash: Option<Hash>,
    pub previous_verification_hash: Option<Hash>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct LastRevision {
    #[serde(rename = "page_title")]
    pub title: String,
    pub page_id: i32,
    pub rev_id: i32,
    pub verification_hash: Hash,
}

// todo: remove once api changes are done
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct SiteInfo {
    // site_name: String,
    // dbname: String,
    // base: String,
    // generator: String,
    // case: String,
    // version: String,
    // namespaces: std::collections::BTreeMap<i32, Namespace>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct HashChainInfo {
    pub genesis_hash: Hash,
    pub domain_id: String,
    pub latest_verification_hash: Hash,
    pub title: String,
    pub namespace: i32,
    pub chain_height: i32,
    pub site_info: SiteInfo,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct HashChain {
    #[serde(flatten)]
    pub hash_chain_info: HashChainInfo,
    pub revisions: std::collections::HashMap<Hash, Revision>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ExportImportRevision {
    pub context: HashChainInfo,

    pub revision: Revision,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct UserFile {
    pub pages: Vec<HashChain>,
    #[serde(rename = "siteInfo")]
    pub site_info: SiteInfo,
}

impl Pkc {
    /// helper function for creating urls for requests to data_accounting
    fn data_accounting_url(&self, endpoint: &str) -> reqwest::Url {
        let mut url = self.url.clone();
        if let Ok(mut segments) = url.path_segments_mut() {
            segments.push("rest.php");
            segments.push("data_accounting");
            segments.push(endpoint);
        }
        url
    }
    /// helper function for other endpoints
    fn get_url(&self, endpoint: &str) -> reqwest::Url {
        let mut url = self.url.clone();
        if let Ok(mut segments) = url.path_segments_mut() {
            segments.push(endpoint);
        }
        url
    }
}

// this is down here so it's out of the way
// a macro to parse the responses from data_accounting api calls
// as they return a weird custom json on errors with more information
macro_rules! parse {
    ($($t:tt)*) => {{
        let resp = $($t)*;
        match resp.error_for_status_ref() {
            Ok(_) => {
                let text = resp.text().await?;
                // eprintln!("attempting to parse:\n{text}");
                Ok(serde_json::from_str(&text)?)
            },
            Err(e) => match serde_json::from_str(&resp.text().await?) {
                Ok(api_err) => Err(crate::error::Error::Api(api_err)),
                Err(_) => Err(crate::error::Error::Http(e)),
            }
        }
    }};
}

pub mod get_branch;
pub mod get_hash_chain_info;
pub mod get_page_all_revs;
pub mod get_page_last_rev;
pub mod get_recent_changes;
pub mod get_revision;
pub mod import;
pub mod special_export;
pub mod verify_page;

// this does not work
// pub mod push_aqua_chain;

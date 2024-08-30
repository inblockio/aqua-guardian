use crate::prelude::Revision;
use serde::{Deserialize, Serialize};

use super::hash::Hash;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NameSpace {
    case: bool,
    title: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SiteInfo {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PageData {
    pub pages: Vec<HashChain>,
    pub site_info: SiteInfo,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HashChain {
    pub genesis_hash: String,
    pub domain_id: String,
    pub title: String,
    pub namespace: u64,
    pub chain_height: u64,
    #[serde(with = "tuple_vec_map")]
    pub revisions: Vec<(Hash, Revision)>,
}

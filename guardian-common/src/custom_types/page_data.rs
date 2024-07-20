use serde::{Deserialize, Serialize};

// use super::hash_chain::HashChain;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NameSpace {
    case: bool,
    title: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SiteInfo {
    namespaces: std::collections::HashMap<i32, NameSpace>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PageData {
    pub pages: Vec<HashChain>,
    #[serde(rename = "siteInfo")]
    pub site_info: SiteInfo,
}

// todo:re move
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HashChain;

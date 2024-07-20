use std::collections::HashMap;

use guardian_common::custom_types::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct RecentChange {
    //title: String,
    //rev: i32,
    hash: Hash,
    #[serde(rename = "type")]
    kind: String,
    //touched: Timestamp,
}

/// [`da::get_revision`](self)
impl super::Pkc {
    /// ## [`/rest.php/data_accounting/recent_changes]
    ///
    /// retrieves all recent changes including new things in the inbox
    ///
    /// [pkc_api_url]: https://github.com/inblockio/mediawiki-extensions-Aqua/blob/63cbdd4047542bc75d5b43793482c3e405933b07/docs/api.yaml
    pub async fn da_get_recent_changes(
        &self,
        timestamp: Timestamp,
        deleted: bool,
    ) -> super::Result<HashMap<Hash, String>> {
        let url = self.data_accounting_url("recent_changes");

        let resp = self
            .client
            .get(url)
            .query(&[("since", &timestamp.to_string()[..])])
            .query(&[("include_deleted", &deleted.to_string()[..])])
            .send()
            .await?;
        let stuff: Vec<RecentChange> = parse!(resp)?;

        Ok(stuff
            .into_iter()
            .map(|stuff| (stuff.hash, stuff.kind))
            .collect())
    }
}

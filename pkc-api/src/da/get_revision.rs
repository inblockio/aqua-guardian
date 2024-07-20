use guardian_common::custom_types::*;

// // a temporary struct to accomodate the output of da/get_revision
// #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, Default)]
// struct ExportRevision {
//     content: RevisionContent,
//     metadata: ExportRevisionMetadata,
// }

/// [`da::get_revision`](self)
impl super::Pkc {
    /// ## [`/rest.php/data_accounting/get_revision/<revision_hash>`][pkc_api_url]
    ///
    /// retrieves the aqua-protocol [`Revision`] for a specified revision_hash (verification_hash).
    ///
    /// [pkc_api_url]: https://github.com/inblockio/mediawiki-extensions-Aqua/blob/63cbdd4047542bc75d5b43793482c3e405933b07/docs/api.yaml#L205
    pub async fn da_get_revision(&self, revision_hash: Hash) -> super::Result<Revision> {
        let mut url = self.data_accounting_url("get_revision");
        if let Ok(mut segments) = url.path_segments_mut() {
            segments.push(&revision_hash.to_stackstr());
        }
        let resp = self.client.get(url).send().await?;
        parse!(resp)
    }
}

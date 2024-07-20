use super::*;

#[derive(serde::Deserialize, Clone)]
#[allow(unused)]
pub struct Status {
    status: String,
}

/// [`da::import`](self)
impl Pkc {
    /// ## [`/rest.php/data_accounting/import`][pkc_api_url]
    ///
    /// inserts a single [`Revision`] into the user's "Inbox".
    ///
    /// [pkc_api_url]: https://github.com/inblockio/mediawiki-extensions-Aqua/blob/63cbdd4047542bc75d5b43793482c3e405933b07/docs/api.yaml#L450
    pub async fn da_import(&self, revision: ExportImportRevision) -> super::Result<Status> {
        let resp = self
            .client
            .post(self.data_accounting_url("import"))
            .query(&[("direct", "false")])
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(serde_json::to_string(&revision)?)
            .send()
            .await?;

        parse!(resp)
    }

    pub async fn da_import_bypass(&self, revision: ExportImportRevision) -> super::Result<Status> {
        let resp = self
            .client
            .post(self.data_accounting_url("import"))
            .query(&[("direct", "true")])
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(serde_json::to_string(&revision)?)
            .send()
            .await?;

        parse!(resp)
    }

    /// simple wrapper around [`da_import`](crate::Pkc::da_import) that checks the returned status
    pub async fn post_revision(&self, revision: ExportImportRevision) -> super::Result<()> {
        let import_status = self.da_import(revision).await?;
        match &import_status.status[..] {
            "ok" => Ok(()),
            other => Err(error::Error::Other(format!(
                "import status not `ok`: {other}"
            ))),
        }
    }
}

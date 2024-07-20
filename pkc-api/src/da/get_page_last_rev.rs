use super::*;

/// [`da::get_page_last_rev`](self)
impl super::Pkc {
    /// ## [`/rest.php/data_accounting/get_page_last_rev?page_title=<title>`][pkc_api_url]
    ///
    /// retrieves [`LastRevision`] of the page with the specified title
    ///
    /// # Errors
    ///
    /// - [`ApiError`](error::Error::Api) with `404 not found` when no page with the given title was found
    ///
    /// [pkc_api_url]: https://github.com/inblockio/mediawiki-extensions-Aqua/blob/63cbdd4047542bc75d5b43793482c3e405933b07/docs/api.yaml#L92
    pub async fn da_get_page_last_rev(&self, title: &str) -> super::Result<LastRevision> {
        let url = self.data_accounting_url("get_page_last_rev");

        let resp = self
            .client
            .get(url)
            .query(&[("page_title", title)])
            .send()
            .await?;

        parse!(resp)
    }
    /// retrieves the verification_hash of the newest revision of the page with the title
    ///
    /// uses [`da_get_page_last_rev`](crate::Pkc::da_get_page_last_rev)
    pub async fn latest_hash_for_title(&self, title: &str) -> super::Result<Hash> {
        let resp = self.da_get_page_last_rev(title).await?;
        Ok(resp.verification_hash)
    }
}

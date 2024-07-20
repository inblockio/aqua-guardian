//! functions for endpoint `/rest.php/data_accounting/get_page_all_revs`
//!
//! # [`da_get_page_all_revs(title)`](super::Pkc::da_get_page_all_revs)
//!
//! mediawiki `rev_id`s of all revisions of a page
//!
//! # [`da_get_page_all_revs_full(title)`](crate::Pkc::da_get_page_all_revs_full)
//!
//! [`VerificationEntity`]s of all revisions of a page

use super::*;

/// [`da::get_page_all_revs`](self)
impl super::Pkc {
    /// ## [`/rest.php/data_accounting/get_page_all_revs/<title>`][pkc_api_url]
    ///
    /// retrieves the MediaWiki `rev_id`s of all revisions of a page, in order from first to last created
    ///
    /// # Errors
    ///
    /// - [`ApiError`](error::Error::Api) with `404 Invalid ID supplied`
    ///   when the given title does not exist
    ///
    /// [pkc_api_url]: https://github.com/inblockio/mediawiki-extensions-Aqua/blob/63cbdd4047542bc75d5b43793482c3e405933b07/docs/api.yaml#L55
    pub async fn da_get_page_all_revs(&self, title: &str) -> super::Result<Vec<usize>> {
        let mut url = self.data_accounting_url("get_page_all_revs");

        if let Ok(mut segments) = url.path_segments_mut() {
            segments.push(title);
        }

        let resp = self.client.get(url).send().await?;
        // todo! does this work?
        parse!(resp) // there is no way this will parse correctly
    }

    /// ## [`/rest.php/data_accounting/get_page_all_revs/<title>?full_entities=true`][pkc_api_url]
    ///
    /// retrieves the [`VerificationEntity`]s of all revisions of a page, in order from first to last created
    ///
    /// # Errors
    ///
    /// - [`ApiError`](error::Error::Api) with `404 Invalid ID supplied`
    ///   when the given title does not exist
    ///
    /// [pkc_api_url]: https://github.com/inblockio/mediawiki-extensions-Aqua/blob/63cbdd4047542bc75d5b43793482c3e405933b07/docs/api.yaml#L55
    pub async fn da_get_page_all_revs_full(
        &self,
        page: &str,
    ) -> super::Result<Vec<VerificationEntity>> {
        let mut url = self.data_accounting_url("get_page_all_revs");
        if let Ok(mut segments) = url.path_segments_mut() {
            segments.push(page);
        }

        let resp = self
            .client
            .get(url)
            .query(&[("full_entities", "true")])
            .send()
            .await?;

        // todo!("parse this properly (regex)");
        // parse!(resp) // there is no way this will parse correctly
        todo!("this does not work, please FIX {:?}", resp);
    }
}

use super::*;

/// [`da::get_hash_chain_info`](self)
impl super::Pkc {
    /// ## [`/rest.php/data_accounting/get_hash_chain_info/genesis_hash?identifier=<genesis_hash>`][pkc_api_url]
    ///
    /// retrieves the [`HashChainInfo`] for a genesis_hash
    ///
    /// # Caveats
    ///
    /// as genesis_hash is not a useful identifier, this actually just looks up the newest page
    /// which has this genesis_hash and outputs information about that.
    ///
    /// # Errors
    ///
    /// - [`ApiError`](error::Error::Api) with `404 not found` when the given hash was not a (known) genesis hash
    ///
    /// [pkc_api_url]: https://github.com/inblockio/mediawiki-extensions-Aqua/blob/63cbdd4047542bc75d5b43793482c3e405933b07/docs/api.yaml#L228
    pub async fn da_get_hash_chain_info_by_genesis_hash(
        &self,
        genesis_hash: Hash,
    ) -> super::Result<HashChainInfo> {
        let mut url = self.data_accounting_url("get_hash_chain_info");

        if let Ok(mut segments) = url.path_segments_mut() {
            segments.push("genesis_hash");
        }

        let resp = self
            .client
            .get(url)
            .query(&[("identifier", &genesis_hash.to_stackstr()[..])])
            .send()
            .await?;

        parse!(resp)
    }

    /// ## [`/rest.php/data_accounting/get_hash_chain_info/title?identifier=<title>`][pkc_api_url]
    ///
    /// retrieves the [`HashChainInfo`] of the page with the specified title
    ///
    /// # Errors
    ///
    /// - [`ApiError`](error::Error::Api) with `404 not found` when no page with the given title was found
    ///
    /// [pkc_api_url]: https://github.com/inblockio/mediawiki-extensions-Aqua/blob/63cbdd4047542bc75d5b43793482c3e405933b07/docs/api.yaml#L228
    pub async fn da_get_hash_chain_info_by_title(
        &self,
        title: &str,
    ) -> super::Result<HashChainInfo> {
        let mut url = self.data_accounting_url("get_hash_chain_info");

        if let Ok(mut segments) = url.path_segments_mut() {
            segments.push("title");
        }

        let resp = self
            .client
            .get(url)
            .query(&[("identifier", title)])
            .send()
            .await?;

        parse!(resp)
    }
}

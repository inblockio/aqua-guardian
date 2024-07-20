use super::*;

/// [`da::verify_page`](self)
impl super::Pkc {
    pub async fn da_verify_page(&self, id: usize) -> super::Result<VerificationEntity> {
        let mut url = self.data_accounting_url("verify_page");
        if let Ok(mut segments) = url.path_segments_mut() {
            segments.push(&id.to_string()[..]);
        }
        let resp = self.client.get(url).send().await?;
        // #TODO make parse work.
        parse!(resp)
    }

    pub async fn revision_hash_for_rev_id(&self, id: usize) -> super::Result<Hash> {
        Ok(self.da_verify_page(id).await?.verification_hash)
    }
}

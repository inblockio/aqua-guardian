#[derive(serde::Deserialize)]
struct Query {
    allpages: Vec<PageInfo>,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct PageInfo {
    #[serde(rename = "pageid")]
    pub id: usize,
    pub ns: usize,
    pub title: String,
}

impl super::Pkc {
    /// an api endpoint for getting information about all pages
    ///
    /// it uses the api <https://www.mediawiki.org/wiki/API:Allpages> to achieve this, hence the name.
    pub async fn mw_allpages(&self) -> super::Result<Vec<PageInfo>> {
        let resp = self
            .client
            .get(self.mw_api_url())
            .query(&[
                ("action", "query"),
                ("list", "allpages"),
                ("aplimit", "max"),
                ("format", "json"),
            ])
            .send()
            .await?;

        let resp: super::Response<Query> = parse!(resp)?;

        // there is no way this will parse correctly
        // yes there is
        Ok(resp.query.allpages)
    }
}

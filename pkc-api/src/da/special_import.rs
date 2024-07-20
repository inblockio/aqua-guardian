use serde::Serialize;

#[derive(Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OfflineData {
    pub pages: Vec<guardian_common::custom_types::HashChain>,
    pub site_info: guardian_common::custom_types::SiteInfo,
}
impl super::Pkc {
    pub async fn post_aqua_chain(&self, aqua_chain: OfflineData) -> super::Result<String> {
        // this is one of the MW APIs, hence get_url.
        let mut url = self.get_url("index.php");
        if let Ok(mut segments) = url.path_segments_mut() {
            segments.push("Special:Verified_Import");
        }

        // do_req is not suited for http post, hence this.
        let builder = self
            .client
            .request(reqwest::Method::POST, url)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(serde_json::to_string(&aqua_chain).unwrap());

        let resp = builder.send().await.unwrap();
        parse!(resp)
    }
}

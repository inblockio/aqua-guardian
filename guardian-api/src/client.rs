/// Stores data required for the receiving guardian to run
pub struct GuardianClient<Context> {
    /// Host URL the server is on
    url: reqwest::Url,
    /// Client connection data
    client: reqwest::Client,
    ctx_marker: std::marker::PhantomData<Context>,
}
#[derive(thiserror::Error, Debug)]
/// Client Error types (request)
pub enum ClientError {
    /// Request API was unhappy
    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
    /// Invalid JSON (propably)
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}
/// Implementation of clientside functions
impl<Ctx> GuardianClient<Ctx> {
    async fn do_req<T: for<'a> serde::Deserialize<'a>>(
        &self,
        req: reqwest::Request,
    ) -> Result<T, ClientError> {
        let resp = self.client.execute(req).await?;
        let t = resp.text().await?;
        // dbg!(&t);
        Ok(serde_json::from_str(&t)?)
    }
}

/// Defines connection interface betweeen two guardians. See ApiHandler
impl<Context: for<'de> serde::Deserialize<'de> + Sync> super::ApiClient
    for GuardianClient<Context>
{
    type ConnInfo = super::ConnInfo;
    type Error = ClientError;
    type Context = Context;

    /// Create a connection to store client infos
    async fn new(conn: crate::ConnInfo) -> Result<Self, Self::Error> {
        let client = reqwest::Client::builder()
            .add_root_certificate(conn.cert)
            .identity(conn.identity)
            .redirect(reqwest::redirect::Policy::none())
            .build()?;

        Ok(Self {
            url: conn.url,
            client,
            ctx_marker: std::marker::PhantomData,
        })
    }

    /// List all available aqua chains (may return none)
    async fn list(
        &self,
    ) -> Result<std::collections::HashSet<guardian_common::prelude::Hash>, Self::Error> {
        // println!("Here's your listing");
        let mut list_url = self.url.clone();
        if let Ok(mut path_mut) = list_url.path_segments_mut() {
            path_mut.push("list");
        }
        let req = reqwest::Request::new(reqwest::Method::GET, list_url);
        self.do_req(req).await
    }

    /// Returns the specified branch back to the genesis hash
    async fn get_branch(
        &self,
        hash: guardian_common::prelude::Hash,
    ) -> Result<guardian_common::custom_types::Branch<Context>, Self::Error> {
        let mut get_branch_url = self.url.clone();
        if let Ok(mut path_mut) = get_branch_url.path_segments_mut() {
            path_mut.push("get_branch");
        }
        get_branch_url
            .query_pairs_mut()
            .append_pair("hash", &hash.to_stackstr());
        let req = reqwest::Request::new(reqwest::Method::GET, get_branch_url);
        self.do_req(req).await
    }

    /// Returns only the specified revision
    async fn get_revision(
        &self,
        hash: guardian_common::prelude::Hash,
    ) -> Result<guardian_common::prelude::Revision, Self::Error> {
        // println!("I'm getting your revision ready");
        let mut get_revision_url = self.url.clone();
        if let Ok(mut path_mut) = get_revision_url.path_segments_mut() {
            path_mut.push("get_revision");
        }
        // println!("valid path");
        get_revision_url
            .query_pairs_mut()
            .append_pair("hash", &hash.to_stackstr());
        // println!("questioning server");
        let req = reqwest::Request::new(reqwest::Method::GET, get_revision_url);
        self.do_req(req).await
    }
}

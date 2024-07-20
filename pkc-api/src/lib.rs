pub mod da;
pub mod error;
pub mod mw;
pub mod storage;

// aliases for convenience
pub use da as data_accounting;
pub use mw as mediawiki;

pub mod prelude {
    pub use super::Pkc;
    pub use guardian_common::storage::*;
}

pub type Result<T> = std::result::Result<T, error::Error>;

/// Client for [Aqua-PKC](todo link) [implementing](crate::Pkc#impl-Storage-for-Pkc) the [`Storage`](guardian_common::storage::Storage) trait
///
/// It implements necessary APIs from the [DataAccounting MediaWiki extension](https://github.com/inblockio/mediawiki-extensions-Aqua)
/// and [MediaWiki](https://mediawiki.org) itself. They are however not meant to be used directly, instead one should only rely on the
/// [`Storage`](guardian_common::storage::Storage) trait defined in [`guardian_common`].
///
/// Authentication to the PKC is done by accessing the user APIs of siwe-oidc to retrieve the cookies for login. This functionality is contained in `siwe-oidc-auth` crate
///
/// todo: link, import and use in new function taking signer
///
/// # Example
///
/// todo!
#[derive(Debug, Clone)]
pub struct Pkc {
    creation: chrono::NaiveDateTime,
    url: reqwest::Url,
    client: reqwest::Client,
}

impl Pkc {
    pub fn new(url: impl reqwest::IntoUrl) -> reqwest::Result<Self> {
        Ok(Pkc::new_with_options(
            chrono::Utc::now().naive_utc(),
            url.into_url()?,
            reqwest::Client::new(),
        ))
    }
    pub fn new_with_options(
        creation: chrono::NaiveDateTime,
        url: reqwest::Url,
        client: reqwest::Client,
    ) -> Self {
        Pkc {
            creation,
            url,
            client,
        }
    }
}

#[test]
fn exercise_pkc() {
    let _ = Pkc::new("localhost:9352");
    let _ = Pkc::new_with_options(
        chrono::Utc::now().naive_utc(),
        "localhost:9352".parse().unwrap(),
        reqwest::Client::default(),
    );
}

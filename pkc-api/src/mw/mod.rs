//! MediaWiki APIs ([documentation](https://www.mediawiki.org/wiki/API:Main_page))
//!
//! - [`mw_recent_changes`](crate::Pkc::mw_recent_changes)
//! - [`mw_allpages`](crate::Pkc::mw_allpages)
//!
//! usually on path /api.php
//!
//! for when the [data_accounting apis][super::da] are not sufficient.

use super::*;

/// MediaWiki Response
///
/// this is returned by all queries to the mediawiki api, just
/// what exactly query is changes based on arguments and is thus
/// a generic to be defined in each function itself
#[derive(serde::Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Response<Query> {
    query: Query,
}

impl Pkc {
    /// helper function for creating urls for requests to mediawiki
    fn mw_api_url(&self) -> reqwest::Url {
        let mut url = self.url.clone();
        if let Ok(mut segments) = url.path_segments_mut() {
            segments.push("api.php");
        }
        url
    }
}

// the mediawiki api probably has more normal or at least
// definitely different error types than data_accounting.
// as such (and for code organization reasons, here is a
// different) parse! macro just for mediawiki
macro_rules! parse {
    ($($t:tt)*) => {{
        let resp = $($t)*;
        match resp.error_for_status_ref() {
            Ok(_) => {
                let text = resp.text().await?;
                Ok(serde_json::from_str(&text)?)
            },
            Err(e) => Err(crate::error::Error::Http(e)),
        }
    }};
}

pub mod allpages;
pub mod recent_changes;

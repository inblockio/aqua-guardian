use super::*;

#[derive(serde::Deserialize)]
struct RecentChange {
    #[serde(rename = "type")]
    action: String,
    revid: i32,
    title: String,
}
#[derive(serde::Deserialize)]
struct Query {
    recentchanges: Vec<RecentChange>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ChangeAction {
    New,
    Edit,
    Other(String),
}

impl From<String> for ChangeAction {
    fn from(value: String) -> Self {
        use ChangeAction::*;
        match &value[..] {
            "new" => New,
            "edit" => Edit,
            _ => Other(value),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChangeInfo {
    pub action: ChangeAction,
    pub rev_id: i32,
}

impl Pkc {
    pub async fn mw_recent_changes(
        &self,
        since: chrono::NaiveDateTime,
    ) -> Result<std::collections::HashMap<String, ChangeInfo>> {
        let resp = self
            .client
            .get(self.mw_api_url())
            .query(&[
                ("action", "query"),
                ("list", "recentchanges"),
                ("formatversion", "2"),
                ("rcprop", "title|loginfo|ids"),
                (
                    "rcend",
                    &since
                        .and_utc()
                        .to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                ),
                ("format", "json"),
            ])
            .send()
            .await?;
        let res: super::Response<Query> = parse!(resp)?;
        Ok(res
            .query
            .recentchanges
            .into_iter()
            .map(|recent| {
                let info = ChangeInfo {
                    action: ChangeAction::from(recent.action),
                    rev_id: recent.revid,
                };
                (recent.title, info)
            })
            .collect())
    }
}

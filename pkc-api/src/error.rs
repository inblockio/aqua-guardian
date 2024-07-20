#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json parsing error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("api error: {0}")]
    Api(#[from] ApiError),
    #[error("{0}")]
    Other(String),
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ApiError {
    #[serde(default)]
    message_translations: std::collections::HashMap<String, String>,
    http_code: u32,
    http_reason: String,
}
impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "Api Error [{}: '{}']",
            self.http_code, self.http_reason
        ))
    }
}
impl std::error::Error for ApiError {}

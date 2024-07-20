use serde::{Deserialize, Serialize};
/// Namespace used in a `page` of Aqua-Chain
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Namespace {
    pub case: bool,
    pub title: String,
}

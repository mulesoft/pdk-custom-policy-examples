use serde::Deserialize;
#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    #[serde(alias = "authorization")]
    pub authorization: String,
    #[serde(alias = "host")]
    pub host: String,
    #[serde(alias = "path")]
    pub path: String,
    #[serde(alias = "tokenExtractor")]
    pub token_extractor: pdk::api::expression::Expression,
    #[serde(alias = "upstream")]
    pub upstream: String,
}

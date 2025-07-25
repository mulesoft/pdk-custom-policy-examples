use serde::Deserialize;
#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    #[serde(alias = "max_retries")]
    pub max_retries: i64,
    #[serde(alias = "namespace")]
    pub namespace: String,
    #[serde(alias = "storage_type")]
    pub storage_type: String,
    #[serde(alias = "ttl_seconds")]
    pub ttl_seconds: i64,
}

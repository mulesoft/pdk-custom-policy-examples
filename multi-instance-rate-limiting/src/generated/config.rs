use serde::Deserialize;
#[derive(Deserialize, Clone, Debug)]
pub struct RateLimits0Config {
    #[serde(alias = "group_name")]
    pub group_name: String,
    #[serde(alias = "key_selector")]
    pub key_selector: String,
    #[serde(alias = "requests_per_window")]
    pub requests_per_window: i64,
    #[serde(alias = "window_size_seconds")]
    pub window_size_seconds: i64,
}
#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    #[serde(alias = "rate_limits")]
    pub rate_limits: Vec<RateLimits0Config>,
    #[serde(alias = "shared_storage")]
    pub shared_storage: Option<String>,
}

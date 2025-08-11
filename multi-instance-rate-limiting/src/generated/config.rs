use serde::Deserialize;
#[derive(Deserialize, Clone, Debug)]
pub struct ApiKeyRateLimitConfig {
    #[serde(alias = "group_name")]
    pub group_name: Option<String>,
    #[serde(alias = "requests_per_window")]
    pub requests_per_window: i64,
    #[serde(alias = "window_size_seconds")]
    pub window_size_seconds: i64,
}
#[derive(Deserialize, Clone, Debug)]
pub struct UserIdRateLimitConfig {
    #[serde(alias = "group_name")]
    pub group_name: Option<String>,
    #[serde(alias = "requests_per_window")]
    pub requests_per_window: i64,
    #[serde(alias = "window_size_seconds")]
    pub window_size_seconds: i64,
}
#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    #[serde(alias = "api_key_rate_limit")]
    pub api_key_rate_limit: ApiKeyRateLimitConfig,
    #[serde(alias = "user_id_rate_limit")]
    pub user_id_rate_limit: UserIdRateLimitConfig,
}

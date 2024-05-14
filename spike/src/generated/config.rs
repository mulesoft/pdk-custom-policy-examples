use serde::Deserialize;
#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    #[serde(alias = "delay")]
    pub delay: i64,
    #[serde(alias = "maxAttempts")]
    pub max_attempts: i64,
    #[serde(alias = "millis")]
    pub millis: i64,
    #[serde(alias = "requests")]
    pub requests: i64,
}

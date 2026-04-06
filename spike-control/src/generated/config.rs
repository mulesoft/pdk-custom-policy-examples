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
#[pdk::hl::entrypoint_flex]
fn init(abi: &dyn pdk::flex_abi::api::FlexAbi) -> Result<(), anyhow::Error> {
    abi.setup()?;
    Ok(())
}

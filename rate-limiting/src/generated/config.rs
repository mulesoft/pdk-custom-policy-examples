use serde::Deserialize;
#[derive(Deserialize, Clone, Debug)]
pub struct RateLimits0Config {
    #[serde(alias = "maximumRequests")]
    pub maximum_requests: f64,
    #[serde(alias = "timePeriodInMilliseconds")]
    pub time_period_in_milliseconds: f64,
}
#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    #[serde(alias = "rateLimits")]
    pub rate_limits: Vec<RateLimits0Config>,
}
#[pdk::hl::entrypoint_flex]
fn init(abi: &dyn pdk::flex_abi::api::FlexAbi) -> Result<(), anyhow::Error> {
    abi.setup()?;
    Ok(())
}

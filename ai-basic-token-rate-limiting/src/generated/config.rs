use serde::Deserialize;
#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    #[serde(alias = "maximumTokens")]
    pub maximum_tokens: i64,
    #[serde(alias = "timePeriodInMilliseconds")]
    pub time_period_in_milliseconds: i64,
}
#[pdk::hl::entrypoint_flex]
fn init(abi: &dyn pdk::flex_abi::api::FlexAbi) -> Result<(), anyhow::Error> {
    abi.setup()?;
    Ok(())
}

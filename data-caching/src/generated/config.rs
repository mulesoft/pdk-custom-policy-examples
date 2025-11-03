use serde::Deserialize;
#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    #[serde(alias = "end_hour")]
    pub end_hour: i64,
    #[serde(alias = "max_cached_values")]
    pub max_cached_values: i64,
    #[serde(alias = "start_hour")]
    pub start_hour: i64,
}
#[pdk::hl::entrypoint_flex]
fn init(abi: &dyn pdk::flex_abi::api::FlexAbi) -> Result<(), anyhow::Error> {
    abi.setup()?;
    Ok(())
}

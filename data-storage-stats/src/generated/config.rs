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
#[pdk::hl::entrypoint_flex]
fn init(abi: &dyn pdk::flex_abi::api::FlexAbi) -> Result<(), anyhow::Error> {
    abi.setup()?;
    Ok(())
}

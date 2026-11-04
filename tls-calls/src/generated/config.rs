use serde::Deserialize;
#[derive(Deserialize, Clone, Debug)]
pub struct ServiceConfig {
    #[serde(alias = "name")]
    pub name: String,
    #[serde(alias = "namespace")]
    pub namespace: String,
    #[serde(alias = "url")]
    pub url: String,
}
#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    #[serde(alias = "service")]
    pub service: ServiceConfig,
}
#[pdk::hl::entrypoint_flex]
fn init(abi: &dyn pdk::flex_abi::api::FlexAbi) -> Result<(), anyhow::Error> {
    abi.setup()?;
    Ok(())
}

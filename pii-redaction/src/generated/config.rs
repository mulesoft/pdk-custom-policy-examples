use serde::Deserialize;
#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    #[serde(alias = "maskFields")]
    pub mask_fields: Vec<String>,
    #[serde(alias = "maskWith")]
    pub mask_with: String,
    #[serde(alias = "patterns")]
    pub patterns: Vec<String>,
    #[serde(alias = "sensitiveHeaders")]
    pub sensitive_headers: Vec<String>,
}
#[pdk::hl::entrypoint_flex]
fn init(abi: &dyn pdk::flex_abi::api::FlexAbi) -> Result<(), anyhow::Error> {
    abi.setup()?;
    Ok(())
}

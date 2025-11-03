use serde::Deserialize;
#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    #[serde(alias = "cardPath")]
    pub card_path: String,
    #[serde(alias = "consumerUrl")]
    pub consumer_url: Option<String>,
    #[serde(alias = "verifySchema")]
    pub verify_schema: bool,
}
#[pdk::hl::entrypoint_flex]
fn init(abi: &dyn pdk::flex_abi::api::FlexAbi) -> Result<(), anyhow::Error> {
    abi.setup()?;
    Ok(())
}

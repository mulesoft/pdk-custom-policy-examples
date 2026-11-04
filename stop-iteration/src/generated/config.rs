use serde::Deserialize;
#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    #[serde(alias = "bodyPrefix")]
    pub body_prefix: String,
    #[serde(alias = "modifyRequest")]
    pub modify_request: bool,
    #[serde(alias = "modifyResponse")]
    pub modify_response: bool,
}
#[pdk::hl::entrypoint_flex]
fn init(abi: &dyn pdk::flex_abi::api::FlexAbi) -> Result<(), anyhow::Error> {
    abi.setup()?;
    Ok(())
}

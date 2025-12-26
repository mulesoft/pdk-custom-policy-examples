use serde::Deserialize;
#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    #[serde(alias = "jku")]
    pub jku: String,
    #[serde(alias = "kid")]
    pub kid: String,
    #[serde(alias = "privateKey")]
    pub private_key: String,
}
#[pdk::hl::entrypoint_flex]
fn init(abi: &dyn pdk::flex_abi::api::FlexAbi) -> Result<(), anyhow::Error> {
    abi.setup()?;
    Ok(())
}

use serde::Deserialize;
#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    #[serde(alias = "ipHeader")]
    pub ip_header: String,
    #[serde(alias = "ipsAllowed")]
    pub ips_allowed: Option<Vec<String>>,
    #[serde(alias = "ipsBlocked")]
    pub ips_blocked: Option<Vec<String>>,
}
#[pdk::hl::entrypoint_flex]
fn init(abi: &dyn pdk::flex_abi::api::FlexAbi) -> Result<(), anyhow::Error> {
    abi.setup()?;
    Ok(())
}

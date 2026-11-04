use serde::Deserialize;
#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    #[serde(alias = "maxArrayLength")]
    pub max_array_length: Option<i64>,
    #[serde(alias = "maxDepth")]
    pub max_depth: Option<i64>,
    #[serde(alias = "maxKeyLength")]
    pub max_key_length: Option<i64>,
    #[serde(alias = "maxObjectEntries")]
    pub max_object_entries: Option<i64>,
    #[serde(alias = "maxStringLength")]
    pub max_string_length: Option<i64>,
}
#[pdk::hl::entrypoint_flex]
fn init(abi: &dyn pdk::flex_abi::api::FlexAbi) -> Result<(), anyhow::Error> {
    abi.setup()?;
    Ok(())
}

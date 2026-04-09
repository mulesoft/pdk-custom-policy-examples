use serde::Deserialize;
#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    #[serde(alias = "maxAttributeCount")]
    pub max_attribute_count: Option<i64>,
    #[serde(alias = "maxAttributeLength")]
    pub max_attribute_length: Option<i64>,
    #[serde(alias = "maxChildCount")]
    pub max_child_count: Option<i64>,
    #[serde(alias = "maxCommentLength")]
    pub max_comment_length: Option<i64>,
    #[serde(alias = "maxDepth")]
    pub max_depth: Option<i64>,
    #[serde(alias = "maxTextLength")]
    pub max_text_length: Option<i64>,
}
#[pdk::hl::entrypoint_flex]
fn init(abi: &dyn pdk::flex_abi::api::FlexAbi) -> Result<(), anyhow::Error> {
    abi.setup()?;
    Ok(())
}

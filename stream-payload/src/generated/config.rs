use serde::Deserialize;
#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    #[serde(alias = "forbiddenStrings")]
    pub forbidden_strings: Vec<String>,
    #[serde(alias = "searchMode")]
    pub search_mode: String,
}

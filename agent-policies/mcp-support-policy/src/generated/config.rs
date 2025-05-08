use serde::Deserialize;
#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    #[serde(alias = "baseUrl")]
    pub base_url: Option<String>,
}

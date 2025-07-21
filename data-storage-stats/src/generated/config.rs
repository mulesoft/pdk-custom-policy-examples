use serde::Deserialize;
#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    #[serde(alias = "namespace")]
    pub namespace: Option<String>,
}

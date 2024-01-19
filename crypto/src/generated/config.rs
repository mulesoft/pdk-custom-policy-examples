use serde::Deserialize;
#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    #[serde(alias = "aes_key")]
    pub aes_key: String,
    #[serde(alias = "rsa_key")]
    pub rsa_key: String,
}

use serde::Deserialize;
#[derive(Deserialize, Clone, Debug)]
pub struct Append0Config {
    #[serde(alias = "content")]
    pub content: String,
    #[serde(alias = "role")]
    pub role: String,
}
#[derive(Deserialize, Clone, Debug)]
pub struct Prepend0Config {
    #[serde(alias = "content")]
    pub content: String,
    #[serde(alias = "role")]
    pub role: String,
}
#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    #[serde(alias = "append")]
    pub append: Vec<Append0Config>,
    #[serde(alias = "prepend")]
    pub prepend: Vec<Prepend0Config>,
}

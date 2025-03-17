use serde::Deserialize;
#[derive(Deserialize, Clone, Debug)]
pub struct Filters0Config {
    #[serde(alias = "omitInsteadOfBlocking")]
    pub omit_instead_of_blocking: bool,
    #[serde(alias = "pattern")]
    pub pattern: String,
}
#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    #[serde(alias = "filters")]
    pub filters: Vec<Filters0Config>,
}

use serde::Deserialize;
#[derive(Deserialize, Clone, Debug)]
pub struct Templates0Config {
    #[serde(alias = "name")]
    pub name: String,
    #[serde(alias = "template")]
    pub template: String,
}
#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    #[serde(alias = "allowUntemplatedRequests")]
    pub allow_untemplated_requests: bool,
    #[serde(alias = "templates")]
    pub templates: Vec<Templates0Config>,
}

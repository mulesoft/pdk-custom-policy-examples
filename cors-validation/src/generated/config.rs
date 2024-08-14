use serde::Deserialize;
#[derive(Deserialize, Clone, Debug)]
pub struct AllowedMethods0Config {
    #[serde(alias = "allowed")]
    pub allowed: bool,
    #[serde(alias = "methodName")]
    pub method_name: String,
}
#[derive(Deserialize, Clone, Debug)]
pub struct OriginGroups0Config {
    #[serde(alias = "accessControlMaxAge")]
    pub access_control_max_age: f64,
    #[serde(alias = "allowedMethods")]
    pub allowed_methods: Vec<AllowedMethods0Config>,
    #[serde(alias = "exposedHeaders")]
    pub exposed_headers: Vec<String>,
    #[serde(alias = "headers")]
    pub headers: Vec<String>,
    #[serde(alias = "name")]
    pub name: String,
    #[serde(alias = "origins")]
    pub origins: Vec<String>,
}
#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    #[serde(alias = "originGroups")]
    pub origin_groups: Vec<OriginGroups0Config>,
    #[serde(alias = "publicResource")]
    pub public_resource: bool,
    #[serde(alias = "supportCredentials")]
    pub support_credentials: bool,
}

use serde::Deserialize;
#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    #[serde(alias = "authenticationTimeout")]
    pub authentication_timeout: f64,
    #[serde(alias = "authorizationValue")]
    pub authorization_value: String,
    #[serde(alias = "expiresInAttribute")]
    pub expires_in_attribute: String,
    #[serde(alias = "exposeHeaders")]
    pub expose_headers: bool,
    #[serde(alias = "introspectionPath")]
    pub introspection_path: String,
    #[serde(
        alias = "introspectionService",
        deserialize_with = "pdk::serde::deserialize_service"
    )]
    pub introspection_service: pdk::hl::Service,
    #[serde(alias = "maxCacheEntries")]
    pub max_cache_entries: f64,
    #[serde(alias = "scopeValidationCriteria")]
    pub scope_validation_criteria: String,
    #[serde(alias = "scopes")]
    pub scopes: Option<String>,
    #[serde(alias = "validatedTokenTTL")]
    pub validated_token_ttl: f64,
}
#[pdk::hl::entrypoint_flex]
fn init(abi: &dyn pdk::flex_abi::api::FlexAbi) -> Result<(), anyhow::Error> {
    let config: Config = serde_json::from_slice(abi.get_configuration())
        .map_err(|err| {
            anyhow::anyhow!(
                "Failed to parse configuration '{}'. Cause: {}",
                String::from_utf8_lossy(abi.get_configuration()), err
            )
        })?;
    abi.service_create(config.introspection_service)?;
    abi.setup()?;
    Ok(())
}

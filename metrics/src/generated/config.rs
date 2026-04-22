use serde::Deserialize;
#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    #[serde(alias = "maxRetries")]
    pub max_retries: Option<i64>,
    #[serde(
        alias = "metricsSink",
        deserialize_with = "pdk::serde::deserialize_service"
    )]
    pub metrics_sink: pdk::hl::Service,
    #[serde(alias = "pushFrequency")]
    pub push_frequency: i64,
}
#[pdk::hl::entrypoint_flex]
fn init(abi: &dyn pdk::flex_abi::api::FlexAbi) -> Result<(), anyhow::Error> {
    let config: Config = serde_json::from_slice(abi.get_configuration()).map_err(|err| {
        anyhow::anyhow!(
            "Failed to parse configuration '{}'. Cause: {}",
            String::from_utf8_lossy(abi.get_configuration()),
            err
        )
    })?;
    abi.service_create(config.metrics_sink)?;
    abi.setup()?;
    Ok(())
}

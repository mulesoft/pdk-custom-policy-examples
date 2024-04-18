use serde::Deserialize;
#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    #[serde(alias = "frequency")]
    pub frequency: i64,
    #[serde(alias = "ip", deserialize_with = "de_ip_0")]
    pub ip: pdk::script::Script,
    #[serde(alias = "source", deserialize_with = "pdk::serde::deserialize_service")]
    pub source: pdk::hl::Service,
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
    abi.service_create(config.source)?;
    Ok(())
}
fn de_ip_0<'de, D>(deserializer: D) -> Result<pdk::script::Script, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    let exp: pdk::script::Expression = serde::de::Deserialize::deserialize(
        deserializer,
    )?;
    pdk::script::ScriptingEngine::script(&exp)
        .input(pdk::script::Input::Attributes)
        .input(pdk::script::Input::Payload(pdk::script::Format::PlainText))
        .compile()
        .map_err(serde::de::Error::custom)
}

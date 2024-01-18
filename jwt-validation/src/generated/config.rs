use serde::Deserialize;
#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    #[serde(alias = "customRule", deserialize_with = "de_custom_rule_0")]
    pub custom_rule: pdk::script::Script,
    #[serde(alias = "secret")]
    pub secret: String,
}
fn de_custom_rule_0<'de, D>(deserializer: D) -> Result<pdk::script::Script, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    let exp: pdk::script::Expression = serde::de::Deserialize::deserialize(
        deserializer,
    )?;
    pdk::script::ScriptingEngine::script(&exp)
        .input(pdk::script::Input::Attributes)
        .input(pdk::script::Input::Payload(pdk::script::Format::PlainText))
        .input(pdk::script::Input::Vars("claimSet"))
        .compile()
        .map_err(serde::de::Error::custom)
}

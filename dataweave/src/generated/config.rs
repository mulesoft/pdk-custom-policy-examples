use serde::Deserialize;
#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    #[serde(
        alias = "exampleDateweaveProperty",
        deserialize_with = "de_example_dateweave_property_0"
    )]
    pub example_dateweave_property: pdk::script::Script,
}
fn de_example_dateweave_property_0<'de, D>(
    deserializer: D,
) -> Result<pdk::script::Script, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    let exp: pdk::script::Expression = serde::de::Deserialize::deserialize(
        deserializer,
    )?;
    pdk::script::ScriptingEngine::script(&exp)
        .input(pdk::script::Input::Attributes)
        .input(pdk::script::Input::Authentication)
        .input(pdk::script::Input::Payload(pdk::script::Format::Json))
        .input(pdk::script::Input::Payload(pdk::script::Format::Xml))
        .input(pdk::script::Input::Payload(pdk::script::Format::PlainText))
        .input(pdk::script::Input::Vars("exampleVar"))
        .compile()
        .map_err(serde::de::Error::custom)
}

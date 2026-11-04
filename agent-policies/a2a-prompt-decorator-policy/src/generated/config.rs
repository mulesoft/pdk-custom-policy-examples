use serde::Deserialize;
#[derive(Deserialize, Clone, Debug)]
pub struct FileDecorators0Config {
    #[serde(alias = "condition", default, deserialize_with = "de_condition_0")]
    pub condition: Option<pdk::script::Script>,
    #[serde(alias = "file", deserialize_with = "de_file_1")]
    pub file: pdk::script::Script,
    #[serde(alias = "fileMimeType", default, deserialize_with = "de_file_mime_type_2")]
    pub file_mime_type: Option<pdk::script::Script>,
    #[serde(alias = "fileName", default, deserialize_with = "de_file_name_3")]
    pub file_name: Option<pdk::script::Script>,
    #[serde(alias = "fileType")]
    pub file_type: String,
}
#[derive(Deserialize, Clone, Debug)]
pub struct TextDecorators0Config {
    #[serde(alias = "condition", default, deserialize_with = "de_condition_4")]
    pub condition: Option<pdk::script::Script>,
    #[serde(alias = "text", deserialize_with = "de_text_5")]
    pub text: pdk::script::Script,
}
#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    #[serde(alias = "fileDecorators")]
    pub file_decorators: Option<Vec<FileDecorators0Config>>,
    #[serde(alias = "textDecorators")]
    pub text_decorators: Option<Vec<TextDecorators0Config>>,
}
#[pdk::hl::entrypoint_flex]
fn init(abi: &dyn pdk::flex_abi::api::FlexAbi) -> Result<(), anyhow::Error> {
    abi.setup()?;
    Ok(())
}
fn de_condition_0<'de, D>(
    deserializer: D,
) -> Result<Option<pdk::script::Script>, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    let exp: Option<pdk::script::Expression> = serde::de::Deserialize::deserialize(
        deserializer,
    )?;
    exp.map(|exp| {
            pdk::script::ScriptingEngine::script(&exp)
                .input(pdk::script::Input::Attributes)
                .input(pdk::script::Input::Authentication)
                .input(pdk::script::Input::Vars("params"))
                .compile()
                .map_err(serde::de::Error::custom)
        })
        .transpose()
}
fn de_file_1<'de, D>(deserializer: D) -> Result<pdk::script::Script, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    let exp: pdk::script::Expression = serde::de::Deserialize::deserialize(
        deserializer,
    )?;
    pdk::script::ScriptingEngine::script(&exp)
        .input(pdk::script::Input::Attributes)
        .input(pdk::script::Input::Authentication)
        .input(pdk::script::Input::Vars("params"))
        .compile()
        .map_err(serde::de::Error::custom)
}
fn de_file_mime_type_2<'de, D>(
    deserializer: D,
) -> Result<Option<pdk::script::Script>, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    let exp: Option<pdk::script::Expression> = serde::de::Deserialize::deserialize(
        deserializer,
    )?;
    exp.map(|exp| {
            pdk::script::ScriptingEngine::script(&exp)
                .input(pdk::script::Input::Attributes)
                .input(pdk::script::Input::Authentication)
                .input(pdk::script::Input::Vars("params"))
                .compile()
                .map_err(serde::de::Error::custom)
        })
        .transpose()
}
fn de_file_name_3<'de, D>(
    deserializer: D,
) -> Result<Option<pdk::script::Script>, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    let exp: Option<pdk::script::Expression> = serde::de::Deserialize::deserialize(
        deserializer,
    )?;
    exp.map(|exp| {
            pdk::script::ScriptingEngine::script(&exp)
                .input(pdk::script::Input::Attributes)
                .input(pdk::script::Input::Authentication)
                .input(pdk::script::Input::Vars("params"))
                .compile()
                .map_err(serde::de::Error::custom)
        })
        .transpose()
}
fn de_condition_4<'de, D>(
    deserializer: D,
) -> Result<Option<pdk::script::Script>, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    let exp: Option<pdk::script::Expression> = serde::de::Deserialize::deserialize(
        deserializer,
    )?;
    exp.map(|exp| {
            pdk::script::ScriptingEngine::script(&exp)
                .input(pdk::script::Input::Attributes)
                .input(pdk::script::Input::Authentication)
                .input(pdk::script::Input::Vars("params"))
                .compile()
                .map_err(serde::de::Error::custom)
        })
        .transpose()
}
fn de_text_5<'de, D>(deserializer: D) -> Result<pdk::script::Script, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    let exp: pdk::script::Expression = serde::de::Deserialize::deserialize(
        deserializer,
    )?;
    pdk::script::ScriptingEngine::script(&exp)
        .input(pdk::script::Input::Attributes)
        .input(pdk::script::Input::Authentication)
        .input(pdk::script::Input::Vars("params"))
        .compile()
        .map_err(serde::de::Error::custom)
}

// Copyright 2023 Salesforce, Inc. All rights reserved.
mod generated;

use crate::generated::config::Config;
use agent_core::a2a::SEND_TASK_FUNCTION_NAME;
use agent_core::http_utils::{
    with_no_timeout, CONTENT_LENGTH_HEADER, CONTENT_TYPE_HEADER, POST_METHOD,
};
use anyhow::{anyhow, Result};
use pdk::hl::*;
use pdk::logger;
use pdk::script::{PayloadBinding, Script};
use serde_json::value::RawValue;
use serde_json::{Error, Map, Value};

use agent_core::attributes::HeadersAttributes;
use agent_core::json_rpc::JsonRpcRequest;
use pdk::authentication::{Authentication, AuthenticationHandler};

async fn request_filter(
    request_state: RequestState,
    config: &Config,
    stream_properties: StreamProperties,
    auth: Authentication,
) -> Flow<()> {
    let header_state = request_state.into_headers_state().await;
    let handler = header_state.handler();

    // Remove the timeout
    with_no_timeout(handler);

    match header_state.method().as_str() {
        POST_METHOD => {
            let content_type_maybe = handler.header(CONTENT_TYPE_HEADER);
            match content_type_maybe {
                None => {}
                Some(content_type) => {
                    let mime: mime::Mime = content_type.parse().unwrap();
                    if mime.subtype() == mime::JSON {
                        // This should be the messages endpoint
                        let attributes = HeadersAttributes::new(
                            header_state.handler().headers(),
                            &stream_properties,
                        );
                        header_state.handler().remove_header(CONTENT_LENGTH_HEADER);
                        let request_body_state = header_state.into_body_state().await;
                        let body_bytes = request_body_state.as_bytes();
                        let mut request: JsonRpcRequest<'_> =
                            serde_json::from_slice(body_bytes.as_slice()).unwrap();

                        if request.method == SEND_TASK_FUNCTION_NAME && request.params.is_some() {
                            let params = request.params.unwrap();
                            let result: Result<Value, Error> = serde_json::from_str(params.get());
                            match result {
                                Ok(mut json_value) => {
                                    let original_value = &json_value.clone();
                                    if let Some(parts) = json_value
                                        .pointer_mut("/message/parts")
                                        .and_then(Value::as_array_mut)
                                    {
                                        decorate_prompts(
                                            &original_value,
                                            parts,
                                            config,
                                            &attributes,
                                            &auth,
                                        );

                                        let new_params =
                                            serde_json::to_string_pretty(&json_value).unwrap();
                                        let x = RawValue::from_string(new_params).unwrap();
                                        request.params = Some(&x);
                                        write_json_value_body(request_body_state, &mut request);
                                    }
                                }
                                e => {
                                    logger::error!("Unable to pars params as json {:?}", e);
                                }
                            }
                        }
                    }
                }
            };
            Flow::Continue(())
        }
        _ => Flow::Continue(()),
    }
}

fn write_json_value_body(request_body_state: RequestBodyState, request: &mut JsonRpcRequest) {
    match serde_json::to_string_pretty(&request) {
        Ok(encoded) => {
            let set_body_result = request_body_state.handler().set_body(encoded.as_bytes());
            match set_body_result {
                Ok(_) => {}
                Err(e) => {
                    logger::error!("Unable to write body -> {}", e);
                }
            }
        }
        Err(e) => {
            logger::error!("Failed to serialize json value :{:?}", e);
        }
    }
}

fn decorate_prompts(
    original_value: &&Value,
    parts: &mut Vec<Value>,
    config: &Config,
    attributes: &HeadersAttributes,
    auth: &Authentication,
) {
    if let Some(text_decorators) = &config.text_decorators {
        text_decorators.iter().for_each(|decorator| {
            let condition = match &decorator.condition {
                None => true,
                Some(c) => evaluate_to_bool(attributes, original_value, c, auth),
            };
            if condition {
                let mut part_properties: Map<String, Value> = serde_json::Map::new();
                part_properties.insert("type".to_string(), Value::from("text".to_string()));
                let value = evaluate_to_json_string_value(
                    attributes,
                    original_value,
                    &decorator.text,
                    auth,
                );
                part_properties.insert("text".to_string(), value);
                parts.insert(0, Value::Object(part_properties));
            }
        })
    }

    if let Some(file_decorators) = &config.file_decorators {
        file_decorators.iter().for_each(|decorator| {
            let condition = match &decorator.condition {
                None => true,
                Some(c) => evaluate_to_bool(attributes, original_value, c, auth),
            };
            if condition {
                let mut part_properties: Map<String, Value> = serde_json::Map::new();
                part_properties.insert("type".to_string(), Value::from("file".to_string()));
                let mut file_properties = serde_json::Map::new();
                if let Some(file_script) = &decorator.file_name {
                    let value = evaluate_to_json_string_value(
                        attributes,
                        original_value,
                        file_script,
                        auth,
                    );
                    file_properties.insert("name".to_string(), value);
                }

                if let Some(file_script) = &decorator.file_mime_type {
                    let value = evaluate_to_json_string_value(
                        attributes,
                        original_value,
                        file_script,
                        auth,
                    );
                    file_properties.insert("file_mime_type".to_string(), value);
                }

                let value = evaluate_to_json_string_value(
                    attributes,
                    original_value,
                    &decorator.file,
                    auth,
                );
                match decorator.file_type.as_str() {
                    "Uri" => {
                        file_properties.insert("uri".to_string(), value);
                    }
                    "Base64" => {
                        file_properties.insert("bytes".to_string(), value);
                    }
                    _ => {
                        logger::warn!("Unknown file type {}", decorator.file_type);
                    }
                }
                part_properties.insert("file".to_string(), Value::Object(file_properties));
                parts.insert(0, Value::Object(part_properties));
            }
        })
    }
}

fn evaluate_to_json_string_value(
    attributes: &HeadersAttributes,
    json_value: &Value,
    config_script: &Script,
    auth: &Authentication,
) -> Value {
    let mut evaluator = config_script.evaluator();
    evaluator.bind_attributes(attributes);
    evaluator.bind_authentication(&auth.authentication());
    let x: Value = json_value.clone();
    evaluator.bind_vars("params", x);

    match evaluator.eval() {
        Ok(value) => as_serde_string_value(&value),
        Err(e) => {
            logger::warn!("Error evaluating text script: {}", e);
            Value::Null
        }
    }
}

fn evaluate_to_bool(
    attributes: &HeadersAttributes,
    json_value: &Value,
    config_script: &Script,
    auth: &Authentication,
) -> bool {
    let mut evaluator = config_script.evaluator();
    evaluator.bind_attributes(attributes);
    evaluator.bind_authentication(&auth.authentication());
    let x: Value = json_value.clone();
    evaluator.bind_vars("params", x);

    match evaluator.eval() {
        Ok(pdk::script::Value::Bool(b)) => b,
        Ok(e) => {
            logger::warn!("Script didn't return a boolean value instead `{:?}`", e);
            false
        }
        Err(e) => {
            logger::warn!("Error evaluating text script: {}", e);
            false
        }
    }
}

fn as_serde_string_value(value: &pdk::script::Value) -> Value {
    match value {
        pdk::script::Value::Null => Value::Null,
        pdk::script::Value::Bool(b) => Value::String(b.clone().to_string()),
        pdk::script::Value::Number(n) => Value::String(n.to_string()),
        pdk::script::Value::String(s) => Value::String(s.clone()),
        _ => {
            logger::warn!("Value: `{:?}` is not a string", value);
            Value::Null
        }
    }
}

#[entrypoint]
async fn configure(launcher: Launcher, Configuration(bytes): Configuration) -> Result<()> {
    let config: Config = serde_json::from_slice(&bytes).map_err(|err| {
        anyhow!(
            "Failed to parse configuration '{}'. Cause: {}",
            String::from_utf8_lossy(&bytes),
            err
        )
    })?;

    let filter = on_request(|rs, stream_properties, auth| {
        request_filter(rs, &config, stream_properties, auth)
    });
    launcher.launch(filter).await?;
    Ok(())
}

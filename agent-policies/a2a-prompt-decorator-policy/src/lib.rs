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

#[cfg(test)]
mod tests {
    use pdk_unit::{
        dw2pel, TraceBackend, UnitHttpMessage, UnitHttpRequest, UnitHttpResponse, UnitTestBuilder,
    };
    use serde_json::json;
    use std::rc::Rc;

    fn tasks_send_body(text: &str) -> String {
        json!({
            "jsonrpc": "2.0",
            "method": "tasks/send",
            "id": 1,
            "params": {
                "id": "task-1",
                "message": {
                    "role": "user",
                    "parts": [{"type": "text", "text": text}]
                }
            }
        })
        .to_string()
    }

    fn parts_from_backend(backend: &TraceBackend<UnitHttpResponse>) -> Vec<serde_json::Value> {
        let req = backend.next().unwrap();
        let body: serde_json::Value = serde_json::from_slice(req.body()).unwrap();
        body["params"]["message"]["parts"]
            .as_array()
            .unwrap()
            .clone()
    }

    #[test]
    fn get_request_passes_through_unchanged() {
        let mut tester = UnitTestBuilder::default()
            .with_config(json!({"textDecorators": [], "fileDecorators": []}).to_string())
            .with_entrypoint(crate::configure);

        let response = tester.request_full(UnitHttpRequest::get().with_path("/api/tasks"));

        assert_eq!(response.status_code(), 200);
    }

    #[test]
    fn post_without_content_type_passes_through_unchanged() {
        let backend = Rc::new(TraceBackend::new(UnitHttpResponse::new(200)));
        let mut tester = UnitTestBuilder::default()
            .with_config(json!({"textDecorators": [], "fileDecorators": []}).to_string())
            .with_backend(Rc::clone(&backend))
            .with_entrypoint(crate::configure);

        let response =
            tester.request_full(UnitHttpRequest::post().with_body(tasks_send_body("Hello")));

        assert_eq!(response.status_code(), 200);
        let req = backend.next().unwrap();
        let body: serde_json::Value = serde_json::from_slice(req.body()).unwrap();
        assert_eq!(body["params"]["message"]["parts"][0]["text"], "Hello");
    }

    #[test]
    fn text_decorator_is_prepended_to_parts() {
        let backend = Rc::new(TraceBackend::new(UnitHttpResponse::new(200)));
        let mut tester = UnitTestBuilder::default()
            .with_config(
                json!({
                    "textDecorators": [{"text": dw2pel("\"You are a helpful assistant.\"")}],
                    "fileDecorators": []
                })
                .to_string(),
            )
            .with_backend(Rc::clone(&backend))
            .with_entrypoint(crate::configure);

        let response = tester.request_full(
            UnitHttpRequest::post()
                .with_header("content-type", "application/json")
                .with_body(tasks_send_body("Hello")),
        );

        assert_eq!(response.status_code(), 200);
        let parts = parts_from_backend(&backend);
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0]["type"], "text");
        assert_eq!(parts[0]["text"], "You are a helpful assistant.");
        assert_eq!(parts[1]["text"], "Hello");
    }

    #[test]
    fn multiple_text_decorators_are_all_prepended() {
        let backend = Rc::new(TraceBackend::new(UnitHttpResponse::new(200)));
        let mut tester = UnitTestBuilder::default()
            .with_config(
                json!({
                    "textDecorators": [
                        {"text": dw2pel("\"First decorator\"")},
                        {"text": dw2pel("\"Second decorator\"")}
                    ],
                    "fileDecorators": []
                })
                .to_string(),
            )
            .with_backend(Rc::clone(&backend))
            .with_entrypoint(crate::configure);

        let response = tester.request_full(
            UnitHttpRequest::post()
                .with_header("content-type", "application/json")
                .with_body(tasks_send_body("Hello")),
        );

        assert_eq!(response.status_code(), 200);
        let parts = parts_from_backend(&backend);
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[2]["text"], "Hello");
    }

    #[test]
    fn text_decorator_with_true_condition_is_applied() {
        let backend = Rc::new(TraceBackend::new(UnitHttpResponse::new(200)));
        let mut tester = UnitTestBuilder::default()
            .with_config(
                json!({
                    "textDecorators": [{
                        "text": dw2pel("\"Conditional text\""),
                        "condition": dw2pel("true")
                    }],
                    "fileDecorators": []
                })
                .to_string(),
            )
            .with_backend(Rc::clone(&backend))
            .with_entrypoint(crate::configure);

        let response = tester.request_full(
            UnitHttpRequest::post()
                .with_header("content-type", "application/json")
                .with_body(tasks_send_body("Hello")),
        );

        assert_eq!(response.status_code(), 200);
        let parts = parts_from_backend(&backend);
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0]["text"], "Conditional text");
    }

    #[test]
    fn text_decorator_with_false_condition_is_skipped() {
        let backend = Rc::new(TraceBackend::new(UnitHttpResponse::new(200)));
        let mut tester = UnitTestBuilder::default()
            .with_config(
                json!({
                    "textDecorators": [{
                        "text": dw2pel("\"Should not appear\""),
                        "condition": dw2pel("false")
                    }],
                    "fileDecorators": []
                })
                .to_string(),
            )
            .with_backend(Rc::clone(&backend))
            .with_entrypoint(crate::configure);

        let response = tester.request_full(
            UnitHttpRequest::post()
                .with_header("content-type", "application/json")
                .with_body(tasks_send_body("Hello")),
        );

        assert_eq!(response.status_code(), 200);
        let parts = parts_from_backend(&backend);
        assert_eq!(parts.len(), 1);
        assert_eq!(parts[0]["text"], "Hello");
    }

    #[test]
    fn file_decorator_uri_type_is_prepended() {
        let backend = Rc::new(TraceBackend::new(UnitHttpResponse::new(200)));
        let mut tester = UnitTestBuilder::default()
            .with_config(
                json!({
                    "textDecorators": [],
                    "fileDecorators": [{
                        "file": dw2pel("\"https://example.com/context.txt\""),
                        "fileType": "Uri",
                        "fileName": dw2pel("\"context.txt\"")
                    }]
                })
                .to_string(),
            )
            .with_backend(Rc::clone(&backend))
            .with_entrypoint(crate::configure);

        let response = tester.request_full(
            UnitHttpRequest::post()
                .with_header("content-type", "application/json")
                .with_body(tasks_send_body("Hello")),
        );

        assert_eq!(response.status_code(), 200);
        let parts = parts_from_backend(&backend);
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0]["type"], "file");
        assert_eq!(parts[0]["file"]["uri"], "https://example.com/context.txt");
        assert_eq!(parts[0]["file"]["name"], "context.txt");
    }

    #[test]
    fn file_decorator_base64_type_is_prepended() {
        let backend = Rc::new(TraceBackend::new(UnitHttpResponse::new(200)));
        let mut tester = UnitTestBuilder::default()
            .with_config(
                json!({
                    "textDecorators": [],
                    "fileDecorators": [{
                        "file": dw2pel("\"SGVsbG8gV29ybGQ=\""),
                        "fileType": "Base64"
                    }]
                })
                .to_string(),
            )
            .with_backend(Rc::clone(&backend))
            .with_entrypoint(crate::configure);

        let response = tester.request_full(
            UnitHttpRequest::post()
                .with_header("content-type", "application/json")
                .with_body(tasks_send_body("Hello")),
        );

        assert_eq!(response.status_code(), 200);
        let parts = parts_from_backend(&backend);
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0]["type"], "file");
        assert_eq!(parts[0]["file"]["bytes"], "SGVsbG8gV29ybGQ=");
    }

    #[test]
    fn non_send_task_method_is_not_decorated() {
        let backend = Rc::new(TraceBackend::new(UnitHttpResponse::new(200)));
        let mut tester = UnitTestBuilder::default()
            .with_config(
                json!({
                    "textDecorators": [{"text": dw2pel("\"Should not appear\"")}],
                    "fileDecorators": []
                })
                .to_string(),
            )
            .with_backend(Rc::clone(&backend))
            .with_entrypoint(crate::configure);

        let body = json!({
            "jsonrpc": "2.0",
            "method": "tasks/get",
            "id": 1,
            "params": {"id": "task-1"}
        })
        .to_string();

        let response = tester.request_full(
            UnitHttpRequest::post()
                .with_header("content-type", "application/json")
                .with_body(body),
        );

        assert_eq!(response.status_code(), 200);
        let req = backend.next().unwrap();
        let backend_body: serde_json::Value = serde_json::from_slice(req.body()).unwrap();
        assert_eq!(backend_body["method"], "tasks/get");
    }
}

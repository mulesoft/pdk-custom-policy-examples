// Copyright 2023 Salesforce, Inc. All rights reserved.
mod generated;
mod pii;

use crate::generated::config::Config;
use agent_core::a2a::{SEND_SUBSCRIBE_TASK_FUNCTION_NAME, SEND_TASK_FUNCTION_NAME};
use agent_core::http_utils::{with_no_timeout, CONTENT_TYPE_HEADER, POST_METHOD};

use anyhow::{anyhow, Result};

use crate::pii::{PiiDetector, RegexPiiDetector};

use pdk::hl::*;
use pdk::logger;

use agent_core::json_rpc::JsonRpcRequest;
use pdk::script::PayloadBinding;
use serde_json::{Error, Value};

const TEXT_FIELD: &str = "text";

async fn request_filter(
    request_state: RequestState,
    config: &Config,
    regex_pii_detector: &dyn PiiDetector,
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
                        let request_body_state = header_state.into_body_state().await;
                        let body_bytes = request_body_state.as_bytes();
                        let request: JsonRpcRequest<'_> =
                            serde_json::from_slice(body_bytes.as_slice()).unwrap();
                        if (request.method == SEND_TASK_FUNCTION_NAME
                            || request.method == SEND_SUBSCRIBE_TASK_FUNCTION_NAME)
                            && request.params.is_some()
                        {
                            let params = request.params.unwrap();
                            let result: Result<Value, Error> = serde_json::from_str(params.get());
                            match result {
                                Ok(json_value) => {
                                    if let Some(Value::Array(parts_values)) =
                                        json_value.pointer("/message/parts")
                                    {
                                        return report_pii_if_detected(
                                            config,
                                            parts_values,
                                            regex_pii_detector,
                                        );
                                    }
                                }
                                e => {
                                    logger::error!("Unable to parse params as json `{:?}`", e);
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

fn report_pii_if_detected(
    config: &Config,
    parts_values: &[Value],
    regex_pii_detector: &dyn PiiDetector,
) -> Flow<()> {
    for value in parts_values {
        let part_type = value
            .get("type")
            .map(|v| v.as_str().unwrap_or_default())
            .unwrap_or_default();
        if value.is_object() && part_type == "text" {
            if let Some(Value::String(s)) = value.get(TEXT_FIELD) {
                let pii = regex_pii_detector.detect(s.as_str()).unwrap();
                if !pii.is_empty() {
                    let result: String = serde_json::to_string_pretty(&pii).unwrap_or_default();
                    match config.action.as_str() {
                        "Reject" => {
                            return Flow::Break(
                                Response::new(401).with_body(format!(
                                    "Request contains PII data: \n`{}`",
                                    result
                                )),
                            );
                        }
                        _ => {
                            logger::warn!("Request: `{}` has sensitive data: `{:?}`", s, result);
                        }
                    }
                }
            }
        }
    }
    Flow::Continue(())
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

    let pii_detector = RegexPiiDetector::new()?;
    let filter = on_request(|rs| request_filter(rs, &config, &pii_detector));

    launcher.launch(filter).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use pdk_unit::{
        TraceBackend, UnitHttpMessage, UnitHttpRequest, UnitHttpResponse, UnitTestBuilder,
    };
    use serde_json::json;
    use std::rc::Rc;

    fn config(action: &str) -> String {
        json!({ "action": action }).to_string()
    }

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

    #[test]
    fn clean_message_passes_through() {
        let backend = Rc::new(TraceBackend::new(UnitHttpResponse::new(200)));
        let mut tester = UnitTestBuilder::default()
            .with_config(config("Reject"))
            .with_backend(Rc::clone(&backend))
            .with_entrypoint(crate::configure);

        let response = tester.request(
            UnitHttpRequest::post()
                .with_header("content-type", "application/json")
                .with_body(tasks_send_body("Hello, how are you?")),
        );

        assert_eq!(response.status_code(), 200);
        let req = backend.next().unwrap();
        let body: serde_json::Value = serde_json::from_slice(req.body()).unwrap();
        assert_eq!(
            body["params"]["message"]["parts"][0]["text"],
            "Hello, how are you?"
        );
    }

    #[test]
    fn email_in_message_is_rejected() {
        let mut tester = UnitTestBuilder::default()
            .with_config(config("Reject"))
            .with_entrypoint(crate::configure);

        let response = tester.request(
            UnitHttpRequest::post()
                .with_header("content-type", "application/json")
                .with_body(tasks_send_body("My email is john.doe@example.com")),
        );

        assert_eq!(response.status_code(), 401);
    }

    #[test]
    fn ssn_in_message_is_rejected() {
        let mut tester = UnitTestBuilder::default()
            .with_config(config("Reject"))
            .with_entrypoint(crate::configure);

        let response = tester.request(
            UnitHttpRequest::post()
                .with_header("content-type", "application/json")
                .with_body(tasks_send_body("My SSN is 123-45-6789")),
        );

        assert_eq!(response.status_code(), 401);
    }

    #[test]
    fn credit_card_in_message_is_rejected() {
        let mut tester = UnitTestBuilder::default()
            .with_config(config("Reject"))
            .with_entrypoint(crate::configure);

        let response = tester.request(
            UnitHttpRequest::post()
                .with_header("content-type", "application/json")
                .with_body(tasks_send_body("Pay with 4111 1111 1111 1111")),
        );

        assert_eq!(response.status_code(), 401);
    }

    #[test]
    fn pii_with_warn_action_passes_through() {
        let backend = Rc::new(TraceBackend::new(UnitHttpResponse::new(200)));
        let mut tester = UnitTestBuilder::default()
            .with_config(config("Warn"))
            .with_backend(Rc::clone(&backend))
            .with_entrypoint(crate::configure);

        let response = tester.request(
            UnitHttpRequest::post()
                .with_header("content-type", "application/json")
                .with_body(tasks_send_body("My email is john.doe@example.com")),
        );

        assert_eq!(response.status_code(), 200);
        let req = backend.next().unwrap();
        let body: serde_json::Value = serde_json::from_slice(req.body()).unwrap();
        assert_eq!(body["method"], "tasks/send");
    }

    #[test]
    fn get_request_passes_through() {
        let mut tester = UnitTestBuilder::default()
            .with_config(config("Reject"))
            .with_entrypoint(crate::configure);

        let response = tester.request(UnitHttpRequest::get().with_path("/api/tasks"));

        assert_eq!(response.status_code(), 200);
    }

    #[test]
    fn post_without_content_type_passes_through() {
        let backend = Rc::new(TraceBackend::new(UnitHttpResponse::new(200)));
        let mut tester = UnitTestBuilder::default()
            .with_config(config("Reject"))
            .with_backend(Rc::clone(&backend))
            .with_entrypoint(crate::configure);

        let response = tester.request(
            UnitHttpRequest::post().with_body(tasks_send_body("My email is john.doe@example.com")),
        );

        assert_eq!(response.status_code(), 200);
    }

    #[test]
    fn non_send_task_method_skips_pii_detection() {
        let backend = Rc::new(TraceBackend::new(UnitHttpResponse::new(200)));
        let mut tester = UnitTestBuilder::default()
            .with_config(config("Reject"))
            .with_backend(Rc::clone(&backend))
            .with_entrypoint(crate::configure);

        let body = json!({
            "jsonrpc": "2.0",
            "method": "tasks/get",
            "id": 1,
            "params": {"id": "task-1"}
        })
        .to_string();

        let response = tester.request(
            UnitHttpRequest::post()
                .with_header("content-type", "application/json")
                .with_body(body),
        );

        assert_eq!(response.status_code(), 200);
        let req = backend.next().unwrap();
        let backend_body: serde_json::Value = serde_json::from_slice(req.body()).unwrap();
        assert_eq!(backend_body["method"], "tasks/get");
    }

    #[test]
    fn send_subscribe_with_pii_is_rejected() {
        let mut tester = UnitTestBuilder::default()
            .with_config(config("Reject"))
            .with_entrypoint(crate::configure);

        let body = json!({
            "jsonrpc": "2.0",
            "method": "tasks/sendSubscribe",
            "id": 1,
            "params": {
                "id": "task-1",
                "message": {
                    "role": "user",
                    "parts": [{"type": "text", "text": "Contact me at 555-867-5309"}]
                }
            }
        })
        .to_string();

        let response = tester.request(
            UnitHttpRequest::post()
                .with_header("content-type", "application/json")
                .with_body(body),
        );

        assert_eq!(response.status_code(), 401);
    }
}

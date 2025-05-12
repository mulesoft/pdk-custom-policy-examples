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

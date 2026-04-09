// Copyright 2023 Salesforce, Inc. All rights reserved.

mod generated;

use anyhow::{anyhow, Result};
use pdk::hl::*;
use pdk::json_validator::{JsonValidator, JsonValidatorBuilder, ValidationResult};
use std::convert::TryFrom;
use std::rc::Rc;

use crate::generated::config::Config;

macro_rules! with_json_limit {
    ($builder:ident, $limit:expr, $with:ident) => {
        if let Some(n) = positive_usize($limit) {
            $builder = $builder.$with(n);
        }
    };
}

fn validator_from_config(config: &Config) -> JsonValidator {
    let mut builder = JsonValidatorBuilder::new();
    with_json_limit!(builder, config.max_depth, with_max_depth);
    with_json_limit!(builder, config.max_array_length, with_max_array_length);
    with_json_limit!(builder, config.max_string_length, with_max_string_length);
    with_json_limit!(builder, config.max_object_entries, with_max_object_entries);
    with_json_limit!(builder, config.max_key_length, with_max_key_length);
    builder.build()
}

fn positive_usize(value: Option<i64>) -> Option<usize> {
    value
        .filter(|&v| v > 0)
        .and_then(|v| usize::try_from(v).ok())
}

async fn request_filter(state: RequestState, config: &Config) -> Flow<()> {
    let headers_state = state.into_headers_state().await;

    if !headers_state.contains_body() {
        return Flow::Continue(());
    }

    let body_stream_state = headers_state.into_body_stream_state().await;
    let mut stream = body_stream_state.stream();
    let mut buf = Vec::new();
    while let Some(chunk) = stream.next().await {
        buf.extend_from_slice(&chunk.into_bytes());
    }

    let mut validator = validator_from_config(config);
    let outcome = validator.validate_chunk(&buf, true);
    match outcome {
        Ok(ValidationResult::Complete) => Flow::Continue(()),
        Ok(ValidationResult::Incomplete) | Err(_) => Flow::Break(Response::new(400)),
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

    let config = Rc::new(config);
    let filter = on_request({
        let config = Rc::clone(&config);
        move |state| {
            let config = Rc::clone(&config);
            async move { request_filter(state, config.as_ref()).await }
        }
    });

    launcher.launch(filter).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdk_unit::{TraceBackend, UnitHttpRequest, UnitHttpResponse, UnitTestBuilder};
    use serde_json::json;
    use std::rc::Rc;

    fn default_config() -> String {
        json!({}).to_string()
    }

    #[test]
    fn test_get_without_body_reaches_backend() {
        let backend = Rc::new(TraceBackend::new(UnitHttpResponse::new(200)));
        let mut tester = UnitTestBuilder::default()
            .with_config(default_config())
            .with_backend(Rc::clone(&backend))
            .with_entrypoint(configure);

        assert_eq!(tester.request(UnitHttpRequest::get()).status_code(), 200);
        assert!(backend.next().is_some());
    }

    #[test]
    fn test_post_valid_json_reaches_backend() {
        let backend = Rc::new(TraceBackend::new(UnitHttpResponse::new(200)));
        let mut tester = UnitTestBuilder::default()
            .with_config(default_config())
            .with_backend(Rc::clone(&backend))
            .with_entrypoint(configure);

        let response = tester.request(
            UnitHttpRequest::post()
                .with_header("Content-Type", "application/json")
                .with_body(r#"{"item":"book","qty":1}"#),
        );
        assert_eq!(response.status_code(), 200);
        assert!(backend.next().is_some());
    }

    #[test]
    fn test_post_invalid_json_returns_400_and_does_not_call_backend() {
        let backend = Rc::new(TraceBackend::new(UnitHttpResponse::new(200)));
        let mut tester = UnitTestBuilder::default()
            .with_config(default_config())
            .with_backend(Rc::clone(&backend))
            .with_entrypoint(configure);

        let response = tester.request(
            UnitHttpRequest::post()
                .with_header("Content-Type", "application/json")
                .with_body(r#"{"a":1]"#),
        );
        assert_eq!(response.status_code(), 400);
        assert!(backend.next().is_none());
    }

    #[test]
    fn test_max_depth_violation_returns_400() {
        let backend = Rc::new(TraceBackend::new(UnitHttpResponse::new(200)));
        let mut tester = UnitTestBuilder::default()
            .with_config(json!({ "maxDepth": 2 }).to_string())
            .with_backend(Rc::clone(&backend))
            .with_entrypoint(configure);

        let response = tester.request(UnitHttpRequest::post().with_body(r#"{"a":{"b":{"c":1}}}"#));
        assert_eq!(response.status_code(), 400);
        assert!(backend.next().is_none());
    }
}

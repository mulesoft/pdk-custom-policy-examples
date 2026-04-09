// Copyright 2023 Salesforce, Inc. All rights reserved.

mod generated;

use anyhow::{anyhow, Result};
use pdk::hl::*;
use pdk::xml_validator::{XmlValidator, XmlValidatorBuilder};
use std::convert::TryFrom;
use std::rc::Rc;

use crate::generated::config::Config;

macro_rules! with_xml_limit {
    ($builder:ident, $limit:expr, $with:ident) => {
        if let Some(n) = positive_usize($limit) {
            $builder = $builder.$with(n);
        }
    };
}

fn validator_from_config(config: &Config) -> XmlValidator {
    let mut builder = XmlValidatorBuilder::new();
    with_xml_limit!(builder, config.max_depth, with_max_depth);
    with_xml_limit!(
        builder,
        config.max_attribute_count,
        with_max_attribute_count
    );
    with_xml_limit!(builder, config.max_child_count, with_max_child_count);
    with_xml_limit!(builder, config.max_text_length, with_max_text_length);
    with_xml_limit!(
        builder,
        config.max_attribute_length,
        with_max_attribute_length
    );
    with_xml_limit!(builder, config.max_comment_length, with_max_comment_length);
    builder.build()
}

fn positive_usize(value: Option<i64>) -> Option<usize> {
    value
        .filter(|&v| v > 0)
        .and_then(|v| usize::try_from(v).ok())
}

async fn request_filter(state: RequestState, validator: &XmlValidator) -> Flow<()> {
    let headers_state = state.into_headers_state().await;

    if !headers_state.contains_body() {
        return Flow::Continue(());
    }

    let body_stream_state = headers_state.into_body_stream_state().await;
    match validator.validate_stream(body_stream_state.stream()).await {
        Ok(()) => Flow::Continue(()),
        Err(_) => Flow::Break(Response::new(400)),
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

    let validator = Rc::new(validator_from_config(&config));
    let filter = on_request({
        let validator = Rc::clone(&validator);
        move |state| {
            let validator = Rc::clone(&validator);
            async move { request_filter(state, validator.as_ref()).await }
        }
    });

    launcher.launch(filter).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use pdk_unit::{TraceBackend, UnitHttpRequest, UnitHttpResponse, UnitTestBuilder};
    use serde_json::json;
    use std::rc::Rc;

    fn empty_config() -> String {
        json!({}).to_string()
    }

    #[test]
    fn get_without_body_reaches_backend() {
        let backend = Rc::new(TraceBackend::new(UnitHttpResponse::new(200)));
        let mut tester = UnitTestBuilder::default()
            .with_config(empty_config())
            .with_backend(Rc::clone(&backend))
            .with_entrypoint(crate::configure);

        assert_eq!(
            tester.request(UnitHttpRequest::get()).status_code(),
            200
        );
        assert!(backend.next().is_some());
    }

    #[test]
    fn post_valid_xml_reaches_backend() {
        let backend = Rc::new(TraceBackend::new(UnitHttpResponse::new(200)));
        let mut tester = UnitTestBuilder::default()
            .with_config(empty_config())
            .with_backend(Rc::clone(&backend))
            .with_entrypoint(crate::configure);

        let response = tester.request(
            UnitHttpRequest::post()
                .with_header("Content-Type", "application/xml")
                .with_body("<order><item id=\"1\">book</item></order>"),
        );
        assert_eq!(response.status_code(), 200);
        assert!(backend.next().is_some());
    }

    #[test]
    fn post_malformed_xml_returns_400_and_does_not_call_backend() {
        let backend = Rc::new(TraceBackend::new(UnitHttpResponse::new(200)));
        let mut tester = UnitTestBuilder::default()
            .with_config(empty_config())
            .with_backend(Rc::clone(&backend))
            .with_entrypoint(crate::configure);

        let response = tester.request(
            UnitHttpRequest::post()
                .with_header("Content-Type", "application/xml")
                .with_body("<root><child></root>"),
        );
        assert_eq!(response.status_code(), 400);
        assert!(backend.next().is_none());
    }

    #[test]
    fn max_depth_violation_returns_400() {
        let backend = Rc::new(TraceBackend::new(UnitHttpResponse::new(200)));
        let mut tester = UnitTestBuilder::default()
            .with_config(json!({ "maxDepth": 3 }).to_string())
            .with_backend(Rc::clone(&backend))
            .with_entrypoint(crate::configure);

        let deep = "<a><b><c><d/></c></b></a>";
        let response = tester.request(
            UnitHttpRequest::post()
                .with_header("Content-Type", "application/xml")
                .with_body(deep),
        );
        assert_eq!(response.status_code(), 400);
        assert!(backend.next().is_none());
    }

    #[test]
    fn max_child_count_violation_returns_400() {
        let backend = Rc::new(TraceBackend::new(UnitHttpResponse::new(200)));
        let mut tester = UnitTestBuilder::default()
            .with_config(json!({ "maxChildCount": 2 }).to_string())
            .with_backend(Rc::clone(&backend))
            .with_entrypoint(crate::configure);

        let response =
            tester.request(UnitHttpRequest::post().with_body("<root><c1/><c2/><c3/></root>"));
        assert_eq!(response.status_code(), 400);
        assert!(backend.next().is_none());
    }
}

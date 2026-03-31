// Copyright 2023 Salesforce, Inc. All rights reserved.
mod generated;

use anyhow::{anyhow, Result};

use pdk::hl::*;

use crate::generated::config::Config;

async fn request_filter(request_state: RequestState, config: &Config) -> Flow<()> {
    if !config.modify_request {
        return Flow::Continue(());
    }

    // Transition directly to headers-body state to access both headers and body
    let state = request_state.into_headers_body_state().await;

    // Modify headers
    let original_method = state.handler().header(":method").unwrap_or_default();
    state
        .handler()
        .set_header("x-original-method", &original_method);

    // Modify body if present
    if state.contains_body() {
        let body = state.handler().body();
        let body_str = String::from_utf8_lossy(&body);
        let modified_body = format!("{}-{}", config.body_prefix, body_str);

        if let Err(err) = state.handler().set_body(modified_body.as_bytes()) {
            // If body cannot be set (e.g., GET request), add as header
            pdk::logger::info!("Cannot set body: {:?}, adding as header instead", err);
            state
                .handler()
                .set_header("x-modified-body", &modified_body);
        }
    }

    Flow::Continue(())
}

async fn response_filter(response_state: ResponseState, config: &Config) {
    if config.modify_response {
        // Transition to headers-body state for response
        let state = response_state.into_headers_body_state().await;

        // Add header indicating response was modified
        state
            .handler()
            .set_header("x-stop-iteration", "response-modified");

        // Access and modify response body
        if state.contains_body() {
            let body = state.handler().body();
            let body_str = String::from_utf8_lossy(&body);
            let modified_body = format!("{}:{}", config.body_prefix, body_str);

            let _ = state.handler().set_body(modified_body.as_bytes());
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
    let filter = on_request(|rs| request_filter(rs, &config))
                    .on_response(|res| response_filter(res, &config));
    launcher.launch(filter).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdk_unit::{TraceBackend, UnitHttpMessage, UnitHttpRequest, UnitHttpResponse, UnitTestBuilder};
    use serde_json::json;
    use std::rc::Rc;

    fn backend_fn(_request: UnitHttpRequest) -> UnitHttpResponse {
        UnitHttpResponse::new(200).with_body(b"backend-response")
    }

    fn default_config() -> String {
        json!({
            "bodyPrefix": "PREFIX",
            "modifyRequest": true,
            "modifyResponse": true
        })
        .to_string()
    }

    #[test]
    fn test_modify_request_disabled() {
        let config = json!({
            "bodyPrefix": "TEST",
            "modifyRequest": false,
            "modifyResponse": false
        })
        .to_string();

        let mut tester = UnitTestBuilder::default()
            .with_backend(backend_fn)
            .with_config(config)
            .with_entrypoint(configure);

        let response = tester.request_full(
            UnitHttpRequest::post()
                .with_path("/test")
                .with_body(b"request-body"),
        );

        assert_eq!(response.status_code(), 200);
        // Original method header should NOT be added
        assert!(response.header("x-original-method").is_none());
        // Response should not be modified
        assert!(response.header("x-stop-iteration").is_none());
    }

    #[test]
    fn test_modify_request_with_body() {
        let backend = Rc::new(TraceBackend::new(backend_fn));
        let config = json!({
            "bodyPrefix": "REQ",
            "modifyRequest": true,
            "modifyResponse": false
        })
        .to_string();

        let mut tester = UnitTestBuilder::default()
            .with_backend(Rc::clone(&backend))
            .with_config(config)
            .with_entrypoint(configure);

        let response = tester.request_full(
            UnitHttpRequest::post()
                .with_path("/api/data")
                .with_body(b"original-body"),
        );

        assert_eq!(response.status_code(), 200);

        // Check that the backend received the modified request
        let backend_request = backend.next().expect("backend should receive request");
        assert_eq!(backend_request.header("x-original-method"), Some("POST"));

        // Check that body was modified
        let body = String::from_utf8_lossy(backend_request.body());
        assert!(body.starts_with("REQ-"), "Expected body to start with 'REQ-', got: {}", body);
    }

    #[test]
    fn test_modify_response_adds_header_and_modifies_body() {
        let config = json!({
            "bodyPrefix": "RESP",
            "modifyRequest": false,
            "modifyResponse": true
        })
        .to_string();

        let mut tester = UnitTestBuilder::default()
            .with_backend(backend_fn)
            .with_config(config)
            .with_entrypoint(configure);

        let response = tester.request_full(UnitHttpRequest::get().with_path("/test"));

        assert_eq!(response.status_code(), 200);
        // Should add stop-iteration header
        assert_eq!(
            response.header("x-stop-iteration"),
            Some("response-modified")
        );
        // Response body should be modified with prefix
        let body = String::from_utf8_lossy(response.body());
        assert!(
            body.starts_with("RESP:"),
            "Expected body to start with 'RESP:', got: {}",
            body
        );
    }

    #[test]
    fn test_modify_both_request_and_response() {
        let backend = Rc::new(TraceBackend::new(backend_fn));
        let config = default_config();

        let mut tester = UnitTestBuilder::default()
            .with_backend(Rc::clone(&backend))
            .with_config(config)
            .with_entrypoint(configure);

        let response = tester.request_full(
            UnitHttpRequest::post()
                .with_path("/api/test")
                .with_body(b"test-data"),
        );

        assert_eq!(response.status_code(), 200);

        // Check request modifications via backend
        let backend_request = backend.next().expect("backend should receive request");
        assert_eq!(backend_request.header("x-original-method"), Some("POST"));

        // Check request body was modified
        let req_body = String::from_utf8_lossy(backend_request.body());
        assert!(req_body.starts_with("PREFIX-"), "Expected body to start with 'PREFIX-', got: {}", req_body);

        // Response modification: header added
        assert_eq!(
            response.header("x-stop-iteration"),
            Some("response-modified")
        );
        // Response body modified
        let body = String::from_utf8_lossy(response.body());
        assert!(
            body.starts_with("PREFIX:"),
            "Expected body to start with 'PREFIX:', got: {}",
            body
        );
    }
}

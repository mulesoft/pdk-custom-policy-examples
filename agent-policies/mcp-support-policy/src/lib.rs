// Copyright 2023 Salesforce, Inc. All rights reserved.
mod generated;

use crate::generated::config::Config;
use agent_core::http_utils::with_no_timeout;
use anyhow::{anyhow, Result};

use pdk::hl::*;
use serde_json::from_slice;

async fn request_filter(request_state: RequestState) -> Flow<()> {
    let headers_state = request_state.into_headers_state().await;
    let header_handler = headers_state.handler();

    // Remove the timeout
    with_no_timeout(header_handler);

    Flow::Continue(())
}

#[entrypoint]
async fn configure(launcher: Launcher, Configuration(bytes): Configuration) -> Result<()> {
    let _config: Config = from_slice(&bytes).map_err(|err| {
        anyhow!(
            "Failed to parse configuration '{}'. Cause: {}",
            String::from_utf8_lossy(&bytes),
            err
        )
    })?;
    let filter = on_request(request_filter);
    launcher.launch(filter).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use pdk_unit::{UnitHttpRequest, UnitTestBuilder};
    use serde_json::json;

    fn config() -> String {
        json!({}).to_string()
    }

    #[test]
    fn get_request_passes_through() {
        let mut tester = UnitTestBuilder::default()
            .with_config(config())
            .with_entrypoint(crate::configure);

        let response = tester.request(UnitHttpRequest::get());

        assert_eq!(response.status_code(), 200);
    }

    #[test]
    fn post_request_passes_through() {
        let mut tester = UnitTestBuilder::default()
            .with_config(config())
            .with_entrypoint(crate::configure);

        let response = tester.request(
            UnitHttpRequest::post()
                .with_body(json!({"jsonrpc": "2.0", "method": "tools/list", "id": 1}).to_string()),
        );

        assert_eq!(response.status_code(), 200);
    }

    #[test]
    fn request_with_timeout_header_passes_through() {
        let mut tester = UnitTestBuilder::default()
            .with_config(config())
            .with_entrypoint(crate::configure);

        let response = tester.request(
            UnitHttpRequest::post()
                .with_header("x-envoy-upstream-rq-timeout-ms", "5000")
                .with_body(json!({"jsonrpc": "2.0", "method": "tools/call", "id": 2}).to_string()),
        );

        assert_eq!(response.status_code(), 200);
    }
}

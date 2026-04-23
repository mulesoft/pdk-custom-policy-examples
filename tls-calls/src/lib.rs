// Copyright 2023 Salesforce, Inc. All rights reserved.
mod generated;

use anyhow::{anyhow, Result};
use std::str::FromStr;

use pdk::hl::*;

use crate::generated::config::Config;

// This function executes an outbound requests and forwards the response to the caller.
async fn request_filter(client: HttpClient, service: &Service) -> Response {
    match client.request(service).get().await {
        Ok(response) => Response::new(response.status_code()).with_body(response.body()),
        Err(e) => Response::new(503).with_body(e.to_string()),
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

    // Create a service pointing to the upstream that requires custom tls configuration.
    let service = Service::from(
        &config.service.name,
        &config.service.namespace,
        Uri::from_str(&config.service.url)?,
    );

    let filter = on_request(|client| request_filter(client, &service));
    launcher.launch(filter).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use pdk_unit::{UnitHttpMessage, UnitHttpRequest, UnitHttpResponse, UnitTestBuilder};
    use serde_json::json;

    #[test]
    fn upstream_response_is_forwarded() {
        let mut tester = UnitTestBuilder::default()
            .with_config(
                json!({
                    "service": {
                        "name": "backend",
                        "namespace": "default",
                        "url": "http://backend"
                    }
                })
                .to_string(),
            )
            .with_http_upstream("backend.default.svc", |_: UnitHttpRequest| {
                UnitHttpResponse::new(202).with_body("hello")
            })
            .with_entrypoint(crate::configure);

        let response = tester.request(UnitHttpRequest::get());

        assert_eq!(response.status_code(), 202);
        assert_eq!(String::from_utf8_lossy(response.body()), "hello");
    }

    #[test]
    fn upstream_error_returns_503() {
        let mut tester = UnitTestBuilder::default()
            .with_config(
                json!({
                    "service": {
                        "name": "invalid",
                        "namespace": "default",
                        "url": "http://backend"
                    }
                })
                .to_string(),
            )
            .with_http_upstream("backend.default.svc", |_: UnitHttpRequest| {
                UnitHttpResponse::new(500)
            })
            .with_entrypoint(crate::configure);

        let response = tester.request(UnitHttpRequest::get());

        assert_eq!(response.status_code(), 503);
    }
}

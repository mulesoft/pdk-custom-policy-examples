// Copyright 2023 Salesforce, Inc. All rights reserved.

mod common;

use httpmock::MockServer;
use pdk_test::{pdk_test, TestComposite};
use pdk_test::port::Port;
use pdk_test::services::flex::{ApiConfig, FlexConfig, Flex, PolicyConfig};
use pdk_test::services::httpmock::{HttpMockConfig, HttpMock};

use common::*;

// Flex port for the internal test network
const FLEX_PORT: Port = 8081;

// This integration test shows how to build a test to compose a local-flex instance
// with a MockServer backend

#[pdk_test]
async fn test_stop_iteration_modifies_response() -> anyhow::Result<()> {
    // Configure an HttpMock backend
    let httpmock_config = HttpMockConfig::builder()
        .port(80)
        .version("latest")
        .hostname("backend")
        .build();

    // Configure the stop-iteration policy
    let policy_config = PolicyConfig::builder()
        .name(POLICY_NAME)
        .configuration(serde_json::json!({
            "bodyPrefix": "MODIFIED",
            "modifyRequest": false,
            "modifyResponse": true
        }))
        .build();

    let api_config = ApiConfig::builder()
        .name("testApi")
        .upstream(&httpmock_config)
        .path("/anything/echo/")
        .port(FLEX_PORT)
        .policies([policy_config])
        .build();

    // Configure Flex service
    let flex_config = FlexConfig::builder()
        .version("1.12.0")
        .hostname("local-flex")
        .with_api(api_config)
        .config_mounts([
            (POLICY_DIR, "policy"),
            (COMMON_CONFIG_DIR, "common"),
        ])
        .build();

    // Compose the services
    let composite = TestComposite::builder()
        .with_service(flex_config)
        .with_service(httpmock_config)
        .build()
        .await?;

    // Get handles to services
    let flex: Flex = composite.service()?;
    let flex_url = flex.external_url(FLEX_PORT).unwrap();
    let httpmock: HttpMock = composite.service()?;

    // Create a MockServer and mock the backend response
    let mock_server = MockServer::connect_async(httpmock.socket()).await;
    mock_server.mock_async(|when, then| {
        when.path_contains("/hello");
        then.status(200).body("original-body");
    }).await;

    // Make a request through Flex
    let response = reqwest::get(format!("{flex_url}/hello")).await?;

    // Verify the policy modified the response
    assert_eq!(response.status(), 200);

    // Check that the response header was added
    assert_eq!(
        response.headers().get("x-stop-iteration").and_then(|v| v.to_str().ok()),
        Some("response-modified")
    );

    // Check that the response body was modified with the prefix
    let body = response.text().await?;
    assert!(
        body.starts_with("MODIFIED:"),
        "Expected body to start with 'MODIFIED:', got: {}",
        body
    );

    Ok(())
}

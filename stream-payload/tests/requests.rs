// Copyright 2023 Salesforce, Inc. All rights reserved.

mod common;

use httpmock::MockServer;
use pdk_test::port::Port;
use pdk_test::services::flex::{ApiConfig, Flex, FlexConfig, PolicyConfig};
use pdk_test::services::httpmock::{HttpMock, HttpMockConfig};
use pdk_test::{pdk_test, TestComposite};

use common::*;

// Flex port for the internal test network
const FLEX_PORT: Port = 8081;
const FORBIDDEN_STRING: &str = "${jndi";

// This integration test shows how to build a test to compose a local-flex instance
// with a MockServer backend
#[pdk_test]
async fn reject_forbidden() -> anyhow::Result<()> {
    // Configure an HttpMock service
    let httpmock_config = HttpMockConfig::builder()
        .port(80)
        .version("latest")
        .hostname("backend")
        .build();

    let policy_config = PolicyConfig::builder()
        .name(POLICY_NAME)
        .configuration(
            serde_json::json!({"searchMode": "collect", "forbiddenStrings": [FORBIDDEN_STRING]}),
        )
        .build();

    let api_config = ApiConfig::builder()
        .name("myApi")
        .upstream(&httpmock_config)
        .path("/anything/echo/")
        .port(FLEX_PORT)
        .policies([policy_config])
        .build();

    // Configure a Flex service
    let flex_config = FlexConfig::builder()
        .version("1.7.0")
        .hostname("local-flex")
        .with_api(api_config)
        .config_mounts([(POLICY_DIR, "policy"), (COMMON_CONFIG_DIR, "common")])
        .build();

    // Compose the services
    let composite = TestComposite::builder()
        .with_service(flex_config)
        .with_service(httpmock_config)
        .build()
        .await?;

    // Get a handle to the Flex service
    let flex: Flex = composite.service()?;

    // Get an external URL to point the Flex service
    let flex_url = flex.external_url(FLEX_PORT).unwrap();

    // Get a handle to the HttpMock service
    let httpmock: HttpMock = composite.service()?;

    // Create a MockServer
    let mock_server = MockServer::connect_async(httpmock.socket()).await;

    // Mock a /hello request
    mock_server
        .mock_async(|when, then| {
            when.path_contains("/hello");
            then.status(202).body("World!");
        })
        .await;

    assert_request(flex_url.as_str(), generate_string(1024 * 1024 + 8), 202).await?;
    assert_request(
        flex_url.as_str(),
        generate_string(1024 * 512) + FORBIDDEN_STRING + &generate_string(1024 * 512),
        400,
    )
    .await?;

    Ok(())
}

async fn assert_request(url: &str, body: String, status_code: u16) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{url}/hello"))
        .body(body)
        .send()
        .await?;

    assert_eq!(response.status().as_u16(), status_code);
    Ok(())
}

/// generates a string with the request byte size.
fn generate_string(bytes: usize) -> String {
    let mut str = String::with_capacity(bytes);
    for _ in 0..bytes {
        str.push('a'); // Utf-8 encoding of `a` is one byte.
    }

    str
}

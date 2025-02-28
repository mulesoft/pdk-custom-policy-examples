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

// This integration test validates that OpenAI API chat completion requests are
// omitted or blocked by applying the configuration filters.
#[pdk_test]
async fn chat() -> anyhow::Result<()> {
    // Configure an HttpMock service
    let httpmock_config = HttpMockConfig::builder()
        .port(80)
        .version("0.6.8")
        .hostname("backend")
        .build();

    let filter_config = serde_json::json!(
        {
            "filters": [
                {
                    // email
                    "pattern": r#"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}"#,
                    "omitInsteadOfBlocking": true
                },
                {
                    // phone number
                    "pattern": r#"(\+?\d{1,3})?[-.\s]?\(?\d{2,4}\)?[-.\s]?\d{3,4}[-.\s]?\d{4}"#,
                    "omitInsteadOfBlocking": false
                }
            ]
        }
    );

    let policy_config = PolicyConfig::builder()
        .name(POLICY_NAME)
        .configuration(filter_config)
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

    let omit_body = serde_json::json!(
        {
            "model": "llama",
            "messages": [
                {
                    "role": "user",
                    "content": "Their name is PDK"
                }
            ]
        }
    );

    // Create an HTTP client.
    let client = reqwest::Client::new();

    // Mock a /chat request
    let omit_mock = mock_server
        .mock_async(|when, then| {
            when.path_contains("/chat").json_body(omit_body);
            then.status(202).body("World!");
        })
        .await;

    let omit_request_body = serde_json::json!(
        {
            "model": "llama",
            "messages": [
                {
                    "role": "user",
                    "content": "Their email is pdk@flex.com"
                },
                {
                    "role": "user",
                    "content": "Their name is PDK"
                }
            ]
        }
    );

    // Create a chat request with messages to omit
    let response = client
        .post(format!("{flex_url}/chat"))
        .json(&omit_request_body)
        .send()
        .await?;

    // This request must pass
    assert_eq!(response.status(), 202);

    // Assert that the API mock was reached only one time.
    omit_mock.assert_async().await;

    let block_request_body = serde_json::json!(
        {
            "model": "llama",
            "messages": [
                {
                    "role": "user",
                    "content": "Their email is pdk@flex.com and their phone number is +1-212-456-7890"
                },
                {
                    "role": "user",
                    "content": "Their name is PDK"
                }
            ]
        }
    );

    // Create a chat request with messages to block
    let response = client
        .post(format!("{flex_url}/chat"))
        .json(&block_request_body)
        .send()
        .await?;

    // Must return an error
    assert_eq!(response.status(), 403);

    let actual_body: serde_json::Value = response.json().await?;
    let expected_body = serde_json::json!({"error": "Forbidden tokens."});

    assert_eq!(actual_body, expected_body);

    Ok(())
}

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

// This integration test configures prepend and append messages and tests if the 
// request is actually decorated.
#[pdk_test]
async fn chat() -> anyhow::Result<()> {
    // Configure an HttpMock service
    let httpmock_config = HttpMockConfig::builder()
        .port(80)
        .version("latest")
        .hostname("backend")
        .build();

    let config_json = serde_json::json!(
        {
            "prepend": [
                {
                    "role": "system",
                    "content": "prepend content 0."
                },
                {
                    "role": "user",
                    "content": "prepend content 1."
                }
            ],
            "append": [
                {
                    "role": "user",
                    "content": "append content 0."
                }
            ]
        }
    );

    let policy_config = PolicyConfig::builder()
        .name(POLICY_NAME)
        .configuration(config_json)
        .build();

    let api_config = ApiConfig::builder()
        .name("myApi")
        .upstream(&httpmock_config)
        .path("/llm/v1/")
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

    let expected_request_body = serde_json::json!(
        {
            "model": "llama",
            "messages": [
                {
                    "role": "system",
                    "content": "prepend content 0."
                },
                {
                    "role": "user",
                    "content": "prepend content 1."
                },
                {
                    "role": "user",
                    "content": "User content"
                },
                {
                    "role": "user",
                    "content": "append content 0."
                }
            ]
        }
    );

    // Mock a /chat request
    let chat_mock = mock_server
        .mock_async(|when, then| {
            // Add the expected request body to be asserted.
            when.path_contains("/chat").json_body(expected_request_body);

            then.status(200).body("Pong!");
        })
        .await;

    let client = reqwest::Client::new();

    let request_body = serde_json::json!(
        {
            "model": "llama",
            "messages": [
                {
                    "role": "user",
                    "content": "User content"
                }
            ]
        }
    );

    // Perform a chat request
    let response = client
        .post(format!("{flex_url}/chat"))
        .json(&request_body)
        .send()
        .await?;

    // Assert on the response
    assert_eq!(response.status(), 200);

    // Assert hit on the mock.
    chat_mock.assert_hits_async(1).await;

    Ok(())
}

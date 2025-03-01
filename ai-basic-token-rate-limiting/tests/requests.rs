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

// Tests token rate limit
#[pdk_test]
async fn rate_limit() -> anyhow::Result<()> {
    // Configure an HttpMock service
    let httpmock_config = HttpMockConfig::builder()
        .port(80)
        .version("latest")
        .hostname("backend")
        .build();

    let config = serde_json::json!(
        {
            "maximumTokens": 5,
            "timePeriodInMilliseconds": 600000,
        }
    );

    let policy_config = PolicyConfig::builder()
        .name(POLICY_NAME)
        .configuration(config)
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
        .config_mounts([(POLICY_DIR, "policy"), (COMMON_CONFIG_DIR, "config")])
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
            when.path_contains("/chat");
            then.status(200).body("Pong!");
        })
        .await;

    let request_body = serde_json::json!(
        {
            "model": "Llama",
            "messages": [
                {
                    "role": "user",
                    "content": "These are five tokens "
                }
            ]
        }
    );

    let client = reqwest::Client::new();

    // Create a chat request
    let request = client.post(format!("{flex_url}/chat")).json(&request_body);

    // perform the first request with 2 tokens.
    let response = request.try_clone().unwrap().send().await?;

    // Rate limit still not reached
    assert_eq!(response.status(), 200);

    let respone = request.send().await?;

    // Rate limit reached. Must return 403.
    assert_eq!(respone.status(), 403);

    Ok(())
}

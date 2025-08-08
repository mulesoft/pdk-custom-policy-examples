// Copyright 2023 Salesforce, Inc. All rights reserved.

mod common;

use httpmock::MockServer;
use pdk_test::port::Port;
use pdk_test::services::flex::{ApiConfig, Flex, FlexConfig, PolicyConfig};
use pdk_test::services::httpmock::{HttpMock, HttpMockConfig};
use pdk_test::{pdk_test, TestComposite};
use serde_json::json;

use common::*;

// Flex port for the internal test network
const FLEX_PORT: Port = 8081;

// This integration test configures a template and hits Flex
// many times to validate template application.
#[pdk_test]
async fn chat() -> anyhow::Result<()> {
    // Configure an HttpMock service
    let httpmock_config = HttpMockConfig::builder()
        .port(80)
        .version("0.6.8")
        .hostname("backend")
        .build();

    let template = json!(
        {
            "messages": [
              {
                "role": "system",
                "content": "You are a {{system}} expert, in {{species}} species."
              },
              {
                "role": "user",
                "content": "Describe me the {{system}} system."
              }
            ]
        }
    );

    let template_string = template.to_string();

    let json_config = json!(
        {
            "allowUntemplatedRequests": false,
            "templates": [
                {
                    "name": "veterinarian-chat",
                    "template": template_string
                }
            ]
        }
    );

    let policy_config = PolicyConfig::builder()
        .name(POLICY_NAME)
        .configuration(json_config)
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
        .version("1.10.0")
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

    let expected_request_body = json!(
        {
            "messages": [
              {
                "role": "system",
                "content": "You are a respiratory expert, in falcon species."
              },
              {
                "role": "user",
                "content": "Describe me the respiratory system."
              }
            ]
        }
    );

    // Mock a /hello request
    let chat_mock = mock_server
        .mock_async(|when, then| {
            when.path_contains("/prompt")
                .json_body(expected_request_body);
            then.status(202).body("World!");
        })
        .await;

    let client = reqwest::Client::new();

    let request_body = json!(
        {
            "prompt": "{template://veterinarian-chat}",
            "properties": {
                "species": "falcon",
                "system": "respiratory"
            }
        }
    );

    let response = client
        .post(format!("{flex_url}/prompt"))
        .json(&request_body)
        .send()
        .await?;

    // Assert on the response
    assert_eq!(response.status(), 202);
    chat_mock.assert_async().await;

    let unknown_template = json!(
        {
            "prompt": "{template://unknown}",
            "properties": {
                "species": "falcon",
                "system": "respiratory"
            }
        }
    );

    let response = client
        .post(format!("{flex_url}/prompt"))
        .json(&unknown_template)
        .send()
        .await?;

    // Template not found, must return bad request.
    assert_eq!(response.status(), 400);

    Ok(())
}

// Copyright 2023 Salesforce, Inc. All rights reserved.
mod common;

use std::time::Duration;

use httpmock::MockServer;
use pdk_test::port::Port;
use pdk_test::services::flex::{ApiConfig, Flex, FlexConfig, PolicyConfig};
use pdk_test::services::httpmock::{HttpMock, HttpMockConfig};
use pdk_test::{pdk_test, TestComposite};

use common::*;
use serde_json::{json, Value};

// Flex port for the internal test network
const FLEX_PORT: Port = 8081;

const ORG_ID: &str = "126bae0e-ead6-4f5d-8c49-4df7f32f5a33";
const ENV_ID: &str = "2b988881-5f9f-4961-8a2d-8b74f11bcd23";
const API_ID: &str = "123";

fn contracts_path() -> String {
    format!(
        "/apigateway/ccs/v3/organizations/{ORG_ID}/environments/{ENV_ID}/apis/{API_ID}/contracts"
    )
}

fn contract_mock() -> Value {
    let contracts_path = contracts_path();
    json!(
        {
            "links": {
              "self": "self-link",
              "next": contracts_path
            },
            "data": [
              {
                "organizationId": ORG_ID,
                "contractId": "cid",
                "apiId": API_ID,
                "versionId": "",
                "slaTierId": "none",
                "clientId": "pdk",
                "clientSecret": "none",
                "clientSecretSalt": "none",
                "contractUpdatedDate": "none",
                "redirectUris": [],
                "clientName": "none",
                "clientDescription": "none",
                "clientUpdatedDate": "none",
                "updatedDate": "none",
                "removed": null
              }
            ]
          }
    )
}

/// This integration test performs an authentication request.
#[pdk_test]
async fn authentication() -> anyhow::Result<()> {
    // Configure an HttpMock service
    let httpmock_config = HttpMockConfig::builder()
        .port(80)
        .version("0.6.8")
        .hostname("backend")
        .build();

    let json_config = json!({
        "mode": "authentication"
    });

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
        .version("1.8.3")
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
    let backend = MockServer::connect_async(httpmock.socket()).await;

    // Mock the login API
    let login_mock = backend
        .mock_async(|when, then| {
            when.path_contains("/accounts/oauth2/token");

            then.status(200).json_body(json!(
                {
                    "access_token": "your-token",
                    "token_type": "oauth"
                }
            ));
        })
        .await;

    // Mock the contracts API
    let contracts_mock = backend
        .mock_async(|when, then| {
            when.path_contains("/contracts");
            then.status(200).json_body(contract_mock());
        })
        .await;

    // Mock a /hello request
    backend
        .mock_async(|when, then| {
            when.path_contains("/hello");
            then.status(202).json_body(json!("world!"));
        })
        .await;

    // Waits until the contracts database is up to date
    tokio::time::sleep(Duration::from_millis(3000)).await;

    // Perform an actual request
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{flex_url}/hello"))
        .basic_auth("pdk", Some("flexpass"))
        .send()
        .await?;

    login_mock.assert_async().await;
    contracts_mock.assert_async().await;

    //let response = reqwest::get()).await?;

    // contracts_mock.assert_async().await;

    // Assert on the response
    let status = response.status();

    assert_eq!(status, 202);

    Ok(())
}

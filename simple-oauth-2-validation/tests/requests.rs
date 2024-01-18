// Copyright 2023 Salesforce, Inc. All rights reserved.

mod common;
use httpmock::MockServer;
use pdk_test::port::Port;
use pdk_test::services::flex::{Flex, FlexConfig};
use pdk_test::services::httpmock::{HttpMock, HttpMockConfig};
use pdk_test::{pdk_test, TestComposite};

use common::*;
use reqwest::StatusCode;
use serde_json::Value;

// Directory with the configurations for the `hello` test.
const TEST_CONFIG_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/requests/authorization");

// Flex port for the internal test network
const FLEX_PORT: Port = 8081;

async fn assert_response(
    response: reqwest::Response,
    expected_status: StatusCode,
    expected_body: &str,
) {
    let status = response.status();
    let body = String::from_utf8(response.bytes().await.unwrap().to_vec()).unwrap();

    assert_eq!(
        status, expected_status,
        "Expected status {} but got {}. The response body was {}",
        expected_status, status, body
    );
    assert_eq!(
        body, expected_body,
        "Expected body {}, but got {}",
        expected_body, body
    );
}

// This integration test shows how to build a test to compose a local-flex instance
// with a MockServer backend
#[pdk_test]
async fn authorization() -> anyhow::Result<()> {
    // Configuring the Flex service
    let flex_config = FlexConfig::builder()
        .version("1.6.1")
        .hostname("local-flex")
        .ports([FLEX_PORT])
        .config_mounts([
            (POLICY_DIR, "policy"),
            (COMMON_CONFIG_DIR, "common"),
            (TEST_CONFIG_DIR, "authorization"),
        ])
        .build();

    // Configuring the upstream service
    let mock_conf = HttpMockConfig::builder()
        .port(80)
        .version("latest")
        .hostname("mock")
        .build();

    // Compose the services
    let composite = TestComposite::builder()
        .with_service(flex_config)
        .with_service(mock_conf)
        .build()
        .await?;

    // Get a handle to the Flex service
    let flex: Flex = composite.service_by_hostname("local-flex")?;

    // Get an external URL to point the Flex service
    let flex_url = flex.external_url(FLEX_PORT).unwrap();

    let mock_server =
        MockServer::connect_async(composite.service_by_hostname::<HttpMock>("mock")?.socket())
            .await;

    mock_server
        .mock_async(|when, then| {
            when.path_contains("/hello");
            then.status(202).body("World!");
        })
        .await;

    const VALID_TOKEN: &str = "valid";
    const INVALID_TOKEN: &str = "not_vlid";

    mock_server
        .mock_async(|when, then| {
            when.path("/auth").and(|when| {
                when.body(serde_urlencoded::to_string([("token", VALID_TOKEN)]).unwrap())
            });
            then.status(200)
                .json_body(serde_json::from_str::<Value>(r#"{"active": true}"#).unwrap());
        })
        .await;

    mock_server
        .mock_async(|when, then| {
            when.path("/auth").and(|when| {
                when.body(serde_urlencoded::to_string([("token", INVALID_TOKEN)]).unwrap())
            });
            then.status(200)
                .json_body(serde_json::from_str::<Value>(r#"{"active": false}"#).unwrap());
        })
        .await;

    let response = reqwest::Client::new()
        .get(format!("{flex_url}/hello"))
        .header("Authorization", format!("Bearer {}", VALID_TOKEN))
        .send()
        .await?;

    assert_response(response, StatusCode::ACCEPTED, "World!").await;

    let response = reqwest::Client::new()
        .get(format!("{flex_url}/hello"))
        .header("Authorization", format!("Bearer {}", INVALID_TOKEN))
        .send()
        .await?;

    assert_response(response, StatusCode::UNAUTHORIZED, "").await;

    Ok(())
}

// Copyright 2023 Salesforce, Inc. All rights reserved.

mod common;

use httpmock::MockServer;
use pdk_test::port::Port;
use pdk_test::services::flex::{ApiConfig, Flex, FlexConfig, PolicyConfig};
use pdk_test::services::httpmock::{HttpMock, HttpMockConfig};
use pdk_test::{pdk_test, TestComposite};

use common::*;
use reqwest::StatusCode;

// Flex port for the internal test network
const FLEX_PORT: Port = 8081;

// This integration test shows how to build a test to compose a local-flex instance
// with a MockServer backend
#[pdk_test]
async fn caching() -> anyhow::Result<()> {
    let backend_config = HttpMockConfig::builder()
        .port(80)
        .hostname("backend")
        .build();

    // Configure a Flex service
    let policy_config = PolicyConfig::builder()
        .name(POLICY_NAME)
        .configuration(serde_json::json!({
            "max_cached_values": 1,
            // Forcing the interval so that current time is always in range
            "start_hour": 00,
            "end_hour": 23
        }))
        .build();

    let api_config = ApiConfig::builder()
        .name("ingress-http")
        .upstream(&backend_config)
        .path("/anything/echo/")
        .port(FLEX_PORT)
        .policies([policy_config])
        .build();

    let flex_config = FlexConfig::builder()
        .version("1.7.0")
        .hostname("local-flex")
        .with_api(api_config)
        .config_mounts([(POLICY_DIR, "policy"), (COMMON_CONFIG_DIR, "common")])
        .build();

    let composite = TestComposite::builder()
        .with_service(flex_config)
        .with_service(backend_config)
        .build()
        .await?;

    // Get a handle to the Flex service
    let flex: Flex = composite.service()?;

    // Get an external URL to point the Flex service
    let flex_url = flex.external_url(FLEX_PORT).unwrap();

    // Get a handle to the upstream service
    let upstream: HttpMock = composite.service()?;

    // Connect to the handle of the upstream service
    let backend_server = MockServer::connect_async(upstream.socket()).await;

    // First we test that the policy is mocking the first value obtained from upstream
    // server.
    let mut first_value_mock = backend_server
        .mock_async(|when, then| {
            when.path_contains("/route_1");
            then.status(200).body("Value 1");
        })
        .await;

    // The first request should go to the upstream server.
    let response = reqwest::get(format!("{flex_url}/route_1")).await?;

    assert_response(response, StatusCode::OK, "Value 1").await;

    // The next request should return the same content, but without hitting the upstream server.
    let response = reqwest::get(format!("{flex_url}/route_1")).await?;

    // The hits should not increase
    first_value_mock.assert_hits(1);
    assert_response(response, StatusCode::OK, "Value 1").await;

    // Now we test that a different route request will get a cache miss, and the actual upstream
    // server.

    // Removing mock to override it
    first_value_mock.delete();

    let second_value_mock = backend_server
        .mock_async(|when, then| {
            when.path_contains("/route_1");
            then.status(200).body("Value 2");
        })
        .await;

    let response: reqwest::Response = reqwest::get(format!("{flex_url}/route_1")).await?;

    second_value_mock.assert_hits(0);
    assert_response(response, StatusCode::OK, "Value 1").await;

    // Now we test a different route that will respond with different content

    let alt_route_mock = backend_server
        .mock_async(|when, then| {
            when.path_contains("/route_2");
            then.status(200).body("Alt route value 1");
        })
        .await;

    let response: reqwest::Response = reqwest::get(format!("{flex_url}/route_2")).await?;

    alt_route_mock.assert_hits(1);
    assert_response(response, StatusCode::OK, "Alt route value 1").await;

    // Finally, since we surpassed the max entries, the first route value should not be cached
    // anymore and should return the changed value

    let response: reqwest::Response = reqwest::get(format!("{flex_url}/route_1")).await?;

    second_value_mock.assert_hits(1);
    assert_response(response, StatusCode::OK, "Value 2").await;

    Ok(())
}

async fn assert_response(
    response: reqwest::Response,
    expected_status: StatusCode,
    expected_body: &str,
) {
    let status = response.status();
    let body = response.text().await.unwrap();

    assert_eq!(status, expected_status);
    assert_eq!(body, expected_body);
}

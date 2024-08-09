// Copyright 2023 Salesforce, Inc. All rights reserved.

use httpmock::MockServer;
use pdk_test::port::Port;
use pdk_test::services::flex::{ApiConfig, Flex, FlexConfig, PolicyConfig, UpstreamServiceConfig};
use pdk_test::services::gripmock::{GripMock, GripMockConfig};
use pdk_test::services::httpmock::{HttpMock, HttpMockConfig};
use pdk_test::{pdk_test, TestComposite, TestError};
use reqwest::{Error, Response, StatusCode};
use serde_json::json;

use common::*;

mod common;

// Flex port for the internal test network
const FLEX_PORT: Port = 8081;

const VALID_TOKEN: &str = "valid";
const INVALID_TOKEN: &str = "not_valid";
const PROTO_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/proto/auth.proto");

// This integration test shows how to build a test to compose a local-flex instance
// with a MockServer backend
#[pdk_test]
async fn token_from_header() -> anyhow::Result<()> {
    // Configure the upstream service
    let upstream_config = HttpMockConfig::builder().port(80).hostname("mock").build();

    // Configure the gripmock service
    let gripmock_config = GripMockConfig::builder()
        .hostname("gripmock")
        .proto(PROTO_PATH)
        .build();

    // Configure a Flex service
    let policy_config = PolicyConfig::builder()
        .name(POLICY_NAME)
        .configuration(serde_json::json!({
            "oauthService": "h2://gripmock:4770",
            "authorization": "whatever"
        }))
        .build();

    // Configure Flex Gateway
    let flex_config = flex_config(policy_config, &upstream_config, &gripmock_config);

    // Compose the services
    let composite = setup_services(flex_config, upstream_config, gripmock_config).await?;

    // Get a handle to the Flex service
    let flex: Flex = composite.service()?;

    // Get a handle to the HttpMock service
    let httpmock: HttpMock = composite.service()?;

    // Get a handle to the upstream service
    let upstream = MockServer::connect_async(httpmock.socket()).await;

    // Get a handle to the GripMock service
    let gripmock: GripMock = composite.service()?;

    // Mock upstream service interactions
    mock_backend_path(&upstream).await;

    // Mock authorization service interactions
    mock_auth_server_path(&gripmock).await?;

    // Get an external URL to point the Flex service
    let flex_url = flex.external_url(FLEX_PORT).unwrap();

    let response = request(format!("{flex_url}/hello").as_str(), VALID_TOKEN).await?;
    assert_response(response, StatusCode::ACCEPTED, "World!").await;

    let response = request(format!("{flex_url}/hello").as_str(), INVALID_TOKEN).await?;
    assert_response(response, StatusCode::UNAUTHORIZED, "").await;

    Ok(())
}

#[pdk_test]
async fn token_from_query_parameter() -> anyhow::Result<()> {
    // Configure the upstream service
    let upstream_config = HttpMockConfig::builder().port(80).hostname("mock").build();

    // Configure a Flex service
    let policy_config = PolicyConfig::builder()
        .name(POLICY_NAME)
        .configuration(serde_json::json!({
            "oauthService": "h2://gripmock:4770",
            "authorization": "whatever",
            "tokenExtractor": "#[attributes.queryParams.token]"
        }))
        .build();

    // Configure the gripmock service
    let gripmock_config = GripMockConfig::builder()
        .hostname("gripmock")
        .proto(PROTO_PATH)
        .build();

    // Configure Flex Gateway
    let flex_config = flex_config(policy_config, &upstream_config, &gripmock_config);

    // Compose the services
    let composite = setup_services(flex_config, upstream_config, gripmock_config).await?;

    // Get a handle to the Flex service
    let flex: Flex = composite.service()?;

    // Get a handle to the GripMock service
    let gripmock: GripMock = composite.service()?;

    // Get a handle to the HttpMock service
    let httpmock: HttpMock = composite.service()?;

    // Get a handle to the upstream service
    let upstream = MockServer::connect_async(httpmock.socket()).await;

    // Mock upstream service interactions
    mock_backend_path(&upstream).await;

    // Mock authorization service interactions
    mock_auth_server_path(&gripmock).await?;

    // Get an external URL to point the Flex service
    let flex_url = flex.external_url(FLEX_PORT).unwrap();

    match request_query_param(format!("{flex_url}/hello").as_str(), VALID_TOKEN).await {
        Ok(response) => assert_response(response, StatusCode::ACCEPTED, "World!").await,
        Err(err) => {
            panic!("Error: {:?}", err)
        }
    }

    let response = request_query_param(format!("{flex_url}/hello").as_str(), INVALID_TOKEN).await?;
    assert_response(response, StatusCode::UNAUTHORIZED, "").await;

    Ok(())
}

async fn mock_auth_server_path(gripmock: &GripMock) -> anyhow::Result<()> {
    let client = reqwest::Client::new();

    let response = client
        .post(format!("{}/add", gripmock.address()))
        .json(&json!([
            {
                "service": "AuthService",
                "method": "Check",
                "input": {
                    "equals": {
                        "token": VALID_TOKEN
                    }
                },
                "output": {
                    "data": {
                        "active": true
                    }
                }
            }, 
            {
                "service": "AuthService",
                "method": "Check",
                "input": {
                    "equals": {
                        "token": INVALID_TOKEN
                    }
                },
                "output": {
                    "data": {
                        "active": false
                    }
                }
            }
        ]))
        .send()
        .await?;

    assert!(response.status().is_success());

    Ok(())
}

async fn mock_backend_path(upstream: &MockServer) {
    upstream
        .mock_async(|when, then| {
            when.path_contains("/hello");
            then.status(202).body("World!");
        })
        .await;
}

/// Configuring the Flex service
fn flex_config(
    policy_config: PolicyConfig,
    upstream_config: &HttpMockConfig,
    gripmock_config: &GripMockConfig,
) -> FlexConfig {
    let grpc_service = UpstreamServiceConfig::builder()
        .address(format!("h2://{}:4770", gripmock_config.hostname()))
        .name(gripmock_config.hostname())
        .build();

    let api_config = ApiConfig::builder()
        .name("ingress-http")
        .upstream(upstream_config)
        .path("/anything/echo/")
        .port(FLEX_PORT)
        .policies([policy_config])
        .build();

    FlexConfig::builder()
        .version("1.7.0")
        .hostname("local-flex")
        .with_api(api_config)
        .with_upstream_service(grpc_service)
        .config_mounts([(POLICY_DIR, "policy"), (COMMON_CONFIG_DIR, "common")])
        .build()
}

/// Creates and initializes the services for performing the test
async fn setup_services(
    flex_config: FlexConfig,
    upstream_config: HttpMockConfig,
    gripmock_config: GripMockConfig,
) -> Result<TestComposite, TestError> {
    TestComposite::builder()
        .with_service(flex_config)
        .with_service(upstream_config)
        .with_service(gripmock_config)
        .build()
        .await
}

async fn request_query_param(url: &str, token: &str) -> Result<Response, Error> {
    let query = vec![("token", token)];
    reqwest::Client::new().get(url).query(&query).send().await
}

async fn request(url: &str, token: &str) -> Result<Response, Error> {
    reqwest::Client::new()
        .get(url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
}

async fn assert_response(response: Response, expected_status: StatusCode, expected_body: &str) {
    let status = response.status();
    let body = response.text().await.unwrap();

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

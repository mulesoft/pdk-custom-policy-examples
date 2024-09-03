// Copyright 2023 Salesforce, Inc. All rights reserved.

use auth::{AuthRequest, AuthResponse};
use httpmock::MockServer;
use pdk_test::port::Port;
use pdk_test::services::flex::{ApiConfig, Flex, FlexConfig, PolicyConfig};
use pdk_test::services::gripmock::{GripMock, GripMockConfig};
use pdk_test::services::httpmock::{HttpMock, HttpMockConfig};
use pdk_test::{pdk_test, TestComposite};
use reqwest::{Error, Response, StatusCode};
use serde_json::json;

use common::*;

mod common;

// Flex port for the internal test network
const FLEX_PORT: Port = 8081;

// Authentication token.
const AUTH_TOKEN: &str = "my_token";

// File containing the protobuf definitions.
const PROTO_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/proto/auth.proto");

// Import the protobuf generated files.
include!(concat!(env!("OUT_DIR"), "/protos/mod.rs"));

#[pdk_test]
async fn accept_token() -> anyhow::Result<()> {
    // Configure the upstream service
    let upstream_config = HttpMockConfig::builder().port(80).hostname("mock").build();

    // Configure the gripmock service
    let gripmock_config = GripMockConfig::builder()
        .version("v1.13")
        .hostname("gripmock")
        .proto(PROTO_PATH)
        .build();

    let policy_config = PolicyConfig::builder()
        .name(POLICY_NAME)
        .configuration(serde_json::json!({
                    "oauthService": "h2://gripmock:4770",
                    "authorization": "whatever",
                    "tokenExtractor": "#[attributes.queryParams.token]"
        }))
        .build();

    let api_config = ApiConfig::builder()
        .name("ingress-http")
        .upstream(&upstream_config)
        .path("/anything/echo/")
        .port(FLEX_PORT)
        .policies([policy_config])
        .build();

    // Configure Flex Gateway
    let flex_config = FlexConfig::builder()
        .version("1.7.0")
        .hostname("local-flex")
        .with_api(api_config)
        .config_mounts([(POLICY_DIR, "policy"), (COMMON_CONFIG_DIR, "common")])
        .build();

    // Compose the services
    let composite = TestComposite::builder()
        .with_service(flex_config)
        .with_service(upstream_config)
        .with_service(gripmock_config)
        .build()
        .await?;

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

    // Get an external URL to point the Flex service
    let flex_url = flex.external_url(FLEX_PORT).unwrap();

    let auth_request = AuthRequest {
        token: AUTH_TOKEN.to_string(),
        ..Default::default()
    };

    let auth_response = AuthResponse {
        active: true,
        ..Default::default()
    };

    // Mock valid token request
    mock_auth_service(&gripmock, auth_request, auth_response).await?;

    let response = request_query_param(format!("{flex_url}/hello").as_str(), AUTH_TOKEN).await?;
    assert_response(response, StatusCode::ACCEPTED, "World!").await;

    Ok(())
}

async fn mock_auth_service(
    gripmock: &GripMock,
    auth_request: AuthRequest,
    auth_response: AuthResponse,
) -> anyhow::Result<()> {
    let client = reqwest::Client::new();

    let response = client
        .post(format!("{}/add", gripmock.address()))
        .json(&json!(
            {
                "service": "AuthService",
                "method": "Check",
                "input": {
                    "equals": {
                        "token": auth_request.token
                    }
                },
                "output": {
                    "data": {
                        "active": auth_response.active
                    }
                }
            }
        ))
        .send()
        .await?;

    assert_eq!(response.status(), 200);

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

async fn request_query_param(url: &str, token: &str) -> Result<Response, Error> {
    let query = vec![("token", token)];
    reqwest::Client::new().get(url).query(&query).send().await
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

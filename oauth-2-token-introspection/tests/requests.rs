// Copyright 2023 Salesforce, Inc. All rights reserved.

mod common;

use httpmock::MockServer;
use pdk_test::{pdk_test, TestComposite, TestError};
use pdk_test::port::Port;
use pdk_test::services::flex::{ApiConfig, FlexConfig, Flex, PolicyConfig};
use pdk_test::services::httpmock::{HttpMockConfig, HttpMock};
use reqwest::{Error, Response, StatusCode};
use serde_json::json;
use common::*;

const FLEX_PORT: Port = 8081;
const VALID_TOKEN: &str = "valid";
const INVALID_TOKEN: &str = "not_valid";

#[pdk_test]
async fn token_from_header() -> anyhow::Result<()> {
    // Configure the upstream service
    let upstream_config = HttpMockConfig::builder().port(80).hostname("mock").build();

    // Configure a Flex service
    let policy_config = PolicyConfig::builder()
        .name(POLICY_NAME)
        .configuration(serde_json::json!({
                    "introspectionService": "http://mock/auth",
                    "introspectionPath": "/introspect",
                    "authorizationValue": "Basic YWRtaW46YWRtaW4=",
                    "expiresInAttribute": "exp",
                    "validatedTokenTTL": 600,
                    "authenticationTimeout": 10000,
                    "exposeHeaders": true,
                    "scopes": "read write",
                    "scopeValidationCriteria": "AND",
                    "maxCacheEntries": 10000
        }))
        .build();

    // Configure Flex Gateway
    let flex_config = flex_config(&upstream_config, policy_config);

    // Compose the services
    let composite = setup_services(flex_config, upstream_config).await?;

    // Get a handle to the Flex service
    let flex: Flex = composite.service_by_hostname("local-flex")?;

    // Get a handle to the upstream service
    let upstream =
        MockServer::connect_async(composite.service_by_hostname::<HttpMock>("mock")?.socket())
            .await;

    // Mock upstream service interactions
    mock_backend_path(&upstream).await;

    // Mock authorization service interactions
    mock_auth_server_path(&upstream).await;

    // Get an external URL to point the Flex service
    let flex_url = flex.external_url(FLEX_PORT).unwrap();

    let response = request(format!("{flex_url}/hello").as_str(), VALID_TOKEN).await?;
    assert_response(response, StatusCode::ACCEPTED, "World!").await;

    let response = request(format!("{flex_url}/hello").as_str(), INVALID_TOKEN).await?;
    assert_response(response, StatusCode::UNAUTHORIZED, "{\"error\":\"Token has been revoked.\"}").await;

    Ok(())
}

#[pdk_test]
async fn invalid_auth_value() -> anyhow::Result<()> {
    // Configure the upstream service
    let upstream_config = HttpMockConfig::builder().port(80).hostname("mock").build();

    // Configure a Flex service
    let policy_config = PolicyConfig::builder()
        .name(POLICY_NAME)
        .configuration(serde_json::json!({
                    "introspectionService": "http://mock/auth",
                    "introspectionPath": "/introspect",
                    "authorizationValue": "Basic invalid",
                    "expiresInAttribute": "exp",
                    "validatedTokenTTL": 600,
                    "authenticationTimeout": 10000,
                    "exposeHeaders": true,
                    "scopes": "read write",
                    "scopeValidationCriteria": "AND",
                    "maxCacheEntries": 10000
        }))
        .build();

    // Configure Flex Gateway
    let flex_config = flex_config(&upstream_config, policy_config);

    // Compose the services
    let composite = setup_services(flex_config, upstream_config).await?;

    // Get a handle to the Flex service
    let flex: Flex = composite.service_by_hostname("local-flex")?;

    // Get a handle to the upstream service
    let upstream =
        MockServer::connect_async(composite.service_by_hostname::<HttpMock>("mock")?.socket())
            .await;

    // Mock upstream service interactions
    mock_backend_path(&upstream).await;

    // Mock authorization service interactions
    mock_auth_server_path(&upstream).await;

    // Get an external URL to point the Flex service
    let flex_url = flex.external_url(FLEX_PORT).unwrap();

    let response = request(format!("{flex_url}/hello").as_str(), VALID_TOKEN).await?;
    assert_response(response, StatusCode::UNAUTHORIZED, "{\"error\":\"Token has been revoked.\"}").await;

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
                    "introspectionService": "http://mock/auth",
                    "introspectionPath": "/introspect",
                    "authorizationValue": "Basic YWRtaW46YWRtaW4=",
                    "expiresInAttribute": "exp",
                    "validatedTokenTTL": 600,
                    "authenticationTimeout": 10000,
                    "exposeHeaders": true,
                    "scopes": "read write",
                    "scopeValidationCriteria": "AND",
                    "maxCacheEntries": 10000
        }))
        .build();

    // Configure Flex Gateway
    let flex_config = flex_config(&upstream_config, policy_config);

    // Compose the services
    let composite = setup_services(flex_config, upstream_config).await?;

    // Get a handle to the Flex service
    let flex: Flex = composite.service_by_hostname("local-flex")?;

    // Get a handle to the upstream service
    let upstream =
        MockServer::connect_async(composite.service_by_hostname::<HttpMock>("mock")?.socket())
            .await;

    // Mock upstream service interactions
    mock_backend_path(&upstream).await;

    // Mock authorization service interactions
    mock_auth_server_path(&upstream).await;

    // Get an external URL to point the Flex service
    let flex_url = flex.external_url(FLEX_PORT).unwrap();

    match request_query_param(format!("{flex_url}/hello").as_str(), VALID_TOKEN).await {
        Ok(response) => assert_response(response, StatusCode::ACCEPTED, "World!").await,
        Err(err) => {
            panic!("Error: {:?}", err)
        }
    }

    let response = request_query_param(format!("{flex_url}/hello").as_str(), INVALID_TOKEN).await?;
    assert_response(response, StatusCode::UNAUTHORIZED, "{\"error\":\"Token has been revoked.\"}").await;

    Ok(())
}

async fn mock_auth_server_path(upstream: &MockServer) {
    upstream
        .mock_async(|when, then| {
            when.path("/introspect")
                .header("Authorization", "Basic YWRtaW46YWRtaW4=")
                .body(serde_urlencoded::to_string([("token", VALID_TOKEN)]).unwrap());
            then.status(200).json_body(json!({"active": true, "scope": "read write"}));
        })
        .await;

    upstream
        .mock_async(|when, then| {
            when.path("/introspect")
                .body(serde_urlencoded::to_string([("token", INVALID_TOKEN)]).unwrap());
            then.status(200).json_body(json!({"active": false}));
        })
        .await;

    upstream
        .mock_async(|when, then| {
            when.path("/introspect")
                .header("Authorization", "Basic invalid");
            then.status(200).json_body(json!({"active": false}));
        })
        .await;
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
fn flex_config(upstream_config: &HttpMockConfig, policy_config: PolicyConfig) -> FlexConfig {
    let api_config = ApiConfig::builder()
        .name("ingress-http")
        .upstream(upstream_config)
        .path("/anything/echo/")
        .port(FLEX_PORT)
        .policies([policy_config])
        .build();

    FlexConfig::builder()
        .version("1.10.0")
        .hostname("local-flex")
        .with_api(api_config)
        .config_mounts([(POLICY_DIR, "policy"), (COMMON_CONFIG_DIR, "common")])
        .build()
}

/// Creates and initializes the services for performing the test
async fn setup_services(
    flex_config: FlexConfig,
    upstream_config: HttpMockConfig,
) -> Result<TestComposite, TestError> {
    TestComposite::builder()
        .with_service(flex_config)
        .with_service(upstream_config)
        .build()
        .await
}

async fn request_query_param(url: &str, token: &str) -> Result<Response, Error> {
    let query = vec![("access_token", token)];
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
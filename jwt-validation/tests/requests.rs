// Copyright 2023 Salesforce, Inc. All rights reserved.

mod common;

use httpmock::MockServer;
use pdk_test::port::Port;
use pdk_test::services::flex::{ApiConfig, Flex, FlexConfig, PolicyConfig};
use pdk_test::services::httpmock::{HttpMock, HttpMockConfig};
use pdk_test::{pdk_test, TestComposite};

use common::*;
use reqwest::StatusCode;
use serde_json::json;

// Flex port for the internal test network
const FLEX_PORT: Port = 8081;

#[pdk_test]
async fn validate_token() -> anyhow::Result<()> {
    // Configure an HttpMock service
    let upstream_config = HttpMockConfig::builder()
        .port(80)
        .hostname("backend")
        .build();

    // Configure a Flex service
    let policy_config = PolicyConfig::builder()
        .name(POLICY_NAME)
        .configuration(serde_json::json!({"secret": "Make-sure-this-is-a-secret-dont-share-it"}))
        .build();

    let api_config = ApiConfig::builder()
        .name("ingress-http")
        .upstream(&upstream_config)
        .path("/anything/echo/")
        .port(FLEX_PORT)
        .policies([policy_config])
        .build();

    let flex_config = FlexConfig::builder()
        .image_name("artifacts.msap.io/mulesoft/microgateway").version("v1.10.1.26e83c4")
        .hostname("local-flex")
        .with_api(api_config)
        .config_mounts([(POLICY_DIR, "policy"), (COMMON_CONFIG_DIR, "common")])
        .build();

    // Compose the services
    let composite = TestComposite::builder()
        .with_service(flex_config)
        .with_service(upstream_config)
        .build()
        .await?;

    // Get a handle to the Flex service
    let flex: Flex = composite.service()?;

    // Get an external URL to point the Flex service
    let flex_url = flex.external_url(FLEX_PORT).unwrap();

    // Get a handle to the upstream service
    let upstream: HttpMock = composite.service()?;

    // Connect to the handle of the upstream service
    let upstream_server = MockServer::connect_async(upstream.socket()).await;

    upstream_server
        .mock_async(|when, then| {
            when.header("Username", "LibraryFan1984");
            then.status(200)
                .json_body(json!({"Username": "LibraryFan1984"}));
        })
        .await;

    // Upon receiving a valid token, assert the echo service
    // response body contains the header produced with JWT claims content
    assert_request(
        flex_url.as_str(),
        &valid_token(),
        StatusCode::OK,
        r#""Username":"LibraryFan1984""#,
    )
    .await?;

    // Validate the response when the token is expired
    assert_request(
        flex_url.as_str(),
        &expired_token(),
        StatusCode::UNAUTHORIZED,
        "Expired token",
    )
    .await?;

    // Validate the response when the bearer token is missing
    assert_request(
        flex_url.as_str(),
        "",
        StatusCode::UNAUTHORIZED,
        "Bearer not found",
    )
    .await?;

    // Validate the response when the token signature is corrupt
    assert_request(
        flex_url.as_str(),
        &invalid_signature_token(),
        StatusCode::UNAUTHORIZED,
        "Invalid token",
    )
    .await?;

    // Validate the response when the custom claim "role" is not "member"
    assert_request(
        flex_url.as_str(),
        &admin_token(),
        StatusCode::BAD_REQUEST,
        "Invalid token: Only members are allowed",
    )
    .await?;

    Ok(())
}

/// Returns valid JWT token signed using the configured HMAC secret
/// Headers
/// {"alg": "HS256", "typ": "JWT", "classid": 439}
/// Payload
/// {
///   "iss": "Library",
///   "sub": "12345",
///   "aud": "member-group",
///   "iat": 1704460407,
///   "nbf": 1704460407,
///   "exp": 2704460407,
///   "username": "LibraryFan1984",
///   "role": "Member"
/// }
fn valid_token() -> String {
    include_str!("resources/valid_token.txt").trim().to_string()
}

fn expired_token() -> String {
    include_str!("resources/expired_token.txt")
        .trim()
        .to_string()
}

// An admin role JWT token with signed using the configured HMAC secret
fn admin_token() -> String {
    include_str!("resources/admin_token.txt").trim().to_string()
}

// A token with invalid signature
fn invalid_signature_token() -> String {
    include_str!("resources/invalid_signature_token.txt")
        .trim()
        .to_string()
}

async fn assert_request(
    flex_url: &str,
    token: &str,
    expected_status: StatusCode,
    expected_body: &str,
) -> anyhow::Result<()> {
    let response = reqwest::Client::new()
        .get(format!("{flex_url}/hello"))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    let status = response.status();
    let body = response.text().await.unwrap();
    // Assert on the response
    assert_eq!(
        status, expected_status,
        "Expected {} but got {}. Respose body was: \"{}\"",
        expected_status, status, body
    );
    assert!(
        body.contains(expected_body),
        "Error: Expected body: {} to contain {}",
        body,
        expected_body
    );

    Ok(())
}

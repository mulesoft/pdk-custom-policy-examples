// Copyright 2023 Salesforce, Inc. All rights reserved.

mod common;

use pdk_test::port::Port;
use pdk_test::services::flex::{Flex, FlexConfig};
use pdk_test::services::httpbin::HttpBinConfig;
use pdk_test::{pdk_test, TestComposite};

use common::*;
use reqwest::StatusCode;

// Directory with the configurations for the `hello` test.
const TESTS_CONFIG_DIR: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/requests/validate_token");

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

// Flex port for the internal test network
const FLEX_PORT: Port = 8081;

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
    let body = String::from_utf8(response.bytes().await.unwrap().to_vec()).unwrap();
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

#[pdk_test]
async fn validate_token() -> anyhow::Result<()> {
    // Configure a Flex service
    let flex_config = FlexConfig::builder()
        .version("1.6.1")
        .hostname("local-flex")
        .ports([FLEX_PORT])
        .config_mounts([
            (POLICY_DIR, "policy"),
            (COMMON_CONFIG_DIR, "common"),
            (TESTS_CONFIG_DIR, "validate_token"),
        ])
        .build();

    // Configure an HttpMock service
    let httpbin_config = HttpBinConfig::builder()
        .version("latest")
        .hostname("backend")
        .build();

    // Compose the services
    let composite = TestComposite::builder()
        .with_service(flex_config)
        .with_service(httpbin_config)
        .build()
        .await?;

    let flex: Flex = composite.service()?;

    let flex_url = flex.external_url(FLEX_PORT).unwrap();

    // Upon receiving a valid token, assert the echo service
    // response body contains the header produced with JWT claims content
    assert_request(
        flex_url.as_str(),
        &valid_token(),
        StatusCode::OK,
        r#""Username": "LibraryFan1984""#,
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
        "Invalid token: Only authenticated customers allowed",
    )
    .await?;

    Ok(())
}

// Copyright 2023 Salesforce, Inc. All rights reserved.

mod common;

use httpmock::MockServer;
use pdk_test::port::Port;
use pdk_test::services::flex::{ApiConfig, Flex, FlexConfig, PolicyConfig};
use pdk_test::services::httpmock::{HttpMock, HttpMockConfig};
use pdk_test::{pdk_test, TestComposite};
use reqwest::StatusCode;

use common::*;

const FLEX_PORT: Port = 8081;

// Test basic rate limiting functionality with API key selector
#[pdk_test]
async fn test_basic_rate_limiting_with_api_key() -> anyhow::Result<()> {
    let backend_config = HttpMockConfig::builder()
        .port(80)
        .hostname("backend")
        .build();

    let policy_config = PolicyConfig::builder()
        .name(POLICY_NAME)
        .configuration(serde_json::json!({
            "rate_limits": [
                {
                    "group_name": "api",
                    "requests_per_window": 2,
                    "window_size_seconds": 60,
                    "key_selector": "api_key",
                    "path_pattern": "*"
                }
            ]
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
        .hostname("local-flex-api-key")
        .with_api(api_config)
        .config_mounts([(POLICY_DIR, "policy"), (COMMON_CONFIG_DIR, "common")])
        .build();

    let composite = TestComposite::builder()
        .with_service(flex_config)
        .with_service(backend_config)
        .build()
        .await?;

    let flex: Flex = composite.service()?;
    let flex_url = flex.external_url(FLEX_PORT).unwrap();
    let upstream: HttpMock = composite.service()?;
    let backend_server = MockServer::connect_async(upstream.socket()).await;

    backend_server
        .mock_async(|when, then| {
            when.path_contains("/test");
            then.status(200).body("OK");
        })
        .await;

    let client = reqwest::Client::new();

    // First request with API key - should be allowed
    let response = client
        .get(format!("{flex_url}/test"))
        .header("x-api-key", "key-123")
        .send()
        .await?;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.text().await?, "OK");

    // Second request with same API key - should be allowed
    let response = client
        .get(format!("{flex_url}/test"))
        .header("x-api-key", "key-123")
        .send()
        .await?;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.text().await?, "OK");

    // Third request with same API key - should be blocked (rate limit exceeded)
    let response = client
        .get(format!("{flex_url}/test"))
        .header("x-api-key", "key-123")
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);

    // Different API key - should be allowed (separate rate limit)
    let response = client
        .get(format!("{flex_url}/test"))
        .header("x-api-key", "key-456")
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::OK);

    Ok(())
}

// Test rate limiting with user_id selector
#[pdk_test]
async fn test_rate_limiting_with_user_id() -> anyhow::Result<()> {
    let backend_config = HttpMockConfig::builder()
        .port(80)
        .hostname("backend")
        .build();

    let policy_config = PolicyConfig::builder()
        .name(POLICY_NAME)
        .configuration(serde_json::json!({
            "rate_limits": [
                {
                    "group_name": "user",
                    "requests_per_window": 3,
                    "window_size_seconds": 60,
                    "key_selector": "user_id",
                    "path_pattern": "*"
                }
            ]
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
        .hostname("local-flex-user-id")
        .with_api(api_config)
        .config_mounts([(POLICY_DIR, "policy"), (COMMON_CONFIG_DIR, "common")])
        .build();

    let composite = TestComposite::builder()
        .with_service(flex_config)
        .with_service(backend_config)
        .build()
        .await?;

    let flex: Flex = composite.service()?;
    let flex_url = flex.external_url(FLEX_PORT).unwrap();
    let upstream: HttpMock = composite.service()?;
    let backend_server = MockServer::connect_async(upstream.socket()).await;

    backend_server
        .mock_async(|when, then| {
            when.path_contains("/test");
            then.status(200).body("OK");
        })
        .await;

    let client = reqwest::Client::new();

    // First request with user ID - should be allowed
    let response = client
        .get(format!("{flex_url}/test"))
        .header("x-user-id", "user-123")
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::OK);

    // Second request with same user ID - should be allowed
    let response = client
        .get(format!("{flex_url}/test"))
        .header("x-user-id", "user-123")
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::OK);

    // Third request with same user ID - should be allowed
    let response = client
        .get(format!("{flex_url}/test"))
        .header("x-user-id", "user-123")
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::OK);

    // Fourth request with same user ID - should be blocked (rate limit exceeded)
    let response = client
        .get(format!("{flex_url}/test"))
        .header("x-user-id", "user-123")
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);

    // Different user ID - should be allowed (separate rate limit)
    let response = client
        .get(format!("{flex_url}/test"))
        .header("x-user-id", "user-456")
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::OK);

    Ok(())
}

// Test simple rate limiting with API key selector
#[pdk_test]
async fn test_simple_rate_limiting() -> anyhow::Result<()> {
    let backend_config = HttpMockConfig::builder()
        .port(80)
        .hostname("backend")
        .build();

    let policy_config = PolicyConfig::builder()
        .name(POLICY_NAME)
        .configuration(serde_json::json!({
            "rate_limits": [
                {
                    "group_name": "api",
                    "requests_per_window": 2,
                    "window_size_seconds": 60,
                    "key_selector": "api_key"
                }
            ]
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
        .hostname("local-flex-multiple")
        .with_api(api_config)
        .config_mounts([(POLICY_DIR, "policy"), (COMMON_CONFIG_DIR, "common")])
        .build();

    let composite = TestComposite::builder()
        .with_service(flex_config)
        .with_service(backend_config)
        .build()
        .await?;

    let flex: Flex = composite.service()?;
    let flex_url = flex.external_url(FLEX_PORT).unwrap();
    let upstream: HttpMock = composite.service()?;
    let backend_server = MockServer::connect_async(upstream.socket()).await;

    backend_server
        .mock_async(|when, then| {
            when.path_contains("/test");
            then.status(200).body("OK");
        })
        .await;

    let client = reqwest::Client::new();

    // Test API rate limit (api_key selector) - 2 requests allowed

    // First request - should be allowed
    let response = client
        .get(format!("{flex_url}/test"))
        .header("x-api-key", "key-123")
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::OK);

    // Second request - should be allowed
    let response = client
        .get(format!("{flex_url}/test"))
        .header("x-api-key", "key-123")
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::OK);

    // Third request - should be blocked (API rate limit exceeded)
    let response = client
        .get(format!("{flex_url}/test"))
        .header("x-api-key", "key-123")
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);

    Ok(())
}

// Test missing headers handling
#[pdk_test]
async fn test_missing_headers() -> anyhow::Result<()> {
    let backend_config = HttpMockConfig::builder()
        .port(80)
        .hostname("backend")
        .build();

    let policy_config = PolicyConfig::builder()
        .name(POLICY_NAME)
        .configuration(serde_json::json!({
            "rate_limits": [
                {
                    "group_name": "api",
                    "requests_per_window": 5,
                    "window_size_seconds": 60,
                    "key_selector": "api_key",
                    "path_pattern": "*"
                }
            ]
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
        .hostname("local-flex-headers")
        .with_api(api_config)
        .config_mounts([(POLICY_DIR, "policy"), (COMMON_CONFIG_DIR, "common")])
        .build();

    let composite = TestComposite::builder()
        .with_service(flex_config)
        .with_service(backend_config)
        .build()
        .await?;

    let flex: Flex = composite.service()?;
    let flex_url = flex.external_url(FLEX_PORT).unwrap();
    let upstream: HttpMock = composite.service()?;
    let backend_server = MockServer::connect_async(upstream.socket()).await;

    backend_server
        .mock_async(|when, then| {
            when.path_contains("/test");
            then.status(200).body("OK");
        })
        .await;

    let client = reqwest::Client::new();

    // Request without API key header - should use "unknown" as key
    let response = client.get(format!("{flex_url}/test")).send().await?;
    assert_eq!(response.status(), StatusCode::OK);

    // Another request without API key - should be allowed (separate "unknown" key)
    let response = client.get(format!("{flex_url}/test")).send().await?;
    assert_eq!(response.status(), StatusCode::OK);

    // Request with empty API key - should use "unknown" as key
    let response = client
        .get(format!("{flex_url}/test"))
        .header("x-api-key", "")
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::OK);

    Ok(())
}

// Test rate limit headers functionality
#[pdk_test]
async fn test_rate_limit_headers() -> anyhow::Result<()> { // TODO: check why failing
    let backend_config = HttpMockConfig::builder()
        .port(80)
        .hostname("backend")
        .build();

    let policy_config = PolicyConfig::builder()
        .name(POLICY_NAME)
        .configuration(serde_json::json!({
            "rate_limits": [
                {
                    "group_name": "api",
                    "requests_per_window": 3,
                    "window_size_seconds": 60,
                    "key_selector": "api_key"
                }
            ]
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
        .hostname("local-flex-headers")
        .with_api(api_config)
        .config_mounts([(POLICY_DIR, "policy"), (COMMON_CONFIG_DIR, "common")])
        .build();

    let composite = TestComposite::builder()
        .with_service(flex_config)
        .with_service(backend_config)
        .build()
        .await?;

    let flex: Flex = composite.service()?;
    let flex_url = flex.external_url(FLEX_PORT).unwrap();
    let upstream: HttpMock = composite.service()?;
    let backend_server = MockServer::connect_async(upstream.socket()).await;

    backend_server
        .mock_async(|when, then| {
            when.path_contains("/test");
            then.status(200).body("OK");
        })
        .await;

    let client = reqwest::Client::new();

    // First request - should include rate limit headers
    let response = client
        .get(format!("{flex_url}/test"))
        .header("x-api-key", "key-123")
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::OK);

    // Check for rate limit headers
    assert!(response.headers().contains_key("x-ratelimit-limit"));
    assert!(response.headers().contains_key("x-ratelimit-remaining"));
    assert!(response.headers().contains_key("x-ratelimit-reset"));

    let limit = response.headers().get("x-ratelimit-limit").unwrap();
    let remaining = response.headers().get("x-ratelimit-remaining").unwrap();
    assert_eq!(limit, "3");
    assert_eq!(remaining, "2");

    // Second request - should show updated remaining count
    let response = client
        .get(format!("{flex_url}/test"))
        .header("x-api-key", "key-123")
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::OK);

    let remaining = response.headers().get("x-ratelimit-remaining").unwrap();
    assert_eq!(remaining, "1");

    // Third request - should show remaining = 0
    let response = client
        .get(format!("{flex_url}/test"))
        .header("x-api-key", "key-123")
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::OK);

    let remaining = response.headers().get("x-ratelimit-remaining").unwrap();
    assert_eq!(remaining, "0");

    // Fourth request - should be blocked and still include headers
    let response = client
        .get(format!("{flex_url}/test"))
        .header("x-api-key", "key-123")
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);

    // Should still include rate limit headers even when blocked
    assert!(response.headers().contains_key("x-ratelimit-limit"));
    assert!(response.headers().contains_key("x-ratelimit-remaining"));
    assert!(response.headers().contains_key("x-ratelimit-reset"));

    Ok(())
}

// Test single request limit (requests_per_window = 1)
#[pdk_test]
async fn test_single_request_limit() -> anyhow::Result<()> {
    let backend_config = HttpMockConfig::builder()
        .port(80)
        .hostname("backend")
        .build();

    let policy_config = PolicyConfig::builder()
        .name(POLICY_NAME)
        .configuration(serde_json::json!({
            "rate_limits": [
                {
                    "group_name": "api",
                    "requests_per_window": 1,
                    "window_size_seconds": 60,
                    "key_selector": "api_key"
                }
            ]
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
        .hostname("local-flex-single")
        .with_api(api_config)
        .config_mounts([(POLICY_DIR, "policy"), (COMMON_CONFIG_DIR, "common")])
        .build();

    let composite = TestComposite::builder()
        .with_service(flex_config)
        .with_service(backend_config)
        .build()
        .await?;

    let flex: Flex = composite.service()?;
    let flex_url = flex.external_url(FLEX_PORT).unwrap();
    let upstream: HttpMock = composite.service()?;
    let backend_server = MockServer::connect_async(upstream.socket()).await;

    backend_server
        .mock_async(|when, then| {
            when.path_contains("/test");
            then.status(200).body("OK");
        })
        .await;

    let client = reqwest::Client::new();

    // First request - should be allowed
    let response = client
        .get(format!("{flex_url}/test"))
        .header("x-api-key", "key-123")
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::OK);

    // Second request - should be blocked immediately (rate limit = 1)
    let response = client
        .get(format!("{flex_url}/test"))
        .header("x-api-key", "key-123")
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);

    // Different API key - should be allowed (separate rate limit)
    let response = client
        .get(format!("{flex_url}/test"))
        .header("x-api-key", "key-456")
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::OK);

    Ok(())
}

// Test different window sizes
#[pdk_test]
async fn test_different_window_sizes() -> anyhow::Result<()> {
    let backend_config = HttpMockConfig::builder()
        .port(80)
        .hostname("backend")
        .build();

    let policy_config = PolicyConfig::builder()
        .name(POLICY_NAME)
        .configuration(serde_json::json!({
            "rate_limits": [
                {
                    "group_name": "api",
                    "requests_per_window": 2,
                    "window_size_seconds": 1, // Very short window for testing
                    "key_selector": "api_key"
                }
            ]
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
        .hostname("local-flex-window")
        .with_api(api_config)
        .config_mounts([(POLICY_DIR, "policy"), (COMMON_CONFIG_DIR, "common")])
        .build();

    let composite = TestComposite::builder()
        .with_service(flex_config)
        .with_service(backend_config)
        .build()
        .await?;

    let flex: Flex = composite.service()?;
    let flex_url = flex.external_url(FLEX_PORT).unwrap();
    let upstream: HttpMock = composite.service()?;
    let backend_server = MockServer::connect_async(upstream.socket()).await;

    backend_server
        .mock_async(|when, then| {
            when.path_contains("/test");
            then.status(200).body("OK");
        })
        .await;

    let client = reqwest::Client::new();

    // First request - should be allowed
    let response = client
        .get(format!("{flex_url}/test"))
        .header("x-api-key", "key-123")
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::OK);

    // Second request - should be allowed
    let response = client
        .get(format!("{flex_url}/test"))
        .header("x-api-key", "key-123")
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::OK);

    // Third request - should be blocked
    let response = client
        .get(format!("{flex_url}/test"))
        .header("x-api-key", "key-123")
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);

    // Wait for window to reset (1 second)
    std::thread::sleep(std::time::Duration::from_millis(1100));

    // After window reset - should be allowed again
    let response = client
        .get(format!("{flex_url}/test"))
        .header("x-api-key", "key-123")
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::OK);

    Ok(())
}

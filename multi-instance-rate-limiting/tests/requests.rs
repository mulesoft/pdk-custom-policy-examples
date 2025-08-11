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
            "api_key_rate_limit": {
                "group_name": "api",
                "requests_per_window": 2,
                "window_size_seconds": 60
            },
            "user_id_rate_limit": {
                "group_name": "user",
                "requests_per_window": 5,
                "window_size_seconds": 120
            }
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
            "api_key_rate_limit": {
                "group_name": "api",
                "requests_per_window": 5,
                "window_size_seconds": 60
            },
            "user_id_rate_limit": {
                "group_name": "user",
                "requests_per_window": 3,
                "window_size_seconds": 60
            }
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

// Test missing headers validation should return 400 Bad Request
#[pdk_test]
async fn test_missing_headers() -> anyhow::Result<()> {
    let backend_config = HttpMockConfig::builder()
        .port(80)
        .hostname("backend")
        .build();

    let policy_config = PolicyConfig::builder()
        .name(POLICY_NAME)
        .configuration(serde_json::json!({
            "api_key_rate_limit": {
                "group_name": "api",
                "requests_per_window": 5,
                "window_size_seconds": 60
            },
            "user_id_rate_limit": {
                "group_name": "user",
                "requests_per_window": 3,
                "window_size_seconds": 60
            }
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

    let response = client.get(format!("{flex_url}/test")).send().await?;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

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
            "api_key_rate_limit": {
                "group_name": "api",
                "requests_per_window": 1,
                "window_size_seconds": 60
            },
            "user_id_rate_limit": {
                "group_name": "user",
                "requests_per_window": 3,
                "window_size_seconds": 60
            }
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
            "api_key_rate_limit": {
                "group_name": "api",
                "requests_per_window": 2,
                "window_size_seconds": 1
            },
            "user_id_rate_limit": {
                "group_name": "user",
                "requests_per_window": 3,
                "window_size_seconds": 60
            }
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

#[pdk_test]
async fn test_api_key_rate_limiting() -> anyhow::Result<()> {
    use httpmock::MockServer;
    use pdk_test::services::flex::{ApiConfig, Flex, FlexConfig, PolicyConfig};
    use pdk_test::services::httpmock::{HttpMock, HttpMockConfig};
    use pdk_test::{port::Port, TestComposite};
    use reqwest::StatusCode;

    const FLEX_PORT: Port = 8081;
    let backend_config = HttpMockConfig::builder()
        .port(80)
        .hostname("backend")
        .build();

    let policy_config = PolicyConfig::builder()
        .name(POLICY_NAME)
        .configuration(serde_json::json!({
            "api_key_rate_limit": {
                "group_name": "api",
                "requests_per_window": 3,
                "window_size_seconds": 10
            },
            "user_id_rate_limit": {
                "group_name": "user",
                "requests_per_window": 5,
                "window_size_seconds": 15
            }
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
        .hostname("local-flex-multi-limits")
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

    // Test API key limit (3 requests per 10s)
    for i in 0..3 {
        let response = client
            .get(format!("{}/test", flex_url))
            .header("x-api-key", "key-1")
            .send()
            .await?;
        assert_eq!(
            response.status(),
            StatusCode::OK,
            "Request {} should succeed",
            i + 1
        );
    }

    // 4th request should be rate limited by API key
    let response = client
        .get(format!("{}/test", flex_url))
        .header("x-api-key", "key-1")
        .send()
        .await?;
    assert_eq!(
        response.status(),
        StatusCode::TOO_MANY_REQUESTS,
        "4th request should be rate limited"
    );

    // Test independence: different API key should be allowed
    let response = client
        .get(format!("{}/test", flex_url))
        .header("x-api-key", "key-2")
        .send()
        .await?;
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Different API key should be allowed"
    );

    Ok(())
}

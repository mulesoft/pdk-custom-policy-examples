// Copyright 2023 Salesforce, Inc. All rights reserved.

mod common;

use httpmock::MockServer;
use pdk_test::port::Port;
use pdk_test::services::flex::{ApiConfig, Flex, FlexConfig, PolicyConfig};
use pdk_test::services::httpmock::{HttpMock, HttpMockConfig};
use pdk_test::{pdk_test, TestComposite};
use reqwest::StatusCode;

use common::*;

// Flex port for the internal test network
const FLEX_PORT: Port = 8081;

// Test that an allowed IP passes through
#[pdk_test]
async fn test_allowed_ip_passes() -> anyhow::Result<()> {
    let backend_config = HttpMockConfig::builder()
        .port(80)
        .hostname("backend")
        .build();

    let policy_config = PolicyConfig::builder()
        .name(POLICY_NAME)
        .configuration(serde_json::json!({
            "ips": ["192.168.1.0/24", "10.0.0.1"],
            "ipHeader": "x-real-ip"
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
        .version("1.10.0")
        .hostname("local-flex")
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
            when.path_contains("/hello");
            then.status(200).body("Hello World!");
        })
        .await;

    let client = reqwest::Client::new();

    // Test allowed IP from CIDR range
    let response = client
        .get(format!("{flex_url}/hello"))
        .header("x-real-ip", "192.168.1.100")
        .send()
        .await?;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.text().await?, "Hello World!");

    // Test specific allowed IP
    let response = client
        .get(format!("{flex_url}/hello"))
        .header("x-real-ip", "10.0.0.1")
        .send()
        .await?;

    assert_eq!(response.status(), StatusCode::OK);

    Ok(())
}

// Test that a forbidden IP is blocked
#[pdk_test]
async fn test_forbidden_ip_blocked() -> anyhow::Result<()> {
    let backend_config = HttpMockConfig::builder()
        .port(80)
        .hostname("backend")
        .build();

    let policy_config = PolicyConfig::builder()
        .name(POLICY_NAME)
        .configuration(serde_json::json!({
            "ips": ["192.168.1.0/24"],
            "ipHeader": "x-real-ip"
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
        .version("1.10.0")
        .hostname("local-flex-forbidden")
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
            when.path_contains("/hello");
            then.status(200).body("Hello World!");
        })
        .await;

    let client = reqwest::Client::new();

    // Test IP not in allowed list - should be blocked
    let response = client
        .get(format!("{flex_url}/hello"))
        .header("x-real-ip", "10.10.10.10")
        .send()
        .await?;

    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    // Test another forbidden IP
    let response = client
        .get(format!("{flex_url}/hello"))
        .header("x-real-ip", "8.8.8.8")
        .send()
        .await?;

    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    Ok(())
}

// Test missing IP header returns forbidden
#[pdk_test]
async fn test_missing_ip_header_blocked() -> anyhow::Result<()> {
    let backend_config = HttpMockConfig::builder()
        .port(80)
        .hostname("backend")
        .build();

    let policy_config = PolicyConfig::builder()
        .name(POLICY_NAME)
        .configuration(serde_json::json!({
            "ips": ["192.168.1.0/24"],
            "ipHeader": "x-real-ip"
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
        .version("1.10.0")
        .hostname("local-flex-missing")
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
            when.path_contains("/hello");
            then.status(200).body("Hello World!");
        })
        .await;

    let client = reqwest::Client::new();

    // Request without IP header - should be blocked
    let response = client.get(format!("{flex_url}/hello")).send().await?;

    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    Ok(())
}

// Test multiple CIDR ranges
#[pdk_test]
async fn test_multiple_cidr_ranges() -> anyhow::Result<()> {
    let backend_config = HttpMockConfig::builder()
        .port(80)
        .hostname("backend")
        .build();

    let policy_config = PolicyConfig::builder()
        .name(POLICY_NAME)
        .configuration(serde_json::json!({
            "ips": [
                "192.168.1.0/24",
                "10.0.0.0/8",
                "172.16.0.0/12"
            ],
            "ipHeader": "x-real-ip"
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
        .version("1.10.0")
        .hostname("local-flex-cidr")
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
            when.path_contains("/hello");
            then.status(200).body("Hello World!");
        })
        .await;

    let client = reqwest::Client::new();

    // Test IP from first CIDR range (192.168.1.0/24)
    let response = client
        .get(format!("{flex_url}/hello"))
        .header("x-real-ip", "192.168.1.50")
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::OK);

    // Test IP from second CIDR range (10.0.0.0/8)
    let response = client
        .get(format!("{flex_url}/hello"))
        .header("x-real-ip", "10.255.255.255")
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::OK);

    // Test IP from third CIDR range (172.16.0.0/12)
    let response = client
        .get(format!("{flex_url}/hello"))
        .header("x-real-ip", "172.31.255.255")
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::OK);

    // Test IP outside all CIDR ranges
    let response = client
        .get(format!("{flex_url}/hello"))
        .header("x-real-ip", "8.8.8.8")
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    Ok(())
}

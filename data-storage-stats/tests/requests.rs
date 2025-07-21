// Copyright 2023 Salesforce, Inc. All rights reserved.

mod common;

use httpmock::MockServer;
use pdk_test::port::Port;
use pdk_test::services::flex::{ApiConfig, Flex, FlexConfig, PolicyConfig};
use pdk_test::services::httpmock::{HttpMock, HttpMockConfig};
use pdk_test::{pdk_test, TestComposite};

use common::*;
use reqwest::StatusCode;

const FLEX_PORT: Port = 8081; // Flex port for the internal test network


#[pdk_test]
async fn increment_request_counter() -> anyhow::Result<()> {
    let backend_config = HttpMockConfig::builder()
        .port(80)
        .hostname("backend")
        .build();

    // Configure a Flex service
    let policy_config = PolicyConfig::builder()
        .name(POLICY_NAME)
        .configuration(serde_json::json!({
            "namespace": "test-stats"
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

    let response = reqwest::Client::new()
        .get(format!("{flex_url}/test"))
        .header("x-client-id", "client-123")
        .send()
        .await?;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers().get("x-request-count").unwrap().to_str().unwrap(), "1");
    assert_eq!(response.headers().get("x-client-id").unwrap().to_str().unwrap(), "client-123");
    assert!(response.headers().get("x-last-request").is_some());

    let response = reqwest::Client::new()
        .get(format!("{flex_url}/test"))
        .header("x-client-id", "client-123")
        .send()
        .await?;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers().get("x-request-count").unwrap().to_str().unwrap(), "2");
    assert_eq!(response.headers().get("x-client-id").unwrap().to_str().unwrap(), "client-123");

    Ok(())
}



#[pdk_test]
async fn reject_request_without_client_id() -> anyhow::Result<()> {
    let backend_config = HttpMockConfig::builder()
        .port(80)
        .hostname("backend")
        .build();

    let policy_config = PolicyConfig::builder()
        .name(POLICY_NAME)
        .configuration(serde_json::json!({
            "namespace": "test-stats"
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

    let response = reqwest::Client::new()
        .get(format!("{flex_url}/test"))
        .send()
        .await?;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = response.text().await?;
    assert!(body.contains("Missing client identification header"));

    Ok(())
}

#[pdk_test]
async fn increment_request_counter_multiple_clients() -> anyhow::Result<()> {
    let backend_config = HttpMockConfig::builder()
        .port(80)
        .hostname("backend")
        .build();

    let policy_config = PolicyConfig::builder()
        .name(POLICY_NAME)
        .configuration(serde_json::json!({
            "namespace": "test-stats"
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

    // First client
    let response1 = client
        .get(format!("{flex_url}/test"))
        .header("x-client-id", "client-1")
        .send()
        .await?;

    assert_eq!(response1.status(), StatusCode::OK);
    assert_eq!(response1.headers().get("x-request-count").unwrap().to_str().unwrap(), "1");

    // Second client
    let response2 = client
        .get(format!("{flex_url}/test"))
        .header("x-client-id", "client-2")
        .send()
        .await?;

    assert_eq!(response2.status(), StatusCode::OK);
    assert_eq!(response2.headers().get("x-request-count").unwrap().to_str().unwrap(), "1");

    // First client again
    let response3 = client
        .get(format!("{flex_url}/test"))
        .header("x-client-id", "client-1")
        .send()
        .await?;

    assert_eq!(response3.status(), StatusCode::OK);
    assert_eq!(response3.headers().get("x-request-count").unwrap().to_str().unwrap(), "2");

    Ok(())
}

#[pdk_test]
async fn retrieve_all_client_stats() -> anyhow::Result<()> {
    let backend_config = HttpMockConfig::builder()
        .port(80)
        .hostname("backend")
        .build();

    let policy_config = PolicyConfig::builder()
        .name(POLICY_NAME)
        .configuration(serde_json::json!({
            "namespace": "test-stats"
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

    client
        .get(format!("{flex_url}/test"))
        .header("x-client-id", "client-1")
        .send()
        .await?;

    client
        .get(format!("{flex_url}/test"))
        .header("x-client-id", "client-2")
        .send()
        .await?;

    client
        .get(format!("{flex_url}/test"))
        .header("x-client-id", "client-1")
        .send()
        .await?;

    // Get all stats (admin operation - no client-id needed)
    let response = client
        .get(format!("{flex_url}/test"))
        .header("x-stats", "true")
        .send()
        .await?;

    let status = response.status();
    
    let all_stats = match response.headers().get("x-all-stats") {
        Some(header) => header.to_str().unwrap(),
        None => {
            return Err(anyhow::anyhow!("x-all-stats header not found"));
        }
    };
    
    assert_eq!(status, StatusCode::OK);
    
    let stats_dict: serde_json::Value = serde_json::from_str(all_stats)?;
    let stats_object = stats_dict.as_object().expect("Expected JSON object");
    
    assert!(stats_object.contains_key("client-1"));
    assert!(stats_object.contains_key("client-2"));
    
    // Verify client-1 has count 2 (2 requests)
    let client1_stats = &stats_object["client-1"];
    assert_eq!(client1_stats["count"], 2);
    
    // Verify client-2 has count 1 (1 request)
    let client2_stats = &stats_object["client-2"];
    assert_eq!(client2_stats["count"], 1);
    
    // Verify both have last_request field
    assert!(client1_stats.get("last_request").is_some());
    assert!(client2_stats.get("last_request").is_some());

    Ok(())
}

#[pdk_test]
async fn clear_all_stats() -> anyhow::Result<()> {
    let backend_config = HttpMockConfig::builder()
        .port(80)
        .hostname("backend")
        .build();

    let policy_config = PolicyConfig::builder()
        .name(POLICY_NAME)
        .configuration(serde_json::json!({
            "namespace": "test-stats"
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

    client
        .get(format!("{flex_url}/test"))
        .header("x-client-id", "client-1")
        .send()
        .await?;

    client
        .get(format!("{flex_url}/test"))
        .header("x-client-id", "client-2")
        .send()
        .await?;

    // Reset stats ("admin" operation, no client-id needed)
    let response = client
        .get(format!("{flex_url}/test"))
        .header("x-reset-stats", "true")
        .send()
        .await?;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers().get("x-stats-reset").unwrap().to_str().unwrap(), "true");

    // Verify stats are reset
    let response = client
        .get(format!("{flex_url}/test"))
        .header("x-client-id", "client-1")
        .send()
        .await?;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers().get("x-request-count").unwrap().to_str().unwrap(), "1"); // Back to 1

    Ok(())
}
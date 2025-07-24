// Copyright 2023 Salesforce, Inc. All rights reserved.

mod common;

use httpmock::MockServer;
use pdk_test::port::Port;
use pdk_test::services::flex::{ApiConfig, Flex, FlexConfig, PolicyConfig};
use pdk_test::services::httpmock::{HttpMock, HttpMockConfig};
use pdk_test::{pdk_test, TestComposite};
use reqwest::StatusCode;
use serde_json::Value;

use common::*;

const FLEX_PORT: Port = 8081;

// Basic local storage functionality - counter increment, multiple clients, empty client ID validation
#[pdk_test]
async fn test_basic_local_storage_functionality() -> anyhow::Result<()> {
    let backend_config = HttpMockConfig::builder()
        .port(80)
        .hostname("backend")
        .build();

    let policy_config = PolicyConfig::builder()
        .name(POLICY_NAME)
        .configuration(serde_json::json!({
            "namespace": "test-basic-stats",
            "storage_type": "local",
            "max_retries": 3
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
        .hostname("local-flex-basic")
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

    // Counter increment for same client
    let client = reqwest::Client::new();

    // First request - should initialize counter
    let response = client
        .get(format!("{flex_url}/test"))
        .header("x-client-id", "client-123")
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.text().await?, "OK");

    // Second request - should increment existing counter
    let response = client
        .get(format!("{flex_url}/test"))
        .header("x-client-id", "client-123")
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.text().await?, "OK");

    // Multiple different clients - should create separate counters
    let response = client
        .get(format!("{flex_url}/test"))
        .header("x-client-id", "client-456")
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::OK);

    // Empty client ID validation - should be rejected with 400
    let response = client
        .get(format!("{flex_url}/test"))
        .header("x-client-id", "")
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    Ok(())
}

// Admin stats operations - GET /stats to retrieve all stats and DELETE /stats to reset them
#[pdk_test]
async fn test_admin_stats_operations() -> anyhow::Result<()> {
    let backend_config = HttpMockConfig::builder()
        .port(80)
        .hostname("backend")
        .build();

    let policy_config = PolicyConfig::builder()
        .name(POLICY_NAME)
        .configuration(serde_json::json!({
            "namespace": "test-admin-stats",
            "storage_type": "local",
            "max_retries": 3
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
        .hostname("local-flex-admin")
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

    // Generate test data by making requests for multiple clients
    for i in 1..=3 {
        let response = client
            .get(format!("{flex_url}/test"))
            .header("x-client-id", format!("client-{}", i))
            .send()
            .await?;
        assert_eq!(response.status(), StatusCode::OK);
    }

    // Retrieve all stats via admin endpoint - GET /stats
    let response = client.get(format!("{flex_url}/stats")).send().await?;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get("Content-Type").unwrap(),
        "application/json"
    );

    let stats: Value = response.json().await?;
    assert!(stats.is_object());
    assert_eq!(stats.as_object().unwrap().len(), 3); // Should have 3 clients

    // Reset all stats via admin endpoint - DELETE /stats
    let response = client.delete(format!("{flex_url}/stats")).send().await?;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get("Content-Type").unwrap(),
        "application/json"
    );

    let reset_response: Value = response.json().await?;
    assert!(reset_response["message"].is_string());
    assert!(reset_response["timestamp"].is_number());

    // Verify stats are properly cleared - GET /stats again
    let response = client.get(format!("{flex_url}/stats")).send().await?;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get("Content-Type").unwrap(),
        "application/json"
    );

    let stats: Value = response.json().await?;
    assert!(stats.is_object());
    assert_eq!(stats.as_object().unwrap().len(), 0); // Should have no clients after reset

    Ok(())
}

// CAS concurrency handling - multiple simultaneous requests to same client key
#[pdk_test]
async fn test_cas_concurrency_handling() -> anyhow::Result<()> {
    let backend_config = HttpMockConfig::builder()
        .port(80)
        .hostname("backend")
        .build();

    let policy_config = PolicyConfig::builder()
        .name(POLICY_NAME)
        .configuration(serde_json::json!({
            "namespace": "test-cas-stats",
            "storage_type": "local",
            "max_retries": 10
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
        .hostname("local-flex-cas")
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

    // CAS concurrency with multiple simultaneous requests to same client key
    let client = reqwest::Client::new();
    let mut concurrent_handles = vec![];

    for _ in 0..5 {
        let client = client.clone();
        let flex_url = flex_url.clone();
        let handle = tokio::spawn(async move {
            let response = client
                .get(format!("{flex_url}/test"))
                .header("x-client-id", "concurrent-client")
                .send()
                .await?;
            Ok::<_, anyhow::Error>(response.status())
        });
        concurrent_handles.push(handle);
    }

    // Wait for all concurrent requests to complete
    for handle in concurrent_handles {
        let status = handle.await??;
        assert_eq!(status, StatusCode::OK);
    }

    // Verify final count matches expected concurrent requests
    let response = client.get(format!("{flex_url}/stats")).send().await?;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get("Content-Type").unwrap(),
        "application/json"
    );

    let stats: Value = response.json().await?;
    let client_stats = &stats["concurrent-client"];
    assert_eq!(client_stats["count"], 5);

    Ok(())
}

// Multiple clients concurrent access - different client keys with sequential and concurrent requests
#[pdk_test]
async fn test_multiple_clients_concurrent_access() -> anyhow::Result<()> {
    let backend_config = HttpMockConfig::builder()
        .port(80)
        .hostname("backend")
        .build();

    let policy_config = PolicyConfig::builder()
        .name(POLICY_NAME)
        .configuration(serde_json::json!({
            "namespace": "test-concurrent-stats",
            "storage_type": "local",
            "max_retries": 3
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
        .hostname("local-flex-concurrent")
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
    let client_ids = vec!["client-a", "client-b", "client-c"];

    // Establish initial counters with sequential requests
    for client_id in &client_ids {
        let response = client
            .get(format!("{flex_url}/test"))
            .header("x-client-id", *client_id)
            .send()
            .await?;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.text().await?, "OK");
    }

    // Concurrent access to different client keys simultaneously
    let mut concurrent_handles = vec![];
    for client_id in &client_ids {
        let client = client.clone();
        let flex_url = flex_url.clone();
        let client_id = client_id.to_string();
        let handle = tokio::spawn(async move {
            let response = client
                .get(format!("{flex_url}/test"))
                .header("x-client-id", client_id)
                .send()
                .await?;
            Ok::<_, anyhow::Error>(response.status())
        });
        concurrent_handles.push(handle);
    }

    // Wait for all concurrent requests to complete
    for handle in concurrent_handles {
        let status = handle.await??;
        assert_eq!(status, StatusCode::OK);
    }

    // Verify each client has exactly 2 requests (1 sequential + 1 concurrent)
    let response = client.get(format!("{flex_url}/stats")).send().await?;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get("Content-Type").unwrap(),
        "application/json"
    );

    let stats: Value = response.json().await?;
    for client_id in &client_ids {
        assert!(stats.get(*client_id).is_some());
        let client_stats = &stats[*client_id];
        assert_eq!(client_stats["count"], 2);
    }

    Ok(())
}

// Remote storage functionality - single client and multiple clients with distributed storage
#[pdk_test]
async fn test_remote_storage_functionality() -> anyhow::Result<()> {
    let backend_config = HttpMockConfig::builder()
        .port(80)
        .hostname("backend")
        .build();

    let policy_config = PolicyConfig::builder()
        .name(POLICY_NAME)
        .configuration(serde_json::json!({
            "namespace": "test-remote-stats",
            "storage_type": "remote",
            "max_retries": 3
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
        .hostname("local-flex-remote")
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

    // Single client with remote storage
    let response = client
        .get(format!("{flex_url}/test"))
        .header("x-client-id", "remote-client-1")
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::OK);

    // Multiple clients with remote storage
    for i in 1..=3 {
        let response = client
            .get(format!("{flex_url}/test"))
            .header("x-client-id", format!("remote-client-{}", i))
            .send()
            .await?;
        assert_eq!(response.status(), StatusCode::OK);
    }

    // Verify stats are properly tracked in remote storage
    let response = client.get(format!("{flex_url}/stats")).send().await?;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get("Content-Type").unwrap(),
        "application/json"
    );

    let stats: Value = response.json().await?;
    assert!(stats.is_object());

    Ok(())
}

// Configuration validation - invalid storage type handling and error responses
#[pdk_test]
async fn test_invalid_configuration_handling() -> anyhow::Result<()> {
    // Invalid storage type configuration handling
    let backend_config = HttpMockConfig::builder()
        .port(80)
        .hostname("backend")
        .build();

    let policy_config = PolicyConfig::builder()
        .name(POLICY_NAME)
        .configuration(serde_json::json!({
            "namespace": "test-invalid-storage",
            "storage_type": "invalid",
            "max_retries": 3
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
        .hostname("local-flex-invalid")
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

    // Request should fail with 503 due to invalid storage type configuration
    let response = client
        .get(format!("{flex_url}/test"))
        .header("x-client-id", "invalid-storage-client")
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);

    Ok(())
}

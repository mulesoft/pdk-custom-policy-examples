// Copyright 2023 Salesforce, Inc. All rights reserved.

mod common;

use httpmock::MockServer;
use pdk_test::port::Port;
use pdk_test::services::flex::{ApiConfig, Flex, FlexConfig, PolicyConfig};
use pdk_test::services::httpmock::{HttpMock, HttpMockConfig};
use pdk_test::{pdk_test, TestComposite};

use common::*;
use reqwest::StatusCode;
use std::time::Duration;

const FLEX_PORT: Port = 8081; // Flex port for the internal test network


#[pdk_test]
async fn test_basic_request_counter_increment() -> anyhow::Result<()> {
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

#[pdk_test]
async fn test_cas_concurrency_handling() -> anyhow::Result<()> {
    let backend_config = HttpMockConfig::builder()
        .port(80)
        .hostname("backend")
        .build();

    let policy_config = PolicyConfig::builder()
        .name(POLICY_NAME)
        .configuration(serde_json::json!({
            "namespace": "test-cas-stats"
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
    let client_id = "cas-test-client";

    // Make initial request to establish the counter
    let response = client
        .get(format!("{flex_url}/test"))
        .header("x-client-id", client_id)
        .send()
        .await?;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers().get("x-request-count").unwrap().to_str().unwrap(), "1");

    // Make multiple concurrent requests to test CAS handling
    let mut handles = vec![];
    for _ in 0..5 {
        let client_clone = client.clone();
        let url_clone = flex_url.clone();
        let client_id_clone = client_id.to_string();
        let handle = tokio::spawn(async move {
            let response = client_clone
                .get(format!("{url_clone}/test"))
                .header("x-client-id", client_id_clone)
                .send()
                .await?;
            Ok::<_, anyhow::Error>(response)
        });
        handles.push(handle);
    }

    // Wait for all concurrent requests to complete
    let mut responses = vec![];
    for handle in handles {
        let response = handle.await??;
        responses.push(response);
    }

    // Verify all responses are successful
    for response in &responses {
        assert_eq!(response.status(), StatusCode::OK);
        assert!(response.headers().get("x-request-count").is_some());
    }

    // Make one final request to verify the final count
    let final_response = client
        .get(format!("{flex_url}/test"))
        .header("x-client-id", client_id)
        .send()
        .await?;

    assert_eq!(final_response.status(), StatusCode::OK);
    // Should be 7 total: 1 initial + 5 concurrent + 1 final
    assert_eq!(final_response.headers().get("x-request-count").unwrap().to_str().unwrap(), "7");

    Ok(())
}

#[pdk_test]
async fn test_multiple_clients_concurrent_access() -> anyhow::Result<()> {
    let backend_config = HttpMockConfig::builder()
        .port(80)
        .hostname("backend")
        .build();

    let policy_config = PolicyConfig::builder()
        .name(POLICY_NAME)
        .configuration(serde_json::json!({
            "namespace": "test-concurrent-stats"
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
    let client_ids = vec!["client-a", "client-b", "client-c"];

    // Make sequential requests first to establish counters
    for client_id in &client_ids {
        for _ in 0..2 {
            let response = client
                .get(format!("{flex_url}/test"))
                .header("x-client-id", *client_id)
                .send()
                .await?;
            assert_eq!(response.status(), StatusCode::OK);
        }
    }

    // Now make one concurrent request per client to test CAS
    let mut handles = vec![];
    for client_id in &client_ids {
        let client_clone = client.clone();
        let url_clone = flex_url.clone();
        let client_id_clone = client_id.to_string();
        let handle = tokio::spawn(async move {
            let response = client_clone
                .get(format!("{url_clone}/test"))
                .header("x-client-id", client_id_clone)
                .send()
                .await?;
            Ok::<_, anyhow::Error>(response)
        });
        handles.push(handle);
    }

    // Wait for all concurrent requests to complete
    for handle in handles {
        let response = handle.await??;
        assert_eq!(response.status(), StatusCode::OK);
        assert!(response.headers().get("x-request-count").is_some());
    }

    // Get all stats to verify isolation
    let response = client
        .get(format!("{flex_url}/test"))
        .header("x-stats", "true")
        .send()
        .await?;

    assert_eq!(response.status(), StatusCode::OK);
    let all_stats = response.headers().get("x-all-stats").unwrap().to_str().unwrap();
    let stats_dict: serde_json::Value = serde_json::from_str(all_stats)?;
    let stats_object = stats_dict.as_object().expect("Expected JSON object");

    // Verify each client has exactly 3 requests (2 sequential + 1 concurrent)
    for client_id in &client_ids {
        assert!(stats_object.contains_key(*client_id));
        let client_stats = &stats_object[*client_id];
        assert_eq!(client_stats["count"], 3);
    }

    Ok(())
}

#[pdk_test]
async fn test_cas_retry_mechanism() -> anyhow::Result<()> {
    let backend_config = HttpMockConfig::builder()
        .port(80)
        .hostname("backend")
        .build();

    let policy_config = PolicyConfig::builder()
        .name(POLICY_NAME)
        .configuration(serde_json::json!({
            "namespace": "test-retry-stats"
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
    let client_id = "retry-test-client";

    // Make successive requests to test CAS retry mechanism
    let mut responses = vec![];
    for _ in 0..10 {
        let response = client
            .get(format!("{flex_url}/test"))
            .header("x-client-id", client_id)
            .send()
            .await?;
        responses.push(response);
        
        // Small delay to increase chance of CAS conflicts
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    // Verify all requests succeeded
    for (i, response) in responses.iter().enumerate() {
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers().get("x-request-count").unwrap().to_str().unwrap(), (i + 1).to_string());
    }

    // Verify final count is correct
    let final_response = client
        .get(format!("{flex_url}/test"))
        .header("x-client-id", client_id)
        .send()
        .await?;

    assert_eq!(final_response.status(), StatusCode::OK);
    assert_eq!(final_response.headers().get("x-request-count").unwrap().to_str().unwrap(), "11");

    Ok(())
}
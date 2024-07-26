// Copyright 2023 Salesforce, Inc. All rights reserved.

mod common;

use httpmock::MockServer;
use pdk_test::port::Port;
use pdk_test::services::flex::{ApiConfig, Flex, FlexConfig, PolicyConfig};
use pdk_test::services::httpmock::{HttpMock, HttpMockConfig};
use pdk_test::{pdk_test, TestComposite};
use std::time::Duration;

use common::*;

// Flex port for the internal test network
const FLEX_PORT: Port = 8081;

// This integration test shows how to build a test to compose a local-flex instance
// with a MockServer backend
#[pdk_test]
async fn block() -> anyhow::Result<()> {
    // Configure an HttpMock service
    let httpmock_config = HttpMockConfig::builder()
        .port(80)
        .version("latest")
        .hostname("backend")
        .build();

    // Configure a Flex service
    let policy_config = PolicyConfig::builder()
        .name(POLICY_NAME)
        .configuration(serde_json::json!({"source": "http://backend/blocked", "frequency": 60}))
        .build();

    let api_config = ApiConfig::builder()
        .name("ingress-http")
        .upstream(&httpmock_config)
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

    // Compose the services
    let composite = TestComposite::builder()
        .with_service(flex_config)
        .with_service(httpmock_config)
        .build()
        .await?;

    // Get a handle to the Flex service
    let flex: Flex = composite.service()?;

    // Get an external URL to point the Flex service
    let flex_url = flex.external_url(FLEX_PORT).unwrap();

    // Get a handle to the HttpMock service
    let httpmock: HttpMock = composite.service()?;

    // Create a MockServer
    let mock_server = MockServer::connect_async(httpmock.socket()).await;

    // Mock a /hello request
    mock_server
        .mock_async(|when, then| {
            when.path_contains("/hello");
            then.status(202).body("World!");
        })
        .await;

    let mock = mock_server
        .mock_async(|when, then| {
            when.path_contains("/blocked");
            then.status(200)
                .body("24.152.57.0/24\n24.232.0.0/16\n45.4.92.0/22");
        })
        .await;

    // wait 2 * freq for policy to fetch ips from seconds.
    std::thread::sleep(Duration::from_secs(3));

    assert_request(flex_url.as_str(), "24.152.57.1", 403).await?;
    assert_request(flex_url.as_str(), "24.232.2.2.1", 403).await?;
    assert_request(flex_url.as_str(), "45.4.92.0", 403).await?;
    assert_request(flex_url.as_str(), "46.4.92.0", 202).await?;

    // Was only hit by one of the workers.
    mock.assert_hits(1);
    Ok(())
}

async fn assert_request(url: &str, ip: &str, status_code: u16) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{url}/hello"))
        .header("ip", ip)
        .send()
        .await?;

    assert_eq!(response.status().as_u16(), status_code);
    Ok(())
}

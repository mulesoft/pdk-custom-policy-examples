// Copyright 2023 Salesforce, Inc. All rights reserved.

mod common;

use std::time::Duration;
use httpmock::MockServer;
use pdk_test::{pdk_test, TestComposite};
use pdk_test::port::Port;
use pdk_test::services::flex::{FlexConfig, Flex};
use pdk_test::services::httpmock::{HttpMockConfig, HttpMock};

use common::*;

const MAX_ATTEMPTS: u32 = 3;
const DELAY: Duration = Duration::from_millis(1000);
const EPSILON: Duration = Duration::from_millis(100);

// Directory with the configurations for the `hello` test.
const HELLO_CONFIG_DIR: &str =  concat!(env!("CARGO_MANIFEST_DIR"), "/tests/requests/hello");

// Flex port for the internal test network
const FLEX_PORT: Port = 8081;

// This integration test shows how to build a test to compose a local-flex instance
// with a MockServer backend
#[pdk_test]
async fn spike() -> anyhow::Result<()> {

    // Configure a Flex service
    let flex_config = FlexConfig::builder()
        .version("1.6.1")
        .hostname("local-flex")
        .ports([FLEX_PORT])
        .config_mounts([
            (POLICY_DIR, "policy"),
            (COMMON_CONFIG_DIR, "common"),
            (HELLO_CONFIG_DIR, "hello"),
        ])
        .build();

    // Configure an HttpMock service
    let httpmock_config = HttpMockConfig::builder()
        .port(80)
        .version("latest")
        .hostname("backend")
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
    mock_server.mock_async(|when, then| {
        when.path_contains("/hello");
        then.status(202).body("World!");
    }).await;

    // Perform requests
    generate_load(100, format!("{flex_url}/hello")).await;

    Ok(())
}

async fn execute_request(url: String) -> u16 {
    let start = std::time::SystemTime::now();
    let response = reqwest::get(url.clone()).await.unwrap();
    let end = std::time::SystemTime::now();

    let elapsed = end.duration_since(start).unwrap();


    if response.status() == 202 {
        // Accepted request should have been served as soon as possible.
        assert!(elapsed <= MAX_ATTEMPTS * (DELAY + EPSILON))
    } else if response.status() == 429 {
        // Rejected request should have spent time waiting.
        assert!(elapsed >= MAX_ATTEMPTS * DELAY)
    }

    response.status().as_u16()
}

async fn generate_load(load: usize, url: String) {
    let vec: Vec<_> = (0..load)
        .into_iter()
        .map(|_| url.clone())
        .map(execute_request)
        .collect();

    let resp = futures::future::join_all(vec).await;

    assert!(resp.contains(&202));
    // This assertion might fail if workers >= amount / 4 or if request are executed sequentially
    assert!(resp.contains(&429));
}

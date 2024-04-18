// Copyright 2023 Salesforce, Inc. All rights reserved.

mod common;

use httpmock::MockServer;
use pdk_test::port::Port;
use pdk_test::services::flex::{Flex, FlexConfig};
use pdk_test::services::httpmock::{HttpMock, HttpMockConfig};
use pdk_test::{pdk_test, TestComposite};
use std::time::Duration;

use common::*;

// Directory with the configurations for the `hello` test.
const HELLO_CONFIG_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/requests/hello");

// Flex port for the internal test network
const FLEX_PORT: Port = 8081;

// This integration test shows how to build a test to compose a local-flex instance
// with a MockServer backend
#[pdk_test]
async fn metrics() -> anyhow::Result<()> {
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
    mock_server
        .mock_async(|when, then| {
            when.path_contains("/hello");
            then.status(202).body("World!");
        })
        .await;

    // Mock a /hello request
    let mock = mock_server
        .mock_async(|when, then| {
            when.path_contains("/metrics")
                .body_contains(r#""methods":{"get":1},"status_codes":{"202":1}"#);
            then.status(200);
        })
        .await;

    // Perform an actual request
    let response = reqwest::get(format!("{flex_url}/hello")).await?;

    // Assert on the response
    assert_eq!(response.status(), 202);

    probe(
        21, // worst case scenario it will be sent in 2 * push frequency + epsilon.
        Duration::from_millis(100),
        &move || mock.hits() == 1,
        "Metrics were not sent",
    );

    Ok(())
}

fn probe(count: usize, delay: Duration, condition: &dyn Fn() -> bool, message: &str) {
    for _ in 0..count {
        if condition() {
            return;
        }
        std::thread::sleep(delay);
    }
    panic!("{}", message);
}

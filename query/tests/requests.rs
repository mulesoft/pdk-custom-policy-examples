// Copyright 2023 Salesforce, Inc. All rights reserved.

use httpmock::MockServer;
use pdk_test::port::Port;
use pdk_test::services::flex::{Flex, FlexConfig};
use pdk_test::services::httpmock::{HttpMock, HttpMockConfig};
use pdk_test::{pdk_test, TestComposite};

use common::*;

mod common;

// Directory with the configurations for the `query` test.
const QUERY_CONFIG_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/requests/query");

// Flex port for the internal test network
const FLEX_PORT: Port = 8081;

// This integration test shows how to build a test to compose a local-flex instance
// with a MockServer backend
#[pdk_test]
async fn query() -> anyhow::Result<()> {
    // Configure a Flex service
    let flex_config = FlexConfig::builder()
        .version("1.6.1")
        .hostname("local-flex")
        .ports([FLEX_PORT])
        .config_mounts([
            (POLICY_DIR, "policy"),
            (COMMON_CONFIG_DIR, "common"),
            (QUERY_CONFIG_DIR, "query"),
        ])
        .build();

    // Configure upstream service
    let upstream_config = HttpMockConfig::builder()
        .port(80)
        .hostname("backend")
        .build();

    // Compose the services
    let composite = TestComposite::builder()
        .with_service(flex_config)
        .with_service(upstream_config)
        .build()
        .await?;

    // Get a handle to the Flex service
    let flex: Flex = composite.service()?;

    // Get an external URL to point the Flex service
    let flex_url = flex.external_url(FLEX_PORT).unwrap();

    // Get a handle to the upstream service
    let upstream: HttpMock = composite.service()?;

    // Connect to the handle of the upstream service
    let upstream_server = MockServer::connect_async(upstream.socket()).await;

    // Set up mock that should not be invoked if everything works correctly. In this case it's a mock for the case when
    // the header that should not be present, actually is
    let failed_mock = upstream_server.mock(|when, then| {
        when.header_exists("X-Query-Absent");
        then.status(500)
            .body("Header X-Query-Absent should not get to the backend");
    });

    // Set up mock that represents the desired behavior
    let success_mock = upstream_server.mock(|when, then| {
        when.header("X-Query-Key", "value")
            .header("X-Query-Missing", "Undefined")
            .header("X-Query-Extra", "")
            .header(
                "X-Envoy-Original-Path",
                "/hello?absent=absent&removed=extra&removed=key",
            );
        then.status(200);
    });

    // Send a request with two query parameters.
    let response = reqwest::Client::new()
        .get(format!("{flex_url}/hello"))
        .query(&[("key", "value"), ("extra", ""), ("absent", "absent")])
        .send()
        .await?;

    assert_eq!(response.status().as_u16(), 200u16);
    success_mock.assert();
    failed_mock.assert_hits(0);

    Ok(())
}

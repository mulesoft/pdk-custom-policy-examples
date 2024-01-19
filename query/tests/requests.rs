// Copyright 2023 Salesforce, Inc. All rights reserved.

mod common;

use pdk_test::port::Port;
use pdk_test::services::flex::{Flex, FlexConfig};
use pdk_test::services::httpbin::HttpBinConfig;
use pdk_test::{pdk_test, TestComposite};
use serde::Deserialize;
use std::collections::HashMap;

use common::*;

// Directory with the configurations for the `query` test.
const QUERY_CONFIG_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/requests/query");

// Flex port for the internal test network
const FLEX_PORT: Port = 8081;

/// Struct to deserialize the response from the httpbin server.
#[derive(Deserialize, Debug)]
struct HttpBinResponse {
    // We are only interested in the headers.
    headers: HashMap<String, String>,
}

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

    // Configure an HttpMock service
    let httpbin_config = HttpBinConfig::builder()
        .version("latest")
        .hostname("backend")
        .build();

    // Compose the services
    let composite = TestComposite::builder()
        .with_service(flex_config)
        .with_service(httpbin_config)
        .build()
        .await?;

    // Get a handle to the Flex service
    let flex: Flex = composite.service()?;

    // Get an external URL to point the Flex service
    let flex_url = flex.external_url(FLEX_PORT).unwrap();

    // Send a request with two query parameters.
    let response = reqwest::Client::new()
        .get(format!("{flex_url}/hello"))
        .query(&[("key", "value"), ("extra", ""), ("absent", "absent")])
        .send()
        .await?;

    assert_eq!(response.status().as_u16(), 200u16);

    // We assert that the query parameters were transformed to headers by the policy.
    let body = response.bytes().await.unwrap().to_vec();
    let echoed: HttpBinResponse = serde_json::from_slice(body.as_slice())?;
    assert_eq!(
        echoed.headers.get("X-Query-Key").map(String::as_str),
        Some("value")
    );
    assert_eq!(
        echoed.headers.get("X-Query-Missing").map(String::as_str),
        Some("Undefined")
    );
    assert_eq!(
        echoed.headers.get("X-Query-Extra").map(String::as_str),
        Some("")
    );
    assert_eq!(
        echoed.headers.get("X-Query-Absent").map(String::as_str),
        None
    );

    Ok(())
}

// Copyright 2023 Salesforce, Inc. All rights reserved.

mod common;
use httpmock::MockServer;
use pdk_test::port::Port;
use pdk_test::services::flex::{Flex, FlexConfig};
use pdk_test::services::httpmock::{HttpMock, HttpMockConfig};
use pdk_test::{pdk_test, TestComposite};

use common::*;
use serde_json::Value;

// Directory with the configurations for the `hello` test.
const HELLO_CONFIG_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/requests/authorization");

// Flex port for the internal test network
const FLEX_PORT: Port = 8081;

// This integration test shows how to build a test to compose a local-flex instance
// with a MockServer backend
#[pdk_test]
async fn authorization() -> anyhow::Result<()> {
    // Configuring the Flex service
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

    // Configuring the upstream service
    let backend_conf = HttpMockConfig::builder()
        .port(80)
        .version("latest")
        .hostname("backend")
        .build();

    // Configuring the request introspection service
    let oauth_conf = HttpMockConfig::builder()
        .port(80)
        .version("latest")
        .hostname("oauth.com")
        .build();

    // Compose the services
    let composite = TestComposite::builder()
        .with_service(flex_config)
        .with_service(backend_conf)
        .with_service(oauth_conf)
        .build()
        .await?;

    // Get a handle to the Flex service
    let flex: Flex = composite.service()?;

    // Get an external URL to point the Flex service
    let flex_url = flex.external_url(FLEX_PORT).unwrap();

    // Mock for backend server
    let backend_server =
        MockServer::connect_async(composite.service_by_name::<HttpMock>("backend")?.socket()).await;

    backend_server
        .mock_async(|when, then| {
            when.path_contains("/hello");
            then.status(202).body("World!");
        })
        .await;

    let introspection_server =
        MockServer::connect_async(composite.service_by_name::<HttpMock>("oauth.com")?.socket())
            .await;

    const VALID_TOKEN: &str = "valid";

    println!("about to mock the instrospection server valid req");

    introspection_server
        .mock_async(|when, then| {
            when.body(serde_urlencoded::to_string([("token", VALID_TOKEN)]).unwrap());
            then.status(200)
                .json_body(serde_json::from_str::<Value>(r#"{"active": true}"#).unwrap());
        })
        .await;

    println!("about to mock the instrospection server invalid req");

    introspection_server
        .mock_async(|when, then| {
            when.any_request();
            then.status(200)
                .json_body(serde_json::from_str::<Value>(r#"{"active": false}"#).unwrap());
        })
        .await;

    println!("performing a request to the flex service");

    // Perform an actual request
    let response = reqwest::Client::new()
        .get(format!("{flex_url}/hello"))
        .header("Authorization", format!("Bearer {}", VALID_TOKEN))
        .send()
        .await?;

    // Assert on the response
    assert_eq!(response.status(), 202);

    println!("performing another request to the flex service");

    let response = reqwest::Client::new()
        .get(format!("{flex_url}/hello"))
        .header("Authorization", format!("Bearer not_valid"))
        .send()
        .await?;

    assert_eq!(response.status(), 401);

    Ok(())
}

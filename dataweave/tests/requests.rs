// Copyright 2023 Salesforce, Inc. All rights reserved.

mod common;

use httpmock::MockServer;
use pdk_test::port::Port;
use pdk_test::services::flex::{ApiConfig, Flex, FlexConfig, PolicyConfig};
use pdk_test::services::httpmock::{HttpMock, HttpMockConfig};
use pdk_test::{pdk_test, TestComposite};

use common::*;

// Flex port for the internal test network
const FLEX_PORT: Port = 8081;

// This integration test shows how to build a test to compose a local-flex instance
// with a MockServer backend
#[pdk_test]
async fn dataweave() -> anyhow::Result<()> {
    // Configure an HttpMock service
    let httpmock_config = HttpMockConfig::builder()
        .port(80)
        .version("latest")
        .hostname("backend")
        .build();

    let policy_config = PolicyConfig::builder()
        .name(POLICY_NAME)
        // Read the id from the body, if absent read it from a header, otherwise use the value defined on the provided var
        .configuration(serde_json::json!({
            "expression": "#[payload.Envelope.Header.id default if (attributes.headers.authorization != null) splitBy(dw::core::Binaries::fromBase64(dw::core::Strings::substringAfter(attributes.headers.authorization, 'Basic ')), ':')[0] else (vars.defaultId ++ '-' ++ vars.version)]",
        }))
        .build();

    let api_config = ApiConfig::builder()
        .name("myApi")
        .upstream(&httpmock_config)
        .path("/anything/echo/")
        .port(FLEX_PORT)
        .policies([policy_config])
        .build();

    // Configure a Flex service
    let flex_config = FlexConfig::builder()
        .version("1.9.0")
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

    // Perform an actual request
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{flex_url}/hello"))
        .header("Content-Type", "application/xml")
        .header("Authorization", "Basic dXNlcjpwYXNz")
        .body(r#"<Envelope><Header><id>bodyId</id></Header></Envelope>"#)
        .send()
        .await?;

    // Assert on the response
    assert_eq!(response.status(), 200);
    assert_eq!(response.text().await.unwrap(), r#"{"result":"bodyId"}"#);

    let response = client
        .post(format!("{flex_url}/hello"))
        .header("Content-Type", "application/xml")
        .header("Authorization", "Basic dXNlcjpwYXNz")
        .body(r#"<Envelope></Envelope>"#)
        .send()
        .await?;

    // Assert on the response
    assert_eq!(response.status(), 200);
    assert_eq!(response.text().await.unwrap(), r#"{"result":"user"}"#);

    let response = client.get(format!("{flex_url}/hello")).send().await?;

    // Assert on the response
    assert_eq!(response.status(), 200);
    assert_eq!(
        response.text().await.unwrap(),
        r#"{"result":"hardcoded-1"}"#
    );

    Ok(())
}

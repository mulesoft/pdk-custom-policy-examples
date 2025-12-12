// Copyright 2023 Salesforce, Inc. All rights reserved.

mod common;

use httpmock::MockServer;
use pdk_test::port::Port;
use pdk_test::services::flex::{ApiConfig, Flex, FlexConfig, PolicyConfig};
use pdk_test::services::httpmock::{HttpMock, HttpMockConfig};
use pdk_test::{pdk_test, TestComposite};

use common::*;

const FLEX_PORT: Port = 8081;

fn policy_config(allowed_ips: Vec<&str>) -> PolicyConfig {
    PolicyConfig::builder()
        .name(POLICY_NAME)
        .configuration(serde_json::json!({
            "ips": allowed_ips,
            "ipExpression": "#[attributes.headers['x-forwarded-for']]"
        }))
        .build()
}

async fn setup_test(allowed_ips: Vec<&str>) -> anyhow::Result<(TestComposite, String, MockServer)> {
    let httpmock_config = HttpMockConfig::builder()
        .port(80)
        .version("latest")
        .hostname("backend")
        .build();

    let api_config = ApiConfig::builder()
        .name("test-api")
        .upstream(&httpmock_config)
        .path("/api/")
        .port(FLEX_PORT)
        .policies([policy_config(allowed_ips)])
        .build();

    let flex_config = FlexConfig::builder()
        .version("1.10.0")
        .hostname("local-flex")
        .with_api(api_config)
        .config_mounts([(POLICY_DIR, "policy"), (COMMON_CONFIG_DIR, "common")])
        .build();

    let composite = TestComposite::builder()
        .with_service(flex_config)
        .with_service(httpmock_config)
        .build()
        .await?;

    let flex: Flex = composite.service()?;
    let flex_url = flex.external_url(FLEX_PORT).unwrap();

    let httpmock: HttpMock = composite.service()?;
    let mock_server = MockServer::connect_async(httpmock.socket()).await;

    mock_server
        .mock_async(|when, then| {
            when.path_contains("/hello");
            then.status(200).body("OK");
        })
        .await;

    Ok((composite, flex_url, mock_server))
}

#[pdk_test]
async fn allowed_ip_returns_200() -> anyhow::Result<()> {
    let (_composite, flex_url, _mock) = setup_test(vec!["192.168.1.1"]).await?;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{flex_url}/hello"))
        .header("x-forwarded-for", "192.168.1.1")
        .send()
        .await?;

    assert_eq!(response.status(), 200);
    Ok(())
}

#[pdk_test]
async fn blocked_ip_returns_403() -> anyhow::Result<()> {
    let (_composite, flex_url, _mock) = setup_test(vec!["192.168.1.1"]).await?;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{flex_url}/hello"))
        .header("x-forwarded-for", "10.0.0.1")
        .send()
        .await?;

    assert_eq!(response.status(), 403);
    Ok(())
}

#[pdk_test]
async fn missing_header_returns_403() -> anyhow::Result<()> {
    let (_composite, flex_url, _mock) = setup_test(vec!["192.168.1.1"]).await?;

    let client = reqwest::Client::new();
    let response = client.get(format!("{flex_url}/hello")).send().await?;

    assert_eq!(response.status(), 403);
    Ok(())
}

#[pdk_test]
async fn cidr_range_allows_matching_ip() -> anyhow::Result<()> {
    let (_composite, flex_url, _mock) = setup_test(vec!["192.168.1.0/24"]).await?;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{flex_url}/hello"))
        .header("x-forwarded-for", "192.168.1.100")
        .send()
        .await?;

    assert_eq!(response.status(), 200);
    Ok(())
}

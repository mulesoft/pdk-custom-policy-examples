// Copyright 2023 Salesforce, Inc. All rights reserved.

use std::fs;

use httpmock::MockServer;
use pdk_test::port::Port;
use pdk_test::services::flex::{Flex, FlexConfig};
use pdk_test::services::httpmock::{HttpMock, HttpMockConfig};
use pdk_test::{pdk_test, TestComposite};

use common::*;

mod common;

// Directory with the configurations for the `cert` test.
const CERTS_CONFIG_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/requests/certs");

// Flex port for the internal test network
const FLEX_PORT: Port = 8081;

#[pdk_test]
async fn certs() -> anyhow::Result<()> {
    // Configure a Flex service
    let flex_config = FlexConfig::builder()
        .version("1.6.1")
        .hostname("local-flex")
        .ports([FLEX_PORT])
        .config_mounts([
            (POLICY_DIR, "policy"),
            (COMMON_CONFIG_DIR, "common"),
            (CERTS_CONFIG_DIR, "certs"),
        ])
        .build();

    // Configure an HttpMock service
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

    // Get a handle to the upstream service
    let upstream: HttpMock = composite.service()?;

    // Connect to the handle of the upstream service
    let upstream_server = MockServer::connect_async(upstream.socket()).await;

    let mock = upstream_server.mock(|when, then| {
        when.header("X-Peer-Email", "joker@phantomthieves.com")
            .header("X-Peer-Name", "Joker");
        then.status(200);
    });

    // Get an external URL to point the Flex service
    let flex_url = flex
        .external_url(FLEX_PORT)
        .unwrap()
        .replace("http", "https");

    // Load the server cert.
    let cert = fs::read("tests/resources/server.crt")?;
    let server_cert = reqwest::Certificate::from_pem(&cert)?;

    // Load the CA cert.
    let cert = fs::read("tests/resources/ca.pem")?;
    let ca_cert = reqwest::Certificate::from_pem(&cert)?;

    // Load the cert and key for the client.
    let cert = fs::read("tests/resources/client.full.pem")?;
    let identity = reqwest::Identity::from_pem(&cert)?;

    let client = reqwest::Client::builder()
        .use_rustls_tls() // Use rust tls to be able to provide custom certs.
        .danger_accept_invalid_certs(true) // Self Signed CA certificates are considered invalid certs.
        .add_root_certificate(server_cert) // Add the server certificate
        .add_root_certificate(ca_cert) // Add the CA certificate.
        .identity(identity) // Add own certificate to do the mtls.
        .build()?;

    client.get(flex_url).send().await?;

    mock.assert();

    Ok(())
}

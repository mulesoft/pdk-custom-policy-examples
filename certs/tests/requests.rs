// Copyright 2023 Salesforce, Inc. All rights reserved.

mod common;

use pdk_test::port::Port;
use pdk_test::services::flex::{Flex, FlexConfig};
use pdk_test::services::httpbin::HttpBinConfig;
use pdk_test::{pdk_test, TestComposite};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

use common::*;

// Directory with the configurations for the `cert` test.
const CERTS_CONFIG_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/requests/certs");

// Flex port for the internal test network
const FLEX_PORT: Port = 8081;

/// Struct to deserialize the response from the httpbin server.
#[derive(Deserialize, Debug)]
struct HttpBinResponse {
    // We are only interested in the headers.
    headers: HashMap<String, String>,
}

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

    let response = client.get(flex_url).send().await?;

    let body = response.bytes().await.unwrap().to_vec();
    let echoed: HttpBinResponse = serde_json::from_slice(body.as_slice())?;

    // We assert that the name and mail were added as headers by the policy.
    assert_eq!(
        echoed.headers.get("X-Peer-Email").map(String::as_str),
        Some("joker@phantomthieves.com")
    );

    assert_eq!(
        echoed.headers.get("X-Peer-Name").map(String::as_str),
        Some("Joker")
    );

    Ok(())
}

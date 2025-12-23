// Copyright 2023 Salesforce, Inc. All rights reserved.

mod common;

use httpmock::MockServer;
use pdk::jwt::model::{SigningAlgorithm, SigningKeyLength};
use pdk::jwt::SignatureValidator;
use pdk_test::port::Port;
use pdk_test::services::flex::{ApiConfig, Flex, FlexConfig, PolicyConfig};
use pdk_test::services::httpmock::{HttpMock, HttpMockConfig};
use pdk_test::{pdk_test, TestComposite};

use common::*;

// Flex port for the internal test network
const FLEX_PORT: Port = 8081;

const TEST_RSA_PRIVATE_KEY: &str = "-----BEGIN PRIVATE KEY-----
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
AhW8CZzWMXVPOgExNZJBW65D7p/V
-----END PRIVATE KEY-----";

const TEST_RSA_PUBLIC_KEY: &str = "-----BEGIN PUBLIC KEY-----
MIICIjANBgkqhkiG9w0BAQEFAAOCAg8AMIICCgKCAgEAvzqxRxjDW7KrjnopVCN/
SlIQpJ7gZcX/t5lzg6w6fIj5vf7UPxNUtiWraeYqfXBhb/xQO53SJoFDCB2Z0VZD
WgpOmJf/kgPtAtSCHAb0eGe666Elx+L0LQQU5z1csRAzma7qafzeQGrYbaoKgpXR
EYj7usvxkopdFITv5xa5Tme+DQPQXgLUIUZRDZEzC/Tm2sTvw7+Y8yeCJCGSfFyn
6K3l6twmQRJtoNRGRFNlg9rw2cb+1nT9c/08evjy25ywU8uRO2VDsfWx6GjPqFzw
s6wNnuXPcYWOjNST/HHJfaqDlIVVATIlyEUkdDnmrAM/3RAvqpcfGE/BaSQkDGnU
ejbqIAFl5Nki9X8bZnNMymFlyY+w4ord6QZlLF7JjsUBhlfbinlibs+Gld23NLjB
klnARqFVjlULcPHSzRVlHWWL69OoQEXEiwkYS2z+AgkAYF3uTKilw8WuF8GMpoRX
Mb2v+F6Xp6xHcUHrk2nA8z6CQJyKrBDhwJezznFsyWx15tD1p0OBlWPuFEU4nXvm
1uWH4QL/xaIwDhvPpzOPZWlWbrkOD5Cg+bc6CzFYeZiGlL3oiRv/3BV7zBrRgC4F
Dc5XKLWxvBmF09lNRC+c58ADq8R8VqxIwSK09dwfjsJmidtX6XHfCiM5WrPYOEsw
DGMTi24MRdH9qfphXIXOrW8CAwEAAQ==
-----END PUBLIC KEY-----";

#[pdk_test]
async fn jwt() -> anyhow::Result<()> {
    let backend_config = HttpMockConfig::builder()
        .port(80)
        .hostname("backend")
        .build();

    let policy_config = PolicyConfig::builder()
        .name(POLICY_NAME)
        .configuration(serde_json::json!({
            "privateKey": TEST_RSA_PRIVATE_KEY,
            "kid": "1",
            "jku": "https://test-jwks.example.com"
        }))
        .build();

    let api_config = ApiConfig::builder()
        .name("ingress-http")
        .upstream(&backend_config)
        .path("/anything/echo/")
        .port(FLEX_PORT)
        .policies([policy_config])
        .build();

    let flex_config = FlexConfig::builder()
        .version("1.10.0")
        .hostname("local-flex")
        .with_api(api_config)
        .config_mounts([(POLICY_DIR, "policy"), (COMMON_CONFIG_DIR, "common")])
        .build();

    let composite = TestComposite::builder()
        .with_service(flex_config)
        .with_service(backend_config)
        .build()
        .await?;

    let flex: Flex = composite.service()?;
    let flex_url = flex.external_url(FLEX_PORT).unwrap();
    let upstream: HttpMock = composite.service()?;
    let backend_server = MockServer::connect_async(upstream.socket()).await;

    backend_server
        .mock_async(|when, then| {
            when.path_contains("/hello");
            then.status(200).body("Hello World!");
        })
        .await;

    let client = reqwest::Client::new();

    let response = client.get(format!("{flex_url}/hello")).send().await?;

    let validator = SignatureValidator::new(
        SigningAlgorithm::Rsa,
        SigningKeyLength::Len256,
        TEST_RSA_PUBLIC_KEY.to_string(),
    )
    .unwrap();

    let jwt = response.text().await?;
    validator.validate(jwt).unwrap();

    Ok(())
}

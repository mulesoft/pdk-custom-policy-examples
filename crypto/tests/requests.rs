// Copyright 2023 Salesforce, Inc. All rights reserved.

mod common;

use aes_gcm::aead::generic_array::GenericArray;
use aes_gcm::aead::{Aead, Payload};
use aes_gcm::{Aes256Gcm, Key, KeyInit};
use anyhow::anyhow;
use common::*;
use httpmock::MockServer;
use pdk_test::port::Port;
use pdk_test::services::flex::{ApiConfig, Flex, FlexConfig, PolicyConfig};
use pdk_test::services::httpmock::{HttpMock, HttpMockConfig};
use pdk_test::{pdk_test, TestComposite};
use rsa::pkcs8::DecodePublicKey;
use rsa::{Pkcs1v15Encrypt, RsaPublicKey};

// Flex port for the internal test network
const FLEX_PORT: Port = 8081;

// The value of the "nonce" to be used in the request. Subsequent requests should use different values.
const NONCE: &str = "0123456789AB";

// Public RSA key to be used in the example. If you modify this value remember to change the private key
// in the config section of the api.yaml
const PUBLIC_KEY: &str = r#"-----BEGIN PUBLIC KEY-----
MIIBITANBgkqhkiG9w0BAQEFAAOCAQ4AMIIBCQKCAQBvyTeYA7UDxlEH7KdCCUE6
MhzMNv76jgXHDFrjoFPphbxTqwBhamc5MIM0VwDPqG639uX3tDwQLEeOnd44U/W4
MgEiZ/NmP5Djh9xdMg+kJs7tEDquve2FpQphss3KPByKOGbExSpGTeMTEaoo6Q5n
0id1TYJSppbPz1YTctT6HorPfWcnyoKpdWLpcKphfOe9xfVmDQqftBoRJB+Xnlo/
c8I8UR3Lt5l7s1rT7DKlby9aUx38++R95vrq5FRZEGRQCOgutLoHWC95sXASNZbG
hJ1p2CFIX23g9up2y54Jf9v+MahaoipnVQVP0+yEgVHJfCreO1WJdRjMY0qHxr0z
AgMBAAE=
-----END PUBLIC KEY-----"#;

const PRIVATE_KEY: &str = r#"-----BEGIN RSA PRIVATE KEY-----
MIIEogIBAAKCAQBvyTeYA7UDxlEH7KdCCUE6MhzMNv76jgXHDFrjoFPphbxTqwBh
amc5MIM0VwDPqG639uX3tDwQLEeOnd44U/W4MgEiZ/NmP5Djh9xdMg+kJs7tEDqu
ve2FpQphss3KPByKOGbExSpGTeMTEaoo6Q5n0id1TYJSppbPz1YTctT6HorPfWcn
yoKpdWLpcKphfOe9xfVmDQqftBoRJB+Xnlo/c8I8UR3Lt5l7s1rT7DKlby9aUx38
++R95vrq5FRZEGRQCOgutLoHWC95sXASNZbGhJ1p2CFIX23g9up2y54Jf9v+Maha
oipnVQVP0+yEgVHJfCreO1WJdRjMY0qHxr0zAgMBAAECggEATVGuRFT8FVpMneCQ
Z9qi5S/YP2AiyQcG9ACVtTAmGjXIZ061+qtwLlxhxAgRpZBo1JHPyp89lCosbHbB
R9C6+uaLZlLkRerduqM0Rrnjm7TEF7DMBiWUboYTQjQ8pw8g5nHQk3WUogApef0T
5ywcLK9tX3N3GtfjJGXSsa0RptYis6L/ergBdzJOeVekFw0XckzxLvonbWtLlqCD
dDKV9xH+HgOYPJWw3aGUjt/yF0/Eaa7laQj//9rkKZZ9erqERQSjQnyNTpDmzWx1
u3Lgigxw3aZ4utEB7/3KxH4XQiiU5fB+DW7vZT+y1DuHK8O6dzOuvsiuD4YYf/qv
+XxJwQKBgQC1WCJvM1/g0CdmzMfb1rDW4Bj7u1aq0TVF7p4TkJItHajWxSu2//MW
enZgm/+hUkc84TkwLTrGgIG2LblHveeXEYF02f+8EcoF65SMktoJKAekB0xo7gWu
upILm6BRrR8IFJ1d4cgOGOofSL7VjmZNDUBAHl1BMnx+9wIMsexJYwKBgQCdzlbl
ZhSb3fWdcI7AqCJ99Ln6iXmgstGXo7nYXESJg+xkLKs5AVcIbLIgpxBD+zEY/e8M
EjNCD6HDn6l9ex2aUWXYslq/Hnyuam+vdlF3hVzYnicouBa9pGEgq5E/derBdv7W
ERdNSp3FQuvGsTCivxF4iysj3n+QacE/tZft8QKBgQCvZHDZkJzY+TpCqSlcLQeY
q+I+BvFKAVI/Mwzc62UWEautHcKsGl1ojkVUJ01VIBkaftMrD3PbbYsHQq4C/1+w
sxO8iuRXZ/U3SKTCWX5cgMTzFsQGcMA6QOTkKT8kAAcVb6rMlIVDbUFzxI4eSr4T
JA/SdAvJ9SSIQCtRT/yy9QKBgHyfc2h+kU/2nf3T4iirn3GxTx5Ya0FJdtQ2bJI9
a7LeCFWkISRHtp/kl1fKF8JYIZIHGD2EMg69oZaIBKYgEGTKW5Aiah/8x6JGVCVR
X0zweT5ox6wUlYD2y+2tEGv5h4d5ng8YODrF7orWKrUjQbGFCxsTbOJK2JsHqaHS
m0VBAoGAPQTCye7Y3BXsojvFRs68TPu8T/XQ9s9WJYpO5t+0/Sb8IkWo7S39v5WR
Zb8IyY43DI5E9Kby62U4bf0okzbno3CBGCEqORZsCQIopMBWHpVXw2ABJLaxOw/+
JkkBm8bQCWZPi2kFXOWBDfwktK1P2t+jQMfQkRAnQBpOwAgeuXs=
-----END RSA PRIVATE KEY-----"#;

// AES key to be used in the example. If you modify this value remember to also change it the config section of the api.yaml
const AES_KEY: &str = "42F56B955DEA9D821F2A38E3CCDCCQWE";

#[pdk_test]
async fn crypto() -> anyhow::Result<()> {
    // Configure an HttpMock service
    let upstream_config = HttpMockConfig::builder()
        .port(80)
        .hostname("backend")
        .build();

    // Configure a Flex service
    let policy_config = PolicyConfig::builder()
        .name(POLICY_NAME)
        .configuration(serde_json::json!({
            "aes_key": "42F56B955DEA9D821F2A38E3CCDCCQWE",
            "rsa_key": PRIVATE_KEY
        }))
        .build();

    let api_config = ApiConfig::builder()
        .name("ingress-http")
        .upstream(&upstream_config)
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

    // We instantiate the rsa encryption tool
    let mut rng = rand::thread_rng();
    let public = RsaPublicKey::from_public_key_pem(PUBLIC_KEY)?;

    // We instantiate the aes decryption tool
    let aes_key = Key::<Aes256Gcm>::from_slice(AES_KEY.as_bytes());
    let aes = Aes256Gcm::new(aes_key);

    // We encrypt and encode the "nonce" for the request
    let encrypted = public.encrypt(&mut rng, Pkcs1v15Encrypt, NONCE.as_bytes())?;
    let encoded = hex::encode(encrypted);

    // Perform an actual request
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{flex_url}/hello"))
        .header("nonce", encoded)
        .send()
        .await?;
    assert_eq!(response.status(), 202);

    // We decoded and decrypt the response from the policy.
    let body = response.text().await?;

    let decoded = hex::decode(body)?;
    let payload = Payload {
        msg: &decoded,
        aad: &[],
    };
    let nonce = GenericArray::from_slice(NONCE.as_bytes());
    let decrypted = aes
        .decrypt(nonce, payload)
        .map_err(|_| anyhow!("Failed to decrypt."))?;
    let decrypted = String::from_utf8(decrypted)?;

    // Assert on the response
    assert_eq!(decrypted, "World!");
    Ok(())
}

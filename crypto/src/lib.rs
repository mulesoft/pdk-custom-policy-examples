// Copyright 2023 Salesforce, Inc. All rights reserved.
mod generated;

use anyhow::{anyhow, Result};

use pdk::hl::*;
use pdk::logger::{debug, warn};

use crate::generated::config::Config;

use aes_gcm::{
    aead::{generic_array::GenericArray, Aead, KeyInit},
    Aes256Gcm, Key,
};
use rsa::pkcs1::DecodeRsaPrivateKey;
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey};
use std::convert::TryInto;

const NONCE_HEADER: &str = "nonce";

/// Function to simplify error handling.
async fn request_filter(state: RequestState, key: &RsaPrivateKey) -> Flow<Vec<u8>> {
    match inner_request_filter(state, key).await {
        Ok(nonce) => Flow::Continue(nonce),
        Err(resp) => Flow::Break(resp),
    }
}

/// This function reads the "nonce" header incoming request, decode it, decrypt it and saves it to be used in the response flow.
async fn inner_request_filter(
    state: RequestState,
    key: &RsaPrivateKey,
) -> Result<Vec<u8>, Response> {
    let state = state.into_headers_state().await;

    // Read the desired header.
    let header = state
        .handler()
        .header(NONCE_HEADER)
        .ok_or_else(|| Response::new(401).with_body(format!("Missing {NONCE_HEADER} header.")))?;

    // Once read we remove the header to avoid propagation to the backend.
    state.handler().remove_header(NONCE_HEADER);

    // This example assumes that the bytes to be used as the "nonce" to be used in the aes-gcm algorithm was encoded in hex.
    let decoded = hex::decode(header).map_err(|_| {
        Response::new(401).with_body(format!("Failed to decode {NONCE_HEADER} header."))
    })?;

    // Once we decoded the bytes we proceed to decrypt them to obtain the "nonce".
    let nonce = key.decrypt(Pkcs1v15Encrypt, &decoded).map_err(|_| {
        Response::new(401).with_body(format!("Failed to decrypt {NONCE_HEADER} header."))
    })?;

    // We log the "nonce" value for debugging purposes.
    debug!("Nonce was: {}", String::from_utf8_lossy(&nonce));

    Ok(nonce)
}

/// This function modifies the payload by encrypting in aes-gcm with the nonce provided in the request.
async fn response_filter(state: ResponseState, nonce_bytes: RequestData<Vec<u8>>, aes: &Aes256Gcm) {
    let RequestData::Continue(nonce_bytes) = nonce_bytes else {
        debug!("Nonce bytes were not fully generated in the request filter.");
        return;
    };

    let state = state.into_headers_state().await;
    // Removing the content-length header enables us to modify the size of the payload, otherwise we might be losing or adding bytes to the response.
    state.handler().remove_header("content-length");

    let state = state.into_body_state().await;
    // We read the body.
    let body = state.handler().body();

    // We encrypt the payload
    let nonce = GenericArray::from_slice(&nonce_bytes);
    if let Ok(bytes) = aes.encrypt(nonce, body.as_slice()) {
        // We encode the encryption result using hex.
        let text = hex::encode(bytes);
        // We set the body bytes.
        if let Err(err) = state.handler().set_body(text.as_bytes()) {
            warn!("Error writing the body. Cause: {err}");
        }
    }
}

#[entrypoint]
async fn configure(launcher: Launcher, Configuration(bytes): Configuration) -> Result<()> {
    let config: Config = serde_json::from_slice(&bytes).map_err(|err| {
        anyhow!(
            "Failed to parse configuration '{}'. Cause: {}",
            String::from_utf8_lossy(&bytes),
            err
        )
    })?;

    // Parse RSA key from the config.
    let rsa_key = RsaPrivateKey::from_pkcs1_pem(config.rsa_key.as_str())
        .map_err(|err| anyhow!("Failed to parse rsa key. Cause: {err}"))?;

    // Parse AES key from the config.
    let aes_key: [u8; 32] = config
        .aes_key
        .as_bytes()
        .try_into()
        .map_err(|err| anyhow!("Provided key is invalid {err}"))?;
    let aes_key = Key::<Aes256Gcm>::from_slice(&aes_key);
    let aes = Aes256Gcm::new(aes_key);

    let filter = on_request(|rs| request_filter(rs, &rsa_key))
        .on_response(|rs, rd| response_filter(rs, rd, &aes));

    launcher.launch(filter).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use aes_gcm::{
        aead::{generic_array::GenericArray, Aead, KeyInit},
        Aes256Gcm, Key,
    };
    use pdk_unit::{UnitHttpMessage, UnitHttpRequest, UnitHttpResponse, UnitTestBuilder};
    use rand::RngCore;
    use rsa::pkcs1::DecodeRsaPrivateKey;
    use rsa::{Pkcs1v15Encrypt, RsaPrivateKey};
    use serde_json::json;
    use std::convert::TryInto;

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

    const AES_KEY: &str = "42F56B955DEA9D821F2A38E3CCDCCQWE";

    fn config() -> String {
        json!({
            "rsa_key": PRIVATE_KEY,
            "aes_key": AES_KEY
        })
        .to_string()
    }

    #[test]
    fn request_without_nonce_header_returns_401() {
        let mut tester = UnitTestBuilder::default()
            .with_config(config())
            .with_entrypoint(crate::configure);

        let response = tester.request_full(UnitHttpRequest::get());

        assert_eq!(response.status_code(), 401);
    }

    #[test]
    fn request_with_invalid_nonce_returns_401() {
        let mut tester = UnitTestBuilder::default()
            .with_config(config())
            .with_entrypoint(crate::configure);

        let response =
            tester.request_full(UnitHttpRequest::get().with_header("nonce", "not-valid-hex"));

        assert_eq!(response.status_code(), 401);
    }

    #[test]
    fn request_with_valid_nonce_returns_200() {
        let rsa_key = RsaPrivateKey::from_pkcs1_pem(PRIVATE_KEY).unwrap();
        let pub_key = rsa_key.to_public_key();

        let mut nonce = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce);

        let encrypted = pub_key
            .encrypt(&mut rand::thread_rng(), Pkcs1v15Encrypt, &nonce)
            .unwrap();
        let nonce_header = hex::encode(encrypted);

        let response_body = "hello world";

        let mut tester = UnitTestBuilder::default()
            .with_config(config())
            .with_backend(UnitHttpResponse::new(200).with_body(response_body))
            .with_entrypoint(crate::configure);

        let response = tester.request_full(
            UnitHttpRequest::get()
                .with_header("nonce", &nonce_header)
                .with_body(response_body),
        );

        assert_eq!(response.status_code(), 200);

        let aes_key: [u8; 32] = AES_KEY.as_bytes().try_into().unwrap();
        let aes = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&aes_key));
        let expected_body = aes
            .encrypt(GenericArray::from_slice(&nonce), response_body.as_bytes())
            .unwrap();
        assert_eq!(response.body(), hex::encode(expected_body).as_bytes());
    }
}

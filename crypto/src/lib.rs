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
async fn response_filter(
    state: ResponseState,
    RequestData(nonce_bytes): RequestData<Vec<u8>>,
    aes: &Aes256Gcm,
) {
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

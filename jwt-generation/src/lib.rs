// Copyright 2023 Salesforce, Inc. All rights reserved.
mod generated;

use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::time::SystemTime;

use crate::generated::config::Config;
use pdk::hl::*;
use pdk::jwt::model::SigningAlgorithm;
use pdk::jwt::model::{JWTClaims, SigningKeyLength};
use pdk::jwt::JwtGenerator;
use serde_json::Value;

async fn request_filter(
    request_state: RequestState,
    config: &Config,
    generator: &JwtGenerator,
) -> Flow<()> {
    let headers_state = request_state.into_headers_state().await;

    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let mut claims: HashMap<String, Value> = HashMap::new();
    claims.insert("iss".to_string(), "test_policy".to_string().into());
    claims.insert(
        "scope".to_string(),
        ["READ".to_string(), "WRITE".to_string()].into(),
    );
    claims.insert("iat".to_string(), now.into());
    claims.insert("exp".to_string(), (now + 60).into());

    let mut headers: HashMap<String, Value> = HashMap::new();
    headers.insert("kid".to_string(), config.kid.clone().into());
    headers.insert("jku".to_string(), config.jku.clone().into());

    let Ok(jwt_claims) = JWTClaims::new(None, None, None, claims, headers) else {
        return Flow::Break(Response::new(503));
    };

    let Ok(jwt) = generator.jwt(jwt_claims) else {
        return Flow::Break(Response::new(503));
    };

    Flow::Break(Response::new(200).with_body(jwt))
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

    let generator = JwtGenerator::new(
        SigningAlgorithm::Rsa,
        SigningKeyLength::Len256,
        &config.private_key,
    )?;

    let filter = on_request(|rs| request_filter(rs, &config, &generator));

    launcher.launch(filter).await?;

    Ok(())
}

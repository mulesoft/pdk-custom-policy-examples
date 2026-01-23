// Copyright 2025 Salesforce, Inc. All rights reserved.
mod generated;

use std::str::FromStr;
use anyhow::{Result};

use pdk::hl::*;
use pdk::logger::{debug, error};
use pdk::token_introspection::{ScopesValidator, TokenValidator, TokenValidatorBuilder};

const AUTHORIZATION_HEADER: &str = "authorization";
const CONTENT_TYPE_JSON: &str = "application/json; charset=UTF-8";

async fn request_filter(
    request_state: RequestState,
    validator: &TokenValidator,
) -> Flow<()> {
    let headers_state = request_state.into_headers_state().await;
    let handler = headers_state.handler();

    let auth_header = handler.header(AUTHORIZATION_HEADER).unwrap_or_else(|| String::from(""));
    let token = auth_header.split_whitespace().collect::<Vec<_>>()[1];

    debug!("Token extracted successfully (len={})", token.len());

    // Validate token.
    let result = validator
        .validate(&token)
        .await;

    match result {
        Ok(_success) => Flow::Continue(()),
        Err(_error) => {
            Flow::Break(
                Response::new(401)
                    .with_headers(vec![(
                        "Content-Type".to_string(),
                        CONTENT_TYPE_JSON.to_string(),
                    )])
                    .with_body(serde_json::json!({"error": "Unauthorized"}).to_string())
            )
        }
    }
}

#[entrypoint]
async fn configure(
    launcher: Launcher,
    Configuration(_bytes): Configuration,
    validator_builder: TokenValidatorBuilder,
) -> Result<()> {

    let upstream = "http://oauth-server:8080";
    let service = Service::new(
        &upstream,
        Uri::from_str(&format!("http://{upstream}")).expect("uri must be valid"),
    );

    let validator_instance = validator_builder
        .new("token-cache")
        .with_path("/introspect")
        .with_authorization_value("Basic YWRtaW46YWRtaW4=")
        .with_expires_in_attribute("exp")
        .with_max_token_ttl(600)
        .with_timeout_ms(10000)
        .with_max_cache_entries(10000)
        .with_scopes_validator(ScopesValidator::all(vec![String::from("read"), String::from("write")]))
        .with_service(service);

    let validator = validator_instance.build().map_err(|e| {
        error!("Failed to build TokenValidator: {e}");
        LaunchError {}
    })?;

    let filter = on_request(|rs| request_filter(rs, &validator));
    launcher.launch(filter).await?;
    Ok(())
}
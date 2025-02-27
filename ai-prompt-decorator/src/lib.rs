// Copyright 2023 Salesforce, Inc. All rights reserved.
mod generated;
mod payload;

use anyhow::{anyhow, Result};

use pdk::hl::*;
use pdk::logger;

use crate::generated::config::Config;

use payload::PayloadDecorator;

/// Decorates a chat request.
async fn decorate_request(
    headers_state: RequestHeadersState,
    decorator: &PayloadDecorator<'_>,
) -> Result<()> {
    let headers_handler = headers_state.handler();

    // Removing old content length header before manipulating body
    headers_handler.remove_header("content-length");

    // Move to the body state.
    let body_state = headers_state.into_body_state().await;
    let body_handler = body_state.handler();

    // Extract body
    let input_body = body_handler.body();

    // Deserialize payload
    let payload = serde_json::from_slice(&input_body)
        .map_err(|e| anyhow!("Could not deserialize body: {e}"))?;

    // Decorate payload
    let decorated_payload = decorator.decorate(&payload);

    let output_body = serde_json::to_vec(&decorated_payload)
        .map_err(|e| anyhow!("Could not serialize decorated body: {e}"))?;

    body_handler
        .set_body(&output_body)
        .map_err(|e| anyhow!("Could not set body: {e}"))?;

    Ok(())
}

/// Decorates the input chat request.
async fn request_filter(
    headers_state: RequestHeadersState,
    decorator: &PayloadDecorator<'_>,
) -> Flow<()> {
    logger::info!("Processing incoming request.");

    match decorate_request(headers_state, decorator).await {
        Ok(_) => {
            logger::info!("Request decorated.");
            Flow::Continue(())
        }
        Err(e) => {
            logger::info!("{e}");
            Flow::Break(Response::new(400).with_body("Bad request."))
        }
    }
}

#[entrypoint]
async fn configure(launcher: Launcher, Configuration(bytes): Configuration) -> Result<()> {
    logger::info!("Initializing AI prompt decorator policy.");

    let config: Config = serde_json::from_slice(&bytes).map_err(|err| {
        anyhow!(
            "Failed to parse configuration '{}'. Cause: {err}",
            String::from_utf8_lossy(&bytes),
        )
    })?;

    let decorator = PayloadDecorator::from_config(&config);

    let filter = on_request(|request_state| request_filter(request_state, &decorator));

    logger::info!("Starting filters.");

    launcher.launch(filter).await?;

    Ok(())
}

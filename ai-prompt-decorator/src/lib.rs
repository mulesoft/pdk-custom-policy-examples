// Copyright 2023 Salesforce, Inc. All rights reserved.
mod decorator;
mod generated;
mod openai;

use anyhow::{anyhow, Result};

use pdk::hl::*;
use pdk::logger;
use serde_json::json;

use crate::generated::config::Config;

use decorator::CompletionDecorator;

/// Decorates a chat request.
async fn decorate_request(
    headers_state: RequestHeadersState,
    decorator: &CompletionDecorator<'_>,
) -> Result<(), (u32, &'static str)> {
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
        .map_err(|_| (400, "Unable to deserialize JSON payload."))?;

    // Decorate payload
    let decorated_payload = decorator.decorate(payload);

    let output_body = serde_json::to_vec(&decorated_payload).map_err(|e| {
        logger::error!("Unable to serialize decorated body: {e:?}");
        (500, "Internal error.")
    })?;

    body_handler.set_body(&output_body).map_err(|e| {
        logger::error!("Unable to set new body: {e:?}");
        (400, "Payload too long.")
    })
}

/// Decorates the input chat request.
async fn request_filter(
    headers_state: RequestHeadersState,
    decorator: &CompletionDecorator<'_>,
) -> Flow<()> {
    logger::info!("Processing incoming request.");

    match decorate_request(headers_state, decorator).await {
        Ok(_) => {
            logger::info!("Request decorated.");
            Flow::Continue(())
        }
        Err((status_code, error)) => Flow::Break(
            Response::new(status_code)
                .with_body(json!({ "error": error }).to_string())
                .with_headers([("Content-Type".to_string(), "application/json".to_string())]),
        ),
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

    let decorator = CompletionDecorator::from_config(&config);

    let filter = on_request(|request_state| request_filter(request_state, &decorator));

    logger::info!("Starting filters.");

    launcher.launch(filter).await?;

    Ok(())
}

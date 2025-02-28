// Copyright 2023 Salesforce, Inc. All rights reserved.
mod generated;
mod openai;
mod sanitizer;

use anyhow::{anyhow, Result};

use pdk::{hl::*, logger};
use sanitizer::CompletionSanitizer;
use serde::Serialize;

use crate::generated::config::Config;

/// Represents an error to be serialized and returned as early response.
#[derive(Serialize)]
struct FilterError {
    #[serde(skip_serializing)]
    status_code: u32,

    error: &'static str,
}

/// Sanitizes an incoming request by omiting or blocking OpenAI chat messages based on configuration
/// filters.
async fn sanitize_request(
    request_state: RequestState,
    sanitizer: &CompletionSanitizer,
) -> Result<(), FilterError> {
    logger::info!("Sanitizing an incoming request.");

    let headers_state = request_state.into_headers_state().await;
    let headers_handler = headers_state.handler();

    // Removing content-length
    headers_handler.remove_header("content-length");

    let body_state = headers_state.into_body_state().await;

    // Skip requests without body
    if !body_state.contains_body() {
        logger::info!("Empty body.");
        return Ok(());
    }

    let handler = body_state.handler();
    let body = handler.body();

    // Deserialize completion from incoming body.
    let completion = serde_json::from_slice(&body).map_err(|_| FilterError {
        status_code: 400,
        error: "Unrecognized JSON structure.",
    })?;

    // Sanitize completion or block request.
    let sanitized_completion = sanitizer.sanitize(completion).ok_or(FilterError {
        status_code: 403,
        error: "Forbidden tokens.",
    })?;
    logger::info!("sanitized completion = {sanitized_completion:?}");
    // Serialize sanitized completion
    let sanitized_body = serde_json::to_vec(&sanitized_completion).map_err(|e| {
        logger::error!("Unable to serialize completion: {e:?}");
        FilterError {
            status_code: 500,
            error: "Internal problem.",
        }
    })?;

    // Set the new body.
    handler.set_body(&sanitized_body).map_err(|e| {
        logger::error!("Unable to set body: {e:?}");
        FilterError {
            status_code: 500,
            error: "Internal problem.",
        }
    })?;

    logger::info!("Request sanitized");

    Ok(())
}

async fn request_filter(request_state: RequestState, sanitizer: &CompletionSanitizer) -> Flow<()> {
    match sanitize_request(request_state, sanitizer).await {
        // No errors, request flow must continue.
        Ok(_) => Flow::Continue(()),

        // Errors must return an early response.
        Err(e) => {
            logger::info!("Early response reached.");
            Flow::Break(
                Response::new(e.status_code)
                    .with_body(serde_json::to_vec(&e).expect("Response body serialization"))
                    .with_headers([("Content-Type".to_string(), "application/json".to_string())]),
            )
        }
    }
}

#[entrypoint]
async fn configure(launcher: Launcher, Configuration(bytes): Configuration) -> Result<()> {
    let config: Config = serde_json::from_slice(&bytes).map_err(|err| {
        anyhow!(
            "Failed to parse configuration '{}'. Cause: {err}",
            String::from_utf8_lossy(&bytes),
        )
    })?;

    let validator = CompletionSanitizer::from_config(config)
        .map_err(|err| anyhow!("Unable to create regex. Cause: {err}"))?;

    logger::info!("Initializing OpenAI API filters");
    let filter = on_request(|rs| request_filter(rs, &validator));

    launcher.launch(filter).await?;

    Ok(())
}

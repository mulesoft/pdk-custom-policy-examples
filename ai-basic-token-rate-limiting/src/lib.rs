// Copyright 2023 Salesforce, Inc. All rights reserved.
mod generated;
mod openai;
mod validator;

use std::time::Duration;

use anyhow::{anyhow, Result};
use pdk::cache::{Cache, CacheBuilder};
use serde_json::json;

use pdk::hl::*;
use pdk::logger;
use validator::{RateLimitError, RateLimitValidator};

use crate::generated::config::Config;

async fn validate_request(
    body_state: RequestBodyState,
    validator: &RateLimitValidator<impl Cache>,
) -> Result<(), (u32, &'static str)> {
    // Avoid validating empty bodies.
    if !body_state.contains_body() {
        return Ok(());
    }

    // Extract current body
    let body = body_state.handler().body();

    validator.validate_payload(&body).map_err(|e| match e {
        RateLimitError::Exceeded => (403, "Too many tokens. Rate Limit exceeded"),
        RateLimitError::BodyDeserialization(_) => (400, "Wrong body format"),
        e => {
            logger::error!("{e}");
            (500, "Internal problem")
        }
    })
}

/// A filter that applies an LLM rate limit validation to the incoming request.
async fn request_filter(
    body_state: RequestBodyState,
    validator: &RateLimitValidator<impl Cache>,
) -> Flow<()> {
    match validate_request(body_state, validator).await {
        // Succesful validation must continue request flow
        Ok(_) => Flow::Continue(()),

        // Error must be blocked
        Err((status_code, error)) => Flow::Break(
            Response::new(status_code)
                .with_body(json!({ "error": error}).to_string())
                .with_headers([("Content-Type".to_string(), "application/json".to_string())]),
        ),
    }
}

#[entrypoint]
async fn configure(
    launcher: Launcher,
    Configuration(bytes): Configuration,
    cache_builder: CacheBuilder,
) -> Result<()> {
    let cache = cache_builder
        .new(String::from("caching"))
        .max_entries(10)
        .build();
    let config: Config = serde_json::from_slice(&bytes).map_err(|err| {
        anyhow!(
            "Failed to parse configuration '{}'. Cause: {err}",
            String::from_utf8_lossy(&bytes),
        )
    })?;

    let validator = RateLimitValidator::new(
        Duration::from_millis(config.time_period_in_milliseconds as u64),
        config.maximum_tokens as usize,
        cache,
    )?;

    let filter = on_request(|rs| request_filter(rs, &validator));

    launcher.launch(filter).await?;
    Ok(())
}

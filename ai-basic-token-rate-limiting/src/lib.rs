// Copyright 2023 Salesforce, Inc. All rights reserved.
mod generated;
mod validator;

use anyhow::{anyhow, Result};
use pdk::cache::{Cache, CacheBuilder};

use pdk::hl::*;
use pdk::logger;
use validator::{RateLimitError, RateLimitValidator};

use crate::generated::config::Config;

/// A filter that applies an LLM rate limit validation to the incoming request.
async fn request_filter(
    body_state: RequestBodyState,
    validator: &RateLimitValidator<impl Cache>,
) -> Flow<()> {
    // Avoid validating empty bodies.
    if !body_state.contains_body() {
        return Flow::Continue(());
    }

    let body = body_state.handler().body();
    match validator.validate_payload(&body) {
        // If validation is ok, the request flow continues.
        Ok(_) => Flow::Continue(()),
        Err(error) => {
            let response = match error {
                RateLimitError::Exceeded => Response::new(403)
                    .with_body(r#"{ "error": "Too many tokens. Rate Limit exceeded" }"#),
                RateLimitError::BodyDeserialization(_) => {
                    Response::new(400).with_body(r#"{ "error": "Wrong body format" }"#)
                }
                e => {
                    logger::error!("{e}");
                    Response::new(500).with_body(r#"{ "error": "Internal problem" }"#)
                }
            };

            // All errors break the request.
            Flow::Break(
                response
                    .with_headers([("Content-Type".to_string(), "application/json".to_string())]),
            )
        }
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
            "Failed to parse configuration '{}'. Cause: {}",
            String::from_utf8_lossy(&bytes),
            err
        )
    })?;

    let validator = RateLimitValidator::new(config, cache)?;

    let filter = on_request(|rs| request_filter(rs, &validator));

    launcher.launch(filter).await?;
    Ok(())
}

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

    let completion = serde_json::from_slice(&body).map_err(|_| (400, "Invalid body structure"))?;

    validator.validate(completion).map_err(|e| match e {
        RateLimitError::Exceeded => (403, "Too many tokens. Rate Limit exceeded"),
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
                .with_body(json!({ "error": error }).to_string())
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

#[cfg(test)]
mod tests {
    use pdk_unit::{UnitHttpRequest, UnitTestBuilder};
    use serde_json::json;

    fn config(maximum_tokens: u64, time_period_ms: u64) -> String {
        json!({
            "maximumTokens": maximum_tokens,
            "timePeriodInMilliseconds": time_period_ms
        })
        .to_string()
    }

    fn completion_body(content: &str) -> String {
        json!({
            "model": "gpt-4",
            "messages": [{"role": "user", "content": content}]
        })
        .to_string()
    }

    #[test]
    fn request_within_token_limit_passes() {
        let mut tester = UnitTestBuilder::default()
            .with_config(config(100, 60000))
            .with_entrypoint(crate::configure);

        // "this has four tokens" = 4 tokens, well within limit
        let response = tester
            .request(UnitHttpRequest::post().with_body(completion_body("this has four tokens")));

        assert_eq!(response.status_code(), 200);
    }

    #[test]
    fn request_exceeding_token_limit_returns_403() {
        let mut tester = UnitTestBuilder::default()
            .with_config(config(1, 60000))
            .with_entrypoint(crate::configure);

        // "this has four tokens" = 4 tokens, exceeds limit of 1
        let response = tester
            .request(UnitHttpRequest::post().with_body(completion_body("this has four tokens")));

        assert_eq!(response.status_code(), 403);
    }

    #[test]
    fn request_without_body_passes() {
        let mut tester = UnitTestBuilder::default()
            .with_config(config(1, 60000))
            .with_entrypoint(crate::configure);

        let response = tester.request(UnitHttpRequest::post());

        assert_eq!(response.status_code(), 200);
    }

    #[test]
    fn invalid_json_body_returns_400() {
        let mut tester = UnitTestBuilder::default()
            .with_config(config(100, 60000))
            .with_entrypoint(crate::configure);

        let response = tester.request(UnitHttpRequest::post().with_body("not valid json"));

        assert_eq!(response.status_code(), 400);
    }

    #[test]
    fn cumulative_requests_exceeding_limit_returns_403() {
        let mut tester = UnitTestBuilder::default()
            .with_config(config(5, 60000))
            .with_entrypoint(crate::configure);

        // "this has four tokens" = 4 tokens, fits in limit of 5
        let first = tester
            .request(UnitHttpRequest::post().with_body(completion_body("this has four tokens")));
        assert_eq!(first.status_code(), 200);

        // second request pushes cumulative count over 5
        let second = tester
            .request(UnitHttpRequest::post().with_body(completion_body("this has four tokens")));
        assert_eq!(second.status_code(), 403);
    }
}

// Copyright 2023 Salesforce, Inc. All rights reserved.

mod generated;

use pdk::hl::timer::Clock;
use pdk::hl::*;
use pdk::logger;
use pdk::metadata::Tier;
use pdk::rl::{RateLimit, RateLimitBuilder, RateLimitInstance, RateLimitResult};
use std::rc::Rc;
use std::time::Duration;

use crate::generated::config::Config;

// HTTP headers for client identification
pub const API_KEY_HEADER: &str = "x-api-key";
pub const USER_ID_HEADER: &str = "x-user-id";

// Default values and configs
const TIMER_PERIOD_MS: u64 = 100;
const BUILDER_ID: &str = "multi-instance-rate-limiting";
const REQUEST_AMOUNT: usize = 1;

// Rate limit group names
const API_KEY_RATE_LIMIT_GROUP: &str = "api_key_rate_limit";
const USER_ID_RATE_LIMIT_GROUP: &str = "user_id_rate_limit";

/// Checks if a rate limit is allowed for a given client key and configuration
async fn is_request_allowed(
    rate_limiter: &RateLimitInstance,
    group_name: &str,
    client_key: &str,
) -> Result<bool, String> {
    match rate_limiter
        .is_allowed(group_name, client_key, REQUEST_AMOUNT)
        .await
    {
        Ok(RateLimitResult::Allowed(_)) => Ok(true),
        Ok(RateLimitResult::TooManyRequests(_)) => {
            logger::warn!(
                "Rate limit exceeded for client: '{client_key}' in group: '{group_name}'"
            );
            Ok(false)
        }
        Err(e) => {
            logger::error!("Rate limiting error for group '{group_name}': {e}");
            Err(format!("Rate limiting error: {e}"))
        }
    }
}

/// Checks rate limit for a header value and returns Flow response
async fn check_header(
    rate_limiter: &RateLimitInstance,
    group_name: &str,
    header_value: &str,
    error_message: &str,
) -> Result<(), Flow<()>> {
    let allowed = is_request_allowed(rate_limiter, group_name, header_value).await;
    match allowed {
        Ok(false) => Err(Flow::Break(Response::new(429).with_body(error_message))),
        Err(_) => Err(Flow::Break(
            Response::new(503).with_body("Service temporarily unavailable"),
        )),
        Ok(true) => Ok(()),
    }
}

/// Checks rate limits for all present headers and returns appropriate response
async fn check_all_rate_limits(
    rate_limiter: &RateLimitInstance,
    api_key_header: Option<String>,
    user_id_header: Option<String>,
) -> Result<(), Flow<()>> {
    let headers = [
        (
            api_key_header.as_deref(),
            API_KEY_RATE_LIMIT_GROUP,
            "API key rate limit exceeded",
        ),
        (
            user_id_header.as_deref(),
            USER_ID_RATE_LIMIT_GROUP,
            "User ID rate limit exceeded",
        ),
    ];

    // Check each header if present
    for (header_value, group_name, error_message) in headers {
        if let Some(value) = header_value {
            check_header(rate_limiter, group_name, value, error_message).await?;
        }
    }

    Ok(())
}

/// Main request filter that applies rate limiting to incoming requests
async fn request_filter(state: RequestHeadersState, rate_limiter: &RateLimitInstance) -> Flow<()> {
    // Extract client identifiers, only if headers are present
    let api_key_header = state.handler().header(API_KEY_HEADER);
    let user_id_header = state.handler().header(USER_ID_HEADER);

    // Check all rate limits
    match check_all_rate_limits(rate_limiter, api_key_header, user_id_header).await {
        Ok(_) => Flow::Continue(()),
        Err(flow) => flow,
    }
}

/// Policy entrypoint that configures rate limiting and launches request handler
#[entrypoint]
async fn configure(
    launcher: Launcher,
    rate_limit_builder: RateLimitBuilder,
    Configuration(configuration): Configuration,
    clock: Clock,
) -> Result<(), String> {
    let config: Config = serde_json::from_slice(&configuration)
        .map_err(|e| format!("Failed to deserialize configuration: {e:?}"))?;

    logger::info!(
        "Initializing multi-instance rate limiting with API key and User ID configurations"
    );

    // Build buckets configuration from the rate limits
    let mut buckets = Vec::new();

    // Add API key rate limit bucket
    let api_config = config.api_key_rate_limit;
    buckets.push((
        API_KEY_RATE_LIMIT_GROUP.to_string(),
        vec![Tier {
            requests: api_config.requests_per_window as u64,
            period_in_millis: api_config.window_size_seconds as u64 * 1000,
        }],
    ));

    // Add User ID rate limit bucket
    let user_config = config.user_id_rate_limit;
    buckets.push((
        USER_ID_RATE_LIMIT_GROUP.to_string(),
        vec![Tier {
            requests: user_config.requests_per_window as u64,
            period_in_millis: user_config.window_size_seconds as u64 * 1000,
        }],
    ));

    // Create timer for rate limit sync (TIMER_PERIOD_MS intervals)
    let timer = clock.period(Duration::from_millis(TIMER_PERIOD_MS));

    // Create rate limiter with shared storage for multi-instance support.
    // This supports both local shared storage and distributed storage (Redis).
    let builder = rate_limit_builder
        .new(BUILDER_ID.to_string())
        .clustered(Rc::new(timer))
        .shared();

    let rate_limiter = builder
        .buckets(buckets)
        .build()
        .map_err(|e| format!("Failed to build the rate limit handler: {e:?}"))?;

    // Launch the request handler with rate limiting applied
    launcher
        .launch(on_request(|request| request_filter(request, &rate_limiter)))
        .await
        .map_err(|e| format!("Failed to launch request handler: {e:?}"))?;

    Ok(())
}

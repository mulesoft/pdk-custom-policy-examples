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

/// Extracts the client key based on the key selector configuration
fn extract_client_key(state: &RequestHeadersState, key_selector: &str) -> String {
    let key = match key_selector {
        "api_key" => state
            .handler()
            .header("x-api-key")
            .unwrap_or("unknown".to_string()),
        "user_id" => state
            .handler()
            .header("x-user-id")
            .unwrap_or("unknown".to_string()),
        _ => {
            logger::warn!("Unknown key selector: '{}', using 'unknown'", key_selector);
            "unknown".to_string()
        }
    };

    key
}

/// Main request filter that applies rate limiting
async fn request_filter(
    state: RequestHeadersState,
    config: &Config,
    rate_limiter: &RateLimitInstance,
) -> Flow<()> {
    // Apply all rate limit configurations
    for rate_limit_config in &config.rate_limits {
        let client_key = extract_client_key(&state, &rate_limit_config.key_selector);
        let group = &rate_limit_config.group_name;
        let amount = 1; // Each request counts as 1

        match rate_limiter.is_allowed(group, &client_key, amount).await {
            Ok(RateLimitResult::Allowed(_)) => {
                // Continue checking other rate limits
                continue;
            }
            Ok(RateLimitResult::TooManyRequests(_)) => {
                logger::warn!(
                    "Rate limit exceeded for client: '{}' in group: '{}'",
                    client_key,
                    group
                );
                return Flow::Break(Response::new(429).with_body("Rate limit exceeded"));
            }
            Err(e) => {
                logger::error!("Rate limiting error for group '{}': {}", group, e);
                return Flow::Break(
                    Response::new(503).with_body("Service temporarily unavailable"),
                );
            }
        }
    }

    // All rate limits passed, allow the request
    Flow::Continue(())
}

/// Policy entrypoint that configures rate limiting and launches request handler
#[entrypoint]
async fn configure(
    launcher: Launcher,
    rate_limit_builder: RateLimitBuilder,
    Configuration(configuration): Configuration,
    clock: Clock, // Inject the clock from PDK
) -> Result<(), String> {
    let config: Config = serde_json::from_slice(&configuration)
        .map_err(|e| format!("Failed to deserialize configuration: {e:?}"))?;

    logger::info!(
        "Initializing multi-instance rate limiting with {} configurations",
        config.rate_limits.len()
    );

    // Create buckets configuration from the config
    let mut buckets = Vec::new();
    for rate_limit_config in &config.rate_limits {
        let tier = Tier {
            requests: rate_limit_config.requests_per_window as u64,
            period_in_millis: rate_limit_config.window_size_seconds as u64 * 1000,
        };
        buckets.push((rate_limit_config.group_name.clone(), vec![tier]));
    }

    let timer = clock.period(Duration::from_millis(100));

    // Create rate limiter with shared storage for multi-instance support.
    // This supports both local shared storage and distributed storage (Redis).
    let builder = rate_limit_builder
        .new("my-builder-id".to_string())
        .clustered(Rc::new(timer))
        .shared();

    let rate_limiter = builder
        .buckets(buckets)
        .build()
        .map_err(|e| format!("Failed to build the rate limit handler: {e:?}"))?;

    launcher
        .launch(on_request(|request| {
            request_filter(request, &config, &rate_limiter)
        }))
        .await
        .map_err(|e| format!("Failed to launch request handler: {e:?}"))?;

    Ok(())
}

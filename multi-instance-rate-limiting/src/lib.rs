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

// Default values and configuration constants
const DEFAULT_CLIENT_KEY: &str = "unknown";
const TIMER_PERIOD_MS: u64 = 100;
const BUILDER_ID: &str = "multi-instance-rate-limiting";
const REQUEST_AMOUNT: usize = 1;

/// Extracts the client key based on the key selector configuration
fn extract_client_key(state: &RequestHeadersState, key_selector: &str) -> String {
    match key_selector {
        "api_key" => state
            .handler()
            .header(API_KEY_HEADER)
            .unwrap_or(DEFAULT_CLIENT_KEY.to_string()),
        "user_id" => state
            .handler()
            .header(USER_ID_HEADER)
            .unwrap_or(DEFAULT_CLIENT_KEY.to_string()),
        _ => {
            logger::warn!("Unknown key selector: '{key_selector}' will use default client key");
            DEFAULT_CLIENT_KEY.to_string()
        }
    }
}

/// Main request filter that applies rate limiting to incoming requests
async fn request_filter(
    state: RequestHeadersState,
    config: &Config,
    rate_limiter: &RateLimitInstance,
) -> Flow<()> {
    // Apply all rate limit configurations to each request
    for rate_limit_config in &config.rate_limits {
        let client_key = extract_client_key(&state, &rate_limit_config.key_selector);
        let group = &rate_limit_config.group_name;

        // Check if this request is allowed according to the rate limit
        match rate_limiter
            .is_allowed(group, &client_key, REQUEST_AMOUNT)
            .await
        {
            Ok(RateLimitResult::Allowed(_)) => {
                // Continue checking other rate limits
                continue;
            }
            Ok(RateLimitResult::TooManyRequests(_)) => {
                // Rate limit exceeded - block the request
                logger::warn!("Rate limit exceeded for client: '{client_key}' in group: '{group}'");
                return Flow::Break(Response::new(429).with_body("Rate limit exceeded"));
            }
            Err(e) => {
                // Rate limiting error - fail closed for safety
                logger::error!("Rate limiting error for group '{group}': {e}");
                return Flow::Break(
                    Response::new(503).with_body("Service temporarily unavailable"),
                );
            }
        }
    }

    Flow::Continue(()) // All rate limits passed, allow the request
}

impl Config {
    /// Builds the buckets configuration from the rate limits
    fn build_buckets(&self) -> Vec<(String, Vec<Tier>)> {
        self.rate_limits
            .iter()
            .map(|config| {
                let tier = Tier {
                    requests: config.requests_per_window as u64,
                    period_in_millis: config.window_size_seconds as u64 * 1000,
                };
                (config.group_name.clone(), vec![tier])
            })
            .collect()
    }
}

/// Policy entrypoint that configures rate limiting and launches request handler
#[entrypoint]
async fn configure(
    launcher: Launcher,
    rate_limit_builder: RateLimitBuilder,
    Configuration(configuration): Configuration,
    clock: Clock, // Inject the clock from PDK
) -> Result<(), String> {
    // Deserialize configuration from JSON
    let config: Config = serde_json::from_slice(&configuration)
        .map_err(|e| format!("Failed to deserialize configuration: {e:?}"))?;

    logger::info!(
        "Initializing multi-instance rate limiting with {} configurations",
        config.rate_limits.len()
    );

    // Build buckets configuration from the rate limits
    let buckets = config.build_buckets();

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
        .launch(on_request(|request| {
            request_filter(request, &config, &rate_limiter)
        }))
        .await
        .map_err(|e| format!("Failed to launch request handler: {e:?}"))?;

    Ok(())
}

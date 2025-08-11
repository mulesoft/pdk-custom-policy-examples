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
async fn check_rate_limit(
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

/// Main request filter that applies rate limiting to incoming requests
async fn request_filter(state: RequestHeadersState, rate_limiter: &RateLimitInstance) -> Flow<()> {
    // Extract client identifiers, only if headers are present
    let api_key_header = state.handler().header(API_KEY_HEADER);
    let user_id_header = state.handler().header(USER_ID_HEADER);

    // Check API key rate limit only if header is present
    if let Some(api_key) = &api_key_header {
        let api_key_allowed =
            check_rate_limit(rate_limiter, API_KEY_RATE_LIMIT_GROUP, api_key).await;
        match api_key_allowed {
            Ok(false) => {
                return Flow::Break(Response::new(429).with_body("API key rate limit exceeded"))
            }
            Err(_) => {
                return Flow::Break(Response::new(503).with_body("Service temporarily unavailable"))
            }
            Ok(true) => (),
        }
    }

    // Check User ID rate limit only if header is present
    if let Some(user_id) = &user_id_header {
        let user_id_allowed =
            check_rate_limit(rate_limiter, USER_ID_RATE_LIMIT_GROUP, user_id).await;
        match user_id_allowed {
            Ok(false) => {
                return Flow::Break(Response::new(429).with_body("User ID rate limit exceeded"))
            }
            Err(_) => {
                return Flow::Break(Response::new(503).with_body("Service temporarily unavailable"))
            }
            Ok(true) => (),
        }
    }

    // At least one header must be present
    if api_key_header.is_none() && user_id_header.is_none() {
        return Flow::Break(
            Response::new(400)
                .with_body("At least one header (x-api-key or x-user-id) is required"),
        );
    }

    // Rate limits passed
    Flow::Continue(())
}

impl Config {
    /// Builds the buckets configuration from the rate limits
    fn build_buckets(&self) -> Vec<(String, Vec<Tier>)> {
        let mut buckets = Vec::new();

        // Add API key rate limit bucket
        let api_config = &self.api_key_rate_limit;
        let tier = Tier {
            requests: api_config.requests_per_window as u64,
            period_in_millis: api_config.window_size_seconds as u64 * 1000,
        };
        buckets.push((API_KEY_RATE_LIMIT_GROUP.to_string(), vec![tier]));

        // Add User ID rate limit bucket
        let user_config = &self.user_id_rate_limit;
        let tier = Tier {
            requests: user_config.requests_per_window as u64,
            period_in_millis: user_config.window_size_seconds as u64 * 1000,
        };
        buckets.push((USER_ID_RATE_LIMIT_GROUP.to_string(), vec![tier]));

        buckets
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
        .launch(on_request(|request| request_filter(request, &rate_limiter)))
        .await
        .map_err(|e| format!("Failed to launch request handler: {e:?}"))?;

    Ok(())
}

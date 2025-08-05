// Copyright 2023 Salesforce, Inc. All rights reserved.

mod generated;

use anyhow::anyhow;
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
    logger::info!("🔍 Extracting client key for selector: '{}'", key_selector);

    let key = match key_selector {
        "api_key" => {
            let value = state
                .handler()
                .header("x-api-key")
                .unwrap_or("unknown".to_string());
            logger::info!("🔑 Found API key header: '{}'", value);
            value
        }
        "user_id" => {
            let value = state
                .handler()
                .header("x-user-id")
                .unwrap_or("unknown".to_string());
            logger::info!("🔑 Found user ID header: '{}'", value);
            value
        }
        _ => {
            logger::warn!(
                "⚠️ Unknown key selector: '{}', using 'unknown'",
                key_selector
            );
            "unknown".to_string()
        }
    };

    logger::info!(
        "🔑 Final extracted key: '{}' for selector '{}'",
        key,
        key_selector
    );
    key
}

/// Main request filter that applies rate limiting
async fn request_filter(
    state: RequestHeadersState,
    config: &Config,
    rate_limiter: &RateLimitInstance,
) -> Flow<()> {
    let path = state.path();
    logger::info!("🔍 Processing request for path: {}", path);

    // Simple rate limiting - just use the first configuration
    if let Some(rate_limit_config) = config.rate_limits.first() {
        logger::info!(
            "✅ Applying rate limit: group={}",
            rate_limit_config.group_name
        );

        let client_key = extract_client_key(&state, &rate_limit_config.key_selector);
        let group = &rate_limit_config.group_name;
        let amount = 1; // Each request counts as 1

        logger::info!(
            "🔑 Client key: '{}', Group: '{}', Amount: {}",
            client_key,
            group,
            amount
        );
        logger::info!(
            "🔍 About to check rate limit for group '{}' with key '{}'",
            group,
            client_key
        );

        match rate_limiter.is_allowed(group, &client_key, amount).await {
            Ok(RateLimitResult::Allowed(_)) => {
                // Request allowed, continue to backend
                logger::info!(
                    "✅ Request ALLOWED for client: '{}' on path: '{}'",
                    client_key,
                    path
                );
                return Flow::Continue(());
            }
            Ok(RateLimitResult::TooManyRequests(_)) => {
                // Request blocked due to rate limit
                logger::warn!(
                    "❌ Rate limit EXCEEDED for client: '{}' on path: '{}'",
                    client_key,
                    path
                );
                return Flow::Break(Response::new(429).with_body("Rate limit exceeded"));
            }
            Err(e) => {
                // Fail closed - block the request if rate limiting fails
                logger::error!("🚨 Rate limiting ERROR: {}", e);
                return Flow::Break(
                    Response::new(503).with_body("Service temporarily unavailable"),
                );
            }
        }
    }

    // No configuration found, allow the request
    logger::info!("⚠️ No rate limit configuration found, allowing request");
    Flow::Continue(())
}

/// Policy entrypoint that configures rate limiting and launches request handler
#[entrypoint]
async fn configure(
    launcher: Launcher,
    rate_limit_builder: RateLimitBuilder,
    Configuration(configuration): Configuration,
    clock: Clock, // Inject the clock from PDK
) -> anyhow::Result<()> {
    let config: Config = serde_json::from_slice(&configuration)?;

    logger::info!(
        "Initializing multi-instance rate limiting with {} configurations",
        config.rate_limits.len()
    );
    
    // Log the distributed storage extraction result
    match &rate_limit_builder.distributed_storage {
        Ok(_) => logger::info!("✅ DistributedStorageClient extracted successfully"),
        Err(e) => logger::warn!("⚠️ Failed to extract DistributedStorageClient: {:?}", e),
    }
    
    // Log the shared storage configuration
    if let Some(shared_storage) = &config.shared_storage {
        logger::info!("📋 Shared storage configured: {}", shared_storage);
    } else {
        logger::warn!("⚠️ No shared storage configured");
    }

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

    // Create rate limiter with shared storage for multi-instance support
    // This supports both local shared storage and distributed storage (Redis)
    let builder = rate_limit_builder
        .new("some-builder-id".to_string())
        .clustered(Rc::new(timer))
        .shared();

    let rate_limiter = builder
        .buckets(buckets)
        .build()
        .map_err(|e| anyhow!("Failed to build the rate limit handler: {e}"))?;
    
    logger::info!("✅ Rate limiter built successfully");

    launcher
        .launch(on_request(|request| {
            request_filter(request, &config, &rate_limiter)
        }))
        .await?;

    Ok(())
}

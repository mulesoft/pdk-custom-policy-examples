// Copyright 2023 Salesforce, Inc. All rights reserved.
mod generated;

use anyhow::{anyhow, Result};

use pdk::hl::*;
use pdk::logger::{error, warn};
use pdk::metadata::Tier;
use pdk::rl::{RateLimit, RateLimitBuilder, RateLimitError, RateLimitInstance, RateLimitResult, RateLimitStatistics};
use crate::generated::config::Config;

pub const TOO_MANY_REQUEST_MESSAGE: &str = "Too Many Requests";
pub const INTERNAL_ERROR_MESSAGE: &str = "Internal Gateway Error";
pub const X_RATE_LIMIT_REMAINING: &str = "x-ratelimit-remaining";
pub const X_RATE_LIMIT_LIMIT: &str = "x-ratelimit-limit";
pub const X_RATE_LIMIT_RESET: &str = "x-ratelimit-reset";
pub const CONTENT_TYPE: &str = "Content-Type";
pub const APPLICATION_JSON: &str = "application/json; charset=UTF-8";

async fn request_filter(_request_state: RequestState, _config: &Config, rate_limit: &RateLimitInstance) -> Flow<RateLimitStatistics> {
    match rate_limit.is_allowed("default", "bucket", 1).await {
        Ok(RateLimitResult::Allowed(stats)) => Flow::Continue(stats),
        Ok(RateLimitResult::TooManyRequests(stats)) => {
            let response = Response::new(429).with_body(TOO_MANY_REQUEST_MESSAGE);
            let expose_headers = vec![
                (CONTENT_TYPE.to_string(), APPLICATION_JSON.to_string()),
                (X_RATE_LIMIT_LIMIT.to_string(), stats.limit.to_string()),
                (X_RATE_LIMIT_REMAINING.to_string(), stats.remaining.to_string()),
                (X_RATE_LIMIT_RESET.to_string(), stats.reset.to_string()),
            ];
            Flow::Break(response.with_headers(expose_headers))
        }
        Err(RateLimitError::MaxHops) => {
            warn!("Unexpected error: Hops reached");
            Flow::Break(Response::new(503).with_body(INTERNAL_ERROR_MESSAGE))
        }
        Err(RateLimitError::Unexpected(_error)) => {
            warn!("Unexpected error: Request error");
            Flow::Break(Response::new(503).with_body(INTERNAL_ERROR_MESSAGE))
        }
        Err(e) => {
            warn!("Unexpected error: {e}");
            Flow::Break(Response::new(503).with_body(INTERNAL_ERROR_MESSAGE))
        }
    }
}

#[entrypoint]
async fn configure(launcher: Launcher,
   Configuration(bytes): Configuration,
   rate_limit_builder: RateLimitBuilder,
) -> Result<()> {
    let config: Config = serde_json::from_slice(&bytes).map_err(|err| {
        anyhow!(
            "Failed to parse configuration '{}'. Cause: {}",
            String::from_utf8_lossy(&bytes),
            err
        )
    })?;

    let mut builder = rate_limit_builder.new("rate-limit-filter".to_string());

    let mut tiers = vec![];
    config.rate_limits.iter().for_each(|rate| {
        tiers.push(Tier {
            requests: rate.maximum_requests as u64,
            period_in_millis: rate.time_period_in_milliseconds as u64,
        })
    });
    // If the rate limit is clusterizable, we can use the clustered feature in the builder.
    // if rate_limit_config.clusterizable {
    //     builder = builder.clustered(Rc::new(clock.period(Duration::from_millis(100))));
    // }
    builder = builder.buckets(vec![("default".to_string(), tiers)]);
    let rate_limit = builder.build().map_err(|e| {
        error!("Failed to build the rate limit handler: {e}");
        LaunchError {}
    })?;


    let filter = on_request(|rs| request_filter(rs, &config, &rate_limit));
    launcher.launch(filter).await?;
    Ok(())
}

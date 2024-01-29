// Copyright 2023 Salesforce, Inc. All rights reserved.
mod generated;

use anyhow::{anyhow, Result};

use pdk::cache::{Cache, CacheBuilder, CacheError};
use pdk::hl::*;
use pdk::logger;

use crate::generated::config::Config;

use chrono::{DateTime, Days, Local, Timelike};
use serde::{Deserialize, Serialize};

/// This enum sends data from the request scope to the response scope.
enum CachingData {
    SaveResponse(String),
    IgnoreCache,
}

/// This struct serializes the response in the cache.
#[derive(Serialize, Deserialize)]
pub struct CachedResponse {
    valid_until: DateTime<Local>,
    status_code: u32,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

impl CachedResponse {
    /// Given the current time check, if the value was cached in the current cache window.
    fn has_expired(&self, now: &DateTime<Local>) -> bool {
        self.valid_until.lt(now)
    }
}

/// Transforms the CachedResponse into a Response
impl From<CachedResponse> for Response {
    fn from(response: CachedResponse) -> Self {
        Response::new(response.status_code)
            .with_headers(response.headers)
            .with_body(response.body)
    }
}

/// Checks if the time is between the given range
fn check_time_in_range(now: u32, start: u32, end: u32) -> bool {
    logger::debug!("Checking {now} in rime range: {start}-{end}.");
    if end >= start {
        // Same day range. Eg: from 10 to 14
        now >= start && now < end
    } else {
        // Partitioned range. Eg: from 20 to 1
        now >= start || now < end
    }
}

/// Sets the given time to the specific hour setting all time subunits to 0
fn set_to_hour(time: DateTime<Local>, hour: u32) -> Result<DateTime<Local>, CachingResponseError> {
    let time = time.with_hour(hour).ok_or(CachingResponseError::Time)?;
    let time = time.with_minute(0).ok_or(CachingResponseError::Time)?;
    let time = time.with_second(0).ok_or(CachingResponseError::Time)?;
    time.with_nanosecond(0).ok_or(CachingResponseError::Time)
}

/// Given the current time, and the range when to cache, calculate until when the cached value should be valid
fn calculate_validity(
    now: DateTime<Local>,
    start: u32,
    end: u32,
) -> Result<DateTime<Local>, CachingResponseError> {
    // If we have a same day range or if we are already on the second day of a partitioned range
    if end >= start || now.hour() < start {
        set_to_hour(now, end)
    } else {
        // We are on the first day of a partitioned range
        let time = now
            .checked_add_days(Days::new(1))
            .ok_or(CachingResponseError::Time)?;
        set_to_hour(time, end)
    }
}

/// Defines custom request errors to handle them in an unified way
enum CachingRequestError {
    OutsideRange,
    CacheMiss(String),
    Deserialize(String, serde_json::Error),
}

/// Trys to read existing requests from the cache.
async fn try_from_cache(
    request_state: RequestState,
    config: &Config,
    cache: &impl Cache,
) -> Result<CachedResponse, CachingRequestError> {
    // Await for the headers
    let headers_state = request_state.into_headers_state().await;

    // Get the time in the current timezone.
    let now = Local::now();

    // Check if cache should be used
    if !check_time_in_range(now.hour(), config.start_hour as u32, config.end_hour as u32) {
        return Err(CachingRequestError::OutsideRange);
    }

    // Get the request path to use as caching key
    let path = headers_state.path();

    // Read the value from the cache
    let cached = cache
        .get(path.as_str())
        .ok_or_else(|| CachingRequestError::CacheMiss(path.clone()))?;

    // Deserialize the retrieved data
    let deserialized: CachedResponse = serde_json::from_slice(cached.as_slice())
        .map_err(|e| CachingRequestError::Deserialize(path.clone(), e))?;

    // Check the logical expiration of the cached value
    if deserialized.has_expired(&now) {
        return Err(CachingRequestError::CacheMiss(path));
    }

    Ok(deserialized)
}

/// Wraps the policy logic to unify the error handling
async fn request_filter(
    request_state: RequestState,
    config: &Config,
    cache: &impl Cache,
) -> Flow<CachingData> {
    match try_from_cache(request_state, config, cache).await {
        Ok(data) => {
            logger::debug!("Data retrieved from the cache.");
            Flow::Break(data.into())
        }
        Err(CachingRequestError::OutsideRange) => {
            logger::debug!("Outside caching hours. Request will proceed to the backend.");
            Flow::Continue(CachingData::IgnoreCache)
        }
        Err(CachingRequestError::CacheMiss(path)) => {
            logger::debug!("Cache Miss. Request will proceed to the backend.");
            Flow::Continue(CachingData::SaveResponse(path))
        }
        Err(CachingRequestError::Deserialize(path, error)) => {
            logger::warn!("Unexpected error deserializing the cached value. Request will proceed to the backend: {error}");
            cache.delete(path.as_str());
            Flow::Continue(CachingData::SaveResponse(path))
        }
    }
}

/// Define the custom response errors to handle them in an unified way
enum CachingResponseError {
    Serialization(serde_json::Error),
    Cache(CacheError),
    Time,
}

/// Try to save the response to the cache.
async fn save_to_cache(
    response_state: ResponseState,
    path: &str,
    config: &Config,
    cache: &impl Cache,
) -> Result<(), CachingResponseError> {
    // Awaits for the headers
    let headers_state = response_state.into_headers_state().await;
    let status_code = headers_state.status_code(); // Get the status code.
    let headers = headers_state.handler().headers(); // Get the headers.

    // Awaits for the body
    let body_state = headers_state.into_body_state().await;
    let body = body_state.handler().body(); // Get the body.

    // Calculates the time of logical expiration of the cached response.
    let valid_until = calculate_validity(
        Local::now(),
        config.start_hour as u32,
        config.end_hour as u32,
    )?;

    // Creates the object that we'll store in the cache.
    let response = CachedResponse {
        valid_until,
        status_code,
        headers,
        body,
    };

    // Serializes the object.
    let serialized = serde_json::to_vec(&response).map_err(CachingResponseError::Serialization)?;

    // Saves the serialized object
    cache
        .save(path, serialized)
        .map_err(CachingResponseError::Cache)?;

    Ok(())
}

/// Wraps the actual policy logic to unify the error handling.
async fn response_filter(
    response_state: ResponseState,
    RequestData(caching_data): RequestData<CachingData>,
    config: &Config,
    cache: &impl Cache,
) {
    // Check if we should save the response to the cache
    if let CachingData::SaveResponse(path) = caching_data {
        match save_to_cache(response_state, path.as_str(), config, cache).await {
            Ok(()) => {
                logger::debug!("Response successfully cached.")
            }
            Err(CachingResponseError::Serialization(error)) => {
                logger::warn!("Unexpected error serializing the response: {error}.")
            }
            Err(CachingResponseError::Cache(error)) => {
                logger::warn!("Unexpected saving the response to the cache: {error}.")
            }
            Err(CachingResponseError::Time) => {
                logger::warn!("Unexpected error calculating cache expiration time.")
            }
        }
    }
}

// Policy Configuration
#[entrypoint]
async fn configure(
    launcher: Launcher,
    Configuration(bytes): Configuration,
    cache_builder: CacheBuilder,
) -> Result<()> {
    // Deserialize the configuration
    let config: Config = serde_json::from_slice(&bytes).map_err(|err| {
        anyhow!(
            "Failed to parse configuration '{}'. Cause: {}",
            String::from_utf8_lossy(&bytes),
            err
        )
    })?;

    // Create the cache
    let cache = cache_builder
        .new("awesome-caching".to_string())
        .max_entries(config.max_cached_values as usize)
        .build();

    let filter = on_request(|request_state| request_filter(request_state, &config, &cache))
        .on_response(|response_state, request_data| {
            response_filter(response_state, request_data, &config, &cache)
        });

    launcher.launch(filter).await?;
    Ok(())
}

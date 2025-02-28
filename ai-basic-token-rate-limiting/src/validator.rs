// Copyright 2023 Salesforce, Inc. All rights reserved.
use anyhow::Result;
use pdk::cache::{Cache, CacheError};

use crate::{generated::config::Config, openai::Completion};

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use tiktoken_rs::{p50k_base, CoreBPE};

const WINDOW_INFO_CACHE_KEY: &str = "window_info";

/// Error raised during LLM rate limit validations.
#[derive(Debug, thiserror::Error)]
pub enum RateLimitError {
    #[error("Unable to initialize BPE: {0}")]
    P50kInitialization(anyhow::Error),

    #[error("{0:?}")]
    CacheStorage(CacheError),

    #[error("Cache serialization problem: {0}")]
    CacheSerialization(serde_json::Error),

    #[error("Body deserialization problem")]
    BodyDeserialization(serde_json::Error),

    #[error("Too many tokens. Rate Limit exceeded")]
    Exceeded,
}

#[derive(Deserialize, Serialize, Debug)]
struct CacheData {
    window_start: u128,
    window_tokens: usize,
}

/// Validates LLM token rate limits.
pub struct RateLimitValidator<C> {
    pub config: Config,
    pub cache: C,
    pub bpe: CoreBPE,
}

impl<C: Cache> RateLimitValidator<C> {
    fn get_cache(&self) -> Result<Option<CacheData>, RateLimitError> {
        self.cache
            .get(WINDOW_INFO_CACHE_KEY)
            .map(|bytes| serde_json::from_slice(&bytes).map_err(RateLimitError::CacheSerialization))
            .transpose()
    }

    fn save_cache(&self, value: CacheData) -> Result<(), RateLimitError> {
        let serialized = serde_json::to_vec(&value).map_err(RateLimitError::CacheSerialization)?;
        self.cache
            .save(WINDOW_INFO_CACHE_KEY, serialized)
            .map_err(RateLimitError::CacheStorage)
    }

    /// Creates a new [RateLimitValidator].
    pub fn new(config: Config, cache: C) -> Result<Self, RateLimitError> {
        Ok(Self {
            bpe: p50k_base().map_err(RateLimitError::P50kInitialization)?,
            config,
            cache,
        })
    }

    /// Applies a token validation to a payload.
    pub fn validate_payload(&self, payload: &[u8]) -> Result<(), RateLimitError> {
        // get request content
        let payload: Completion =
            serde_json::from_slice(payload).map_err(RateLimitError::BodyDeserialization)?;
        let messages = payload.messages;

        // count tokens with tiktoken
        let tokens: usize = messages
            .iter()
            .map(|m| self.bpe.encode_with_special_tokens(m.content).len())
            .sum();

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();

        // Get window start from cache
        if let Some(cache_data) = self.get_cache()? {
            let window_start = cache_data.window_start;
            let window_tokens = cache_data.window_tokens;

            // check if we're still in the window
            let window_length = self.config.time_period_in_milliseconds as u128;

            if now < window_start + window_length {
                // we are in the window
                let new_window_tokens = window_tokens + tokens;
                if new_window_tokens > self.config.maximum_tokens as usize {
                    return Err(RateLimitError::Exceeded);
                }
                self.save_cache(CacheData {
                    window_start,
                    window_tokens: new_window_tokens,
                })?;

                return Ok(());
            }
        }
        // save current time to window start
        self.save_cache(CacheData {
            window_start: now,
            window_tokens: tokens,
        })?;

        Ok(())
    }
}

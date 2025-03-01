// Copyright 2023 Salesforce, Inc. All rights reserved.
use anyhow::Result;
use pdk::cache::{Cache, CacheError};

use crate::openai::Completion;

use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};
use tiktoken_rs::{p50k_base, CoreBPE};

/// Key name for sharing rate limit window between filters.
const WINDOW_CACHE_KEY: &str = "token-rate-limit-window";

/// Error raised during LLM rate limit validations.
#[derive(Debug, thiserror::Error)]
pub enum RateLimitError {
    #[error("Unable to initialize BPE: {0}")]
    P50kInitialization(anyhow::Error),

    #[error("{0:?}")]
    CacheStorage(CacheError),

    #[error("Cache serialization problem: {0}")]
    CacheSerialization(serde_json::Error),

    #[error("Too many tokens. Rate Limit exceeded")]
    Exceeded,
}

#[derive(Deserialize, Serialize, Debug)]
struct Window {
    expiration: SystemTime,
    token_count: usize,
}

/// Validates LLM token rate limits.
pub struct RateLimitValidator<C> {
    maximum_tokens: usize,
    window_period: Duration,
    cache: C,
    bpe: CoreBPE,
}

impl<C: Cache> RateLimitValidator<C> {
    fn get_window(&self) -> Result<Option<Window>, RateLimitError> {
        self.cache
            .get(WINDOW_CACHE_KEY)
            .map(|bytes| serde_json::from_slice(&bytes).map_err(RateLimitError::CacheSerialization))
            .transpose()
    }

    fn save_window(&self, window: Window) -> Result<(), RateLimitError> {
        let serialized = serde_json::to_vec(&window).map_err(RateLimitError::CacheSerialization)?;
        self.cache
            .save(WINDOW_CACHE_KEY, serialized)
            .map_err(RateLimitError::CacheStorage)
    }

    /// Applies a token validation to a [Completion].
    pub fn validate(&self, completion: Completion<'_>) -> Result<(), RateLimitError> {
        let messages = completion.messages;

        // count tokens with tiktoken
        let tokens: usize = messages
            .iter()
            .map(|m| self.bpe.encode_with_special_tokens(m.content).len())
            .sum();

        let now = now();

        // Get window start from cache
        let mut window = self
            .get_window()?
            // Check if the existent window is not expired.
            .filter(|w| now < w.expiration)
            // Return a fresh window if the previous one was expired or it was not cached.
            .unwrap_or(Window {
                expiration: now + self.window_period,
                token_count: 0,
            });

        // Increase the token count
        window.token_count += tokens;

        let token_count = window.token_count;

        // Save the window with the updated values
        self.save_window(window)?;

        // If token count exceeds maxium, return an error
        if token_count > self.maximum_tokens {
            return Err(RateLimitError::Exceeded);
        }

        Ok(())
    }

    /// Creates a new [RateLimitValidator].
    pub fn new(
        window_period: Duration,
        maximum_tokens: usize,
        cache: C,
    ) -> Result<Self, RateLimitError> {
        Ok(Self {
            bpe: p50k_base().map_err(RateLimitError::P50kInitialization)?,
            window_period,
            maximum_tokens,
            cache,
        })
    }
}

#[cfg(not(test))]
fn now() -> SystemTime {
    SystemTime::now()
}

#[cfg(test)]
use tests::now;

#[cfg(test)]
mod tests {
    use std::{
        cell::{Cell, RefCell},
        collections::HashMap,
        time::{Duration, SystemTime},
    };

    use pdk::cache::Cache;

    use crate::openai::{Completion, Message};

    use super::{RateLimitError, RateLimitValidator};

    thread_local! {
        static NOW: Cell<SystemTime> = Cell::new(SystemTime::now());
    }

    pub fn now() -> SystemTime {
        NOW.get()
    }

    pub fn move_forward(duration: Duration) {
        NOW.set(NOW.get() + duration);
    }

    #[derive(Default)]
    struct CacheMock {
        values: RefCell<HashMap<String, Vec<u8>>>,
    }

    impl Cache for CacheMock {
        fn save(&self, key: &str, value: Vec<u8>) -> Result<(), pdk::cache::CacheError> {
            self.values.borrow_mut().insert(key.to_string(), value);
            Ok(())
        }

        fn get(&self, key: &str) -> Option<Vec<u8>> {
            self.values.borrow().get(key).cloned()
        }

        fn delete(&self, _key: &str) -> Option<Vec<u8>> {
            unimplemented!()
        }

        fn purge(&self) {
            unimplemented!()
        }
    }

    #[test]
    fn pass_at_first() {
        let cache = CacheMock::default();
        let period = Duration::from_millis(2000);

        let validator = RateLimitValidator::new(period, 5, cache).expect("validator created");

        let completion = Completion {
            model: "llama",
            messages: vec![Message {
                role: "user",
                content: "this has four tokens",
            }],
            extra: HashMap::default(),
        };

        let validation = validator.validate(completion);

        assert!(validation.is_ok());
    }

    #[test]
    fn reach_limit_at_first() {
        let cache = CacheMock::default();
        let period = Duration::from_millis(2000);

        let validator = RateLimitValidator::new(period, 5, cache).expect("validator created");

        let completion = Completion {
            model: "llama",
            messages: vec![Message {
                role: "user",
                content: "this has more than five tokens",
            }],
            extra: HashMap::default(),
        };

        let validation = validator
            .validate(completion)
            .expect_err("validation error");

        assert!(matches!(validation, RateLimitError::Exceeded));
    }

    #[test]
    fn reach_limit_at_third() {
        let cache = CacheMock::default();
        let period = Duration::from_millis(2000);

        let validator = RateLimitValidator::new(period, 8, cache).expect("validator created");

        let completion = Completion {
            model: "llama",
            messages: vec![Message {
                role: "user",
                content: "these are four tokens",
            }],
            extra: HashMap::default(),
        };

        let _ = validator.validate(completion.clone()).expect("pass 1");
        let _ = validator.validate(completion.clone()).expect("pass 2");
        let validation = validator
            .validate(completion)
            .expect_err("validation error");

        assert!(matches!(validation, RateLimitError::Exceeded));
    }

    #[test]
    fn clean_window() {
        let cache = CacheMock::default();
        let period = Duration::from_millis(2000);

        let validator = RateLimitValidator::new(period, 4, cache).expect("validator created");

        let completion = Completion {
            model: "llama",
            messages: vec![Message {
                role: "user",
                content: "these are four tokens",
            }],
            extra: HashMap::default(),
        };

        let success = validator.validate(completion.clone());

        assert!(success.is_ok());

        let fail = validator
            .validate(completion.clone())
            .expect_err("validation error");

        assert!(matches!(fail, RateLimitError::Exceeded));

        // move time forward to clean the window
        move_forward(period + Duration::from_millis(10));

        let validation = validator.validate(completion);

        assert!(validation.is_ok());
    }
}

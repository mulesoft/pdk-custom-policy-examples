// Copyright 2023 Salesforce, Inc. All rights reserved.
mod generated;

use anyhow::{anyhow, Result};
use pdk::hl::timer::{Clock, Timer};
use pdk::hl::*;
use pdk::logger;
use std::cell::RefCell;
use std::ops::Add;
use std::time::Duration;
use std::time::SystemTime;

use crate::generated::config::Config;

/// This struct keeps in memory the timestamps of the executed requests.
struct State<'a> {
    // Each worker is single threaded so no need for locking mechanism, as long as the mutable
    // reference is released before the next 'await' directive.
    requests: RefCell<Vec<SystemTime>>,
    config: &'a Config,
}

impl<'a> State<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self {
            requests: RefCell::new(vec![]),
            config,
        }
    }

    /// Check if the request is allowed to reach the backend.
    pub fn allowed(&self, timer: &Timer) -> bool {
        let now = timer.now();
        let mut reqs = self.requests.borrow_mut();

        // Discards requests that have fallen out of the sliding window.
        while reqs.first().map(|first| first.lt(&now)).unwrap_or(false) {
            let _ = reqs.pop();
        }

        // If we haven't reached the maximum of requests we store the timestamp and we'll let the request through.
        if reqs.len() < self.config.requests as usize {
            let exp = now.add(Duration::from_millis(self.config.millis as u64));
            reqs.push(exp);
            true
        } else {
            false
        }
    }
}

/// Wrap the sleep function to log how many millis were actually slept.
async fn logged_sleep(timer: &Timer, duration: Duration) -> bool {
    let init = timer.now();
    let slept = timer.sleep(duration).await;
    let end = timer.now();
    logger::debug!(
        "Slept for {} millis.",
        end.duration_since(init).unwrap().as_millis()
    );
    slept
}

/// Function that will handle each request
async fn request_filter(timer: &Timer, state: &State<'_>, config: &Config) -> Flow<()> {
    let mut retries = 0;
    // We check if the request is allowed.
    while !state.allowed(timer) {
        if retries + 1 > config.max_attempts // Check if the maximum amount of retries was reached
            || !logged_sleep(timer, Duration::from_millis(config.delay as u64)).await
        // Wait for the specified time
        {
            logger::debug!("Retries: {retries}");
            return Flow::Break(Response::new(429));
        }
        retries += 1;
    }
    logger::debug!("Retries: {retries}");
    Flow::Continue(())
}

#[entrypoint]
async fn configure(
    launcher: Launcher,
    Configuration(bytes): Configuration,
    clock: Clock, // Inject the clock to be able to launch async tasks.
) -> Result<()> {
    let config: Config = serde_json::from_slice(&bytes).map_err(|err| {
        anyhow!(
            "Failed to parse configuration '{}'. Cause: {}",
            String::from_utf8_lossy(&bytes),
            err
        )
    })?;

    let state = State::new(&config);
    let timer = clock.period(Duration::from_millis(100));
    launcher
        .launch(on_request(|| request_filter(&timer, &state, &config)))
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use pdk_unit::{UnitHttpRequest, UnitTestBuilder};
    use serde_json::json;
    use std::task::Poll;
    use std::time::Duration;

    #[test]
    fn request_within_limit_passes_through() {
        let mut tester = UnitTestBuilder::default()
            .with_config(
                json!({
                    "requests": 5,
                    "millis": 1000,
                    "delay": 100,
                    "maxAttempts": 3
                })
                .to_string(),
            )
            .with_entrypoint(crate::configure);

        let response = tester.request(UnitHttpRequest::get());

        assert_eq!(response.status_code(), 200);
    }

    #[test]
    fn requests_exceeding_limit_return_429() {
        let mut tester = UnitTestBuilder::default()
            .with_config(
                json!({
                    "requests": 1,
                    "millis": 60000,
                    "delay": 0,
                    "maxAttempts": 0
                })
                .to_string(),
            )
            .with_entrypoint(crate::configure);

        tester.request(UnitHttpRequest::get());
        let response = tester.request(UnitHttpRequest::get());

        assert_eq!(response.status_code(), 429);

        tester.sleep(Duration::from_millis(60001));
        let response = tester.request(UnitHttpRequest::get());
        assert_eq!(response.status_code(), 200);
    }

    #[test]
    fn requests_exceeding_limit_re_attempts_after_delay() {
        let mut tester = UnitTestBuilder::default()
            .with_config(
                json!({
                    "requests": 1,
                    "millis": 59999,
                    "delay": 20000,
                    "maxAttempts": 3
                })
                .to_string(),
            )
            .with_entrypoint(crate::configure);

        tester.request(UnitHttpRequest::get());
        let mut req = tester.request(UnitHttpRequest::get());
        assert!(!req.poll().is_ready());

        tester.sleep(Duration::from_millis(20000));
        assert!(!req.poll().is_ready());

        tester.sleep(Duration::from_millis(20000));
        assert!(!req.poll().is_ready());

        tester.sleep(Duration::from_millis(20000));
        let Poll::Ready(resp) = req.poll() else {
            panic!("Request should not be polled after 3 attempts");
        };
        assert_eq!(resp.status_code(), 200);
    }

    #[test]
    fn requests_exceeding_limit_re_attempts() {
        let mut tester = UnitTestBuilder::default()
            .with_config(
                json!({
                    "requests": 1,
                    "millis": 59999,
                    "delay": 20000,
                    "maxAttempts": 3
                })
                .to_string(),
            )
            .with_entrypoint(crate::configure);

        tester.request(UnitHttpRequest::get());
        let response = tester.request(UnitHttpRequest::get());
        assert_eq!(response.status_code(), 200);
    }

    #[test]
    fn requests_exceeding_limit_re_attempts_429() {
        let mut tester = UnitTestBuilder::default()
            .with_config(
                json!({
                    "requests": 1,
                    "millis": 59999,
                    "delay": 20000,
                    "maxAttempts": 1
                })
                .to_string(),
            )
            .with_entrypoint(crate::configure);

        tester.request(UnitHttpRequest::get());
        let response = tester.request(UnitHttpRequest::get());
        assert_eq!(response.status_code(), 429);
    }
}

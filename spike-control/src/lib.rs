// Copyright 2023 Salesforce, Inc. All rights reserved.

mod generated;

use anyhow::{anyhow, Result};
use pdk::hl::timer::Clock;
use pdk::hl::*;
use pdk::metadata::Tier;
use pdk::spike_control::{SpikeControlBuilder, SpikeControlError, SpikeControlHandler};
use std::rc::Rc;
use std::time::Duration;

use crate::generated::config::Config;

const BUCKET_ID: &str = "default";
const ROUTE_KEY: &str = "spike";

async fn request_filter(
    state: RequestState,
    handler: &SpikeControlHandler,
    config: &Config,
) -> Flow<()> {
    let _ = state.into_headers_state().await;

    match handler
        .is_allowed(BUCKET_ID, ROUTE_KEY, 1, config.max_attempts > 0)
        .await
    {
        Ok(_) => Flow::Continue(()),
        Err(SpikeControlError::TooManyRequests(_)) => Flow::Break(Response::new(429)),
        Err(_) => Flow::Break(Response::new(503)),
    }
}

#[entrypoint]
async fn configure(
    launcher: Launcher,
    Configuration(bytes): Configuration,
    clock: Clock,
    spike_control: SpikeControlBuilder,
) -> Result<()> {
    let config: Config = serde_json::from_slice(&bytes).map_err(|err| {
        anyhow!(
            "Failed to parse configuration '{}'. Cause: {}",
            String::from_utf8_lossy(&bytes),
            err
        )
    })?;

    let delay_ms = config.delay.max(0) as u64;
    let max_retries = config.max_attempts.max(0) as u32;

    let mut builder = spike_control
        .new("spike-control-example".to_string())
        .with_bucket(
            BUCKET_ID.to_string(),
            vec![Tier {
                requests: config.requests.max(0) as u64,
                period_in_millis: config.millis.max(0) as u64,
            }],
        );

    if max_retries > 0 {
        builder = builder
            .with_ticker(Rc::new(clock.period(Duration::from_millis(100))))
            .with_retry(delay_ms, max_retries);
    }

    let handler = builder
        .build()
        .map_err(|e| anyhow!("failed to build spike control: {e}"))?;

    let filter = on_request(|state| request_filter(state, &handler, &config));
    launcher.launch(filter).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use pdk_unit::{TraceBackend, UnitHttpRequest, UnitHttpResponse, UnitTestBuilder};
    use serde_json::json;
    use std::rc::Rc;
    use std::time::Duration;

    fn tight_limit_config() -> String {
        json!({
            "requests": 1,
            "millis": 60000,
            "delay": 0,
            "maxAttempts": 0
        })
        .to_string()
    }

    #[test]
    fn first_request_reaches_backend() {
        let backend = Rc::new(TraceBackend::new(UnitHttpResponse::new(200)));
        let mut tester = UnitTestBuilder::default()
            .with_config(tight_limit_config())
            .with_backend(Rc::clone(&backend))
            .with_entrypoint(crate::configure);

        assert_eq!(tester.request(UnitHttpRequest::get()).status_code(), 200);
        assert!(backend.next().is_some());
    }

    #[test]
    fn second_immediate_request_returns_429() {
        let backend = Rc::new(TraceBackend::new(UnitHttpResponse::new(200)));
        let mut tester = UnitTestBuilder::default()
            .with_config(tight_limit_config())
            .with_backend(Rc::clone(&backend))
            .with_entrypoint(crate::configure);

        assert_eq!(tester.request(UnitHttpRequest::get()).status_code(), 200);
        assert!(backend.next().is_some());
        assert_eq!(tester.request(UnitHttpRequest::get()).status_code(), 429);
        assert!(backend.next().is_none());
    }

    #[test]
    fn limit_resets_after_window() {
        let backend = Rc::new(TraceBackend::new(UnitHttpResponse::new(200)));
        let mut tester = UnitTestBuilder::default()
            .with_config(tight_limit_config())
            .with_backend(Rc::clone(&backend))
            .with_entrypoint(crate::configure);

        assert_eq!(tester.request(UnitHttpRequest::get()).status_code(), 200);
        let _ = backend.next();
        assert_eq!(tester.request(UnitHttpRequest::get()).status_code(), 429);

        tester.sleep(Duration::from_millis(60_001));
        assert_eq!(tester.request(UnitHttpRequest::get()).status_code(), 200);
    }

    #[test]
    fn retry_succeeds_after_window_expires() {
        let backend = Rc::new(TraceBackend::new(UnitHttpResponse::new(200)));
        let mut tester = UnitTestBuilder::default()
            .with_config(
                json!({
                    "requests": 1,
                    "millis": 1000,
                    "delay": 500,
                    "maxAttempts": 3
                })
                .to_string(),
            )
            .with_backend(Rc::clone(&backend))
            .with_entrypoint(crate::configure);

        assert_eq!(tester.request(UnitHttpRequest::get()).status_code(), 200);
        assert!(backend.next().is_some());

        let response = tester.request(UnitHttpRequest::get());
        assert_eq!(response.status_code(), 200);
        assert!(backend.next().is_some());
    }
}

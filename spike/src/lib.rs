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

struct State<'a> {
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

    pub fn allowed(&self) -> bool {
        let now = SystemTime::now();
        let mut reqs = self.requests.borrow_mut();

        while reqs.first().map(|first| first.lt(&now)).unwrap_or(false) {
            let _ = reqs.pop();
        }

        if reqs.len() < self.config.requests as usize {
            let exp = now.add(Duration::from_millis(self.config.millis as u64));
            reqs.push(exp);
            true
        } else {
            false
        }
    }
}

async fn logged_sleep(timer: &Timer, duration: Duration) -> bool {
    let init = SystemTime::now();
    let slept = timer.sleep(duration).await;
    let end = SystemTime::now();
    logger::debug!(
       "Slept for {} millis.",
       end.duration_since(init).unwrap().as_millis()
   );
    slept
}

async fn request_filter(timer: &Timer, state: &State<'_>, config: &Config) -> Flow<()> {
    let mut retries = 0;
    while !state.allowed() {
        if retries + 1 > config.max_attempts
            || !logged_sleep(timer, Duration::from_millis(config.delay as u64)).await
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
    clock: Clock,
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

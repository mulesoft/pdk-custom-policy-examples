// Copyright 2023 Salesforce, Inc. All rights reserved.
mod generated;

use anyhow::{anyhow, Result};
use futures::join;
use ipnet::Ipv4Net;
use iprange::IpRange;
use pdk::cache::{Cache, CacheBuilder};
use std::cell::RefCell;
use std::net::Ipv4Addr;
use std::time::{Duration, SystemTime};

use pdk::hl::timer::{Clock, Timer};
use pdk::hl::*;
use pdk::lock::{LockBuilder, TryLock};
use pdk::logger;
use pdk::script::{HandlerAttributesBinding, PayloadBinding, Value};

use crate::generated::config::Config;

/// Identifier for the cache and the lock to share data between workers.
const ID: &str = "block";

/// Key for cache entry that keeps the time of the last update to the stored data.
const LAST_UPDATE: &str = "lastUpdate";
/// Key for cache entry that keeps data to be shared between workers.
const DATA_KEY: &str = "data";

/// This struct keeps in memory the ips to be blocked to avoid deserializing the data from the cache
/// on each request.
#[derive(Default)]
struct BlockedIPs {
    // Each worker is single threaded so no need for locking mechanism, as long as the mutable
    // reference is released before the next 'await' directive.
    update: RefCell<Option<SystemTime>>,
    ip_range: RefCell<IpRange<Ipv4Net>>,
}

impl BlockedIPs {
    /// Update the ip ranges to be blocked
    pub fn update(&self, update_time: SystemTime, ips: &str) {
        let ip_range: IpRange<Ipv4Net> = ips
            .split("\n")
            .into_iter()
            .filter_map(|s| s.parse().ok())
            .collect();

        self.ip_range.replace(ip_range);
        self.update.replace(Some(update_time));
    }

    /// Inquires if the specified ip is in one of the forbidden ranges.
    pub fn allowed(&self, ip: &str) -> bool {
        self.update.borrow().is_some()
            && ip
                .parse::<Ipv4Addr>()
                .ok()
                .map(|ip| !self.ip_range.borrow().contains(&ip))
                .unwrap_or_default()
    }

    /// Get the timestamp of the last update of data.
    pub fn last_update(&self) -> Option<SystemTime> {
        self.update.borrow().clone()
    }
}

/// Get the last update value from the cache.
fn last_update(cache: &impl Cache) -> Option<SystemTime> {
    cache
        .get(LAST_UPDATE)
        .and_then(|data| serde_json::from_slice::<SystemTime>(data.as_slice()).ok())
}

/// Queries the service providing the range of ips to be blocked if necessary.
async fn fetch_blocked_ips(
    config: &Config,
    client: &HttpClient,
    cache: &impl Cache,
    lock: &TryLock,
) -> Result<()> {
    let now = SystemTime::now();

    // Update only if it has passed enough time since the last update, some services have
    // usage limits.
    if !last_update(cache)
        .map(|val| now.gt(&(val + Duration::from_secs(config.frequency as u64))))
        .unwrap_or(true)
    {
        return Ok(()); // No update necessary.
    }

    // Acquire the lock to ensure only one worker is hitting the backend at a time
    if let Some(acquired) = lock.try_lock() {
        let response = client
            .request(&config.source)
            .timeout(Duration::from_secs(10))
            .get()
            .await?;

        // If we lost the lock we discard the response.
        if !acquired.refresh_lock() {
            return Err(anyhow!("Lost the lock!"));
        }

        // If the request was successful we share the result through the cache.
        if response.status_code() == 200 {
            cache.save(LAST_UPDATE, serde_json::to_vec(&now)?)?;
            cache.save(DATA_KEY, response.body().as_bytes())?;
        } else {
            return Err(anyhow!(
                "{} - {}",
                response.status_code(),
                String::from_utf8_lossy(response.body())
            ));
        }
    }
    Ok(())
}

/// Reads the ips to be blocked from the cache to the worker memory if there is any update.
fn load_ips_from_cache(cache: &impl Cache, blocked_ips: &BlockedIPs) {
    // If there is data available.
    if let Some(update) = last_update(cache) {
        // If the data was updated.
        if blocked_ips
            .last_update()
            .map(|last| last.ne(&update))
            .unwrap_or(true)
        {
            // Retrieve the data from the cache.
            if let Some(data) = cache.get(DATA_KEY) {
                // Update our struct that holds the business logic.
                blocked_ips.update(update, String::from_utf8_lossy(data.as_slice()).as_ref())
            }
        }
    }
}

/// This function executed the periodic checks to see if new information should be feched.
async fn fetch_loop(
    config: &Config,
    client: &HttpClient,
    timer: &Timer,
    cache: &impl Cache,
    lock: &TryLock,
    blocked_ips: &BlockedIPs,
) {
    while timer.next_tick().await {
        // Fetch the ip data from the server and share it with the other workers through the cache.
        if let Err(err) = fetch_blocked_ips(config, client, cache, lock).await {
            logger::warn!("Unexpected error while fetching the ips: {err}.");
        }
        // Load the ip ranges from the cache to the worker memory.
        load_ips_from_cache(cache, blocked_ips);
    }
}

async fn request_filter(
    state: RequestState,
    config: &Config,
    properties: StreamProperties,
    blocked_ips: &BlockedIPs,
) -> Flow<()> {
    let state = state.into_headers_state().await;
    let mut eval = config.ip.evaluator();
    eval.bind_attributes(&HandlerAttributesBinding::new(state.handler(), &properties));

    if let Ok(Value::String(val)) = eval.eval() {
        if blocked_ips.allowed(val.as_str()) {
            return Flow::Continue(());
        }
    }
    Flow::Break(Response::new(403))
}

#[entrypoint]
async fn configure(
    launcher: Launcher,
    Configuration(bytes): Configuration,
    client: HttpClient,
    cache: CacheBuilder, // Inject the cache to be able to share data between the workers.
    clock: Clock,        // Inject the clock to be able to launch async tasks.
    lock: LockBuilder,   // Inject the lock to be able to synchronise the workers.
) -> Result<()> {
    let config: Config = serde_json::from_slice(&bytes).map_err(|err| {
        anyhow!(
            "Failed to parse configuration '{}'. Cause: {}",
            String::from_utf8_lossy(&bytes),
            err
        )
    })?;

    // The time is configured with a short frequency, this time will be granularity of the task
    // execution. In this scenario it represents the amount of time in which the new IP ranges are
    // propagated between workers. An additional validation is done to ensure the service is not
    // flooded.
    let timer = clock.period(Duration::from_secs(1));

    // Cache to share the ip data between workers.
    let cache = cache.new(ID.to_string()).build();

    // Configure the lock to expire with a value bigger that all possible timeouts in the async task,
    // this way, if some worker stops responding, the other will be able to recover the lock and
    // continue working as expected.
    let lock = lock
        .new(ID.to_string())
        .expiration(Duration::from_secs(20))
        .build();

    let blocked_ips = BlockedIPs::default();

    // Create the future tasks.
    // Note: We don't do individual 'await's here because we want both task to progress their execution.

    // Future that will fetch the ip ranges periodically
    let fetch = fetch_loop(&config, &client, &timer, &cache, &lock, &blocked_ips);

    // Future that will handle the requests
    let launched = launcher.launch(on_request(|rs, st| {
        request_filter(rs, &config, st, &blocked_ips)
    }));

    // Await for both futures to finish
    // Note: Proxy-Wasm Guarantees that they won't be executed in a parallel fashion. Only one tas will
    // progress at a time, interleaving only at points where functions are 'await'ed.
    let joined = join!(launched, fetch);
    // Propagate the error of the launcher
    joined.0?;
    Ok(())
}

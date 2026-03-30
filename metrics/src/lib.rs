// Copyright 2023 Salesforce, Inc. All rights reserved.
mod generated;

use anyhow::{anyhow, Result};
use futures::join;
use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::Hash;
use std::ops::{Deref, DerefMut};
use std::time::Duration;

use pdk::hl::timer::{Clock, Timer};
use pdk::hl::*;
use pdk::logger;
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};

use crate::generated::config::Config;

/// This struct will collect the data of the incoming requests and serializes them to send to the metrics service.
/// It uses interior mutability pattern to hide the complexity of metrics collection and consumption across different scopes.
struct Metrics {
    node: String,
    // Each worker is single threaded so no need for locking mechanism, as long as the mutable
    // reference is released before the next 'await' directive.
    data: RefCell<HashMap<(String, u32), u64>>,
}

impl Metrics {
    /// Create a new metrics collector with an id.
    pub fn new(node: String) -> Self {
        Self {
            node,
            data: RefCell::new(HashMap::new()),
        }
    }

    /// Indicates that a new request was made with the specific method and status code response.
    pub fn track(&self, method: String, status: u32) {
        Self::increment(self.data.borrow_mut().deref_mut(), (method, status), 1);
    }

    /// returns true if there is at information of at least 1 tracked request.
    pub fn is_empty(&self) -> bool {
        self.data.borrow().is_empty()
    }

    /// Clears the data collected to start collecting new one.
    pub fn clear(&self) {
        self.data.replace(HashMap::new());
    }

    /// A function that given a map, a key and a number will increment the value stored in it.
    fn increment<Key>(map: &mut HashMap<Key, u64>, key: Key, count: u64)
    where
        Key: Eq,
        Key: Hash,
    {
        let entry = map.entry(key).or_insert_with(|| 0);
        *entry = *entry + count;
    }
}

/// Implement serialize for the collected metrics.
/// Here we process the data to adapt to the metrics server.
impl Serialize for Metrics {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let data = self.data.borrow();
        let mut method_count: HashMap<&str, u64> = HashMap::new();
        let mut status_count: HashMap<u32, u64> = HashMap::new();
        for ((method, status), count) in data.deref() {
            Self::increment(&mut method_count, method, *count);
            Self::increment(&mut status_count, *status, *count);
        }

        let mut s = serializer.serialize_struct("Metrics", 3)?;
        s.serialize_field("node", &self.node)?;
        s.serialize_field("methods", &method_count)?;
        s.serialize_field("status_codes", &status_count)?;

        s.end()
    }
}

/// Function that will send the provided serialized body to the metrics server. Returns true if
/// the request was successful or false otherwise.
async fn publish_metrics(client: &HttpClient, config: &Config, body: &str) -> bool {
    let response = client
        .request(&config.metrics_sink)
        .timeout(Duration::from_secs(10))
        .body(body.as_bytes())
        .headers(vec![("content-type", "application/json")])
        .post()
        .await;

    match response {
        Ok(resp) => {
            if [200, 202, 204].contains(&resp.status_code()) {
                logger::debug!("Metrics posted successfully! {}", body);
                true
            } else {
                logger::warn!(
                    "Unexpected response from metrics service: {} - {}",
                    resp.status_code(),
                    String::from_utf8_lossy(resp.body())
                );
                false
            }
        }
        Err(err) => {
            logger::warn!("Unexpected error sending metrics to the server: {}.", err);
            false
        }
    }
}

/// Function that will periodically publish collected metrics to the server.
async fn publish_loop(timer: &Timer, client: &HttpClient, config: &Config, metrics: &Metrics) {
    // While the policy is still running.
    // Wait for the next cycle.
    while timer.next_tick().await {
        // If there are no metrics to send skip the cycle.
        if metrics.is_empty() {
            continue;
        }

        // Serialize the metrics to json format.
        let body = serde_json::to_string(metrics).unwrap_or_else(|_| "{}".to_string());

        // Clear the collected metrics to remove the ones that we are sending.
        metrics.clear();

        let mut retry = 0; // Counter to keep track of retries.
        while !publish_metrics(client, config, body.as_str()).await // If the metrics were not published
            && config.max_retries.map(|max| retry <= max).unwrap_or(true)
        // and we haven't reached the maximum amount of retries
        {
            retry += 1;
            // We do an increasing backoff before retrying.
            if !timer
                .sleep(Duration::from_secs((config.push_frequency * retry) as u64))
                .await
            {
                // If the sleep method failed it means that no more ticks will arrive and the policy is stopping.
                break;
            }
        }
    }
}

/// Function that will handle the request part of the requests.
async fn request_filter(state: RequestState) -> Flow<String> {
    let state = state.into_headers_state().await;
    // Collect data from the request and forward the data to the response filter.
    Flow::Continue(state.method().to_lowercase())
}

/// Function that will handle the response part of the requests.
async fn response_filter(state: ResponseState, data: RequestData<String>, metrics: &Metrics) {
    if let RequestData::Continue(method) = data {
        let state = state.into_headers_state().await;
        // Collect data from the response and track along with the request data.
        metrics.track(method, state.status_code());
    }
}

#[entrypoint]
async fn configure(
    launcher: Launcher,
    Configuration(bytes): Configuration,
    clock: Clock, // Inject the clock to be able to launch async tasks.
    client: HttpClient,
) -> Result<()> {
    let config: Config = serde_json::from_slice(&bytes).map_err(|err| {
        anyhow!(
            "Failed to parse configuration '{}'. Cause: {}",
            String::from_utf8_lossy(&bytes),
            err
        )
    })?;

    // set the period between ticks of the timer.
    let timer = clock.period(Duration::from_secs(config.push_frequency as u64));

    // Create the object that will handle the business logic of the policy.
    let metrics = Metrics::new(uuid::Uuid::new_v4().to_string());

    // Create the future tasks.
    // Note: We don't do individual 'await's here because we want both task to progress their execution.

    // Future that will publish the metrics periodically
    let publish = publish_loop(&timer, &client, &config, &metrics);

    // Future that will handle the requests
    let launched = launcher
        .launch(on_request(request_filter).on_response(|rs, rd| response_filter(rs, rd, &metrics)));

    // Await for both futures to finish
    // Note: Proxy-Wasm Guarantees that they won't be executed in a parallel fashion. Only one tas will
    // progress at a time, interleaving only at points where functions are 'await'ed.
    let joined = join!(launched, publish);
    // Propagate the error of the launcher
    joined.0?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use pdk_unit::{
        Backend, TraceBackend, UnitHttpMessage, UnitHttpRequest, UnitHttpResponse, UnitTestBuilder,
    };
    use serde_json::json;
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::time::Duration;

    pub struct MetricsBackend {
        count: RefCell<u32>,
        reject_until: u32,
    }

    impl MetricsBackend {
        pub fn new(reject_until: u32) -> Self {
            Self {
                count: RefCell::new(0),
                reject_until,
            }
        }
    }

    impl Backend for MetricsBackend {
        fn call(&self, _req: UnitHttpRequest) -> UnitHttpResponse {
            let count = *self.count.borrow();
            self.count.replace(count + 1);
            if count < self.reject_until {
                UnitHttpResponse::new(503)
            } else {
                UnitHttpResponse::new(202)
            }
        }
    }

    fn config() -> String {
        json!({
            "metricsSink": "http://metrics-sink",
            "pushFrequency": 60,
            "maxRetries": 3
        })
        .to_string()
    }

    #[test]
    fn metrics_sent_to_backend() {
        let metric_server = Rc::new(TraceBackend::new(MetricsBackend::new(0)));

        let mut tester = UnitTestBuilder::default()
            .with_config(config())
            .with_http_upstream_from_authority("metrics-sink", Rc::clone(&metric_server))
            .with_entrypoint(crate::configure);

        let response = tester.request_full(UnitHttpRequest::get());
        assert_eq!(response.status_code(), 200);

        assert!(metric_server.next().is_none());

        tester.sleep(Duration::from_secs(120));

        let sent = metric_server.next().unwrap();
        assert!(String::from_utf8_lossy(sent.body())
            .contains("\"methods\":{\"get\":1},\"status_codes\":{\"200\":1}"));
        assert!(metric_server.next().is_none());
    }

    #[test]
    fn metric_sent_retries() {
        let metric_server = Rc::new(TraceBackend::new(MetricsBackend::new(1)));

        let mut tester = UnitTestBuilder::default()
            .with_config(config())
            .with_http_upstream_from_authority("metrics-sink", Rc::clone(&metric_server))
            .with_entrypoint(crate::configure);

        let response = tester.request_full(UnitHttpRequest::get());
        assert_eq!(response.status_code(), 200);

        assert!(metric_server.next().is_none());

        tester.sleep(Duration::from_secs(120));

        let sent = metric_server.next().unwrap();
        assert!(String::from_utf8_lossy(sent.body())
            .contains("\"methods\":{\"get\":1},\"status_codes\":{\"200\":1}"));
        assert!(String::from_utf8_lossy(sent.body())
            .contains("\"methods\":{\"get\":1},\"status_codes\":{\"200\":1}"));
    }
}
